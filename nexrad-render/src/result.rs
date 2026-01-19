//! Result and error types for NEXRAD rendering operations.

use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD rendering operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD rendering.
#[derive(ThisError, Debug)]
pub enum Error {
    /// The requested radar product was not found in the radial data.
    #[error("requested product not found")]
    ProductNotFound,
    /// An error occurred in the graphics rendering backend.
    #[error("error rendering image")]
    RenderError(#[from] piet::Error),
}
