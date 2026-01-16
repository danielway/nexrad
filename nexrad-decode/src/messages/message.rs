use crate::messages::{
    digital_radar_data, rda_status_data, volume_coverage_pattern, MessageContents, MessageHeader,
    MessageType,
};
use crate::result::{Error, Result};
use crate::slice_reader::SliceReader;

/// Expected segment contents size for fixed-length segments.
const FIXED_SEGMENT_SIZE: usize = 2432 - size_of::<MessageHeader>();

/// A decoded NEXRAD Level II message with its metadata header.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    header: &'a MessageHeader,
    contents: MessageContents<'a>,
    offset: usize,
    size: usize,
}

impl<'a> Message<'a> {
    pub(crate) fn parse(reader: &mut SliceReader<'a>) -> Result<Self> {
        let offset = reader.position();
        let header = reader.take_ref::<MessageHeader>()?;

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
            header,
            contents,
            offset,
            size,
        })
    }

    /// This message's header.
    pub fn header(&self) -> &MessageHeader {
        self.header
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
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The total size of this message in bytes, including the header.
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
