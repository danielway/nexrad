//! Render result types with metadata for geographic placement and data inspection.
//!
//! The [`RenderResult`] type bundles a rendered image with all the metadata a consuming
//! application needs to accurately place it on a map, draw overlays, and inspect data values.

use crate::RgbaImage;
use nexrad_model::data::{CartesianField, GateStatus, SweepField, VerticalField};
use nexrad_model::geo::{GeoExtent, GeoPoint, GeoPoint3D, PolarPoint, RadarCoordinateSystem};
use std::io::{Cursor, Write};
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
/// [`render_sweep`](crate::render_sweep) produces [`Polar`](Projection::Polar), or
/// [`Geographic`](Projection::Geographic) when the options set an extent;
/// [`render_cartesian`](crate::render_cartesian) produces
/// [`Geographic`](Projection::Geographic) and [`render_vertical`](crate::render_vertical)
/// produces [`Vertical`](Projection::Vertical).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Projection {
    /// A radar-centred plan view with north at the top.
    #[non_exhaustive]
    Polar {
        /// Pixel coordinate of the radar site.
        center_pixel: (f64, f64),
        /// Scale factor in pixels per kilometre.
        pixels_per_km: f64,
        /// Maximum range of the rendered data in kilometres.
        max_range_km: f64,
        /// Elevation angle of the rendered sweep in degrees.
        elevation_angle_degrees: f64,
    },
    /// A latitude/longitude grid where x spans the extent west to east and y spans
    /// it north to south.
    #[non_exhaustive]
    Geographic {
        /// Geographic extent covered by the image.
        extent: GeoExtent,
    },
    /// A vertical cross-section where x spans distance from the radar and y spans
    /// altitude (the top of the image is the highest altitude).
    #[non_exhaustive]
    Vertical {
        /// Horizontal axis range `(min, max)` in kilometres.
        distance_range_km: (f64, f64),
        /// Vertical axis range `(min, max)` in metres.
        altitude_range_meters: (f64, f64),
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
#[non_exhaustive]
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
    /// The output format is inferred from the file extension and must be one whose
    /// [`image`] codec is compiled in. This crate enables only the `png` feature of
    /// `image`; for other formats (`.jpg`, `.webp`, ...) enable the matching feature
    /// on your own `image` dependency (for example
    /// `image = { version = "0.25", default-features = false, features = ["jpeg"] }`),
    /// which Cargo's feature unification makes available here.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`](crate::Error::Image) if the file cannot be written or
    /// the format is unsupported.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> crate::result::Result<()> {
        self.image.save(path)?;
        Ok(())
    }

    /// Encode the rendered image in `format` and write it to `writer`, without touching
    /// the filesystem.
    ///
    /// As with [`save`](Self::save), the format's [`image`] codec must be compiled in:
    /// [`ImageFormat::Png`](image::ImageFormat::Png) always is, other formats need the
    /// matching feature enabled on your own `image` dependency.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Image`](crate::Error::Image) if the format is unsupported or
    /// encoding or writing fails.
    ///
    /// # Example
    ///
    /// ```
    /// use nexrad_render::image::ImageFormat;
    /// # use nexrad_model::data::Product;
    /// # use nexrad_render::{default_scale, render_sweep, RenderOptions};
    /// # let field = nexrad_model::data::SweepField::builder("Reflectivity", "dBZ")
    /// #     .product(nexrad_model::data::Product::Reflectivity)
    /// #     .elevation_angle_degrees(0.5)
    /// #     .azimuth_angles_degrees((0..360).map(|a| a as f32).collect())
    /// #     .first_gate_range_km(2.0)
    /// #     .gate_interval_km(0.25)
    /// #     .gate_count(400)
    /// #     .build()?;
    /// # let result = render_sweep(&field, &default_scale(Product::Reflectivity), &RenderOptions::new(64, 64))?;
    ///
    /// let mut png = Vec::new();
    /// result.write_to(&mut png, ImageFormat::Png)?;
    /// # assert!(png.starts_with(b"\x89PNG"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_to(
        &self,
        writer: &mut impl Write,
        format: image::ImageFormat,
    ) -> crate::result::Result<()> {
        // Some encoders need to seek, so encode into memory first.
        let mut encoded = Cursor::new(Vec::new());
        self.image.write_to(&mut encoded, format)?;
        writer
            .write_all(encoded.get_ref())
            .map_err(image::ImageError::IoError)?;
        Ok(())
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
    pub fn elevation_angle_degrees(&self) -> Option<f64> {
        match self.projection {
            Projection::Polar {
                elevation_angle_degrees,
                ..
            } => Some(elevation_angle_degrees),
            Projection::Geographic { .. } | Projection::Vertical { .. } => None,
        }
    }

    /// Geographic extent of the rendered area.
    ///
    /// For [`Projection::Polar`] this is the sweep's extent computed from the
    /// coordinate system, or `None` without one. For [`Projection::Geographic`] it is
    /// the extent the image covers: the render options' extent for a sweep, the
    /// field's extent for a Cartesian field. For [`Projection::Vertical`] it is always
    /// `None`.
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
            elevation_angle_degrees,
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
        let azimuth_angle_degrees = (azimuth_rad.to_degrees() + 360.0) % 360.0;

        Some(PolarPoint {
            azimuth_angle_degrees,
            range_km,
            elevation_angle_degrees,
        })
    }

    /// Convert polar coordinates to a pixel coordinate.
    ///
    /// Returns `None` if the image does not use a [`Polar`](Projection::Polar) projection.
    pub fn polar_to_pixel(&self, azimuth_angle_degrees: f64, range_km: f64) -> Option<(f64, f64)> {
        let center_pixel = self.center_pixel()?;
        let distance_px = self.km_to_pixel_distance(range_km)?;
        let az_rad = azimuth_angle_degrees.to_radians();
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
                elevation_angle_degrees,
                ..
            } => {
                let coord_system = self.coord_system.as_ref()?;
                let polar = coord_system.geo_to_polar(*point, *elevation_angle_degrees);
                self.polar_to_pixel(polar.azimuth_angle_degrees, polar.range_km)
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

    /// Convert a pixel coordinate to `(distance_km, altitude_meters)`.
    ///
    /// Returns `None` if the image does not use a [`Vertical`](Projection::Vertical)
    /// projection or the pixel is outside the image.
    pub fn pixel_to_vertical(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        let Projection::Vertical {
            distance_range_km: (d_min, d_max),
            altitude_range_meters: (a_min, a_max),
        } = self.projection
        else {
            return None;
        };
        let width = f64::from(self.width);
        let height = f64::from(self.height);
        if !(0.0..=width).contains(&x) || !(0.0..=height).contains(&y) {
            return None;
        }
        Some((
            d_min + x / width * (d_max - d_min),
            a_max - y / height * (a_max - a_min),
        ))
    }

    /// Convert a distance/altitude coordinate to a pixel coordinate.
    ///
    /// Returns `None` if the image does not use a [`Vertical`](Projection::Vertical)
    /// projection. Coordinates outside the cross-section map to pixels outside the
    /// image.
    pub fn vertical_to_pixel(&self, distance_km: f64, altitude_meters: f64) -> Option<(f64, f64)> {
        let Projection::Vertical {
            distance_range_km: (d_min, d_max),
            altitude_range_meters: (a_min, a_max),
        } = self.projection
        else {
            return None;
        };
        Some((
            (distance_km - d_min) / (d_max - d_min) * f64::from(self.width),
            (a_max - altitude_meters) / (a_max - a_min) * f64::from(self.height),
        ))
    }

    /// Query the sweep value at a pixel coordinate.
    ///
    /// `field` must be the field that was rendered with
    /// [`render_sweep`](crate::render_sweep); the metadata cannot check this. For
    /// [`Projection::Polar`] the pixel is mapped through
    /// [`pixel_to_polar`](Self::pixel_to_polar); for [`Projection::Geographic`] (a sweep
    /// rendered into an extent) through [`pixel_to_geo`](Self::pixel_to_geo) and the
    /// coordinate system. Returns `None` for other projections, without the needed
    /// coordinate system, or when the pixel is outside the field.
    pub fn query_pixel(&self, field: &SweepField, x: f64, y: f64) -> Option<PointQuery> {
        match self.projection {
            Projection::Polar { .. } => {
                let polar = self.pixel_to_polar(x, y)?;
                self.build_query(field, polar)
            }
            Projection::Geographic { .. } => {
                let point = self.pixel_to_geo(x, y)?;
                self.query_geo(
                    field,
                    &GeoPoint {
                        latitude: point.latitude,
                        longitude: point.longitude,
                    },
                )
            }
            Projection::Vertical { .. } => None,
        }
    }

    /// Query the sweep value at a polar coordinate.
    ///
    /// `field` must be the field that was rendered with
    /// [`render_sweep`](crate::render_sweep). Returns `None` if the image is not a
    /// sweep render ([`Projection::Polar`], or [`Projection::Geographic`] with a
    /// coordinate system) or the point is outside the field.
    pub fn query_polar(
        &self,
        field: &SweepField,
        azimuth_angle_degrees: f64,
        range_km: f64,
    ) -> Option<PointQuery> {
        let polar = PolarPoint {
            azimuth_angle_degrees,
            range_km,
            elevation_angle_degrees: self.sweep_elevation(field)?,
        };
        self.build_query(field, polar)
    }

    /// Query the sweep value at a geographic coordinate.
    ///
    /// `field` must be the field that was rendered with
    /// [`render_sweep`](crate::render_sweep). Requires a coordinate system and a
    /// [`Polar`](Projection::Polar) or [`Geographic`](Projection::Geographic)
    /// projection. Returns `None` otherwise, or if the point is outside the field.
    pub fn query_geo(&self, field: &SweepField, point: &GeoPoint) -> Option<PointQuery> {
        let elevation = self.sweep_elevation(field)?;
        let coord_system = self.coord_system.as_ref()?;
        let polar = coord_system.geo_to_polar(*point, elevation);
        self.build_query(field, polar)
    }

    /// Query the value of a Cartesian field at a pixel coordinate.
    ///
    /// `field` must be the field that was rendered with
    /// [`render_cartesian`](crate::render_cartesian). The pixel is mapped through
    /// [`pixel_to_geo`](Self::pixel_to_geo) and looked up with
    /// [`CartesianField::value_at_geo`]. Returns `None` if the pixel has no geographic
    /// position or is outside the field.
    pub fn query_cartesian(
        &self,
        field: &CartesianField,
        x: f64,
        y: f64,
    ) -> Option<(f32, GateStatus)> {
        let point = self.pixel_to_geo(x, y)?;
        field.value_at_geo(GeoPoint {
            latitude: point.latitude,
            longitude: point.longitude,
        })
    }

    /// Query the value of a vertical field at a pixel coordinate.
    ///
    /// `field` must be the field that was rendered with
    /// [`render_vertical`](crate::render_vertical). The pixel is mapped through
    /// [`pixel_to_vertical`](Self::pixel_to_vertical) and looked up with
    /// [`VerticalField::value_at`]. Returns `None` if the image does not use a
    /// [`Vertical`](Projection::Vertical) projection or the pixel is outside the field.
    pub fn query_vertical(
        &self,
        field: &VerticalField,
        x: f64,
        y: f64,
    ) -> Option<(f32, GateStatus)> {
        let (distance_km, altitude_meters) = self.pixel_to_vertical(x, y)?;
        field.value_at(distance_km, altitude_meters)
    }

    /// The elevation angle of a sweep render: the projection's for a polar image, the
    /// field's for a geographic image with a coordinate system.
    fn sweep_elevation(&self, field: &SweepField) -> Option<f64> {
        match self.projection {
            Projection::Polar {
                elevation_angle_degrees,
                ..
            } => Some(elevation_angle_degrees),
            Projection::Geographic { .. } if self.coord_system.is_some() => {
                Some(f64::from(field.elevation_angle_degrees()))
            }
            Projection::Geographic { .. } | Projection::Vertical { .. } => None,
        }
    }

    fn build_query(&self, field: &SweepField, polar: PolarPoint) -> Option<PointQuery> {
        let (value, status) = field.value_at_polar(polar.azimuth_angle_degrees, polar.range_km)?;

        let geo = self.coord_system.as_ref().map(|cs| cs.polar_to_geo(polar));

        Some(PointQuery {
            polar,
            geo,
            value,
            status,
        })
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
                altitude_range_meters: (0.0, 18_000.0),
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
                elevation_angle_degrees: 0.5,
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
        assert!(meta.elevation_angle_degrees().is_none());
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
        assert!(meta.elevation_angle_degrees().is_none());
        assert!(meta.pixel_to_polar(10.0, 10.0).is_none());
        assert!(meta.polar_to_pixel(90.0, 10.0).is_none());
        assert!(meta.pixel_to_geo(10.0, 10.0).is_none());
        assert!(meta.geo_to_pixel(&point()).is_none());
        assert!(meta.pixel_distance_to_km(10.0).is_none());
        assert!(meta.km_to_pixel_distance(10.0).is_none());
    }

    #[test]
    fn vertical_pixel_mapping_round_trips() {
        let meta = vertical();
        assert_pixel(meta.pixel_to_vertical(0.0, 0.0), 0.0, 18_000.0);
        assert_pixel(meta.pixel_to_vertical(600.0, 300.0), 230.0, 0.0);
        assert_pixel(meta.pixel_to_vertical(300.0, 100.0), 115.0, 12_000.0);
        assert_pixel(meta.vertical_to_pixel(115.0, 12_000.0), 300.0, 100.0);

        let (x, y) = (123.0, 45.0);
        let Some((distance, altitude)) = meta.pixel_to_vertical(x, y) else {
            panic!("pixel is inside the image");
        };
        assert_pixel(meta.vertical_to_pixel(distance, altitude), x, y);

        assert!(meta.pixel_to_vertical(-1.0, 10.0).is_none());
        assert!(meta.pixel_to_vertical(10.0, 301.0).is_none());
    }

    #[test]
    fn non_vertical_has_no_vertical_conversions() {
        for meta in [geographic(), polar()] {
            assert!(meta.pixel_to_vertical(10.0, 10.0).is_none());
            assert!(meta.vertical_to_pixel(10.0, 1_000.0).is_none());
        }
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
        assert_close(back.azimuth_angle_degrees, 90.0);
        assert_close(back.range_km, 100.0);
        assert_close(back.elevation_angle_degrees, 0.5);
        assert!(meta.pixel_to_polar(-200.0, 100.0).is_none());
    }
}
