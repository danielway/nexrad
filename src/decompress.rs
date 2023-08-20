//!
//! Provides utilities like [decompress_file] for decompressing BZIP2-compressed NEXRAD data.
//!

use crate::file::is_compressed;
use crate::model::FileHeader;
use crate::result::{Error, Result};
use std::io::Read;

/// Given a compressed data file, decompresses it and returns a new copy of the decompressed data.
/// Will fail if the file is already decompressed.
pub fn decompress_file(data: &[u8]) -> Result<Vec<u8>> {
    if !is_compressed(data) {
        return Err(Error::DecompressionError(
            "Cannot decompress uncompressed data".into(),
        ));
    };

    let mut decompressed_buffer = Vec::new();

    // Start the decompressed data by copying the file header, which is not compressed
    let header_size = std::mem::size_of::<FileHeader>();
    let (header, mut reader) = data.split_at(header_size);
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

    Ok(decompressed_buffer)
}
