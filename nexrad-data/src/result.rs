//!
//! Contains the Result and Error types for NEXRAD operations.
//!

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("data file IO error")]
    FileError(#[from] std::io::Error),
    #[cfg(feature = "aws")]
    #[error("unexpected truncated S3 list objects response")]
    TruncatedListObjectsResponse,
    #[cfg(feature = "aws")]
    #[error("error decompressing uncompressed data")]
    UncompressedDataError,
    #[cfg(feature = "aws")]
    #[error("error decoding date/time")]
    DateTimeError(String),
    #[cfg(feature = "aws")]
    #[error("invalid radar site identifier")]
    InvalidSiteIdentifier(String),
    #[cfg(feature = "aws")]
    #[error("ldm record decompression error")]
    DecompressionError(#[from] bzip2::Error),
    #[cfg(feature = "aws")]
    #[error("file deserialization error")]
    DeserializationError(#[from] bincode::Error),
    #[cfg(feature = "aws")]
    #[error("error listing AWS S3 objects")]
    S3ListObjectsError(reqwest::Error),
    #[cfg(feature = "aws")]
    #[error("error getting AWS S3 object")]
    S3GetObjectError(reqwest::Error),
    #[cfg(feature = "aws")]
    #[error("error streaming/downloading AWS S3 object")]
    S3StreamingError(reqwest::Error),
}
