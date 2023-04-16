//!
//! Provides utilities like [decompress_chunk] for decompressing BZIP2-compressed NEXRAD chunk data.
//!

use crate::chunk::EncodedChunk;
use crate::result::Result;

/// Given a compressed chunk, decompresses it and returns a new copy of the chunk with the 
/// decompressed data. Will fail if the chunk is already decompressed.
pub fn decompress_chunk(_chunk: &EncodedChunk) -> Result<EncodedChunk> {
    todo!()
}