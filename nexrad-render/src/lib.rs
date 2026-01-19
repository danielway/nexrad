//! Rendering functions for NEXRAD weather radar data.
//!
//! This crate provides functions to render radar data into visual images. It converts
//! radar moment data (reflectivity, velocity, etc.) into color-mapped visualizations.
//!
//! # New API (Recommended)
//!
//! The new API uses canonical interchange types for better composability:
//!
//! ```ignore
//! use nexrad_model::field::{PolarSweep, radials_to_polar_sweep, ProductSelector};
//! use nexrad_render::render::{render_polar, RenderOpts};
//! use nexrad_render::get_nws_reflectivity_scale;
//! use piet_common::Device;
//!
//! // Convert radials to PolarSweep
//! let sweep = radials_to_polar_sweep(radials, ProductSelector::Reflectivity)?;
//!
//! // Render to image
//! let mut device = Device::new().unwrap();
//! let opts = RenderOpts::new(800, 800);
//! let mut image = render_polar(&mut device, &sweep, &get_nws_reflectivity_scale(), &opts)?;
//! image.save_to_file("output.png")?;
//! ```
//!
//! # Legacy API
//!
//! The original API that works directly with `Radial` slices is still available:
//!
//! ```ignore
//! use nexrad_render::{render_radials, Product, get_nws_reflectivity_scale};
//! use piet_common::Device;
//!
//! let mut device = Device::new().unwrap();
//! let target = render_radials(
//!     &mut device,
//!     scan.sweeps()[0].radials(),
//!     Product::Reflectivity,
//!     &get_nws_reflectivity_scale(),
//!     (800, 800),
//! ).unwrap();
//! ```
//!
//! # Crate Boundaries
//!
//! This crate provides **visualization and rendering** with the following responsibilities
//! and constraints:
//!
//! ## Responsibilities
//!
//! - ✓ Render radar data to in-memory images
//! - ✓ Apply color scales to moment data
//! - ✓ Handle geometric transformations (polar coordinates, rotation)
//! - ✓ Consume `nexrad-model` types (Radial, MomentData, PolarSweep, CartesianGrid)
//!
//! ## Constraints
//!
//! - ✗ **No I/O operations** (returns in-memory bitmap targets)
//! - ✗ **No data access or network operations**
//! - ✗ **No binary parsing or decoding**
//!
//! This crate can be used standalone or through the `nexrad` facade crate (re-exported
//! via the `render` feature, which is enabled by default).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]
#![deny(missing_docs)]

use nexrad_model::data::{MomentData, MomentValue, Radial};
use piet::{Color, RenderContext};
use piet_common::kurbo::{Arc, Point, Vec2};
use piet_common::{BitmapTarget, Device};
use result::{Error, Result};
use std::cmp::max;
use std::f32::consts::PI;

mod color;
pub use crate::color::*;

pub mod result;

// New modules for canonical interchange types
pub mod render;
pub mod sampler;

// Re-export new primary API at crate root for convenience
pub use render::{render_grid, render_polar, RenderOpts, RenderedImage};
pub use sampler::{GridSampler, PolarSampler, Sampler};

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
    /// Specific differential phase (degrees/km). Rate of differential phase change.
    SpecificDiffPhase,
}

/// Options for rendering radar radials.
///
/// Use the builder methods to configure rendering options, then pass to
/// [`render_radials_with_options`].
///
/// # Example
///
/// ```
/// use nexrad_render::RenderOptions;
/// use piet::Color;
///
/// // Render with transparent background for compositing
/// let options = RenderOptions::new(800, 800).transparent();
///
/// // Render with custom background color
/// let options = RenderOptions::new(800, 800).with_background(Color::WHITE);
/// ```
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Background color. `None` means transparent.
    pub background: Option<Color>,
    /// Output image dimensions (width, height) in pixels.
    pub size: (usize, usize),
}

impl RenderOptions {
    /// Creates new render options with the specified dimensions and black background.
    ///
    /// This is the default behavior matching the original `render_radials` function.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            background: Some(Color::BLACK),
            size: (width, height),
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

    /// Sets a custom background color.
    pub fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }
}

/// Renders radar radials to an image with full configuration options.
///
/// Each radial is rendered as an arc segment, with colors determined by the moment
/// values and the provided color scale. The image is centered with North at the top.
///
/// # Arguments
///
/// * `device` - The piet rendering device
/// * `radials` - Slice of radials to render (typically from a single sweep)
/// * `product` - The radar product (moment type) to visualize
/// * `scale` - Color scale mapping moment values to colors
/// * `options` - Rendering options (size, background, etc.)
///
/// # Errors
///
/// Returns an error if the requested product is not present in the radials,
/// or if rendering fails.
///
/// # Example
///
/// ```ignore
/// use nexrad_render::{render_radials_with_options, Product, get_nws_reflectivity_scale, RenderOptions};
/// use piet_common::Device;
///
/// let mut device = Device::new().unwrap();
/// let options = RenderOptions::new(800, 800).transparent();
/// let target = render_radials_with_options(
///     &mut device,
///     sweep.radials(),
///     Product::Reflectivity,
///     &get_nws_reflectivity_scale(),
///     &options,
/// ).unwrap();
/// ```
pub fn render_radials_with_options<'a>(
    device: &'a mut Device,
    radials: &[Radial],
    product: Product,
    scale: &DiscreteColorScale,
    options: &RenderOptions,
) -> Result<BitmapTarget<'a>> {
    let size = options.size;
    let mut target = device.bitmap_target(size.0, size.1, 1.0)?;

    let mut render_context = target.render_context();

    // Clear with background color or transparent
    match options.background {
        Some(color) => render_context.clear(None, color),
        None => render_context.clear(None, Color::TRANSPARENT),
    }

    let image_center = Point::new(size.0 as f64 / 2.0, size.1 as f64 / 2.0);

    for radial in radials {
        let azimuth = radial.azimuth_angle_degrees().to_radians();

        // Rotate 90 degrees to align North with the top
        let rotated_azimuth = azimuth - PI / 2.0;

        let data_moment = get_radial_moment(product, radial).ok_or(Error::ProductNotFound)?;
        let first_gate_distance = data_moment.first_gate_range_km();

        let radar_range =
            first_gate_distance + data_moment.gate_count() as f64 * data_moment.gate_interval_km();

        let render_scale = max(size.0, size.1) as f64 / 2.0 / radar_range;

        let scaled_gate_interval = data_moment.gate_interval_km() * render_scale;

        for (gate_index, value) in data_moment.values().into_iter().enumerate() {
            if let MomentValue::Value(value) = value {
                let gate_distance =
                    first_gate_distance + gate_index as f64 * data_moment.gate_interval_km();
                let gate_midpoint = gate_distance - data_moment.gate_interval_km() / 2.0;
                let scaled_gate_midpoint = render_scale * gate_midpoint;

                render_context.stroke(
                    Arc::new(
                        image_center,
                        Vec2::new(scaled_gate_midpoint, scaled_gate_midpoint),
                        rotated_azimuth.into(),
                        radial.azimuth_spacing_degrees().to_radians().into(),
                        0.0,
                    ),
                    &scale.get_color(value),
                    scaled_gate_interval,
                );
            }
        }
    }

    render_context.finish()?;
    drop(render_context);

    Ok(target)
}

/// Renders radar radials to an image.
///
/// Each radial is rendered as an arc segment, with colors determined by the moment
/// values and the provided color scale. The image is centered with North at the top.
///
/// This is a convenience wrapper around [`render_radials_with_options`] that uses
/// a black background.
///
/// # Arguments
///
/// * `device` - The piet rendering device
/// * `radials` - Slice of radials to render (typically from a single sweep)
/// * `product` - The radar product (moment type) to visualize
/// * `scale` - Color scale mapping moment values to colors
/// * `size` - Output image dimensions (width, height) in pixels
///
/// # Errors
///
/// Returns an error if the requested product is not present in the radials,
/// or if rendering fails.
pub fn render_radials<'a>(
    device: &'a mut Device,
    radials: &[Radial],
    product: Product,
    scale: &DiscreteColorScale,
    size: (usize, usize),
) -> Result<BitmapTarget<'a>> {
    let options = RenderOptions::new(size.0, size.1);
    render_radials_with_options(device, radials, product, scale, &options)
}

/// Renders radar radials using the default color scale for the product.
///
/// This is the simplest way to render radar data. It automatically selects
/// an appropriate color scale based on the product type.
///
/// # Arguments
///
/// * `device` - The piet rendering device
/// * `radials` - Slice of radials to render (typically from a single sweep)
/// * `product` - The radar product (moment type) to visualize
/// * `size` - Output image dimensions (width, height) in pixels
///
/// # Errors
///
/// Returns an error if the requested product is not present in the radials,
/// or if rendering fails.
///
/// # Example
///
/// ```ignore
/// use nexrad_render::{render_radials_default, Product};
/// use piet_common::Device;
///
/// let mut device = Device::new().unwrap();
/// let target = render_radials_default(
///     &mut device,
///     sweep.radials(),
///     Product::Velocity,
///     (800, 800),
/// ).unwrap();
/// ```
pub fn render_radials_default<'a>(
    device: &'a mut Device,
    radials: &[Radial],
    product: Product,
    size: (usize, usize),
) -> Result<BitmapTarget<'a>> {
    let scale = get_default_scale(product);
    let options = RenderOptions::new(size.0, size.1);
    render_radials_with_options(device, radials, product, &scale, &options)
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
/// | SpecificDiffPhase | Sequential (0 to 10 deg/km) |
pub fn get_default_scale(product: Product) -> DiscreteColorScale {
    match product {
        Product::Reflectivity => get_nws_reflectivity_scale(),
        Product::Velocity => get_velocity_scale(),
        Product::SpectrumWidth => get_spectrum_width_scale(),
        Product::DifferentialReflectivity => get_differential_reflectivity_scale(),
        Product::DifferentialPhase => get_differential_phase_scale(),
        Product::CorrelationCoefficient => get_correlation_coefficient_scale(),
        Product::SpecificDiffPhase => get_specific_diff_phase_scale(),
    }
}

/// Retrieve the generic data block in this radial for the given product.
fn get_radial_moment(product: Product, radial: &Radial) -> Option<&MomentData> {
    match product {
        Product::Reflectivity => radial.reflectivity(),
        Product::Velocity => radial.velocity(),
        Product::SpectrumWidth => radial.spectrum_width(),
        Product::DifferentialReflectivity => radial.differential_reflectivity(),
        Product::DifferentialPhase => radial.differential_phase(),
        Product::CorrelationCoefficient => radial.correlation_coefficient(),
        Product::SpecificDiffPhase => radial.specific_differential_phase(),
    }
}
