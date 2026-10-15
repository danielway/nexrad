//! Rendering of gridded (Cartesian and vertical cross-section) data.

use crate::options::Palette;
use crate::{
    ColorScale, Error, Interpolation, Projection, RenderMetadata, RenderOptions, RenderResult,
    Result, RgbaImage,
};
use nexrad_model::data::{CartesianField, GateStatus, VerticalField};

/// Renders a [`CartesianField`] (geographic grid) to an image with metadata.
///
/// This renders volume-derived products like composite reflectivity, echo tops,
/// and VIL — data that is already projected onto a geographic grid.
///
/// # Arguments
///
/// * `field` - The Cartesian field to render
/// * `color_scale` - Color scale to apply
/// * `options` - Rendering options (size, background, interpolation, value range, status
///   colors); the extent option is ignored because the field carries its own extent
///
/// # Errors
///
/// Returns [`Error::InvalidDimensions`] if either output dimension is zero and
/// [`Error::EmptyField`] if the field has no rows or columns.
///
/// # Example
///
/// ```ignore
/// use nexrad_render::{render_cartesian, default_scale, RenderOptions};
/// use nexrad_model::data::Product;
/// use nexrad_process::{CartesianGrid, ScanDerivedProduct, derived::CompositeReflectivity};
///
/// // Compute composite reflectivity from all elevation tilts
/// let grid = CartesianGrid::new(coord_sys.sweep_extent(230.0), 800, 800)?;
/// let cartesian = CompositeReflectivity.compute(&ref_fields, &coord_sys, &grid)?;
///
/// // Render the cartesian field
/// let scale = default_scale(Product::Reflectivity);
/// let result = render_cartesian(&cartesian, &scale, &RenderOptions::new(800, 800))?;
/// result.save("composite_reflectivity.png")?;
/// ```
pub fn render_cartesian(
    field: &CartesianField,
    color_scale: &ColorScale,
    options: &RenderOptions,
) -> Result<RenderResult> {
    let palette = options.palette(color_scale, None, field.value_range())?;
    let image = render_grid(
        field.width(),
        field.height(),
        |row, col| field.get(row, col),
        &palette,
        options,
    )?;

    let extent = *field.extent();
    let metadata = RenderMetadata::new(
        image.width(),
        image.height(),
        Projection::Geographic { extent },
        Some(extent),
        options.coord_system.clone(),
    );

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
/// * `options` - Rendering options (size, background, interpolation, value range, status
///   colors); the extent option is ignored because the field carries its own extent
///
/// # Errors
///
/// Returns [`Error::InvalidDimensions`] if either output dimension is zero and
/// [`Error::EmptyField`] if the field has no rows or columns.
///
/// # Example
///
/// ```ignore
/// use nexrad_render::{render_vertical, default_scale, RenderOptions};
/// use nexrad_model::data::Product;
/// use nexrad_process::derived::VerticalCrossSection;
///
/// // Create a vertical cross-section at 200° azimuth
/// let vcs = VerticalCrossSection::builder(200.0)
///     .max_range_km(230.0)
///     .max_altitude_meters(18000.0)
///     .build()?;
/// let vertical = vcs.compute(&ref_fields, &coord_sys)?;
///
/// // Render the vertical field
/// let scale = default_scale(Product::Reflectivity);
/// let result = render_vertical(&vertical, &scale, &RenderOptions::new(1200, 600))?;
/// result.save("vertical_cross_section.png")?;
/// ```
pub fn render_vertical(
    field: &VerticalField,
    color_scale: &ColorScale,
    options: &RenderOptions,
) -> Result<RenderResult> {
    let palette = options.palette(color_scale, None, field.value_range())?;
    let image = render_grid(
        field.width(),
        field.height(),
        |row, col| field.get(row, col),
        &palette,
        options,
    )?;

    let projection = Projection::Vertical {
        distance_range_km: field.distance_range_km(),
        altitude_range_meters: field.altitude_range_meters(),
    };
    let metadata = RenderMetadata::new(
        image.width(),
        image.height(),
        projection,
        None,
        options.coord_system.clone(),
    );

    Ok(RenderResult::new(image, metadata))
}

/// Rasterise a row-major grid of `field_width` x `field_height` cells, stretching it
/// to fill the output image.
fn render_grid(
    field_width: usize,
    field_height: usize,
    get: impl Fn(usize, usize) -> Option<(f32, GateStatus)>,
    palette: &Palette,
    options: &RenderOptions,
) -> Result<RgbaImage> {
    let (image_width, image_height) = options.size;
    let mut buffer = options.background_buffer()?;
    let (width, height) = (image_width as usize, image_height as usize);

    if field_width == 0 || field_height == 0 {
        return Err(Error::EmptyField);
    }

    // Map output pixels to field cells
    let use_bilinear = options.interpolation == Interpolation::Bilinear;
    let max_row = field_height - 1;
    let max_col = field_width - 1;

    for y in 0..height {
        let row_f = y as f64 / height as f64 * field_height as f64 - 0.5;
        let row_f = row_f.max(0.0);

        for x in 0..width {
            let col_f = x as f64 / width as f64 * field_width as f64 - 0.5;
            let col_f = col_f.max(0.0);

            // Try bilinear interpolation first, then fall back to nearest-neighbor
            if use_bilinear {
                if let Some(val) = sample_grid_bilinear(row_f, col_f, max_row, max_col, &get) {
                    let color = palette.value_color(val);
                    let pixel_index = (y * width + x) * 4;
                    buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
                    continue;
                }
            }

            // Nearest-neighbor path (default, or bilinear fallback)
            let field_row = (row_f + 0.5) as usize;
            let field_row = field_row.min(max_row);
            let field_col = (col_f + 0.5) as usize;
            let field_col = field_col.min(max_col);

            let Some((val, status)) = get(field_row, field_col) else {
                continue;
            };
            if let Some(color) = palette.gate_color(val, status) {
                let pixel_index = (y * width + x) * 4;
                buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
            }
        }
    }

    RgbaImage::from_raw(image_width, image_height, buffer).ok_or(Error::InvalidDimensions)
}

/// Sample a row-major grid using bilinear interpolation in data-value space.
///
/// `row_f` and `col_f` are fractional coordinates into the grid.
/// Returns the interpolated value, or `None` if any of the four neighbors is non-Valid.
#[inline]
fn sample_grid_bilinear(
    row_f: f64,
    col_f: f64,
    max_row: usize,
    max_col: usize,
    get_fn: impl Fn(usize, usize) -> Option<(f32, GateStatus)>,
) -> Option<f32> {
    let r0 = row_f as usize;
    let c0 = col_f as usize;
    let r1 = (r0 + 1).min(max_row);
    let c1 = (c0 + 1).min(max_col);

    let rf = (row_f - r0 as f64) as f32;
    let cf = (col_f - c0 as f64) as f32;

    let (v00, s00) = get_fn(r0, c0)?;
    let (v01, s01) = get_fn(r0, c1)?;
    let (v10, s10) = get_fn(r1, c0)?;
    let (v11, s11) = get_fn(r1, c1)?;

    if s00 != GateStatus::Valid
        || s01 != GateStatus::Valid
        || s10 != GateStatus::Valid
        || s11 != GateStatus::Valid
    {
        return None;
    }

    let v_top = v00 + (v01 - v00) * cf;
    let v_bot = v10 + (v11 - v10) * cf;
    Some(v_top + (v_bot - v_top) * rf)
}
