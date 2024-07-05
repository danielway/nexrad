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
    #[error("AWS API error")]
    S3GeneralError(#[from] aws_sdk_s3::Error),
    #[cfg(feature = "download")]
    #[error("error listing AWS S3 objects")]
    S3ListObjectsError(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>),
    #[cfg(feature = "download")]
    #[error("error getting AWS S3 object")]
    S3GetObjectError(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>),
    #[cfg(feature = "download")]
    #[error("error streaming/downloading AWS S3 object")]
    S3StreamingError(#[from] aws_sdk_s3::primitives::ByteStreamError),
}
