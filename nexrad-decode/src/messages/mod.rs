pub mod clutter_censor_zones;
pub mod clutter_filter_bypass_map;
pub mod clutter_filter_map;
pub mod console_message;
pub mod digital_radar_data;
pub mod digital_radar_data_legacy;
pub mod loopback_test;
pub mod performance_maintenance_data;
pub mod rda_adaptation_data;
pub mod rda_control_commands;
pub mod rda_log_data;
pub mod rda_prf_data;
pub mod rda_status_data;
pub mod request_for_data;
pub mod volume_coverage_pattern;

mod raw;
pub(crate) use raw::primitive_aliases;
pub use raw::RedundantChannel;

mod message_header;
pub use message_header::MessageHeader;

mod message_type;
pub use message_type::MessageType;

mod message;
pub use message::{Message, MessageHeaders};

mod message_contents;
pub use message_contents::MessageContents;

mod vcp_number;
pub use vcp_number::VcpNumber;

use crate::result::{Error, Result};
use crate::segmented_slice_reader::SegmentedSliceReader;
use crate::slice_reader::SliceReader;
use std::borrow::Cow;
use std::iter::FusedIterator;

/// Size of a single fixed-length message frame (header + content + padding).
const SEGMENT_FRAME_SIZE: usize = 2432;

/// Size of the leading bytes of [`MessageHeader`] that precede the ICD message header. The
/// message size a header declares counts the ICD header and the payload, but not these bytes.
const PRE_HEADER_SIZE: usize = 12;

/// The size of the payload following the header, from the message size the header declares.
fn declared_payload_size(header: &MessageHeader<'_>) -> usize {
    (header.message_size_bytes() as usize).saturating_sub(MessageHeader::SIZE - PRE_HEADER_SIZE)
}

/// Decode every NEXRAD Level II message in `data`, a sequence of messages such as one
/// decompressed Archive II record.
///
/// Decoding is strict: the result is either every message in `data` or the error for the first
/// message that cannot be decoded. Use [`iter_messages`] to keep the messages before a failure.
///
/// Segmented messages (like Clutter Filter Map) are returned as a single logical message with
/// one header per segment; their segments must appear consecutively and in order.
///
/// Archive II frames are zero-padded, so when fewer bytes remain than a message header needs
/// and all of them are zero, decoding ends cleanly.
///
/// # Errors
///
/// Returns the error for the first message that cannot be decoded:
///
/// - [`Error::UnexpectedEof`] if the input ends inside a message, or a tail shorter than a
///   message header contains a non-zero byte.
/// - [`Error::SegmentOutOfSequence`] if a segment of a multi-segment message is missing or out of
///   order.
/// - [`Error::InvalidDataBlockPointer`] or [`Error::UnknownDataBlockType`] if a digital radar
///   data message's data blocks cannot be located or identified.
pub fn decode_messages(data: &[u8]) -> Result<Vec<Message<'_>>> {
    iter_messages(data).collect()
}

/// Iterate over the NEXRAD Level II messages in `data`, decoding one message per step.
///
/// Each item is either a decoded message or the error for the message at the current offset,
/// following the same rules as [`decode_messages`]. After an error the iterator is finished,
/// since a message that cannot be decoded gives no reliable offset for the next one. Collecting
/// the iterator into a `Result<Vec<_>>` is equivalent to [`decode_messages`]; keeping the
/// messages before an error decodes leniently:
///
/// ```
/// use nexrad_decode::iter_messages;
///
/// # let data: &[u8] = &[];
/// let mut messages = Vec::new();
/// let mut iter = iter_messages(data);
/// while let Some(result) = iter.next() {
///     match result {
///         Ok(message) => messages.push(message),
///         Err(error) => eprintln!("stopped at offset {}: {error}", iter.offset()),
///     }
/// }
/// ```
pub fn iter_messages(data: &[u8]) -> MessageIter<'_> {
    MessageIter {
        data,
        offset: 0,
        finished: false,
    }
}

/// An iterator over the messages in a byte slice, created by [`iter_messages`].
///
/// Yields `Ok` for each decoded message and at most one `Err`, after which it is finished.
#[derive(Debug, Clone)]
pub struct MessageIter<'a> {
    data: &'a [u8],
    offset: usize,
    finished: bool,
}

impl<'a> MessageIter<'a> {
    /// The number of bytes consumed so far: the offset of the next message to decode.
    ///
    /// After the iterator returns an error, this is the offset of the message that could not be
    /// decoded. After it finishes cleanly, this is the length of the input.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Decode the message at the current offset, returning `None` at a clean end of input.
    fn decode_next(&mut self) -> Option<Result<Message<'a>>> {
        let remaining = self.data.get(self.offset..).unwrap_or_default();
        if remaining.len() < MessageHeader::SIZE {
            if remaining.iter().all(|&byte| byte == 0) {
                self.offset = self.data.len();
                return None;
            }
            return Some(Err(Error::UnexpectedEof {
                offset: self.offset,
            }));
        }

        let mut reader = SliceReader::at(self.data, self.offset);
        let result = decode_message(&mut reader);
        if result.is_ok() {
            self.offset = reader.position();
        }
        Some(result)
    }
}

impl<'a> Iterator for MessageIter<'a> {
    type Item = Result<Message<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let item = self.decode_next();
        if !matches!(item, Some(Ok(_))) {
            self.finished = true;
        }
        item
    }
}

impl FusedIterator for MessageIter<'_> {}

/// Decode the message starting at the reader's position, leaving the reader after it.
fn decode_message<'a>(reader: &mut SliceReader<'a>) -> Result<Message<'a>> {
    let offset = reader.position();
    let header = MessageHeader::from_raw(reader.take_ref::<raw::MessageHeader>()?);

    if header.segmented() {
        decode_segmented_message(reader, header, offset)
    } else {
        decode_variable_length_message(reader, header, offset)
    }
}

/// Decode a fixed-segment message, reading its continuation segments from the reader.
fn decode_segmented_message<'a>(
    reader: &mut SliceReader<'a>,
    header: MessageHeader<'a>,
    offset: usize,
) -> Result<Message<'a>> {
    let message_type = header.message_type();

    // Message type 0 is not assigned by the ICD. Frames carrying it are unused slots in the
    // Archive II metadata record, and some sites leave stale segment fields in them, so each
    // one is a message of its own.
    let segment_count = if message_type == MessageType::Unknown(0) {
        1
    } else {
        header.segment_count().unwrap_or(1)
    };
    let segment_number = header.segment_number().unwrap_or(1);

    if segment_count > 1 && segment_number != 1 {
        return Err(Error::SegmentOutOfSequence {
            message_type,
            expected_segment: 1,
            segment_count,
            offset,
        });
    }

    let mut payloads = vec![take_segment_payload(
        reader,
        &header,
        segment_count,
        offset,
    )?];
    let mut payload_offsets = vec![offset + MessageHeader::SIZE];
    let mut headers = vec![header];

    while headers.len() < usize::from(segment_count) {
        let segment_offset = reader.position();
        let expected_segment = headers.len() as u16 + 1;

        let segment_header = MessageHeader::from_raw(reader.take_ref::<raw::MessageHeader>()?);
        if segment_header.message_type() != message_type
            || segment_header.segment_number() != Some(expected_segment)
        {
            return Err(Error::SegmentOutOfSequence {
                message_type,
                expected_segment,
                segment_count,
                offset: segment_offset,
            });
        }

        payloads.push(take_segment_payload(
            reader,
            &segment_header,
            segment_count,
            segment_offset,
        )?);
        payload_offsets.push(segment_offset + MessageHeader::SIZE);
        headers.push(segment_header);
    }

    // A legacy (pre-ORDA) RDA sends a clutter filter notch width map as message type 15, whose
    // layout differs from the ORDA clutter filter map this crate decodes.
    let undecoded = message_type == MessageType::RdaClutterFilterMap
        && matches!(
            headers[0].redundant_channel(),
            RedundantChannel::LegacySingleChannel
                | RedundantChannel::LegacyRedundantChannel1
                | RedundantChannel::LegacyRedundantChannel2
        );

    let mut segmented_reader = SegmentedSliceReader::new(&payloads, &payload_offsets);
    let decoded = if undecoded {
        None
    } else {
        message::decode_fixed_segment_contents(&mut segmented_reader, message_type)?
    };
    let contents = match decoded {
        Some(contents) => contents,
        None => {
            let data = match payloads.as_slice() {
                [payload] => {
                    let declared = declared_payload_size(&headers[0]);
                    Cow::Borrowed(&payload[..declared.min(payload.len())])
                }
                payloads => Cow::Owned(payloads.concat()),
            };
            MessageContents::Other { message_type, data }
        }
    };

    let message_headers = if headers.len() == 1 {
        MessageHeaders::Single(headers.swap_remove(0))
    } else {
        MessageHeaders::Multiple(headers)
    };

    Ok(Message::new(
        message_headers,
        contents,
        offset,
        reader.position() - offset,
    ))
}

/// Take the payload of the segment whose header was just read, advancing the reader past the
/// segment's frame.
///
/// A segment of a multi-segment message carries its declared payload size, padded to the frame
/// size. A single-segment message's payload is the frame's full content area, so parsers can
/// read structures that extend beyond the declared message size.
fn take_segment_payload<'a>(
    reader: &mut SliceReader<'a>,
    header: &MessageHeader<'a>,
    segment_count: u16,
    frame_offset: usize,
) -> Result<&'a [u8]> {
    let content_size = SEGMENT_FRAME_SIZE - MessageHeader::SIZE;
    let eof = |_| Error::UnexpectedEof {
        offset: frame_offset,
    };

    if segment_count > 1 {
        let payload_size = declared_payload_size(header);
        let body = reader
            .take_bytes(payload_size.max(content_size))
            .map_err(eof)?;
        Ok(&body[..payload_size])
    } else {
        reader.take_bytes(content_size).map_err(eof)
    }
}

/// Decode a variable-length message (e.g. Type 31 Digital Radar Data).
fn decode_variable_length_message<'a>(
    reader: &mut SliceReader<'a>,
    header: MessageHeader<'a>,
    offset: usize,
) -> Result<Message<'a>> {
    let message_type = header.message_type();
    let contents = if message_type == MessageType::RdaDigitalRadarDataGenericFormat {
        let radar_data_message = digital_radar_data::Message::parse(reader)?;
        MessageContents::DigitalRadarData(Box::new(radar_data_message))
    } else {
        let payload_size = declared_payload_size(&header);
        let data = reader
            .take_bytes(payload_size)
            .map_err(|_| Error::UnexpectedEof { offset })?;
        MessageContents::Other {
            message_type,
            data: Cow::Borrowed(data),
        }
    };

    let size = reader.position() - offset;
    Ok(Message::new(
        MessageHeaders::Single(header),
        contents,
        offset,
        size,
    ))
}
