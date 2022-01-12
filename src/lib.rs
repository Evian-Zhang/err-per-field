pub use err_per_field_derive::ErrPerField;

/// Error trait used for using this crate.
///
/// If the marked struct can throw an error, then such error
/// should implement this trait
pub trait Error {
    /// There is something wrong in the internal fields, but not
    /// the whole struct
    fn map_per_field_err() -> Self;
}
