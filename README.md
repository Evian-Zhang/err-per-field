# `err-per-field`

This crate provides a more fine-grained control over field-level error handling.

When we are writing a huge complex project, we may gather a lot of information into a single struct, such as:

```rust,ignore
struct GatheredInformation {
    info1: Info1,
    info2: Info2,
    info3: Info3,
}

fn generate_info1() -> Result<Info1, Error1> { /* ... */ }
fn generate_info2() -> Option<Info2> { /* ... */ }
fn generate_info3() -> Info3 { /* ... */ }
```

However, we may encounter `Result`s or `Option`s when generating such subinfos. The core logic is to deal with `GatheredInformation` if all subinfos are retrieved successfully. If some fields are failed to retrieve, let's say `info2`, we should inform the user that failure, and may provide other successfully retrieved infos (such as `info1` and `info3`) to user.

To handle this gracefully, dynamically-typed languages such as Javascript can be rather straight forward:

```javascript
const gathered_information = gather_information();
if (gathered_information.info1 && gathered_information.info2) {
    deal_with_gathered_information(gathered_information);
} else {
    deal_with_error();
}
```

For Rust, a statically-typed langugage, things become complicated:

```rust,ignore
struct GatheredInformationWrapper {
    info1: Result<Info1, Error1>,
    info2: Option<Info2>,
    info3: Info3,
}

let gathered_information_wrapper = gather_information();
if let GatheredInformationWrapper {
    info1: Ok(info1),
    info2: Some(info2),
    info3,
} = gathered_information_wrappper {
    let gathered_information = GatheredInformation {
        info1,
        info2,
        info3,
    };
    deal_with_gathered_information(gathered_information);
} else {
    deal_with_error();
}
```

We must:

1. Define a wrapper struct of `GatheredInformation`, whose fields are `Result`s or `Option`s according to the generating functions;
2. Build the real `GatheredInformation` after validation checks;
3. Pass the real gathered information to the downstream functions.

This is a boring process and the extra wrapper structs are very distracting.

To handle this problem, this crate provides a rather simple and graceful method.

## Example

```rust
use err_per_field::{ErrPerField, Wrapper};

struct AnError;

#[derive(ErrPerField)]
struct Foo {
    pub bar1: u8,
    #[err_per_field(maybe_none)]
    pub bar2: u16,
    #[err_per_field(maybe_error = "AnError")]
    pub bar3: u32,
    pub bar4: u64,
}

fn baz1() -> u8 { 0 }
fn baz2() -> Option<u16> { None }
fn baz3() -> Result<u32, AnError> { Err(AnError) }
fn baz4() -> u64 { 0 }

fn generate_foo() -> Wrapper<Foo> {
    let bar1 = baz1();
    let bar2 = baz2();
    let bar3 = baz3();
    let bar4 = baz4();
    Wrapper::<Foo> {
        bar1,
        bar2,
        bar3,
        bar4,
    }
}

// If we write this in another file, we can simply import `Wrapper` type
// from `err_per_field` crate.
let result: Result<Foo, Wrapper<Foo>> = generate_foo().try_into();
assert!(result.is_err());
match result {
    Ok(foo) => {
        // `foo` is of type `Foo`, and you can directly use it without any worries
    },
    Err(foo_wrapper) => {
        // `foo_wrapper` has the same fields as `foo`, but in different types,
        // such as `foo_wrapper.bar2` is of type `Option<u16>` and
        // `foo_wrapper.bar3` is of type `Result<u32, AnError>`
    }
}
```

By using this crate, we don't need to write wrapper structs by our own, and we can focus on the core logic of our product. All we need to do is:

1. Derive the core struct `Foo` with macro `ErrPerField` and mark fields which may be `Result`s or `Option`s with attributes;
2. When generating the struct `Foo`, use `Wrapper<Foo>` as return type. This wrapper struct has the same field names with `Foo`, and their types may be `Result`s or `Option`s as we mark when declaration;
3. When using the generated struct, we can validate its fields by calling `try_into`, and if it is valid, the result is `Ok` and the inner type is `Foo`, and we can directly deal with it as the core logic; if it is invalid, the result is `Err` and the inner type is `Foo`'s wrapper, and we can check each fields for error handling.

## Usage

When we use `#[derive(ErrPerField)]` on a struct `Foo`, the macro generates a wrapper struct and we can use `Wrapper<Foo>` to access it. This wrapper struct has the same field names as `Foo`, and for a field `bar`, if `foo.bar` has type `Bar`, then `foo_wrapper.bar`'s type is:

* `Result<Bar, AnError>` if there is a field-level attribute `#[err_per_field(maybe_error = "AnError")]`;
* `Option<Bar>` if there is a field-level attribute `#[err_per_field(maybe_none)]`;
* `Bar` if otherwise.

`Foo` will automatically implements `TryFrom<Wrapper<Foo>>`. If there is a field being `Err` or `None`, the conversion fails and the result is `Err`, the inner value is `foo`'s wrapper itself without any change; otherwise the conversion succeeds and the result is `Ok`, the inner type is the final `foo`, with all fields extracted from `Result`s and `Option`s.
