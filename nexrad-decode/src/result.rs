//!
//! Contains the Result and Error types for NEXRAD operations.
//!

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("data file IO error")]
    FileError(#[from] std::io::Error),
    #[error("file deserialization error")]
    DeserializationError(#[from] bincode::Error),
    #[error("file decoding error: {0}")]
    DecodingError(String),
}
