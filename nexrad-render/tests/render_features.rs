//! Tests for status colors, value ranges, geographic sweep rendering, metadata queries
//! and image output.

use nexrad_model::data::{CartesianField, GateStatus, Product, SweepField, VerticalField};
use nexrad_model::geo::{GeoExtent, GeoPoint, PolarPoint, RadarCoordinateSystem};
use nexrad_model::meta::Site;
use nexrad_render::image::ImageFormat;
use nexrad_render::{
    default_scale, render_cartesian, render_sweep, render_vertical, Color, ColorLookupTable,
    ColorScale, ColorStop, ContinuousColorScale, Error, Interpolation, Projection, RenderOptions,
};

const BACKGROUND: [u8; 4] = [1, 2, 3, 255];
const PURPLE: [u8; 4] = [128, 0, 128, 255];

/// A full-circle reflectivity sweep with 10-degree radials (centred on 0, 10, ..., 350)
/// and 10 km gates from 0 to 300 km, every gate valid at 20 dBZ.
fn coarse_sweep() -> SweepField {
    let mut field = SweepField::builder("Reflectivity", "dBZ")
        .product(Product::Reflectivity)
        .elevation_angle_degrees(0.5)
        .azimuth_angles_degrees((0..36).map(|i| i as f32 * 10.0).collect())
        .azimuth_spacing_degrees(10.0)
        .first_gate_range_km(0.0)
        .gate_interval_km(10.0)
        .gate_count(30)
        .build()
        .expect("valid geometry");
    for az in 0..field.azimuth_count() {
        for gate in 0..field.gate_count() {
            field.set(az, gate, 20.0, GateStatus::Valid).unwrap();
        }
    }
    field
}

fn coord_system() -> RadarCoordinateSystem {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .build()
        .expect("valid site");
    RadarCoordinateSystem::new(&site)
}

/// The color a reflectivity render paints `value` with: the scale sampled through the
/// lookup table built over the reflectivity product range.
fn reflectivity_color(scale: &ColorScale, value: f32) -> [u8; 4] {
    let (min, max) = Product::Reflectivity.value_range();
    ColorLookupTable::new(scale, min, max, 256)
        .expect("valid range")
        .color(value)
}

fn pixel(image: &nexrad_render::RgbaImage, x: f64, y: f64) -> [u8; 4] {
    image.get_pixel(x as u32, y as u32).0
}

#[test]
fn status_colors_paint_non_valid_gates() {
    let mut field = coarse_sweep();
    // Radial at 90 degrees, gates 10..20 (100-200 km) are range folded.
    for gate in 10..20 {
        field.set(9, gate, 0.0, GateStatus::RangeFolded).unwrap();
    }
    let scale = default_scale(Product::Reflectivity);
    let base = RenderOptions::new(200, 200).with_background(BACKGROUND.into());
    let options = base
        .clone()
        .with_status_color(GateStatus::RangeFolded, Color::from_rgba8(PURPLE));

    let plain = render_sweep(&field, &scale, &base).expect("renders");
    let folded = render_sweep(&field, &scale, &options).expect("renders");

    let meta = folded.metadata();
    let (x, y) = meta.polar_to_pixel(90.0, 150.0).expect("polar image");
    assert_eq!(pixel(plain.image(), x, y), BACKGROUND);
    assert_eq!(pixel(folded.image(), x, y), PURPLE);

    // Valid gates are unaffected by the status color.
    let valid = reflectivity_color(&scale, 20.0);
    let (x, y) = meta.polar_to_pixel(270.0, 150.0).expect("polar image");
    assert_eq!(pixel(plain.image(), x, y), valid);
    assert_eq!(pixel(folded.image(), x, y), valid);

    // Valid gates also keep their color with bilinear interpolation.
    let bilinear = options.with_interpolation(Interpolation::Bilinear);
    let smooth = render_sweep(&field, &scale, &bilinear).expect("renders");
    assert_eq!(pixel(smooth.image(), x, y), valid);
    let (x, y) = meta.polar_to_pixel(90.0, 150.0).expect("polar image");
    assert_eq!(pixel(smooth.image(), x, y), PURPLE);
}

#[test]
fn status_colors_apply_to_grids() {
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
    let mut cartesian = CartesianField::new("Test", "dBZ", extent, 2, 2).unwrap();
    cartesian.set(0, 0, 0.0, GateStatus::RangeFolded).unwrap();
    let scale = default_scale(Product::Reflectivity);
    let options = RenderOptions::new(4, 4)
        .with_background(BACKGROUND.into())
        .with_status_color(GateStatus::RangeFolded, Color::from_rgba8(PURPLE));

    let result = render_cartesian(&cartesian, &scale, &options).expect("renders");
    assert_eq!(result.image().get_pixel(0, 0).0, PURPLE);
    // NoData has no status color, so it stays background.
    assert_eq!(result.image().get_pixel(3, 3).0, BACKGROUND);
}

/// A continuous gradient so the lookup-table range is visible in the output.
fn gradient() -> ColorScale {
    ContinuousColorScale::new(vec![
        ColorStop::new(-32.0, Color::BLACK),
        ColorStop::new(150.0, Color::WHITE),
    ])
    .unwrap()
    .into()
}

#[test]
fn value_range_defaults_to_the_product_range() {
    let mut field = coarse_sweep();
    // Field values span 100..120, above the top of the reflectivity range.
    for az in 0..field.azimuth_count() {
        for gate in 0..field.gate_count() {
            let value = if az % 2 == 0 { 100.0 } else { 120.0 };
            field.set(az, gate, value, GateStatus::Valid).unwrap();
        }
    }
    let scale = gradient();
    let options = RenderOptions::new(100, 100);
    let (product_min, product_max) = Product::Reflectivity.value_range();

    let default = render_sweep(&field, &scale, &options).expect("renders");
    let product = render_sweep(
        &field,
        &scale,
        &options.clone().with_value_range(product_min, product_max),
    )
    .expect("renders");
    let field_range =
        render_sweep(&field, &scale, &options.with_value_range(100.0, 120.0)).expect("renders");

    assert_eq!(default.image(), product.image());
    assert_ne!(default.image(), field_range.image());
}

#[test]
fn invalid_value_range_is_an_error() {
    let options = RenderOptions::new(100, 100).with_value_range(10.0, 10.0);
    assert!(matches!(
        render_sweep(&coarse_sweep(), &gradient(), &options),
        Err(Error::InvalidRange { .. })
    ));
}

fn sweep_extent() -> GeoExtent {
    coord_system().sweep_extent(300.0)
}

#[test]
fn sweep_renders_into_a_geographic_extent() {
    let mut field = coarse_sweep();
    // One distinctive gate: radial 90 degrees, 100-110 km.
    field.set(9, 10, 60.0, GateStatus::Valid).unwrap();
    let scale = default_scale(Product::Reflectivity);
    let extent = sweep_extent();
    let options = RenderOptions::new(512, 512)
        .with_background(BACKGROUND.into())
        .with_extent(extent)
        .with_coord_system(coord_system());

    let result = render_sweep(&field, &scale, &options).expect("renders");
    let meta = result.metadata();
    let Projection::Geographic {
        extent: projected, ..
    } = meta.projection()
    else {
        panic!(
            "expected a geographic projection, got {:?}",
            meta.projection()
        );
    };
    assert_eq!(projected, &extent);
    assert_eq!(meta.geo_extent(), Some(&extent));
    assert!(meta.coord_system().is_some());

    let gate = coord_system().polar_to_geo(PolarPoint {
        azimuth_angle_degrees: 90.0,
        range_km: 105.0,
        elevation_angle_degrees: 0.5,
    });
    let gate = GeoPoint {
        latitude: gate.latitude,
        longitude: gate.longitude,
    };
    let (x, y) = meta.geo_to_pixel(&gate).expect("geographic image");
    assert_eq!(
        pixel(result.image(), x, y),
        reflectivity_color(&scale, 60.0)
    );

    // The opposite gate keeps the sweep's base value, and the corners are outside range.
    let (x2, y2) = meta
        .geo_to_pixel(&{
            let p = coord_system().polar_to_geo(PolarPoint {
                azimuth_angle_degrees: 270.0,
                range_km: 105.0,
                elevation_angle_degrees: 0.5,
            });
            GeoPoint {
                latitude: p.latitude,
                longitude: p.longitude,
            }
        })
        .expect("geographic image");
    assert_eq!(
        pixel(result.image(), x2, y2),
        reflectivity_color(&scale, 20.0)
    );
    assert_eq!(result.image().get_pixel(0, 0).0, BACKGROUND);

    let query = meta.query_geo(&field, &gate).expect("inside the field");
    assert_eq!((query.value, query.status), (60.0, GateStatus::Valid));
    assert!((query.polar.range_km - 105.0).abs() < 1e-6);
    assert!((query.polar.azimuth_angle_degrees - 90.0).abs() < 1e-6);

    let query = meta.query_pixel(&field, x, y).expect("inside the field");
    assert_eq!((query.value, query.status), (60.0, GateStatus::Valid));

    let query = meta
        .query_polar(&field, 90.0, 105.0)
        .expect("inside the field");
    assert_eq!(query.value, 60.0);
}

#[test]
fn geographic_sweep_render_supports_bilinear() {
    let scale = default_scale(Product::Reflectivity);
    let options = RenderOptions::new(128, 128)
        .with_extent(sweep_extent())
        .with_coord_system(coord_system())
        .with_interpolation(Interpolation::Bilinear);
    let result = render_sweep(&coarse_sweep(), &scale, &options).expect("renders");
    let (x, y) = result
        .metadata()
        .geo_to_pixel(&GeoPoint {
            latitude: 35.3331,
            longitude: -96.5,
        })
        .expect("geographic image");
    assert_eq!(
        pixel(result.image(), x, y),
        reflectivity_color(&scale, 20.0)
    );
}

#[test]
fn full_range_sweep_renders_into_an_extent() {
    // 720 radials x 1840 gates of 250 m: a full 460 km super-resolution sweep.
    let mut field = SweepField::builder("Reflectivity", "dBZ")
        .product(Product::Reflectivity)
        .elevation_angle_degrees(0.5)
        .azimuth_angles_degrees((0..720).map(|i| i as f32 * 0.5).collect())
        .azimuth_spacing_degrees(0.5)
        .first_gate_range_km(0.0)
        .gate_interval_km(0.25)
        .gate_count(1840)
        .build()
        .expect("valid geometry");
    for az in 0..field.azimuth_count() {
        for gate in 0..field.gate_count() {
            field.set(az, gate, 30.0, GateStatus::Valid).unwrap();
        }
    }
    let options = RenderOptions::new(512, 512)
        .with_extent(coord_system().sweep_extent(460.0))
        .with_coord_system(coord_system());
    let scale = default_scale(Product::Reflectivity);
    let result = render_sweep(&field, &scale, &options).expect("renders");
    assert_eq!(
        result.image().get_pixel(256, 256).0,
        reflectivity_color(&scale, 30.0)
    );
}

#[test]
fn geographic_sweep_render_needs_a_coordinate_system() {
    let scale = default_scale(Product::Reflectivity);
    let options = RenderOptions::new(64, 64).with_extent(sweep_extent());
    assert!(matches!(
        render_sweep(&coarse_sweep(), &scale, &options),
        Err(Error::MissingCoordinateSystem)
    ));

    let mut reversed = sweep_extent();
    std::mem::swap(&mut reversed.min, &mut reversed.max);
    let options = options
        .with_extent(reversed)
        .with_coord_system(coord_system());
    assert!(matches!(
        render_sweep(&coarse_sweep(), &scale, &options),
        Err(Error::InvalidExtent)
    ));
}

#[test]
fn cartesian_and_vertical_ignore_the_extent_option() {
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
    let cartesian = CartesianField::new("Test", "dBZ", extent, 2, 2).unwrap();
    let scale = default_scale(Product::Reflectivity);
    let options = RenderOptions::new(4, 4).with_extent(sweep_extent());
    let result = render_cartesian(&cartesian, &scale, &options).expect("renders");
    assert_eq!(result.metadata().geo_extent(), Some(&extent));
}

#[test]
fn query_cartesian_reads_the_rendered_cell() {
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
    let mut field = CartesianField::new("Test", "dBZ", extent, 4, 2).unwrap();
    field.set(1, 3, 42.0, GateStatus::Valid).unwrap();
    let scale = default_scale(Product::Reflectivity);
    let result = render_cartesian(&field, &scale, &RenderOptions::new(40, 20)).expect("renders");
    let meta = result.metadata();

    assert_eq!(
        meta.query_cartesian(&field, 35.0, 15.0),
        Some((42.0, GateStatus::Valid))
    );
    assert_eq!(
        meta.query_cartesian(&field, 5.0, 5.0),
        Some((0.0, GateStatus::NoData))
    );
    assert_eq!(meta.query_cartesian(&field, 41.0, 5.0), None);
    assert_eq!(
        pixel(result.image(), 35.0, 15.0),
        reflectivity_color(&scale, 42.0)
    );
}

#[test]
fn query_vertical_reads_the_rendered_cell() {
    let mut field =
        VerticalField::new("Test", "dBZ", (0.0, 100.0), (0.0, 10_000.0), 10, 5).unwrap();
    // Row 0 is the top (8-10 km), column 9 is the far end (90-100 km).
    field.set(0, 9, 55.0, GateStatus::Valid).unwrap();
    let scale = default_scale(Product::Reflectivity);
    let result = render_vertical(&field, &scale, &RenderOptions::new(100, 50)).expect("renders");
    let meta = result.metadata();

    let (x, y) = meta
        .vertical_to_pixel(95.0, 9_000.0)
        .expect("vertical image");
    assert_eq!(
        meta.query_vertical(&field, x, y),
        Some((55.0, GateStatus::Valid))
    );
    assert_eq!(
        pixel(result.image(), x, y),
        reflectivity_color(&scale, 55.0)
    );
    assert_eq!(
        meta.query_vertical(&field, 5.0, 45.0),
        Some((0.0, GateStatus::NoData))
    );
    assert_eq!(meta.query_vertical(&field, 101.0, 5.0), None);
}

#[test]
fn write_to_encodes_png() {
    let scale = default_scale(Product::Reflectivity);
    let result =
        render_sweep(&coarse_sweep(), &scale, &RenderOptions::new(32, 32)).expect("renders");

    let mut png = Vec::new();
    result
        .write_to(&mut png, ImageFormat::Png)
        .expect("encodes");
    assert!(png.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']));

    let decoded = nexrad_render::image::load_from_memory_with_format(&png, ImageFormat::Png)
        .expect("decodes")
        .to_rgba8();
    assert_eq!(&decoded, result.image());
}
