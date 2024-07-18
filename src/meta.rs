//!
//! This module contains models containing metadata about the radar data collected by the NEXRAD
//! weather network. This data may not change between radials, sweeps, or even scans, and thus it
//! is represented separately to avoid duplication in storage.
//!

#[cfg(feature = "uom")]
use uom::si::{
    angle::degree,
    f32::{Angle, Length},
    length::meter,
};

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

    /// The height of the radar site above sea level in meters.
    pub fn height_meters(&self) -> i16 {
        self.height_meters
    }

    /// The height of the radar site above sea level.
    #[cfg(feature = "uom")]
    pub fn height(&self) -> Length {
        Length::new::<meter>(self.height_meters as f32)
    }

    /// The height of the radar site's feedhorn above sea level in meters.
    pub fn feedhorn_height_meters(&self) -> u16 {
        self.feedhorn_height_meters
    }

    /// The height of the radar site's feedhorn above sea level.
    #[cfg(feature = "uom")]
    pub fn feedhorn_height(&self) -> Length {
        Length::new::<meter>(self.feedhorn_height_meters as f32)
    }
}
