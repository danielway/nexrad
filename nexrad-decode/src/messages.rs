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
use crate::reader::SegmentedMessageReader;
use crate::result::Result;
use crate::util::deserialize;
use log::{debug, trace};
use std::io::Read;

/// Decode a series of NEXRAD Level II messages from a reader.
pub fn decode_messages<R: Read>(reader: &mut R) -> Result<Vec<Message>> {
    debug!("Decoding messages");

    let mut messages = Vec::new();

    // TODO: scrutinize this error more
    while let Ok(message) = decode_message(reader) {
        messages.push(message);
    }

    debug!("Decoded {} messages", messages.len());

    Ok(messages)
}

/// Decode a NEXRAD Level II message from a reader.
pub fn decode_message<R: Read>(reader: &mut R) -> Result<Message> {
    let (mut message_reader, message_type) = SegmentedMessageReader::new(reader)?;

    trace!("Decoding message type {:?}", message_type);
    let contents = match message_type {
        MessageType::RDADigitalRadarDataGenericFormat => MessageContents::DigitalRadarData(
            Box::new(decode_digital_radar_data(&mut message_reader)?),
        ),
        MessageType::RDAStatusData => MessageContents::RDAStatusData(Box::new(
            decode_rda_status_message(&mut message_reader)?,
        )),
        MessageType::RDAVolumeCoveragePattern => MessageContents::VolumeCoveragePattern(Box::new(
            decode_volume_coverage_pattern(&mut message_reader)?,
        )),
        // TODO: this message type is segmented which is not supported well currently
        // MessageType::RDAClutterFilterMap => {
        //     Message::ClutterFilterMap(Box::new(decode_clutter_filter_map(message_reader)?))
        // }
        _ => MessageContents::Other,
    };

    Ok(Message::new(message_reader.into_headers(), contents))
}

/// Decode a NEXRAD Level II message header from a reader.
pub fn decode_message_header<R: Read>(reader: &mut R) -> Result<MessageHeader> {
    deserialize(reader)
}
