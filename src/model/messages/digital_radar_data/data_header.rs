use crate::model::primitive_aliases::{Code1, Integer2, Integer4, Real4};
use crate::model::util::get_datetime;
use crate::model::{CompressionIndicator, RadialStatus};
use chrono::{DateTime, Duration, Utc};
use uom::si::angle::degree;
use uom::si::f64::{Angle, Information};
use uom::si::information::byte;

/// The digital radar data message header block precedes base data information for a particular
/// radial and includes parameters for that radial and information about the following data blocks.
pub struct DataHeaderBlock {
    /// ICAO radar identifier.
    radar_identifier: [u8; 4],

    /// Collection time in milliseconds past midnight, GMT.
    time: Integer4,

    /// This message's date represented as a count of days since 1 January 1970 00:00 GMT. It is
    /// also referred-to as a "modified Julian date" where it is the Julian date - 2440586.5.
    date: Integer2,

    /// Radial number within the elevation scan. These range up to 720, in 0.5 degree increments.
    azimuth_number: Integer2,

    /// Azimuth angle at which the radial was collected in degrees.
    azimuth_angle: Real4,

    /// Indicates if the message is compressed and what type of compression was used. This header is
    /// not compressed.
    ///
    /// Values:
    ///   0 = Uncompressed
    ///   1 = Compressed using BZIP2
    ///   2 = Compressed using ZLIB
    ///   3 = Future use
    compression_indicator: Code1,

    /// Spare to force halfword alignment.
    spare: u8,

    /// Uncompressed length of the radial in bytes (including the data header block).
    radial_length: Integer2,

    /// Azimuthal spacing between adjacent radials. Note this is the commanded value, not
    /// necessarily the actual spacing.
    ///
    /// Values:
    ///   1 = 0.5 degrees
    ///   2 = 1.0 degrees
    azimuth_resolution_spacing: Code1,
    
    /// The radial's status within the larger scan (e.g. first, last).
    /// 
    /// Statuses:
    ///   0 = Start of elevation
    ///   1 = Intermediate radial data
    ///   2 = End of elevation
    ///   3 = Start of volume scan
    ///   4 = End of volume scan
    ///   5 = Start of new elevation which is the last in the VCP
    radial_status: Code1,
}

impl DataHeaderBlock {
    /// ICAO radar identifier.
    pub fn radar_identifier(&self) -> String {
        String::from_utf8_lossy(&self.radar_identifier).to_string()
    }

    /// The collection date and time for this data.
    pub fn date_time(&self) -> DateTime<Utc> {
        get_datetime(self.date, Duration::milliseconds(self.time as i64))
    }

    /// Azimuth angle at which the radial was collected.
    pub fn azimuth_angle(&self) -> Angle {
        Angle::new::<degree>(self.azimuth_angle as f64)
    }

    /// Whether the message is compressed and what type of compression was used.
    pub fn compression_indicator(&self) -> CompressionIndicator {
        match self.compression_indicator {
            0 => CompressionIndicator::Uncompressed,
            1 => CompressionIndicator::CompressedBZIP2,
            2 => CompressionIndicator::CompressedZLIB,
            _ => CompressionIndicator::FutureUse,
        }
    }

    /// Uncompressed length of the radial (including the data header block).
    pub fn radial_length(&self) -> Information {
        Information::new::<byte>(self.radial_length as f64)
    }

    /// Azimuthal spacing between adjacent radials.
    pub fn azimuth_resolution_spacing(&self) -> Angle {
        Angle::new::<degree>(self.azimuth_resolution_spacing as f64 * 0.5)
    }
    
    /// The radial's status within the larger scan.
    pub fn radial_status(&self) -> RadialStatus {
        match self.radial_status {
            0 => RadialStatus::ElevationStart,
            1 => RadialStatus::IntermediateRadialData,
            2 => RadialStatus::ElevationEnd,
            3 => RadialStatus::VolumeScanStart,
            4 => RadialStatus::VolumeScanEnd,
            _ => RadialStatus::ElevationStartVCPFinal,
        }
    }
}
