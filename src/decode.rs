//!
//! Provides utilities like [decode_chunk] for decoding NEXRAD chunk data.
//!

use std::io::Read;

use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;

use crate::chunk::{Chunk, EncodedChunk, FileHeader, MessageHeader};
use crate::result::Result;

/// Given a chunk, decodes it and returns the decoded structure.
pub fn decode_chunk(encoded_chunk: &EncodedChunk) -> Result<Chunk> {
    let mut reader = encoded_chunk.data().as_slice();
    
    let file_header: FileHeader = deserialize(&mut reader)?;
    
    loop {
        let message_header: MessageHeader = deserialize(&mut reader)?;
        
        println!("Message type: {}", message_header.msg_type);
        
        break;
    }

    Ok(Chunk::new(
        encoded_chunk.meta().clone(),
        file_header,
    ))
}

/// Given a chunk, decodes and returns just the file header.
pub fn decode_file_header(chunk: &EncodedChunk) -> Result<FileHeader> {
    Ok(deserialize(chunk.data().as_slice())?)
}

/// Attempts to deserialize some struct from the provided binary reader.
fn deserialize<R: Read, S: DeserializeOwned>(t: R) -> Result<S> {
    Ok(DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
        .deserialize_from(t)?)
}