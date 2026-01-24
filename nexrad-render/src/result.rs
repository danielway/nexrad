//! Result and error types for NEXRAD rendering operations.

use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD rendering operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD rendering.
#[derive(ThisError, Debug)]
pub enum Error {
    /// The requested radar product was not found in the radial data.
    #[error("requested product not found in radial data")]
    ProductNotFound,
    /// No radials were provided for rendering.
    #[error("no radials provided for rendering")]
    NoRadials,
    /// The image dimensions were invalid for creating an image buffer.
    #[error("invalid image dimensions")]
    InvalidDimensions,
}
