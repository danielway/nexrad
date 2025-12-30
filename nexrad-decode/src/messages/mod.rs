pub mod clutter_filter_map;
pub mod digital_radar_data;
pub mod rda_status_data;
pub mod volume_coverage_pattern;

mod raw;
pub use raw::*;

mod message;
pub use message::Message;

mod message_contents;
pub use message_contents::MessageContents;

use crate::{result::Result, slice_reader::SliceReader};
use log::{trace, warn};

/// Decode a series of NEXRAD Level II messages from a reader.
pub fn decode_messages<'a>(input: &mut &'a [u8]) -> Result<Vec<Message<'a>>> {
    trace!("Decoding messages");

    let mut reader = SliceReader::new(input);

    let mut messages = Vec::new();
    while let Ok(message) = Message::parse(&mut reader) {
        messages.push(message);
    }

    let bytes_remaining = input.len() - reader.position();
    if bytes_remaining > 0 {
        warn!(
            "Bytes remaining after decoding all messages: {}",
            bytes_remaining
        );
    }

    Ok(messages)
}
