use crate::result::aws::AWSError;
use crate::result::aws::AWSError::{S3GetObjectError, S3GetObjectRequestError, S3StreamingError};
use crate::result::Error;
use reqwest::StatusCode;

/// Downloads an object from S3 and returns its contents.
pub async fn download_object(bucket: &str, key: &str) -> crate::result::Result<Vec<u8>> {
    let path = format!("https://{bucket}.s3.amazonaws.com/{key}");

    let response = reqwest::get(path).await.map_err(S3GetObjectRequestError)?;
    match response.status() {
        StatusCode::NOT_FOUND => Err(Error::AWS(AWSError::S3ObjectNotFoundError)),
        StatusCode::OK => Ok(response.bytes().await.map_err(S3StreamingError)?.to_vec()),
        _ => Err(Error::AWS(S3GetObjectError(response.text().await.ok()))),
    }
}
