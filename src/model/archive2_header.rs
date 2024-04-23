use crate::model::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Debug, Formatter};

#[repr(C)]
#[derive(Serialize, Deserialize)]
pub struct Archive2Header {
    /// The tape's filename which indicates the version of the data. Name is in the format
    /// `AR2V0 0xx.` where `xx` indicates the version of the data.
    ///
    /// Versions:
    ///   02 = Super Resolution disabled at the RDA (pre RDA Build 12.0)
    ///   03 = Super Resolution (pre RDA Build 12.0)
    ///   04 = Recombined Super Resolution
    ///   05 = Super Resolution disabled at the RDA (RDA Build 12.0 and later)
    ///   06 = Super Resolution (RDA Build 12.0 and later)
    ///   07 = Recombined Super Resolution (RDA Build 12.0 and later)
    /// NOTE: Dual-pol data introduced in RDA Build 12.0
    pub tape_filename: [u8; 9],

    /// Sequential number assigned to each volume of radar data in the queue, rolling over to 001
    /// after 999.
    pub extension_number: [u8; 3],

    /// This archive's date represented as a count of days since 1 January 1970 00:00 GMT. It is
    /// also referred-to as a "modified Julian date" where it is the Julian date - 2440586.5.
    pub date: u32,

    /// Milliseconds past midnight, GMT.
    pub time: u32,

    /// The ICAO identifier of the radar site.
    pub icao_of_radar: [u8; 4],
}

impl Archive2Header {
    /// The tape's filename which indicates the version of the data. Name is in the format
    /// `AR2V0 0xx.` where `xx` indicates the version of the data.
    ///
    /// Versions:
    ///   02 = Super Resolution disabled at the RDA (pre RDA Build 12.0)
    ///   03 = Super Resolution (pre RDA Build 12.0)
    ///   04 = Recombined Super Resolution
    ///   05 = Super Resolution disabled at the RDA (RDA Build 12.0 and later)
    ///   06 = Super Resolution (RDA Build 12.0 and later)
    ///   07 = Recombined Super Resolution (RDA Build 12.0 and later)
    /// NOTE: Dual-pol data introduced in RDA Build 12.0
    pub fn tape_filename(&self) -> String {
        String::from_utf8(self.tape_filename.to_vec()).unwrap()
    }

    /// Sequential number assigned to each volume of radar data in the queue, rolling over to 001
    /// after 999.
    pub fn extension_number(&self) -> String {
        String::from_utf8(self.extension_number.to_vec()).unwrap()
    }

    /// Returns the date and time of the archive.
    pub fn date_time(&self) -> DateTime<Utc> {
        get_datetime(self.date as u16, Duration::milliseconds(self.time as i64))
    }

    /// The ICAO identifier of the radar site.
    pub fn icao_of_radar(&self) -> String {
        String::from_utf8(self.icao_of_radar.to_vec()).unwrap()
    }
}

impl Debug for Archive2Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Archive2Header")
            .field("tape_filename", &self.tape_filename())
            .field("extension_number", &self.extension_number())
            .field("date_time", &self.date_time())
            .field("icao_of_radar", &self.icao_of_radar())
            .finish()
    }
}
