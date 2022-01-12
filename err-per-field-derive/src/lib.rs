use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Error, Fields, Lit, Meta,
    MetaNameValue, NestedMeta, Path,
};

fn process_meta_name_value<F: FnMut(&MetaNameValue)>(attrs: &Vec<Attribute>, f: &mut F) {
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
            let meta_name_value = if let Meta::NameValue(meta_name_value) = submeta {
                meta_name_value
            } else {
                continue;
            };
            f(meta_name_value);
        }
    }
}

#[proc_macro_derive(ErrPerField, attributes(err_per_field))]
pub fn err_per_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut compiler_errors = vec![];

    let DeriveInput {
        attrs: container_attrs,
        vis: container_vis,
        ident: container_ident,
        generics: container_generics,
        data: container_data,
    } = &input;

    // Build wrapper struct name
    let wrapper_struct_ident = Ident::new(
        &format!("{}PerErrFieldWrapper", container_ident),
        Span::call_site(),
    );

    // Build wrapper struct generics
    let (impl_generics, ty_generics, where_clause) = container_generics.split_for_impl();

    // Get top-level wrapper
    let mut error_path = None;
    process_meta_name_value(container_attrs, &mut |meta_name_value| {
        let ident = if let Some(ident) = meta_name_value.path.get_ident() {
            ident
        } else {
            return;
        };
        if ident != "error" {
            compiler_errors.push(
                Error::new(ident.span(), &format!("Unknown key {}", ident)).to_compile_error(),
            );
            return;
        }
        let str_lit = if let Lit::Str(str_lit) = &meta_name_value.lit {
            str_lit
        } else {
            compiler_errors.push(
                Error::new(meta_name_value.lit.span(), "Unsupported value type").to_compile_error(),
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
        error_path = Some(path);
    });

    let top_level_wrapper = if let Some(error_path) = error_path {
        quote! { ::core::result::Result<#container_ident #ty_generics, #error_path> }
    } else {
        quote! { ::core::option::Option<#container_ident> }
    };

    // Build wrapper struct body
    // let mut wrapper_body = vec![];
    match container_data {
        Data::Struct(struct_data) => match &struct_data.fields {
            Fields::Named(fields) => {
                for field in &fields.named {
                    let field_attrs = &field.attrs;
                    let field_name = &field.ident;
                    let field_ty = &field.ty;
                    process_meta_name_value(field_attrs, &mut |meta_name_value| {
                        let ident = if let Some(ident) = meta_name_value.path.get_ident() {
                            ident
                        } else {
                            return;
                        };
                        match ident.to_string().as_ref() {
                            "error" => {}
                            "skip_error" => {}
                            _ => {
                                compiler_errors.push(
                                    Error::new(ident.span(), &format!("Unknown key {}", ident))
                                        .to_compile_error(),
                                );
                                return;
                            }
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
                                    &format!(
                                        "Unable to convert {} to a valid path.",
                                        str_lit.value()
                                    ),
                                )
                                .to_compile_error(),
                            );
                            return;
                        };
                    });
                }
            }
            _ => {
                return TokenStream::from(
                    Error::new(input.span(), "Unsupported data type").to_compile_error(),
                )
            }
        },
        _ => {
            return TokenStream::from(
                Error::new(input.span(), "Unsupported data type").to_compile_error(),
            )
        }
    }

    let expanded = quote! {
        #(#compiler_errors)*
        #container_vis #wrapper_struct_ident #container_generics {
            data: #top_level_wrapper,
        }

        impl #impl_generics #wrapper_struct_ident #ty_generics #where_clause {

        }
    };

    TokenStream::from(expanded)
}
