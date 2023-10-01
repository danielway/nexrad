//!
//! Contains the Result and Error types for NEXRAD operations.
//!

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    DecompressionError(String),
    FileError(std::io::Error),
    DeserializationError(bincode::Error),
    #[cfg(feature = "download")]
    S3GeneralError(aws_sdk_s3::Error),
    #[cfg(feature = "download")]
    S3ListObjectsError,
    #[cfg(feature = "download")]
    S3GetObjectError,
    #[cfg(feature = "download")]
    S3StreamingError,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::FileError(err)
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::DeserializationError(err)
    }
}

#[cfg(feature = "download")]
impl From<aws_sdk_s3::Error> for Error {
    fn from(err: aws_sdk_s3::Error) -> Self {
        Error::S3GeneralError(err)
    }
}
