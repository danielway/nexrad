use nexrad_model::data::{MomentData, MomentValue, Radial};
use result::{Result, Error};
use piet::{Color, RenderContext};
use piet_common::kurbo::{Arc, Point, Vec2};
use piet_common::{BitmapTarget, Device};
use std::cmp::max;

mod color;
pub use crate::color::*;

pub mod result;

/// Radar data products to render.
#[derive(Debug, Copy, Clone)]
pub enum Product {
    Reflectivity,
    Velocity,
    SpectrumWidth,
    DifferentialReflectivity,
    DifferentialPhase,
    CorrelationCoefficient,
    SpecificDiffPhase,
}

/// Render the specified radials to an image.
pub fn render_radials<'a>(
    device: &'a mut Device,
    radials: &Vec<Radial>,
    product: Product,
    scale: &DiscreteColorScale,
    size: (usize, usize),
) -> Result<BitmapTarget<'a>> {
    let mut target = device.bitmap_target(size.0, size.1, 1.0)?;

    let mut render_context = target.render_context();
    render_context.clear(None, Color::BLACK);

    let image_center = Point::new(size.0 as f64 / 2.0, size.1 as f64 / 2.0);

    for radial in radials {
        let azimuth = radial.azimuth_angle_degrees();
        let data_moment = get_radial_moment(product, radial).ok_or(Error::ProductNotFound)?;

        let first_gate_distance = data_moment.first_gate_range_km();

        for (gate_index, value) in data_moment.values().into_iter().enumerate() {
            if let MomentValue::Value(value) = value {
                let radar_range = first_gate_distance
                    + data_moment.gate_count() as f64 * data_moment.gate_interval_km();

                let render_scale = max(size.0, size.1) as f64 / 2.0 / radar_range;

                let gate_distance = first_gate_distance + gate_index as f64 * data_moment.gate_interval_km();

                let scaled_gate_interval = data_moment.gate_interval_km() * render_scale;

                // todo: why do we subtract half an interval instead of adding?
                let gate_midpoint = gate_distance - data_moment.gate_interval_km() / 2.0;
                let scaled_gate_midpoint = render_scale * gate_midpoint;

                // TODO: Rotate 90 degrees to align North with the top
                // let rotated_azimuth = azimuth - 90.0;

                render_context.stroke(
                    Arc::new(
                        image_center,
                        Vec2::new(scaled_gate_midpoint, scaled_gate_midpoint),
                        azimuth.into(),
                        radial.azimuth_spacing_degrees().into(),
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
