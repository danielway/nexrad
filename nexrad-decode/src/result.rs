//!
//! Contains the Result and Error types for NEXRAD operations.
//!

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("data file IO error")]
    FileError(#[from] std::io::Error),
    #[error("zerocopy conversion error: {0}")]
    ZerocopyError(String),
    #[error("file decoding error: {0}")]
    DecodingError(String),
    #[error("message is missing collection date/time")]
    MessageMissingDateError,
}

impl<A: std::fmt::Display, S: std::fmt::Display, V: std::fmt::Display>
    From<zerocopy::ConvertError<A, S, V>> for Error
{
    fn from(err: zerocopy::ConvertError<A, S, V>) -> Self {
        Error::ZerocopyError(err.to_string())
    }
}
