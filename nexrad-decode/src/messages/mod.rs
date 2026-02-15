pub mod clutter_filter_map;
pub mod digital_radar_data;
pub mod rda_status_data;
pub mod volume_coverage_pattern;

mod raw;
pub(crate) use raw::primitive_aliases;
pub use raw::{MessageHeader, MessageType, RedundantChannel};

mod message;
pub use message::{Message, MessageHeaders};

mod message_contents;
pub use message_contents::MessageContents;

use crate::result::{Error, Result};
use crate::segmented_slice_reader::SegmentedSliceReader;
use crate::slice_reader::SliceReader;
use log::{trace, warn};

/// Decode a series of NEXRAD Level II messages from a reader.
///
/// Segmented messages (like Clutter Filter Map) are accumulated and returned
/// as a single logical message with multiple headers. Segments are assumed to
/// appear consecutively in the input.
pub fn decode_messages<'a>(input: &'a [u8]) -> Result<Vec<Message<'a>>> {
    trace!("Decoding messages");

    let mut reader = SliceReader::new(input);
    let mut messages = Vec::new();

    // Accumulator for in-progress segmented message (segments are consecutive)
    let mut segment_headers: Vec<&'a MessageHeader> = Vec::new();
    let mut segment_payloads: Vec<&'a [u8]> = Vec::new();
    let mut segment_first_offset: usize = 0;
    let mut segment_expected_count: u16 = 0;

    while reader.remaining().len() >= size_of::<MessageHeader>() {
        let offset = reader.position();

        // Read the header (used for both segmented and non-segmented paths)
        let header = match reader.take_ref::<MessageHeader>() {
            Ok(h) => h,
            Err(_) => break,
        };

        // Check if this message uses the segmented path (multi-segment, or a message
        // type that always requires a SegmentedSliceReader even with one segment).
        let uses_segmented_path = header.segment_count().is_some_and(|c| c > 1)
            || header.message_type() == MessageType::RDAClutterFilterMap;

        if uses_segmented_path {
            trace!(
                "Segmented message: type={:?}, segment={:?}/{:?}",
                header.message_type(),
                header.segment_number(),
                header.segment_count()
            );

            let segment_num = header.segment_number().unwrap_or(1);
            let segment_count = header.segment_count().unwrap_or(1);

            // Calculate payload size (segment_size is in half-words)
            let payload_size =
                (header.message_size_bytes() as usize).saturating_sub(size_of::<MessageHeader>());
            let payload = match reader.take_bytes(payload_size) {
                Ok(p) => p,
                Err(_) => break,
            };

            // Skip padding to reach 2432-byte segment boundary
            let segment_content_size = size_of::<MessageHeader>() + payload_size;
            if segment_content_size < 2432 {
                reader.advance(2432 - segment_content_size);
            }

            if segment_num == 1 {
                // Start new segmented message
                segment_headers.clear();
                segment_payloads.clear();
                segment_first_offset = offset;
                segment_expected_count = segment_count;
            }

            segment_headers.push(header);
            segment_payloads.push(payload);

            // Check if complete
            if segment_headers.len() as u16 >= segment_expected_count {
                match parse_segmented_message(
                    &segment_headers,
                    &segment_payloads,
                    segment_first_offset,
                    &reader,
                ) {
                    Ok(message) => {
                        trace!(
                            "Parsed segmented message with {} headers",
                            message.headers().count()
                        );
                        messages.push(message);
                    }
                    Err(e) => {
                        warn!("Failed to parse segmented message: {:?}", e);
                    }
                }
                segment_headers.clear();
                segment_payloads.clear();
            }
        } else {
            // Non-segmented message
            match Message::parse(&mut reader, header, offset) {
                Ok(message) => messages.push(message),
                Err(e) => {
                    if matches!(e, Error::UnexpectedEof) {
                        break;
                    }

                    warn!(
                        "Failed to parse message (type {:?}) at offset {}: {:?}",
                        header.message_type(),
                        offset,
                        e
                    );

                    if !skip_to_message_end(&mut reader, header, offset) {
                        break;
                    }

                    continue;
                }
            }
        }
    }

    // Warn about incomplete segmented message
    if !segment_headers.is_empty() {
        warn!(
            "Incomplete segmented message: got {}/{} segments",
            segment_headers.len(),
            segment_expected_count
        );
    }

    let bytes_remaining = input.len().saturating_sub(reader.position());
    if bytes_remaining > 0 {
        warn!(
            "Bytes remaining after decoding all messages: {}",
            bytes_remaining
        );
    }

    Ok(messages)
}

/// Advance the reader past the current message to the next boundary.
///
/// For fixed-segment messages, this skips to the 2432-byte boundary.
/// For variable-length digital radar data, this uses the header's message size.
/// Returns `false` if there isn't enough data remaining (caller should break).
fn skip_to_message_end(
    reader: &mut SliceReader<'_>,
    header: &MessageHeader,
    offset: usize,
) -> bool {
    let target_pos = if header.message_type() == MessageType::RDADigitalRadarDataGenericFormat {
        offset + header.message_size_bytes() as usize
    } else {
        offset + size_of::<MessageHeader>() + message::FIXED_SEGMENT_SIZE
    };

    if target_pos > reader.position() {
        let skip = target_pos - reader.position();
        if skip <= reader.remaining().len() {
            reader.advance(skip);
        } else {
            return false;
        }
    }

    true
}

/// Parse a complete segmented message from accumulated segments.
fn parse_segmented_message<'a>(
    headers: &[&'a MessageHeader],
    payloads: &[&'a [u8]],
    offset: usize,
    reader: &SliceReader<'_>,
) -> Result<Message<'a>> {
    let message_type = headers[0].message_type();

    let total_size: usize = headers.len() * size_of::<MessageHeader>()
        + payloads.iter().map(|p| p.len()).sum::<usize>();

    let mut segmented_reader = SegmentedSliceReader::new(payloads.to_vec());

    // Propagate build number for version-aware parsing
    if let Some(bn) = reader.build_number() {
        segmented_reader.set_build_number(bn);
    }

    let contents = match message_type {
        MessageType::RDAClutterFilterMap => {
            let clutter_filter_message = clutter_filter_map::Message::parse(&mut segmented_reader)?;
            MessageContents::ClutterFilterMap(Box::new(clutter_filter_message))
        }
        _ => {
            // For other segmented types we don't handle yet, return Other
            warn!("Unhandled segmented message type: {:?}", message_type);
            MessageContents::Other
        }
    };

    Ok(Message::new(
        MessageHeaders::Multiple(headers.to_vec()),
        contents,
        offset,
        total_size,
    ))
}
