use crate::archive::identifier::Identifier;
use crate::archive::{split_records, Header, Record};
use crate::result::Result;

/// A NEXRAD Archive II volume data file.
pub struct File(Option<Identifier>, Vec<u8>);

impl File {
    /// Creates a new Archive II file with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        Self(None, data)
    }

    /// Creates a new Archive II file with the provided identifier and data.
    pub fn new_with_identifier(identifier: Identifier, data: Vec<u8>) -> Self {
        Self(Some(identifier), data)
    }

    /// If available, the file's identifier.
    pub fn identifier(&self) -> Option<&Identifier> {
        self.0.as_ref()
    }

    /// The file's encoded and compressed data.
    pub fn data(&self) -> &Vec<u8> {
        &self.1
    }

    /// The file's decoded Archive II header.
    pub fn header(&self) -> Result<Header> {
        Header::deserialize(&mut self.1.as_slice())
    }

    /// The file's LDM records.
    pub fn records(&self) -> Vec<Record> {
        split_records(&self.1[size_of::<Header>()..])
    }
}
