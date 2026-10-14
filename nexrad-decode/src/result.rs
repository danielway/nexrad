//! Result and error types for NEXRAD decoding operations.

use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD decoding operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD data decoding.
#[derive(ThisError, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Malformed input that does not match a more specific variant.
    ///
    /// This is the catch-all for decoding failures; the message describes
    /// what was wrong with the input.
    #[error("file decoding error: {0}")]
    Decoding(String),
    /// A message is missing required collection date/time fields.
    #[error("message is missing collection date/time")]
    MessageMissingDate,
    /// Reached end of input data unexpectedly during parsing.
    #[error("unexpected end of file to input data")]
    UnexpectedEof,
    /// Message length field is invalid for the message type.
    #[error("invalid message length for type {message_type}: cannot rewind {delta} bytes")]
    InvalidMessageLength {
        /// The message type being parsed.
        message_type: String,
        /// The invalid byte offset.
        delta: i32,
    },
    /// A data block pointer points outside valid message bounds.
    #[error("invalid data block pointer: cannot rewind {bytes} bytes at position {position}")]
    InvalidDataBlockPointer {
        /// Number of bytes the pointer tried to rewind.
        bytes: usize,
        /// Current position in the data.
        position: usize,
    },
    /// Encountered an unrecognized data block type identifier.
    #[error("unknown data block type: {block_type}")]
    UnknownDataBlockType {
        /// The unrecognized block type identifier.
        block_type: String,
    },
    /// A data structure spans across segment boundaries.
    #[error("data structure spans segment boundary at position {position}")]
    DataSpansSegmentBoundary {
        /// Position where the boundary was crossed.
        position: usize,
    },
    /// No volume coverage pattern message was found while assembling a scan.
    #[error("no volume coverage pattern message found")]
    MissingCoveragePattern,
    /// Decoded values were rejected by a [`nexrad_model`] constructor, such as moment data
    /// whose gate count does not match its encoded data or a radial with an out-of-range
    /// azimuth.
    #[cfg(feature = "model")]
    #[error("decoded data is invalid for the model: {0}")]
    Model(#[from] nexrad_model::result::Error),
}
