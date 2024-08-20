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
    #[cfg(feature = "aws")]
    #[error(transparent)]
    AWS(#[from] aws::AWSError),
}

#[cfg(feature = "aws")]
pub mod aws {
    use thiserror::Error as ThisError;

    #[derive(ThisError, Debug)]
    pub enum AWSError {
        #[error("unexpected truncated S3 list objects response")]
        TruncatedListObjectsResponse,
        #[error("error decompressing uncompressed data")]
        UncompressedDataError,
        #[error("error decoding date/time")]
        DateTimeError(String),
        #[error("invalid radar site identifier")]
        InvalidSiteIdentifier(String),
        #[error("ldm record decompression error")]
        DecompressionError(#[from] bzip2::Error),
        #[error("error listing AWS S3 objects")]
        S3ListObjectsError(reqwest::Error),
        #[error("error getting AWS S3 object")]
        S3GetObjectError(reqwest::Error),
        #[error("error streaming/downloading AWS S3 object")]
        S3StreamingError(reqwest::Error),
    }
}
