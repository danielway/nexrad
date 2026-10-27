//! Result and error types for NEXRAD decoding operations.

use crate::messages::MessageType;
use thiserror::Error as ThisError;

/// A specialized Result type for NEXRAD decoding operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during NEXRAD data decoding.
///
/// Offsets are byte offsets into the slice passed to the decoding entry point.
#[derive(ThisError, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// A radar data message lacks the collection date/time needed to convert it to a model
    /// radial.
    #[error("message type {} is missing its collection date/time", .message_type.number())]
    MessageMissingDate {
        /// The type of the message without a date/time.
        message_type: MessageType,
    },
    /// The input ended inside a structure being decoded.
    #[error("unexpected end of input reading the structure at offset {offset}")]
    UnexpectedEof {
        /// Offset of the structure that extends past the end of the input.
        offset: usize,
    },
    /// A data block pointer points before the end of the data already read.
    #[error("invalid data block pointer: cannot rewind {bytes} bytes at offset {position}")]
    InvalidDataBlockPointer {
        /// Number of bytes the pointer tried to rewind.
        bytes: usize,
        /// Offset of the read position when the pointer was followed.
        position: usize,
    },
    /// A digital radar data block has an unrecognized name and cannot be handled.
    ///
    /// Decoding returns this only for a block whose name is unrecognized and whose type byte is
    /// neither `'R'` nor `'D'`, which indicates a malformed message or a misplaced block pointer;
    /// unrecognized `'D'` blocks are kept in
    /// [`Message::other_data_blocks`](crate::messages::digital_radar_data::Message::other_data_blocks)
    /// and unrecognized `'R'` blocks are skipped. Converting one of those other data blocks to
    /// moment data with `DataBlock::moment_data` also returns it, since no model product matches.
    #[error(
        "unknown data block name {:?} at offset {offset}",
        String::from_utf8_lossy(.block_type)
    )]
    UnknownDataBlockType {
        /// The unrecognized data block name.
        block_type: [u8; 3],
        /// Offset of the data block's identifier.
        offset: usize,
    },
    /// A segment of a multi-segment message is missing or out of order.
    #[error(
        "expected segment {expected_segment} of {segment_count} of message type {} at offset {offset}",
        .message_type.number()
    )]
    SegmentOutOfSequence {
        /// The type of the multi-segment message being assembled.
        message_type: MessageType,
        /// The segment number that was expected next.
        expected_segment: u16,
        /// The number of segments the message declares.
        segment_count: u16,
        /// Offset where the expected segment should begin.
        offset: usize,
    },
    /// No volume coverage pattern message was found while assembling a scan.
    #[cfg(feature = "model")]
    #[error("no volume coverage pattern message found")]
    MissingCoveragePattern,
    /// Decoded values were rejected by a [`nexrad_model`] constructor, such as moment data
    /// whose gate count does not match its encoded data or a radial with an out-of-range
    /// azimuth.
    #[cfg(feature = "model")]
    #[error("decoded data is invalid for the model")]
    Model(#[from] nexrad_model::result::Error),
}
