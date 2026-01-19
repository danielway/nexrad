//! Result and Error types for NEXRAD model operations.

use crate::field::FieldError;
use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD model operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD model operations.
#[derive(ThisError, Debug)]
pub enum Error {
    /// Attempted to merge sweeps with mismatched elevation numbers.
    ///
    /// This error occurs when calling [`Sweep::merge`](crate::data::Sweep::merge)
    /// with two sweeps that have different elevation numbers.
    #[error("two sweeps' elevation numbers do not match")]
    ElevationMismatchError,

    /// Error during field data operations.
    ///
    /// This error occurs during conversion from radials to polar sweeps
    /// or other field data operations.
    #[error("field error: {0}")]
    Field(#[from] FieldError),
}
