//!
//! Provides utilities like [decompress_chunk] for decompressing BZIP2-compressed NEXRAD chunk data.
//!

use std::io::Read;

use crate::chunk::{EncodedChunk, FileHeader};
use crate::file::is_compressed;
use crate::result::{Error, Result};

/// Given a compressed chunk, decompresses it and returns a new copy of the chunk with the
/// decompressed data. Will fail if the chunk is already decompressed.
pub fn decompress_chunk(chunk: &EncodedChunk) -> Result<EncodedChunk> {
    if !is_compressed(chunk.data()) {
        return Err(Error::DecompressionError("Cannot decompress uncompressed chunk".into()));
    };

    let mut decompressed_buffer = Vec::new();

    // Start the decompressed data by copying the file header, which is not compressed
    let header_size = std::mem::size_of::<FileHeader>();
    let (header, mut reader) = chunk.data().as_slice().split_at(header_size);
    decompressed_buffer.extend_from_slice(&header);

    loop {
        // Skip the first 4 bytes of the compressed block, which is the size of the block
        reader = reader.split_at(4).1;

        let mut decoder = bzip2::read::BzDecoder::new(reader);

        // Read the decompressed block into a buffer
        let mut block_buffer = Vec::new();
        decoder.read_to_end(&mut block_buffer)?;

        // Advance the reader to the next compressed block
        reader = reader.split_at(decoder.total_in() as usize).1;

        // Append the decompressed block to the decompressed data
        decompressed_buffer.extend(block_buffer);

        if reader.len() == 0 {
            break;
        }
    }

    Ok(EncodedChunk::new(decompressed_buffer))
}