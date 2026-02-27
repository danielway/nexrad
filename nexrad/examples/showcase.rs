//! Comprehensive showcase of NEXRAD rendering and processing capabilities.
//!
//! Uses the KDMX (Des Moines, Iowa) radar volume from March 5, 2022 at ~23:16 UTC,
//! a full 23-sweep volume scan during a significant severe weather event, to demonstrate:
//!
//! 1. All radar products (reflectivity, velocity, spectrum width, ZDR, CC, PhiDP, CFP)
//! 2. Multiple elevation angles (0.5 to 19.5 degrees)
//! 3. Processing: thresholding, smoothing, median filtering, CC-based clutter removal
//! 4. Processing pipelines combining multiple steps
//! 5. Storm-relative velocity
//! 6. Composite reflectivity (cartesian rendering)
//! 7. Vertical cross-sections (RHI-style rendering)
//! 8. Render metadata and point queries
//! 9. Full native-resolution rendering
//!
//! Run with:
//! ```bash
//! cargo run --example showcase --release
//! ```
//!
//! Output images are saved to `renders/`.

use nexrad::model::data::Product;
use nexrad::model::geo::GeoPoint;
use nexrad::process::derived::{CompositeReflectivity, StormRelativeVelocity};
use nexrad::process::filter::{
    CorrelationCoefficientFilter, GaussianSmooth, MedianFilter, ThresholdFilter,
};
use nexrad::process::{SweepPipeline, SweepProcessor, VolumeDerivedProduct};
use nexrad::render::{
    get_default_scale, render_cartesian, render_sweep, render_vertical, ColorScale, RenderOptions,
};

use nexrad::model::data::{GateStatus, VerticalField};

/// Full 23-sweep volume downloaded from AWS via `download_volume` example.
/// Falls back to the 3-sweep fixture if the full volume is not present.
const FULL_VOLUME: &str = "tests/fixtures/full/KDMX20220305_231630_V06";
const FALLBACK_FIXTURE: &str = "tests/fixtures/convective/KDMX20220305_232324.bin";
const OUTPUT_DIR: &str = "renders";

fn main() -> nexrad::Result<()> {
    // ========================================================================
    // Load the volume
    // ========================================================================
    let fixture = if std::path::Path::new(FULL_VOLUME).exists() {
        FULL_VOLUME
    } else {
        println!("Full volume not found at {}", FULL_VOLUME);
        println!(
            "Run: cargo run --example download_volume -- KDMX 2022-03-05 23:23\n\
             Falling back to 3-sweep fixture...\n"
        );
        FALLBACK_FIXTURE
    };

    println!("Loading radar volume from {}...", fixture);
    let volume = nexrad::load_file(fixture)?;

    let vcp = volume.coverage_pattern_number();
    let num_sweeps = volume.sweeps().len();
    println!("  VCP: {}, {} sweeps", vcp, num_sweeps);

    if let Some((start, end)) = volume.time_range() {
        println!("  Time range: {} to {}", start, end);
    }

    // List available elevations
    for (i, sweep) in volume.sweeps().iter().enumerate() {
        if let Some(angle) = sweep.elevation_angle_degrees() {
            let has_ref = sweep
                .radials()
                .first()
                .and_then(|r| r.reflectivity())
                .is_some();
            let has_vel = sweep
                .radials()
                .first()
                .and_then(|r| r.velocity())
                .is_some();
            println!(
                "  Sweep {}: {:.1} deg, {} radials [REF:{} VEL:{}]",
                i,
                angle,
                sweep.radials().len(),
                if has_ref { "Y" } else { "N" },
                if has_vel { "Y" } else { "N" },
            );
        }
    }

    // Build coordinate system for geographic operations
    let coord_sys = nexrad::coordinate_system(&volume).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "No site metadata in volume")
    })?;

    // Create output directory
    std::fs::create_dir_all(OUTPUT_DIR)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // ========================================================================
    // Compute native resolution from the data itself
    // ========================================================================
    let sweep_0 = &volume.sweeps()[0];
    let ref_field = nexrad::extract_field(sweep_0, Product::Reflectivity)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No reflectivity"))?;

    // Native resolution: 2 * gate_count ensures ~1 pixel per gate at the outer edge.
    // For the KDMX reflectivity data: 1832 gates * 0.25 km = 460 km range -> 3664 px image.
    let native_size = ref_field.gate_count() * 2;
    println!(
        "\n  Native resolution: {} px (from {} gates x {:.2} km = {:.0} km range)",
        native_size,
        ref_field.gate_count(),
        ref_field.gate_interval_km(),
        ref_field.max_range_km(),
    );
    println!(
        "  Azimuthal resolution: {} radials x {:.1} deg spacing",
        ref_field.azimuth_count(),
        ref_field.azimuth_spacing_degrees(),
    );

    // Use native resolution for key renders, a moderate size for processing comparisons
    let native_options = RenderOptions::new(native_size, native_size);
    let compare_size: usize = 1600;
    let compare_options = RenderOptions::new(compare_size, compare_size);

    let ref_scale = ColorScale::from(get_default_scale(Product::Reflectivity));

    // ========================================================================
    // Section 1: All Radar Products at Native Resolution
    // ========================================================================
    println!("\n=== Section 1: All Radar Products (native resolution) ===");

    let products = [
        Product::Reflectivity,
        Product::Velocity,
        Product::SpectrumWidth,
        Product::DifferentialReflectivity,
        Product::DifferentialPhase,
        Product::CorrelationCoefficient,
        Product::ClutterFilterPower,
    ];

    // For products not on sweep 0, find the first sweep that has them
    for product in &products {
        let (sweep_idx, field) = volume
            .sweeps()
            .iter()
            .enumerate()
            .find_map(|(i, s)| nexrad::extract_field(s, *product).map(|f| (i, f)))
            .map(|(i, f)| (Some(i), Some(f)))
            .unwrap_or((None, None));

        match field {
            Some(f) => {
                // Use native resolution matched to this field's gate count
                let field_native = f.gate_count() * 2;
                let opts = RenderOptions::new(field_native, field_native);
                let scale = ColorScale::from(get_default_scale(*product));
                let result = render_sweep(&f, &scale, &opts)?;
                let path = format!("{}/01_product_{}.png", OUTPUT_DIR, slug(product.label()));
                result.image().save(&path).map_err(io_err)?;
                println!(
                    "  {} (sweep {}) - {:.1} deg, {:.0} km, {}x{} gates -> {}x{} px",
                    product.label(),
                    sweep_idx.unwrap_or(0),
                    f.elevation_degrees(),
                    f.max_range_km(),
                    f.azimuth_count(),
                    f.gate_count(),
                    field_native,
                    field_native,
                );
                if let Some((min, max)) = f.value_range() {
                    println!(
                        "    Value range: {:.1} to {:.1} {}",
                        min,
                        max,
                        product.unit()
                    );
                }
                println!("    -> {}", path);
            }
            None => {
                println!("  {} - not available in any sweep", product.label());
            }
        }
    }

    // ========================================================================
    // Section 2: Reflectivity at Multiple Elevation Angles
    // ========================================================================
    println!("\n=== Section 2: Multiple Elevation Angles ===");

    // Deduplicate elevations — only render the first (REF-only) sweep per angle
    let mut seen_elevations = std::collections::HashSet::new();
    for (i, sweep) in volume.sweeps().iter().enumerate() {
        if let Some(field) = nexrad::extract_field(sweep, Product::Reflectivity) {
            // Round to 1 decimal to group identical angles
            let angle_key = (field.elevation_degrees() * 10.0).round() as i32;
            if !seen_elevations.insert(angle_key) {
                continue;
            }

            let angle = field.elevation_degrees();
            let result = render_sweep(&field, &ref_scale, &compare_options)?;
            let path = format!("{}/02_elevation_{:02}_{:.1}deg.png", OUTPUT_DIR, i, angle);
            result.image().save(&path).map_err(io_err)?;
            println!("  Sweep {}: {:.1} deg -> {}", i, angle, path);
        }
    }

    // ========================================================================
    // Section 3: Processing - Individual Effects
    // ========================================================================
    println!("\n=== Section 3: Processing Effects ===");

    // 3a. Unprocessed (raw) at native resolution
    {
        let result = render_sweep(&ref_field, &ref_scale, &native_options)?;
        let path = format!("{}/03a_raw.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Raw (unprocessed, native {}px) -> {}", native_size, path);
    }

    // 3b. Threshold filter - remove weak echoes below 10 dBZ
    {
        let filter = ThresholdFilter {
            min: Some(10.0),
            max: None,
        };
        let filtered = filter.process(&ref_field).map_err(process_err)?;
        let result = render_sweep(&filtered, &ref_scale, &compare_options)?;
        let path = format!("{}/03b_threshold_10dbz.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Threshold (>10 dBZ) -> {}", path);
    }

    // 3c. Threshold filter - isolate strong echoes (>35 dBZ)
    {
        let filter = ThresholdFilter {
            min: Some(35.0),
            max: None,
        };
        let filtered = filter.process(&ref_field).map_err(process_err)?;
        let result = render_sweep(&filtered, &ref_scale, &compare_options)?;
        let path = format!("{}/03c_threshold_35dbz.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Threshold (>35 dBZ, strong echoes) -> {}", path);
    }

    // 3d. Gaussian smoothing - light
    {
        let smoother = GaussianSmooth {
            sigma_azimuth: 1.5,
            sigma_range: 1.5,
        };
        let smoothed = smoother.process(&ref_field).map_err(process_err)?;
        let result = render_sweep(&smoothed, &ref_scale, &compare_options)?;
        let path = format!("{}/03d_gaussian_light.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Gaussian smooth (sigma=1.5) -> {}", path);
    }

    // 3e. Gaussian smoothing - heavy
    {
        let smoother = GaussianSmooth {
            sigma_azimuth: 4.0,
            sigma_range: 4.0,
        };
        let smoothed = smoother.process(&ref_field).map_err(process_err)?;
        let result = render_sweep(&smoothed, &ref_scale, &compare_options)?;
        let path = format!("{}/03e_gaussian_heavy.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Gaussian smooth (sigma=4.0) -> {}", path);
    }

    // 3f. Median filter - 3x3 (spike removal)
    {
        let filter = MedianFilter {
            azimuth_kernel: 3,
            range_kernel: 3,
        };
        let filtered = filter.process(&ref_field).map_err(process_err)?;
        let result = render_sweep(&filtered, &ref_scale, &compare_options)?;
        let path = format!("{}/03f_median_3x3.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Median filter (3x3) -> {}", path);
    }

    // 3g. Median filter - 7x7 (aggressive smoothing)
    {
        let filter = MedianFilter {
            azimuth_kernel: 7,
            range_kernel: 7,
        };
        let filtered = filter.process(&ref_field).map_err(process_err)?;
        let result = render_sweep(&filtered, &ref_scale, &compare_options)?;
        let path = format!("{}/03g_median_7x7.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Median filter (7x7) -> {}", path);
    }

    // 3h. Correlation coefficient-based noise removal
    {
        let cc_field = nexrad::extract_field(sweep_0, Product::CorrelationCoefficient);
        match cc_field {
            Some(cc) => {
                if cc.azimuth_count() == ref_field.azimuth_count()
                    && cc.gate_count() == ref_field.gate_count()
                {
                    let filter =
                        CorrelationCoefficientFilter::new(0.90, cc).map_err(process_err)?;
                    let cleaned = filter.process(&ref_field).map_err(process_err)?;
                    let result = render_sweep(&cleaned, &ref_scale, &compare_options)?;
                    let path = format!("{}/03h_cc_noise_removal.png", OUTPUT_DIR);
                    result.image().save(&path).map_err(io_err)?;
                    println!("  CC noise removal (threshold=0.90) -> {}", path);
                } else {
                    // REF has more gates (460 km) than CC (300 km).
                    // Render CC at its own native resolution for reference.
                    println!(
                        "  CC noise removal - geometry mismatch (REF: {}x{}, CC: {}x{}), skipping",
                        ref_field.azimuth_count(),
                        ref_field.gate_count(),
                        cc.azimuth_count(),
                        cc.gate_count()
                    );
                    let cc_scale =
                        ColorScale::from(get_default_scale(Product::CorrelationCoefficient));
                    let result = render_sweep(&cc, &cc_scale, &compare_options)?;
                    let path = format!("{}/03h_cc_field.png", OUTPUT_DIR);
                    result.image().save(&path).map_err(io_err)?;
                    println!("    Rendered CC field for reference -> {}", path);
                }
            }
            None => {
                println!("  CC noise removal - CC product not available, skipping");
            }
        }
    }

    // ========================================================================
    // Section 4: Processing Pipelines (Combined Effects)
    // ========================================================================
    println!("\n=== Section 4: Processing Pipelines ===");

    // 4a. Clean + smooth pipeline: threshold -> median -> gaussian
    {
        let pipeline = SweepPipeline::new()
            .then(ThresholdFilter {
                min: Some(5.0),
                max: None,
            })
            .then(MedianFilter {
                azimuth_kernel: 3,
                range_kernel: 3,
            })
            .then(GaussianSmooth {
                sigma_azimuth: 1.5,
                sigma_range: 1.5,
            });

        let processed = pipeline.execute(&ref_field).map_err(process_err)?;
        let result = render_sweep(&processed, &ref_scale, &compare_options)?;
        let path = format!("{}/04a_pipeline_clean_smooth.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!(
            "  Pipeline (threshold>5 + median 3x3 + gaussian 1.5) -> {}",
            path
        );
    }

    // 4b. Analysis pipeline: threshold -> heavy median for structure identification
    {
        let pipeline = SweepPipeline::new()
            .then(ThresholdFilter {
                min: Some(20.0),
                max: None,
            })
            .then(MedianFilter {
                azimuth_kernel: 5,
                range_kernel: 5,
            });

        let processed = pipeline.execute(&ref_field).map_err(process_err)?;
        let result = render_sweep(&processed, &ref_scale, &compare_options)?;
        let path = format!("{}/04b_pipeline_structure.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Pipeline (threshold>20 + median 5x5) -> {}", path);
    }

    // ========================================================================
    // Section 5: Storm-Relative Velocity
    // ========================================================================
    println!("\n=== Section 5: Storm-Relative Velocity ===");

    // Find a sweep with velocity data
    let vel_sweep = volume.sweeps().iter().find(|s| {
        s.radials()
            .first()
            .and_then(|r| r.velocity())
            .is_some()
    });

    if let Some(sweep) = vel_sweep {
        let vel_field = nexrad::extract_field(sweep, Product::Velocity);
        if let Some(vf) = vel_field {
            let vel_native = vf.gate_count() * 2;
            let vel_native_options = RenderOptions::new(vel_native, vel_native);
            let vel_scale = ColorScale::from(get_default_scale(Product::Velocity));

            // 5a. Raw velocity at native resolution
            {
                let result = render_sweep(&vf, &vel_scale, &vel_native_options)?;
                let path = format!("{}/05a_velocity_raw.png", OUTPUT_DIR);
                result.image().save(&path).map_err(io_err)?;
                println!(
                    "  Raw velocity ({:.1} deg, {}px native) -> {}",
                    vf.elevation_degrees(),
                    vel_native,
                    path
                );
            }

            // 5b. Storm-relative velocity (storm moving from SW at 20 m/s)
            {
                let srv = StormRelativeVelocity::new(240.0, 20.0).map_err(process_err)?;
                let sr_field = srv.process(&vf).map_err(process_err)?;
                let result = render_sweep(&sr_field, &vel_scale, &vel_native_options)?;
                let path = format!("{}/05b_velocity_storm_relative.png", OUTPUT_DIR);
                result.image().save(&path).map_err(io_err)?;
                println!(
                    "  Storm-relative velocity (from 240 deg @ 20 m/s) -> {}",
                    path
                );
            }

            // 5c. Storm-relative + smoothing pipeline
            {
                let pipeline = SweepPipeline::new()
                    .then(StormRelativeVelocity::new(240.0, 20.0).map_err(process_err)?)
                    .then(GaussianSmooth {
                        sigma_azimuth: 1.5,
                        sigma_range: 1.5,
                    });

                let processed = pipeline.execute(&vf).map_err(process_err)?;
                let result = render_sweep(&processed, &vel_scale, &compare_options)?;
                let path = format!("{}/05c_velocity_sr_smoothed.png", OUTPUT_DIR);
                result.image().save(&path).map_err(io_err)?;
                println!("  Storm-relative + gaussian smooth -> {}", path);
            }
        }
    } else {
        println!("  No velocity data found in this volume");
    }

    // ========================================================================
    // Section 6: Composite Reflectivity (Cartesian Rendering)
    // ========================================================================
    println!("\n=== Section 6: Composite Reflectivity ===");

    let ref_fields = nexrad::extract_fields(&volume, Product::Reflectivity);
    println!(
        "  Extracted {} reflectivity fields across elevations",
        ref_fields.len()
    );

    if !ref_fields.is_empty() {
        let max_range = ref_fields[0].max_range_km();
        let extent = coord_sys.sweep_extent(max_range);

        println!(
            "  Extent: ({:.2}, {:.2}) to ({:.2}, {:.2})",
            extent.min.latitude,
            extent.min.longitude,
            extent.max.latitude,
            extent.max.longitude
        );

        let cref = CompositeReflectivity;

        // Full-range composite at native resolution
        let composite = cref
            .compute(
                &volume,
                &ref_fields,
                &coord_sys,
                &extent,
                (native_size, native_size),
            )
            .map_err(process_err)?;

        let result = render_cartesian(
            &composite,
            &ref_scale,
            &RenderOptions::new(native_size, native_size),
        )?;

        let path = format!("{}/06a_composite_reflectivity.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;

        if let Some((min, max)) = composite.value_range() {
            println!("  Composite value range: {:.1} to {:.1} dBZ", min, max);
        }
        println!(
            "  Composite reflectivity ({} tilts, {}px) -> {}",
            ref_fields.len(),
            native_size,
            path
        );

        // Zoomed composite (half range for more detail)
        let zoom_extent = coord_sys.sweep_extent(max_range / 2.0);
        let composite_zoom = cref
            .compute(
                &volume,
                &ref_fields,
                &coord_sys,
                &zoom_extent,
                (native_size, native_size),
            )
            .map_err(process_err)?;

        let result = render_cartesian(
            &composite_zoom,
            &ref_scale,
            &RenderOptions::new(native_size, native_size),
        )?;
        let path = format!("{}/06b_composite_zoomed.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!("  Composite reflectivity (zoomed to {:.0} km) -> {}", max_range / 2.0, path);
    }

    // ========================================================================
    // Section 7: Vertical Cross-Section (RHI-Style)
    // ========================================================================
    println!("\n=== Section 7: Vertical Cross-Section ===");

    if !ref_fields.is_empty() {
        let distinct_elevations: Vec<f32> = {
            let mut elevs: Vec<f32> = ref_fields.iter().map(|f| f.elevation_degrees()).collect();
            elevs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            elevs.dedup_by(|a, b| (*a - *b).abs() < 0.05);
            elevs
        };
        println!(
            "  {} distinct elevations: {:.1?} deg",
            distinct_elevations.len(),
            distinct_elevations
        );

        // Helper closure to build a vertical cross-section at a given azimuth
        let build_vertical =
            |target_azimuth: f32, max_range_km: f64, max_altitude_m: f64| -> VerticalField {
                let vf_width = 600;
                let vf_height = 300;

                let mut vert_field = VerticalField::new(
                    "Reflectivity RHI",
                    "dBZ",
                    (0.0, max_range_km),
                    (0.0, max_altitude_m),
                    vf_width,
                    vf_height,
                );

                let re = 6371.0 * 4.0 / 3.0; // effective earth radius in km

                for field in &ref_fields {
                    let elevation = field.elevation_degrees();
                    let elev_rad = (elevation as f64).to_radians();

                    for col in 0..vf_width {
                        let range_km = (col as f64 + 0.5) / vf_width as f64 * max_range_km;

                        // Beam height using 4/3 earth radius model
                        let altitude_km =
                            range_km * elev_rad.sin() + (range_km * range_km) / (2.0 * re);
                        let altitude_m = altitude_km * 1000.0;

                        if altitude_m < 0.0 || altitude_m > max_altitude_m {
                            continue;
                        }

                        // row 0 = top = highest altitude
                        let row = ((max_altitude_m - altitude_m) / max_altitude_m
                            * vf_height as f64) as usize;
                        if row >= vf_height {
                            continue;
                        }

                        if let Some((val, status)) =
                            field.value_at_polar(target_azimuth, range_km)
                        {
                            if status == GateStatus::Valid {
                                let (existing_val, existing_status) = vert_field.get(row, col);
                                if existing_status != GateStatus::Valid || val > existing_val {
                                    vert_field.set(row, col, val, GateStatus::Valid);
                                }
                            }
                        }
                    }
                }

                vert_field
            };

        let vert_options = RenderOptions::new(1200, 600);

        // 7a. Cross-section through the storm line (~200 deg azimuth)
        let az1 = 200.0_f32;
        let vert1 = build_vertical(az1, 230.0, 18000.0);
        let result = render_vertical(&vert1, &ref_scale, &vert_options)?;
        let path = format!("{}/07a_vertical_az{:.0}.png", OUTPUT_DIR, az1);
        result.image().save(&path).map_err(io_err)?;
        println!(
            "  Vertical cross-section at {:.0} deg, 0-230 km, 0-18 km altitude -> {}",
            az1, path
        );

        // 7b. Another azimuth for comparison
        let az2 = 315.0_f32;
        let vert2 = build_vertical(az2, 230.0, 18000.0);
        let result = render_vertical(&vert2, &ref_scale, &vert_options)?;
        let path = format!("{}/07b_vertical_az{:.0}.png", OUTPUT_DIR, az2);
        result.image().save(&path).map_err(io_err)?;
        println!(
            "  Vertical cross-section at {:.0} deg -> {}",
            az2, path
        );

        // 7c. Zoomed vertical (close-range, lower altitude for detail)
        let az3 = 200.0_f32;
        let vert3 = build_vertical(az3, 100.0, 12000.0);
        let result = render_vertical(&vert3, &ref_scale, &vert_options)?;
        let path = format!("{}/07c_vertical_az{:.0}_zoomed.png", OUTPUT_DIR, az3);
        result.image().save(&path).map_err(io_err)?;
        println!(
            "  Vertical zoomed at {:.0} deg, 0-100 km, 0-12 km altitude -> {}",
            az3, path
        );
    }

    // ========================================================================
    // Section 8: Render Metadata and Point Queries
    // ========================================================================
    println!("\n=== Section 8: Render Metadata & Point Queries ===");

    let options_with_geo =
        RenderOptions::new(compare_size, compare_size).with_coord_system(coord_sys.clone());
    let result = render_sweep(&ref_field, &ref_scale, &options_with_geo)?;

    let meta = result.metadata();
    println!("  Image: {}x{}", meta.width, meta.height);
    println!(
        "  Center pixel: ({:.1}, {:.1})",
        meta.center_pixel.0, meta.center_pixel.1
    );
    println!("  Pixels per km: {:.2}", meta.pixels_per_km);
    println!("  Max range: {:.1} km", meta.max_range_km);

    if let Some(extent) = &meta.geo_extent {
        println!(
            "  Geo extent: ({:.4}, {:.4}) to ({:.4}, {:.4})",
            extent.min.latitude,
            extent.min.longitude,
            extent.max.latitude,
            extent.max.longitude
        );
    }

    // Query center pixel
    if let Some(query) = result.query_pixel(
        &ref_field,
        compare_size as f64 / 2.0,
        compare_size as f64 / 2.0,
    ) {
        println!(
            "  Center pixel query: value={:.1}, status={:?}",
            query.value, query.status,
        );
        println!(
            "    Polar: az={:.1} deg, range={:.1} km",
            query.polar.azimuth_degrees, query.polar.range_km,
        );
    }

    // Query at a specific polar coordinate (100 km north)
    if let Some(query) = result.query_polar(&ref_field, 0.0, 100.0) {
        println!(
            "  Query at 0 deg / 100 km: value={:.1}, status={:?}",
            query.value, query.status,
        );
    }

    // Query at a geographic point (the radar site itself)
    let site_point = GeoPoint {
        latitude: coord_sys.latitude(),
        longitude: coord_sys.longitude(),
    };
    if let Some(query) = result.query_geo(&ref_field, &site_point) {
        println!(
            "  Query at radar site ({:.4}, {:.4}): value={:.1}, status={:?}",
            site_point.latitude, site_point.longitude, query.value, query.status,
        );
    }

    let path = format!("{}/08_with_geo_metadata.png", OUTPUT_DIR);
    result.image().save(&path).map_err(io_err)?;
    println!("  Rendered with geo metadata -> {}", path);

    // ========================================================================
    // Section 9: Transparent Background (for Compositing/Overlays)
    // ========================================================================
    println!("\n=== Section 9: Transparent Background ===");

    {
        let transparent_options = RenderOptions::new(native_size, native_size).transparent();
        let result = render_sweep(&ref_field, &ref_scale, &transparent_options)?;
        let path = format!("{}/09_transparent_bg.png", OUTPUT_DIR);
        result.image().save(&path).map_err(io_err)?;
        println!(
            "  Transparent background ({}px, for map overlay) -> {}",
            native_size, path
        );
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n=== Done! ===");

    let count = std::fs::read_dir(OUTPUT_DIR)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "png"))
        .count();

    println!("Generated {} images in {}/", count, OUTPUT_DIR);
    println!("\nHighlights:");
    println!(
        "  - 01_* : All 7 radar products at native resolution ({}px for REF, {}px for dual-pol)",
        native_size,
        volume
            .sweeps()
            .iter()
            .find_map(|s| nexrad::extract_field(s, Product::Velocity))
            .map(|f| f.gate_count() * 2)
            .unwrap_or(0)
    );
    println!("  - 02_* : Reflectivity at every distinct elevation angle in the volume");
    println!("  - 03_* : Individual processing effects (threshold, smooth, filter, CC noise removal)");
    println!("  - 04_* : Combined processing pipelines");
    println!("  - 05_* : Storm-relative velocity at native resolution");
    println!("  - 06_* : Composite reflectivity (cartesian projection, full + zoomed)");
    println!("  - 07_* : Vertical cross-sections (RHI-style) at multiple azimuths");
    println!("  - 08   : Render with geographic metadata and point queries");
    println!("  - 09   : Transparent background for map overlay compositing");

    Ok(())
}

fn slug(s: &str) -> String {
    s.to_lowercase().replace(' ', "_")
}

fn io_err(e: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
}

fn process_err(e: nexrad::process::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
}
