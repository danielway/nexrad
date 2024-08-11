use crate::result::Error;

/// Downloads an object from S3 and returns its contents.
pub async fn download_object(bucket: &str, key: &str) -> crate::result::Result<Vec<u8>> {
    let path = format!("https://{bucket}.s3.amazonaws.com/{key}");
    let response = reqwest::get(path).await.map_err(Error::S3GetObjectError)?;

    let bytes = response.bytes().await.map_err(Error::S3StreamingError)?;
    Ok(bytes.to_vec())
}
