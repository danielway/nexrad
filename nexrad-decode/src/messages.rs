pub mod clutter_filter_map;
pub mod digital_radar_data;
pub mod message_header;
pub mod rda_status_data;

mod message_type;
pub use message_type::MessageType;

mod message;
pub use message::{Message, MessageBody};

mod definitions;
mod primitive_aliases;

use crate::messages::digital_radar_data::decode_digital_radar_data;
use crate::messages::message_header::MessageHeader;
use crate::messages::rda_status_data::decode_rda_status_message;
use crate::result::Result;
use crate::util::deserialize;
use log::{debug, trace};
use std::io::{Read, Seek};

/// Decode a NEXRAD Level II message from a reader.
pub fn decode_message_header<R: Read>(reader: &mut R) -> Result<MessageHeader> {
    deserialize(reader)
}

/// Decode a series of NEXRAD Level II messages from a reader.
pub fn decode_messages<R: Read + Seek>(reader: &mut R) -> Result<Vec<Message>> {
    debug!("Decoding messages");

    let mut messages = Vec::new();
    while let Ok(header) = decode_message_header(reader) {
        let message = decode_message(reader, header.message_type())?;
        messages.push(Message::new(header, message));
    }

    debug!(
        "Decoded {} messages ending at {:?}",
        messages.len(),
        reader.stream_position()
    );

    Ok(messages)
}

/// Decode a NEXRAD Level II message of the specified type from a reader. Note that segmented
/// messages will be read fully from the reader.
pub fn decode_message<R: Read + Seek>(
    reader: &mut R,
    message_type: MessageType,
) -> Result<MessageBody> {
    let position = reader.stream_position();
    trace!("Decoding message type {:?} at {:?}", message_type, position);

    if message_type == MessageType::RDADigitalRadarDataGenericFormat {
        let decoded_message = decode_digital_radar_data(reader)?;
        return Ok(MessageBody::DigitalRadarData(Box::new(decoded_message)));
    }

    let mut message_buffer = [0; 2432 - size_of::<MessageHeader>()];
    reader.read_exact(&mut message_buffer)?;

    let message_reader = &mut message_buffer.as_ref();
    Ok(match message_type {
        MessageType::RDAStatusData => {
            MessageBody::RDAStatusData(Box::new(decode_rda_status_message(message_reader)?))
        }
        // TODO: this message type is segmented which is not supported well currently
        // MessageType::RDAClutterFilterMap => {
        //     Message::ClutterFilterMap(Box::new(decode_clutter_filter_map(message_reader)?))
        // }
        _ => MessageBody::Other,
    })
}
