//! Result and error types for NEXRAD data operations.

use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD data operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD data operations.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    /// An I/O error occurred while reading or writing data.
    #[error("data file IO error")]
    Io(#[from] std::io::Error),
    /// Attempted to decompress data that is not compressed.
    #[error("error decompressing uncompressed data")]
    UncompressedData,
    /// An AWS-related error occurred (requires `aws` feature).
    #[cfg(feature = "aws")]
    #[error(transparent)]
    Aws(#[from] AwsError),
    /// An error occurred during message decoding.
    #[error("error decoding NEXRAD data")]
    Decode(#[from] nexrad_decode::result::Error),
    /// An error occurred in the common model layer (requires `model` feature).
    #[cfg(feature = "model")]
    #[error("error in common model")]
    Model(#[from] nexrad_model::result::Error),
    /// Cannot decode compressed record data without decompression.
    #[error("compressed data cannot be decoded")]
    CompressedData,
    /// Cannot borrow records from a gzip-compressed volume file (`File::records`).
    /// Call `decompress()` first.
    #[error("gzip-compressed volume file must be decompressed before accessing records")]
    CompressedFile,
    /// No volume coverage pattern message (type 5) was found where one is required, such as by
    /// `File::coverage_pattern_message`.
    #[error("volume missing coverage pattern (message type 5)")]
    MissingCoveragePattern,
    /// LDM record was truncated and contains fewer bytes than expected.
    ///
    /// Byte counts are relative to the start of the record region (the bytes following the
    /// 24-byte Archive II volume header). If the volume data is shorter than that header, the
    /// counts are the header size and the volume's length instead.
    #[error("truncated record: expected {expected} bytes, got {actual}")]
    TruncatedRecord {
        /// Expected number of bytes.
        expected: usize,
        /// Actual number of bytes present.
        actual: usize,
    },
    /// LDM record size is invalid at the given offset.
    #[error("invalid record size {size} at offset {offset}")]
    InvalidRecordSize {
        /// The invalid size value.
        size: usize,
        /// Offset where the invalid size was found, relative to the start of the record region
        /// (the bytes following the 24-byte Archive II volume header), not the start of the file.
        offset: usize,
    },
}

/// Errors that can occur during AWS S3 operations (requires `aws` feature).
#[cfg(feature = "aws")]
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum AwsError {
    /// S3 list objects response was unexpectedly truncated.
    #[error("unexpected truncated S3 list objects response")]
    TruncatedListObjectsResponse,
    /// An archive object name does not follow the archive naming grammar.
    #[error("invalid archive identifier {name:?}: {reason}")]
    InvalidIdentifier {
        /// The rejected name.
        name: String,
        /// Which part of the grammar the name violates.
        reason: &'static str,
    },
    /// A real-time volume index is outside `1..=999`.
    #[error("invalid real-time volume index {index}, expected 1..=999")]
    InvalidVolumeIndex {
        /// The rejected index.
        index: usize,
    },
    /// The shared HTTP client could not be built.
    #[error("error building HTTP client")]
    HttpClient(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    /// Real-time chunk data is in an unrecognized format.
    #[error("chunk data in unrecognized format")]
    UnrecognizedChunkFormat,
    /// Could not parse date/time from chunk filename.
    #[error("unrecognized chunk date time")]
    UnrecognizedChunkDateTime(String),
    /// Could not parse sequence number from chunk filename.
    #[error("unrecognized chunk sequence")]
    UnrecognizedChunkSequence(String),
    /// Chunk type character is not recognized.
    #[error("unrecognized chunk type")]
    UnrecognizedChunkType(Option<char>),
    /// S3 list objects request failed.
    #[error("error listing AWS S3 objects")]
    S3ListObjects(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    /// S3 get object request failed to send.
    #[error("error requesting AWS S3 object")]
    S3GetObjectRequest(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    /// S3 get object returned an unsuccessful status other than 404.
    #[error("error getting AWS S3 object: HTTP status {status}")]
    S3GetObject {
        /// The HTTP status code of the response.
        status: u16,
        /// The response body, if it could be read.
        body: Option<String>,
    },
    /// Requested S3 object was not found (404).
    #[error("AWS S3 object not found")]
    S3ObjectNotFound,
    /// Error while streaming/downloading S3 object content.
    #[error("error streaming/downloading AWS S3 object")]
    S3Streaming(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    /// Could not find the latest volume for the requested site/date.
    #[error("failed to locate latest volume")]
    LatestVolumeNotFound,
    /// An expected chunk was not found during real-time polling.
    #[error("a chunk was not found as expected")]
    ExpectedChunkNotFound,
    /// Could not determine the next chunk to poll.
    #[error("failed to determine next chunk")]
    FailedToDetermineNextChunk,
    /// Failed to decode XML response from S3 list objects.
    #[error("error decoding S3 list objects response")]
    S3ListObjectsDecoding,
}

#[cfg(feature = "aws")]
impl AwsError {
    /// Whether this error is a transport-level failure where retrying the same request later can
    /// succeed.
    ///
    /// Returns `true` for [`S3GetObjectRequest`](Self::S3GetObjectRequest),
    /// [`S3GetObject`](Self::S3GetObject), [`S3Streaming`](Self::S3Streaming),
    /// [`S3ListObjects`](Self::S3ListObjects) and
    /// [`TruncatedListObjectsResponse`](Self::TruncatedListObjectsResponse).
    ///
    /// [`S3ObjectNotFound`](Self::S3ObjectNotFound) is deliberately not transient: an archive 404
    /// is terminal, and real-time polling handles not-yet-published chunks itself.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::S3GetObjectRequest(_)
                | Self::S3GetObject { .. }
                | Self::S3Streaming(_)
                | Self::S3ListObjects(_)
                | Self::TruncatedListObjectsResponse
        )
    }
}

#[cfg(all(test, feature = "aws"))]
mod tests {
    use super::*;

    fn boxed() -> Box<dyn std::error::Error + Send + Sync + 'static> {
        Box::new(std::io::Error::other("transport"))
    }

    #[test]
    fn transient_aws_errors() {
        assert!(AwsError::S3GetObjectRequest(boxed()).is_transient());
        assert!(AwsError::S3GetObject {
            status: 503,
            body: None
        }
        .is_transient());
        assert!(AwsError::S3Streaming(boxed()).is_transient());
        assert!(AwsError::S3ListObjects(boxed()).is_transient());
        assert!(AwsError::TruncatedListObjectsResponse.is_transient());
    }

    #[test]
    fn terminal_aws_errors() {
        assert!(!AwsError::S3ObjectNotFound.is_transient());
        assert!(!AwsError::S3ListObjectsDecoding.is_transient());
        assert!(!AwsError::LatestVolumeNotFound.is_transient());
        assert!(!AwsError::ExpectedChunkNotFound.is_transient());
        assert!(!AwsError::FailedToDetermineNextChunk.is_transient());
        assert!(!AwsError::UnrecognizedChunkFormat.is_transient());
        assert!(!AwsError::UnrecognizedChunkType(None).is_transient());
        assert!(!AwsError::InvalidIdentifier {
            name: String::new(),
            reason: "empty",
        }
        .is_transient());
        assert!(!AwsError::InvalidVolumeIndex { index: 0 }.is_transient());
        assert!(!AwsError::HttpClient(boxed()).is_transient());
    }

    #[test]
    fn boxed_payload_is_error_source() {
        use std::error::Error as _;
        let error = AwsError::S3Streaming(boxed());
        let source = error.source().map(|source| source.to_string());
        assert_eq!(source.as_deref(), Some("transport"));
    }
}
