use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Error, Field, Fields, Lit,
    Meta, NestedMeta, Path,
};

fn process_submeta<F: FnMut(&Meta)>(attrs: &Vec<Attribute>, f: &mut F) {
    for attr in attrs {
        let list = if let Ok(Meta::List(list)) = attr.parse_meta() {
            list
        } else {
            continue;
        };
        let ident = if let Some(ident) = list.path.get_ident() {
            ident
        } else {
            continue;
        };
        if ident != "err_per_field" {
            continue;
        }
        for nested_meta in &list.nested {
            let submeta = if let NestedMeta::Meta(submeta) = nested_meta {
                submeta
            } else {
                continue;
            };
            f(submeta);
        }
    }
}

enum FieldWrapper {
    Result(Path),
    Option,
    Raw,
}

#[proc_macro_derive(ErrPerField, attributes(err_per_field))]
pub fn err_per_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut compiler_errors = vec![];

    let DeriveInput {
        vis: container_vis,
        ident: container_ident,
        generics: container_generics,
        data: container_data,
        ..
    } = &input;

    // Build wrapper struct name
    let wrapper_struct_ident = Ident::new(
        &format!("{}PerErrFieldWrapper", container_ident),
        Span::call_site(),
    );

    // Build wrapper struct generics
    let (impl_generics, ty_generics, where_clause) = container_generics.split_for_impl();

    // Iterate struct body
    let struct_data = if let Data::Struct(struct_data) = container_data {
        struct_data
    } else {
        return TokenStream::from(
            Error::new(input.span(), "Unsupported data type").to_compile_error(),
        );
    };
    let fields = if let Fields::Named(fields) = &struct_data.fields {
        fields
    } else {
        return TokenStream::from(
            Error::new(input.span(), "Unsupported data type").to_compile_error(),
        );
    };
    let mut wrapper_body = vec![];
    let mut inner_body = vec![];
    let mut matched_field_patterns = vec![];
    for field in &fields.named {
        let Field {
            attrs: field_attrs,
            vis: field_vis,
            ident: field_name,
            ty: field_ty,
            ..
        } = &field;
        inner_body.push(field_name);
        let mut field_wrapper = FieldWrapper::Raw;
        process_submeta(field_attrs, &mut |submeta| match submeta {
            Meta::Path(meta_path) => {
                let ident = if let Some(ident) = meta_path.get_ident() {
                    ident
                } else {
                    compiler_errors
                        .push(Error::new(meta_path.span(), "Unknown key").to_compile_error());
                    return;
                };
                if ident != "maybe_none" {
                    compiler_errors.push(
                        Error::new(ident.span(), &format!("Unknown key {}", ident))
                            .to_compile_error(),
                    );
                    return;
                }
                field_wrapper = FieldWrapper::Option;
            }
            Meta::NameValue(meta_name_value) => {
                let ident = if let Some(ident) = meta_name_value.path.get_ident() {
                    ident
                } else {
                    return;
                };
                if ident != "maybe_error" {
                    compiler_errors.push(
                        Error::new(ident.span(), &format!("Unknown key {}", ident))
                            .to_compile_error(),
                    );
                    return;
                }
                let str_lit = if let Lit::Str(str_lit) = &meta_name_value.lit {
                    str_lit
                } else {
                    compiler_errors.push(
                        Error::new(meta_name_value.lit.span(), "Unsupported value type")
                            .to_compile_error(),
                    );
                    return;
                };
                let path = if let Ok(path) = syn::parse_str::<Path>(&str_lit.value()) {
                    path
                } else {
                    compiler_errors.push(
                        Error::new(
                            meta_name_value.lit.span(),
                            &format!("Unable to convert {} to a valid path.", str_lit.value()),
                        )
                        .to_compile_error(),
                    );
                    return;
                };
                field_wrapper = FieldWrapper::Result(path);
            }
            _ => {
                return;
            }
        });
        let wrapper_body_part = match &field_wrapper {
            FieldWrapper::Raw => {
                quote! { #field_vis #field_name: #field_ty }
            }
            FieldWrapper::Option => {
                quote! { #field_vis #field_name: ::core::option::Option<#field_ty> }
            }
            FieldWrapper::Result(path) => {
                quote! { #field_vis #field_name: ::core::result::Result<#field_ty, #path> }
            }
        };
        let matched_field_pattern = match &field_wrapper {
            FieldWrapper::Raw => {
                quote! { #field_name }
            }
            FieldWrapper::Option => {
                quote! { #field_name: Some(#field_name) }
            }
            FieldWrapper::Result(_) => {
                quote! { #field_name: Ok(#field_name) }
            }
        };
        wrapper_body.push(wrapper_body_part);
        matched_field_patterns.push(matched_field_pattern);
    }

    let expanded = quote! {
        #(#compiler_errors)*
        #container_vis struct #wrapper_struct_ident #container_generics {
            #(#wrapper_body),*
        }

        impl #impl_generics err_per_field::FieldMayErr for #container_ident #ty_generics #where_clause {
            type Wrapper = #wrapper_struct_ident #ty_generics;
        }

        impl #impl_generics ::core::convert::TryFrom<#wrapper_struct_ident #ty_generics> for #container_ident #ty_generics #where_clause {
            type Error = #wrapper_struct_ident #ty_generics;
            fn try_from(value: #wrapper_struct_ident #ty_generics) -> Result<Self, Self::Error> {
                if let #wrapper_struct_ident #ty_generics {
                    #(#matched_field_patterns),*
                } = value {
                    Ok(#container_ident {
                        #(#inner_body),*
                    })
                } else {
                    Err(value)
                }
            }
        }
    };

    TokenStream::from(expanded)
}
