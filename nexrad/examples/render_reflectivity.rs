//! Render a reflectivity image from a radar volume.
//!
//! This example demonstrates using the render feature to create
//! a PNG image of radar reflectivity data.
//!
//! Run with:
//! ```bash
//! cargo run --example render_reflectivity --features nexrad/render -- \
//!     tests/fixtures/convective/KDMX20220305_232324.bin output.png
//! ```

use nexrad::render::{get_nws_reflectivity_scale, render_radials, Product, RenderOptions};
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

    println!("Loading volume from: {}", input_path);
    let volume = nexrad::load_file(input_path)?;

    println!(
        "VCP: {}, {} sweeps",
        volume.coverage_pattern_number(),
        volume.sweeps().len()
    );

    // Get the first (lowest) sweep
    let sweep = volume
        .sweeps()
        .first()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No sweeps in volume"))?;

    println!(
        "Rendering sweep {} with {} radials...",
        sweep.elevation_number(),
        sweep.radials().len()
    );

    // Set up rendering options and color scale
    let options = RenderOptions::new(1024, 1024);
    let color_scale = get_nws_reflectivity_scale();

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
