//! Unit tests for nexrad-model types.

use nexrad_model::data::{
    CfpMomentData, CfpMomentValue, CfpStatus, DataMoment, MomentData, MomentValue, PulseWidth,
    Scan, Sweep, VcpNumber, VolumeCoveragePattern,
};
use nexrad_model::meta::Site;

/// Helper to create a minimal VCP for testing
fn test_vcp(pattern_number: u16) -> VolumeCoveragePattern {
    VolumeCoveragePattern::builder(VcpNumber::from_number(pattern_number))
        .version(1)
        .doppler_velocity_resolution(0.5)
        .pulse_width(PulseWidth::Short)
        .build()
}

#[test]
fn test_site_creation() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .height_meters(370)
        .tower_height_meters(401)
        .build();

    assert_eq!(site.identifier(), b"KTLX");
    assert_eq!(site.identifier_string(), "KTLX");
    assert!((site.latitude() - 35.3331).abs() < 0.0001);
    assert!((site.longitude() - (-97.2778)).abs() < 0.0001);
    assert_eq!(site.height_meters(), 370);
    assert_eq!(site.tower_height_meters(), 401);
}

#[test]
fn test_site_display() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .height_meters(370)
        .tower_height_meters(401)
        .build();
    let display = format!("{}", site);

    assert!(display.contains("KTLX"));
    assert!(display.contains("35.3331"));
    assert!(display.contains("-97.2778"));
    assert!(display.contains("370"));
}

#[test]
fn test_site_debug() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .height_meters(370)
        .tower_height_meters(401)
        .build();
    let debug = format!("{:?}", site);

    assert!(debug.contains("Site"));
    assert!(debug.contains("identifier"));
}

#[test]
fn test_site_clone() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778)
        .height_meters(370)
        .tower_height_meters(401)
        .build();
    let cloned = site.clone();

    assert_eq!(site.identifier(), cloned.identifier());
    assert_eq!(site.latitude(), cloned.latitude());
}

#[test]
fn test_scan_creation() {
    let sweeps = vec![Sweep::new(1, vec![]), Sweep::new(2, vec![])];
    let scan = Scan::new(test_vcp(212), sweeps);

    assert_eq!(
        scan.coverage_pattern_number(),
        VcpNumber::PrecipitationSz2_212
    );
    assert_eq!(scan.sweeps().len(), 2);
}

#[test]
fn test_scan_display() {
    let sweeps = vec![
        Sweep::new(1, vec![]),
        Sweep::new(2, vec![]),
        Sweep::new(3, vec![]),
    ];
    let scan = Scan::new(test_vcp(212), sweeps);
    let display = format!("{}", scan);

    assert!(display.contains("VCP 212"));
    assert!(display.contains("3 sweeps"));
}

#[test]
fn test_sweep_creation() {
    let sweep = Sweep::new(5, vec![]);

    assert_eq!(sweep.elevation_number(), 5);
    assert!(sweep.radials().is_empty());
}

#[test]
fn test_sweep_display() {
    let sweep = Sweep::new(5, vec![]);
    let display = format!("{}", sweep);

    // Empty sweep should have a reasonable display
    assert!(display.contains("no radials") || display.contains("Sweep"));
}

#[test]
fn test_sweep_merge_same_elevation() {
    let sweep1 = Sweep::new(3, vec![]);
    let sweep2 = Sweep::new(3, vec![]);

    let merged = sweep1.merge(sweep2);
    assert!(merged.is_ok());
    assert_eq!(merged.unwrap().elevation_number(), 3);
}

#[test]
fn test_sweep_merge_different_elevation() {
    let sweep1 = Sweep::new(3, vec![]);
    let sweep2 = Sweep::new(5, vec![]);

    let merged = sweep1.merge(sweep2);
    assert!(merged.is_err());
}

#[test]
fn test_moment_data_creation() {
    let values = vec![0, 1, 50, 100, 150, 200, 255];
    let moment = MomentData::from_fixed_point(7, 500, 250, 8, 2.0, 33.0, values);

    assert_eq!(moment.gate_count(), 7);
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
    let moment = MomentData::from_fixed_point(4, 500, 250, 8, 2.0, 33.0, values);

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
    // When scale=0.0, raw values are passed through directly
    let values = vec![0, 1, 50, 100];
    let moment = MomentData::from_fixed_point(4, 500, 250, 8, 0.0, 0.0, values);

    let decoded_values = moment.values();

    // All should be Value variants with raw values
    for (i, v) in decoded_values.iter().enumerate() {
        match v {
            MomentValue::Value(val) => {
                let expected = [0.0, 1.0, 50.0, 100.0][i];
                assert!((*val - expected).abs() < 0.01);
            }
            _ => panic!("Expected Value variant, got {:?}", v),
        }
    }
}

#[test]
fn test_moment_data_values_16bit_with_scale() {
    // Two 16-bit big-endian values: 20 and 30
    // With scale=2.0, offset=10.0 => 5.0 and 10.0
    let values = vec![0x00, 0x14, 0x00, 0x1E];
    let moment = MomentData::from_fixed_point(2, 500, 250, 16, 2.0, 10.0, values);

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
    let moment = MomentData::from_fixed_point(2, 500, 250, 16, 2.0, 0.0, values);

    let decoded_values = moment.values();
    assert_eq!(decoded_values.len(), 2);
    assert_eq!(decoded_values[0], MomentValue::BelowThreshold);
    assert_eq!(decoded_values[1], MomentValue::RangeFolded);
}

#[test]
fn test_cfp_moment_values_status_and_data() {
    let values = vec![0, 1, 2, 3, 7, 8, 10];
    let moment = CfpMomentData::from_fixed_point(7, 500, 250, 8, 1.0, 0.0, values);

    let decoded = moment.values();
    assert_eq!(
        decoded[0],
        CfpMomentValue::Status(CfpStatus::FilterNotApplied)
    );
    assert_eq!(
        decoded[1],
        CfpMomentValue::Status(CfpStatus::PointClutterFilterApplied)
    );
    assert_eq!(
        decoded[2],
        CfpMomentValue::Status(CfpStatus::DualPolOnlyFilterApplied)
    );
    assert_eq!(decoded[3], CfpMomentValue::Status(CfpStatus::Reserved(3)));
    assert_eq!(decoded[4], CfpMomentValue::Status(CfpStatus::Reserved(7)));
    assert_eq!(decoded[5], CfpMomentValue::Value(8.0));
    assert_eq!(decoded[6], CfpMomentValue::Value(10.0));
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
fn test_cfp_moment_value_equality() {
    assert_eq!(
        CfpMomentValue::Status(CfpStatus::FilterNotApplied),
        CfpMomentValue::Status(CfpStatus::FilterNotApplied)
    );
    assert_ne!(
        CfpMomentValue::Status(CfpStatus::FilterNotApplied),
        CfpMomentValue::Status(CfpStatus::PointClutterFilterApplied)
    );
    assert_eq!(CfpMomentValue::Value(10.0), CfpMomentValue::Value(10.0));
    assert_ne!(CfpMomentValue::Value(10.0), CfpMomentValue::Value(20.0));
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

    let debug = format!("{:?}", CfpMomentValue::Status(CfpStatus::Reserved(7)));
    assert!(debug.contains("Status"));
    assert!(debug.contains("Reserved"));
}

#[test]
fn test_radial_builder_defaults() {
    use nexrad_model::data::{Radial, RadialStatus};

    let radial = Radial::builder(12, 6.5, 2, 0.9).build();

    assert_eq!(radial.azimuth_number(), 12);
    assert_eq!(radial.azimuth_angle_degrees(), 6.5);
    assert_eq!(radial.elevation_number(), 2);
    assert_eq!(radial.elevation_angle_degrees(), 0.9);
    assert_eq!(radial.collection_timestamp(), 0);
    assert_eq!(radial.azimuth_spacing_degrees(), 1.0);
    assert_eq!(radial.radial_status(), RadialStatus::IntermediateRadialData);
    assert!(radial.reflectivity().is_none());
    assert!(radial.velocity().is_none());
    assert!(radial.spectrum_width().is_none());
    assert!(radial.differential_reflectivity().is_none());
    assert!(radial.differential_phase().is_none());
    assert!(radial.correlation_coefficient().is_none());
    assert!(radial.clutter_filter_power().is_none());
}

#[test]
fn test_radial_builder_setters() {
    use nexrad_model::data::{Radial, RadialStatus};

    let moment = MomentData::from_fixed_point(3, 2000, 250, 8, 2.0, 66.0, vec![100, 110, 120]);
    let radial = Radial::builder(1, 0.25, 1, 0.5)
        .collection_timestamp(1_700_000_000_000)
        .azimuth_spacing_degrees(0.5)
        .radial_status(RadialStatus::ScanStart)
        .reflectivity(moment.clone())
        .build();

    assert_eq!(radial.collection_timestamp(), 1_700_000_000_000);
    assert_eq!(radial.azimuth_spacing_degrees(), 0.5);
    assert_eq!(radial.radial_status(), RadialStatus::ScanStart);
    assert_eq!(radial.reflectivity(), Some(&moment));
    assert!(radial.velocity().is_none());
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

    let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212)).build();
    assert_eq!(vcp.pattern_number(), VcpNumber::from_number(212));
    assert_eq!(vcp.version(), 0);
    assert_eq!(vcp.doppler_velocity_resolution(), 0.5);
    assert_eq!(vcp.pulse_width(), PulseWidth::Short);
    assert!(!vcp.sails_enabled());
    assert!(vcp.elevation_cuts().is_empty());

    let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212))
        .elevation_cut(cut.clone())
        .elevation_cut(cut.clone())
        .build();
    assert_eq!(vcp.number_of_elevation_cuts(), 2);

    let vcp = VolumeCoveragePattern::builder(VcpNumber::from_number(212))
        .elevation_cut(cut.clone())
        .elevation_cuts(vec![cut])
        .build();
    assert_eq!(vcp.number_of_elevation_cuts(), 1);
}

#[test]
fn test_site_builder_defaults() {
    let site = Site::builder(*b"KTLX", 35.3331, -97.2778).build();
    assert_eq!(site.identifier(), b"KTLX");
    assert_eq!(site.height_meters(), 0);
    assert_eq!(site.tower_height_meters(), 0);
}

#[test]
fn test_sweep_field_builder() {
    use nexrad_model::data::{GateStatus, SweepField};

    let field = SweepField::builder("Reflectivity", "dBZ")
        .elevation_degrees(0.5)
        .azimuths(vec![0.0, 1.0, 2.0])
        .first_gate_range_km(2.0)
        .gate_interval_km(0.25)
        .gate_count(4)
        .build()
        .expect("valid geometry");

    assert_eq!(field.label(), "Reflectivity");
    assert_eq!(field.unit(), "dBZ");
    assert_eq!(field.elevation_degrees(), 0.5);
    assert_eq!(field.azimuth_count(), 3);
    assert_eq!(field.azimuth_spacing_degrees(), 1.0);
    assert_eq!(field.first_gate_range_km(), 2.0);
    assert_eq!(field.gate_interval_km(), 0.25);
    assert_eq!(field.gate_count(), 4);
    assert_eq!(field.values(), &[0.0; 12]);
    assert!(field.statuses().iter().all(|s| *s == GateStatus::NoData));

    let like = field.new_like("Copy", "m/s");
    assert_eq!(like.label(), "Copy");
    assert_eq!(like.azimuths(), field.azimuths());
    assert_eq!(like.gate_count(), field.gate_count());
}

#[test]
fn test_sweep_field_builder_rejects_invalid_geometry() {
    use nexrad_model::data::SweepField;
    use nexrad_model::Error;

    let valid = SweepField::builder("Test", "dBZ")
        .azimuths(vec![0.0])
        .gate_interval_km(0.25)
        .gate_count(1);

    assert!(matches!(
        valid.clone().azimuths(vec![]).build(),
        Err(Error::InvalidSweepGeometry(_))
    ));
    assert!(matches!(
        valid.clone().gate_count(0).build(),
        Err(Error::InvalidSweepGeometry(_))
    ));
    assert!(matches!(
        valid.clone().gate_interval_km(0.0).build(),
        Err(Error::InvalidSweepGeometry(_))
    ));
    assert!(matches!(
        valid.clone().gate_interval_km(f64::NAN).build(),
        Err(Error::InvalidSweepGeometry(_))
    ));
    assert!(valid.build().is_ok());

    // The defaults alone do not describe a valid geometry.
    assert!(SweepField::builder("Test", "dBZ").build().is_err());
}
