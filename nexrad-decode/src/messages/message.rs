use crate::messages::{
    digital_radar_data, rda_status_data, volume_coverage_pattern, MessageContents, MessageHeader,
    MessageType,
};
use crate::result::{Error, Result};
use crate::slice_reader::SliceReader;
use std::mem::size_of;

/// Expected segment contents size for fixed-length segments.
pub(crate) const FIXED_SEGMENT_SIZE: usize = 2432 - size_of::<MessageHeader>();

/// Container for message headers, supporting both single-segment and multi-segment messages.
///
/// Most NEXRAD messages are single-segment, but some message types (like Clutter Filter Map)
/// span multiple fixed-length segments. This enum provides a unified interface for accessing
/// headers in both cases.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageHeaders<'a> {
    /// A single-segment message (most common case).
    Single(&'a MessageHeader),
    /// A multi-segment message with headers from each segment.
    Multiple(Vec<&'a MessageHeader>),
}

impl<'a> MessageHeaders<'a> {
    /// Returns the primary header (first segment's header for segmented messages).
    ///
    /// For single-segment messages, this returns the only header.
    /// For multi-segment messages, this returns the first segment's header.
    pub fn primary(&self) -> &'a MessageHeader {
        match self {
            MessageHeaders::Single(h) => h,
            MessageHeaders::Multiple(headers) => headers[0],
        }
    }

    /// Returns an iterator over all headers.
    ///
    /// For single-segment messages, yields one header.
    /// For multi-segment messages, yields headers in segment order.
    pub fn iter(&self) -> impl Iterator<Item = &'a MessageHeader> + '_ {
        let slice: &[&'a MessageHeader] = match self {
            MessageHeaders::Single(h) => std::slice::from_ref(h),
            MessageHeaders::Multiple(headers) => headers.as_slice(),
        };
        slice.iter().copied()
    }

    /// Returns the number of segments (headers).
    pub fn count(&self) -> usize {
        match self {
            MessageHeaders::Single(_) => 1,
            MessageHeaders::Multiple(headers) => headers.len(),
        }
    }

    /// Returns true if this is a multi-segment message.
    pub fn is_segmented(&self) -> bool {
        matches!(self, MessageHeaders::Multiple(_))
    }
}

/// A decoded NEXRAD Level II message with its metadata header(s).
///
/// For most message types, this contains a single header and decoded contents.
/// For segmented message types (like Clutter Filter Map), this contains headers
/// from all segments that compose the logical message.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    headers: MessageHeaders<'a>,
    contents: MessageContents<'a>,
    offset: usize,
    size: usize,
}

impl<'a> Message<'a> {
    /// Parse a single (non-segmented) message from the reader.
    ///
    /// The header should already be read; `offset` is the position where it started.
    pub(crate) fn parse(
        reader: &mut SliceReader<'a>,
        header: &'a MessageHeader,
        offset: usize,
    ) -> Result<Self> {
        let contents_start = reader.position();
        let contents = decode_message_contents(reader, header.message_type())?;

        if header.message_type() != MessageType::RDADigitalRadarDataGenericFormat {
            let actual_length = reader.position() - contents_start;

            let length_delta: i32 = FIXED_SEGMENT_SIZE as i32 - actual_length as i32;
            if length_delta > 0 {
                reader.advance(length_delta as usize);
            } else if length_delta < 0 {
                return Err(Error::InvalidMessageLength {
                    message_type: format!("{:?}", header.message_type()),
                    delta: length_delta,
                });
            }
        }

        let size = reader.position() - offset;

        Ok(Message {
            headers: MessageHeaders::Single(header),
            contents,
            offset,
            size,
        })
    }

    /// Create a new message from pre-parsed components.
    ///
    /// Used by decode_messages() when constructing segmented messages.
    pub(crate) fn new(
        headers: MessageHeaders<'a>,
        contents: MessageContents<'a>,
        offset: usize,
        size: usize,
    ) -> Self {
        Message {
            headers,
            contents,
            offset,
            size,
        }
    }

    /// This message's primary header.
    ///
    /// For single-segment messages, returns the only header.
    /// For multi-segment messages, returns the first segment's header.
    pub fn header(&self) -> &MessageHeader {
        self.headers.primary()
    }

    /// All headers for this message.
    ///
    /// For single-segment messages, contains one header.
    /// For multi-segment messages, contains headers from all segments in order.
    pub fn headers(&self) -> &MessageHeaders<'a> {
        &self.headers
    }

    /// Whether this message spans multiple segments.
    pub fn is_segmented(&self) -> bool {
        self.headers.is_segmented()
    }

    /// This message's contents.
    pub fn contents(&self) -> &MessageContents<'_> {
        &self.contents
    }

    /// Consume this message, returning ownership of its contents.
    pub fn into_contents(self) -> MessageContents<'a> {
        self.contents
    }

    /// The byte offset where this message starts in the source data.
    ///
    /// For segmented messages, this is the offset of the first segment.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The total size of this message in bytes, including all headers.
    ///
    /// For segmented messages, this is the combined size of all segments.
    pub fn size(&self) -> usize {
        self.size
    }
}

/// Decode the content of a NEXRAD Level II message of the specified type from a reader.
fn decode_message_contents<'a>(
    reader: &mut SliceReader<'a>,
    message_type: MessageType,
) -> Result<MessageContents<'a>> {
    if message_type == MessageType::RDADigitalRadarDataGenericFormat {
        let radar_data_message = digital_radar_data::Message::parse(reader)?;
        return Ok(MessageContents::DigitalRadarData(Box::new(
            radar_data_message,
        )));
    }

    Ok(match message_type {
        MessageType::RDAStatusData => {
            let rda_status_message = rda_status_data::Message::parse(reader)?;

            // Capture build number for version-aware parsing of subsequent messages
            reader.set_build_number(rda_status_message.build_number());

            MessageContents::RDAStatusData(Box::new(rda_status_message))
        }
        MessageType::RDAVolumeCoveragePattern => {
            let volume_coverage_message = volume_coverage_pattern::Message::parse(reader)?;
            MessageContents::VolumeCoveragePattern(Box::new(volume_coverage_message))
        }
        MessageType::RDAClutterFilterMap => {
            // let clutter_filter_message = clutter_filter_map::Message::parse(reader)?;
            // MessageContents::ClutterFilterMap(Box::new(clutter_filter_message))
            MessageContents::Other
        }
        _ => MessageContents::Other,
    })
}
