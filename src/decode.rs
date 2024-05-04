//!
//! TODO
//!

use crate::model::messages::digital_radar_data;
use crate::model::messages::message_header::MessageHeader;
use crate::model::messages::rda_status_data;
use crate::model::Archive2Header;
use crate::result::Result;
use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;
use std::io::Read;

/// Decodes an Archive II header from the provided reader.
pub fn decode_archive2_header<R: Read>(
    reader: &mut R,
) -> Result<Archive2Header> {
    deserialize(reader)
}

/// Decodes a message header from the provided reader.
pub fn decode_message_header<R: Read>(
    reader: &mut R,
) -> Result<MessageHeader> {
    deserialize(reader)
}

/// Decodes an RDA status message type 2 from the provided reader.
pub fn decode_rda_status_message<R: Read>(
    reader: &mut R,
) -> Result<rda_status_data::Message> {
    deserialize(reader)
}

/// Decodes a digital radar data message type 31 from the provided reader.
pub fn decode_digital_radar_data<R: Read>(
    reader: &mut R,
) -> Result<digital_radar_data::DataHeaderBlock> {
    deserialize(reader)
}

/// Attempts to deserialize some struct from the provided binary reader.
fn deserialize<R: Read, S: DeserializeOwned>(
    reader: &mut R,
) -> Result<S> {
    Ok(DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
        .deserialize_from(reader.by_ref())?)
}
