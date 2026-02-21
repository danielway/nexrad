//! Utility test to extract individual message type fixtures from volume files.
//!
//! Run with: cargo test --all-features --test extract_fixtures -- --ignored --nocapture

use nexrad_data::volume;
use nexrad_decode::messages::decode_messages;
use std::collections::HashMap;
use std::fs;

/// Scans multiple volume files for all message types and extracts fixtures.
#[test]
#[ignore]
fn extract_message_fixtures() {
    let volume_files: Vec<(&str, &[u8])> = vec![
        (
            "KABR2005-legacy",
            include_bytes!("../../downloads/KABR20050101_000745.gz"),
        ),
        (
            "KDMX2022",
            include_bytes!("../../downloads/KDMX20220305_232324_V06"),
        ),
        (
            "KDMX2025-modern",
            include_bytes!("../../downloads/KDMX20250314_175512_V06"),
        ),
        (
            "KDMX2026",
            include_bytes!("../../downloads/KDMX20260101_154351_V06"),
        ),
    ];

    // Message types we already have fixtures for
    let skip_types: std::collections::HashSet<u8> = [2, 5, 15, 31].into_iter().collect();

    let mut extracted: HashMap<u8, Vec<u8>> = HashMap::new();

    for (vol_name, vol_data) in &volume_files {
        println!("\n=== Scanning {} ({} bytes) ===", vol_name, vol_data.len());
        let vol = volume::File::new(vol_data.to_vec())
            .decompress()
            .expect("decompresses gzip file");
        let records: Vec<_> = vol.records().expect("records").into_iter().collect();

        let mut type_counts: HashMap<String, usize> = HashMap::new();

        for mut record in records {
            if record.compressed() {
                record = record.decompress().expect("decompresses record");
            }

            let raw_data = record.data();
            let messages = decode_messages(raw_data).expect("decodes messages");

            for message in &messages {
                let type_name = format!("{:?}", message.header().message_type());
                *type_counts.entry(type_name.clone()).or_insert(0) += 1;

                let type_id = message.header().message_type;
                if skip_types.contains(&type_id) || extracted.contains_key(&type_id) {
                    continue;
                }

                let offset = message.offset();
                let size = message.size();
                if offset + size <= raw_data.len() {
                    let frame_bytes = &raw_data[offset..offset + size];
                    extracted.insert(type_id, frame_bytes.to_vec());
                    println!(
                        "  Extracted {:?} (type {}, {} bytes) from offset {}",
                        type_name, type_id, size, offset
                    );
                }
            }
        }

        let mut counts: Vec<_> = type_counts.into_iter().collect();
        counts.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        for (type_name, count) in &counts {
            println!("  {}: {}", type_name, count);
        }
    }

    println!("\n=== Writing Fixtures ===");
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/messages");
    for (type_id, data) in &extracted {
        let filename = match *type_id {
            1 => "digital_radar_data_legacy.bin",
            3 => "performance_maintenance_data.bin",
            4 | 10 => "console_message.bin",
            6 => "rda_control_commands.bin",
            8 => "clutter_censor_zones.bin",
            9 => "request_for_data.bin",
            11 | 12 => "loopback_test.bin",
            13 => "clutter_filter_bypass_map.bin",
            18 => "rda_adaptation_data.bin",
            32 => "rda_prf_data.bin",
            33 => "rda_log_data.bin",
            _ => {
                println!(
                    "  Type {}: {} bytes (no filename mapping, skipped)",
                    type_id,
                    data.len()
                );
                continue;
            }
        };
        let path = fixture_dir.join(filename);
        if path.exists() {
            println!(
                "  Type {} -> {} (already exists, skipping)",
                type_id, filename
            );
            continue;
        }
        println!("  Type {} -> {} ({} bytes)", type_id, filename, data.len());
        fs::write(&path, data).expect("writes fixture");
    }

    // Report what's still missing
    let needed: Vec<(u8, &str)> = vec![
        (3, "performance_maintenance_data"),
        (4, "console_message"),
        (6, "rda_control_commands"),
        (8, "clutter_censor_zones"),
        (9, "request_for_data"),
        (11, "loopback_test"),
        (13, "clutter_filter_bypass_map"),
        (18, "rda_adaptation_data"),
        (32, "rda_prf_data"),
        (33, "rda_log_data"),
    ];

    println!("\n=== Missing Fixtures (need synthetic data) ===");
    for (type_id, name) in &needed {
        if !extracted.contains_key(type_id) {
            println!("  Type {} ({}): NOT FOUND in any volume", type_id, name);
        }
    }
}
