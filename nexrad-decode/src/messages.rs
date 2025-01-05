pub mod clutter_filter_map;
pub mod digital_radar_data;
pub mod message_header;
pub mod rda_status_data;
pub mod volume_coverage_pattern;

mod message_type;
pub use message_type::MessageType;

mod message;
pub use message::{Message, MessageContents};

mod definitions;
mod primitive_aliases;

use crate::messages::digital_radar_data::decode_digital_radar_data;
use crate::messages::message_header::MessageHeader;
use crate::messages::rda_status_data::decode_rda_status_message;
use crate::messages::volume_coverage_pattern::decode_volume_coverage_pattern;
use crate::result::Result;
use crate::util::deserialize;
use log::{debug, trace};
use std::io::{Read, Seek};

/// Decode a series of NEXRAD Level II messages from a reader.
pub fn decode_messages<R: Read + Seek>(reader: &mut R) -> Result<Vec<Message>> {
    debug!("Decoding messages");

    let mut messages = Vec::new();

    // TODO: scrutinize this error more
    while let Ok(message) = decode_message(reader) {
        messages.push(message);
    }

    debug!(
        "Decoded {} messages ending at {:?}",
        messages.len(),
        reader.stream_position()
    );

    Ok(messages)
}

/// Decode a NEXRAD Level II message from a reader.
pub fn decode_message<R: Read + Seek>(reader: &mut R) -> Result<Message> {
    let header = decode_message_header(reader)?;
    let contents = decode_message_contents(reader, header.message_type())?;
    Ok(Message::unsegmented(header, contents))
}

/// Decode a NEXRAD Level II message header from a reader.
pub fn decode_message_header<R: Read>(reader: &mut R) -> Result<MessageHeader> {
    deserialize(reader)
}

/// Decode the content of a NEXRAD Level II message of the specified type from a reader.
pub fn decode_message_contents<R: Read + Seek>(
    reader: &mut R,
    message_type: MessageType,
) -> Result<MessageContents> {
    let position = reader.stream_position();
    trace!("Decoding message type {:?} at {:?}", message_type, position);

    if message_type == MessageType::RDADigitalRadarDataGenericFormat {
        let radar_data_message = decode_digital_radar_data(reader)?;
        return Ok(MessageContents::DigitalRadarData(Box::new(
            radar_data_message,
        )));
    }

    let mut message_buffer = [0; 2432 - size_of::<MessageHeader>()];
    reader.read_exact(&mut message_buffer)?;

    let contents_reader = &mut message_buffer.as_ref();
    Ok(match message_type {
        MessageType::RDAStatusData => {
            MessageContents::RDAStatusData(Box::new(decode_rda_status_message(contents_reader)?))
        }
        MessageType::RDAVolumeCoveragePattern => MessageContents::VolumeCoveragePattern(Box::new(
            decode_volume_coverage_pattern(contents_reader)?,
        )),
        // TODO: this message type is segmented which is not supported well currently
        // MessageType::RDAClutterFilterMap => {
        //     Message::ClutterFilterMap(Box::new(decode_clutter_filter_map(message_reader)?))
        // }
        _ => MessageContents::Other,
    })
}
