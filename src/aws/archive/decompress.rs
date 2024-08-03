use crate::result::Result;
use bzip2::read::BzDecoder;
use std::io::Read;

/// Decompresses a BZIP2-compressed LDM record, returning the decompressed data.
pub fn decompress_ldm_record(record_data: &[u8]) -> Result<Vec<u8>> {
    let mut decompressed_data = Vec::new();
    BzDecoder::new(record_data.iter().as_slice()).read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}
