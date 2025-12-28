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

use crate::result::Result;

/// Decode a series of NEXRAD Level II messages from a reader.
pub fn decode_messages<'a, 'b>(input: &'b mut &'a [u8]) -> Result<Vec<Message<'a>>> {
    let mut messages = Vec::new();
    while let Ok(message) = Message::parse(input) {
        messages.push(message);
    }

    Ok(messages)
}
