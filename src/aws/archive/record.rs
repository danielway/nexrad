use crate::result::{Error, Result};
use bzip2::read::BzDecoder;
use std::io::Read;

enum LDMRecordData<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

/// Represents a single LDM record with its data which may be compressed.
/// 
/// The Unidata Local Data Manager (LDM) is a data distribution system used by the NWS to distribute
/// NEXRAD archival radar data. A NEXRAD "Archive II" file starts with an
/// [crate::aws::archive::Archive2Header] followed by a series of compressed LDM records, each
/// containing messages with radar data.
pub struct LDMRecord<'a>(LDMRecordData<'a>);

impl<'a> LDMRecord<'a> {
    /// Creates a new LDM record with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        LDMRecord(LDMRecordData::Owned(data))
    }
    
    /// Creates a new LDM record with the provided data slice.
    pub fn from_slice(data: &'a [u8]) -> Self {
        LDMRecord(LDMRecordData::Borrowed(data))
    }

    /// The data contained in this LDM record.
    pub fn data(&self) -> &[u8] {
        match &self.0 {
            LDMRecordData::Borrowed(data) => data,
            LDMRecordData::Owned(data) => data,
        }
    }
    
    /// Whether this LDM record's data is compressed.
    pub fn compressed(&self) -> bool {
        self.data().len() >= 2 && self.data()[0..2].as_ref() == b"BZ"
    }
    
    /// Decompresses this LDM record's data.
    pub fn decompress(&self) -> Result<LDMRecord> {
        if !self.compressed() {
            return Err(Error::UncompressedDataError);
        }

        let mut decompressed_data = Vec::new();
        BzDecoder::new(self.data()).read_to_end(&mut decompressed_data)?;
        Ok(LDMRecord::new(decompressed_data))
    }
}

/// Splits compressed LDM record data into individual records.
fn split_records(data: &Vec<u8>) -> Vec<LDMRecord> {
    let mut records = Vec::new();

    let mut position = 0;
    loop {
        if position >= data.len() {
            break;
        }

        let mut record_size = [0; 4];
        record_size.copy_from_slice(&data[position..position + 4]);
        let record_size = i32::from_be_bytes(record_size).abs();

        let whole_record_size = record_size as usize + 4;
        records.push(LDMRecord::from_slice(&data[position..position + whole_record_size]));
        position += whole_record_size;
    }

    records
}
