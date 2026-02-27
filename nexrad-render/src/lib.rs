//! Rendering functions for NEXRAD weather radar data.
//!
//! This crate provides functions to render radar data into visual images. It converts
//! radar moment data (reflectivity, velocity, etc.) into color-mapped images that can
//! be saved to common formats like PNG.
//!
//! # Example
//!
//! ```ignore
//! use nexrad_model::data::Product;
//! use nexrad_render::{render_radials, RenderOptions, get_nws_reflectivity_scale};
//!
//! let options = RenderOptions::new(800, 800);
//! let image = render_radials(
//!     sweep.radials(),
//!     Product::Reflectivity,
//!     &get_nws_reflectivity_scale(),
//!     &options,
//! ).unwrap();
//!
//! // Save directly to PNG
//! image.save("radar.png").unwrap();
//! ```
//!
//! # Crate Boundaries
//!
//! This crate provides **visualization and rendering** with the following responsibilities
//! and constraints:
//!
//! ## Responsibilities
//!
//! - Render radar data to images ([`image::RgbaImage`])
//! - Apply color scales to moment data
//! - Handle geometric transformations (polar to Cartesian coordinates)
//! - Consume `nexrad-model` types (Radial, MomentData)
//!
//! ## Constraints
//!
//! - **No data access or network operations**
//! - **No binary parsing or decoding**
//!
//! This crate can be used standalone or through the `nexrad` facade crate (re-exported
//! via the `render` feature, which is enabled by default).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![deny(missing_docs)]

pub use image::RgbaImage;
use nexrad_model::data::{
    CFPMomentValue, CartesianField, DataMoment, GateStatus, MomentValue, Product, Radial,
    SweepField, VerticalField,
};
use nexrad_model::geo::{GeoExtent, RadarCoordinateSystem};
use result::{Error, Result};

mod color;
pub use crate::color::*;

mod render_result;
pub use render_result::{PointQuery, RenderMetadata, RenderResult};

pub mod result;

/// Options for rendering radar radials.
///
/// Use the builder methods to configure rendering options, then pass to
/// [`render_radials`].
///
/// # Example
///
/// ```
/// use nexrad_render::RenderOptions;
///
/// // Render 800x800 with black background (default)
/// let options = RenderOptions::new(800, 800);
///
/// // Render with transparent background for compositing
/// let options = RenderOptions::new(800, 800).transparent();
///
/// // Render with custom background color (RGBA)
/// let options = RenderOptions::new(800, 800).with_background([255, 255, 255, 255]);
/// ```
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Output image dimensions (width, height) in pixels.
    pub size: (usize, usize),
    /// Background color as RGBA bytes. `None` means transparent (all zeros).
    pub background: Option<[u8; 4]>,
    /// Geographic extent to render. If `None`, auto-computed from data range.
    ///
    /// When set, the image covers exactly this extent, enabling consistent
    /// spatial mapping across multiple renders for side-by-side comparison.
    pub extent: Option<GeoExtent>,
    /// Radar coordinate system for geographic projection.
    ///
    /// When provided, the [`RenderResult`] will include geographic metadata
    /// enabling pixel-to-geo and geo-to-pixel coordinate conversions.
    pub coord_system: Option<RadarCoordinateSystem>,
}

impl RenderOptions {
    /// Creates new render options with the specified dimensions and black background.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            size: (width, height),
            background: Some([0, 0, 0, 255]),
            extent: None,
            coord_system: None,
        }
    }

    /// Sets the background to transparent for compositing.
    ///
    /// When rendering with a transparent background, areas without radar data
    /// will be fully transparent, allowing multiple renders to be layered.
    pub fn transparent(mut self) -> Self {
        self.background = None;
        self
    }

    /// Sets a custom background color as RGBA bytes.
    pub fn with_background(mut self, rgba: [u8; 4]) -> Self {
        self.background = Some(rgba);
        self
    }

    /// Sets a geographic extent for the rendered area.
    ///
    /// When set, the image covers exactly this extent. This enables
    /// consistent spatial mapping for side-by-side comparison of multiple renders.
    pub fn with_extent(mut self, extent: GeoExtent) -> Self {
        self.extent = Some(extent);
        self
    }

    /// Sets a radar coordinate system for geographic projection.
    ///
    /// This enables geographic metadata in the [`RenderResult`], including
    /// pixel-to-geo and geo-to-pixel coordinate conversions.
    pub fn with_coord_system(mut self, coord_system: RadarCoordinateSystem) -> Self {
        self.coord_system = Some(coord_system);
        self
    }

    /// Creates render options sized for native resolution of a sweep field.
    ///
    /// Sets both width and height to `gate_count * 2`, which ensures approximately
    /// one pixel per gate at the outer edge of the sweep. This produces the highest
    /// fidelity rendering without wasting pixels.
    pub fn native_for(field: &SweepField) -> Self {
        let size = field.gate_count() * 2;
        Self::new(size, size)
    }
}

/// Renders radar radials to an RGBA image.
///
/// Converts polar radar data into a Cartesian image representation. Each radial's
/// moment values are mapped to colors using the provided color scale, producing
/// a centered radar image with North at the top.
///
/// # Arguments
///
/// * `radials` - Slice of radials to render (typically from a single sweep)
/// * `product` - The radar product (moment type) to visualize
/// * `scale` - Color scale mapping moment values to colors
/// * `options` - Rendering options (size, background, etc.)
///
/// # Errors
///
/// Returns an error if:
/// - No radials are provided
/// - The requested product is not present in the radials
///
/// # Example
///
/// ```ignore
/// use nexrad_render::{render_radials, Product, RenderOptions, get_nws_reflectivity_scale};
///
/// let scale = get_nws_reflectivity_scale();
/// let options = RenderOptions::new(800, 800);
///
/// let image = render_radials(
///     sweep.radials(),
///     Product::Reflectivity,
///     &scale,
///     &options,
/// ).unwrap();
///
/// image.save("radar.png").unwrap();
/// ```
pub fn render_radials(
    radials: &[Radial],
    product: Product,
    scale: &DiscreteColorScale,
    options: &RenderOptions,
) -> Result<RgbaImage> {
    let (width, height) = options.size;
    let mut buffer = vec![0u8; width * height * 4];

    // Fill background
    if let Some(bg) = options.background {
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg);
        }
    }

    if radials.is_empty() {
        return Err(Error::NoRadials);
    }

    // Build lookup table for fast color mapping
    let (min_val, max_val) = product.value_range();
    let lut = ColorLookupTable::from_scale(scale, min_val, max_val, 256);

    // Get radar parameters from the first radial
    let first_radial = &radials[0];
    let gate_params = get_gate_params(product, first_radial).ok_or(Error::ProductNotFound)?;
    let first_gate_km = gate_params.first_gate_km;
    let gate_interval_km = gate_params.gate_interval_km;
    let gate_count = gate_params.gate_count;
    let radar_range_km = first_gate_km + gate_count as f64 * gate_interval_km;

    // Pre-extract all moment float values indexed by azimuth for efficient lookup
    let mut radial_data: Vec<(f32, Vec<Option<f32>>)> = Vec::with_capacity(radials.len());
    for radial in radials {
        let azimuth = radial.azimuth_angle_degrees();
        if let Some(values) = get_radial_float_values(product, radial) {
            radial_data.push((azimuth, values));
        }
    }

    // Sort by azimuth for binary search
    radial_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Extract azimuths for binary search
    let sorted_azimuths: Vec<f32> = radial_data.iter().map(|(az, _)| *az).collect();

    // Calculate max azimuth gap threshold based on radial spacing
    // Use 1.5x the expected spacing to allow for minor gaps while rejecting large ones
    let azimuth_spacing = first_radial.azimuth_spacing_degrees();
    let max_azimuth_gap = azimuth_spacing * 1.5;

    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let scale_factor = width.max(height) as f64 / 2.0 / radar_range_km;

    // Render each pixel by mapping to radar coordinates
    for y in 0..height {
        let dy = y as f64 - center_y;

        for x in 0..width {
            let dx = x as f64 - center_x;

            // Convert pixel position to distance in km
            let distance_pixels = (dx * dx + dy * dy).sqrt();
            let distance_km = distance_pixels / scale_factor;

            // Skip pixels outside radar coverage
            if distance_km < first_gate_km || distance_km >= radar_range_km {
                continue;
            }

            // Calculate azimuth angle (0 = North, clockwise)
            let azimuth_rad = dx.atan2(-dy);
            let azimuth_deg = (azimuth_rad.to_degrees() + 360.0) % 360.0;

            // Find the closest radial and check if it's within acceptable range
            let (radial_idx, angular_distance) =
                find_closest_radial(&sorted_azimuths, azimuth_deg as f32);

            // Skip pixels where no radial is close enough (partial sweep gaps)
            if angular_distance > max_azimuth_gap {
                continue;
            }

            // Calculate gate index
            let gate_index = ((distance_km - first_gate_km) / gate_interval_km) as usize;
            if gate_index >= gate_count {
                continue;
            }

            // Look up the value and apply color
            let (_, ref values) = radial_data[radial_idx];
            if let Some(Some(value)) = values.get(gate_index) {
                let color = lut.get_color(*value);
                let pixel_index = (y * width + x) * 4;
                buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
            }
        }
    }

    // Convert buffer to RgbaImage
    RgbaImage::from_raw(width as u32, height as u32, buffer).ok_or(Error::InvalidDimensions)
}

/// Renders radar radials using the default color scale for the product.
///
/// This is a convenience function that automatically selects an appropriate
/// color scale based on the product type, using standard meteorological conventions.
///
/// # Arguments
///
/// * `radials` - Slice of radials to render (typically from a single sweep)
/// * `product` - The radar product (moment type) to visualize
/// * `options` - Rendering options (size, background, etc.)
///
/// # Errors
///
/// Returns an error if:
/// - No radials are provided
/// - The requested product is not present in the radials
///
/// # Example
///
/// ```ignore
/// use nexrad_render::{render_radials_default, Product, RenderOptions};
///
/// let options = RenderOptions::new(800, 800);
/// let image = render_radials_default(
///     sweep.radials(),
///     Product::Velocity,
///     &options,
/// ).unwrap();
///
/// image.save("velocity.png").unwrap();
/// ```
pub fn render_radials_default(
    radials: &[Radial],
    product: Product,
    options: &RenderOptions,
) -> Result<RgbaImage> {
    let scale = get_default_scale(product);
    render_radials(radials, product, &scale, options)
}

/// Renders a [`SweepField`] (polar grid) to an image with metadata.
///
/// This is the primary rendering entry point for processed data. It accepts
/// a `SweepField` — the output of data extraction or processing — and produces
/// a [`RenderResult`] that includes both the rendered image and metadata for
/// geographic placement and data inspection.
///
/// # Arguments
///
/// * `field` - The sweep field to render
/// * `color_scale` - Color scale to apply (discrete or continuous)
/// * `options` - Rendering options (size, background, extent, coordinate system)
///
/// # Example
///
/// ```ignore
/// use nexrad_render::{render_sweep, RenderOptions, ColorScale};
/// use nexrad_model::data::{SweepField, Product};
///
/// let field = SweepField::from_radials(sweep.radials(), Product::Reflectivity).unwrap();
/// let scale = ColorScale::from(nexrad_render::get_nws_reflectivity_scale());
/// let result = render_sweep(&field, &scale, &RenderOptions::new(800, 800))?;
///
/// result.image().save("radar.png").unwrap();
/// let meta = result.metadata();
/// let ring_px = meta.km_to_pixel_distance(100.0);
/// ```
pub fn render_sweep(
    field: &SweepField,
    color_scale: &ColorScale,
    options: &RenderOptions,
) -> Result<RenderResult> {
    let (width, height) = options.size;
    let mut buffer = vec![0u8; width * height * 4];

    // Fill background
    if let Some(bg) = options.background {
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg);
        }
    }

    if field.azimuth_count() == 0 {
        return Err(Error::NoRadials);
    }

    let first_gate_km = field.first_gate_range_km();
    let gate_interval_km = field.gate_interval_km();
    let gate_count = field.gate_count();
    let radar_range_km = field.max_range_km();

    // Build LUT from the field's value range or product-typical range
    let (min_val, max_val) = field.value_range().unwrap_or((-32.0, 95.0));
    let lut = ColorLookupTable::from_color_scale(color_scale, min_val, max_val, 256);

    // Calculate max azimuth gap for partial sweep support
    let azimuth_spacing = field.azimuth_spacing_degrees();
    let max_azimuth_gap = azimuth_spacing * 1.5;

    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let scale_factor = width.max(height) as f64 / 2.0 / radar_range_km;

    // Render each pixel by mapping to radar coordinates
    for y in 0..height {
        let dy = y as f64 - center_y;

        for x in 0..width {
            let dx = x as f64 - center_x;

            let distance_pixels = (dx * dx + dy * dy).sqrt();
            let distance_km = distance_pixels / scale_factor;

            if distance_km < first_gate_km || distance_km >= radar_range_km {
                continue;
            }

            let azimuth_rad = dx.atan2(-dy);
            let azimuth_deg = ((azimuth_rad.to_degrees() + 360.0) % 360.0) as f32;

            let (radial_idx, angular_distance) = find_closest_radial(field.azimuths(), azimuth_deg);

            if angular_distance > max_azimuth_gap {
                continue;
            }

            let gate_index = ((distance_km - first_gate_km) / gate_interval_km) as usize;
            if gate_index >= gate_count {
                continue;
            }

            let (val, status) = field.get(radial_idx, gate_index);
            if status == GateStatus::Valid {
                let color = lut.get_color(val);
                let pixel_index = (y * width + x) * 4;
                buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
            }
        }
    }

    let image =
        RgbaImage::from_raw(width as u32, height as u32, buffer).ok_or(Error::InvalidDimensions)?;

    let geo_extent = options.extent.or_else(|| {
        options
            .coord_system
            .as_ref()
            .map(|cs| cs.sweep_extent(radar_range_km))
    });

    let metadata = RenderMetadata {
        width: width as u32,
        height: height as u32,
        center_pixel: (center_x, center_y),
        pixels_per_km: scale_factor,
        max_range_km: radar_range_km,
        elevation_degrees: Some(field.elevation_degrees()),
        geo_extent,
        coord_system: options.coord_system.clone(),
    };

    Ok(RenderResult::new(image, metadata))
}

/// Renders a [`CartesianField`] (geographic grid) to an image with metadata.
///
/// This renders volume-derived products like composite reflectivity, echo tops,
/// and VIL — data that is already projected onto a geographic grid.
///
/// # Arguments
///
/// * `field` - The Cartesian field to render
/// * `color_scale` - Color scale to apply
/// * `options` - Rendering options (size, background)
pub fn render_cartesian(
    field: &CartesianField,
    color_scale: &ColorScale,
    options: &RenderOptions,
) -> Result<RenderResult> {
    let (width, height) = options.size;
    let mut buffer = vec![0u8; width * height * 4];

    if let Some(bg) = options.background {
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg);
        }
    }

    let field_width = field.width();
    let field_height = field.height();

    if field_width == 0 || field_height == 0 {
        return Err(Error::NoRadials);
    }

    // Build LUT
    let (min_val, max_val) = field.value_range().unwrap_or((-32.0, 95.0));
    let lut = ColorLookupTable::from_color_scale(color_scale, min_val, max_val, 256);

    // Map output pixels to field cells
    for y in 0..height {
        let field_row = (y as f64 / height as f64 * field_height as f64) as usize;
        let field_row = field_row.min(field_height - 1);

        for x in 0..width {
            let field_col = (x as f64 / width as f64 * field_width as f64) as usize;
            let field_col = field_col.min(field_width - 1);

            let (val, status) = field.get(field_row, field_col);
            if status == GateStatus::Valid {
                let color = lut.get_color(val);
                let pixel_index = (y * width + x) * 4;
                buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
            }
        }
    }

    let image =
        RgbaImage::from_raw(width as u32, height as u32, buffer).ok_or(Error::InvalidDimensions)?;

    let metadata = RenderMetadata {
        width: width as u32,
        height: height as u32,
        center_pixel: (width as f64 / 2.0, height as f64 / 2.0),
        pixels_per_km: 0.0, // Not applicable for Cartesian fields
        max_range_km: 0.0,
        elevation_degrees: None,
        geo_extent: Some(*field.extent()),
        coord_system: options.coord_system.clone(),
    };

    Ok(RenderResult::new(image, metadata))
}

/// Renders a [`VerticalField`] (RHI / cross-section display) to an image with metadata.
///
/// This renders vertical cross-sections where the horizontal axis represents
/// distance from the radar and the vertical axis represents altitude.
///
/// # Arguments
///
/// * `field` - The vertical field to render
/// * `color_scale` - Color scale to apply
/// * `options` - Rendering options (size, background)
pub fn render_vertical(
    field: &VerticalField,
    color_scale: &ColorScale,
    options: &RenderOptions,
) -> Result<RenderResult> {
    let (width, height) = options.size;
    let mut buffer = vec![0u8; width * height * 4];

    if let Some(bg) = options.background {
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg);
        }
    }

    let field_width = field.width();
    let field_height = field.height();

    if field_width == 0 || field_height == 0 {
        return Err(Error::NoRadials);
    }

    // Build LUT
    let (min_val, max_val) = field.value_range().unwrap_or((-32.0, 95.0));
    let lut = ColorLookupTable::from_color_scale(color_scale, min_val, max_val, 256);

    // Map output pixels to field cells
    for y in 0..height {
        let field_row = (y as f64 / height as f64 * field_height as f64) as usize;
        let field_row = field_row.min(field_height - 1);

        for x in 0..width {
            let field_col = (x as f64 / width as f64 * field_width as f64) as usize;
            let field_col = field_col.min(field_width - 1);

            let (val, status) = field.get(field_row, field_col);
            if status == GateStatus::Valid {
                let color = lut.get_color(val);
                let pixel_index = (y * width + x) * 4;
                buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
            }
        }
    }

    let image =
        RgbaImage::from_raw(width as u32, height as u32, buffer).ok_or(Error::InvalidDimensions)?;

    let (d_min, d_max) = field.distance_range_km();
    let max_range = d_max - d_min;

    let metadata = RenderMetadata {
        width: width as u32,
        height: height as u32,
        center_pixel: (0.0, height as f64 / 2.0), // Left edge is range 0
        pixels_per_km: width as f64 / max_range,
        max_range_km: max_range,
        elevation_degrees: None,
        geo_extent: None,
        coord_system: options.coord_system.clone(),
    };

    Ok(RenderResult::new(image, metadata))
}

/// Returns the default color scale for a given product.
///
/// This function selects an appropriate color scale based on the product type,
/// using standard meteorological conventions.
///
/// | Product | Scale |
/// |---------|-------|
/// | Reflectivity | NWS Reflectivity (dBZ) |
/// | Velocity | Divergent Green-Red (-64 to +64 m/s) |
/// | SpectrumWidth | Sequential (0 to 30 m/s) |
/// | DifferentialReflectivity | Divergent (-2 to +6 dB) |
/// | DifferentialPhase | Sequential (0 to 360 deg) |
/// | CorrelationCoefficient | Sequential (0 to 1) |
/// | ClutterFilterPower | Divergent (-20 to +20 dB) |
pub fn get_default_scale(product: Product) -> DiscreteColorScale {
    match product {
        Product::Reflectivity => get_nws_reflectivity_scale(),
        Product::Velocity => get_velocity_scale(),
        Product::SpectrumWidth => get_spectrum_width_scale(),
        Product::DifferentialReflectivity => get_differential_reflectivity_scale(),
        Product::DifferentialPhase => get_differential_phase_scale(),
        Product::CorrelationCoefficient => get_correlation_coefficient_scale(),
        Product::ClutterFilterPower => get_clutter_filter_power_scale(),
    }
}

/// Returns the default color scale for a product, wrapped in a [`ColorScale`] enum.
///
/// This is a convenience wrapper around [`get_default_scale`] that returns the
/// [`ColorScale`] enum directly, avoiding the need to call `.into()` at each call site.
pub fn get_default_color_scale(product: Product) -> ColorScale {
    get_default_scale(product).into()
}

/// Find the index in sorted_azimuths closest to the given azimuth and return
/// the angular distance to that radial.
///
/// Returns `(index, angular_distance)` where `angular_distance` is in degrees.
#[inline]
fn find_closest_radial(sorted_azimuths: &[f32], azimuth: f32) -> (usize, f32) {
    let len = sorted_azimuths.len();
    if len == 0 {
        return (0, f32::MAX);
    }

    // Binary search for insertion point
    let pos = sorted_azimuths
        .binary_search_by(|a| a.partial_cmp(&azimuth).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or_else(|i| i);

    if pos == 0 {
        // Check if wrapping around (360° boundary) is closer
        let dist_to_first = (sorted_azimuths[0] - azimuth).abs();
        let dist_to_last = 360.0 - sorted_azimuths[len - 1] + azimuth;
        if dist_to_last < dist_to_first {
            return (len - 1, dist_to_last);
        }
        return (0, dist_to_first);
    }

    if pos >= len {
        // Check if wrapping around is closer
        let dist_to_last = (azimuth - sorted_azimuths[len - 1]).abs();
        let dist_to_first = 360.0 - azimuth + sorted_azimuths[0];
        if dist_to_first < dist_to_last {
            return (0, dist_to_first);
        }
        return (len - 1, dist_to_last);
    }

    // Compare distances to neighbors
    let dist_to_prev = (azimuth - sorted_azimuths[pos - 1]).abs();
    let dist_to_curr = (sorted_azimuths[pos] - azimuth).abs();

    if dist_to_prev <= dist_to_curr {
        (pos - 1, dist_to_prev)
    } else {
        (pos, dist_to_curr)
    }
}

/// Gate metadata extracted from a moment data block.
struct GateParams {
    first_gate_km: f64,
    gate_interval_km: f64,
    gate_count: usize,
}

/// Retrieve gate metadata from a radial for the given product.
fn get_gate_params(product: Product, radial: &Radial) -> Option<GateParams> {
    fn extract(m: &impl DataMoment) -> GateParams {
        GateParams {
            first_gate_km: m.first_gate_range_km(),
            gate_interval_km: m.gate_interval_km(),
            gate_count: m.gate_count() as usize,
        }
    }
    if let Some(moment) = product.moment_data(radial) {
        return Some(extract(moment));
    }
    if let Some(cfp) = product.cfp_moment_data(radial) {
        return Some(extract(cfp));
    }
    None
}

/// Extract decoded float values from a radial for the given product.
///
/// Returns `None` for below-threshold, range-folded, and CFP status gates.
fn get_radial_float_values(product: Product, radial: &Radial) -> Option<Vec<Option<f32>>> {
    if let Some(moment) = product.moment_data(radial) {
        return Some(
            moment
                .iter()
                .map(|v| match v {
                    MomentValue::Value(f) => Some(f),
                    _ => None,
                })
                .collect(),
        );
    }

    if let Some(cfp) = product.cfp_moment_data(radial) {
        return Some(
            cfp.iter()
                .map(|v| match v {
                    CFPMomentValue::Value(f) => Some(f),
                    CFPMomentValue::Status(_) => None,
                })
                .collect(),
        );
    }

    None
}
