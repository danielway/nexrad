//! Cartesian grid rendering.

use super::image::RenderedImage;
use super::options::RenderOpts;
use crate::color::DiscreteColorScale;
use crate::result::Result;
use nexrad_model::field::CartesianGrid;
use piet::{Color, RenderContext};
use piet_common::kurbo::Rect;
use piet_common::Device;

/// Renders a Cartesian grid to an image.
///
/// This function renders gridded radar data directly to an image,
/// mapping grid pixels to output pixels with scaling as needed.
///
/// # Arguments
///
/// * `device` - The piet rendering device
/// * `grid` - The Cartesian grid data to render
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
/// use nexrad_model::field::CartesianGrid;
/// use nexrad_render::render::{render_grid, RenderOpts};
/// use nexrad_render::get_nws_reflectivity_scale;
/// use piet_common::Device;
///
/// let mut device = Device::new().unwrap();
/// let opts = RenderOpts::new(800, 800);
/// let mut image = render_grid(&mut device, &grid, &get_nws_reflectivity_scale(), &opts)?;
/// image.save_to_file("grid.png")?;
/// ```
pub fn render_grid<'a>(
    device: &'a mut Device,
    grid: &CartesianGrid<f32>,
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

    // Calculate scaling from grid coordinates to output pixels
    let grid_width_m = grid.width() as f32 * grid.pixel_size_m();
    let grid_height_m = grid.height() as f32 * grid.pixel_size_m();

    let scale_x = width as f64 / grid_width_m as f64;
    let scale_y = height as f64 / grid_height_m as f64;

    let pixel_width = scale_x * grid.pixel_size_m() as f64;
    let pixel_height = scale_y * grid.pixel_size_m() as f64;

    // Render each grid cell
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let value = *grid.get(x, y);

            // Skip NaN values or render with nodata color
            if value.is_nan() {
                if let Some(nodata_color) = opts.nodata_color {
                    let rect = Rect::new(
                        x as f64 * pixel_width,
                        y as f64 * pixel_height,
                        (x + 1) as f64 * pixel_width,
                        (y + 1) as f64 * pixel_height,
                    );
                    ctx.fill(rect, &nodata_color);
                }
                continue;
            }

            let color = scale.get_color(value);
            let rect = Rect::new(
                x as f64 * pixel_width,
                y as f64 * pixel_height,
                (x + 1) as f64 * pixel_width,
                (y + 1) as f64 * pixel_height,
            );
            ctx.fill(rect, &color);
        }
    }

    ctx.finish()?;
    drop(ctx);

    Ok(RenderedImage::new(target, width, height))
}
