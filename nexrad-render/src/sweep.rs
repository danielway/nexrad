//! Rendering of polar sweep data.

use crate::{
    default_scale, ColorLookupTable, ColorScale, DiscreteColorScale, Error, Interpolation,
    RenderMetadata, RenderOptions, RenderResult, Result, RgbaImage,
};
use nexrad_model::data::{GateStatus, MomentValue, Product, Radial, SweepField};

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
/// use nexrad_render::{render_radials, Product, RenderOptions, nws_reflectivity_scale};
///
/// let scale = nws_reflectivity_scale();
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
        for pixel in buffer.as_chunks_mut::<4>().0 {
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
                let color = lut.color(*value);
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
    let scale = default_scale(product);
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
/// let scale = ColorScale::from(nexrad_render::nws_reflectivity_scale());
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
        for pixel in buffer.as_chunks_mut::<4>().0 {
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
    let use_bilinear = options.interpolation == Interpolation::Bilinear;

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

            // Try bilinear interpolation first, then fall back to nearest-neighbor
            if use_bilinear {
                if let Some(val) =
                    sample_sweep_bilinear(field, azimuth_deg, distance_km, max_azimuth_gap)
                {
                    let color = lut.color(val);
                    let pixel_index = (y * width + x) * 4;
                    buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
                    continue;
                }
            }

            // Nearest-neighbor path (default, or bilinear fallback)
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
                let color = lut.color(val);
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
        elevation_degrees: Some(field.elevation_degrees() as f64),
        geo_extent,
        coord_system: options.coord_system.clone(),
    };

    Ok(RenderResult::new(image, metadata))
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

/// Find the two radials bracketing the given azimuth and the fractional position between them.
///
/// Returns `Some((lo_idx, hi_idx, frac))` where:
/// - `lo_idx` is the index of the radial at or just below `azimuth`
/// - `hi_idx` is the index of the radial just above `azimuth`
/// - `frac` is in \[0, 1\] — the fractional distance from lo toward hi
///
/// Returns `None` if there are fewer than 2 radials, or if the angular gap between
/// the two bracketing radials exceeds `max_gap` degrees (partial sweep boundary).
#[inline]
fn find_bracketing_radials(
    sorted_azimuths: &[f32],
    azimuth: f32,
    max_gap: f32,
) -> Option<(usize, usize, f32)> {
    let len = sorted_azimuths.len();
    if len < 2 {
        return None;
    }

    let pos = sorted_azimuths
        .binary_search_by(|a| a.partial_cmp(&azimuth).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or_else(|i| i);

    // Determine the bracketing pair (lo, hi) such that lo_az <= azimuth <= hi_az
    // with 360-degree wraparound at the edges.
    let (lo_idx, hi_idx) = if pos == 0 || pos >= len {
        // Wraps around: last azimuth → first azimuth
        (len - 1, 0)
    } else {
        (pos - 1, pos)
    };

    let lo_az = sorted_azimuths[lo_idx];
    let hi_az = sorted_azimuths[hi_idx];

    // Angular span with wraparound
    let span = if hi_az >= lo_az {
        hi_az - lo_az
    } else {
        360.0 - lo_az + hi_az
    };

    if span > max_gap || span == 0.0 {
        return None;
    }

    // Fractional position of azimuth between lo and hi
    let offset = if azimuth >= lo_az {
        azimuth - lo_az
    } else {
        360.0 - lo_az + azimuth
    };

    let frac = (offset / span).clamp(0.0, 1.0);
    Some((lo_idx, hi_idx, frac))
}

/// Sample a sweep field using bilinear interpolation in data-value space.
///
/// Returns the interpolated f32 value, or `None` if any of the four neighbors
/// is non-Valid, the pixel falls in a sweep gap, or the coordinates are at the
/// outer edge where interpolation is not possible.
#[inline]
fn sample_sweep_bilinear(
    field: &SweepField,
    azimuth_deg: f32,
    distance_km: f64,
    max_azimuth_gap: f32,
) -> Option<f32> {
    let gate_count = field.gate_count();
    let first_gate_km = field.first_gate_range_km();
    let gate_interval_km = field.gate_interval_km();

    // Azimuth axis: find bracketing radials
    let (az_lo, az_hi, az_frac) =
        find_bracketing_radials(field.azimuths(), azimuth_deg, max_azimuth_gap)?;

    // Range axis: find bracketing gates
    let gate_f = (distance_km - first_gate_km) / gate_interval_km;
    let g_lo = gate_f as usize;
    let g_hi = g_lo + 1;

    if g_hi >= gate_count {
        return None;
    }

    let g_frac = (gate_f - g_lo as f64) as f32;

    // Fetch the four corners and require all Valid
    let (v00, s00) = field.get(az_lo, g_lo);
    let (v01, s01) = field.get(az_lo, g_hi);
    let (v10, s10) = field.get(az_hi, g_lo);
    let (v11, s11) = field.get(az_hi, g_hi);

    if s00 != GateStatus::Valid
        || s01 != GateStatus::Valid
        || s10 != GateStatus::Valid
        || s11 != GateStatus::Valid
    {
        return None;
    }

    // Bilinear blend: lerp along range for each azimuth, then across azimuths
    let v_lo = v00 + (v01 - v00) * g_frac;
    let v_hi = v10 + (v11 - v10) * g_frac;
    Some(v_lo + (v_hi - v_lo) * az_frac)
}

/// Gate metadata extracted from a moment data block.
struct GateParams {
    first_gate_km: f64,
    gate_interval_km: f64,
    gate_count: usize,
}

/// Retrieve gate metadata from a radial for the given product.
fn get_gate_params(product: Product, radial: &Radial) -> Option<GateParams> {
    radial.moment(product).map(|moment| GateParams {
        first_gate_km: moment.first_gate_range_km(),
        gate_interval_km: moment.gate_interval_km(),
        gate_count: moment.gate_count() as usize,
    })
}

/// Extract decoded float values from a radial for the given product.
///
/// Returns `None` for below-threshold, range-folded, and CFP status gates.
fn get_radial_float_values(product: Product, radial: &Radial) -> Option<Vec<Option<f32>>> {
    radial.moment(product).map(|moment| {
        moment
            .iter()
            .map(|v| match v {
                MomentValue::Value(f) => Some(f),
                MomentValue::BelowThreshold
                | MomentValue::RangeFolded
                | MomentValue::CfpStatus(_) => None,
            })
            .collect()
    })
}
