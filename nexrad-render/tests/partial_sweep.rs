//! Tests for rendering partial sweeps (subsets of radials).
//!
//! When rendering a subset of radials that doesn't cover the full 360° azimuth,
//! pixels in uncovered azimuth sectors should remain the background color.

use nexrad_model::data::{MomentData, Radial, RadialStatus};
use nexrad_render::{render_radials, Product, RenderOptions};

/// Creates a test radial at the given azimuth angle with uniform reflectivity data.
fn create_test_radial(azimuth_degrees: f32, reflectivity_value: u8) -> Radial {
    // Create moment data with 100 gates, starting at 2km, 0.25km intervals
    // Using scale=2.0, offset=66.0 which maps raw value 100 to ~17 dBZ
    let gate_count = 100;
    let first_gate_range_m = 2000; // 2km in meters
    let gate_interval_m = 250; // 0.25km in meters
    let scale = 2.0;
    let offset = 66.0;

    let values = vec![reflectivity_value; gate_count as usize];
    let moment_data = MomentData::from_fixed_point(
        gate_count,
        first_gate_range_m,
        gate_interval_m,
        8, // data_word_size: 8-bit encoding
        scale,
        offset,
        values,
    );

    Radial::new(
        0,               // collection_timestamp
        0,               // azimuth_number
        azimuth_degrees, // azimuth_angle_degrees
        1.0,             // azimuth_spacing_degrees
        RadialStatus::IntermediateRadialData,
        1,                 // elevation_number
        0.5,               // elevation_angle_degrees
        Some(moment_data), // reflectivity
        None,              // velocity
        None,              // spectrum_width
        None,              // differential_reflectivity
        None,              // differential_phase
        None,              // correlation_coefficient
        None,              // specific_differential_phase
    )
}

/// Creates radials covering a specific azimuth range.
fn create_partial_sweep(
    start_az: f32,
    end_az: f32,
    spacing: f32,
    reflectivity_value: u8,
) -> Vec<Radial> {
    let mut radials = Vec::new();
    let mut az = start_az;
    while az < end_az {
        radials.push(create_test_radial(az, reflectivity_value));
        az += spacing;
    }
    radials
}

/// Get the pixel color at a specific (x, y) coordinate from the image.
fn get_pixel(image: &nexrad_render::RgbaImage, x: u32, y: u32) -> [u8; 4] {
    let pixel = image.get_pixel(x, y);
    [pixel[0], pixel[1], pixel[2], pixel[3]]
}

/// Convert pixel coordinates to azimuth angle (0 = North, clockwise).
fn pixel_to_azimuth(x: u32, y: u32, width: u32, height: u32) -> f32 {
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let dx = x as f32 - center_x;
    let dy = y as f32 - center_y;
    let azimuth_rad = dx.atan2(-dy);
    (azimuth_rad.to_degrees() + 360.0) % 360.0
}

#[test]
fn test_partial_sweep_leaves_gaps_as_background() {
    // Create radials only from 0° to 90° (northeast quadrant)
    let radials = create_partial_sweep(0.0, 90.0, 1.0, 100);

    let background = [0, 0, 0, 255]; // Black background
    let options = RenderOptions::new(200, 200).with_background(background);

    let image = render_radials(
        &radials,
        Product::Reflectivity,
        &nexrad_render::get_nws_reflectivity_scale(),
        &options,
    )
    .unwrap();

    // Sample a pixel in the covered region (azimuth ~45°, which is northeast)
    // At 45°: dx = positive, dy = negative (upper right quadrant)
    // For a 200x200 image centered at (100, 100):
    // Pixel at (130, 70) is at azimuth ~45° and within radar range
    let covered_pixel = get_pixel(&image, 130, 70);
    let covered_azimuth = pixel_to_azimuth(130, 70, 200, 200);
    assert!(
        covered_azimuth >= 0.0 && covered_azimuth <= 90.0,
        "Test setup error: covered pixel azimuth {} should be in [0, 90]",
        covered_azimuth
    );

    // This pixel should have radar data (not background)
    assert_ne!(
        covered_pixel, background,
        "Pixel in covered azimuth range should have radar data, not background"
    );

    // Sample a pixel in the UNCOVERED region (azimuth ~180°, which is south)
    // At 180°: dx = 0, dy = positive (bottom of image)
    // Pixel at (100, 130) is at azimuth ~180° and within radar range
    let uncovered_pixel = get_pixel(&image, 100, 130);
    let uncovered_azimuth = pixel_to_azimuth(100, 130, 200, 200);
    assert!(
        uncovered_azimuth >= 135.0 && uncovered_azimuth <= 225.0,
        "Test setup error: uncovered pixel azimuth {} should be around 180°",
        uncovered_azimuth
    );

    // BUG: Currently this assertion FAILS because the renderer stretches
    // the available radials to fill the entire 360° sweep.
    // After the fix, pixels in uncovered azimuths should remain background.
    assert_eq!(
        uncovered_pixel, background,
        "Pixel at azimuth {:.1}° should be background since no radials cover 90°-360°. \
         Got {:?} instead of {:?}. This indicates the bug where partial sweeps \
         incorrectly fill uncovered regions.",
        uncovered_azimuth, uncovered_pixel, background
    );
}

#[test]
fn test_full_sweep_covers_all_azimuths() {
    // Create a full 360° sweep
    let radials = create_partial_sweep(0.0, 360.0, 1.0, 100);

    let background = [0, 0, 0, 255];
    let options = RenderOptions::new(200, 200).with_background(background);

    let image = render_radials(
        &radials,
        Product::Reflectivity,
        &nexrad_render::get_nws_reflectivity_scale(),
        &options,
    )
    .unwrap();

    // Sample pixels at various azimuths - all should have data
    let test_points = [
        (130, 70),  // ~45° NE
        (130, 130), // ~135° SE
        (70, 130),  // ~225° SW
        (70, 70),   // ~315° NW
    ];

    for (x, y) in test_points {
        let pixel = get_pixel(&image, x, y);
        let azimuth = pixel_to_azimuth(x, y, 200, 200);
        assert_ne!(
            pixel, background,
            "Full sweep: pixel at azimuth {:.1}° should have data",
            azimuth
        );
    }
}

#[test]
fn test_partial_sweep_gap_detection_multiple_sectors() {
    // Create radials only from 45° to 135° (east sector)
    let radials = create_partial_sweep(45.0, 135.0, 1.0, 100);

    let background = [128, 128, 128, 255]; // Gray background for visibility
    let options = RenderOptions::new(200, 200).with_background(background);

    let image = render_radials(
        &radials,
        Product::Reflectivity,
        &nexrad_render::get_nws_reflectivity_scale(),
        &options,
    )
    .unwrap();

    // Pixel in covered region (~90° east)
    let covered_pixel = get_pixel(&image, 130, 100);
    assert_ne!(
        covered_pixel, background,
        "Pixel in covered azimuth (east) should have radar data"
    );

    // Pixel in uncovered region (~270° west)
    let uncovered_pixel = get_pixel(&image, 70, 100);
    assert_eq!(
        uncovered_pixel, background,
        "Pixel in uncovered azimuth (west) should be background"
    );

    // Pixel in uncovered region (~0° north)
    let north_pixel = get_pixel(&image, 100, 70);
    assert_eq!(
        north_pixel, background,
        "Pixel in uncovered azimuth (north) should be background"
    );
}
