use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "uom")]
use uom::si::{f64::Length, length::meter};

/// A radar site's metadata including a variety of infrequently-changing properties.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Site {
    identifier: [u8; 4],
    latitude: f64,
    longitude: f64,
    site_elevation_meters: i16,
    feedhorn_height_meters: u16,
}

impl Site {
    /// Start building radar site metadata from its identifier and location.
    ///
    /// All other properties take the defaults documented on [`SiteBuilder`].
    pub fn builder(identifier: [u8; 4], latitude: f64, longitude: f64) -> SiteBuilder {
        SiteBuilder {
            site: Site {
                identifier,
                latitude,
                longitude,
                site_elevation_meters: 0,
                feedhorn_height_meters: 0,
            },
        }
    }

    /// The four-letter ICAO identifier for the radar site.
    pub fn identifier(&self) -> &[u8; 4] {
        &self.identifier
    }

    /// The four-letter ICAO identifier for the radar site as a string.
    pub fn identifier_string(&self) -> String {
        String::from_utf8_lossy(&self.identifier).to_string()
    }

    /// The latitude of the radar site in degrees.
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    /// The longitude of the radar site in degrees.
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    /// The elevation of the radar site above mean sea level in meters.
    pub fn site_elevation_meters(&self) -> i16 {
        self.site_elevation_meters
    }

    /// The elevation of the radar site above mean sea level.
    #[cfg(feature = "uom")]
    pub fn site_elevation(&self) -> Length {
        Length::new::<meter>(self.site_elevation_meters as f64)
    }

    /// The height of the radar feedhorn above ground level in meters.
    pub fn feedhorn_height_meters(&self) -> u16 {
        self.feedhorn_height_meters
    }

    /// The height of the radar feedhorn above ground level.
    #[cfg(feature = "uom")]
    pub fn feedhorn_height(&self) -> Length {
        Length::new::<meter>(self.feedhorn_height_meters as f64)
    }
}

/// Builder for [`Site`], created with [`Site::builder`].
///
/// The identifier, latitude, and longitude are required. All other properties are optional and
/// default to:
///
/// - `site_elevation_meters`: `0`
/// - `feedhorn_height_meters`: `0`
#[derive(Debug, Clone)]
#[must_use]
pub struct SiteBuilder {
    site: Site,
}

impl SiteBuilder {
    /// Set the elevation of the radar site above mean sea level in meters. Defaults to `0`.
    pub fn site_elevation_meters(mut self, site_elevation_meters: i16) -> Self {
        self.site.site_elevation_meters = site_elevation_meters;
        self
    }

    /// Set the height of the radar feedhorn above ground level in meters. Defaults to `0`.
    pub fn feedhorn_height_meters(mut self, feedhorn_height_meters: u16) -> Self {
        self.site.feedhorn_height_meters = feedhorn_height_meters;
        self
    }

    /// Build the [`Site`].
    #[must_use]
    pub fn build(self) -> Site {
        self.site
    }
}

impl Display for Site {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:.4}°, {:.4}°, {}m)",
            self.identifier_string(),
            self.latitude,
            self.longitude,
            self.site_elevation_meters
        )
    }
}
