//! Unit tests for nexrad-model types.

use nexrad_model::data::{
    CfpStatus, GateStatus, MomentData, MomentValue, Product, PulseWidth, Scan, Sweep, VcpNumber,
    VolumeCoveragePattern, WaveformType,
};
use nexrad_model::meta::Site;

/// Helper to create a minimal VCP for testing
fn test_vcp(pattern_number: u16) -> VolumeCoveragePattern {
    VolumeCoveragePattern::builder(VcpNumber::new(pattern_number))
        .version(1)
        .doppler_velocity_resolution_mps(0.5)
        .pulse_width(PulseWidth::Short)
        .build()
}

#[test]
fn test_site_creation() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .feedhorn_height_meters(401)
        .build()
        .unwrap();

    assert_eq!(site.identifier(), "KTLX");
    assert!((site.latitude() - 35.3331).abs() < 0.0001);
    assert!((site.longitude() - (-97.2778)).abs() < 0.0001);
    assert_eq!(site.site_elevation_meters(), 370);
    assert_eq!(site.feedhorn_height_meters(), 401);
}

#[test]
fn test_site_display() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .feedhorn_height_meters(401)
        .build()
        .unwrap();
    let display = format!("{}", site);

    assert!(display.contains("KTLX"));
    assert!(display.contains("35.3331"));
    assert!(display.contains("-97.2778"));
    assert!(display.contains("370"));
}

#[test]
fn test_site_debug() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .feedhorn_height_meters(401)
        .build()
        .unwrap();
    let debug = format!("{:?}", site);

    assert!(debug.contains("Site"));
    assert!(debug.contains("identifier"));
}

#[test]
fn test_site_clone() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .feedhorn_height_meters(401)
        .build()
        .unwrap();
    let cloned = site.clone();

    assert_eq!(site.identifier(), cloned.identifier());
    assert_eq!(site.latitude(), cloned.latitude());
}

#[test]
fn test_scan_creation() {
    let sweeps = vec![
        Sweep::new(1, vec![]).unwrap(),
        Sweep::new(2, vec![]).unwrap(),
    ];
    let scan = Scan::new(VcpNumber::VCP_212, Some(test_vcp(212)), sweeps);

    assert_eq!(scan.coverage_pattern_number(), VcpNumber::VCP_212);
    assert_eq!(scan.sweeps().len(), 2);
}

#[test]
fn test_scan_without_coverage_pattern() {
    let scan = Scan::new(
        VcpNumber::new(32),
        None,
        vec![Sweep::new(1, vec![]).unwrap()],
    );

    assert!(scan.coverage_pattern().is_none());
    assert_eq!(scan.coverage_pattern_number().number(), 32);
    assert!(scan.elevation_cut(&scan.sweeps()[0]).is_none());
    assert_eq!(
        scan.sweeps_with_waveform(WaveformType::ContiguousSurveillance)
            .count(),
        0
    );
    assert!(format!("{scan}").starts_with("Scan (VCP 32"));
}

#[test]
fn test_scan_keeps_its_own_coverage_pattern_number() {
    let vcp = VolumeCoveragePattern::builder(VcpNumber::new(0)).build();
    let scan = Scan::new(VcpNumber::new(32), Some(vcp), vec![]);

    assert_eq!(scan.coverage_pattern_number().number(), 32);
    assert_eq!(
        scan.coverage_pattern()
            .map(|vcp| vcp.pattern_number().number()),
        Some(0)
    );
}

#[test]
fn test_scan_display() {
    let sweeps = vec![
        Sweep::new(1, vec![]).unwrap(),
        Sweep::new(2, vec![]).unwrap(),
        Sweep::new(3, vec![]).unwrap(),
    ];
    let scan = Scan::new(VcpNumber::VCP_212, Some(test_vcp(212)), sweeps);
    let display = format!("{}", scan);

    assert!(display.contains("VCP 212"));
    assert!(display.contains("3 sweeps"));
}

#[test]
fn test_sweep_creation() {
    let sweep = Sweep::new(5, vec![]).unwrap();

    assert_eq!(sweep.elevation_number(), 5);
    assert!(sweep.radials().is_empty());
}

#[test]
fn test_sweep_display() {
    let sweep = Sweep::new(5, vec![]).unwrap();
    let display = format!("{}", sweep);

    // Empty sweep should have a reasonable display
    assert!(display.contains("no radials") || display.contains("Sweep"));
}

#[test]
fn test_sweep_merge_same_elevation() {
    let sweep1 = Sweep::new(3, vec![]).unwrap();
    let sweep2 = Sweep::new(3, vec![]).unwrap();

    let merged = sweep1.merge(sweep2);
    assert!(merged.is_ok());
    assert_eq!(merged.unwrap().elevation_number(), 3);
}

#[test]
fn test_sweep_merge_different_elevation() {
    let sweep1 = Sweep::new(3, vec![]).unwrap();
    let sweep2 = Sweep::new(5, vec![]).unwrap();

    let merged = sweep1.merge(sweep2);
    assert!(merged.is_err());
}

#[test]
fn test_moment_data_creation() {
    let values = vec![0, 1, 50, 100, 150, 200, 255];
    let moment =
        MomentData::from_fixed_point(Product::Reflectivity, 7, 500, 250, 8, 2.0, 33.0, values)
            .unwrap();

    assert_eq!(moment.product(), Product::Reflectivity);
    assert_eq!(moment.gate_count(), 7);
    assert_eq!(moment.data_word_size(), 8);
    assert_eq!(moment.scale(), 2.0);
    assert_eq!(moment.offset(), 33.0);
    assert_eq!(moment.raw_values(), &[0, 1, 50, 100, 150, 200, 255]);
    assert_eq!(
        moment.raw_gate_values().collect::<Vec<_>>(),
        vec![0, 1, 50, 100, 150, 200, 255]
    );
    assert!((moment.first_gate_range_km() - 0.5).abs() < 0.001);
    assert!((moment.gate_interval_km() - 0.25).abs() < 0.001);
}

#[test]
fn test_moment_data_values_with_scale() {
    // Test value conversion: (raw - offset) / scale
    // With scale=2.0, offset=33.0:
    // raw=0 -> BelowThreshold
    // raw=1 -> RangeFolded
    // raw=50 -> (50 - 33) / 2 = 8.5
    // raw=100 -> (100 - 33) / 2 = 33.5

    let values = vec![0, 1, 50, 100];
    let moment =
        MomentData::from_fixed_point(Product::Reflectivity, 4, 500, 250, 8, 2.0, 33.0, values)
            .unwrap();

    let decoded_values = moment.values();
    assert_eq!(decoded_values.len(), 4);

    assert_eq!(decoded_values[0], MomentValue::BelowThreshold);
    assert_eq!(decoded_values[1], MomentValue::RangeFolded);

    if let MomentValue::Value(v) = decoded_values[2] {
        assert!((v - 8.5).abs() < 0.01, "Expected 8.5, got {}", v);
    } else {
        panic!("Expected Value variant");
    }

    if let MomentValue::Value(v) = decoded_values[3] {
        assert!((v - 33.5).abs() < 0.01, "Expected 33.5, got {}", v);
    } else {
        panic!("Expected Value variant");
    }
}

#[test]
fn test_moment_data_values_without_scale() {
    // When scale=0.0, raw values 0 and 1 are still the below-threshold and range-folded
    // sentinels; larger raw values are passed through directly.
    let values = vec![0, 1, 50, 100];
    let moment =
        MomentData::from_fixed_point(Product::Reflectivity, 4, 500, 250, 8, 0.0, 0.0, values)
            .unwrap();

    assert_eq!(
        moment.values(),
        vec![
            MomentValue::BelowThreshold,
            MomentValue::RangeFolded,
            MomentValue::Value(50.0),
            MomentValue::Value(100.0),
        ]
    );
}

#[test]
fn test_moment_data_values_16bit_with_scale() {
    // Two 16-bit big-endian values: 20 and 30
    // With scale=2.0, offset=10.0 => 5.0 and 10.0
    let values = vec![0x00, 0x14, 0x00, 0x1E];
    let moment =
        MomentData::from_fixed_point(Product::Reflectivity, 2, 500, 250, 16, 2.0, 10.0, values)
            .unwrap();

    let decoded_values = moment.values();
    assert_eq!(decoded_values.len(), 2);

    if let MomentValue::Value(v) = decoded_values[0] {
        assert!((v - 5.0).abs() < 0.01, "Expected 5.0, got {}", v);
    } else {
        panic!("Expected Value variant");
    }

    if let MomentValue::Value(v) = decoded_values[1] {
        assert!((v - 10.0).abs() < 0.01, "Expected 10.0, got {}", v);
    } else {
        panic!("Expected Value variant");
    }
}

#[test]
fn test_moment_data_values_16bit_special_values() {
    // 16-bit raw values 0 and 1 should map to special cases when scale != 0.
    let values = vec![0x00, 0x00, 0x00, 0x01];
    let moment =
        MomentData::from_fixed_point(Product::Reflectivity, 2, 500, 250, 16, 2.0, 0.0, values)
            .unwrap();

    let decoded_values = moment.values();
    assert_eq!(decoded_values.len(), 2);
    assert_eq!(decoded_values[0], MomentValue::BelowThreshold);
    assert_eq!(decoded_values[1], MomentValue::RangeFolded);
}

#[test]
fn test_cfp_moment_values_status_and_data() {
    let values = vec![0, 1, 2, 3, 7, 8, 10];
    let moment = MomentData::from_fixed_point(
        Product::ClutterFilterPower,
        7,
        500,
        250,
        8,
        1.0,
        0.0,
        values,
    )
    .unwrap();

    let decoded = moment.values();
    assert_eq!(
        decoded[0],
        MomentValue::CfpStatus(CfpStatus::FilterNotApplied)
    );
    assert_eq!(
        decoded[1],
        MomentValue::CfpStatus(CfpStatus::PointClutterFilterApplied)
    );
    assert_eq!(
        decoded[2],
        MomentValue::CfpStatus(CfpStatus::DualPolOnlyFilterApplied)
    );
    assert_eq!(decoded[3], MomentValue::CfpStatus(CfpStatus::Reserved(3)));
    assert_eq!(decoded[4], MomentValue::CfpStatus(CfpStatus::Reserved(7)));
    assert_eq!(decoded[5], MomentValue::Value(8.0));
    assert_eq!(decoded[6], MomentValue::Value(10.0));
}

#[test]
fn test_cfp_moment_values_without_scale() {
    // Status codes take precedence over the unscaled passthrough.
    let values = vec![0, 7, 8, 42];
    let moment = MomentData::from_fixed_point(
        Product::ClutterFilterPower,
        4,
        500,
        250,
        8,
        0.0,
        0.0,
        values,
    )
    .unwrap();

    let decoded = moment.values();
    assert_eq!(
        decoded[0],
        MomentValue::CfpStatus(CfpStatus::FilterNotApplied)
    );
    assert_eq!(decoded[1], MomentValue::CfpStatus(CfpStatus::Reserved(7)));
    assert_eq!(decoded[2], MomentValue::Value(8.0));
    assert_eq!(decoded[3], MomentValue::Value(42.0));
}

#[test]
fn test_non_cfp_moment_values_have_no_status_codes() {
    // Raw values 2-7 are ordinary values for every product other than CFP.
    let values = vec![2, 7];
    let moment =
        MomentData::from_fixed_point(Product::Velocity, 2, 500, 250, 8, 1.0, 0.0, values).unwrap();

    assert_eq!(
        moment.values(),
        vec![MomentValue::Value(2.0), MomentValue::Value(7.0)]
    );
}

#[test]
fn test_gate_status_from_moment_value() {
    assert_eq!(GateStatus::from(MomentValue::Value(1.0)), GateStatus::Valid);
    assert_eq!(
        GateStatus::from(MomentValue::BelowThreshold),
        GateStatus::BelowThreshold
    );
    assert_eq!(
        GateStatus::from(MomentValue::RangeFolded),
        GateStatus::RangeFolded
    );
    assert_eq!(
        GateStatus::from(MomentValue::CfpStatus(CfpStatus::FilterNotApplied)),
        GateStatus::NoData
    );
}

#[test]
fn test_moment_value_equality() {
    assert_eq!(MomentValue::BelowThreshold, MomentValue::BelowThreshold);
    assert_eq!(MomentValue::RangeFolded, MomentValue::RangeFolded);
    assert_eq!(MomentValue::Value(10.0), MomentValue::Value(10.0));

    assert_ne!(MomentValue::BelowThreshold, MomentValue::RangeFolded);
    assert_ne!(MomentValue::Value(10.0), MomentValue::Value(20.0));
}

#[test]
fn test_cfp_status_moment_value_equality() {
    assert_eq!(
        MomentValue::CfpStatus(CfpStatus::FilterNotApplied),
        MomentValue::CfpStatus(CfpStatus::FilterNotApplied)
    );
    assert_ne!(
        MomentValue::CfpStatus(CfpStatus::FilterNotApplied),
        MomentValue::CfpStatus(CfpStatus::PointClutterFilterApplied)
    );
    assert_ne!(
        MomentValue::CfpStatus(CfpStatus::FilterNotApplied),
        MomentValue::BelowThreshold
    );
}

#[test]
fn test_moment_value_debug() {
    let debug = format!("{:?}", MomentValue::Value(42.5));
    assert!(debug.contains("Value"));
    assert!(debug.contains("42.5"));

    let debug = format!("{:?}", MomentValue::BelowThreshold);
    assert!(debug.contains("BelowThreshold"));

    let debug = format!("{:?}", MomentValue::RangeFolded);
    assert!(debug.contains("RangeFolded"));

    let debug = format!("{:?}", MomentValue::CfpStatus(CfpStatus::Reserved(7)));
    assert!(debug.contains("CfpStatus"));
    assert!(debug.contains("Reserved"));
}

#[test]
fn test_radial_builder_defaults() {
    use nexrad_model::data::{Radial, RadialStatus};

    let radial = Radial::builder(12, 6.5, 2, 0.9).build().unwrap();

    assert_eq!(radial.azimuth_number(), 12);
    assert_eq!(radial.azimuth_angle_degrees(), 6.5);
    assert_eq!(radial.elevation_number(), 2);
    assert_eq!(radial.elevation_angle_degrees(), 0.9);
    assert_eq!(radial.collection_timestamp_millis(), 0);
    assert_eq!(radial.azimuth_spacing_degrees(), 1.0);
    assert_eq!(radial.radial_status(), RadialStatus::IntermediateRadialData);
    assert!(radial.reflectivity().is_none());
    assert!(radial.velocity().is_none());
    assert!(radial.spectrum_width().is_none());
    assert!(radial.differential_reflectivity().is_none());
    assert!(radial.differential_phase().is_none());
    assert!(radial.correlation_coefficient().is_none());
    assert!(radial.clutter_filter_power().is_none());
    assert!(radial.moment(Product::Reflectivity).is_none());
    assert!(radial.moment(Product::ClutterFilterPower).is_none());
}

#[test]
fn test_radial_builder_setters() {
    use nexrad_model::data::{Radial, RadialStatus};

    let moment = MomentData::from_fixed_point(
        Product::Reflectivity,
        3,
        2000,
        250,
        8,
        2.0,
        66.0,
        vec![100, 110, 120],
    )
    .unwrap();
    let radial = Radial::builder(1, 0.25, 1, 0.5)
        .collection_timestamp_millis(1_700_000_000_000)
        .azimuth_spacing_degrees(0.5)
        .radial_status(RadialStatus::ScanStart)
        .moment(moment.clone())
        .build()
        .unwrap();

    assert_eq!(radial.collection_timestamp_millis(), 1_700_000_000_000);
    assert_eq!(radial.azimuth_spacing_degrees(), 0.5);
    assert_eq!(radial.radial_status(), RadialStatus::ScanStart);
    assert_eq!(radial.reflectivity(), Some(&moment));
    assert_eq!(radial.moment(Product::Reflectivity), Some(&moment));
    assert!(radial.velocity().is_none());
}

#[test]
fn test_radial_builder_moment_stores_by_product() {
    use nexrad_model::data::Radial;

    let products = [
        Product::Reflectivity,
        Product::Velocity,
        Product::SpectrumWidth,
        Product::DifferentialReflectivity,
        Product::DifferentialPhase,
        Product::CorrelationCoefficient,
        Product::ClutterFilterPower,
    ];

    let mut builder = Radial::builder(1, 0.25, 1, 0.5);
    for (i, product) in products.iter().enumerate() {
        builder = builder.moment(
            MomentData::from_fixed_point(*product, 1, 2000, 250, 8, 2.0, 66.0, vec![100 + i as u8])
                .unwrap(),
        );
    }
    let radial = builder.build().unwrap();

    for product in products {
        let moment = radial.moment(product).expect("moment is set");
        assert_eq!(moment.product(), product);
    }
    assert_eq!(radial.reflectivity().unwrap().raw_values(), &[100]);
    assert_eq!(radial.velocity().unwrap().raw_values(), &[101]);
    assert_eq!(radial.spectrum_width().unwrap().raw_values(), &[102]);
    assert_eq!(
        radial.differential_reflectivity().unwrap().raw_values(),
        &[103]
    );
    assert_eq!(radial.differential_phase().unwrap().raw_values(), &[104]);
    assert_eq!(
        radial.correlation_coefficient().unwrap().raw_values(),
        &[105]
    );
    assert_eq!(radial.clutter_filter_power().unwrap().raw_values(), &[106]);
}

#[test]
fn test_elevation_cut_builder_defaults() {
    use nexrad_model::data::{ChannelConfiguration, ElevationCut, WaveformType};

    let cut = ElevationCut::builder(
        0.5,
        WaveformType::ContiguousSurveillance,
        ChannelConfiguration::Sz2Phase,
    )
    .is_sails_cut(true)
    .sails_sequence_number(1)
    .build();

    assert_eq!(cut.elevation_angle_degrees(), 0.5);
    assert_eq!(cut.waveform_type(), WaveformType::ContiguousSurveillance);
    assert_eq!(cut.channel_configuration(), ChannelConfiguration::Sz2Phase);
    assert_eq!(cut.azimuth_rate_degrees_per_second(), 0.0);
    assert!(!cut.super_resolution_half_degree_azimuth());
    assert_eq!(cut.surveillance_prf_number(), 0);
    assert_eq!(cut.reflectivity_threshold_db(), 0.0);
    assert!(cut.is_sails_cut());
    assert_eq!(cut.sails_sequence_number(), 1);
    assert!(!cut.is_mrle_cut());
}

#[test]
fn test_vcp_builder_defaults_and_cuts() {
    use nexrad_model::data::{ChannelConfiguration, ElevationCut, WaveformType};

    let cut = ElevationCut::builder(
        0.5,
        WaveformType::ContiguousSurveillance,
        ChannelConfiguration::ConstantPhase,
    )
    .build();

    let vcp = VolumeCoveragePattern::builder(VcpNumber::new(212)).build();
    assert_eq!(vcp.pattern_number(), VcpNumber::new(212));
    assert_eq!(vcp.version(), 0);
    assert_eq!(vcp.doppler_velocity_resolution_mps(), 0.5);
    assert_eq!(vcp.pulse_width(), PulseWidth::Short);
    assert!(!vcp.sails_enabled());
    assert!(vcp.elevation_cuts().is_empty());

    let vcp = VolumeCoveragePattern::builder(VcpNumber::new(212))
        .elevation_cut(cut.clone())
        .elevation_cut(cut.clone())
        .build();
    assert_eq!(vcp.number_of_elevation_cuts(), 2);

    let vcp = VolumeCoveragePattern::builder(VcpNumber::new(212))
        .elevation_cut(cut.clone())
        .elevation_cuts(vec![cut])
        .build();
    assert_eq!(vcp.number_of_elevation_cuts(), 1);
}

#[test]
fn test_site_builder_defaults() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778).build().unwrap();
    assert_eq!(site.identifier(), "KTLX");
    assert_eq!(site.site_elevation_meters(), 0);
    assert_eq!(site.feedhorn_height_meters(), 0);
}

#[test]
fn test_sweep_field_builder() {
    use nexrad_model::data::{GateStatus, SweepField};

    let field = SweepField::builder("Reflectivity", "dBZ")
        .elevation_angle_degrees(0.5)
        .azimuth_angles_degrees(vec![0.0, 1.0, 2.0])
        .first_gate_range_km(2.0)
        .gate_interval_km(0.25)
        .gate_count(4)
        .build()
        .expect("valid geometry");

    assert_eq!(field.label(), "Reflectivity");
    assert_eq!(field.unit(), "dBZ");
    assert_eq!(field.elevation_angle_degrees(), 0.5);
    assert_eq!(field.azimuth_count(), 3);
    assert_eq!(field.azimuth_spacing_degrees(), 1.0);
    assert_eq!(field.first_gate_range_km(), 2.0);
    assert_eq!(field.gate_interval_km(), 0.25);
    assert_eq!(field.gate_count(), 4);
    assert_eq!(field.values(), &[0.0; 12]);
    assert!(field.statuses().iter().all(|s| *s == GateStatus::NoData));

    let like = field.new_like("Copy", "m/s");
    assert_eq!(like.label(), "Copy");
    assert_eq!(
        like.azimuth_angles_degrees(),
        field.azimuth_angles_degrees()
    );
    assert_eq!(like.gate_count(), field.gate_count());
}

#[test]
fn test_sweep_field_builder_rejects_invalid_geometry() {
    use nexrad_model::data::SweepField;
    use nexrad_model::Error;

    let valid = SweepField::builder("Test", "dBZ")
        .azimuth_angles_degrees(vec![0.0])
        .gate_interval_km(0.25)
        .gate_count(1);

    assert!(matches!(
        valid.clone().azimuth_angles_degrees(vec![]).build(),
        Err(Error::InvalidGeometry { .. })
    ));
    assert!(matches!(
        valid.clone().gate_count(0).build(),
        Err(Error::InvalidGeometry { .. })
    ));
    assert!(matches!(
        valid.clone().gate_interval_km(0.0).build(),
        Err(Error::InvalidGeometry { .. })
    ));
    assert!(matches!(
        valid.clone().gate_interval_km(f64::NAN).build(),
        Err(Error::InvalidGeometry { .. })
    ));
    assert!(valid.build().is_ok());

    // The defaults alone do not describe a valid geometry.
    assert!(SweepField::builder("Test", "dBZ").build().is_err());
}

#[test]
fn test_sweep_field_product() {
    use nexrad_model::data::{Radial, SweepField};

    // CFP raw values 0-7 are status codes and become NoData gates in the field.
    let cfp = MomentData::from_fixed_point(
        Product::ClutterFilterPower,
        3,
        2000,
        250,
        8,
        1.0,
        0.0,
        vec![1, 8, 12],
    )
    .unwrap();
    let radials = vec![Radial::builder(0, 0.0, 1, 0.5).moment(cfp).build().unwrap()];

    let field = SweepField::from_radials(&radials, Product::ClutterFilterPower).expect("cfp data");
    assert_eq!(field.product(), Some(Product::ClutterFilterPower));
    assert_eq!(field.get(0, 0), Some((0.0, GateStatus::NoData)));
    assert_eq!(field.get(0, 1), Some((8.0, GateStatus::Valid)));
    assert_eq!(field.get(0, 2), Some((12.0, GateStatus::Valid)));
    assert!(SweepField::from_radials(&radials, Product::Reflectivity).is_none());

    let owned = SweepField::from_radials_owned(radials, Product::ClutterFilterPower).unwrap();
    assert_eq!(owned, field);

    let like = field.new_like("Derived", "dB");
    assert_eq!(like.product(), Some(Product::ClutterFilterPower));

    let built = SweepField::builder("Reflectivity", "dBZ")
        .azimuth_angles_degrees(vec![0.0])
        .gate_count(1)
        .gate_interval_km(0.25)
        .build()
        .unwrap();
    assert_eq!(built.product(), None);

    let built = SweepField::builder("Reflectivity", "dBZ")
        .product(Product::Reflectivity)
        .azimuth_angles_degrees(vec![0.0])
        .gate_count(1)
        .gate_interval_km(0.25)
        .build()
        .unwrap();
    assert_eq!(built.product(), Some(Product::Reflectivity));
}

/// Helper to create a single-radial sweep carrying the given moments.
fn sweep_with_moments(elevation_number: u8, moments: Vec<MomentData>) -> Sweep {
    use nexrad_model::data::Radial;

    let mut builder = Radial::builder(0, 0.0, elevation_number, 0.5);
    for moment in moments {
        builder = builder.moment(moment);
    }
    Sweep::new(elevation_number, vec![builder.build().unwrap()]).unwrap()
}

/// Helper to create a moment with one gate per raw value.
fn test_moment(product: Product, values: Vec<u8>) -> MomentData {
    MomentData::from_fixed_point(product, values.len(), 2000, 250, 8, 2.0, 66.0, values).unwrap()
}

#[test]
fn test_sweep_field() {
    use nexrad_model::data::SweepField;

    let sweep = sweep_with_moments(1, vec![test_moment(Product::Reflectivity, vec![100, 110])]);

    let field = sweep
        .field(Product::Reflectivity)
        .expect("reflectivity data");
    assert_eq!(
        Some(field),
        SweepField::from_radials(sweep.radials(), Product::Reflectivity)
    );
    assert!(sweep.field(Product::Velocity).is_none());
    assert!(Sweep::new(1, vec![])
        .unwrap()
        .field(Product::Reflectivity)
        .is_none());
}

#[test]
fn test_scan_fields() {
    let scan = Scan::new(
        VcpNumber::VCP_212,
        Some(test_vcp(212)),
        vec![
            sweep_with_moments(1, vec![test_moment(Product::Reflectivity, vec![100])]),
            sweep_with_moments(2, vec![test_moment(Product::Velocity, vec![120])]),
            sweep_with_moments(
                3,
                vec![test_moment(Product::Reflectivity, vec![100, 110, 120])],
            ),
        ],
    );

    let fields = scan.fields(Product::Reflectivity);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].gate_count(), 1);
    assert_eq!(fields[1].gate_count(), 3);

    assert_eq!(scan.fields(Product::Velocity).len(), 1);
    assert!(scan.fields(Product::SpectrumWidth).is_empty());
}

#[test]
fn test_scan_first_field() {
    let scan = Scan::new(
        VcpNumber::VCP_212,
        Some(test_vcp(212)),
        vec![
            sweep_with_moments(1, vec![test_moment(Product::Reflectivity, vec![100])]),
            sweep_with_moments(2, vec![test_moment(Product::Velocity, vec![120, 130])]),
            sweep_with_moments(3, vec![test_moment(Product::Velocity, vec![120])]),
        ],
    );

    let (index, field) = scan.first_field(Product::Velocity).expect("velocity data");
    assert_eq!(index, 1);
    assert_eq!(field.gate_count(), 2);
    assert_eq!(field.product(), Some(Product::Velocity));

    let (index, _) = scan
        .first_field(Product::Reflectivity)
        .expect("reflectivity data");
    assert_eq!(index, 0);

    assert!(scan.first_field(Product::SpectrumWidth).is_none());
}

#[test]
fn test_scan_coordinate_system() {
    use nexrad_model::geo::RadarCoordinateSystem;

    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .site_elevation_meters(370)
        .feedhorn_height_meters(20)
        .build()
        .unwrap();
    let scan = Scan::with_site(
        site.clone(),
        VcpNumber::VCP_212,
        Some(test_vcp(212)),
        vec![],
    );

    let coords = scan.coordinate_system().expect("site metadata");
    assert_eq!(coords, RadarCoordinateSystem::new(&site));
    assert_eq!(coords.antenna_height_meters(), 390.0);

    assert!(Scan::new(VcpNumber::VCP_212, Some(test_vcp(212)), vec![])
        .coordinate_system()
        .is_none());
}
