use crate::aws::s3::bucket_object::BucketObject;

/// The result of a list objects request.
pub struct BucketListResult {
    /// Whether the list of objects is truncated.
    pub truncated: bool,
    /// The objects returned by the request.
    pub objects: Vec<BucketObject>,
}
