//!
//! Decompression functions for compressed LDM records containing radar data in Archive II files.
//!

use crate::decode::{decode_archive2_header, decode_digital_radar_data, decode_message_header};
use crate::model::messages::message_header::MessageHeader;
use crate::model::messages::MessageWithHeader;
use crate::model::messages::{Message, MessageType};
use crate::model::Archive2File;
use crate::result::Result;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::mem::size_of;

#[cfg(not(feature = "decompress-wasm"))]
use bzip2::read::BzDecoder;

#[cfg(feature = "decompress-wasm")]
use bzip2_rs::DecoderReader;

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
fn decompress_and_decode_messages(
    compressed_records: Vec<Vec<u8>>,
) -> Result<Vec<MessageWithHeader>> {
    let decoded_messages = compressed_records
        .iter()
        .map(|compressed_data| decompress_and_decode_message(&mut compressed_data.as_slice()))
        .collect::<Result<Vec<Vec<MessageWithHeader>>>>()?;

    Ok(decoded_messages
        .into_iter()
        .flatten()
        .collect::<Vec<MessageWithHeader>>())
}

#[cfg(feature = "rayon")]
fn decompress_and_decode_messages(
    compressed_records: Vec<Vec<u8>>,
) -> Result<Vec<MessageWithHeader>> {
    let decoded_messages = compressed_records
        .par_iter()
        .map(|compressed_data| decompress_and_decode_message(&mut compressed_data.as_slice()))
        .collect::<Result<Vec<Vec<MessageWithHeader>>>>()?;

    Ok(decoded_messages
        .into_iter()
        .flatten()
        .collect::<Vec<MessageWithHeader>>())
}

/// Decompresses and decodes a message from the provided reader.
pub fn decompress_and_decode_message<R: Read>(reader: &mut R) -> Result<Vec<MessageWithHeader>> {
    let decompressed_data = decompress_ldm_record(reader)?;
    let mut cursor = Cursor::new(decompressed_data.as_slice());

    let mut messages = Vec::new();
    while cursor.position() < decompressed_data.len() as u64 {
        let header = decode_message_header(&mut cursor)?;

        let message = match header.message_type() {
            // todo: this is reading more data than it should be. needs to be debugged.
            // MessageType::RDAStatusData => {
            //     Message::RDAStatusData(decode_rda_status_message(&mut cursor)?)
            // }
            MessageType::RDADigitalRadarDataGenericFormat => {
                Message::DigitalRadarData(decode_digital_radar_data(&mut cursor)?)
            }
            // todo: this is reading more data than it should be. needs to be debugged.
            // MessageType::RDAClutterFilterMap => {
            //     Message::ClutterFilterMap(decode_clutter_filter_map(&mut cursor)?)
            // }
            _ => {
                let fast_forward_distance = 2432 - size_of::<MessageHeader>();
                cursor.seek(SeekFrom::Current(fast_forward_distance as i64))?;
                Message::Other
            }
        };

        messages.push(MessageWithHeader { header, message });
    }

    Ok(messages)
}

/// Decompresses an LDM record from the provided reader.
pub fn decompress_ldm_record<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut record_size = [0; 4];
    reader.read_exact(&mut record_size)?;
    let record_size = i32::from_be_bytes(record_size).abs();

    let mut decompressed_data = Vec::new();
    decompress(&mut reader.take(record_size as u64), &mut decompressed_data)?;

    Ok(decompressed_data)
}

#[cfg(not(feature = "decompress-wasm"))]
fn decompress<R: Read>(reader: &mut R, decompressed_data: &mut Vec<u8>) -> Result<()> {
    BzDecoder::new(reader).read_to_end(decompressed_data)?;
    Ok(())
}

#[cfg(feature = "decompress-wasm")]
fn decompress<R: Read>(reader: &mut R, decompressed_data: &mut Vec<u8>) -> Result<()> {
    DecoderReader::new(reader).read_to_end(decompressed_data)?;
    Ok(())
}
