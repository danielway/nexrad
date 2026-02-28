//! Process and render radar data through a filtering pipeline.
//!
//! This example demonstrates the full workflow:
//! 1. Load a scan from a local file
//! 2. Extract a SweepField for reflectivity
//! 3. Apply processing filters via a SweepPipeline
//! 4. Render the original and filtered data to PNG images
//! 5. Compute a composite reflectivity product and render it
//!
//! Run with:
//! ```bash
//! cargo run --example processing_pipeline --features "render,process" -- \
//!     tests/fixtures/convective/KDMX20220305_232324.bin
//! ```

use nexrad::model::data::Product;
use nexrad::process::derived::CompositeReflectivity;
use nexrad::process::filter::ThresholdFilter;
use nexrad::process::ScanDerivedProduct;
use nexrad::process::SweepPipeline;
use nexrad::render::{default_color_scale, render_sweep, ColorScale, RenderOptions};
use std::env;

fn main() -> nexrad::Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("tests/fixtures/convective/KDMX20220305_232324.bin");

    println!("Loading scan from: {}", path);
    let scan = nexrad::load_file(path)?;
    println!(
        "{}, {} sweeps",
        scan.coverage_pattern_number(),
        scan.sweeps().len()
    );

    // --- Extract a SweepField ---
    let (sweep_idx, field) = nexrad::extract_first_field(&scan, Product::Reflectivity)
        .expect("scan should contain reflectivity data");
    println!(
        "Extracted reflectivity from sweep {} ({:.1}° elevation, {} azimuths × {} gates)",
        sweep_idx,
        field.elevation_degrees(),
        field.azimuth_count(),
        field.gate_count(),
    );

    let color_scale: ColorScale = default_color_scale(Product::Reflectivity);

    // --- Render original (unprocessed) ---
    let options = RenderOptions::native_for(&field);
    let result = render_sweep(&field, &color_scale, &options)?;
    result.save("pipeline_original.png")?;
    println!("Saved pipeline_original.png");

    // --- Apply a processing pipeline ---
    let pipeline = SweepPipeline::new().then(ThresholdFilter {
        min: Some(5.0),
        max: Some(75.0),
    });

    let filtered = pipeline.execute(&field)?;

    let result = render_sweep(&filtered, &color_scale, &options)?;
    result.save("pipeline_filtered.png")?;
    println!("Saved pipeline_filtered.png (threshold 5-75 dBZ)");

    // --- Compute composite reflectivity ---
    let ref_fields = nexrad::extract_fields(&scan, Product::Reflectivity);
    println!(
        "Extracted {} elevation tilts for composite",
        ref_fields.len()
    );

    let coord_sys = nexrad::coordinate_system_required(&scan)?;
    let extent = coord_sys.sweep_extent(ref_fields[0].max_range_km());

    let composite =
        CompositeReflectivity.compute(&scan, &ref_fields, &coord_sys, &extent, (800, 800))?;

    let cart_result =
        nexrad::render::render_cartesian(&composite, &color_scale, &RenderOptions::new(800, 800))?;
    cart_result.save("pipeline_composite.png")?;
    println!("Saved pipeline_composite.png (composite reflectivity)");

    // --- Demonstrate RenderResult metadata ---
    let result = render_sweep(&field, &color_scale, &options)?;
    let meta = result.metadata();
    println!("\nRender metadata:");
    println!("  Image size: {}x{}", meta.width(), meta.height());
    println!("  Pixels per km: {:.2}", meta.pixels_per_km());
    println!("  Max range: {:.1} km", meta.max_range_km());
    if let Some(extent) = meta.geo_extent() {
        println!(
            "  Geo extent: ({:.4}, {:.4}) to ({:.4}, {:.4})",
            extent.min.latitude, extent.min.longitude, extent.max.latitude, extent.max.longitude
        );
    }

    // Query a specific pixel
    let center = (meta.width() as f64 / 2.0, meta.height() as f64 / 2.0);
    if let Some(query) = result.query_pixel(&field, center.0 + 100.0, center.1) {
        println!(
            "\nPoint query at pixel ({:.0}, {:.0}):",
            center.0 + 100.0,
            center.1
        );
        println!(
            "  Polar: {:.1}° azimuth, {:.1} km range",
            query.polar.azimuth_degrees, query.polar.range_km
        );
        println!("  Value: {:.1} {}", query.value, field.unit());
        println!("  Status: {:?}", query.status);
    }

    Ok(())
}
