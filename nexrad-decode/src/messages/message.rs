use crate::messages::{
    clutter_filter_map, digital_radar_data, rda_status_data, volume_coverage_pattern,
    MessageContents, MessageHeader, MessageType,
};
use crate::result::Result;
use crate::util::take_ref;

/// A decoded NEXRAD Level II message with its metadata header.
#[derive(Debug, Clone, PartialEq)]
pub struct Message<'a> {
    header: &'a MessageHeader,
    contents: MessageContents<'a>,
}

impl<'a> Message<'a> {
    // todo
    pub(crate) fn parse<'b>(input: &'b mut &'a [u8]) -> Result<Self> {
        let header = take_ref::<MessageHeader>(input)?;
        let contents = decode_message_contents(input, header.message_type())?;

        Ok(Message { header, contents })
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
}

/// Decode the content of a NEXRAD Level II message of the specified type from a reader.
fn decode_message_contents<'a>(
    input: &mut &'a [u8],
    message_type: MessageType,
) -> Result<MessageContents<'a>> {
    if message_type == MessageType::RDADigitalRadarDataGenericFormat {
        let radar_data_message = digital_radar_data::Message::parse(input)?;
        return Ok(MessageContents::DigitalRadarData(Box::new(
            radar_data_message,
        )));
    }

    let (mut message_buffer, remaining_input) = input.split_at(2432 - size_of::<MessageHeader>());
    *input = remaining_input;

    Ok(match message_type {
        MessageType::RDAStatusData => {
            let rda_status_message = rda_status_data::Message::parse(&mut message_buffer)?;
            MessageContents::RDAStatusData(Box::new(rda_status_message))
        }
        MessageType::RDAVolumeCoveragePattern => {
            let volume_coverage_message = volume_coverage_pattern::Message::parse(input)?;
            MessageContents::VolumeCoveragePattern(Box::new(volume_coverage_message))
        }
        MessageType::RDAClutterFilterMap => {
            let clutter_filter_message = clutter_filter_map::Message::parse(input)?;
            MessageContents::ClutterFilterMap(Box::new(clutter_filter_message))
        }
        _ => MessageContents::Other,
    })
}
