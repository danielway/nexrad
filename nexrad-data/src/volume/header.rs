use crate::volume::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use std::fmt::Debug;

/// Header for an Archive II volume file containing metadata about the radar data. This header is
/// located at the beginning of the file.
#[repr(C)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Header {
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
    tape_filename: [u8; 9],

    /// Sequential number assigned to each volume of radar data in the queue, rolling over to 001
    /// after 999.
    extension_number: [u8; 3],

    /// This volume's date represented as a count of days since 1 January 1970 00:00 GMT. It is
    /// also referred-to as a "modified Julian date" where it is the Julian date - 2440586.5.
    date: u32,

    /// Milliseconds past midnight, GMT.
    time: u32,

    /// The ICAO identifier of the radar site.
    icao_of_radar: [u8; 4],
}

impl Header {
    /// Deserializes an Archive II header from the provided reader.
    #[cfg(all(feature = "serde", feature = "bincode"))]
    pub fn deserialize<R: std::io::Read>(reader: &mut R) -> crate::result::Result<Self> {
        use bincode::{DefaultOptions, Options};
        Ok(DefaultOptions::new()
            .with_fixint_encoding()
            .with_big_endian()
            .deserialize_from(reader.by_ref())?)
    }

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
    pub fn tape_filename(&self) -> Option<String> {
        String::from_utf8(self.tape_filename.to_vec()).ok()
    }

    /// Sequential number assigned to each volume of radar data in the queue, rolling over to 001
    /// after 999.
    pub fn extension_number(&self) -> Option<String> {
        String::from_utf8(self.extension_number.to_vec()).ok()
    }

    /// Returns the date and time of the volume.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        get_datetime(self.date as u16, Duration::milliseconds(self.time as i64))
    }

    /// The ICAO identifier of the radar site.
    pub fn icao_of_radar(&self) -> Option<String> {
        String::from_utf8(self.icao_of_radar.to_vec()).ok()
    }
}
