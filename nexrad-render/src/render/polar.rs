//! Polar sweep rendering.

use super::image::RenderedImage;
use super::options::RenderOpts;
use crate::color::DiscreteColorScale;
use crate::result::Result;
use nexrad_model::field::PolarSweep;
use piet::{Color, RenderContext};
use piet_common::kurbo::{Arc, Point, Vec2};
use piet_common::Device;
use std::cmp::max;
use std::f32::consts::PI;

/// Renders a polar sweep to an image.
///
/// This is the primary rendering function for polar radar data using the
/// canonical [`PolarSweep`] type. Each ray is rendered as an arc segment,
/// with colors determined by the values and the provided color scale.
///
/// # Arguments
///
/// * `device` - The piet rendering device
/// * `sweep` - The polar sweep data to render
/// * `scale` - Color scale mapping values to colors
/// * `opts` - Rendering options (size, background, etc.)
///
/// # Errors
///
/// Returns an error if rendering fails.
///
/// # Example
///
/// ```ignore
/// use nexrad_model::field::PolarSweep;
/// use nexrad_render::render::{render_polar, RenderOpts};
/// use nexrad_render::get_nws_reflectivity_scale;
/// use piet_common::Device;
///
/// let mut device = Device::new().unwrap();
/// let opts = RenderOpts::new(800, 800);
/// let mut image = render_polar(&mut device, &sweep, &get_nws_reflectivity_scale(), &opts)?;
/// image.save_to_file("radar.png")?;
/// ```
pub fn render_polar<'a>(
    device: &'a mut Device,
    sweep: &PolarSweep<f32>,
    scale: &DiscreteColorScale,
    opts: &RenderOpts,
) -> Result<RenderedImage<'a>> {
    let (width, height) = opts.size();
    let mut target = device.bitmap_target(width, height, 1.0)?;

    let mut ctx = target.render_context();

    // Clear background
    match opts.background {
        Some(color) => ctx.clear(None, color),
        None => ctx.clear(None, Color::TRANSPARENT),
    }

    let image_center = Point::new(width as f64 / 2.0, height as f64 / 2.0);

    // Determine render scale
    let max_range = opts.max_range_m.unwrap_or_else(|| sweep.max_range_m());
    let render_scale = max(width, height) as f64 / 2.0 / max_range as f64;

    let gate_size = sweep.gate_size_m();
    let scaled_gate_size = gate_size as f64 * render_scale;
    let first_gate = sweep.first_gate_range_m();

    let azimuths = sweep.azimuth_deg();
    let ray_count = sweep.ray_count();

    // Iterate over rays and gates
    for ray in 0..ray_count {
        let azimuth_deg = azimuths[ray];
        let azimuth_rad = azimuth_deg.to_radians();

        // Rotate to align North with top of image
        // In piet, 0 radians is to the right (East), and angles increase counter-clockwise
        // Meteorological azimuth is 0=North, increasing clockwise
        // So we need: piet_angle = -(azimuth - 90°) = 90° - azimuth
        // Or equivalently: azimuth_rad - PI/2 for the starting angle
        let rotated_azimuth = (azimuth_rad - PI / 2.0) as f64;

        // Calculate azimuth spacing
        let azimuth_spacing = calculate_azimuth_spacing(azimuths, ray);
        let sweep_angle = azimuth_spacing.to_radians() as f64;

        for gate in 0..sweep.gate_count() {
            let value = *sweep.get(ray, gate);

            // Skip NaN values or render with nodata color
            if value.is_nan() {
                if let Some(nodata_color) = opts.nodata_color {
                    let gate_range = first_gate + gate as f32 * gate_size;
                    let gate_midpoint = gate_range as f64 * render_scale;

                    ctx.stroke(
                        Arc::new(
                            image_center,
                            Vec2::new(gate_midpoint, gate_midpoint),
                            rotated_azimuth,
                            sweep_angle,
                            0.0,
                        ),
                        &nodata_color,
                        scaled_gate_size,
                    );
                }
                continue;
            }

            let color = scale.get_color(value);
            let gate_range = first_gate + gate as f32 * gate_size;
            let gate_midpoint = gate_range as f64 * render_scale;

            ctx.stroke(
                Arc::new(
                    image_center,
                    Vec2::new(gate_midpoint, gate_midpoint),
                    rotated_azimuth,
                    sweep_angle,
                    0.0,
                ),
                &color,
                scaled_gate_size,
            );
        }
    }

    ctx.finish()?;
    drop(ctx);

    Ok(RenderedImage::new(target, width, height))
}

/// Calculate the azimuth spacing for a ray.
fn calculate_azimuth_spacing(azimuths: &[f32], ray: usize) -> f32 {
    if azimuths.len() <= 1 {
        return 1.0;
    }

    if ray < azimuths.len() - 1 {
        // Use forward difference
        let diff = azimuths[ray + 1] - azimuths[ray];
        // Handle wrap-around
        if diff < 0.0 {
            diff + 360.0
        } else {
            diff
        }
    } else {
        // Last ray: use backward difference
        let diff = azimuths[ray] - azimuths[ray - 1];
        if diff < 0.0 {
            diff + 360.0
        } else {
            diff
        }
    }
}
