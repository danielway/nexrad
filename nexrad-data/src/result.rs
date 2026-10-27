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
    /// Cannot decode messages or radials from a bzip2-compressed LDM record.
    /// Call `Record::decompress()` first.
    #[error("compressed record must be decompressed before decoding")]
    CompressedRecord,
    /// Cannot access records or scan from a gzip-compressed volume file.
    /// Call `decompress()` first.
    #[error("gzip-compressed volume file must be decompressed before accessing records")]
    CompressedFile,
    /// Volume file is missing the required VCP (message type 5).
    #[error("volume missing coverage pattern (message type 5)")]
    MissingCoveragePattern,
    /// LDM record was truncated and contains fewer bytes than expected.
    ///
    /// Byte counts are relative to the start of the record region (the bytes following the
    /// 24-byte Archive II volume header). If the volume data is shorter than that header, the
    /// counts are the header size and the volume's length instead.
    #[error("truncated record: expected {expected} bytes, got {actual}")]
    #[non_exhaustive]
    TruncatedRecord {
        /// Expected number of bytes.
        expected: usize,
        /// Actual number of bytes present.
        actual: usize,
    },
    /// LDM record size is invalid at the given offset.
    #[error("invalid record size {size} at offset {offset}")]
    #[non_exhaustive]
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
    #[non_exhaustive]
    InvalidIdentifier {
        /// The rejected name.
        name: String,
        /// Which part of the grammar the name violates.
        reason: &'static str,
    },
    /// A real-time volume index is outside `1..=999`.
    #[error("invalid real-time volume index {index}, expected 1..=999")]
    #[non_exhaustive]
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
    #[non_exhaustive]
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
    /// Whether retrying the same request later can succeed.
    ///
    /// The contract is narrow on purpose, so a caller can retry on `true` without repeating
    /// requests the server has already refused:
    ///
    /// - Transport failures are transient: a request that could not be sent or completed, a
    ///   connection failure, a timeout, or a response body that could not be read
    ///   ([`S3GetObjectRequest`](Self::S3GetObjectRequest), [`S3Streaming`](Self::S3Streaming)
    ///   and the transport case of [`S3ListObjects`](Self::S3ListObjects)), as is
    ///   [`TruncatedListObjectsResponse`](Self::TruncatedListObjectsResponse).
    /// - An HTTP error status is transient only when it is 408 (Request Timeout), 429 (Too Many
    ///   Requests) or a 5xx server error. This applies to [`S3GetObject`](Self::S3GetObject) and
    ///   to a listing request that [`S3ListObjects`](Self::S3ListObjects) reports with a status.
    ///   Every other 4xx status (400, 403 and so on) is terminal.
    /// - [`S3ObjectNotFound`](Self::S3ObjectNotFound) (404) is terminal: an archive 404 does not
    ///   resolve itself, and real-time polling handles not-yet-published chunks itself.
    /// - Every other variant (parsing, validation and lookup failures) is terminal.
    pub fn is_transient(&self) -> bool {
        match self {
            Self::S3GetObject { status, .. } => is_transient_status(*status),
            Self::S3GetObjectRequest(source)
            | Self::S3Streaming(source)
            | Self::S3ListObjects(source) => is_transient_source(source.as_ref()),
            Self::TruncatedListObjectsResponse => true,
            _ => false,
        }
    }
}

/// Whether an HTTP error status means the same request can succeed later: 408, 429 and 5xx.
#[cfg(feature = "aws")]
fn is_transient_status(status: u16) -> bool {
    matches!(status, 408 | 429 | 500..=599)
}

/// Classifies a boxed transport error. An HTTP client error that carries a status (from
/// `error_for_status`) is classified by that status; any other error is a transport failure.
#[cfg(feature = "aws")]
fn is_transient_source(source: &(dyn std::error::Error + Send + Sync + 'static)) -> bool {
    match source
        .downcast_ref::<reqwest::Error>()
        .and_then(reqwest::Error::status)
    {
        Some(status) => is_transient_status(status.as_u16()),
        None => true,
    }
}

#[cfg(all(test, feature = "aws"))]
mod tests {
    use super::*;

    fn boxed() -> Box<dyn std::error::Error + Send + Sync + 'static> {
        Box::new(std::io::Error::other("transport"))
    }

    fn status(status: u16) -> AwsError {
        AwsError::S3GetObject { status, body: None }
    }

    #[test]
    fn transient_aws_errors() {
        assert!(AwsError::S3GetObjectRequest(boxed()).is_transient());
        for code in [408, 429, 500, 502, 503, 504, 599] {
            assert!(status(code).is_transient(), "{code} should be transient");
        }
        assert!(AwsError::S3Streaming(boxed()).is_transient());
        assert!(AwsError::S3ListObjects(boxed()).is_transient());
        assert!(AwsError::TruncatedListObjectsResponse.is_transient());
    }

    #[test]
    fn client_error_statuses_are_terminal() {
        for code in [400, 401, 403, 404, 405, 409, 410, 412, 416, 418, 499] {
            assert!(!status(code).is_transient(), "{code} should be terminal");
        }
        // Informational, success and redirect statuses are not retry signals either.
        for code in [100, 200, 301, 304, 600] {
            assert!(!status(code).is_transient(), "{code} should be terminal");
        }
    }

    #[tokio::test]
    async fn listing_status_errors_are_classified_by_status() {
        async fn listing_error(code: u16) -> AwsError {
            let response = http::Response::builder().status(code).body("").unwrap();
            let error = reqwest::Response::from(response)
                .error_for_status()
                .expect_err("status is an error");
            AwsError::S3ListObjects(Box::new(error))
        }
        assert!(listing_error(503).await.is_transient());
        assert!(listing_error(429).await.is_transient());
        assert!(listing_error(408).await.is_transient());
        assert!(!listing_error(403).await.is_transient());
        assert!(!listing_error(400).await.is_transient());
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
