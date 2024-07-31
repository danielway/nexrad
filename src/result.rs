//!
//! Contains the Result and Error types for NEXRAD operations.
//!

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("data file IO error")]
    FileError(#[from] std::io::Error),
    #[error("error listing AWS S3 objects")]
    S3ListObjectsError(reqwest::Error),
    #[error("error getting AWS S3 object")]
    S3GetObjectError(reqwest::Error),
    #[error("error streaming/downloading AWS S3 object")]
    S3StreamingError(reqwest::Error),
}
