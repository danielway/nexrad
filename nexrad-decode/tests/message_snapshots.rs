//! Individual message snapshot tests.
//!
//! These tests decode individual messages from binary files, producing smaller,
//! more manageable snapshots than a full volume decode test.

use insta::assert_debug_snapshot;
use nexrad_decode::messages::decode_messages;

/// Test directory for message-specific binary files.
const TEST_DATA_DIR: &str = "tests/data/messages";

// =============================================================================
// Message Type 2: RDA Status Data
// =============================================================================

/// Tests decoding of a single RDA Status Data message (type 2).
///
/// The test file should contain a complete message including the 28-byte MessageHeader
/// followed by the RDA status data payload. Fixed-length messages are padded to 2432 bytes.
#[test]
fn test_decode_rda_status_data() {
    let data = std::fs::read(format!("{TEST_DATA_DIR}/rda_status_data.bin"))
        .expect("failed to read test data file");

    let messages = decode_messages(&data).expect("decodes successfully");

    assert_eq!(messages.len(), 1, "expected exactly one message");
    assert_debug_snapshot!("rda_status_data", messages[0]);
}

// =============================================================================
// Message Type 5: Volume Coverage Pattern
// =============================================================================

/// Tests decoding of a single Volume Coverage Pattern message (type 5).
///
/// The test file should contain a complete message including the 28-byte MessageHeader
/// followed by the VCP header and elevation data blocks. Fixed-length messages are
/// padded to 2432 bytes.
#[test]
fn test_decode_volume_coverage_pattern() {
    let data = std::fs::read(format!("{TEST_DATA_DIR}/volume_coverage_pattern.bin"))
        .expect("failed to read test data file");

    let messages = decode_messages(&data).expect("decodes successfully");

    assert_eq!(messages.len(), 1, "expected exactly one message");
    assert_debug_snapshot!("volume_coverage_pattern", messages[0]);
}

// =============================================================================
// Message Type 31: Digital Radar Data (Generic Format)
// =============================================================================

/// Tests decoding of a single Digital Radar Data message (type 31) with all data blocks.
///
/// This test file should contain a message that includes all possible data blocks:
/// VOL, ELV, RAD, REF, VEL, SW, ZDR, PHI, RHO, CFP.
#[test]
fn test_decode_digital_radar_data_full() {
    let data = std::fs::read(format!("{TEST_DATA_DIR}/digital_radar_data_full.bin"))
        .expect("failed to read test data file");

    let messages = decode_messages(&data).expect("decodes successfully");

    assert_eq!(messages.len(), 1, "expected exactly one message");
    assert_debug_snapshot!("digital_radar_data_full", messages[0]);
}
