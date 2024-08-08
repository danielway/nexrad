use chrono::{DateTime, Utc};

/// A bucket object returned from an S3 list objects request.
pub struct BucketObject {
    /// The key of the object.
    pub key: String,
    /// The last modified time of the object.
    pub last_modified: DateTime<Utc>,
    /// The size of the object.
    pub size: u64,
}
