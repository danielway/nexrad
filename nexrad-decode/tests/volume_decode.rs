//! Volume decode integration test.
//!
//! This test decodes a complete NEXRAD volume file and verifies aggregate
//! properties of the decoded data, such as message counts and ordering.

use nexrad_data::volume;
use nexrad_decode::messages::{decode_messages, MessageContents, MessageType};

const TEST_NEXRAD_FILE: &[u8] = include_bytes!("../../downloads/KDMX20220305_232324_V06");

#[test]
fn test_decode_volume_structure() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());

    // Verify volume header
    let header = volume.header().expect("volume should have header");
    let tape_filename = header.tape_filename().expect("should have tape filename");
    assert!(
        tape_filename.starts_with("AR2V"),
        "expected Archive II format, got {tape_filename}"
    );

    // Decode all records and collect messages
    let mut status_count = 0;
    let mut vcp_count = 0;
    let mut radar_data_count = 0;
    let mut other_count = 0;

    let records: Vec<_> = volume.records().into_iter().collect();
    assert!(!records.is_empty(), "expected at least one record");

    for mut record in records {
        if record.compressed() {
            record = record.decompress().expect("decompresses record");
        }

        let messages = decode_messages(record.data()).expect("decodes messages");

        for message in &messages {
            match message.contents() {
                MessageContents::RDAStatusData(_) => status_count += 1,
                MessageContents::VolumeCoveragePattern(_) => vcp_count += 1,
                MessageContents::DigitalRadarData(_) => radar_data_count += 1,
                MessageContents::ClutterFilterMap(_) => other_count += 1,
                MessageContents::Other => other_count += 1,
            }
        }
    }

    // Verify expected message distribution for a typical volume scan
    assert!(
        status_count >= 1,
        "expected at least one RDA status message"
    );
    assert!(vcp_count >= 1, "expected at least one VCP message");
    assert!(
        radar_data_count > 1000,
        "expected significant radar data messages, got {radar_data_count}"
    );

    // Radar data should be the dominant message type
    assert!(
        radar_data_count > status_count + vcp_count + other_count,
        "radar data should be the most common message type"
    );
}

#[test]
fn test_decode_volume_message_ordering() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());

    // The first record typically contains metadata messages
    let mut first_record = volume.records().into_iter().next().expect("has records");
    if first_record.compressed() {
        first_record = first_record.decompress().expect("decompresses");
    }

    let messages = decode_messages(first_record.data()).expect("decodes");

    // First record should have status and VCP messages
    let has_status = messages
        .iter()
        .any(|m| matches!(m.contents(), MessageContents::RDAStatusData(_)));
    let has_vcp = messages
        .iter()
        .any(|m| matches!(m.contents(), MessageContents::VolumeCoveragePattern(_)));

    assert!(has_status, "first record should contain RDA status message");
    assert!(has_vcp, "first record should contain VCP message");
}

#[test]
fn test_decode_volume_radar_data_properties() {
    let volume = volume::File::new(TEST_NEXRAD_FILE.to_vec());

    let mut elevation_numbers_seen = std::collections::HashSet::new();
    let mut found_volume_start = false;
    let mut found_volume_end = false;

    for mut record in volume.records() {
        if record.compressed() {
            record = record.decompress().expect("decompresses");
        }

        let messages = decode_messages(record.data()).expect("decodes");

        for message in messages {
            if message.header().message_type() != MessageType::RDADigitalRadarDataGenericFormat {
                continue;
            }

            if let MessageContents::DigitalRadarData(radar_data) = message.contents() {
                elevation_numbers_seen.insert(radar_data.header().elevation_number());

                // Check for volume scan boundaries
                match radar_data.header().radial_status_raw() {
                    3 => found_volume_start = true, // VolumeScanStart
                    4 => found_volume_end = true,   // VolumeScanEnd
                    _ => {}
                }
            }
        }
    }

    // A complete volume should have multiple elevations
    assert!(
        elevation_numbers_seen.len() > 1,
        "expected multiple elevation cuts, got {}",
        elevation_numbers_seen.len()
    );

    // Should find volume scan boundaries
    assert!(found_volume_start, "expected to find volume scan start");
    assert!(found_volume_end, "expected to find volume scan end");
}
