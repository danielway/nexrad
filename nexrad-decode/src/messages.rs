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

use crate::messages::clutter_filter_map::decode_clutter_filter_map;
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
pub fn decode_messages<R: Read + Seek>(reader: &mut R, data_length: u64) -> Result<Vec<Message>> {
    debug!("Decoding messages");

    let mut messages = Vec::new();
    while reader.stream_position()? < data_length {
        messages.push(decode_message(reader)?);
    }

    debug!(
        "Decoded {} messages ending at {:?}",
        messages.len(),
        reader.stream_position()
    );

    Ok(messages)
}

/// Decode a NEXRAD Level II message from a reader. Note that segmented messages will be read fully
/// from the reader.
pub fn decode_message<R: Read + Seek>(reader: &mut R) -> Result<Message> {
    let position = reader.stream_position()?;
    trace!("Decoding message header at {:?}", position);

    let header = decode_message_header(reader)?;
    let decoding_function = match header.message_type() {
        MessageType::RDADigitalRadarDataGenericFormat => decode_variable_length_message,
        _ => decode_fixed_length_message,
    };

    decoding_function(reader, header)
}

/// Decodes a variable-length message, such as message type 31 "Digital radar data".
fn decode_variable_length_message<R: Read + Seek>(
    reader: &mut R,
    header: MessageHeader,
) -> Result<Message> {
    if header.message_type() != MessageType::RDADigitalRadarDataGenericFormat {
        return Ok(Message::header_only(header));
    }

    trace!("Decoding digital radar data message (type 31)");
    let radar_data_message = decode_digital_radar_data(reader)?;
    Ok(Message::unsegmented(
        header,
        MessageBody::DigitalRadarData(Box::new(radar_data_message)),
    ))
}

/// Decodes a fixed-length message, such as message type 2 "RDA status data".
fn decode_fixed_length_message<R: Read + Seek>(
    reader: &mut R,
    header: MessageHeader,
) -> Result<Message> {
    let mut message_buffer = [0; 2432 - size_of::<MessageHeader>()];
    reader.read_exact(&mut message_buffer)?;

    let message_reader = &mut message_buffer.as_ref();
    let message_body = match header.message_type() {
        MessageType::RDAStatusData => {
            trace!("Decoding RDA status message (type 2)");
            MessageBody::RDAStatusData(Box::new(decode_rda_status_message(message_reader)?))
        }
        MessageType::RDAClutterFilterMap => {
            trace!("Decoding clutter filter map message (type 15)");
            MessageBody::ClutterFilterMap(Box::new(decode_clutter_filter_map(reader)?))
        }
        _ => {
            return Ok(Message::header_only(header));
        }
    };

    Ok(Message::unsegmented(header, message_body))
}
