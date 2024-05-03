//!
//! TODO
//!

use crate::decode::{decode_archive2_header, decode_message_header};
use crate::model::messages::message_header::MessageHeader;
use crate::model::Archive2Header;
use crate::result::Result;
use bzip2::read::BzDecoder;
use std::io::{Read, Seek};
use uom::num_traits::abs;

#[derive(Debug)]
pub struct Archive2File {
    pub header: Archive2Header,
    pub records: Vec<MessageHeader>,
}

/// Decompresses and decodes an Archive II file from the provided reader.
pub fn decompress_and_decode_archive2_file<R: Read + Seek>(
    reader: &mut R,
    size: u64,
) -> Result<Archive2File> {
    let header = decode_archive2_header(reader)?;

    let mut records = Vec::new();
    while reader.stream_position()? < size {
        match decompress_ldm_record(reader) {
            Ok(decompressed_data) => {
                records.push(decode_message_header(&mut decompressed_data.as_slice())?)
            }
            Err(err) => panic!("Error decompressing LDM record: {:?}", err),
        }
    }

    Ok(Archive2File { header, records })
}

/// Decompresses an LDM record from the provided reader.
fn decompress_ldm_record<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut record_size = [0; 4];
    reader.read_exact(&mut record_size)?;
    let record_size = abs(i32::from_be_bytes(record_size));

    let mut decompressed_data = Vec::new();
    BzDecoder::new(reader.take(record_size as u64)).read_to_end(&mut decompressed_data)?;

    Ok(decompressed_data)
}
