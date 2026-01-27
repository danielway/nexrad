//! Snapshot tests for curated fixtures.
//!
//! These tests capture stable derived outputs that should not change
//! across decoder versions unless intentionally modified.

#![cfg(feature = "serde")]

use insta::assert_yaml_snapshot;
use nexrad_data::volume;
use nexrad_model::data::Scan;
use serde::Serialize;
use sha2::{Digest, Sha256};

// =============================================================================
// FIXTURE LOADING
// =============================================================================

const FIXTURE_EXISTING_KDMX: &[u8] =
    include_bytes!("../../tests/fixtures/convective/KDMX20220305_232324.bin");
const FIXTURE_HURRICANE_HARVEY: &[u8] =
    include_bytes!("../../tests/fixtures/tropical/KCRP20170826_044114.bin");
const FIXTURE_MODERN_BUILD: &[u8] =
    include_bytes!("../../tests/fixtures/clear_air/KDMX20250314_175512.bin");
const FIXTURE_CLEAR_AIR_PLAINS: &[u8] =
    include_bytes!("../../tests/fixtures/clear_air/KTLX20190715_120037.bin");
const FIXTURE_TORNADO_MAY2019: &[u8] =
    include_bytes!("../../tests/fixtures/convective/KTLX20190525_222419.bin");
const FIXTURE_SUPERCELL_TEXAS: &[u8] =
    include_bytes!("../../tests/fixtures/convective/KFWS20190609_222523.bin");
const FIXTURE_WINTER_GREATLAKES: &[u8] =
    include_bytes!("../../tests/fixtures/winter/KMKX20200115_150035.bin");
const FIXTURE_STRATIFORM_NOREASTER: &[u8] =
    include_bytes!("../../tests/fixtures/stratiform/KLWX20180302_115347.bin");

fn load_scan(data: &[u8]) -> Scan {
    let volume = volume::File::new(data.to_vec());
    volume.scan().expect("should decode")
}

// =============================================================================
// METADATA SNAPSHOT STRUCTURES
// =============================================================================

/// Snapshot of scan metadata (non-binary fields).
#[derive(Serialize)]
struct ScanMetadataSnapshot {
    coverage_pattern_number: u16,
    sweep_count: usize,
    sweeps: Vec<SweepMetadataSnapshot>,
}

#[derive(Serialize)]
struct SweepMetadataSnapshot {
    elevation_number: u8,
    radial_count: usize,
    first_azimuth_degrees: f32,
    elevation_angle_degrees: f32,
    azimuth_spacing_degrees: f32,
    moments_present: Vec<String>,
}

fn create_scan_metadata_snapshot(scan: &Scan) -> ScanMetadataSnapshot {
    ScanMetadataSnapshot {
        coverage_pattern_number: scan.coverage_pattern_number(),
        sweep_count: scan.sweeps().len(),
        sweeps: scan
            .sweeps()
            .iter()
            .map(|sweep| {
                let first_radial = sweep.radials().first();
                SweepMetadataSnapshot {
                    elevation_number: sweep.elevation_number(),
                    radial_count: sweep.radials().len(),
                    first_azimuth_degrees: first_radial
                        .map(|r| r.azimuth_angle_degrees())
                        .unwrap_or(0.0),
                    elevation_angle_degrees: first_radial
                        .map(|r| r.elevation_angle_degrees())
                        .unwrap_or(0.0),
                    azimuth_spacing_degrees: first_radial
                        .map(|r| r.azimuth_spacing_degrees())
                        .unwrap_or(0.0),
                    moments_present: first_radial
                        .map(|r| {
                            let mut moments = Vec::new();
                            if r.reflectivity().is_some() {
                                moments.push("REF".to_string());
                            }
                            if r.velocity().is_some() {
                                moments.push("VEL".to_string());
                            }
                            if r.spectrum_width().is_some() {
                                moments.push("SW".to_string());
                            }
                            if r.differential_reflectivity().is_some() {
                                moments.push("ZDR".to_string());
                            }
                            if r.differential_phase().is_some() {
                                moments.push("PHI".to_string());
                            }
                            if r.correlation_coefficient().is_some() {
                                moments.push("RHO".to_string());
                            }
                            if r.clutter_filter_power().is_some() {
                                moments.push("CFP".to_string());
                            }
                            moments
                        })
                        .unwrap_or_default(),
                }
            })
            .collect(),
    }
}

// =============================================================================
// MOMENT DATA STATISTICS SNAPSHOTS
// =============================================================================

/// Snapshot of moment data statistics (for regression detection).
#[derive(Serialize)]
struct MomentStatsSnapshot {
    sweep_index: usize,
    radial_index: usize,
    reflectivity: Option<MomentBlockStats>,
    velocity: Option<MomentBlockStats>,
    spectrum_width: Option<MomentBlockStats>,
    differential_reflectivity: Option<MomentBlockStats>,
}

#[derive(Serialize)]
struct MomentBlockStats {
    gate_count: u16,
    first_gate_range_km: f64,
    gate_interval_km: f64,
    data_sha256: String,
}

fn hash_moment_values(values: &[nexrad_model::data::MomentValue]) -> String {
    use nexrad_model::data::MomentValue;

    let mut hasher = Sha256::new();
    for v in values {
        match v {
            MomentValue::Value(f) => {
                hasher.update(f.to_le_bytes());
            }
            MomentValue::BelowThreshold => {
                hasher.update([0u8]);
            }
            MomentValue::RangeFolded => {
                hasher.update([1u8]);
            }
            MomentValue::CfpStatus(status) => {
                let code = match status {
                    nexrad_model::data::CfpStatus::FilterNotApplied => 0u8,
                    nexrad_model::data::CfpStatus::PointClutterFilterApplied => 1u8,
                    nexrad_model::data::CfpStatus::DualPolOnlyFilterApplied => 2u8,
                    nexrad_model::data::CfpStatus::Reserved(v) => *v,
                };
                hasher.update([2u8, code]);
            }
        }
    }
    hex::encode(hasher.finalize())
}

fn create_moment_stats(
    scan: &Scan,
    sweep_index: usize,
    radial_index: usize,
) -> MomentStatsSnapshot {
    let sweep = &scan.sweeps()[sweep_index];
    let radial = &sweep.radials()[radial_index];

    MomentStatsSnapshot {
        sweep_index,
        radial_index,
        reflectivity: radial.reflectivity().map(|m| MomentBlockStats {
            gate_count: m.gate_count(),
            first_gate_range_km: m.first_gate_range_km(),
            gate_interval_km: m.gate_interval_km(),
            data_sha256: hash_moment_values(&m.values()),
        }),
        velocity: radial.velocity().map(|m| MomentBlockStats {
            gate_count: m.gate_count(),
            first_gate_range_km: m.first_gate_range_km(),
            gate_interval_km: m.gate_interval_km(),
            data_sha256: hash_moment_values(&m.values()),
        }),
        spectrum_width: radial.spectrum_width().map(|m| MomentBlockStats {
            gate_count: m.gate_count(),
            first_gate_range_km: m.first_gate_range_km(),
            gate_interval_km: m.gate_interval_km(),
            data_sha256: hash_moment_values(&m.values()),
        }),
        differential_reflectivity: radial
            .differential_reflectivity()
            .map(|m| MomentBlockStats {
                gate_count: m.gate_count(),
                first_gate_range_km: m.first_gate_range_km(),
                gate_interval_km: m.gate_interval_km(),
                data_sha256: hash_moment_values(&m.values()),
            }),
    }
}

// =============================================================================
// METADATA SNAPSHOT TESTS
// =============================================================================

#[test]
fn test_fixture_existing_kdmx_metadata_snapshot() {
    let scan = load_scan(FIXTURE_EXISTING_KDMX);
    let snapshot = create_scan_metadata_snapshot(&scan);
    assert_yaml_snapshot!("existing_kdmx_metadata", snapshot);
}

#[test]
fn test_fixture_hurricane_harvey_metadata_snapshot() {
    let scan = load_scan(FIXTURE_HURRICANE_HARVEY);
    let snapshot = create_scan_metadata_snapshot(&scan);
    assert_yaml_snapshot!("hurricane_harvey_metadata", snapshot);
}

#[test]
fn test_fixture_modern_build_metadata_snapshot() {
    let scan = load_scan(FIXTURE_MODERN_BUILD);
    let snapshot = create_scan_metadata_snapshot(&scan);
    assert_yaml_snapshot!("modern_build_kdmx_metadata", snapshot);
}

// =============================================================================
// MOMENT DATA SNAPSHOT TESTS
// =============================================================================

#[test]
fn test_fixture_existing_kdmx_moment_stats_snapshot() {
    let scan = load_scan(FIXTURE_EXISTING_KDMX);
    // Sample first radial of first sweep
    let stats = create_moment_stats(&scan, 0, 0);
    assert_yaml_snapshot!("existing_kdmx_moment_stats", stats);
}

#[test]
fn test_fixture_hurricane_harvey_moment_stats_snapshot() {
    let scan = load_scan(FIXTURE_HURRICANE_HARVEY);
    // Sample first radial of first sweep
    let stats = create_moment_stats(&scan, 0, 0);
    assert_yaml_snapshot!("hurricane_harvey_moment_stats", stats);
}

#[test]
fn test_fixture_modern_build_moment_stats_snapshot() {
    let scan = load_scan(FIXTURE_MODERN_BUILD);
    // Sample first radial of first sweep
    let stats = create_moment_stats(&scan, 0, 0);
    assert_yaml_snapshot!("modern_build_kdmx_moment_stats", stats);
}

// =============================================================================
// CLEAR AIR PLAINS (KTLX VCP 32)
// =============================================================================

#[test]
fn test_fixture_clear_air_plains_metadata_snapshot() {
    let scan = load_scan(FIXTURE_CLEAR_AIR_PLAINS);
    let snapshot = create_scan_metadata_snapshot(&scan);
    assert_yaml_snapshot!("clear_air_plains_metadata", snapshot);
}

#[test]
fn test_fixture_clear_air_plains_moment_stats_snapshot() {
    let scan = load_scan(FIXTURE_CLEAR_AIR_PLAINS);
    let stats = create_moment_stats(&scan, 0, 0);
    assert_yaml_snapshot!("clear_air_plains_moment_stats", stats);
}

// =============================================================================
// TORNADO MAY 2019 (KTLX)
// =============================================================================

#[test]
fn test_fixture_tornado_may2019_metadata_snapshot() {
    let scan = load_scan(FIXTURE_TORNADO_MAY2019);
    let snapshot = create_scan_metadata_snapshot(&scan);
    assert_yaml_snapshot!("tornado_may2019_metadata", snapshot);
}

#[test]
fn test_fixture_tornado_may2019_moment_stats_snapshot() {
    let scan = load_scan(FIXTURE_TORNADO_MAY2019);
    let stats = create_moment_stats(&scan, 0, 0);
    assert_yaml_snapshot!("tornado_may2019_moment_stats", stats);
}

// =============================================================================
// SUPERCELL TEXAS (KFWS)
// =============================================================================

#[test]
fn test_fixture_supercell_texas_metadata_snapshot() {
    let scan = load_scan(FIXTURE_SUPERCELL_TEXAS);
    let snapshot = create_scan_metadata_snapshot(&scan);
    assert_yaml_snapshot!("supercell_texas_metadata", snapshot);
}

#[test]
fn test_fixture_supercell_texas_moment_stats_snapshot() {
    let scan = load_scan(FIXTURE_SUPERCELL_TEXAS);
    let stats = create_moment_stats(&scan, 0, 0);
    assert_yaml_snapshot!("supercell_texas_moment_stats", stats);
}

// =============================================================================
// WINTER GREAT LAKES (KMKX VCP 215)
// =============================================================================

#[test]
fn test_fixture_winter_greatlakes_metadata_snapshot() {
    let scan = load_scan(FIXTURE_WINTER_GREATLAKES);
    let snapshot = create_scan_metadata_snapshot(&scan);
    assert_yaml_snapshot!("winter_greatlakes_metadata", snapshot);
}

#[test]
fn test_fixture_winter_greatlakes_moment_stats_snapshot() {
    let scan = load_scan(FIXTURE_WINTER_GREATLAKES);
    let stats = create_moment_stats(&scan, 0, 0);
    assert_yaml_snapshot!("winter_greatlakes_moment_stats", stats);
}

// =============================================================================
// STRATIFORM NOR'EASTER (KLWX)
// =============================================================================

#[test]
fn test_fixture_stratiform_noreaster_metadata_snapshot() {
    let scan = load_scan(FIXTURE_STRATIFORM_NOREASTER);
    let snapshot = create_scan_metadata_snapshot(&scan);
    assert_yaml_snapshot!("stratiform_noreaster_metadata", snapshot);
}

#[test]
fn test_fixture_stratiform_noreaster_moment_stats_snapshot() {
    let scan = load_scan(FIXTURE_STRATIFORM_NOREASTER);
    let stats = create_moment_stats(&scan, 0, 0);
    assert_yaml_snapshot!("stratiform_noreaster_moment_stats", stats);
}
