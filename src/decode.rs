//!
//! Provides utilities like [decode_file] for decoding NEXRAD data.
//!

use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;

use bincode::{DefaultOptions, Options};
use serde::de::DeserializeOwned;

use crate::model::{DataFile, DataHeader, FileHeader, MessageHeader};
use crate::result::Result;

/// Given an uncompressed data file, decodes it and returns the decoded structure.
pub fn decode_file<R: Read + Seek>(reader: &mut R) -> Result<DataFile> {
    let file_header: FileHeader = decode_file_header(reader)?;
    println!("File header: {:?}", file_header);

    loop {
        let message_header: MessageHeader = deserialize(reader)?;
        println!("Message header: {:?}", message_header);

        if message_header.msg_type == 31 {
            let data_header: DataHeader = deserialize(reader)?;
            println!("Data header: {:?}", data_header);
        } else {
            let ff_distance = 2432 - size_of::<MessageHeader>();
            reader.seek(SeekFrom::Current(ff_distance as i64))?;
        }
    }

    Ok(DataFile::new(file_header))
}

/// Given a data file, decodes and returns just the file header.
pub fn decode_file_header<R: Read + Seek>(reader: &mut R) -> Result<FileHeader> {
    Ok(deserialize(reader)?)
}

/// Attempts to deserialize some struct from the provided binary reader.
fn deserialize<R: Read + Seek, S: DeserializeOwned>(reader: &mut R) -> Result<S> {
    Ok(DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
        .deserialize_from(reader.by_ref())?)
}
