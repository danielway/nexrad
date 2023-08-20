//!
//! Provides utilities like [decode_file] for decoding NEXRAD data.
//!

use std::io::Read;

use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;

use crate::model::{FileHeader, MessageHeader, VolumeScan};
use crate::result::Result;

/// Given an uncompressed data file, decodes it and returns the decoded structure.
pub fn decode_file(data: &[u8]) -> Result<VolumeScan> {
    let file_header: FileHeader = deserialize(data)?;

    loop {
        let message_header: MessageHeader = deserialize(data)?;

        println!("Message type: {}", message_header.msg_type);

        break;
    }

    Ok(VolumeScan::new(file_header))
}

/// Given a data file, decodes and returns just the file header.
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
