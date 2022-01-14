use err_per_field::{ErrPerField, Wrapper};

#[derive(Debug, PartialEq)]
struct AnError;

#[derive(ErrPerField, Debug)]
struct Foo<T> {
    bar1: u8,
    #[err_per_field(maybe_none)]
    bar2: T,
    #[err_per_field(maybe_error = "AnError")]
    bar3: u32,
    bar4: u64,
}

#[test]
fn test_all_field_valid() {
    let foo_wrapper = Wrapper::<Foo<_>> {
        bar1: 1,
        bar2: Some(2),
        bar3: Ok(3),
        bar4: 4,
    };
    let result: Result<Foo<_>, Wrapper<Foo<_>>> = foo_wrapper.try_into();
    assert!(result.is_ok());
    let foo = result.unwrap();
    assert_eq!(foo.bar1, 1);
    assert_eq!(foo.bar2, 2);
    assert_eq!(foo.bar3, 3);
    assert_eq!(foo.bar4, 4);
}
