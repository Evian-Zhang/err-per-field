#![doc=include_str!("../README.md")]

/// Re-export of derived macro
pub use err_per_field_derive::ErrPerField;

/// Trait used for inner usage. DO NOT USE DIRECTLY!
#[doc(hidden)]
pub trait ErrPerField {
    /// Wrapper of this struct
    type Wrapper;
}

/// A wrapper of the raw struct.
pub type Wrapper<T> = <T as ErrPerField>::Wrapper;
