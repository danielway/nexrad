//! Utility to generate synthetic binary fixtures for message types that don't appear
//! in real Archive II volume files (control messages, RPG-originated messages, etc.).
//!
//! Run with: cargo test --test generate_synthetic_fixtures -- --ignored --nocapture

use std::fs;
use std::path::Path;

const FRAME_SIZE: usize = 2432;
const HEADER_SIZE: usize = 28;
const CONTENT_SIZE: usize = FRAME_SIZE - HEADER_SIZE;

/// Build a 28-byte message header.
fn build_header(
    message_type: u8,
    segment_size_halfwords: u16,
    segment_count: u16,
    segment_number: u16,
) -> [u8; HEADER_SIZE] {
    let mut header = [0u8; HEADER_SIZE];
    // bytes 0-11: rpg_unknown = zeros
    // bytes 12-13: segment_size in halfwords (big-endian)
    header[12..14].copy_from_slice(&segment_size_halfwords.to_be_bytes());
    // byte 14: redundant_channel = 8 (ORDA single channel)
    header[14] = 8;
    // byte 15: message_type
    header[15] = message_type;
    // bytes 16-17: sequence_number = 1
    header[16..18].copy_from_slice(&1u16.to_be_bytes());
    // bytes 18-19: date = 20089 (days since epoch, ~Dec 31, 2024)
    header[18..20].copy_from_slice(&20089u16.to_be_bytes());
    // bytes 20-23: time = 43200000 (noon GMT in milliseconds)
    header[20..24].copy_from_slice(&43200000u32.to_be_bytes());
    // bytes 24-25: segment_count
    header[24..26].copy_from_slice(&segment_count.to_be_bytes());
    // bytes 26-27: segment_number
    header[26..28].copy_from_slice(&segment_number.to_be_bytes());
    header
}

/// Build a single-segment 2432-byte frame.
fn build_single_segment_frame(message_type: u8, content: &[u8]) -> Vec<u8> {
    let mut frame = vec![0u8; FRAME_SIZE];
    let header = build_header(message_type, (FRAME_SIZE / 2) as u16, 1, 1);
    frame[..HEADER_SIZE].copy_from_slice(&header);
    frame[HEADER_SIZE..HEADER_SIZE + content.len()].copy_from_slice(content);
    frame
}

/// Build multi-segment frames from content that spans multiple segments.
fn build_multi_segment_frames(message_type: u8, content: &[u8]) -> Vec<u8> {
    let segment_count = (content.len() + CONTENT_SIZE - 1) / CONTENT_SIZE;
    let mut frames = Vec::with_capacity(segment_count * FRAME_SIZE);

    for seg_idx in 0..segment_count {
        let offset = seg_idx * CONTENT_SIZE;
        let remaining = content.len() - offset;
        let payload_size = remaining.min(CONTENT_SIZE);

        // segment_size includes the header
        let segment_size_halfwords = ((HEADER_SIZE + payload_size) / 2) as u16;
        let header = build_header(
            message_type,
            segment_size_halfwords,
            segment_count as u16,
            (seg_idx + 1) as u16,
        );

        let mut frame = vec![0u8; FRAME_SIZE];
        frame[..HEADER_SIZE].copy_from_slice(&header);
        frame[HEADER_SIZE..HEADER_SIZE + payload_size]
            .copy_from_slice(&content[offset..offset + payload_size]);
        frames.extend_from_slice(&frame);
    }

    frames
}

/// Type 4: Console Message (RDA → RPG)
fn generate_console_message() -> Vec<u8> {
    let text = b"NEXRAD TEST MSG";
    let mut content = Vec::new();
    // message_size: length of text in bytes
    content.extend_from_slice(&(text.len() as u16).to_be_bytes());
    content.extend_from_slice(text);
    build_single_segment_frame(4, &content)
}

/// Type 6: RDA Control Commands (RPG → RDA)
fn generate_rda_control_commands() -> Vec<u8> {
    let mut content = vec![0u8; 52];
    let mut offset = 0;

    // rda_state_command: 32772 = Operate
    content[offset..offset + 2].copy_from_slice(&32772u16.to_be_bytes());
    offset += 2;
    // rda_log_command: 1 = Enable
    content[offset..offset + 2].copy_from_slice(&1u16.to_be_bytes());
    offset += 2;
    // auxiliary_power_generator_control: 0 = No change
    offset += 2;
    // rda_control_authorization: 8 = Remote control accepted
    content[offset..offset + 2].copy_from_slice(&8u16.to_be_bytes());
    offset += 2;
    // restart_vcp_or_elevation_cut: 0
    offset += 2;
    // select_local_vcp_number: 35
    content[offset..offset + 2].copy_from_slice(&35u16.to_be_bytes());
    offset += 2;
    // spare_7: 0
    offset += 2;
    // super_resolution_control: 2 = Enable
    content[offset..offset + 2].copy_from_slice(&2u16.to_be_bytes());
    offset += 2;
    // clutter_mitigation_decision_control: 2 = Enable
    content[offset..offset + 2].copy_from_slice(&2u16.to_be_bytes());
    offset += 2;
    // avset_control: 2 = Enable
    content[offset..offset + 2].copy_from_slice(&2u16.to_be_bytes());
    offset += 2;
    // spare_11: 0
    offset += 2;
    // channel_control_command: 1 = Controlling
    content[offset..offset + 2].copy_from_slice(&1u16.to_be_bytes());
    offset += 2;
    // performance_check_control: 0
    offset += 2;
    // zdr_bias_estimate: 500 (as signed i16)
    content[offset..offset + 2].copy_from_slice(&500i16.to_be_bytes());
    offset += 2;
    // spare_15_20: [0; 6] (12 bytes)
    offset += 12;
    // spot_blanking: 4 = Disable
    content[offset..offset + 2].copy_from_slice(&4u16.to_be_bytes());
    // spare_22_26: [0; 5] already zeros

    build_single_segment_frame(6, &content)
}

/// Type 8: Clutter Censor Zones (RPG → RDA)
fn generate_clutter_censor_zones() -> Vec<u8> {
    let mut content = Vec::new();

    // override_region_count: 2
    content.extend_from_slice(&2u16.to_be_bytes());

    // Region 1: range 10-50km, azimuth 0-90°, elevation 1, clutter forced
    content.extend_from_slice(&10u16.to_be_bytes()); // start_range
    content.extend_from_slice(&50u16.to_be_bytes()); // stop_range
    content.extend_from_slice(&0u16.to_be_bytes()); // start_azimuth
    content.extend_from_slice(&90u16.to_be_bytes()); // stop_azimuth
    content.extend_from_slice(&1u16.to_be_bytes()); // elevation_segment
    content.extend_from_slice(&2u16.to_be_bytes()); // operator_select_code (clutter forced)

    // Region 2: range 100-200km, azimuth 180-270°, elevation 3, bypass forced
    content.extend_from_slice(&100u16.to_be_bytes()); // start_range
    content.extend_from_slice(&200u16.to_be_bytes()); // stop_range
    content.extend_from_slice(&180u16.to_be_bytes()); // start_azimuth
    content.extend_from_slice(&270u16.to_be_bytes()); // stop_azimuth
    content.extend_from_slice(&3u16.to_be_bytes()); // elevation_segment
    content.extend_from_slice(&0u16.to_be_bytes()); // operator_select_code (bypass forced)

    build_single_segment_frame(8, &content)
}

/// Type 9: Request for Data (RPG → RDA)
fn generate_request_for_data() -> Vec<u8> {
    let mut content = Vec::new();
    // data_request_type: 0x009F = bits 0-4 and 7 set (all request types)
    content.extend_from_slice(&0x009Fu16.to_be_bytes());
    build_single_segment_frame(9, &content)
}

/// Type 11: Loopback Test (RDA → RPG)
fn generate_loopback_test() -> Vec<u8> {
    let mut content = Vec::new();
    // message_size: 5 halfwords (1 size + 4 pattern halfwords = 8 bytes of pattern)
    content.extend_from_slice(&5u16.to_be_bytes());
    // bit_pattern: alternating 0xAA/0x55 pattern (8 bytes)
    content.extend_from_slice(&[0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55]);
    build_single_segment_frame(11, &content)
}

/// Type 13: Clutter Filter Bypass Map (RDA → RPG)
///
/// Multi-segment message: 1 elevation with 360 radials × 32 halfwords.
fn generate_clutter_filter_bypass_map() -> Vec<u8> {
    let mut content = Vec::new();

    // Header (6 bytes)
    content.extend_from_slice(&20089u16.to_be_bytes()); // bypass_map_generation_date
    content.extend_from_slice(&720u16.to_be_bytes()); // bypass_map_generation_time (minutes)
    content.extend_from_slice(&1u16.to_be_bytes()); // number_of_elevation_segments

    // Elevation segment 1
    content.extend_from_slice(&1u16.to_be_bytes()); // elevation_segment_number

    // Range bin data: 360 radials × 32 halfwords × 2 bytes = 23040 bytes
    // Set a recognizable pattern: first radial has bypass enabled for first 16 bins
    let mut range_bins = vec![0u8; 23040];
    // Radial 0, halfword 0: set all bits (bypass all first 16 range bins)
    range_bins[0] = 0xFF;
    range_bins[1] = 0xFF;
    // Radial 180, halfword 0: set alternating bits
    let radial_180_offset = 180 * 32 * 2; // 180 * 64 bytes
    range_bins[radial_180_offset] = 0xAA;
    range_bins[radial_180_offset + 1] = 0x55;
    content.extend_from_slice(&range_bins);

    build_multi_segment_frames(13, &content)
}

/// Type 33: RDA Log Data (RDA → RPG)
fn generate_rda_log_data() -> Vec<u8> {
    let log_data = b"NEXRAD LOG TEST\n";

    let mut content = Vec::new();

    // Header (68 bytes)
    content.extend_from_slice(&1u32.to_be_bytes()); // version
    let mut identifier = [0u8; 26]; // identifier
    identifier[..7].copy_from_slice(b"TestLog");
    content.extend_from_slice(&identifier);
    content.extend_from_slice(&1u32.to_be_bytes()); // data_version
    content.extend_from_slice(&0u32.to_be_bytes()); // compression_type (uncompressed)
    content.extend_from_slice(&(log_data.len() as u32).to_be_bytes()); // compressed_size
    content.extend_from_slice(&(log_data.len() as u32).to_be_bytes()); // decompressed_size
    content.extend_from_slice(&[0u8; 22]); // spare

    // Log data payload
    content.extend_from_slice(log_data);

    build_single_segment_frame(33, &content)
}

#[test]
#[ignore]
fn generate_all_synthetic_fixtures() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/messages");

    let fixtures: Vec<(&str, Vec<u8>)> = vec![
        ("console_message.bin", generate_console_message()),
        ("rda_control_commands.bin", generate_rda_control_commands()),
        ("clutter_censor_zones.bin", generate_clutter_censor_zones()),
        ("request_for_data.bin", generate_request_for_data()),
        ("loopback_test.bin", generate_loopback_test()),
        (
            "clutter_filter_bypass_map.bin",
            generate_clutter_filter_bypass_map(),
        ),
        ("rda_log_data.bin", generate_rda_log_data()),
    ];

    for (filename, data) in &fixtures {
        let path = fixture_dir.join(filename);
        println!("Writing {} ({} bytes)", filename, data.len());
        fs::write(&path, data).expect("writes fixture");
    }

    println!("\nAll synthetic fixtures generated successfully.");

    // Verify each fixture can be decoded
    println!("\nVerifying fixtures decode correctly...");
    for (filename, data) in &fixtures {
        match nexrad_decode::messages::decode_messages(data) {
            Ok(messages) => {
                assert!(!messages.is_empty(), "{filename} produced no messages");
                let msg = &messages[0];
                println!(
                    "  {} -> {:?} ({})",
                    filename,
                    msg.header().message_type(),
                    match msg.contents() {
                        nexrad_decode::messages::MessageContents::Other => "Other (UNEXPECTED)",
                        _ => "decoded OK",
                    }
                );
                assert!(
                    !matches!(
                        msg.contents(),
                        nexrad_decode::messages::MessageContents::Other
                    ),
                    "{filename} decoded as Other instead of specific type"
                );
            }
            Err(e) => panic!("{filename} failed to decode: {e:?}"),
        }
    }
}
