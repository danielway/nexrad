//! Render result types with metadata for geographic placement and data inspection.
//!
//! The [`RenderResult`] type bundles a rendered image with all the metadata a consuming
//! application needs to accurately place it on a map, draw overlays, and inspect data values.

use crate::RgbaImage;
use image::ImageError;
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

/// Metadata describing the pixel-to-coordinate mapping of a rendered image.
///
/// This is everything a consumer needs to place and query the rendered image.
#[derive(Debug, Clone)]
pub struct RenderMetadata {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Center of the image in pixel coordinates.
    pub center_pixel: (f64, f64),
    /// Scale factor: pixels per kilometer.
    pub pixels_per_km: f64,
    /// Maximum range of the rendered data in km.
    pub max_range_km: f64,
    /// The elevation angle of the rendered sweep (if applicable).
    pub elevation_degrees: Option<f32>,
    /// Geographic extent of the rendered area (if coordinate system was provided).
    pub geo_extent: Option<GeoExtent>,
    /// The coordinate system used (if available), enabling conversion between
    /// pixel, polar, and geographic coordinates.
    pub coord_system: Option<RadarCoordinateSystem>,
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
    pub fn new(image: RgbaImage, metadata: RenderMetadata) -> Self {
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
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::result::Result<(), ImageError> {
        self.image.save(path)
    }

    /// Query the data value at a pixel coordinate.
    ///
    /// Uses the metadata's pixel-to-polar conversion and then looks up the value
    /// in the provided field. Returns `None` if the pixel is outside the rendered area.
    pub fn query_pixel(&self, field: &SweepField, x: f64, y: f64) -> Option<PointQuery> {
        let polar = self.metadata.pixel_to_polar(x, y)?;
        self.build_query(field, polar)
    }

    /// Query the data value at a polar coordinate.
    pub fn query_polar(
        &self,
        field: &SweepField,
        azimuth_degrees: f32,
        range_km: f64,
    ) -> Option<PointQuery> {
        let polar = PolarPoint {
            azimuth_degrees,
            range_km,
            elevation_degrees: self.metadata.elevation_degrees.unwrap_or(0.0),
        };
        self.build_query(field, polar)
    }

    /// Query the data value at a geographic coordinate.
    ///
    /// Requires a coordinate system in the metadata. Returns `None` if no coordinate
    /// system is available or the point is outside the rendered area.
    pub fn query_geo(&self, field: &SweepField, point: &GeoPoint) -> Option<PointQuery> {
        let coord_system = self.metadata.coord_system.as_ref()?;
        let elevation = self.metadata.elevation_degrees.unwrap_or(0.0);
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
    /// Convert a pixel coordinate to polar (azimuth, range) coordinates.
    ///
    /// Returns `None` if the pixel is outside the rendered radar coverage area.
    pub fn pixel_to_polar(&self, x: f64, y: f64) -> Option<PolarPoint> {
        let dx = x - self.center_pixel.0;
        let dy = y - self.center_pixel.1;
        let distance_px = (dx * dx + dy * dy).sqrt();
        let range_km = distance_px / self.pixels_per_km;

        if range_km > self.max_range_km {
            return None;
        }

        let azimuth_rad = dx.atan2(-dy);
        let azimuth_degrees = (azimuth_rad.to_degrees() + 360.0) % 360.0;

        Some(PolarPoint {
            azimuth_degrees: azimuth_degrees as f32,
            range_km,
            elevation_degrees: self.elevation_degrees.unwrap_or(0.0),
        })
    }

    /// Convert polar coordinates to a pixel coordinate.
    pub fn polar_to_pixel(&self, azimuth_degrees: f32, range_km: f64) -> (f64, f64) {
        let az_rad = (azimuth_degrees as f64).to_radians();
        let distance_px = range_km * self.pixels_per_km;
        let x = self.center_pixel.0 + distance_px * az_rad.sin();
        let y = self.center_pixel.1 - distance_px * az_rad.cos();
        (x, y)
    }

    /// Convert a pixel coordinate to geographic coordinates.
    ///
    /// Requires a coordinate system. Returns `None` if no coordinate system is
    /// available or the pixel is outside the radar coverage area.
    pub fn pixel_to_geo(&self, x: f64, y: f64) -> Option<GeoPoint3D> {
        let polar = self.pixel_to_polar(x, y)?;
        let coord_system = self.coord_system.as_ref()?;
        Some(coord_system.polar_to_geo(polar))
    }

    /// Convert geographic coordinates to a pixel coordinate.
    ///
    /// Requires a coordinate system. Returns `None` if no coordinate system is available.
    pub fn geo_to_pixel(&self, point: &GeoPoint) -> Option<(f64, f64)> {
        let coord_system = self.coord_system.as_ref()?;
        let elevation = self.elevation_degrees.unwrap_or(0.0);
        let polar = coord_system.geo_to_polar(*point, elevation);
        Some(self.polar_to_pixel(polar.azimuth_degrees, polar.range_km))
    }

    /// Get the range in km for a given pixel distance from center.
    pub fn pixel_distance_to_km(&self, pixels: f64) -> f64 {
        pixels / self.pixels_per_km
    }

    /// Get the pixel distance from center for a given range in km.
    pub fn km_to_pixel_distance(&self, km: f64) -> f64 {
        km * self.pixels_per_km
    }
}
