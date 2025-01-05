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
use std::io::{self, Read};
use uom::si::information::byte;

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

struct SegmentedMessageReader<R> {
    inner: R,
    headers: Vec<MessageHeader>,
    current_segment_bytes_left: usize,
    message_finished: bool,
}

impl<R: Read> SegmentedMessageReader<R> {
    fn new(mut inner: R) -> Result<(Self, MessageType)> {
        let (header, segment_size_bytes, is_final_segment) =
            SegmentedMessageReader::decode_message_header(&mut inner)?;
        let message_type = header.message_type();

        let segment_count = header.segment_count().unwrap_or(1);
        let mut headers = Vec::with_capacity(segment_count as usize);
        headers[0] = header;

        Ok((
            SegmentedMessageReader {
                inner,
                headers,
                current_segment_bytes_left: segment_size_bytes,
                message_finished: is_final_segment,
            },
            message_type,
        ))
    }

    fn decode_message_header(reader: &mut R) -> Result<(MessageHeader, usize, bool)> {
        let header = decode_message_header(reader)?;
        let segment_size_bytes = header.message_size().get::<byte>() as usize;
        let is_final_segment = header.segment_number() == header.segment_count();
        Ok((header, segment_size_bytes, is_final_segment))
    }

    fn into_headers(self) -> Vec<MessageHeader> {
        self.headers
    }
}

impl<R: Read> Read for SegmentedMessageReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.message_finished {
            return Ok(0);
        }

        if self.current_segment_bytes_left == 0 {
            let (header, segment_size_bytes, is_final_segment) =
                SegmentedMessageReader::decode_message_header(&mut self.inner)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            self.current_segment_bytes_left = segment_size_bytes;
            self.message_finished = is_final_segment;
            self.headers.push(header);

            if self.current_segment_bytes_left == 0 && self.message_finished {
                return Ok(0);
            }
        }

        let to_read = self.current_segment_bytes_left.min(buf.len());
        let bytes_read = self.inner.read(&mut buf[..to_read])?;
        if bytes_read == 0 {
            // TODO: EOF
            return Ok(0);
        }

        self.current_segment_bytes_left -= bytes_read;

        Ok(bytes_read)
    }
}

/// Decode a NEXRAD Level II message from a reader.
pub fn decode_message<R: Read>(reader: &mut R) -> Result<Message> {
    let (mut message_reader, message_type) = SegmentedMessageReader::new(reader)?;
    let contents = decode_message_contents(&mut message_reader, message_type)?;
    Ok(Message::new(message_reader.into_headers(), contents))
}

/// Decode a NEXRAD Level II message header from a reader.
pub fn decode_message_header<R: Read>(reader: &mut R) -> Result<MessageHeader> {
    deserialize(reader)
}

/// Decode the content of a NEXRAD Level II message of the specified type from a reader.
pub fn decode_message_contents<R: Read>(
    reader: &mut R,
    message_type: MessageType,
) -> Result<MessageContents> {
    trace!("Decoding message type {:?}", message_type);

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
