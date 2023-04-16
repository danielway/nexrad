//!
//! Provides utilities like [decode_chunk] for decoding NEXRAD chunk data.
//!

use crate::chunk::{Chunk, EncodedChunk};
use crate::result::Result;

/// Given a chunk, decodes it and returns the decoded structure. If [decompress] is true it will
/// decompress compressed chunks, otherwise it will fail if the chunk is compressed.
pub fn decode_chunk(_chunk: &EncodedChunk, _decompress: bool) -> Result<Chunk> {
    todo!()
}