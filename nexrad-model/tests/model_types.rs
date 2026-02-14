//! Unit tests for nexrad-model types.

use nexrad_model::data::{
    CFPMomentData, CFPMomentValue, CFPStatus, DataMoment, MomentData, MomentValue, PulseWidth,
    Scan, Sweep, VolumeCoveragePattern,
};
use nexrad_model::meta::Site;

/// Helper to create a minimal VCP for testing
fn test_vcp(pattern_number: u16) -> VolumeCoveragePattern {
    VolumeCoveragePattern::new(
        pattern_number,
        1,   // version
        0.5, // doppler_velocity_resolution
        PulseWidth::Short,
        false,  // sails_enabled
        0,      // sails_cuts
        false,  // mrle_enabled
        0,      // mrle_cuts
        false,  // mpda_enabled
        false,  // base_tilt_enabled
        0,      // base_tilt_count
        false,  // sequence_active
        false,  // truncated
        vec![], // elevation_cuts
    )
}

#[test]
fn test_site_creation() {
    let site = Site::new(*b"KTLX", 35.3331, -97.2778, 370, 401);

    assert_eq!(site.identifier(), b"KTLX");
    assert_eq!(site.identifier_string(), "KTLX");
    assert!((site.latitude() - 35.3331).abs() < 0.0001);
    assert!((site.longitude() - (-97.2778)).abs() < 0.0001);
    assert_eq!(site.height_meters(), 370);
    assert_eq!(site.feedhorn_height_meters(), 401);
}

#[test]
fn test_site_display() {
    let site = Site::new(*b"KTLX", 35.3331, -97.2778, 370, 401);
    let display = format!("{}", site);

    assert!(display.contains("KTLX"));
    assert!(display.contains("35.3331"));
    assert!(display.contains("-97.2778"));
    assert!(display.contains("370"));
}

#[test]
fn test_site_debug() {
    let site = Site::new(*b"KTLX", 35.3331, -97.2778, 370, 401);
    let debug = format!("{:?}", site);

    assert!(debug.contains("Site"));
    assert!(debug.contains("KTLX"));
}

#[test]
fn test_site_clone() {
    let site = Site::new(*b"KTLX", 35.3331, -97.2778, 370, 401);
    let cloned = site.clone();

    assert_eq!(site.identifier(), cloned.identifier());
    assert_eq!(site.latitude(), cloned.latitude());
}

#[test]
fn test_scan_creation() {
    let sweeps = vec![Sweep::new(1, vec![]), Sweep::new(2, vec![])];
    let scan = Scan::new(test_vcp(212), sweeps);

    assert_eq!(scan.coverage_pattern_number(), 212);
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
    let moment = CFPMomentData::from_fixed_point(7, 500, 250, 8, 1.0, 0.0, values);

    let decoded = moment.values();
    assert_eq!(
        decoded[0],
        CFPMomentValue::Status(CFPStatus::FilterNotApplied)
    );
    assert_eq!(
        decoded[1],
        CFPMomentValue::Status(CFPStatus::PointClutterFilterApplied)
    );
    assert_eq!(
        decoded[2],
        CFPMomentValue::Status(CFPStatus::DualPolOnlyFilterApplied)
    );
    assert_eq!(decoded[3], CFPMomentValue::Status(CFPStatus::Reserved(3)));
    assert_eq!(decoded[4], CFPMomentValue::Status(CFPStatus::Reserved(7)));
    assert_eq!(decoded[5], CFPMomentValue::Value(8.0));
    assert_eq!(decoded[6], CFPMomentValue::Value(10.0));
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
        CFPMomentValue::Status(CFPStatus::FilterNotApplied),
        CFPMomentValue::Status(CFPStatus::FilterNotApplied)
    );
    assert_ne!(
        CFPMomentValue::Status(CFPStatus::FilterNotApplied),
        CFPMomentValue::Status(CFPStatus::PointClutterFilterApplied)
    );
    assert_eq!(CFPMomentValue::Value(10.0), CFPMomentValue::Value(10.0));
    assert_ne!(CFPMomentValue::Value(10.0), CFPMomentValue::Value(20.0));
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

    let debug = format!("{:?}", CFPMomentValue::Status(CFPStatus::Reserved(7)));
    assert!(debug.contains("Status"));
    assert!(debug.contains("Reserved"));
}
