//!
//! This module contains models containing metadata about the radar data collected by the NEXRAD
//! weather network. This data may not change between radials, sweeps, or even scans, and thus it
//! is represented separately to avoid duplication in storage.
//!

use uom::si::angle::degree;
use uom::si::f32::Angle;

/// A radar site's metadata including a variety of infrequently-changing properties.
pub struct Site {
    identifier: [u8; 4],
    latitude: f32,
    longitude: f32,
    height_meters: i16,
    feedhorn_height_meters: u16,
}

impl Site {
    /// The four-letter ICAO identifier for the radar site.
    pub fn identifier(&self) -> &[u8; 4] {
        &self.identifier
    }

    /// The four-letter ICAO identifier for the radar site as a string.
    pub fn identifier_string(&self) -> String {
        String::from_utf8_lossy(&self.identifier).to_string()
    }

    /// The latitude of the radar site in degrees.
    pub fn latitude_degrees(&self) -> f32 {
        self.latitude
    }

    /// The latitude of the radar site.
    #[cfg(feature = "uom")]
    pub fn latitude(&self) -> Angle {
        Angle::new::<degree>(self.latitude)
    }

    /// The longitude of the radar site in degrees.
    pub fn longitude_degrees(&self) -> f32 {
        self.longitude
    }

    /// The longitude of the radar site.
    #[cfg(feature = "uom")]
    pub fn longitude(&self) -> Angle {
        Angle::new::<degree>(self.longitude)
    }
}
