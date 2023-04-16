//! 
//! Contains the Result and Error types for NEXRAD operations.
//! 

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileError(std::io::Error),
    S3GeneralError(aws_sdk_s3::Error),
    S3ListObjectsError(aws_smithy_http::result::SdkError<
        aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error
    >),
    S3GetObjectError(aws_smithy_http::result::SdkError<
        aws_sdk_s3::operation::get_object::GetObjectError
    >),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::FileError(err)
    }
}

impl From<aws_sdk_s3::Error> for Error {
    fn from(err: aws_sdk_s3::Error) -> Self {
        Error::S3GeneralError(err)
    }
}

impl From<aws_smithy_http::result::SdkError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>> for Error {
    fn from(err: aws_smithy_http::result::SdkError<aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error>) -> Self {
        Error::S3ListObjectsError(err)
    }
}

impl From<aws_smithy_http::result::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>> for Error {
    fn from(err: aws_smithy_http::result::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>) -> Self {
        Error::S3GetObjectError(err)
    }
}