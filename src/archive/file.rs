use crate::archive::identifier::Identifier;
use crate::archive::{split_records, Header, Record};
use crate::result::Result;

/// A NEXRAD Archive II data file with identifier, decoded header,
pub struct File(Identifier, Header, Vec<u8>);

impl File {
    /// Creates a new Archive II file with the provided identifier and data.
    pub fn new(identifier: Identifier, data: Vec<u8>) -> Result<Self> {
        let header = Header::deserialize(&mut data.as_slice())?;
        Ok(File(identifier, header, data))
    }

    /// The file's identifier.
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    /// The file's Archive II header.
    pub fn header(&self) -> &Header {
        &self.1
    }

    /// The file's LDM records.
    pub fn records(&self) -> Vec<Record> {
        split_records(&self.2)
    }
}
