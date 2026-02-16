use crate::messages::{
    clutter_filter_map, rda_status_data, volume_coverage_pattern, MessageContents, MessageHeader,
    MessageType,
};
use crate::result::Result;
use crate::segmented_slice_reader::SegmentedSliceReader;

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
    /// Create a new message from pre-parsed components.
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

/// Decode the content of a fixed-segment NEXRAD Level II message.
///
/// This is the single dispatch table for all fixed-segment message types, both
/// single-segment (e.g. RDA Status, VCP) and multi-segment (e.g. Clutter Filter Map).
/// All use `SegmentedSliceReader` — for single-segment messages this is simply a
/// one-element reader that behaves identically to a contiguous slice.
pub(super) fn decode_fixed_segment_contents<'a>(
    reader: &mut SegmentedSliceReader<'a>,
    message_type: MessageType,
) -> Result<MessageContents<'a>> {
    Ok(match message_type {
        MessageType::RDAStatusData => {
            let rda_status_message = rda_status_data::Message::parse(reader)?;
            MessageContents::RDAStatusData(Box::new(rda_status_message))
        }
        MessageType::RDAVolumeCoveragePattern => {
            let volume_coverage_message = volume_coverage_pattern::Message::parse(reader)?;
            MessageContents::VolumeCoveragePattern(Box::new(volume_coverage_message))
        }
        MessageType::RDAClutterFilterMap => {
            let clutter_filter_message = clutter_filter_map::Message::parse(reader)?;
            MessageContents::ClutterFilterMap(Box::new(clutter_filter_message))
        }
        _ => MessageContents::Other,
    })
}
