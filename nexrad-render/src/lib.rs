//! Rendering functions for NEXRAD weather radar data.
//!
//! This crate provides functions to render radar data into visual images. It converts
//! radar moment data (reflectivity, velocity, etc.) into color-mapped visualizations.
//!
//! # Example
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

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(clippy::correctness)]

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

/// Renders radar radials to an image.
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
    let mut target = device.bitmap_target(size.0, size.1, 1.0)?;

    let mut render_context = target.render_context();
    render_context.clear(None, Color::BLACK);

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
