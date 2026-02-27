//! Tests for bilinear interpolation rendering.

use nexrad_model::data::{
    CartesianField, GateStatus, MomentData, Product, Radial, RadialStatus, SweepField,
    VerticalField,
};
use nexrad_model::geo::{GeoExtent, GeoPoint};
use nexrad_render::{render_cartesian, render_sweep, render_vertical, ColorScale, RenderOptions};

// ---------------------------------------------------------------------------
// Helper: create a test radial at a given azimuth with uniform reflectivity
// ---------------------------------------------------------------------------

fn create_test_radial(azimuth_degrees: f32, raw_value: u8) -> Radial {
    let gate_count = 100;
    let first_gate_range_m = 2000;
    let gate_interval_m = 250;
    let scale = 2.0;
    let offset = 66.0;

    let values = vec![raw_value; gate_count as usize];
    let moment_data = MomentData::from_fixed_point(
        gate_count,
        first_gate_range_m,
        gate_interval_m,
        8,
        scale,
        offset,
        values,
    );

    Radial::new(
        0,
        0,
        azimuth_degrees,
        1.0,
        RadialStatus::IntermediateRadialData,
        1,
        0.5,
        Some(moment_data),
        None,
        None,
        None,
        None,
        None,
        None,
    )
}

/// Create a full 360-degree sweep of radials with a gradient in raw value.
fn create_gradient_sweep() -> Vec<Radial> {
    (0..360)
        .map(|az| {
            // Raw value varies by azimuth: 50..150 range to stay in valid encoding
            let raw = 50 + (az as u16 * 100 / 360) as u8;
            create_test_radial(az as f32, raw)
        })
        .collect()
}

/// Get pixel color at (x, y).
fn get_pixel(image: &nexrad_render::RgbaImage, x: u32, y: u32) -> [u8; 4] {
    let pixel = image.get_pixel(x, y);
    [pixel[0], pixel[1], pixel[2], pixel[3]]
}

// ---------------------------------------------------------------------------
// render_sweep tests
// ---------------------------------------------------------------------------

#[test]
fn test_bilinear_sweep_produces_different_output() {
    let radials = create_gradient_sweep();
    let field =
        SweepField::from_radials(&radials, Product::Reflectivity).expect("field is created");
    let scale = ColorScale::from(nexrad_render::nws_reflectivity_scale());

    let nearest_opts = RenderOptions::new(200, 200);
    let bilinear_opts = RenderOptions::new(200, 200).bilinear();

    let nearest_result = render_sweep(&field, &scale, &nearest_opts).expect("nearest renders");
    let bilinear_result = render_sweep(&field, &scale, &bilinear_opts).expect("bilinear renders");

    let nearest_img = nearest_result.image();
    let bilinear_img = bilinear_result.image();

    // At least some pixels should differ between nearest and bilinear
    let mut differ_count = 0u32;
    for y in 0..200 {
        for x in 0..200 {
            if get_pixel(nearest_img, x, y) != get_pixel(bilinear_img, x, y) {
                differ_count += 1;
            }
        }
    }

    assert!(
        differ_count > 0,
        "Bilinear should produce at least some different pixels than nearest-neighbor"
    );
}

#[test]
fn test_bilinear_sweep_preserves_gap_detection() {
    // Create radials only from 45 to 135 degrees (east sector)
    let radials: Vec<Radial> = (45..135)
        .map(|az| create_test_radial(az as f32, 100))
        .collect();

    let field =
        SweepField::from_radials(&radials, Product::Reflectivity).expect("field is created");
    let scale = ColorScale::from(nexrad_render::nws_reflectivity_scale());

    let background = [128, 128, 128, 255];
    let options = RenderOptions::new(200, 200)
        .with_background(background)
        .bilinear();

    let result = render_sweep(&field, &scale, &options).expect("renders");
    let image = result.image();

    // Pixel in covered region (~90 degrees east)
    let covered_pixel = get_pixel(image, 130, 100);
    assert_ne!(
        covered_pixel, background,
        "Pixel in covered azimuth should have radar data"
    );

    // Pixel in uncovered region (~270 degrees west)
    let uncovered_pixel = get_pixel(image, 70, 100);
    assert_eq!(
        uncovered_pixel, background,
        "Pixel in uncovered azimuth should remain background with bilinear"
    );
}

#[test]
fn test_bilinear_fallback_at_invalid_gates() {
    // Create a full sweep but with a known NoData region:
    // build a SweepField manually where some gates are NoData
    let radials = create_gradient_sweep();
    let mut field =
        SweepField::from_radials(&radials, Product::Reflectivity).expect("field is created");

    // Set a block of gates to NoData in the middle of the field
    let mid_az = field.azimuth_count() / 2;
    let mid_gate = field.gate_count() / 2;
    field.set(mid_az, mid_gate, 0.0, GateStatus::NoData);
    field.set(mid_az, mid_gate + 1, 0.0, GateStatus::NoData);

    let scale = ColorScale::from(nexrad_render::nws_reflectivity_scale());
    let background = [0, 0, 0, 255];
    let options = RenderOptions::new(200, 200)
        .with_background(background)
        .bilinear();

    // Should render without panicking — bilinear gracefully falls back to nearest-neighbor
    let result = render_sweep(&field, &scale, &options);
    assert!(
        result.is_ok(),
        "Bilinear render with NoData gates should succeed"
    );
}

#[test]
fn test_nearest_sweep_unchanged() {
    // Verify that nearest-neighbor mode is completely unchanged
    let radials = create_gradient_sweep();
    let field =
        SweepField::from_radials(&radials, Product::Reflectivity).expect("field is created");
    let scale = ColorScale::from(nexrad_render::nws_reflectivity_scale());

    let explicit_nearest =
        RenderOptions::new(200, 200).with_interpolation(nexrad_render::Interpolation::Nearest);
    let default_opts = RenderOptions::new(200, 200);

    let explicit_result = render_sweep(&field, &scale, &explicit_nearest).expect("nearest renders");
    let default_result = render_sweep(&field, &scale, &default_opts).expect("default renders");

    // Every pixel should be identical
    for y in 0..200 {
        for x in 0..200 {
            assert_eq!(
                get_pixel(explicit_result.image(), x, y),
                get_pixel(default_result.image(), x, y),
                "Explicit Nearest and default should produce identical output at ({}, {})",
                x,
                y
            );
        }
    }
}

// ---------------------------------------------------------------------------
// render_cartesian tests
// ---------------------------------------------------------------------------

#[test]
fn test_bilinear_cartesian_smooth() {
    // Create a small 4x4 CartesianField with a gradient
    let extent = GeoExtent {
        min: GeoPoint {
            latitude: 40.0,
            longitude: -94.0,
        },
        max: GeoPoint {
            latitude: 42.0,
            longitude: -92.0,
        },
    };
    let mut field = CartesianField::new("Test", "dBZ", extent, 4, 4);

    // Fill with a gradient: value = 10 * row + col (0..40 range)
    for row in 0..4 {
        for col in 0..4 {
            let val = (10 * row + col) as f32;
            field.set(row, col, val, GateStatus::Valid);
        }
    }

    let scale = ColorScale::from(nexrad_render::nws_reflectivity_scale());

    let nearest_opts = RenderOptions::new(16, 16);
    let bilinear_opts = RenderOptions::new(16, 16).bilinear();

    let nearest_result = render_cartesian(&field, &scale, &nearest_opts).expect("nearest renders");
    let bilinear_result =
        render_cartesian(&field, &scale, &bilinear_opts).expect("bilinear renders");

    // The images should differ (bilinear produces intermediate values)
    let mut differ_count = 0u32;
    for y in 0..16 {
        for x in 0..16 {
            if get_pixel(nearest_result.image(), x, y) != get_pixel(bilinear_result.image(), x, y) {
                differ_count += 1;
            }
        }
    }

    assert!(
        differ_count > 0,
        "Bilinear cartesian should produce at least some different pixels"
    );
}

#[test]
fn test_bilinear_cartesian_with_nodata_border() {
    // Create a 4x4 field where the edge is NoData
    let extent = GeoExtent {
        min: GeoPoint {
            latitude: 40.0,
            longitude: -94.0,
        },
        max: GeoPoint {
            latitude: 42.0,
            longitude: -92.0,
        },
    };
    let mut field = CartesianField::new("Test", "dBZ", extent, 4, 4);

    // Only fill interior 2x2 with valid data
    field.set(1, 1, 20.0, GateStatus::Valid);
    field.set(1, 2, 30.0, GateStatus::Valid);
    field.set(2, 1, 40.0, GateStatus::Valid);
    field.set(2, 2, 50.0, GateStatus::Valid);

    let scale = ColorScale::from(nexrad_render::nws_reflectivity_scale());
    let options = RenderOptions::new(16, 16).bilinear();

    // Should render without panicking — bilinear falls back to nearest at NoData borders
    let result = render_cartesian(&field, &scale, &options);
    assert!(result.is_ok(), "Bilinear with NoData border should succeed");
}

// ---------------------------------------------------------------------------
// render_vertical tests
// ---------------------------------------------------------------------------

#[test]
fn test_bilinear_vertical_smooth() {
    // Create a small 4x4 VerticalField with a gradient
    let mut field = VerticalField::new("Test", "dBZ", (0.0, 100.0), (0.0, 10000.0), 4, 4);

    for row in 0..4 {
        for col in 0..4 {
            let val = (10 * row + col) as f32;
            field.set(row, col, val, GateStatus::Valid);
        }
    }

    let scale = ColorScale::from(nexrad_render::nws_reflectivity_scale());

    let nearest_opts = RenderOptions::new(16, 16);
    let bilinear_opts = RenderOptions::new(16, 16).bilinear();

    let nearest_result = render_vertical(&field, &scale, &nearest_opts).expect("nearest renders");
    let bilinear_result =
        render_vertical(&field, &scale, &bilinear_opts).expect("bilinear renders");

    let mut differ_count = 0u32;
    for y in 0..16 {
        for x in 0..16 {
            if get_pixel(nearest_result.image(), x, y) != get_pixel(bilinear_result.image(), x, y) {
                differ_count += 1;
            }
        }
    }

    assert!(
        differ_count > 0,
        "Bilinear vertical should produce at least some different pixels"
    );
}
