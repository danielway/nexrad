#[cfg(feature = "bzip2")]
use bzip2::read::BzDecoder;

enum RecordData<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

/// Represents a single LDM record with its data which may be compressed.
///
/// The Unidata Local Data Manager (LDM) is a data distribution system used by the NWS to distribute
/// NEXRAD archival radar data. A NEXRAD "Archive II" file starts with an
/// [crate::volume::Header] followed by a series of compressed LDM records, each
/// containing messages with radar data.
pub struct Record<'a>(RecordData<'a>);

impl<'a> Record<'a> {
    /// Creates a new LDM record with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        Record(RecordData::Owned(data))
    }

    /// Creates a new LDM record with the provided data slice.
    pub fn from_slice(data: &'a [u8]) -> Self {
        Record(RecordData::Borrowed(data))
    }

    /// The data contained in this LDM record.
    pub fn data(&self) -> &[u8] {
        match &self.0 {
            RecordData::Borrowed(data) => data,
            RecordData::Owned(data) => data,
        }
    }

    /// Whether this LDM record's data is compressed.
    pub fn compressed(&self) -> bool {
        self.data().len() >= 6 && self.data()[4..6].as_ref() == b"BZ"
    }

    /// Decompresses this LDM record's data.
    #[cfg(feature = "bzip2")]
    pub fn decompress<'b>(&self) -> crate::result::Result<Record<'b>> {
        use std::io::Read;

        if !self.compressed() {
            return Err(crate::result::Error::UncompressedDataError);
        }

        // Skip the four-byte record size prefix
        let data = self.data().split_at(4).1;

        let mut decompressed_data = Vec::new();
        BzDecoder::new(data).read_to_end(&mut decompressed_data)?;

        Ok(Record::new(decompressed_data))
    }
}

/// Splits compressed LDM record data into individual records. Will omit the record size prefix from
/// each record.
pub fn split_compressed_records(data: &[u8]) -> Vec<Record> {
    let mut records = Vec::new();

    let mut position = 0;
    loop {
        if position >= data.len() {
            break;
        }

        let mut record_size = [0; 4];
        record_size.copy_from_slice(&data[position..position + 4]);
        let record_size = i32::from_be_bytes(record_size).unsigned_abs() as usize;

        records.push(Record::from_slice(&data[position..position + record_size + 4]));
        position += record_size + 4;
    }

    records
}
