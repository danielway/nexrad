use crate::aws::archive::identifier::Identifier;
use crate::aws::archive::{Archive2Header, LDMRecord};

/// A NEXRAD Archive II data file with identifier, decoded header,
pub struct File<'a>(Identifier, Archive2Header, Vec<LDMRecord<'a>>);

impl<'a> File<'a> {
    fn new(identifier: Identifier, header: Archive2Header, records: Vec<LDMRecord<'a>>) -> Self {
        File(identifier, header, records)
    }

    /// The file's identifier.
    pub fn identifier(&self) -> &Identifier {
        &self.0
    }

    /// The file's Archive II header.
    pub fn header(&self) -> &Archive2Header {
        &self.1
    }

    /// The file's LDM records.
    pub fn records(&self) -> &[LDMRecord<'a>] {
        &self.2
    }
}
