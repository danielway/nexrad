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
    #[cfg(feature = "bzip2")]
    #[error("error decompressing uncompressed data")]
    UncompressedDataError,
    #[cfg(feature = "aws")]
    #[error(transparent)]
    AWS(#[from] aws::AWSError),
    #[cfg(feature = "decode")]
    #[error("error decoding NEXRAD data")]
    Decode(#[from] nexrad_decode::result::Error),
    #[cfg(feature = "decode")]
    #[error("compressed data cannot be decoded")]
    CompressedDataError,
    #[cfg(feature = "decode")]
    #[error("volume missing coverage pattern number")]
    MissingCoveragePattern,
}

#[cfg(feature = "aws")]
pub mod aws {
    use thiserror::Error as ThisError;

    #[derive(ThisError, Debug)]
    pub enum AWSError {
        #[error("unexpected truncated S3 list objects response")]
        TruncatedListObjectsResponse,
        #[error("error decoding date/time")]
        DateTimeError(String),
        #[error("invalid radar site identifier")]
        InvalidSiteIdentifier(String),
        #[error("chunk data in unrecognized format")]
        UnrecognizedChunkFormat,
        #[error("ldm record decompression error")]
        DecompressionError(#[from] bzip2::Error),
        #[error("error listing AWS S3 objects")]
        S3ListObjectsError(reqwest::Error),
        #[error("error requesting AWS S3 object")]
        S3GetObjectRequestError(reqwest::Error),
        #[error("error getting AWS S3 object")]
        S3GetObjectError(Option<String>),
        #[error("AWS S3 object not found")]
        S3ObjectNotFoundError,
        #[error("error streaming/downloading AWS S3 object")]
        S3StreamingError(reqwest::Error),
        #[error("failed to locate latest volume")]
        LatestVolumeNotFound,
        #[error("a chunk was not found as expected")]
        ExpectedChunkNotFound,
        #[error("error sending chunk to receiver")]
        PollingAsyncError,
        #[error("failed to determine next chunk")]
        FailedToDetermineNextChunk,
        #[error("error decoding S3 list objects response")]
        S3ListObjectsDecodingError,
    }
}
