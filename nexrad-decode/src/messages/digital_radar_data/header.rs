use super::raw;
use super::{AzimuthResolutionSpacing, CompressionIndicator, RadialStatus, SpotBlankingFlags};
use crate::util::get_datetime;
use chrono::{DateTime, Duration, Utc};
use std::borrow::Cow;

/// The digital radar data message header block precedes base data information for a particular
/// radial and includes parameters for that radial and information about the following data blocks.
#[derive(Clone, PartialEq, Debug)]
pub struct Header<'a> {
    inner: Cow<'a, raw::Header>,
}

impl<'a> Header<'a> {
    /// Create a new Header wrapper from a raw Header reference.
    pub(crate) fn new(inner: &'a raw::Header) -> Self {
        Self {
            inner: Cow::Borrowed(inner),
        }
    }

    /// Convert this header to an owned version with `'static` lifetime.
    pub fn into_owned(self) -> Header<'static> {
        Header {
            inner: Cow::Owned(self.inner.into_owned()),
        }
    }

    /// ICAO radar identifier (raw bytes). See [`radar_identifier()`](Self::radar_identifier) for
    /// the decoded value.
    pub fn radar_identifier_raw(&self) -> &[u8; 4] {
        &self.inner.radar_identifier
    }

    /// Collection time in milliseconds past midnight, GMT (raw value). See
    /// [`date_time()`](Self::date_time) for the decoded value.
    pub fn time_raw(&self) -> u32 {
        self.inner.time.get()
    }

    /// This message's date represented as a count of days since 1 January 1970 00:00 GMT (raw
    /// value). See [`date_time()`](Self::date_time) for the decoded value.
    pub fn date_raw(&self) -> u16 {
        self.inner.date.get()
    }

    /// Radial number within the elevation scan. These range up to 720, in 0.5 degree increments.
    pub fn azimuth_number(&self) -> u16 {
        self.inner.azimuth_number.get()
    }

    /// Azimuth angle at which the radial was collected, in degrees.
    pub fn azimuth_angle_degrees(&self) -> f32 {
        self.inner.azimuth_angle.get()
    }

    /// Indicates if the message is compressed and what type of compression was used (raw value).
    pub fn compression_indicator_raw(&self) -> u8 {
        self.inner.compression_indicator
    }

    /// Uncompressed length of the radial (including the data header block), in bytes.
    pub fn radial_length_bytes(&self) -> u16 {
        self.inner.radial_length.get()
    }

    /// Azimuthal spacing between adjacent radials (raw code).
    /// Values: 1 = 0.5 degrees, 2 = 1.0 degrees
    pub fn azimuth_resolution_spacing_raw(&self) -> u8 {
        self.inner.azimuth_resolution_spacing
    }

    /// The radial's status within the larger scan (raw value).
    pub fn radial_status_raw(&self) -> u8 {
        self.inner.radial_status
    }

    /// The radial's elevation number within the volume scan.
    pub fn elevation_number(&self) -> u8 {
        self.inner.elevation_number
    }

    /// The sector number within cut. A value of 0 is only valid for continuous surveillance cuts.
    pub fn cut_sector_number(&self) -> u8 {
        self.inner.cut_sector_number
    }

    /// The radial's collection elevation angle, in degrees.
    pub fn elevation_angle_degrees(&self) -> f32 {
        self.inner.elevation_angle.get()
    }

    /// The spot blanking status for the current radial, elevation, and volume scan (raw value).
    pub fn radial_spot_blanking_status_raw(&self) -> u8 {
        self.inner.radial_spot_blanking_status
    }

    /// The azimuth indexing value (raw scaled value).
    /// Values: 0 = No indexing, 1-100 = Indexing angle of 0.01 to 1.00 degrees
    pub fn azimuth_indexing_mode_raw(&self) -> u8 {
        self.inner.azimuth_indexing_mode
    }

    /// The number of "data moment" blocks following this header block.
    pub fn data_block_count(&self) -> u16 {
        self.inner.data_block_count.get()
    }

    /// ICAO radar identifier, or `None` if the bytes are not valid UTF-8.
    pub fn radar_identifier(&self) -> Option<&str> {
        std::str::from_utf8(&self.inner.radar_identifier).ok()
    }

    /// The collection date and time for this data.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        get_datetime(
            self.inner.date.get(),
            Duration::milliseconds(self.inner.time.get() as i64),
        )
    }

    /// Whether the message is compressed and what type of compression was used.
    pub fn compression_indicator(&self) -> CompressionIndicator {
        match self.inner.compression_indicator {
            0 => CompressionIndicator::Uncompressed,
            1 => CompressionIndicator::CompressedBzip2,
            2 => CompressionIndicator::CompressedZlib,
            other => CompressionIndicator::Unknown(other),
        }
    }

    /// The radial's status within the larger scan.
    pub fn radial_status(&self) -> RadialStatus {
        RadialStatus::from_number(self.inner.radial_status)
    }

    /// The spot blanking status for the current radial, elevation, and volume scan.
    pub fn radial_spot_blanking_status(&self) -> SpotBlankingFlags {
        SpotBlankingFlags::new(self.inner.radial_spot_blanking_status)
    }

    /// Azimuthal spacing between adjacent radials.
    pub fn azimuth_resolution_spacing(&self) -> AzimuthResolutionSpacing {
        match self.inner.azimuth_resolution_spacing {
            1 => AzimuthResolutionSpacing::HalfDegree,
            2 => AzimuthResolutionSpacing::OneDegree,
            other => AzimuthResolutionSpacing::Unknown(other),
        }
    }

    /// Azimuthal spacing between adjacent radials, in degrees (the raw code times 0.5).
    pub fn azimuth_resolution_spacing_degrees(&self) -> f32 {
        self.inner.azimuth_resolution_spacing as f32 * 0.5
    }

    /// The azimuth indexing value in degrees (if keyed to constant angles).
    /// Returns `None` if no indexing (value is 0), otherwise the indexing angle in degrees.
    pub fn azimuth_indexing_mode_degrees(&self) -> Option<f32> {
        if self.inner.azimuth_indexing_mode == 0 {
            None
        } else {
            Some(self.inner.azimuth_indexing_mode as f32 * 0.01)
        }
    }
}
