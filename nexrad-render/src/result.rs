//!
//! Contains the Result and Error types for NEXRAD operations.
//!

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("requested product not found")]
    ProductNotFound,
    #[error("error rendering image")]
    RenderError(#[from] piet::Error),
}
