//!
//! Decompression functions for compressed LDM records containing radar data in Archive II files.
//!

use crate::decode::{
    decode_archive2_header, decode_clutter_filter_map, decode_digital_radar_data,
    decode_message_header, decode_rda_status_message,
};
use crate::model::messages::MessageWithHeader;
use crate::model::messages::{Message, MessageType};
use crate::model::Archive2File;
use crate::result::Result;
use bzip2::read::BzDecoder;
use std::io::{Cursor, Read, Seek, SeekFrom};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// Decompresses and decodes an Archive II file from the provided reader.
pub fn decompress_and_decode_archive2_file<R: Read + Seek>(
    reader: &mut R,
    size: u64,
) -> Result<Archive2File> {
    let archive2_header = decode_archive2_header(reader)?;

    let mut compressed_records = Vec::new();
    loop {
        let position = reader.stream_position()?;
        if position >= size {
            break;
        }
        
        let mut record_size = [0; 4];
        reader.read_exact(&mut record_size)?;
        let record_size = i32::from_be_bytes(record_size).abs();
        reader.seek(SeekFrom::Current(-4))?;
        
        let mut compressed_data = vec![0; record_size as usize + 4];
        reader.read_exact(&mut compressed_data)?;
        
        compressed_records.push(compressed_data);
    }
    
    let messages = decompress_and_decode_messages(compressed_records)?;

    Ok(Archive2File {
        header: archive2_header,
        messages,
    })
}

#[cfg(not(feature = "rayon"))]
fn decompress_and_decode_messages(compressed_records: Vec<Vec<u8>>) -> Result<Vec<MessageWithHeader>> {
    compressed_records
        .iter()
        .map(|compressed_data| decompress_and_decode_message(&mut compressed_data.as_slice()))
        .collect::<Result<Vec<MessageWithHeader>>>()
}

#[cfg(feature = "rayon")]
fn decompress_and_decode_messages(compressed_records: Vec<Vec<u8>>) -> Result<Vec<MessageWithHeader>> {
    compressed_records
        .par_iter()
        .map(|compressed_data| decompress_and_decode_message(&mut compressed_data.as_slice()))
        .collect::<Result<Vec<MessageWithHeader>>>()
}

/// Decompresses and decodes a message from the provided reader.
pub fn decompress_and_decode_message<R: Read>(reader: &mut R) -> Result<MessageWithHeader> {
    let decompressed_data = decompress_ldm_record(reader)?;
    let mut cursor = Cursor::new(decompressed_data.as_slice());
    
    let header = decode_message_header(&mut cursor)?;

    let message = match header.message_type() {
        MessageType::RDAStatusData => {
            Message::RDAStatusData(decode_rda_status_message(&mut cursor)?)
        }
        MessageType::RDADigitalRadarDataGenericFormat => {
            Message::DigitalRadarData(decode_digital_radar_data(&mut cursor)?)
        }
        MessageType::RDAClutterFilterMap => {
            Message::ClutterFilterMap(decode_clutter_filter_map(&mut cursor)?)
        }
        _ => Message::Other,
    };
    
    Ok(MessageWithHeader { header, message })
}

/// Decompresses an LDM record from the provided reader.
pub fn decompress_ldm_record<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut record_size = [0; 4];
    reader.read_exact(&mut record_size)?;
    let record_size = i32::from_be_bytes(record_size).abs();

    let mut decompressed_data = Vec::new();
    BzDecoder::new(reader.take(record_size as u64)).read_to_end(&mut decompressed_data)?;

    Ok(decompressed_data)
}
