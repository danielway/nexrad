//! Rendering functions for NEXRAD weather radar data.
//!
//! This crate provides functions to render radar data into visual images. It converts
//! radar moment data (reflectivity, velocity, etc.) into color-mapped images that can
//! be saved to common formats like PNG.
//!
//! # Example
//!
//! ```ignore
//! use nexrad_render::{render_radials, Product, RenderOptions, get_nws_reflectivity_scale};
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
use nexrad_model::data::{CFPMomentValue, MomentDataBlock, MomentValue, Radial};
use result::{Error, Result};
use std::ops::Deref;

mod color;
pub use crate::color::*;

pub mod result;

/// Radar data products that can be rendered.
///
/// Each product corresponds to a different type of moment data captured by the radar.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Product {
    /// Base reflectivity (dBZ). Measures the intensity of precipitation.
    Reflectivity,
    /// Radial velocity (m/s). Measures motion toward or away from the radar.
    Velocity,
    /// Spectrum width (m/s). Measures turbulence within the radar beam.
    SpectrumWidth,
    /// Differential reflectivity (dB). Compares horizontal and vertical reflectivity.
    DifferentialReflectivity,
    /// Differential phase (degrees). Phase difference between polarizations.
    DifferentialPhase,
    /// Correlation coefficient. Correlation between polarizations (0-1).
    CorrelationCoefficient,
    /// Clutter filter power (CFP). Difference between clutter-filtered and unfiltered reflectivity.
    ClutterFilterPower,
}

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
}

impl RenderOptions {
    /// Creates new render options with the specified dimensions and black background.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            size: (width, height),
            background: Some([0, 0, 0, 255]),
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
    let (min_val, max_val) = get_product_value_range(product);
    let lut = ColorLookupTable::from_scale(scale, min_val, max_val, 256);

    // Get radar parameters from the first radial
    let first_radial = &radials[0];
    let data_block =
        get_radial_moment_block(product, first_radial).ok_or(Error::ProductNotFound)?;
    let first_gate_km = data_block.first_gate_range_km();
    let gate_interval_km = data_block.gate_interval_km();
    let gate_count = data_block.gate_count() as usize;
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

/// Returns the value range (min, max) for a given product.
///
/// These ranges cover the typical data values for each product type and are
/// used internally for color mapping.
pub fn get_product_value_range(product: Product) -> (f32, f32) {
    match product {
        Product::Reflectivity => (-32.0, 95.0),
        Product::Velocity => (-64.0, 64.0),
        Product::SpectrumWidth => (0.0, 30.0),
        Product::DifferentialReflectivity => (-2.0, 6.0),
        Product::DifferentialPhase => (0.0, 360.0),
        Product::CorrelationCoefficient => (0.0, 1.0),
        Product::ClutterFilterPower => (-20.0, 20.0),
    }
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

/// Retrieve the moment data block (gate metadata) from a radial for the given product.
fn get_radial_moment_block<'a>(product: Product, radial: &'a Radial) -> Option<&'a MomentDataBlock> {
    match product {
        Product::Reflectivity => radial.reflectivity().map(|m| m.deref()),
        Product::Velocity => radial.velocity().map(|m| m.deref()),
        Product::SpectrumWidth => radial.spectrum_width().map(|m| m.deref()),
        Product::DifferentialReflectivity => radial.differential_reflectivity().map(|m| m.deref()),
        Product::DifferentialPhase => radial.differential_phase().map(|m| m.deref()),
        Product::CorrelationCoefficient => radial.correlation_coefficient().map(|m| m.deref()),
        Product::ClutterFilterPower => radial.clutter_filter_power().map(|m| m.deref()),
    }
}

/// Extract decoded float values from a radial for the given product.
///
/// Returns `None` for below-threshold, range-folded, and CFP status gates.
fn get_radial_float_values(product: Product, radial: &Radial) -> Option<Vec<Option<f32>>> {
    fn moment_floats(moment: Option<&nexrad_model::data::MomentData>) -> Option<Vec<Option<f32>>> {
        moment.map(|m| {
            m.values()
                .into_iter()
                .map(|v| match v {
                    MomentValue::Value(f) => Some(f),
                    _ => None,
                })
                .collect()
        })
    }

    match product {
        Product::Reflectivity => moment_floats(radial.reflectivity()),
        Product::Velocity => moment_floats(radial.velocity()),
        Product::SpectrumWidth => moment_floats(radial.spectrum_width()),
        Product::DifferentialReflectivity => moment_floats(radial.differential_reflectivity()),
        Product::DifferentialPhase => moment_floats(radial.differential_phase()),
        Product::CorrelationCoefficient => moment_floats(radial.correlation_coefficient()),
        Product::ClutterFilterPower => radial.clutter_filter_power().map(|cfp| {
            cfp.values()
                .into_iter()
                .map(|v| match v {
                    CFPMomentValue::Value(f) => Some(f),
                    CFPMomentValue::Status(_) => None,
                })
                .collect()
        }),
    }
}
