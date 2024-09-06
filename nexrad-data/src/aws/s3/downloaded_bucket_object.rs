use chrono::{DateTime, Utc};

/// A bucket object returned from an S3 list objects request.
pub struct DownloadedBucketObject {
    /// The key of the object.
    pub key: String,
    /// The last modified time of the object.
    pub last_modified: Option<DateTime<Utc>>,
    /// The object data.
    pub data: Vec<u8>,
}
