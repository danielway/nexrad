use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use crate::aws::archive::{Archive2Header, LDMRecord};

/// A NEXRAD Archive II data file with identifier, decoded header, 
pub struct File<'a>(FileIdentifier, Archive2Header, Vec<LDMRecord<'a>>);

/// Identifying metadata for a NEXRAD data file.
pub struct FileIdentifier(String);

impl FileIdentifier {
    /// The file name.
    pub fn name(&self) -> &str {
        &self.0
    }

    /// The radar site this file was produced at, e.g. KDMX.
    pub fn site(&self) -> Option<&str> {
        self.0.get(0..4)
    }

    /// This file's data collection time.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        let date_string = self.0.get(4..12)?;
        if let Ok(date) = NaiveDate::parse_from_str(date_string, "%Y%m%d") {
            let time_string = self.0.get(13..19)?;
            if let Ok(time) = NaiveTime::parse_from_str(time_string, "%H%M%S") {
                let naive_datetime = NaiveDateTime::new(date, time);
                return Some(DateTime::from_naive_utc_and_offset(naive_datetime, Utc));
            }
        }

        None
    }
}
