//! ```rust
//! use err_per_field::{ErrPerField, Wrapper};
//!
//! struct AnError;
//!
//! #[derive(ErrPerField)]
//! struct Foo {
//!     pub bar1: u8,
//!     #[err_per_field(maybe_none)]
//!     pub bar2: u16,
//!     #[err_per_field(maybe_error = "AnError")]
//!     pub bar3: u32,
//!     pub bar4: u64,
//! }
//!
//! fn baz1() -> u8 { 0 }
//! fn baz2() -> Option<u16> { None }
//! fn baz3() -> Result<u32, AnError> { Err(AnError) }
//! fn baz4() -> u64 { 0 }
//!
//! fn generate_foo() -> Wrapper<Foo> {
//!     let bar1: u8 = baz1();
//!     let bar2: Option<u16> = baz2();
//!     let bar3: Result<u32, AnError> = baz3();
//!     let bar4: u64 = baz4();
//!     Wrapper::<Foo> {
//!         bar1,
//!         bar2,
//!         bar3,
//!         bar4,
//!     }
//! }
//!
//! let result: Result<Foo, Wrapper<Foo>> = generate_foo().try_into();
//! assert!(result.is_err());
//! match result {
//!     Ok(foo) => {
//!         // foo is of type Foo, and you can directly use foo without any worries
//!     },
//!     Err(foo_wrapper) => {
//!         // foo_wrapper has the same fields as foo, but in different types
//!         // such as foo_wrapper.bar2 is of type Option<u16> and
//!         // foo_wrapper.bar3 is of type Result<u32, AnError>
//!     }
//! }
//! ```

/// Re-export of derived macro
pub use err_per_field_derive::ErrPerField;

/// Trait used for inner usage. DO NOT USE DIRECTLY!
#[doc(hidden)]
pub trait FieldMayErr {
    /// Wrapper of this struct
    type Wrapper;
}

/// A wrapper of the raw struct.
pub type Wrapper<T> = <T as FieldMayErr>::Wrapper;
