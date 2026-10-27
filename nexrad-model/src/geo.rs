//! Geographic coordinate types and radar coordinate system.
//!
//! This module provides types for working with geographic coordinates and converting
//! between radar-relative polar coordinates and geographic coordinates.
//!
//! The earth is modelled as a sphere of radius 6371 km. Beam propagation uses the standard
//! 4/3 effective earth radius to account for atmospheric refraction, and the radar antenna
//! sits at its height above mean sea level. Latitudes and longitudes are geodetic degrees as
//! reported by the radar; altitudes are above mean sea level.

use crate::meta::Site;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A geographic point in latitude and longitude degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeoPoint {
    /// Latitude in degrees (-90 to 90).
    pub latitude: f64,
    /// Longitude in degrees (-180 to 180).
    pub longitude: f64,
}

/// A geographic point with altitude.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeoPoint3D {
    /// Latitude in degrees (-90 to 90).
    pub latitude: f64,
    /// Longitude in degrees (-180 to 180).
    pub longitude: f64,
    /// Altitude above mean sea level in meters.
    pub altitude_meters: f64,
}

/// A rectangular geographic extent (bounding box).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeoExtent {
    /// Southwest corner (minimum latitude and longitude).
    pub min: GeoPoint,
    /// Northeast corner (maximum latitude and longitude).
    pub max: GeoPoint,
}

/// A point in radar-relative polar coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PolarPoint {
    /// Azimuth angle in degrees clockwise from north (0-360).
    pub azimuth_angle_degrees: f64,
    /// Slant range from the radar in kilometers.
    pub range_km: f64,
    /// Elevation angle in degrees above the horizon.
    pub elevation_angle_degrees: f64,
}

/// Mean earth radius in meters, the radius of the spherical earth model.
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Effective earth radius for beam propagation in meters: 4/3 of `EARTH_RADIUS_M`, which
/// accounts for refraction in a standard atmosphere.
const EFFECTIVE_EARTH_RADIUS_M: f64 = EARTH_RADIUS_M * 4.0 / 3.0;

/// Radar coordinate system for converting between polar and geographic coordinates.
///
/// Uses a spherical earth of radius 6371 km with the 4/3 effective earth radius beam propagation
/// model, and accounts for the antenna's height above mean sea level.
/// [`polar_to_geo`](Self::polar_to_geo) and [`geo_to_polar`](Self::geo_to_polar) are exact
/// inverses under this model.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RadarCoordinateSystem {
    /// Radar site latitude in degrees.
    latitude: f64,
    /// Radar site longitude in degrees.
    longitude: f64,
    /// Radar antenna height above mean sea level in meters.
    antenna_height_meters: f64,
}

impl RadarCoordinateSystem {
    /// Create a new radar coordinate system from a site's metadata.
    ///
    /// The antenna height is the sum of the site elevation above mean sea level and the feedhorn
    /// height above ground level.
    pub fn new(site: &Site) -> Self {
        Self {
            latitude: site.latitude(),
            longitude: site.longitude(),
            antenna_height_meters: site.site_elevation_meters() as f64
                + site.feedhorn_height_meters() as f64,
        }
    }

    /// The radar site latitude in degrees.
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    /// The radar site longitude in degrees.
    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    /// The radar antenna height above mean sea level in meters: the sum of
    /// [`Site::site_elevation_meters`] and [`Site::feedhorn_height_meters`].
    pub fn antenna_height_meters(&self) -> f64 {
        self.antenna_height_meters
    }

    /// The height of the beam centre above mean sea level in meters at `range_km` slant range
    /// along a beam at `elevation_angle_degrees`.
    ///
    /// With `R'` the 4/3 effective earth radius, `h0` the
    /// [antenna height](Self::antenna_height_meters), `r` the slant range and `theta` the
    /// elevation angle, the height is
    /// `h = sqrt(r^2 + (R' + h0)^2 + 2 r (R' + h0) sin(theta)) - R'`, equivalently
    /// `(R' + h0) cos(theta) / cos(theta + phi) - R'` with `phi` the angle subtended at the
    /// earth's centre (see [`ground_range_km`](Self::ground_range_km)).
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::geo::RadarCoordinateSystem;
    /// use nexrad_model::meta::Site;
    ///
    /// let site = Site::builder(*b"KTLX", 35.3331, -97.2778).site_elevation_meters(370).build()?;
    /// let coords = RadarCoordinateSystem::new(&site);
    ///
    /// // The 0.5 degree beam is about 1.46 km above the antenna at 100 km.
    /// let above_antenna = coords.beam_height_meters(100.0, 0.5) - coords.antenna_height_meters();
    /// assert!((above_antenna - 1460.0).abs() < 10.0);
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn beam_height_meters(&self, range_km: f64, elevation_angle_degrees: f64) -> f64 {
        let range_m = range_km * 1000.0;
        let antenna_radius_m = EFFECTIVE_EARTH_RADIUS_M + self.antenna_height_meters;
        let sin_elevation = elevation_angle_degrees.to_radians().sin();
        (range_m * range_m
            + antenna_radius_m * antenna_radius_m
            + 2.0 * range_m * antenna_radius_m * sin_elevation)
            .sqrt()
            - EFFECTIVE_EARTH_RADIUS_M
    }

    /// The great-circle distance in kilometers along the earth's surface from the radar to the
    /// point beneath the beam centre at `range_km` slant range along a beam at
    /// `elevation_angle_degrees`.
    ///
    /// The angle subtended at the earth's centre is
    /// `phi = atan2(r cos(theta), (R' + h0) + r sin(theta))`, and the ground range is `R' * phi`;
    /// see [`beam_height_meters`](Self::beam_height_meters) for the symbols.
    pub fn ground_range_km(&self, range_km: f64, elevation_angle_degrees: f64) -> f64 {
        let range_m = range_km * 1000.0;
        let antenna_radius_m = EFFECTIVE_EARTH_RADIUS_M + self.antenna_height_meters;
        let elevation_rad = elevation_angle_degrees.to_radians();
        let central_angle =
            (range_m * elevation_rad.cos()).atan2(antenna_radius_m + range_m * elevation_rad.sin());
        EFFECTIVE_EARTH_RADIUS_M * central_angle / 1000.0
    }

    /// Convert radar-relative polar coordinates to geographic coordinates.
    ///
    /// The altitude is [`beam_height_meters`](Self::beam_height_meters) and the point lies
    /// [`ground_range_km`](Self::ground_range_km) from the radar along the great circle at the
    /// given azimuth. [`geo_to_polar`](Self::geo_to_polar) is its exact inverse.
    pub fn polar_to_geo(&self, polar: PolarPoint) -> GeoPoint3D {
        let altitude_meters =
            self.beam_height_meters(polar.range_km, polar.elevation_angle_degrees);
        let ground_range_m =
            self.ground_range_km(polar.range_km, polar.elevation_angle_degrees) * 1000.0;
        let az_rad = polar.azimuth_angle_degrees.to_radians();

        let lat_rad = self.latitude.to_radians();
        let lon_rad = self.longitude.to_radians();

        let angular_distance = ground_range_m / EARTH_RADIUS_M;

        let new_lat = (lat_rad.sin() * angular_distance.cos()
            + lat_rad.cos() * angular_distance.sin() * az_rad.cos())
        .asin();

        let new_lon = lon_rad
            + (az_rad.sin() * angular_distance.sin() * lat_rad.cos())
                .atan2(angular_distance.cos() - lat_rad.sin() * new_lat.sin());

        GeoPoint3D {
            latitude: new_lat.to_degrees(),
            longitude: new_lon.to_degrees(),
            altitude_meters,
        }
    }

    /// Convert geographic coordinates to radar-relative polar coordinates.
    ///
    /// Computes the azimuth to `point` and the slant range at which a beam at
    /// `elevation_angle_degrees` passes over it. With `phi = ground_range / R'` the angle
    /// subtended at the earth's centre, the slant range is
    /// `r = (R' + h0) sin(phi) / cos(theta + phi)`; see
    /// [`beam_height_meters`](Self::beam_height_meters) for the symbols. This is the exact
    /// inverse of [`polar_to_geo`](Self::polar_to_geo).
    ///
    /// Returns `None` when no beam at that elevation passes over the point (`theta + phi` of 90
    /// degrees or more, which includes elevations above 90 degrees and points more than a
    /// quarter of the effective earth's circumference away) or when an input is not finite.
    /// At the radar's own location the range is `0.0` and the azimuth `0.0`.
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_model::geo::{GeoPoint, RadarCoordinateSystem};
    /// use nexrad_model::meta::Site;
    ///
    /// let site = Site::builder(*b"KTLX", 35.3331, -97.2778).build()?;
    /// let coords = RadarCoordinateSystem::new(&site);
    ///
    /// // A point about 185 km due north.
    /// let north = GeoPoint { latitude: 37.0, longitude: -97.2778 };
    /// let polar = coords.geo_to_polar(north, 0.5).unwrap();
    /// assert!(polar.azimuth_angle_degrees < 0.01);
    /// assert!(polar.range_km > 180.0 && polar.range_km < 190.0);
    ///
    /// // A beam at 89 degrees passes over no point that far away.
    /// assert!(coords.geo_to_polar(north, 89.0).is_none());
    /// # Ok::<(), nexrad_model::result::Error>(())
    /// ```
    pub fn geo_to_polar(
        &self,
        point: GeoPoint,
        elevation_angle_degrees: f64,
    ) -> Option<PolarPoint> {
        let lat1 = self.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let lat2 = point.latitude.to_radians();
        let lon2 = point.longitude.to_radians();

        let d_lon = lon2 - lon1;

        // Azimuth (forward bearing)
        let y = d_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();
        let azimuth_rad = y.atan2(x);
        let azimuth_angle_degrees = (azimuth_rad.to_degrees() + 360.0) % 360.0;

        // Great-circle (haversine) distance
        let a = ((lat2 - lat1) / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let angular_distance = 2.0 * a.sqrt().asin();
        let ground_range_m = angular_distance * EARTH_RADIUS_M;

        // Slant range of the beam above that ground range
        let central_angle = ground_range_m / EFFECTIVE_EARTH_RADIUS_M;
        let elevation_rad = elevation_angle_degrees.to_radians();
        let antenna_radius_m = EFFECTIVE_EARTH_RADIUS_M + self.antenna_height_meters;
        let beam_angle_cos = (elevation_rad + central_angle).cos();
        if beam_angle_cos.is_nan() || beam_angle_cos <= 0.0 {
            // No beam at this elevation reaches the point, or an input was NaN.
            return None;
        }
        let range_m = antenna_radius_m * central_angle.sin() / beam_angle_cos;
        if !range_m.is_finite() || !azimuth_angle_degrees.is_finite() {
            return None;
        }

        Some(PolarPoint {
            azimuth_angle_degrees,
            range_km: range_m / 1000.0,
            elevation_angle_degrees,
        })
    }

    /// Compute the geographic bounding box of a radar sweep at the given maximum range.
    ///
    /// Returns the extent that encompasses the full 360-degree coverage circle at
    /// the specified range in kilometers.
    pub fn sweep_extent(&self, max_range_km: f64) -> GeoExtent {
        // Compute the geographic offset for the max range in each cardinal direction
        let north = self.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 0.0,
            range_km: max_range_km,
            elevation_angle_degrees: 0.0,
        });
        let south = self.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 180.0,
            range_km: max_range_km,
            elevation_angle_degrees: 0.0,
        });
        let east = self.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 90.0,
            range_km: max_range_km,
            elevation_angle_degrees: 0.0,
        });
        let west = self.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 270.0,
            range_km: max_range_km,
            elevation_angle_degrees: 0.0,
        });

        GeoExtent {
            min: GeoPoint {
                latitude: south.latitude,
                longitude: west.longitude,
            },
            max: GeoPoint {
                latitude: north.latitude,
                longitude: east.longitude,
            },
        }
    }

    /// Compute the geographic location of a specific gate in a radial.
    pub fn gate_location(
        &self,
        azimuth_angle_degrees: f64,
        elevation_angle_degrees: f64,
        gate_index: usize,
        first_gate_km: f64,
        gate_interval_km: f64,
    ) -> GeoPoint3D {
        let range_km = first_gate_km + (gate_index as f64) * gate_interval_km;
        self.polar_to_geo(PolarPoint {
            azimuth_angle_degrees,
            range_km,
            elevation_angle_degrees,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Site;

    fn test_site() -> Site {
        // KTLX (Oklahoma City) approximate coordinates
        Site::builder(*b"KTLX", 35.3331, -97.2778)
            .site_elevation_meters(370)
            .feedhorn_height_meters(10)
            .build()
            .unwrap()
    }

    #[test]
    fn test_geo_to_polar_at_the_radar() {
        let cs = RadarCoordinateSystem::new(&test_site());
        let here = GeoPoint {
            latitude: cs.latitude(),
            longitude: cs.longitude(),
        };
        for elevation in [-1.0, 0.5, 19.5, 90.0] {
            let polar = cs.geo_to_polar(here, elevation).unwrap();
            assert_eq!(polar.range_km, 0.0, "elevation {elevation}");
            assert_eq!(polar.azimuth_angle_degrees, 0.0);
            assert_eq!(polar.elevation_angle_degrees, elevation);
        }
    }

    #[test]
    fn test_geo_to_polar_none_when_no_beam_reaches_the_point() {
        let cs = RadarCoordinateSystem::new(&test_site());
        // About 185 km north: the central angle is about 1.25 degrees.
        let point = GeoPoint {
            latitude: 37.0,
            longitude: cs.longitude(),
        };
        assert!(cs.geo_to_polar(point, 88.5).is_some());
        assert!(cs.geo_to_polar(point, 89.0).is_none());
        assert!(cs.geo_to_polar(point, 90.0).is_none());
        assert!(cs.geo_to_polar(point, 120.0).is_none());

        // The antipode lies 135 degrees around the effective earth, beyond any beam.
        let antipode = GeoPoint {
            latitude: -cs.latitude(),
            longitude: cs.longitude() + 180.0,
        };
        assert!(cs.geo_to_polar(antipode, 0.0).is_none());
        assert!(cs.geo_to_polar(antipode, -10.0).is_none());
    }

    #[test]
    fn test_geo_to_polar_none_for_non_finite_input() {
        let cs = RadarCoordinateSystem::new(&test_site());
        let point = GeoPoint {
            latitude: 36.0,
            longitude: -97.0,
        };
        assert!(cs.geo_to_polar(point, f64::NAN).is_none());
        assert!(cs.geo_to_polar(point, f64::INFINITY).is_none());
        for (latitude, longitude) in [(f64::NAN, -97.0), (36.0, f64::NAN), (f64::INFINITY, -97.0)] {
            assert!(cs
                .geo_to_polar(
                    GeoPoint {
                        latitude,
                        longitude
                    },
                    0.5
                )
                .is_none());
        }
    }

    #[test]
    fn test_polar_to_geo_at_origin() {
        let cs = RadarCoordinateSystem::new(&test_site());
        let result = cs.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 0.0,
            range_km: 0.0,
            elevation_angle_degrees: 0.5,
        });

        // At zero range, the point should be at the radar location
        assert!((result.latitude - 35.3331).abs() < 0.001);
        assert!((result.longitude - (-97.2778)).abs() < 0.001);
    }

    #[test]
    fn test_polar_to_geo_northward() {
        let cs = RadarCoordinateSystem::new(&test_site());
        let result = cs.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 0.0,
            range_km: 100.0,
            elevation_angle_degrees: 0.5,
        });

        // 100km north should increase latitude by roughly 0.9 degrees
        assert!(result.latitude > 35.3331);
        assert!((result.latitude - 35.3331 - 0.9).abs() < 0.1);
        // Longitude should barely change for due-north azimuth
        assert!((result.longitude - (-97.2778)).abs() < 0.01);
    }

    #[test]
    fn test_sweep_extent_symmetric() {
        let cs = RadarCoordinateSystem::new(&test_site());
        let extent = cs.sweep_extent(230.0);

        // Extent should be roughly symmetric around the radar
        let lat_center = (extent.min.latitude + extent.max.latitude) / 2.0;
        let lon_center = (extent.min.longitude + extent.max.longitude) / 2.0;

        assert!((lat_center - 35.3331).abs() < 0.05);
        assert!((lon_center - (-97.2778)).abs() < 0.05);
    }

    #[test]
    fn test_beam_height_increases_with_range() {
        let cs = RadarCoordinateSystem::new(&test_site());
        let near = cs.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 0.0,
            range_km: 50.0,
            elevation_angle_degrees: 0.5,
        });
        let far = cs.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 0.0,
            range_km: 200.0,
            elevation_angle_degrees: 0.5,
        });

        assert!(far.altitude_meters > near.altitude_meters);
    }

    #[test]
    fn test_beam_height_increases_with_elevation() {
        let cs = RadarCoordinateSystem::new(&test_site());
        let low = cs.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 0.0,
            range_km: 100.0,
            elevation_angle_degrees: 0.5,
        });
        let high = cs.polar_to_geo(PolarPoint {
            azimuth_angle_degrees: 0.0,
            range_km: 100.0,
            elevation_angle_degrees: 5.0,
        });

        assert!(high.altitude_meters > low.altitude_meters);
    }
}
