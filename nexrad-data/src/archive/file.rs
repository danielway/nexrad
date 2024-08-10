use crate::archive::identifier::Identifier;
use crate::archive::{split_records, Header, Record};
use crate::result::Result;

/// A NEXRAD Archive II data file with identifier, decoded header,
pub struct File(Identifier, Vec<u8>);

impl File {
    /// Creates a new Archive II file with the provided identifier and data.
    pub(crate) fn new(identifier: Identifier, data: Vec<u8>) -> Self {
        File(identifier, data)
    }

    /// The file's identifier.
    pub fn identifier(&self) -> &Identifier {
        &self.0
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
        split_records(&self.1)
    }
}
