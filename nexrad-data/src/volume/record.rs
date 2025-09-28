use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq, Hash)]
enum RecordData<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl Debug for RecordData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordData::Borrowed(data) => write!(f, "RecordData::Borrowed({} bytes)", data.len()),
            RecordData::Owned(data) => write!(f, "RecordData::Owned({} bytes)", data.len()),
        }
    }
}

/// Represents a single LDM record with its data which may be compressed.
///
/// The Unidata Local Data Manager (LDM) is a data distribution system used by the NWS to distribute
/// NEXRAD archival radar data. A NEXRAD "Archive II" file starts with an
/// [crate::volume::Header] followed by a series of compressed LDM records, each
/// containing messages with radar data.
#[derive(Clone, PartialEq, Eq, Hash)]
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
        use crate::result::Error;
        use bzip2::read::BzDecoder;
        use std::io::Read;

        if !self.compressed() {
            return Err(Error::UncompressedDataError);
        }

        // Skip the four-byte record size prefix
        let data = self.data().split_at(4).1;

        let mut decompressed_data = Vec::new();
        BzDecoder::new(data).read_to_end(&mut decompressed_data)?;

        Ok(Record::new(decompressed_data))
    }

    /// Decodes the NEXRAD level II messages contained in this LDM record.
    #[cfg(feature = "nexrad-decode")]
    pub fn messages(&self) -> crate::result::Result<Vec<nexrad_decode::messages::Message>> {
        use crate::result::Error;
        use nexrad_decode::messages::decode_messages;
        use std::io::Cursor;

        if self.compressed() {
            return Err(Error::CompressedDataError);
        }

        let mut reader = Cursor::new(self.data());
        Ok(decode_messages(&mut reader)?)
    }
}

impl Debug for Record<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Record");
        debug.field("data.len()", &self.data().len());
        debug.field(
            "borrowed",
            match &self.0 {
                RecordData::Borrowed(_) => &true,
                RecordData::Owned(_) => &false,
            },
        );
        debug.field("compressed", &self.compressed());

        #[cfg(feature = "decode")]
        debug.field(
            "messages.len()",
            &self.messages().map(|messages| messages.len()),
        );

        debug.finish()
    }
}

/// Splits compressed LDM record data into individual records. Will omit the record size prefix from
/// each record.
pub fn split_compressed_records(data: &[u8]) -> Vec<Record<'_>> {
    let mut records = Vec::new();

    let mut position = 0;
    loop {
        if position >= data.len() {
            break;
        }

        let mut record_size = [0; 4];
        record_size.copy_from_slice(&data[position..position + 4]);
        let record_size = i32::from_be_bytes(record_size).unsigned_abs() as usize;

        records.push(Record::from_slice(
            &data[position..position + record_size + 4],
        ));
        position += record_size + 4;
    }

    records
}
