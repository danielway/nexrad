use crate::result::{Error, Result};
use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};


/// A radar site's metadata including a variety of infrequently-changing properties.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Site {
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_identifier"))]
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

    /// The four-letter ICAO identifier for the radar site, such as `"KTLX"`.
    pub fn identifier(&self) -> &str {
        // The builder only accepts four ASCII alphanumeric bytes, which are always valid UTF-8.
        std::str::from_utf8(&self.identifier).unwrap_or_default()
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

    /// The height of the radar feedhorn above ground level in meters.
    pub fn feedhorn_height_meters(&self) -> u16 {
        self.feedhorn_height_meters
    }

}

/// Builder for [`Site`], created with [`Site::builder`].
///
/// The identifier, latitude, and longitude are required. All other properties are optional and
/// default to:
///
/// - `site_elevation_meters`: `0`
/// - `feedhorn_height_meters`: `0`
///
/// [`build`](Self::build) returns [`Error::InvalidSite`] unless the identifier is four ASCII
/// alphanumeric bytes, the latitude is finite and within `-90.0..=90.0`, and the longitude is
/// finite and within `-180.0..=180.0`.
///
/// # Example
///
/// ```
/// use nexrad_model::meta::Site;
///
/// let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
///     .site_elevation_meters(370)
///     .build()?;
/// assert_eq!(site.identifier(), "KTLX");
/// assert!(Site::builder(*b"KT X", 35.3331, -97.2778).build().is_err());
/// # Ok::<(), nexrad_model::result::Error>(())
/// ```
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
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidSite`] if any rule listed on [`SiteBuilder`] is broken.
    pub fn build(self) -> Result<Site> {
        let site = &self.site;
        if !site.identifier.iter().all(u8::is_ascii_alphanumeric) {
            return Err(Error::InvalidSite {
                parameter: "identifier",
                reason: "must be four ASCII alphanumeric characters",
            });
        }
        if !(-90.0..=90.0).contains(&site.latitude) {
            return Err(Error::InvalidSite {
                parameter: "latitude",
                reason: "must be finite and within -90.0..=90.0",
            });
        }
        if !(-180.0..=180.0).contains(&site.longitude) {
            return Err(Error::InvalidSite {
                parameter: "longitude",
                reason: "must be finite and within -180.0..=180.0",
            });
        }
        Ok(self.site)
    }
}

/// Deserializes through [`Site::builder`], so the identifier and location are validated.
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Site {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let repr = SiteRepr::deserialize(deserializer)?;
        let identifier = <[u8; 4]>::try_from(repr.identifier.as_bytes()).map_err(|_| {
            serde::de::Error::custom(format_args!(
                "site identifier {:?} is not four characters",
                repr.identifier
            ))
        })?;
        Site::builder(identifier, repr.latitude, repr.longitude)
            .site_elevation_meters(repr.site_elevation_meters)
            .feedhorn_height_meters(repr.feedhorn_height_meters)
            .build()
            .map_err(serde::de::Error::custom)
    }
}

/// The serialized form of [`Site`], validated before it becomes one.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(rename = "Site")]
struct SiteRepr {
    identifier: String,
    latitude: f64,
    longitude: f64,
    site_elevation_meters: i16,
    feedhorn_height_meters: u16,
}

/// Serializes a site identifier as the string [`Site::identifier`] returns.
#[cfg(feature = "serde")]
fn serialize_identifier<S: serde::Serializer>(
    identifier: &[u8; 4],
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    let identifier = std::str::from_utf8(identifier).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(identifier)
}

impl Display for Site {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:.4}°, {:.4}°, {}m)",
            self.identifier(),
            self.latitude,
            self.longitude,
            self.site_elevation_meters
        )
    }
}
