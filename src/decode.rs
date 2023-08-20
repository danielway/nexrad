//!
//! Provides utilities like [decode_chunk] for decoding NEXRAD chunk data.
//!

use std::io::Read;

use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;

use crate::chunk::{Chunk, FileHeader, MessageHeader};
use crate::result::Result;

/// Given a chunk, decodes it and returns the decoded structure.
pub fn decode_chunk(reader: &[u8]) -> Result<Chunk> {
    let file_header: FileHeader = deserialize(reader)?;

    loop {
        let message_header: MessageHeader = deserialize(reader)?;

        println!("Message type: {}", message_header.msg_type);

        break;
    }

    Ok(Chunk::new(file_header))
}

/// Given a chunk, decodes and returns just the file header.
pub fn decode_file_header(data: &[u8]) -> Result<FileHeader> {
    Ok(deserialize(data)?)
}

/// Attempts to deserialize some struct from the provided binary reader.
fn deserialize<R: Read, S: DeserializeOwned>(t: R) -> Result<S> {
    Ok(DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
        .deserialize_from(t)?)
}
