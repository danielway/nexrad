use crate::messages::primitive_aliases::Integer2;
use crate::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::fmt::Debug;

/// Header information for a clutter filter map to be read directly from the Archive II file.
#[derive(Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Header {
    /// The date the clutter filter map was generated represented as a count of days since 1 January
    /// 1970 00:00 GMT. It is also referred-to as a "modified Julian date" where it is the Julian
    /// date - 2440586.5.
    pub map_generation_date: Integer2,

    /// The time the clutter filter map was generated in minutes past midnight, GMT.
    pub map_generation_time: Integer2,

    /// The number of elevation segments defined in this clutter filter map. There may be 1 to 5,
    /// though there are typically 2. They will follow this header in order of increasing elevation.
    pub elevation_segment_count: Integer2,
}

impl Header {
    /// The date and time the clutter filter map was generated.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        get_datetime(
            self.map_generation_date,
            Duration::minutes(self.map_generation_time as i64),
        )
    }
}

impl Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("map_generation_date_time", &self.date_time())
            .field("elevation_segment_count", &self.elevation_segment_count)
            .finish()
    }
}
