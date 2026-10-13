//! Result and error types for NEXRAD rendering operations.

use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD rendering operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD rendering.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The field to render contains no data (no radials, gates, rows or columns).
    #[error("field contains no data")]
    EmptyField,
    /// The output image dimensions were invalid (zero, or too large for an image
    /// buffer).
    #[error("invalid image dimensions")]
    InvalidDimensions,
    /// An error from the [`image`] crate, such as failing to encode or save an image.
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
}
