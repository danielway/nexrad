//!
//! Struct definitions and utilities for NEXRAD Level II data files.
//!

use chrono::NaiveDate;

/// Metadata describing a NEXRAD WSR-88D radar data file.
pub struct NexradFileMetadata {
    site: String,
    date: NaiveDate,
    identifier: String,
}

impl NexradFileMetadata {
    /// Create new file metadata.
    pub fn new(site: String, date: NaiveDate, identifier: String) -> Self {
        Self {
            site,
            date,
            identifier,
        }
    }

    /// The radar site this file was produced at, e.g. KDMX.
    pub fn site(&self) -> &String {
        &self.site
    }

    /// The date this file's data was collected on.
    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    /// The unique identifier for this file for its site on the date.
    pub fn identifier(&self) -> &String {
        &self.identifier
    }
}

/// Determines whether the provided NEXRAD data file is compressed.
pub fn is_compressed(data: &[u8]) -> bool {
    data.len() >= 30 && &data[28..30] == b"BZ"
}
