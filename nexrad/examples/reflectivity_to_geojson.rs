//! Convert composite reflectivity to GeoJSON — one Polygon file (rectangles
//! per grid cell) and one Point file (cell centers), both filtered by a dBZ
//! threshold.
//!
//! Run with:
//! ```bash
//! cargo run --example reflectivity_to_geojson --features "process,data" -- \
//!     tests/fixtures/convective/KDMX20220305_232324.bin 35.0 reflectivity
//! ```

use geojson::{Feature, FeatureCollection, Geometry, JsonObject, JsonValue, Value};
use nexrad::model::data::{GateStatus, Product};
use nexrad::process::derived::CompositeReflectivity;
use nexrad::process::ScanDerivedProduct;
use std::env;

const OUTPUT_RESOLUTION: (usize, usize) = (800, 800);

fn main() -> nexrad::Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("tests/fixtures/convective/KDMX20220305_232324.bin");
    let threshold: f32 = args
        .get(2)
        .map(|s| s.parse().expect("threshold_dbz must be a number"))
        .unwrap_or(35.0);
    let prefix = args.get(3).map(String::as_str).unwrap_or("reflectivity");

    println!("Loading scan from: {}", path);
    let scan = nexrad::load_file(path)?;
    let site_id = scan
        .site()
        .map(|s| s.identifier_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("Site: {} (VCP {})", site_id, scan.coverage_pattern_number());

    let ref_fields = nexrad::extract_fields(&scan, Product::Reflectivity);
    println!("Reflectivity tilts: {}", ref_fields.len());

    let coord_sys = nexrad::coordinate_system_required(&scan)?;
    let extent = coord_sys.sweep_extent(ref_fields[0].max_range_km());

    println!(
        "Computing composite reflectivity at {}x{}...",
        OUTPUT_RESOLUTION.0, OUTPUT_RESOLUTION.1
    );
    let composite =
        CompositeReflectivity.compute(&scan, &ref_fields, &coord_sys, &extent, OUTPUT_RESOLUTION)?;

    let extent = *composite.extent();
    let width = composite.width();
    let height = composite.height();
    let lat_range = extent.max.latitude - extent.min.latitude;
    let lon_range = extent.max.longitude - extent.min.longitude;

    let mut polygon_features: Vec<Feature> = Vec::new();
    let mut point_features: Vec<Feature> = Vec::new();
    let mut considered = 0usize;
    let mut emitted = 0usize;

    for row in 0..height {
        let lat_max = extent.max.latitude - row as f64 / height as f64 * lat_range;
        let lat_min = extent.max.latitude - (row + 1) as f64 / height as f64 * lat_range;
        for col in 0..width {
            considered += 1;
            let (value, status) = composite.get(row, col);
            if status != GateStatus::Valid || value < threshold {
                continue;
            }
            emitted += 1;

            let lon_min = extent.min.longitude + col as f64 / width as f64 * lon_range;
            let lon_max = extent.min.longitude + (col + 1) as f64 / width as f64 * lon_range;

            let ring: Vec<Vec<f64>> = vec![
                vec![lon_min, lat_min],
                vec![lon_max, lat_min],
                vec![lon_max, lat_max],
                vec![lon_min, lat_max],
                vec![lon_min, lat_min],
            ];

            let mut props = JsonObject::new();
            props.insert("dbz".to_string(), JsonValue::from(value));

            polygon_features.push(Feature {
                bbox: None,
                geometry: Some(Geometry::new(Value::Polygon(vec![ring]))),
                id: None,
                properties: Some(props.clone()),
                foreign_members: None,
            });

            let center_lon = (lon_min + lon_max) / 2.0;
            let center_lat = (lat_min + lat_max) / 2.0;
            point_features.push(Feature {
                bbox: None,
                geometry: Some(Geometry::new(Value::Point(vec![center_lon, center_lat]))),
                id: None,
                properties: Some(props),
                foreign_members: None,
            });
        }
    }

    let bbox = vec![
        extent.min.longitude,
        extent.min.latitude,
        extent.max.longitude,
        extent.max.latitude,
    ];
    let mut foreign = JsonObject::new();
    foreign.insert("site_id".to_string(), JsonValue::from(site_id.clone()));
    foreign.insert("threshold_dbz".to_string(), JsonValue::from(threshold));

    let polygons_path = format!("{}_polygons.geojson", prefix);
    let points_path = format!("{}_points.geojson", prefix);

    std::fs::write(
        &polygons_path,
        FeatureCollection {
            bbox: Some(bbox.clone()),
            features: polygon_features,
            foreign_members: Some(foreign.clone()),
        }
        .to_string(),
    )?;
    std::fs::write(
        &points_path,
        FeatureCollection {
            bbox: Some(bbox),
            features: point_features,
            foreign_members: Some(foreign),
        }
        .to_string(),
    )?;

    println!();
    println!("Site:          {}", site_id);
    println!("Tilts used:    {}", ref_fields.len());
    println!(
        "Cells:         {} considered, {} emitted (threshold {:.1} dBZ)",
        considered, emitted, threshold
    );
    println!("Wrote:");
    println!("  {}", polygons_path);
    println!("  {}", points_path);

    Ok(())
}
