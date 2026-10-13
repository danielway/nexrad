//! Render result types with metadata for geographic placement and data inspection.
//!
//! The [`RenderResult`] type bundles a rendered image with all the metadata a consuming
//! application needs to accurately place it on a map, draw overlays, and inspect data values.

use crate::RgbaImage;
use nexrad_model::data::{GateStatus, SweepField};
use nexrad_model::geo::{GeoExtent, GeoPoint, GeoPoint3D, PolarPoint, RadarCoordinateSystem};
use std::path::Path;

/// The result of a render operation.
///
/// Bundles the rendered RGBA image with metadata describing the pixel-to-coordinate
/// mapping, enabling a consuming application to accurately place the image on a map,
/// draw overlays, and query data values.
#[derive(Debug)]
pub struct RenderResult {
    image: RgbaImage,
    metadata: RenderMetadata,
}

/// How image pixels map to the coordinate space of the rendered data.
///
/// Each render entry point produces a different projection: [`render_sweep`](crate::render_sweep)
/// produces [`Polar`](Projection::Polar), [`render_cartesian`](crate::render_cartesian) produces
/// [`Geographic`](Projection::Geographic) and [`render_vertical`](crate::render_vertical)
/// produces [`Vertical`](Projection::Vertical).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Projection {
    /// A radar-centred plan view with north at the top.
    Polar {
        /// Pixel coordinate of the radar site.
        center_pixel: (f64, f64),
        /// Scale factor in pixels per kilometre.
        pixels_per_km: f64,
        /// Maximum range of the rendered data in kilometres.
        max_range_km: f64,
        /// Elevation angle of the rendered sweep in degrees.
        elevation_degrees: f64,
    },
    /// A latitude/longitude grid where x spans the extent west to east and y spans
    /// it north to south.
    Geographic {
        /// Geographic extent covered by the image.
        extent: GeoExtent,
    },
    /// A vertical cross-section where x spans distance from the radar and y spans
    /// altitude (the top of the image is the highest altitude).
    Vertical {
        /// Horizontal axis range `(min, max)` in kilometres.
        distance_range_km: (f64, f64),
        /// Vertical axis range `(min, max)` in metres.
        altitude_range_m: (f64, f64),
    },
}

/// Metadata describing the pixel-to-coordinate mapping of a rendered image.
///
/// This is everything a consumer needs to place and query the rendered image.
/// Conversions that only make sense for a particular [`Projection`] return `None`
/// for the others.
#[derive(Debug, Clone)]
pub struct RenderMetadata {
    width: u32,
    height: u32,
    projection: Projection,
    geo_extent: Option<GeoExtent>,
    coord_system: Option<RadarCoordinateSystem>,
}

/// Result of querying a data value at a specific point.
#[derive(Debug, Clone)]
pub struct PointQuery {
    /// The polar coordinate of the queried point.
    pub polar: PolarPoint,
    /// The geographic coordinate of the queried point (if coordinate system available).
    pub geo: Option<GeoPoint3D>,
    /// The data value at the queried point.
    pub value: f32,
    /// The gate status at the queried point.
    pub status: GateStatus,
}

impl RenderResult {
    /// Create a new render result from an image and metadata.
    pub(crate) fn new(image: RgbaImage, metadata: RenderMetadata) -> Self {
        Self { image, metadata }
    }

    /// The rendered RGBA image.
    pub fn image(&self) -> &RgbaImage {
        &self.image
    }

    /// Consume the result and return the image.
    pub fn into_image(self) -> RgbaImage {
        self.image
    }

    /// Metadata describing the pixel-to-coordinate mapping.
    pub fn metadata(&self) -> &RenderMetadata {
        &self.metadata
    }

    /// Save the rendered image to a file.
    ///
    /// The output format is inferred from the file extension (e.g., `.png`, `.jpg`).
    /// This is a convenience wrapper around [`image::RgbaImage::save`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written or the format is unsupported.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> crate::result::Result<()> {
        self.image.save(path)?;
        Ok(())
    }

    /// Query the data value at a pixel coordinate.
    ///
    /// Uses the metadata's pixel-to-polar conversion and then looks up the value
    /// in the provided field. Returns `None` if the image does not use a
    /// [`Polar`](Projection::Polar) projection or the pixel is outside the rendered area.
    pub fn query_pixel(&self, field: &SweepField, x: f64, y: f64) -> Option<PointQuery> {
        let polar = self.metadata.pixel_to_polar(x, y)?;
        self.build_query(field, polar)
    }

    /// Query the data value at a polar coordinate.
    ///
    /// Returns `None` if the image does not use a [`Polar`](Projection::Polar)
    /// projection or the point is outside the field.
    pub fn query_polar(
        &self,
        field: &SweepField,
        azimuth_degrees: f64,
        range_km: f64,
    ) -> Option<PointQuery> {
        let polar = PolarPoint {
            azimuth_degrees,
            range_km,
            elevation_degrees: self.metadata.elevation_degrees()?,
        };
        self.build_query(field, polar)
    }

    /// Query the data value at a geographic coordinate.
    ///
    /// Requires a [`Polar`](Projection::Polar) projection and a coordinate system in
    /// the metadata. Returns `None` otherwise, or if the point is outside the field.
    pub fn query_geo(&self, field: &SweepField, point: &GeoPoint) -> Option<PointQuery> {
        let elevation = self.metadata.elevation_degrees()?;
        let coord_system = self.metadata.coord_system.as_ref()?;
        let polar = coord_system.geo_to_polar(*point, elevation);
        self.build_query(field, polar)
    }

    fn build_query(&self, field: &SweepField, polar: PolarPoint) -> Option<PointQuery> {
        let (value, status) = field.value_at_polar(polar.azimuth_degrees, polar.range_km)?;

        let geo = self
            .metadata
            .coord_system
            .as_ref()
            .map(|cs| cs.polar_to_geo(polar));

        Some(PointQuery {
            polar,
            geo,
            value,
            status,
        })
    }
}

impl RenderMetadata {
    /// Creates metadata for an image of the given size and projection.
    pub(crate) fn new(
        width: u32,
        height: u32,
        projection: Projection,
        geo_extent: Option<GeoExtent>,
        coord_system: Option<RadarCoordinateSystem>,
    ) -> Self {
        Self {
            width,
            height,
            projection,
            geo_extent,
            coord_system,
        }
    }

    /// Image width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Image height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// How image pixels map to the coordinate space of the rendered data.
    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    /// Pixel coordinate of the radar site. `Some` only for [`Projection::Polar`].
    pub fn center_pixel(&self) -> Option<(f64, f64)> {
        match self.projection {
            Projection::Polar { center_pixel, .. } => Some(center_pixel),
            Projection::Geographic { .. } | Projection::Vertical { .. } => None,
        }
    }

    /// Scale factor in pixels per kilometre. `Some` only for [`Projection::Polar`].
    pub fn pixels_per_km(&self) -> Option<f64> {
        match self.projection {
            Projection::Polar { pixels_per_km, .. } => Some(pixels_per_km),
            Projection::Geographic { .. } | Projection::Vertical { .. } => None,
        }
    }

    /// Maximum range of the rendered data in kilometres. `Some` only for
    /// [`Projection::Polar`].
    pub fn max_range_km(&self) -> Option<f64> {
        match self.projection {
            Projection::Polar { max_range_km, .. } => Some(max_range_km),
            Projection::Geographic { .. } | Projection::Vertical { .. } => None,
        }
    }

    /// Elevation angle of the rendered sweep in degrees. `Some` only for
    /// [`Projection::Polar`].
    pub fn elevation_degrees(&self) -> Option<f64> {
        match self.projection {
            Projection::Polar {
                elevation_degrees, ..
            } => Some(elevation_degrees),
            Projection::Geographic { .. } | Projection::Vertical { .. } => None,
        }
    }

    /// Geographic extent of the rendered area.
    ///
    /// For [`Projection::Polar`] this is the extent from the render options, or the
    /// sweep's extent computed from the coordinate system; `None` if neither was
    /// provided. For [`Projection::Geographic`] it is the field's extent. For
    /// [`Projection::Vertical`] it is always `None`.
    pub fn geo_extent(&self) -> Option<&GeoExtent> {
        self.geo_extent.as_ref()
    }

    /// The coordinate system used (if available), enabling conversion between
    /// pixel, polar, and geographic coordinates.
    pub fn coord_system(&self) -> Option<&RadarCoordinateSystem> {
        self.coord_system.as_ref()
    }

    /// Convert a pixel coordinate to polar (azimuth, range) coordinates.
    ///
    /// Returns `None` if the image does not use a [`Polar`](Projection::Polar)
    /// projection or the pixel is outside the rendered radar coverage area.
    pub fn pixel_to_polar(&self, x: f64, y: f64) -> Option<PolarPoint> {
        let Projection::Polar {
            center_pixel,
            pixels_per_km,
            max_range_km,
            elevation_degrees,
        } = self.projection
        else {
            return None;
        };

        let dx = x - center_pixel.0;
        let dy = y - center_pixel.1;
        let distance_px = (dx * dx + dy * dy).sqrt();
        let range_km = distance_px / pixels_per_km;

        if range_km > max_range_km {
            return None;
        }

        let azimuth_rad = dx.atan2(-dy);
        let azimuth_degrees = (azimuth_rad.to_degrees() + 360.0) % 360.0;

        Some(PolarPoint {
            azimuth_degrees,
            range_km,
            elevation_degrees,
        })
    }

    /// Convert polar coordinates to a pixel coordinate.
    ///
    /// Returns `None` if the image does not use a [`Polar`](Projection::Polar) projection.
    pub fn polar_to_pixel(&self, azimuth_degrees: f64, range_km: f64) -> Option<(f64, f64)> {
        let center_pixel = self.center_pixel()?;
        let distance_px = self.km_to_pixel_distance(range_km)?;
        let az_rad = azimuth_degrees.to_radians();
        let x = center_pixel.0 + distance_px * az_rad.sin();
        let y = center_pixel.1 - distance_px * az_rad.cos();
        Some((x, y))
    }

    /// Convert a pixel coordinate to geographic coordinates.
    ///
    /// For [`Projection::Polar`] this requires a coordinate system and returns `None`
    /// without one or when the pixel is outside the radar coverage area. For
    /// [`Projection::Geographic`] the pixel is mapped linearly through the extent
    /// (altitude 0) and `None` is returned outside the image. For
    /// [`Projection::Vertical`] it always returns `None`.
    pub fn pixel_to_geo(&self, x: f64, y: f64) -> Option<GeoPoint3D> {
        match &self.projection {
            Projection::Polar { .. } => {
                let polar = self.pixel_to_polar(x, y)?;
                let coord_system = self.coord_system.as_ref()?;
                Some(coord_system.polar_to_geo(polar))
            }
            Projection::Geographic { extent } => {
                let width = f64::from(self.width);
                let height = f64::from(self.height);
                if !(0.0..=width).contains(&x) || !(0.0..=height).contains(&y) {
                    return None;
                }
                let lon_span = extent.max.longitude - extent.min.longitude;
                let lat_span = extent.max.latitude - extent.min.latitude;
                Some(GeoPoint3D {
                    latitude: extent.max.latitude - y / height * lat_span,
                    longitude: extent.min.longitude + x / width * lon_span,
                    altitude_meters: 0.0,
                })
            }
            Projection::Vertical { .. } => None,
        }
    }

    /// Convert geographic coordinates to a pixel coordinate.
    ///
    /// For [`Projection::Polar`] this requires a coordinate system and returns `None`
    /// without one. For [`Projection::Geographic`] the point is mapped linearly
    /// through the extent; points outside the extent map to pixels outside the
    /// image. For [`Projection::Vertical`] it always returns `None`.
    pub fn geo_to_pixel(&self, point: &GeoPoint) -> Option<(f64, f64)> {
        match &self.projection {
            Projection::Polar {
                elevation_degrees, ..
            } => {
                let coord_system = self.coord_system.as_ref()?;
                let polar = coord_system.geo_to_polar(*point, *elevation_degrees);
                self.polar_to_pixel(polar.azimuth_degrees, polar.range_km)
            }
            Projection::Geographic { extent } => {
                let lon_span = extent.max.longitude - extent.min.longitude;
                let lat_span = extent.max.latitude - extent.min.latitude;
                let x_frac = (point.longitude - extent.min.longitude) / lon_span;
                let y_frac = (extent.max.latitude - point.latitude) / lat_span;
                Some((
                    x_frac * f64::from(self.width),
                    y_frac * f64::from(self.height),
                ))
            }
            Projection::Vertical { .. } => None,
        }
    }

    /// Get the range in km for a given pixel distance from the radar.
    ///
    /// Returns `None` if the image does not use a [`Polar`](Projection::Polar) projection.
    pub fn pixel_distance_to_km(&self, pixels: f64) -> Option<f64> {
        Some(pixels / self.pixels_per_km()?)
    }

    /// Get the pixel distance from the radar for a given range in km.
    ///
    /// Returns `None` if the image does not use a [`Polar`](Projection::Polar) projection.
    pub fn km_to_pixel_distance(&self, km: f64) -> Option<f64> {
        Some(km * self.pixels_per_km()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extent() -> GeoExtent {
        GeoExtent {
            min: GeoPoint {
                latitude: 40.0,
                longitude: -100.0,
            },
            max: GeoPoint {
                latitude: 42.0,
                longitude: -96.0,
            },
        }
    }

    fn geographic() -> RenderMetadata {
        let extent = extent();
        RenderMetadata::new(
            400,
            200,
            Projection::Geographic { extent },
            Some(extent),
            None,
        )
    }

    fn vertical() -> RenderMetadata {
        RenderMetadata::new(
            600,
            300,
            Projection::Vertical {
                distance_range_km: (0.0, 230.0),
                altitude_range_m: (0.0, 18_000.0),
            },
            None,
            None,
        )
    }

    fn polar() -> RenderMetadata {
        RenderMetadata::new(
            200,
            200,
            Projection::Polar {
                center_pixel: (100.0, 100.0),
                pixels_per_km: 0.5,
                max_range_km: 200.0,
                elevation_degrees: 0.5,
            },
            None,
            None,
        )
    }

    fn point() -> GeoPoint {
        GeoPoint {
            latitude: 41.5,
            longitude: -99.0,
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_geo(actual: Option<GeoPoint3D>, latitude: f64, longitude: f64) {
        let Some(actual) = actual else {
            panic!("expected a geographic point, got None");
        };
        assert_close(actual.latitude, latitude);
        assert_close(actual.longitude, longitude);
        assert_close(actual.altitude_meters, 0.0);
    }

    fn assert_pixel(actual: Option<(f64, f64)>, x: f64, y: f64) {
        let Some((actual_x, actual_y)) = actual else {
            panic!("expected a pixel, got None");
        };
        assert_close(actual_x, x);
        assert_close(actual_y, y);
    }

    #[test]
    fn geographic_pixel_to_geo_maps_linearly() {
        let meta = geographic();
        assert_geo(meta.pixel_to_geo(0.0, 0.0), 42.0, -100.0);
        assert_geo(meta.pixel_to_geo(400.0, 200.0), 40.0, -96.0);
        assert_geo(meta.pixel_to_geo(100.0, 50.0), 41.5, -99.0);
    }

    #[test]
    fn geographic_pixel_to_geo_outside_image_is_none() {
        let meta = geographic();
        assert!(meta.pixel_to_geo(-1.0, 10.0).is_none());
        assert!(meta.pixel_to_geo(10.0, 201.0).is_none());
    }

    #[test]
    fn geographic_geo_to_pixel_maps_linearly() {
        let meta = geographic();
        assert_pixel(meta.geo_to_pixel(&point()), 100.0, 50.0);
        assert_pixel(
            meta.geo_to_pixel(&GeoPoint {
                latitude: 42.0,
                longitude: -100.0,
            }),
            0.0,
            0.0,
        );
    }

    #[test]
    fn geographic_has_no_polar_accessors() {
        let meta = geographic();
        assert_eq!(meta.geo_extent(), Some(&extent()));
        assert!(meta.center_pixel().is_none());
        assert!(meta.pixels_per_km().is_none());
        assert!(meta.max_range_km().is_none());
        assert!(meta.elevation_degrees().is_none());
        assert!(meta.pixel_to_polar(10.0, 10.0).is_none());
        assert!(meta.polar_to_pixel(90.0, 10.0).is_none());
        assert!(meta.pixel_distance_to_km(10.0).is_none());
        assert!(meta.km_to_pixel_distance(10.0).is_none());
    }

    #[test]
    fn vertical_has_no_polar_or_geo_conversions() {
        let meta = vertical();
        assert!(meta.geo_extent().is_none());
        assert!(meta.center_pixel().is_none());
        assert!(meta.pixels_per_km().is_none());
        assert!(meta.max_range_km().is_none());
        assert!(meta.elevation_degrees().is_none());
        assert!(meta.pixel_to_polar(10.0, 10.0).is_none());
        assert!(meta.polar_to_pixel(90.0, 10.0).is_none());
        assert!(meta.pixel_to_geo(10.0, 10.0).is_none());
        assert!(meta.geo_to_pixel(&point()).is_none());
        assert!(meta.pixel_distance_to_km(10.0).is_none());
        assert!(meta.km_to_pixel_distance(10.0).is_none());
    }

    #[test]
    fn polar_without_coord_system_has_no_geo_conversions() {
        let meta = polar();
        assert!(meta.pixel_to_geo(110.0, 100.0).is_none());
        assert!(meta.geo_to_pixel(&point()).is_none());
    }

    #[test]
    fn polar_conversions_round_trip() {
        let meta = polar();
        assert_eq!(meta.km_to_pixel_distance(100.0), Some(50.0));
        assert_eq!(meta.pixel_distance_to_km(50.0), Some(100.0));
        assert_pixel(meta.polar_to_pixel(90.0, 100.0), 150.0, 100.0);

        let Some(back) = meta.pixel_to_polar(150.0, 100.0) else {
            panic!("pixel should be inside coverage");
        };
        assert_close(back.azimuth_degrees, 90.0);
        assert_close(back.range_km, 100.0);
        assert_close(back.elevation_degrees, 0.5);
        assert!(meta.pixel_to_polar(-200.0, 100.0).is_none());
    }
}
