use err_per_field::{ErrPerField, Wrapper};

#[derive(Debug, PartialEq)]
struct AnError;

#[derive(ErrPerField, Debug)]
struct Foo {
    bar1: u8,
    #[err_per_field(maybe_none)]
    bar2: u16,
    #[err_per_field(maybe_error = "AnError")]
    bar3: u32,
    bar4: u64,
}

#[test]
fn test_all_field_valid() {
    let foo_wrapper = Wrapper::<Foo> {
        bar1: 1,
        bar2: Some(2),
        bar3: Ok(3),
        bar4: 4,
    };
    let result: Result<Foo, Wrapper<Foo>> = foo_wrapper.try_into();
    assert!(result.is_ok());
    let foo = result.unwrap();
    assert_eq!(foo.bar1, 1);
    assert_eq!(foo.bar2, 2);
    assert_eq!(foo.bar3, 3);
    assert_eq!(foo.bar4, 4);
}

#[test]
fn test_none_field() {
    let foo_wrapper = Wrapper::<Foo> {
        bar1: 1,
        bar2: None,
        bar3: Ok(3),
        bar4: 4,
    };
    let result: Result<Foo, Wrapper<Foo>> = foo_wrapper.try_into();
    assert!(result.is_err());
    let foo_wrapper = result.unwrap_err();
    assert_eq!(foo_wrapper.bar1, 1);
    assert_eq!(foo_wrapper.bar2, None);
    assert_eq!(foo_wrapper.bar3, Ok(3));
    assert_eq!(foo_wrapper.bar4, 4);
}

#[test]
fn test_err_field() {
    let foo_wrapper = Wrapper::<Foo> {
        bar1: 1,
        bar2: Some(2),
        bar3: Err(AnError),
        bar4: 4,
    };
    let result: Result<Foo, Wrapper<Foo>> = foo_wrapper.try_into();
    assert!(result.is_err());
    let foo_wrapper = result.unwrap_err();
    assert_eq!(foo_wrapper.bar1, 1);
    assert_eq!(foo_wrapper.bar2, Some(2));
    assert_eq!(foo_wrapper.bar3, Err(AnError));
    assert_eq!(foo_wrapper.bar4, 4);
}
