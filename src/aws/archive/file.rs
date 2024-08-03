use crate::aws::archive::identifier::Identifier;
use crate::aws::archive::{Header, Record};

/// A NEXRAD Archive II data file with identifier, decoded header,
pub struct File<'a>(Identifier, Header, Vec<Record<'a>>);

impl<'a> File<'a> {
    fn new(identifier: Identifier, header: Header, records: Vec<Record<'a>>) -> Self {
        File(identifier, header, records)
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
    pub fn records(&self) -> &[Record<'a>] {
        &self.2
    }
}
