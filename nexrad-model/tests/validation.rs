//! Tests for constructor validation, checked field indexing, and the field extraction fixes.

use nexrad_model::data::{
    CartesianField, GateStatus, MomentData, MomentValue, Product, Radial, Sweep, SweepField,
    VerticalField,
};
use nexrad_model::geo::{GeoExtent, GeoPoint};
use nexrad_model::meta::Site;
use nexrad_model::result::Error;

fn extent(min: (f64, f64), max: (f64, f64)) -> GeoExtent {
    GeoExtent {
        min: GeoPoint {
            latitude: min.0,
            longitude: min.1,
        },
        max: GeoPoint {
            latitude: max.0,
            longitude: max.1,
        },
    }
}

fn valid_extent() -> GeoExtent {
    extent((34.0, -99.0), (36.0, -97.0))
}

fn geometry_parameter(result: Result<impl std::fmt::Debug, Error>) -> &'static str {
    match result {
        Err(Error::InvalidGeometry { parameter, .. }) => parameter,
        other => panic!("expected InvalidGeometry, got {other:?}"),
    }
}

fn moment_parameter(result: Result<MomentData, Error>) -> (&'static str, &'static str) {
    match result {
        Err(Error::InvalidMomentData { parameter, reason }) => (parameter, reason),
        other => panic!("expected InvalidMomentData, got {other:?}"),
    }
}

fn moment_length(result: Result<MomentData, Error>) -> (usize, usize) {
    match result {
        Err(Error::MomentDataLength { expected, actual }) => (expected, actual),
        other => panic!("expected MomentDataLength, got {other:?}"),
    }
}

fn site_parameter(result: Result<Site, Error>) -> &'static str {
    match result {
        Err(Error::InvalidSite { parameter, .. }) => parameter,
        other => panic!("expected InvalidSite, got {other:?}"),
    }
}

fn sweep_builder() -> nexrad_model::data::SweepFieldBuilder {
    SweepField::builder("Reflectivity", "dBZ")
        .elevation_angle_degrees(0.5)
        .azimuth_angles_degrees(vec![0.0, 1.0, 2.0])
        .first_gate_range_km(2.0)
        .gate_interval_km(0.25)
        .gate_count(4)
}

fn reflectivity(first_gate_range_meters: u16, raw: Vec<u8>) -> MomentData {
    MomentData::from_fixed_point(
        Product::Reflectivity,
        raw.len(),
        first_gate_range_meters,
        250,
        8,
        2.0,
        66.0,
        raw,
    )
    .unwrap()
}

// --- Checked indexing ---

#[test]
fn sweep_field_get_checks_each_dimension() {
    let field = sweep_builder().build().unwrap();

    assert_eq!(field.get(2, 3), Some((0.0, GateStatus::NoData)));
    // Column past the gate count must not wrap into the next row.
    assert_eq!(field.get(0, 4 + 3), None);
    assert_eq!(field.get(3, 0), None);
    assert_eq!(field.get(usize::MAX, usize::MAX), None);
}

#[test]
fn sweep_field_set_reports_out_of_bounds() {
    let mut field = sweep_builder().build().unwrap();

    field.set(1, 2, 42.0, GateStatus::Valid).unwrap();
    assert_eq!(field.get(1, 2), Some((42.0, GateStatus::Valid)));

    assert_eq!(
        field.set(0, 4, 1.0, GateStatus::Valid),
        Err(Error::IndexOutOfBounds {
            row: 0,
            column: 4,
            rows: 3,
            columns: 4,
        })
    );
    assert!(matches!(
        field.set(3, 0, 1.0, GateStatus::Valid),
        Err(Error::IndexOutOfBounds { row: 3, .. })
    ));
    // A rejected write leaves the field untouched.
    assert_eq!(field.get(1, 0), Some((0.0, GateStatus::NoData)));
}

#[test]
fn sweep_field_row_accessors_are_checked() {
    let mut field = sweep_builder().build().unwrap();

    field.radial_values_mut(2).unwrap()[1] = 7.0;
    field.radial_statuses_mut(2).unwrap()[1] = GateStatus::Valid;
    assert_eq!(field.radial_values(2), Some(&[0.0, 7.0, 0.0, 0.0][..]));
    assert_eq!(
        field.radial_statuses(2).map(|row| row[1]),
        Some(GateStatus::Valid)
    );
    assert_eq!(field.get(2, 1), Some((7.0, GateStatus::Valid)));

    assert!(field.radial_values(3).is_none());
    assert!(field.radial_statuses(3).is_none());
    assert!(field.radial_values_mut(3).is_none());
    assert!(field.radial_statuses_mut(usize::MAX).is_none());
}

#[test]
fn cartesian_field_indexing_is_checked() {
    let mut field = CartesianField::new("Test", "dBZ", valid_extent(), 5, 2).unwrap();

    field.set(1, 4, 3.0, GateStatus::Valid).unwrap();
    assert_eq!(field.get(1, 4), Some((3.0, GateStatus::Valid)));
    assert_eq!(field.get(0, 5 + 3), None);
    assert_eq!(field.get(2, 0), None);
    assert_eq!(
        field.set(0, 5, 1.0, GateStatus::Valid),
        Err(Error::IndexOutOfBounds {
            row: 0,
            column: 5,
            rows: 2,
            columns: 5,
        })
    );
}

#[test]
fn vertical_field_indexing_is_checked() {
    let mut field = VerticalField::new("Test", "dBZ", (0.0, 100.0), (0.0, 10_000.0), 5, 2).unwrap();

    field.set(1, 4, 3.0, GateStatus::Valid).unwrap();
    assert_eq!(field.get(1, 4), Some((3.0, GateStatus::Valid)));
    assert_eq!(field.get(0, 5 + 3), None);
    assert_eq!(field.get(2, 0), None);
    assert!(matches!(
        field.set(2, 0, 1.0, GateStatus::Valid),
        Err(Error::IndexOutOfBounds {
            row: 2,
            rows: 2,
            ..
        })
    ));
}

#[test]
fn grid_edge_queries_stay_in_bounds() {
    let mut cartesian = CartesianField::new("Test", "dBZ", valid_extent(), 2, 2).unwrap();
    cartesian.set(1, 1, 9.0, GateStatus::Valid).unwrap();
    let south_east = GeoPoint {
        latitude: 34.0,
        longitude: -97.0,
    };
    assert_eq!(
        cartesian.value_at_geo(south_east),
        Some((9.0, GateStatus::Valid))
    );
    assert!(cartesian
        .value_at_geo(GeoPoint {
            latitude: f64::NAN,
            longitude: -98.0,
        })
        .is_none());

    let mut vertical =
        VerticalField::new("Test", "dBZ", (0.0, 100.0), (0.0, 10_000.0), 2, 2).unwrap();
    vertical.set(1, 1, 9.0, GateStatus::Valid).unwrap();
    assert_eq!(
        vertical.value_at(100.0, 0.0),
        Some((9.0, GateStatus::Valid))
    );
    assert!(vertical.value_at(f64::NAN, 0.0).is_none());
}

// --- CartesianField::new ---

#[test]
fn cartesian_field_rejects_invalid_dimensions() {
    let new = |width, height| CartesianField::new("Test", "dBZ", valid_extent(), width, height);

    assert_eq!(geometry_parameter(new(0, 4)), "width");
    assert_eq!(geometry_parameter(new(4, 0)), "height");
    assert_eq!(geometry_parameter(new(usize::MAX, 2)), "width");
    assert!(new(1, 1).is_ok());
}

#[test]
fn cartesian_field_rejects_invalid_extents() {
    let new = |extent| CartesianField::new("Test", "dBZ", extent, 4, 4);

    // min must be strictly less than max
    assert_eq!(
        geometry_parameter(new(extent((36.0, -99.0), (34.0, -97.0)))),
        "extent.latitude"
    );
    assert_eq!(
        geometry_parameter(new(extent((34.0, -97.0), (34.0, -97.0)))),
        "extent.latitude"
    );
    assert_eq!(
        geometry_parameter(new(extent((34.0, -97.0), (36.0, -99.0)))),
        "extent.longitude"
    );
    // finite and in range
    assert_eq!(
        geometry_parameter(new(extent((f64::NAN, -99.0), (36.0, -97.0)))),
        "extent.latitude"
    );
    assert_eq!(
        geometry_parameter(new(extent((34.0, -99.0), (34.5, f64::INFINITY)))),
        "extent.longitude"
    );
    assert_eq!(
        geometry_parameter(new(extent((80.0, -99.0), (90.5, -97.0)))),
        "extent.latitude"
    );
    assert_eq!(
        geometry_parameter(new(extent((34.0, -180.5), (36.0, -97.0)))),
        "extent.longitude"
    );
    assert!(new(extent((-90.0, -180.0), (90.0, 180.0))).is_ok());
}

// --- VerticalField::new ---

#[test]
fn vertical_field_rejects_invalid_geometry() {
    let new = |distance, altitude, width, height| {
        VerticalField::new("Test", "dBZ", distance, altitude, width, height)
    };

    assert_eq!(
        geometry_parameter(new((0.0, 100.0), (0.0, 1.0), 0, 4)),
        "width"
    );
    assert_eq!(
        geometry_parameter(new((0.0, 100.0), (0.0, 1.0), 4, 0)),
        "height"
    );
    assert_eq!(
        geometry_parameter(new((0.0, 100.0), (0.0, 1.0), 2, usize::MAX)),
        "width"
    );
    assert_eq!(
        geometry_parameter(new((100.0, 100.0), (0.0, 1.0), 4, 4)),
        "distance_range_km"
    );
    assert_eq!(
        geometry_parameter(new((0.0, f64::NAN), (0.0, 1.0), 4, 4)),
        "distance_range_km"
    );
    assert_eq!(
        geometry_parameter(new((0.0, 100.0), (5.0, -5.0), 4, 4)),
        "altitude_range_meters"
    );
    assert_eq!(
        geometry_parameter(new((0.0, 100.0), (f64::NEG_INFINITY, 0.0), 4, 4)),
        "altitude_range_meters"
    );
}

// --- SweepFieldBuilder::build ---

#[test]
fn sweep_field_builder_rejects_invalid_azimuths() {
    let azimuths = |angles: Vec<f32>| sweep_builder().azimuth_angles_degrees(angles).build();

    assert_eq!(
        geometry_parameter(azimuths(vec![])),
        "azimuth_angles_degrees"
    );
    assert_eq!(
        geometry_parameter(azimuths(vec![0.0, f32::NAN])),
        "azimuth_angles_degrees"
    );
    assert_eq!(
        geometry_parameter(azimuths(vec![-0.5, 1.0])),
        "azimuth_angles_degrees"
    );
    assert_eq!(
        geometry_parameter(azimuths(vec![10.0, 360.0])),
        "azimuth_angles_degrees"
    );
    // Strictly ascending: no descending or duplicate azimuths
    assert_eq!(
        geometry_parameter(azimuths(vec![2.0, 1.0])),
        "azimuth_angles_degrees"
    );
    assert_eq!(
        geometry_parameter(azimuths(vec![1.0, 1.0])),
        "azimuth_angles_degrees"
    );
    assert!(azimuths(vec![0.0, 359.5]).is_ok());
}

#[test]
fn sweep_field_builder_rejects_invalid_ranges_and_angles() {
    let build_parameter =
        |builder: nexrad_model::data::SweepFieldBuilder| geometry_parameter(builder.build());

    assert_eq!(
        build_parameter(sweep_builder().first_gate_range_km(-0.1)),
        "first_gate_range_km"
    );
    assert_eq!(
        build_parameter(sweep_builder().first_gate_range_km(f64::NAN)),
        "first_gate_range_km"
    );
    assert_eq!(
        build_parameter(sweep_builder().gate_interval_km(0.0)),
        "gate_interval_km"
    );
    assert_eq!(
        build_parameter(sweep_builder().gate_interval_km(f64::INFINITY)),
        "gate_interval_km"
    );
    assert_eq!(
        build_parameter(sweep_builder().elevation_angle_degrees(f32::NAN)),
        "elevation_angle_degrees"
    );
    assert_eq!(
        build_parameter(sweep_builder().azimuth_spacing_degrees(0.0)),
        "azimuth_spacing_degrees"
    );
    assert_eq!(
        build_parameter(sweep_builder().azimuth_spacing_degrees(f32::NAN)),
        "azimuth_spacing_degrees"
    );
    assert_eq!(build_parameter(sweep_builder().gate_count(0)), "gate_count");
    assert_eq!(
        build_parameter(sweep_builder().gate_count(usize::MAX)),
        "gate_count"
    );
    assert!(sweep_builder().first_gate_range_km(0.0).build().is_ok());
}

#[test]
fn new_like_copies_validated_geometry() {
    let field = sweep_builder().product(Product::Velocity).build().unwrap();
    let like = field.new_like("Dealiased Velocity", "m/s");

    assert_eq!(
        like.azimuth_angles_degrees(),
        field.azimuth_angles_degrees()
    );
    assert_eq!(like.gate_count(), field.gate_count());
    assert_eq!(like.product(), Some(Product::Velocity));
    assert_eq!(like.get(2, 3), Some((0.0, GateStatus::NoData)));
    assert_eq!(like.get(3, 0), None);
}

// --- MomentData::from_fixed_point ---

#[test]
fn moment_data_rejects_invalid_word_size() {
    let build = |word_size| {
        MomentData::from_fixed_point(
            Product::Reflectivity,
            2,
            2125,
            250,
            word_size,
            2.0,
            66.0,
            vec![10, 20],
        )
    };

    assert_eq!(
        moment_parameter(build(4)),
        ("data_word_size", "must be 8 or 16")
    );
    assert_eq!(
        moment_parameter(build(32)),
        ("data_word_size", "must be 8 or 16")
    );
    assert!(build(8).is_ok());
}

#[test]
fn moment_data_rejects_mismatched_value_length() {
    let build = |gate_count, word_size, values| {
        MomentData::from_fixed_point(
            Product::DifferentialReflectivity,
            gate_count,
            2125,
            250,
            word_size,
            16.0,
            128.0,
            values,
        )
    };

    assert_eq!(moment_length(build(3, 8, vec![1, 2])), (3, 2));
    assert_eq!(moment_length(build(2, 16, vec![1, 2, 3])), (4, 3));
    assert_eq!(moment_length(build(1, 16, vec![1])), (2, 1));
    assert_eq!(
        moment_parameter(build(70_000, 8, vec![])),
        ("gate_count", "must fit in a u16")
    );
    assert_eq!(
        build(3, 8, vec![1, 2]).unwrap_err().to_string(),
        "invalid moment data length: expected 3 bytes for the gate count and word size, got 2"
    );
    assert!(build(2, 16, vec![0, 2, 1, 0]).is_ok());
    assert!(build(0, 8, vec![]).is_ok());
}

#[test]
fn moment_data_rejects_non_finite_scale_and_offset() {
    let build = |scale, offset| {
        MomentData::from_fixed_point(
            Product::Reflectivity,
            1,
            2125,
            250,
            8,
            scale,
            offset,
            vec![10],
        )
    };

    assert_eq!(
        moment_parameter(build(f32::NAN, 66.0)),
        ("scale", "must be finite")
    );
    assert_eq!(
        moment_parameter(build(f32::INFINITY, 66.0)),
        ("scale", "must be finite")
    );
    assert_eq!(
        moment_parameter(build(2.0, f32::NEG_INFINITY)),
        ("offset", "must be finite")
    );
    assert_eq!(
        build(f32::NAN, 66.0).unwrap_err().to_string(),
        "invalid moment data: `scale` must be finite"
    );
    assert!(build(0.0, 0.0).is_ok());
}

#[test]
fn moment_data_exposes_constructor_units() {
    let moment = MomentData::from_fixed_point(
        Product::Velocity,
        3,
        2125,
        250,
        8,
        2.0,
        129.0,
        vec![2, 3, 4],
    )
    .unwrap();

    assert_eq!(moment.gate_count(), 3usize);
    assert_eq!(moment.first_gate_range_meters(), 2125);
    assert_eq!(moment.gate_interval_meters(), 250);
    assert_eq!(moment.first_gate_range_km(), 2.125);
    assert_eq!(moment.gate_interval_km(), 0.25);
}

#[test]
fn unscaled_moment_decodes_sentinels_before_raw_values() {
    // 16-bit, scale 0: raw 0 and 1 are sentinels, not the values 0.0 and 1.0.
    let moment = MomentData::from_fixed_point(
        Product::DifferentialPhase,
        3,
        2125,
        250,
        16,
        0.0,
        0.0,
        vec![0, 0, 0, 1, 0x01, 0x00],
    )
    .unwrap();
    assert_eq!(
        moment.values(),
        vec![
            MomentValue::BelowThreshold,
            MomentValue::RangeFolded,
            MomentValue::Value(256.0),
        ]
    );

    // Clutter filter power keeps its status codes when unscaled.
    let cfp = MomentData::from_fixed_point(
        Product::ClutterFilterPower,
        2,
        2125,
        250,
        8,
        0.0,
        0.0,
        vec![1, 9],
    )
    .unwrap();
    assert!(matches!(cfp.values()[0], MomentValue::CfpStatus(_)));
    assert_eq!(cfp.values()[1], MomentValue::Value(9.0));

    // The sweep field sees the same sentinels.
    let radial = Radial::builder(0, 0.0, 1, 0.5)
        .moment(moment)
        .build()
        .unwrap();
    let field = SweepField::from_radials(&[radial], Product::DifferentialPhase).unwrap();
    assert_eq!(field.get(0, 0), Some((0.0, GateStatus::BelowThreshold)));
    assert_eq!(field.get(0, 1), Some((0.0, GateStatus::RangeFolded)));
    assert_eq!(field.get(0, 2), Some((256.0, GateStatus::Valid)));
}

// --- RadialBuilder::build ---

#[test]
fn radial_builder_rejects_invalid_angles() {
    let parameter = |azimuth: f32, elevation: f32, spacing: f32| {
        geometry_parameter(
            Radial::builder(0, azimuth, 1, elevation)
                .azimuth_spacing_degrees(spacing)
                .build(),
        )
    };

    assert_eq!(parameter(360.0, 0.5, 1.0), "azimuth_angle_degrees");
    assert_eq!(parameter(-0.1, 0.5, 1.0), "azimuth_angle_degrees");
    assert_eq!(parameter(f32::NAN, 0.5, 1.0), "azimuth_angle_degrees");
    assert_eq!(
        parameter(10.0, f32::INFINITY, 1.0),
        "elevation_angle_degrees"
    );
    assert_eq!(parameter(10.0, 0.5, 0.0), "azimuth_spacing_degrees");
    assert_eq!(parameter(10.0, 0.5, f32::NAN), "azimuth_spacing_degrees");

    // Negative elevation angles are valid (e.g. below-horizon scans).
    assert!(Radial::builder(0, 359.99, 1, -0.2).build().is_ok());
}

// --- Sweep::new and Sweep::merge ---

#[test]
fn sweep_new_rejects_radials_from_other_elevations() {
    let radial = |elevation_number| {
        Radial::builder(0, 0.0, elevation_number, 0.5)
            .build()
            .unwrap()
    };

    assert_eq!(
        Sweep::new(2, vec![radial(2), radial(3)]),
        Err(Error::ElevationMismatch {
            expected: 2,
            actual: 3,
        })
    );
    assert_eq!(Sweep::new(2, vec![radial(2)]).unwrap().radials().len(), 1);

    // Splitting by elevation upholds the invariant without an error path.
    let sweeps = Sweep::from_radials(vec![radial(1), radial(1), radial(2)]);
    assert_eq!(sweeps.len(), 2);
    assert_eq!(sweeps[1].elevation_number(), 2);
}

#[test]
fn sweep_merge_reports_both_elevation_numbers() {
    let merged = Sweep::new(3, vec![])
        .unwrap()
        .merge(Sweep::new(5, vec![]).unwrap());

    assert_eq!(
        merged,
        Err(Error::ElevationMismatch {
            expected: 3,
            actual: 5,
        })
    );
}

// --- SiteBuilder::build ---

#[test]
fn site_builder_rejects_invalid_identifier_and_location() {
    assert_eq!(
        site_parameter(Site::builder(*b"KT X", 35.0, -97.0).build()),
        "identifier"
    );
    assert_eq!(
        site_parameter(Site::builder([b'K', b'T', b'L', 0], 35.0, -97.0).build()),
        "identifier"
    );
    assert_eq!(
        site_parameter(Site::builder([b'K', b'T', b'L', 0xC3], 35.0, -97.0).build()),
        "identifier"
    );
    assert_eq!(
        site_parameter(Site::builder(*b"KTLX", 90.5, -97.0).build()),
        "latitude"
    );
    assert_eq!(
        site_parameter(Site::builder(*b"KTLX", f64::NAN, -97.0).build()),
        "latitude"
    );
    assert_eq!(
        site_parameter(Site::builder(*b"KTLX", 35.0, -180.5).build()),
        "longitude"
    );
    assert_eq!(
        site_parameter(Site::builder(*b"KTLX", 35.0, f64::INFINITY).build()),
        "longitude"
    );

    let site = Site::builder(*b"PGUA", -90.0, 180.0).build().unwrap();
    assert_eq!(site.identifier(), "PGUA");
    assert_eq!(site.to_string(), "PGUA (-90.0000°, 180.0000°, 0m)");
}

// --- SweepField extraction ---

#[test]
fn sweep_field_geometry_comes_from_product_bearing_radials() {
    // The first radial carries no reflectivity and has an unrelated elevation and spacing.
    let without_product = Radial::builder(0, 0.0, 1, 9.9)
        .azimuth_spacing_degrees(1.0)
        .build()
        .unwrap();
    let with_product = |azimuth: f32, elevation: f32| {
        Radial::builder(0, azimuth, 1, elevation)
            .azimuth_spacing_degrees(0.5)
            .moment(reflectivity(2125, vec![100, 110]))
            .build()
            .unwrap()
    };
    let radials = vec![
        without_product,
        with_product(1.0, 0.4),
        with_product(1.5, 0.5),
        with_product(2.0, 0.6),
        with_product(2.5, 2.0),
    ];

    let field = SweepField::from_radials(&radials, Product::Reflectivity).unwrap();
    assert_eq!(field.azimuth_count(), 4);
    // Median of the product-bearing radials' elevations (0.4, 0.5, 0.6, 2.0).
    assert!((field.elevation_angle_degrees() - 0.55).abs() < 1e-6);
    assert_eq!(field.azimuth_spacing_degrees(), 0.5);

    let owned = SweepField::from_radials_owned(radials, Product::Reflectivity).unwrap();
    assert_eq!(owned, field);
}

#[test]
fn sweep_field_skips_radials_with_different_gate_geometry() {
    let radial = |azimuth: f32, moment: MomentData| {
        Radial::builder(0, azimuth, 1, 0.5)
            .moment(moment)
            .build()
            .unwrap()
    };
    let shifted_interval = MomentData::from_fixed_point(
        Product::Reflectivity,
        2,
        2125,
        1000,
        8,
        2.0,
        66.0,
        vec![200, 200],
    )
    .unwrap();
    let radials = vec![
        radial(10.0, reflectivity(2125, vec![100, 110])),
        radial(11.0, reflectivity(4125, vec![200, 200])),
        radial(12.0, shifted_interval),
        radial(13.0, reflectivity(2125, vec![120, 130])),
    ];

    let field = SweepField::from_radials(&radials, Product::Reflectivity).unwrap();
    assert_eq!(field.azimuth_angles_degrees(), &[10.0, 13.0]);
    assert_eq!(field.first_gate_range_km(), 2.125);
    assert_eq!(field.gate_interval_km(), 0.25);
    assert_eq!(field.get(1, 0), Some((27.0, GateStatus::Valid)));

    let owned = SweepField::from_radials_owned(radials, Product::Reflectivity).unwrap();
    assert_eq!(owned, field);
}

// --- SweepField::value_at_polar ---

fn field_with_azimuths(azimuths: Vec<f32>) -> SweepField {
    let mut field = SweepField::builder("Reflectivity", "dBZ")
        .azimuth_angles_degrees(azimuths)
        .azimuth_spacing_degrees(1.0)
        .first_gate_range_km(0.0)
        .gate_interval_km(1.0)
        .gate_count(10)
        .build()
        .unwrap();
    for row in 0..field.azimuth_count() {
        field.set(row, 0, row as f32, GateStatus::Valid).unwrap();
    }
    field
}

fn row_at(field: &SweepField, azimuth: f64) -> Option<f32> {
    field.value_at_polar(azimuth, 0.5).map(|(value, _)| value)
}

#[test]
fn value_at_polar_wraps_only_for_full_circle() {
    let full = field_with_azimuths((0..360).map(|a| a as f32 + 0.5).collect());

    assert_eq!(row_at(&full, 359.9), Some(359.0));
    assert_eq!(row_at(&full, 0.1), Some(0.0));
    // Normalized into 0..360 before the lookup.
    assert_eq!(row_at(&full, -0.4), Some(359.0));
    assert_eq!(row_at(&full, 360.6), Some(0.0));
    assert_eq!(row_at(&full, 720.0 + 90.5), Some(90.0));
    assert_eq!(row_at(&full, f64::NAN), None);
}

#[test]
fn value_at_polar_does_not_match_outside_a_sector() {
    // A 90-degree sector from 10.5 to 99.5 with 1-degree spacing.
    let sector = field_with_azimuths((10..100).map(|a| a as f32 + 0.5).collect());

    assert_eq!(row_at(&sector, 10.5), Some(0.0));
    assert_eq!(row_at(&sector, 99.5), Some(89.0));
    // Within half the spacing beyond either edge.
    assert_eq!(row_at(&sector, 10.0), Some(0.0));
    assert_eq!(row_at(&sector, 100.0), Some(89.0));
    // Beyond the tolerance, and never matching the far end across 0/360.
    assert_eq!(row_at(&sector, 9.9), None);
    assert_eq!(row_at(&sector, 100.1), None);
    assert_eq!(row_at(&sector, 359.9), None);
    assert_eq!(row_at(&sector, 200.0), None);
    assert_eq!(row_at(&sector, -260.0), Some(89.0));
}

#[test]
fn value_at_polar_sector_tolerance_crosses_north() {
    // A sector starting at north matches just west of north within half the spacing.
    let sector = field_with_azimuths(vec![0.2, 1.2, 2.2]);

    assert_eq!(row_at(&sector, 359.8), Some(0.0));
    assert_eq!(row_at(&sector, 359.6), None);
    assert_eq!(row_at(&sector, 2.6), Some(2.0));
    assert_eq!(row_at(&sector, 2.8), None);
}

// --- value_range ---

#[test]
fn value_range_ignores_filtered_gates() {
    let mut sweep = SweepField::builder("Test", "dBZ")
        .azimuth_angles_degrees(vec![0.0, 1.0])
        .gate_interval_km(0.25)
        .gate_count(2)
        .build()
        .unwrap();
    sweep.set(0, 0, 10.0, GateStatus::Valid).unwrap();
    sweep.set(0, 1, 20.0, GateStatus::Valid).unwrap();
    sweep.set(1, 0, -50.0, GateStatus::Filtered).unwrap();
    sweep.set(1, 1, 90.0, GateStatus::Filtered).unwrap();
    assert_eq!(sweep.value_range(), Some((10.0, 20.0)));
    assert_eq!(sweep.get(1, 1), Some((90.0, GateStatus::Filtered)));

    let mut cartesian = CartesianField::new("Test", "dBZ", valid_extent(), 2, 1).unwrap();
    cartesian.set(0, 0, 5.0, GateStatus::Valid).unwrap();
    cartesian.set(0, 1, 90.0, GateStatus::Filtered).unwrap();
    assert_eq!(cartesian.value_range(), Some((5.0, 5.0)));

    let mut vertical =
        VerticalField::new("Test", "dBZ", (0.0, 100.0), (0.0, 10_000.0), 2, 1).unwrap();
    vertical.set(0, 0, 90.0, GateStatus::Filtered).unwrap();
    vertical.set(0, 1, 90.0, GateStatus::Filtered).unwrap();
    assert_eq!(vertical.value_range(), None);
}
