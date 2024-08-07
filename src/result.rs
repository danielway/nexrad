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
    #[cfg(feature = "download")]
    #[error("error listing AWS S3 objects")]
    S3ListObjectsError(reqwest::Error),
    #[cfg(feature = "download")]
    #[error("error getting AWS S3 object")]
    S3GetObjectError(reqwest::Error),
    #[cfg(feature = "download")]
    #[error("error streaming/downloading AWS S3 object")]
    S3StreamingError(reqwest::Error),
}
