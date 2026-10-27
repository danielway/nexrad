//! Rendering of polar sweep data.

use crate::options::Palette;
use crate::{
    ColorScale, Error, Interpolation, Projection, RenderMetadata, RenderOptions, RenderResult,
    Result, RgbaImage,
};
use nexrad_model::data::{GateStatus, SweepField};
use nexrad_model::geo::{GeoExtent, GeoPoint, RadarCoordinateSystem};

/// Renders a [`SweepField`] (polar grid) to an image with metadata.
///
/// This is the primary rendering entry point for processed data. It accepts
/// a `SweepField` — the output of data extraction or processing — and produces
/// a [`RenderResult`] that includes both the rendered image and metadata for
/// geographic placement and data inspection.
///
/// Without an [extent](RenderOptions::with_extent) the image is a radar-centred plan
/// view with north up, scaled so the field's maximum range touches the edges of the
/// longer image side ([`Projection::Polar`]). With an extent the sweep is drawn onto
/// a latitude/longitude grid covering exactly that extent
/// ([`Projection::Geographic`]): each pixel centre is mapped to the polar point a beam
/// at the sweep's elevation passes over, using the options' coordinate system.
///
/// # Arguments
///
/// * `field` - The sweep field to render
/// * `color_scale` - Color scale to apply (discrete or continuous)
/// * `options` - Rendering options (size, background, extent, coordinate system,
///   interpolation, value range, status colors)
///
/// # Errors
///
/// Returns [`Error::InvalidDimensions`] if either output dimension is zero,
/// [`Error::EmptyField`] if the field has no radials, [`Error::InvalidRange`] if the
/// options' value range is invalid, and, when an extent is set,
/// [`Error::MissingCoordinateSystem`] without a coordinate system and
/// [`Error::InvalidExtent`] if the extent is not finite and ordered.
///
/// # Example
///
/// ```no_run
/// use nexrad_model::data::Product;
/// use nexrad_render::{default_scale, render_sweep, RenderOptions};
/// # use nexrad_model::data::{MomentData, Radial, Sweep};
/// # let reflectivity = MomentData::builder(Product::Reflectivity, vec![100, 110, 120])
/// #     .first_gate_range_meters(2000)
/// #     .gate_interval_meters(250)
/// #     .scale(2.0)
/// #     .offset(66.0)
/// #     .build()?;
/// # let radial = Radial::builder(0, 0.0, 1, 0.5).moment(reflectivity).build()?;
/// # let sweep = Sweep::new(1, vec![radial])?;
///
/// let field = sweep.field(Product::Reflectivity).unwrap();
/// let scale = default_scale(Product::Reflectivity);
/// let result = render_sweep(&field, &scale, &RenderOptions::new(800, 800))?;
///
/// result.save("radar.png")?;
/// let meta = result.metadata();
/// // Polar renders support range/pixel conversions.
/// let ring_px = meta.km_to_pixel_distance(100.0);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn render_sweep(
    field: &SweepField,
    color_scale: &ColorScale,
    options: &RenderOptions,
) -> Result<RenderResult> {
    let mut buffer = options.background_buffer()?;

    if field.azimuth_count() == 0 {
        return Err(Error::EmptyField);
    }

    let palette = options.palette(color_scale, field.product(), field.value_range())?;

    match options.extent {
        Some(extent) => {
            let coord_system = options
                .coord_system
                .as_ref()
                .ok_or(Error::MissingCoordinateSystem)?;
            render_geographic(field, &palette, options, extent, coord_system, &mut buffer)?;
            finish(
                buffer,
                options,
                Projection::Geographic { extent },
                Some(extent),
            )
        }
        None => {
            let projection = render_polar(field, &palette, options, &mut buffer);
            let geo_extent = options
                .coord_system
                .as_ref()
                .map(|cs| cs.sweep_extent(field.max_range_km()));
            finish(buffer, options, projection, geo_extent)
        }
    }
}

/// Wraps a filled buffer and its projection into a [`RenderResult`].
fn finish(
    buffer: Vec<u8>,
    options: &RenderOptions,
    projection: Projection,
    geo_extent: Option<GeoExtent>,
) -> Result<RenderResult> {
    let (width, height) = options.size;
    let image = RgbaImage::from_raw(width, height, buffer).ok_or(Error::InvalidDimensions)?;
    let metadata = RenderMetadata::new(
        width,
        height,
        projection,
        geo_extent,
        options.coord_system.clone(),
    );
    Ok(RenderResult::new(image, metadata))
}

/// Writes `color` to pixel `(x, y)` of a row-major RGBA buffer `width` pixels wide.
#[inline]
fn put_pixel(buffer: &mut [u8], width: usize, x: usize, y: usize, color: [u8; 4]) {
    let pixel_index = (y * width + x) * 4;
    buffer[pixel_index..pixel_index + 4].copy_from_slice(&color);
}

/// Renders the sweep as a radar-centred plan view and returns its projection.
fn render_polar(
    field: &SweepField,
    palette: &Palette,
    options: &RenderOptions,
    buffer: &mut [u8],
) -> Projection {
    let (width, height) = (options.size.0 as usize, options.size.1 as usize);

    let first_gate_km = field.first_gate_range_km();
    let gate_interval_km = field.gate_interval_km();
    let gate_count = field.gate_count();
    let radar_range_km = field.max_range_km();

    // Calculate max azimuth gap for partial sweep support
    let max_azimuth_gap = field.azimuth_spacing_degrees() * 1.5;

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
                    put_pixel(buffer, width, x, y, palette.value_color(val));
                    continue;
                }
            }

            // Nearest-neighbor path (default, or bilinear fallback)
            let (radial_idx, angular_distance) =
                find_closest_radial(field.azimuth_angles_degrees(), azimuth_deg);

            if angular_distance > max_azimuth_gap {
                continue;
            }

            let gate_index = ((distance_km - first_gate_km) / gate_interval_km) as usize;
            if gate_index >= gate_count {
                continue;
            }

            let Some((val, status)) = field.get(radial_idx, gate_index) else {
                continue;
            };
            if let Some(color) = palette.gate_color(val, status) {
                put_pixel(buffer, width, x, y, color);
            }
        }
    }

    Projection::Polar {
        center_pixel: (center_x, center_y),
        pixels_per_km: scale_factor,
        max_range_km: radar_range_km,
        elevation_angle_degrees: f64::from(field.elevation_angle_degrees()),
    }
}

/// Renders the sweep onto a latitude/longitude grid covering `extent`.
///
/// Each pixel centre is converted to geographic coordinates (linear in latitude and
/// longitude, matching [`RenderMetadata::pixel_to_geo`]) and then to the polar point
/// a beam at the sweep's elevation passes over.
fn render_geographic(
    field: &SweepField,
    palette: &Palette,
    options: &RenderOptions,
    extent: GeoExtent,
    coord_system: &RadarCoordinateSystem,
    buffer: &mut [u8],
) -> Result<()> {
    let GeoExtent { min, max } = extent;
    let ordered = |low: f64, high: f64| low.is_finite() && high.is_finite() && low < high;
    if !(ordered(min.latitude, max.latitude) && ordered(min.longitude, max.longitude)) {
        return Err(Error::InvalidExtent);
    }

    let (width, height) = (options.size.0 as usize, options.size.1 as usize);
    let elevation = f64::from(field.elevation_angle_degrees());
    let first_gate_km = field.first_gate_range_km();
    let max_range_km = field.max_range_km();
    let max_azimuth_gap = field.azimuth_spacing_degrees() * 1.5;
    let use_bilinear = options.interpolation == Interpolation::Bilinear;

    let lat_span = max.latitude - min.latitude;
    let lon_span = max.longitude - min.longitude;
    let longitudes: Vec<f64> = (0..width)
        .map(|x| min.longitude + (x as f64 + 0.5) / width as f64 * lon_span)
        .collect();

    for y in 0..height {
        let latitude = max.latitude - (y as f64 + 0.5) / height as f64 * lat_span;

        for (x, &longitude) in longitudes.iter().enumerate() {
            let Some(polar) = coord_system.geo_to_polar(
                GeoPoint {
                    latitude,
                    longitude,
                },
                elevation,
            ) else {
                continue;
            };
            let range_km = polar.range_km;
            if !(range_km >= first_gate_km && range_km < max_range_km) {
                continue;
            }

            if use_bilinear {
                if let Some(val) = sample_sweep_bilinear(
                    field,
                    polar.azimuth_angle_degrees as f32,
                    range_km,
                    max_azimuth_gap,
                ) {
                    put_pixel(buffer, width, x, y, palette.value_color(val));
                    continue;
                }
            }

            let Some((val, status)) = field.value_at_polar(polar.azimuth_angle_degrees, range_km)
            else {
                continue;
            };
            if let Some(color) = palette.gate_color(val, status) {
                put_pixel(buffer, width, x, y, color);
            }
        }
    }

    Ok(())
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
        find_bracketing_radials(field.azimuth_angles_degrees(), azimuth_deg, max_azimuth_gap)?;

    // Range axis: find bracketing gates
    let gate_f = (distance_km - first_gate_km) / gate_interval_km;
    let g_lo = gate_f as usize;
    let g_hi = g_lo + 1;

    if g_hi >= gate_count {
        return None;
    }

    let g_frac = (gate_f - g_lo as f64) as f32;

    // Fetch the four corners and require all Valid
    let (v00, s00) = field.get(az_lo, g_lo)?;
    let (v01, s01) = field.get(az_lo, g_hi)?;
    let (v10, s10) = field.get(az_hi, g_lo)?;
    let (v11, s11) = field.get(az_hi, g_hi)?;

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
