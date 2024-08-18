use crate::archive::{split_compressed_records, Header, Record};
use crate::result::Result;

/// A NEXRAD Archive II volume data file.
pub struct File(Vec<u8>);

impl File {
    /// Creates a new Archive II file with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// The file's encoded and compressed data.
    pub fn data(&self) -> &Vec<u8> {
        &self.0
    }

    /// The file's decoded Archive II header.
    pub fn header(&self) -> Result<Header> {
        Header::deserialize(&mut self.0.as_slice())
    }

    /// The file's LDM records.
    pub fn records(&self) -> Vec<Record> {
        split_compressed_records(&self.0[size_of::<Header>()..])
    }
}
