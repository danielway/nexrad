use crate::messages::primitive_aliases::Integer2;
use crate::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use std::fmt::Debug;
use zerocopy::{TryFromBytes, Immutable, KnownLayout};

/// Header information for a clutter filter map to be read directly from the Archive II file.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash, Debug, TryFromBytes, Immutable, KnownLayout)]
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
            self.map_generation_date.get(),
            Duration::minutes(self.map_generation_time.get() as i64),
        )
    }

    /// Decodes a reference to a Header from a byte slice, returning the header and remaining bytes.
    pub fn decode_ref(bytes: &[u8]) -> crate::result::Result<(&Self, &[u8])> {
        Ok(Self::try_ref_from_prefix(bytes)?)
    }

    /// Decodes an owned copy of a Header from a byte slice.
    pub fn decode_owned(bytes: &[u8]) -> crate::result::Result<Self> {
        let (header, _) = Self::decode_ref(bytes)?;
        Ok(header.clone())
    }
}
