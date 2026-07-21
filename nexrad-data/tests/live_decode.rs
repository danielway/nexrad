//! Live NEXRAD data decode integration tests.
//!
//! These tests download real data from the AWS archive bucket and verify
//! that the library can decode current production NEXRAD data correctly.
//!
//! Run with: cargo test --package nexrad-data --test live_decode --features aws -- --ignored

#![cfg(feature = "aws")]

use chrono::{Duration, Utc};
use nexrad_data::aws::archive::{download_file, list_files, Identifier};
use nexrad_data::volume;
use nexrad_decode::messages::rda_status_data::RDABuildNumber;
use nexrad_decode::messages::{decode_messages, MessageContents};

/// Maximum RDA build number the library is known to support.
/// Update this when a new build version is added to the RDABuildNumber enum.
const MAX_SUPPORTED_BUILD: RDABuildNumber = RDABuildNumber::Build24_0;

/// Sites to test against live data.
const TEST_SITES: &[&str] = &[
    "KDMX", // Des Moines, IA - Central US
    "KTLX", // Norman, OK - Tornado Alley
    "KLOT", // Chicago/Romeoville, IL - Great Lakes
    "KJAX", // Jacksonville, FL - Southeast US
    "KATX", // Seattle, WA - Pacific Northwest
];

/// Minimum number of radar data messages expected in a complete volume.
const MIN_RADAR_DATA_MESSAGES: usize = 1000;

/// Number of recent volumes to try before declaring a site failed.
///
/// The newest archived volume is occasionally anomalous for a single site (an
/// incomplete upload, a one-off decode hiccup, or a transient download error).
/// Validating the newest *and* falling back to slightly older volumes keeps a
/// single bad volume from producing a false failure, while a genuine decode
/// regression still fails every attempt.
const MAX_VOLUME_ATTEMPTS: usize = 3;

/// Result of decoding a volume file.
struct DecodeResult {
    site: String,
    file_name: String,
    build_number: RDABuildNumber,
    rda_status_count: usize,
    vcp_count: usize,
    radar_data_count: usize,
    has_volume_start: bool,
    has_volume_end: bool,
}

impl DecodeResult {
    fn print_report(&self) {
        let build_ok = self.build_number.is_known() && self.build_number <= MAX_SUPPORTED_BUILD;
        let build_status = if build_ok { "PASS" } else { "FAIL" };

        println!("=== Decode Results for {} ===", self.site);
        println!("File: {}", self.file_name);
        println!(
            "Build Number: {:?} (max supported: {:?}) - {}",
            self.build_number, MAX_SUPPORTED_BUILD, build_status
        );
        println!("RDA Status Messages: {}", self.rda_status_count);
        println!("VCP Messages: {}", self.vcp_count);
        println!("Radar Data Messages: {}", self.radar_data_count);
        println!("Volume Start Found: {}", self.has_volume_start);
        println!("Volume End Found: {}", self.has_volume_end);
        println!();
    }
}

/// List recent volume-file identifiers for a site, newest first.
///
/// Looks back across today, yesterday, and 2 days ago so a site that is briefly
/// quiet still yields candidates. MDM (metadata) files are excluded. The
/// returned order lets callers try the newest volume first and fall back to
/// older ones.
async fn list_recent_volume_files(site: &str) -> Vec<Identifier> {
    let today = Utc::now().date_naive();
    let mut candidates = Vec::new();

    for days_ago in 0..3i64 {
        let date = today - Duration::days(days_ago);

        match list_files(site, &date).await {
            Ok(files) => {
                // list_files returns keys in lexicographic (chronological) order,
                // so the newest volume for the day is last. Reverse to newest-first
                // and append; each earlier day's volumes are older than this day's.
                let mut volume_files: Vec<_> = files
                    .into_iter()
                    .filter(|f| !f.name().ends_with("_MDM"))
                    .collect();
                volume_files.reverse();
                candidates.extend(volume_files);
            }
            Err(e) => {
                eprintln!("Failed to list files for {} on {}: {}", site, date, e);
            }
        }
    }

    candidates
}

/// Decode a volume file and extract statistics.
fn decode_volume(site: &str, file_name: &str, file: &volume::File) -> Result<DecodeResult, String> {
    let mut rda_status_count = 0;
    let mut vcp_count = 0;
    let mut radar_data_count = 0;
    let mut build_number: Option<RDABuildNumber> = None;
    let mut has_volume_start = false;
    let mut has_volume_end = false;

    let records: Vec<_> = file.records().expect("records").into_iter().collect();
    if records.is_empty() {
        return Err("No records found in volume".to_string());
    }

    for mut record in records {
        if record.compressed() {
            record = record
                .decompress()
                .map_err(|e| format!("Decompression failed: {}", e))?;
        }

        let messages =
            decode_messages(record.data()).map_err(|e| format!("Message decode failed: {}", e))?;

        for message in messages {
            match message.contents() {
                MessageContents::RDAStatusData(status) => {
                    rda_status_count += 1;
                    if build_number.is_none() {
                        build_number = Some(status.build_number());
                    }
                }
                MessageContents::VolumeCoveragePattern(_) => {
                    vcp_count += 1;
                }
                MessageContents::DigitalRadarData(radar_data) => {
                    radar_data_count += 1;
                    // Check for scan boundaries
                    match radar_data.header().radial_status_raw() {
                        3 => has_volume_start = true, // ScanStart
                        4 => has_volume_end = true,   // ScanEnd
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    let build_number =
        build_number.ok_or_else(|| "No RDA Status Data message found".to_string())?;

    Ok(DecodeResult {
        site: site.to_string(),
        file_name: file_name.to_string(),
        build_number,
        rda_status_count,
        vcp_count,
        radar_data_count,
        has_volume_start,
        has_volume_end,
    })
}

/// Validate a decoded volume, returning a list of human-readable failure
/// reasons. An empty list means the volume passed every check.
fn validation_failures(result: &DecodeResult) -> Vec<String> {
    let mut failures = Vec::new();

    // Build number check
    if !result.build_number.is_known() {
        failures.push(format!(
            "Unknown build number: {:?}. Add a new variant to RDABuildNumber.",
            result.build_number
        ));
    } else if result.build_number > MAX_SUPPORTED_BUILD {
        failures.push(format!(
            "Build {:?} exceeds MAX_SUPPORTED_BUILD ({:?}). \
             Update MAX_SUPPORTED_BUILD in live_decode.rs if this is expected.",
            result.build_number, MAX_SUPPORTED_BUILD
        ));
    }

    // RDA Status check
    if result.rda_status_count == 0 {
        failures.push("No RDA Status Data messages found".to_string());
    }

    // VCP check
    if result.vcp_count == 0 {
        failures.push("No Volume Coverage Pattern messages found".to_string());
    }

    // Radar data count check
    if result.radar_data_count < MIN_RADAR_DATA_MESSAGES {
        failures.push(format!(
            "Insufficient radar data messages: {} (expected >= {})",
            result.radar_data_count, MIN_RADAR_DATA_MESSAGES
        ));
    }

    // Volume boundary check
    if !result.has_volume_start {
        failures.push("Volume scan start marker not found".to_string());
    }
    if !result.has_volume_end {
        failures.push("Volume scan end marker not found".to_string());
    }

    failures
}

/// Emit GitHub Actions error annotations for build-number problems so they
/// surface prominently in the workflow UI. Only build issues get annotations
/// since they signal that the library itself needs updating.
fn emit_build_annotations(site: &str, result: &DecodeResult) {
    if !result.build_number.is_known() {
        println!(
            "::error file=nexrad-data/tests/live_decode.rs,title=Unknown Build::\
             Site {} reported unknown build {:?}. Add new variant to RDABuildNumber enum.",
            site, result.build_number
        );
    } else if result.build_number > MAX_SUPPORTED_BUILD {
        println!(
            "::error file=nexrad-data/tests/live_decode.rs,title=Build Exceeds Max::\
             Site {} build {:?} exceeds MAX_SUPPORTED_BUILD {:?}. Update the constant.",
            site, result.build_number, MAX_SUPPORTED_BUILD
        );
    }
}

/// Run the live decode test for a specific site.
///
/// Tries the most recent volumes in turn (see [`MAX_VOLUME_ATTEMPTS`]) and
/// passes as soon as one fully validates. A single anomalous volume is skipped
/// rather than failing the site; a genuine decode regression fails every
/// attempt and is reported with the details from the last one.
async fn run_site_test(site: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Testing site: {}", site);
    println!("{}\n", "=".repeat(60));

    let candidates = list_recent_volume_files(site).await;
    if candidates.is_empty() {
        panic!(
            "SKIP: No data available for {} in the last 3 days. \
             Site may be under maintenance.",
            site
        );
    }

    let mut attempts = 0;
    let mut last_failure: Option<(DecodeResult, Vec<String>)> = None;

    for identifier in candidates.iter().take(MAX_VOLUME_ATTEMPTS) {
        attempts += 1;
        let file_name = identifier.name().to_string();

        let file = match download_file(identifier.clone()).await {
            Ok(file) => file,
            Err(e) => {
                eprintln!(
                    "Attempt {}: failed to download {}: {}",
                    attempts, file_name, e
                );
                continue;
            }
        };

        let result = match decode_volume(site, &file_name, &file) {
            Ok(result) => result,
            Err(e) => {
                eprintln!(
                    "Attempt {}: decode error for {}: {}",
                    attempts, file_name, e
                );
                continue;
            }
        };

        result.print_report();

        let failures = validation_failures(&result);
        if failures.is_empty() {
            println!(
                "PASS: {} decoded successfully from {} (attempt {})\n",
                site, file_name, attempts
            );
            return;
        }

        println!(
            "Attempt {} ({}) did not fully validate; trying an older volume if available:\n  - {}",
            attempts,
            file_name,
            failures.join("\n  - ")
        );
        last_failure = Some((result, failures));
    }

    // No recent volume fully validated.
    match last_failure {
        Some((result, failures)) => {
            emit_build_annotations(site, &result);
            panic!(
                "FAIL: {} - no recent volume fully validated after {} attempt(s). \
                 Last volume's failures:\n  - {}",
                site,
                attempts,
                failures.join("\n  - ")
            );
        }
        None => {
            panic!(
                "FAIL: {} - could not download or decode any of the {} most recent volume(s).",
                site, attempts
            );
        }
    }
}

// Individual test functions for each site (allows matrix strategy in CI)

#[tokio::test]
#[ignore = "requires AWS access - run weekly"]
async fn test_live_decode_kdmx() {
    run_site_test("KDMX").await;
}

#[tokio::test]
#[ignore = "requires AWS access - run weekly"]
async fn test_live_decode_ktlx() {
    run_site_test("KTLX").await;
}

#[tokio::test]
#[ignore = "requires AWS access - run weekly"]
async fn test_live_decode_klot() {
    run_site_test("KLOT").await;
}

#[tokio::test]
#[ignore = "requires AWS access - run weekly"]
async fn test_live_decode_kjax() {
    run_site_test("KJAX").await;
}

#[tokio::test]
#[ignore = "requires AWS access - run weekly"]
async fn test_live_decode_katx() {
    run_site_test("KATX").await;
}

/// Combined test for local development - runs all sites sequentially.
#[tokio::test]
#[ignore = "requires AWS access - run weekly"]
async fn test_live_decode_all_sites() {
    for site in TEST_SITES {
        run_site_test(site).await;
    }
    println!("\nAll sites decoded successfully!");
}
