//! Render a reflectivity image from a radar scan.
//!
//! This example demonstrates using the render feature to create
//! a PNG image of radar reflectivity data.
//!
//! Run with:
//! ```bash
//! cargo run --example render_reflectivity --features nexrad/render -- \
//!     tests/fixtures/convective/KDMX20220305_232324.bin output.png
//! ```

use nexrad::model::data::Product;
use nexrad::render::{nws_reflectivity_scale, render_radials, RenderOptions};
use std::env;

fn main() -> nexrad::Result<()> {
    let args: Vec<String> = env::args().collect();

    let input_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("tests/fixtures/convective/KDMX20220305_232324.bin");
    let output_path = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("reflectivity.png");

    println!("Loading scan from: {}", input_path);
    let scan = nexrad::load_file(input_path)?;

    println!(
        "{}, {} sweeps",
        scan.coverage_pattern_number(),
        scan.sweeps().len()
    );

    // Get the first (lowest) sweep
    let sweep = scan
        .sweeps()
        .first()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No sweeps in scan"))?;

    println!(
        "Rendering sweep {} with {} radials...",
        sweep.elevation_number(),
        sweep.radials().len()
    );

    // Set up rendering options and color scale
    let options = RenderOptions::new(1024, 1024);
    let color_scale = nws_reflectivity_scale();

    // Render the image
    let image = render_radials(
        sweep.radials(),
        Product::Reflectivity,
        &color_scale,
        &options,
    )?;

    // Save to file
    image
        .save(output_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    println!("Saved reflectivity image to: {}", output_path);

    Ok(())
}
