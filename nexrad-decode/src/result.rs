//!
//! Contains the Result and Error types for NEXRAD operations.
//!

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("data file IO error")]
    FileError(#[from] std::io::Error),
    #[error("file decoding error: {0}")]
    DecodingError(String),
    #[error("message is missing collection date/time")]
    MessageMissingDateError,
    #[error("unexpected end of file to input data")]
    UnexpectedEof,
    #[error("invalid message length for type {message_type}: cannot rewind {delta} bytes")]
    InvalidMessageLength { message_type: String, delta: i32 },
    #[error("invalid data block pointer: cannot rewind {bytes} bytes at position {position}")]
    InvalidDataBlockPointer { bytes: usize, position: usize },
    #[error("unknown data block type: {block_type}")]
    UnknownDataBlockType { block_type: String },
    #[error("segmented message has out-of-order segment: expected {expected}, got {actual}")]
    SegmentOutOfOrder { expected: u16, actual: u16 },
    #[error("data structure spans segment boundary at position {position}")]
    DataSpansSegmentBoundary { position: usize },
}
