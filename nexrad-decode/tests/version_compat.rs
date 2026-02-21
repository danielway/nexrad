//! Version compatibility integration tests.
//!
//! These tests verify that the decoder correctly handles NEXRAD data from different
//! RDA build versions and message formats:
//!
//! - Message Type 1 (legacy): fixed 2432-byte frames, 1 byte/gate, used 1991–2008
//! - Message Type 31 (modern): variable-length, 1-2 bytes/gate, used 2008–present
//! - Volume Data Block (VOL): expanded from 44 to 52 bytes at Build 20.0 (July 2021)
//! - Radial Data Block (RAD): expanded from 20 to 28 bytes at Build 12.0 (July 2011)
//!
//! This test exercises both legacy and modern code paths with real archive data
//! spanning the earliest available NEXRAD data (June 1991) through Build 24.0 (2026).

use nexrad_data::volume;
use nexrad_decode::messages::rda_status_data::RDABuildNumber;
use nexrad_decode::messages::{decode_messages, MessageContents};

/// Helper: decode a volume file and extract analysis data.
fn analyze_volume(data: &[u8]) -> VolumeAnalysis {
    let file = volume::File::new(data.to_vec())
        .decompress()
        .expect("decompresses gzip file");

    let mut build_number: Option<RDABuildNumber> = None;
    let mut raw_build_value: Option<u16> = None;
    let mut legacy_vol_count: usize = 0;
    let mut modern_vol_count: usize = 0;
    let mut total_radials: usize = 0;
    let mut has_zdr = false;
    let mut has_phi = false;
    let mut has_rho = false;
    let mut legacy_rad_count: usize = 0;
    let mut modern_rad_count: usize = 0;
    let mut type1_radials: usize = 0;
    let mut type1_has_reflectivity = false;
    let mut type1_has_velocity = false;
    let mut type1_has_spectrum_width = false;

    let records = file.records().expect("records");
    for mut record in records {
        if record.compressed() {
            record = record.decompress().expect("decompresses record");
        }

        let messages = decode_messages(record.data()).expect("decodes messages");

        for message in &messages {
            match message.contents() {
                MessageContents::RDAStatusData(status) => {
                    if build_number.is_none() {
                        build_number = Some(status.build_number());
                        raw_build_value = Some(status.raw_rda_build_number());
                    }
                }
                MessageContents::DigitalRadarData(radar_data) => {
                    total_radials += 1;

                    if let Some(vol_block) = radar_data.volume_data_block() {
                        if vol_block.inner().is_legacy() {
                            legacy_vol_count += 1;
                        } else {
                            modern_vol_count += 1;
                        }
                    }

                    if let Some(rad_block) = radar_data.radial_data_block() {
                        if rad_block.inner().is_legacy() {
                            legacy_rad_count += 1;
                        } else {
                            modern_rad_count += 1;
                        }
                    }

                    if radar_data.differential_reflectivity_data_block().is_some() {
                        has_zdr = true;
                    }
                    if radar_data.differential_phase_data_block().is_some() {
                        has_phi = true;
                    }
                    if radar_data.correlation_coefficient_data_block().is_some() {
                        has_rho = true;
                    }
                }
                MessageContents::DigitalRadarDataLegacy(legacy_data) => {
                    type1_radials += 1;

                    if legacy_data.reflectivity_gates().is_some() {
                        type1_has_reflectivity = true;
                    }
                    if legacy_data.velocity_gates().is_some() {
                        type1_has_velocity = true;
                    }
                    if legacy_data.spectrum_width_gates().is_some() {
                        type1_has_spectrum_width = true;
                    }
                }
                _ => {}
            }
        }
    }

    VolumeAnalysis {
        build_number,
        raw_build_value,
        legacy_vol_count,
        modern_vol_count,
        legacy_rad_count,
        modern_rad_count,
        total_radials,
        has_zdr,
        has_phi,
        has_rho,
        type1_radials,
        type1_has_reflectivity,
        type1_has_velocity,
        type1_has_spectrum_width,
    }
}

struct VolumeAnalysis {
    build_number: Option<RDABuildNumber>,
    raw_build_value: Option<u16>,
    legacy_vol_count: usize,
    modern_vol_count: usize,
    legacy_rad_count: usize,
    modern_rad_count: usize,
    total_radials: usize,
    has_zdr: bool,
    has_phi: bool,
    has_rho: bool,
    type1_radials: usize,
    type1_has_reflectivity: bool,
    type1_has_velocity: bool,
    type1_has_spectrum_width: bool,
}

// -- Pre-2008 Message Type 1 tests (legacy digital radar data) --

/// KTLX 1991-06-05: Original WSR-88D deployment, Message Type 1 only.
/// This is the earliest available NEXRAD data on AWS (June 5, 1991).
const KTLX_1991: &[u8] = include_bytes!("../../downloads/KTLX19910605_162126.gz");

/// KABR 2005-01-01: Pre-Build 10.0, Message Type 1 format, AR2V0001.
const KABR_2005: &[u8] = include_bytes!("../../downloads/KABR20050101_000745.gz");

#[test]
fn test_message_type1_ktlx_1991() {
    let analysis = analyze_volume(KTLX_1991);

    // 1991 files use Message Type 1 exclusively — no Type 31
    assert_eq!(
        analysis.total_radials, 0,
        "1991 file should have no Type 31 radials"
    );
    assert!(
        analysis.type1_radials > 1000,
        "1991 file should have many Type 1 radials, got {}",
        analysis.type1_radials
    );

    // Should have reflectivity data
    assert!(
        analysis.type1_has_reflectivity,
        "1991 file should have reflectivity data"
    );

    // No dual-pol in 1991 (predates dual-pol by ~20 years)
    assert!(!analysis.has_zdr);
    assert!(!analysis.has_phi);
    assert!(!analysis.has_rho);
}

#[test]
fn test_message_type1_kabr_2005() {
    let analysis = analyze_volume(KABR_2005);

    // 2005 files use Message Type 1 — no Type 31 (added at Build 10.0 in 2008)
    assert_eq!(
        analysis.total_radials, 0,
        "2005 file should have no Type 31 radials"
    );
    assert!(
        analysis.type1_radials > 1000,
        "2005 file should have many Type 1 radials, got {}",
        analysis.type1_radials
    );

    // Should have reflectivity data
    assert!(
        analysis.type1_has_reflectivity,
        "2005 file should have reflectivity data"
    );

    // Should have velocity and spectrum width data
    assert!(
        analysis.type1_has_velocity,
        "2005 file should have velocity data"
    );
    assert!(
        analysis.type1_has_spectrum_width,
        "2005 file should have spectrum width data"
    );

    // No dual-pol in 2005
    assert!(!analysis.has_zdr);
    assert!(!analysis.has_phi);
    assert!(!analysis.has_rho);
}

/// Validates Message Type 1 field values are physically reasonable.
#[test]
fn test_message_type1_field_values() {
    let file = volume::File::new(KABR_2005.to_vec())
        .decompress()
        .expect("decompresses gzip file");
    let records = file.records().expect("records");

    let mut checked = false;
    for mut record in records {
        if record.compressed() {
            record = record.decompress().expect("decompresses record");
        }

        let messages = decode_messages(record.data()).expect("decodes messages");
        for message in &messages {
            if let MessageContents::DigitalRadarDataLegacy(drd) = message.contents() {
                // Azimuth should be 0-360
                let az = drd.azimuth_angle();
                assert!((0.0..360.0).contains(&az), "azimuth {az} out of range");

                // Elevation should be reasonable (-1 to 60 degrees)
                let el = drd.elevation_angle();
                assert!((-1.0..60.0).contains(&el), "elevation {el} out of range");

                // Elevation number should be 1-25
                let elev_num = drd.elevation_number();
                assert!(
                    (1..=25).contains(&elev_num),
                    "elevation number {elev_num} out of range"
                );

                // VCP should be a valid number
                let vcp = drd.vcp_number();
                assert!(vcp > 0, "VCP number should be positive, got {vcp}");

                // Gate counts should be reasonable
                let surv = drd.num_surveillance_gates();
                let dopp = drd.num_doppler_gates();
                assert!(surv <= 460, "surveillance gates {surv} exceeds max 460");
                assert!(dopp <= 920, "doppler gates {dopp} exceeds max 920");
                assert!(
                    surv > 0 || dopp > 0,
                    "radial should have at least one type of gate data"
                );

                // Unambiguous range should be positive
                let range = drd.unambiguous_range_km();
                assert!(range > 0.0, "unambiguous range should be positive");

                checked = true;
                if elev_num >= 3 {
                    break; // Checked enough
                }
            }
        }
        if checked {
            break;
        }
    }
    assert!(checked, "should have checked at least one Type 1 message");
}

// -- Legacy format tests (gzip-compressed, CTM frames) --

/// KTLX 2010-05-01: Legacy CTM format (V03), gzip-compressed on disk.
/// Exercises gzip decompression, CTM frame parsing, and legacy RAD block detection.
const KTLX_2010: &[u8] = include_bytes!("../../downloads/KTLX20100501_000444_V03.gz");

#[test]
fn test_legacy_ctm_ktlx_2010() {
    let analysis = analyze_volume(KTLX_2010);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        build.uses_legacy_volume_data_block(),
        "KTLX 2010 should use legacy VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert!(analysis.legacy_vol_count > 0);
    assert_eq!(analysis.modern_vol_count, 0);

    assert!(
        analysis.legacy_rad_count > 0,
        "KTLX 2010 should have legacy RAD blocks (lrtup=20)"
    );
    assert_eq!(
        analysis.modern_rad_count, 0,
        "KTLX 2010 should not have modern RAD blocks"
    );

    assert!(
        analysis.total_radials > 100,
        "should have significant radar data, got {}",
        analysis.total_radials
    );
}

/// KTLX 2014-01-01: Legacy CTM format (V06), gzip-compressed on disk.
const KTLX_2014: &[u8] = include_bytes!("../../downloads/KTLX20140101_000624_V06.gz");

#[test]
fn test_legacy_ctm_ktlx_2014() {
    let analysis = analyze_volume(KTLX_2014);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        build.uses_legacy_volume_data_block(),
        "KTLX 2014 should use legacy VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert!(analysis.legacy_vol_count > 0);
    assert_eq!(analysis.modern_vol_count, 0);

    assert!(
        analysis.legacy_rad_count > 0,
        "KTLX 2014 should have legacy RAD blocks (lrtup=20)"
    );
    assert_eq!(
        analysis.modern_rad_count, 0,
        "KTLX 2014 should not have modern RAD blocks"
    );

    assert!(
        analysis.total_radials > 100,
        "should have significant radar data, got {}",
        analysis.total_radials
    );
}

// -- Legacy VOL block tests (Build 17.0–19.0, LDM format) --

/// KCRP 2017-08-26: Build 17.0, legacy VOL block (44 bytes).
const KCRP_2017: &[u8] = include_bytes!("../../downloads/KCRP20170826_044114_V06");

#[test]
fn test_legacy_vol_block_kcrp_2017() {
    let analysis = analyze_volume(KCRP_2017);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        build.uses_legacy_volume_data_block(),
        "KCRP 2017 should use legacy VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert!(
        analysis.legacy_vol_count > 0,
        "should have parsed legacy VOL blocks"
    );
    assert_eq!(
        analysis.modern_vol_count, 0,
        "should not have parsed any modern VOL blocks"
    );
    assert!(
        analysis.total_radials > 1000,
        "should have significant radar data, got {}",
        analysis.total_radials
    );
}

/// KLWX 2018-03-02: Build 18.0, legacy VOL block (44 bytes).
const KLWX_2018: &[u8] = include_bytes!("../../downloads/KLWX20180302_115347_V06");

#[test]
fn test_legacy_vol_block_klwx_2018() {
    let analysis = analyze_volume(KLWX_2018);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        build.uses_legacy_volume_data_block(),
        "KLWX 2018 should use legacy VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert!(analysis.legacy_vol_count > 0);
    assert_eq!(analysis.modern_vol_count, 0);
}

/// KTLX 2019-07-15: Build 19.0, legacy VOL block (44 bytes).
/// This is the last build before the VOL block expansion.
const KTLX_2019: &[u8] = include_bytes!("../../downloads/KTLX20190715_120037_V06");

#[test]
fn test_legacy_vol_block_ktlx_2019_last_legacy_build() {
    let analysis = analyze_volume(KTLX_2019);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        build.uses_legacy_volume_data_block(),
        "KTLX 2019 should use legacy VOL block (Build 19.0 is last legacy), got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert!(analysis.legacy_vol_count > 0);
    assert_eq!(analysis.modern_vol_count, 0);
}

// -- Modern VOL block tests (Build 20.0 and later) --

/// KDMX 2022-03-05: Build 20.0+, modern VOL block (52 bytes).
const KDMX_2022: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

#[test]
fn test_modern_vol_block_kdmx_2022() {
    let analysis = analyze_volume(KDMX_2022);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        !build.uses_legacy_volume_data_block(),
        "KDMX 2022 should use modern VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert_eq!(
        analysis.legacy_vol_count, 0,
        "should not have parsed any legacy VOL blocks"
    );
    assert!(
        analysis.modern_vol_count > 0,
        "should have parsed modern VOL blocks"
    );
    assert!(
        analysis.total_radials > 1000,
        "should have significant radar data, got {}",
        analysis.total_radials
    );
}

/// KDMX 2025-01-01: Build 21.0+, modern VOL block (52 bytes).
const KDMX_2025: &[u8] = include_bytes!("../../downloads/KDMX20250101_225441_V06");

#[test]
fn test_modern_vol_block_kdmx_2025() {
    let analysis = analyze_volume(KDMX_2025);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        !build.uses_legacy_volume_data_block(),
        "KDMX 2025 should use modern VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert_eq!(analysis.legacy_vol_count, 0);
    assert!(analysis.modern_vol_count > 0);
}

/// KDMX 2026-01-01: Latest available, modern VOL block (52 bytes).
const KDMX_2026: &[u8] = include_bytes!("../../downloads/KDMX20260101_154351_V06");

#[test]
fn test_modern_vol_block_kdmx_2026() {
    let analysis = analyze_volume(KDMX_2026);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        !build.uses_legacy_volume_data_block(),
        "KDMX 2026 should use modern VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert_eq!(analysis.legacy_vol_count, 0);
    assert!(analysis.modern_vol_count > 0);
}

// -- Build 19→20 transition era tests --

/// KCRP 2020-01-01: Should still be Build 19.0 (legacy).
const KCRP_2020: &[u8] = include_bytes!("../../downloads/KCRP20200101_000431_V06");

#[test]
fn test_transition_era_kcrp_2020() {
    let analysis = analyze_volume(KCRP_2020);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        build.uses_legacy_volume_data_block(),
        "KCRP 2020-01-01 should use legacy VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert!(analysis.legacy_vol_count > 0);
    assert_eq!(analysis.modern_vol_count, 0);
}

/// KCRP 2021-01-01: Transition era - could be Build 19 or 20 depending on site update schedule.
const KCRP_2021_JAN: &[u8] = include_bytes!("../../downloads/KCRP20210101_000031_V06");

#[test]
fn test_transition_era_kcrp_2021_jan() {
    let analysis = analyze_volume(KCRP_2021_JAN);

    let build = analysis.build_number.expect("should have build number");

    // Regardless of which build this site is on, the VOL block format
    // should be consistent: build number determination and actual parsing
    // should agree.
    if build.uses_legacy_volume_data_block() {
        assert!(
            analysis.legacy_vol_count > 0,
            "build {:?} says legacy but no legacy VOL blocks found",
            build
        );
        assert_eq!(
            analysis.modern_vol_count, 0,
            "build {:?} says legacy but found modern VOL blocks",
            build
        );
    } else {
        assert_eq!(
            analysis.legacy_vol_count, 0,
            "build {:?} says modern but found legacy VOL blocks",
            build
        );
        assert!(
            analysis.modern_vol_count > 0,
            "build {:?} says modern but no modern VOL blocks found",
            build
        );
    }
}

/// KCRP 2021-08-01: Mid-transition era — Build 20.0 was released July 2021.
const KCRP_2021_AUG: &[u8] = include_bytes!("../../downloads/KCRP20210801_000338_V06");

/// KCRP 2021-12-31: Late transition era — most sites should be on Build 20.0+ by now.
const KCRP_2021_DEC: &[u8] = include_bytes!("../../downloads/KCRP20211231_000526_V06");

/// KCRP 2022-01-01: Should be Build 20.0+ (modern).
const KCRP_2022: &[u8] = include_bytes!("../../downloads/KCRP20220101_000413_V06");

/// KCRP 2021-08-01: This captures the transition era just after Build 20.0 release.
#[test]
fn test_transition_era_kcrp_2021_aug() {
    let analysis = analyze_volume(KCRP_2021_AUG);

    let build = analysis.build_number.expect("should have build number");

    // VOL block format should be consistent with build number
    if build.uses_legacy_volume_data_block() {
        assert!(analysis.legacy_vol_count > 0);
        assert_eq!(analysis.modern_vol_count, 0);
    } else {
        assert_eq!(analysis.legacy_vol_count, 0);
        assert!(analysis.modern_vol_count > 0);
    }
}

/// KCRP 2021-12-31: KCRP upgraded to Build 20.0 between Aug and Dec 2021.
#[test]
fn test_transition_era_kcrp_2021_dec() {
    let analysis = analyze_volume(KCRP_2021_DEC);

    let build = analysis.build_number.expect("should have build number");

    // VOL block format should be consistent with build number
    if build.uses_legacy_volume_data_block() {
        assert!(analysis.legacy_vol_count > 0);
        assert_eq!(analysis.modern_vol_count, 0);
    } else {
        assert_eq!(analysis.legacy_vol_count, 0);
        assert!(analysis.modern_vol_count > 0);
    }
}

#[test]
fn test_transition_era_kcrp_2022() {
    let analysis = analyze_volume(KCRP_2022);

    let build = analysis.build_number.expect("should have build number");
    assert!(
        !build.uses_legacy_volume_data_block(),
        "KCRP 2022-01-01 should use modern VOL block, got build {:?} (raw: {:?})",
        build,
        analysis.raw_build_value
    );
    assert_eq!(analysis.legacy_vol_count, 0);
    assert!(analysis.modern_vol_count > 0);
}

// -- Build number consistency validation --

/// Validates that build number decoding is consistent across all available test volumes.
/// This test extracts the raw build number field and verifies:
/// 1. The scaling factor (÷100 vs ÷10) produces a reasonable build number
/// 2. The decoded build number is a known variant
/// 3. The build number is consistent with the file's date
/// 4. The VOL block format detected matches the build number's expectation
#[test]
fn test_build_number_consistency_all_volumes() {
    struct TestCase {
        name: &'static str,
        data: &'static [u8],
        min_expected_build: f32,
        max_expected_build: f32,
        expect_legacy_vol: bool,
    }

    let test_cases = [
        TestCase {
            name: "KTLX 2010",
            data: KTLX_2010,
            min_expected_build: 11.0,
            max_expected_build: 14.0,
            expect_legacy_vol: true,
        },
        TestCase {
            name: "KTLX 2014",
            data: KTLX_2014,
            min_expected_build: 12.0,
            max_expected_build: 16.0,
            expect_legacy_vol: true,
        },
        TestCase {
            name: "KCRP 2017",
            data: KCRP_2017,
            min_expected_build: 14.0,
            max_expected_build: 19.0,
            expect_legacy_vol: true,
        },
        TestCase {
            name: "KLWX 2018",
            data: KLWX_2018,
            min_expected_build: 17.0,
            max_expected_build: 19.0,
            expect_legacy_vol: true,
        },
        TestCase {
            name: "KTLX 2019",
            data: KTLX_2019,
            min_expected_build: 18.0,
            max_expected_build: 19.0,
            expect_legacy_vol: true,
        },
        TestCase {
            name: "KCRP 2020",
            data: KCRP_2020,
            min_expected_build: 17.0,
            max_expected_build: 20.0,
            expect_legacy_vol: true,
        },
        TestCase {
            name: "KCRP 2021 Aug",
            data: KCRP_2021_AUG,
            min_expected_build: 18.0,
            max_expected_build: 21.0,
            expect_legacy_vol: true, // KCRP still on Build 19.0 in Aug 2021
        },
        TestCase {
            name: "KCRP 2021 Dec",
            data: KCRP_2021_DEC,
            min_expected_build: 20.0,
            max_expected_build: 21.0,
            expect_legacy_vol: false, // KCRP upgraded to Build 20.0 between Aug-Dec 2021
        },
        TestCase {
            name: "KDMX 2022",
            data: KDMX_2022,
            min_expected_build: 20.0,
            max_expected_build: 24.0,
            expect_legacy_vol: false,
        },
        TestCase {
            name: "KDMX 2025",
            data: KDMX_2025,
            min_expected_build: 20.0,
            max_expected_build: 25.0,
            expect_legacy_vol: false,
        },
        TestCase {
            name: "KDMX 2026",
            data: KDMX_2026,
            min_expected_build: 20.0,
            max_expected_build: 25.0,
            expect_legacy_vol: false,
        },
        TestCase {
            name: "KCRP 2022",
            data: KCRP_2022,
            min_expected_build: 20.0,
            max_expected_build: 24.0,
            expect_legacy_vol: false,
        },
    ];

    for tc in &test_cases {
        let analysis = analyze_volume(tc.data);

        let build = analysis
            .build_number
            .unwrap_or_else(|| panic!("{}: should have build number", tc.name));
        let build_float = build.as_float();

        // Verify build number is in expected range for this file's date
        assert!(
            build_float >= tc.min_expected_build && build_float <= tc.max_expected_build,
            "{}: build number {:.1} (raw: {:?}) outside expected range [{:.1}, {:.1}]",
            tc.name,
            build_float,
            analysis.raw_build_value,
            tc.min_expected_build,
            tc.max_expected_build
        );

        // Verify build number is a known variant (not Unknown) for builds >= 12.0.
        // Older builds (e.g. 11.3) predate our enum and are expected to be Unknown.
        if build_float >= 12.0 {
            assert!(
                build.is_known(),
                "{}: build number should be known, got {:?} (raw: {:?})",
                tc.name,
                build,
                analysis.raw_build_value
            );
        }

        // Verify VOL block format matches build number expectation
        assert_eq!(
            build.uses_legacy_volume_data_block(),
            tc.expect_legacy_vol,
            "{}: VOL block format mismatch for build {:?}",
            tc.name,
            build
        );

        // Verify actual parsed VOL blocks match expectation
        if tc.expect_legacy_vol {
            assert!(
                analysis.legacy_vol_count > 0,
                "{}: expected legacy VOL blocks but found none",
                tc.name
            );
            assert_eq!(
                analysis.modern_vol_count, 0,
                "{}: expected no modern VOL blocks",
                tc.name
            );
        } else {
            assert_eq!(
                analysis.legacy_vol_count, 0,
                "{}: expected no legacy VOL blocks",
                tc.name
            );
            assert!(
                analysis.modern_vol_count > 0,
                "{}: expected modern VOL blocks but found none",
                tc.name
            );
        }

        // Verify reasonable radial count
        assert!(
            analysis.total_radials > 100,
            "{}: expected at least 100 radials, got {}",
            tc.name,
            analysis.total_radials
        );
    }
}

/// Verifies the zdr_bias_estimate_weighted_mean field is correctly handled
/// for both legacy (None) and modern (Some) VOL blocks.
#[test]
fn test_vol_block_zdr_bias_field_by_build() {
    // Legacy: field should be None
    let legacy_analysis = analyze_volume_first_vol_block(KCRP_2017);
    assert!(
        legacy_analysis.zdr_bias.is_none(),
        "legacy VOL block should not have zdr_bias_estimate_weighted_mean"
    );
    assert!(
        legacy_analysis.is_legacy,
        "KCRP 2017 VOL block should be legacy"
    );
    assert_eq!(
        legacy_analysis.lrtup, 44,
        "legacy VOL block lrtup should be 44"
    );

    // Modern: field should be Some
    let modern_analysis = analyze_volume_first_vol_block(KDMX_2022);
    assert!(
        modern_analysis.zdr_bias.is_some(),
        "modern VOL block should have zdr_bias_estimate_weighted_mean"
    );
    assert!(
        !modern_analysis.is_legacy,
        "KDMX 2022 VOL block should be modern"
    );
    assert_eq!(
        modern_analysis.lrtup, 52,
        "modern VOL block lrtup should be 52"
    );
}

struct VolBlockAnalysis {
    is_legacy: bool,
    lrtup: u16,
    zdr_bias: Option<u16>,
}

/// Helper: extract the first VOL block from a volume file.
fn analyze_volume_first_vol_block(data: &[u8]) -> VolBlockAnalysis {
    let file = volume::File::new(data.to_vec())
        .decompress()
        .expect("decompresses gzip file");
    let records = file.records().expect("records");

    for mut record in records {
        if record.compressed() {
            record = record.decompress().expect("decompresses record");
        }

        let messages = decode_messages(record.data()).expect("decodes messages");

        for message in &messages {
            if let MessageContents::DigitalRadarData(radar_data) = message.contents() {
                if let Some(vol_block) = radar_data.volume_data_block() {
                    let inner = vol_block.inner();
                    return VolBlockAnalysis {
                        is_legacy: inner.is_legacy(),
                        lrtup: inner.lrtup_raw(),
                        zdr_bias: inner.zdr_bias_estimate_weighted_mean(),
                    };
                }
            }
        }
    }

    panic!("no VOL block found in volume file");
}

/// Verifies that all dual-pol data blocks (ZDR, PHI, RHO) are present in
/// volumes from all build eras. Dual-pol data was added at Build 12.0 and
/// should be present in all our test volumes.
#[test]
fn test_dual_pol_data_blocks_all_eras() {
    // Note: KTLX 2010 is excluded because Build 11.x predates dual-pol deployment.
    let volumes: &[(&str, &[u8])] = &[
        ("KTLX 2014 (legacy VOL)", KTLX_2014),
        ("KCRP 2017 (legacy VOL)", KCRP_2017),
        ("KLWX 2018 (legacy VOL)", KLWX_2018),
        ("KTLX 2019 (legacy VOL)", KTLX_2019),
        ("KDMX 2022 (modern VOL)", KDMX_2022),
        ("KDMX 2025 (modern VOL)", KDMX_2025),
    ];

    for (name, data) in volumes {
        let analysis = analyze_volume(data);
        assert!(
            analysis.has_zdr,
            "{}: should have differential reflectivity (ZDR) data",
            name
        );
        assert!(
            analysis.has_phi,
            "{}: should have differential phase (PHI) data",
            name
        );
        assert!(
            analysis.has_rho,
            "{}: should have correlation coefficient (RHO) data",
            name
        );
    }
}

/// Verifies that the RAD block format (legacy vs modern) is correctly detected
/// across eras. The RAD block expanded from 20 to 28 bytes at Build 12.0
/// (ICD 2620002K) for dual polarization.
#[test]
fn test_rad_block_format_across_eras() {
    // These files have legacy RAD blocks (lrtup=20, no calibration constants).
    let legacy_volumes: &[(&str, &[u8])] = &[("KTLX 2010", KTLX_2010), ("KTLX 2014", KTLX_2014)];

    for (name, data) in legacy_volumes {
        let analysis = analyze_volume(data);
        assert!(
            analysis.legacy_rad_count > 0,
            "{}: should have legacy RAD blocks (lrtup=20)",
            name
        );
        assert_eq!(
            analysis.modern_rad_count, 0,
            "{}: should not have modern RAD blocks",
            name
        );
    }

    // These files have modern RAD blocks (lrtup=28, with calibration constants).
    let modern_volumes: &[(&str, &[u8])] = &[
        ("KCRP 2017", KCRP_2017),
        ("KLWX 2018", KLWX_2018),
        ("KTLX 2019", KTLX_2019),
        ("KDMX 2022", KDMX_2022),
        ("KDMX 2025", KDMX_2025),
    ];

    for (name, data) in modern_volumes {
        let analysis = analyze_volume(data);
        assert_eq!(
            analysis.legacy_rad_count, 0,
            "{}: should not have legacy RAD blocks",
            name
        );
        assert!(
            analysis.modern_rad_count > 0,
            "{}: should have modern RAD blocks (lrtup=28)",
            name
        );
    }
}

/// Verifies the RDABuildNumber scaling logic produces correct results for
/// known raw values.
#[test]
fn test_build_number_scaling() {
    // Build 19.0: raw value 190 (190/100 = 1.9 < 2.0, so 190/10 = 19.0)
    assert_eq!(
        RDABuildNumber::from_raw(190),
        RDABuildNumber::Build19_0,
        "raw 190 should decode to Build 19.0"
    );

    // Build 20.0: raw value 200 (200/100 = 2.0, not > 2.0, so 200/10 = 20.0)
    assert_eq!(
        RDABuildNumber::from_raw(200),
        RDABuildNumber::Build20_0,
        "raw 200 should decode to Build 20.0"
    );

    // Build 21.0: raw value 2100 (2100/100 = 21.0 > 2.0, so 2100/100 = 21.0)
    assert_eq!(
        RDABuildNumber::from_raw(2100),
        RDABuildNumber::Build21_0,
        "raw 2100 should decode to Build 21.0"
    );

    // Build 24.0: raw value 2400 (2400/100 = 24.0 > 2.0, so 2400/100 = 24.0)
    assert_eq!(
        RDABuildNumber::from_raw(2400),
        RDABuildNumber::Build24_0,
        "raw 2400 should decode to Build 24.0"
    );

    // Build 12.0: raw value 120 (120/100 = 1.2 < 2.0, so 120/10 = 12.0)
    assert_eq!(
        RDABuildNumber::from_raw(120),
        RDABuildNumber::Build12_0,
        "raw 120 should decode to Build 12.0"
    );

    // Verify legacy boundary
    assert!(RDABuildNumber::Build19_0.uses_legacy_volume_data_block());
    assert!(!RDABuildNumber::Build20_0.uses_legacy_volume_data_block());
}
