use crate::aws::s3::bucket_object::BucketObject;
use crate::aws::s3::downloaded_bucket_object::DownloadedBucketObject;
use crate::result::aws::AWSError;
use crate::result::aws::AWSError::{S3GetObjectError, S3GetObjectRequestError, S3StreamingError};
use crate::result::Error;
use chrono::{DateTime, Utc};
use log::{debug, trace};
use reqwest::header::HeaderMap;
use reqwest::StatusCode;

/// Downloads an object from S3 and returns its contents.
pub async fn download_object(
    bucket: &str,
    key: &str,
) -> crate::result::Result<DownloadedBucketObject> {
    debug!(
        "Downloading object key \"{}\" from bucket \"{}\"",
        key, bucket
    );
    let path = format!("https://{bucket}.s3.amazonaws.com/{key}");

    let response = reqwest::get(path).await.map_err(S3GetObjectRequestError)?;
    trace!(
        "  Object \"{}\" download response status: {}",
        key,
        response.status()
    );

    match response.status() {
        StatusCode::NOT_FOUND => Err(Error::AWS(AWSError::S3ObjectNotFoundError)),
        StatusCode::OK => {
            let last_modified = get_last_modified_header(response.headers());
            trace!("  Object \"{}\" last modified: {:?}", key, last_modified);

            let data = response.bytes().await.map_err(S3StreamingError)?.to_vec();
            trace!("  Object \"{}\" data length: {}", key, data.len());

            Ok(DownloadedBucketObject {
                metadata: BucketObject {
                    key: key.to_string(),
                    last_modified,
                    size: data.len() as u64,
                },
                data,
            })
        }
        _ => Err(Error::AWS(S3GetObjectError(response.text().await.ok()))),
    }
}

/// Extracts the `Last-Modified` header from a response and returns it as a `DateTime<Utc>`.
fn get_last_modified_header(response_headers: &HeaderMap) -> Option<DateTime<Utc>> {
    let header = response_headers.get("Last-Modified");
    let date_string = header.and_then(|value| value.to_str().ok());

    date_string.and_then(|string| {
        DateTime::parse_from_rfc2822(string)
            .ok()
            .map(|date_time| date_time.with_timezone(&Utc))
    })
}
