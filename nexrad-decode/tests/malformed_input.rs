//! Negative tests for malformed input handling.
//!
//! These tests verify that the decode functions gracefully handle invalid,
//! truncated, and corrupted input data without panicking.

use nexrad_decode::messages::decode_messages;

#[test]
fn test_empty_input() {
    let result = decode_messages(&[]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_single_byte_input() {
    let result = decode_messages(&[0]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_truncated_header() {
    // Message header is 28 bytes, test with less
    let truncated = vec![0u8; 15];
    let result = decode_messages(&truncated);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_partial_header() {
    // Just under the header size
    let partial = vec![0u8; 27];
    let result = decode_messages(&partial);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_header_only_no_content() {
    // Exactly 28 bytes - a header with no content
    let header_only = vec![0u8; 28];
    let result = decode_messages(&header_only);
    // Should either return empty or a message with no content, but not panic
    assert!(result.is_ok());
}

#[test]
fn test_random_garbage_data() {
    // Random-looking data that doesn't conform to any format
    let garbage: Vec<u8> = (0..1000).map(|i| (i * 17 % 256) as u8).collect();
    let result = decode_messages(&garbage);
    // Should not panic regardless of input
    assert!(result.is_ok());
}

#[test]
fn test_all_zeros() {
    let zeros = vec![0u8; 1000];
    let result = decode_messages(&zeros);
    assert!(result.is_ok());
}

#[test]
fn test_all_ones() {
    let ones = vec![0xFFu8; 1000];
    let result = decode_messages(&ones);
    assert!(result.is_ok());
}

#[test]
fn test_repeating_pattern() {
    let pattern: Vec<u8> = (0..1000).map(|i| (i % 16) as u8).collect();
    let result = decode_messages(&pattern);
    assert!(result.is_ok());
}

/// Test that truncated real data doesn't cause panics
#[test]
fn test_truncated_real_data() {
    const TEST_DATA: &[u8] = include_bytes!("data/messages/digital_radar_data_full.bin");

    // Truncate at various points
    for truncate_at in [1, 10, 50, 100, 500, TEST_DATA.len() / 2] {
        if truncate_at < TEST_DATA.len() {
            let truncated = &TEST_DATA[..truncate_at];
            let result = decode_messages(truncated);
            // Should not panic, may return error or partial results
            let _ = result;
        }
    }
}

/// Test with corrupted header fields
#[test]
fn test_corrupted_message_type() {
    // Create a minimal valid-looking header with invalid message type
    let mut header = vec![0u8; 2432]; // Standard segment size
                                      // Set message type to an invalid value
    header[17] = 255;
    let result = decode_messages(&header);
    assert!(result.is_ok());
}

/// Test invalid redundant channel values
#[test]
fn test_invalid_redundant_channel() {
    // This exercises the Unknown variant for redundant channel
    let mut header = vec![0u8; 2432];
    // Set redundant channel to an invalid value
    header[18] = 99;
    let result = decode_messages(&header);
    assert!(result.is_ok());
}

/// Test that the decoder handles minimal sized inputs at boundaries
#[test]
fn test_boundary_sizes() {
    // Test sizes around the header boundary (28 bytes)
    for size in 26..32 {
        let data = vec![0u8; size];
        let result = decode_messages(&data);
        // Should not panic
        let _ = result;
    }

    // Test sizes around the fixed segment size (2432 bytes)
    for size in 2430..2440 {
        let data = vec![0u8; size];
        let result = decode_messages(&data);
        // Should not panic
        let _ = result;
    }
}

/// Test with oversized message length field
#[test]
fn test_oversized_message_length() {
    let mut data = vec![0u8; 100];
    // Set message size field to a very large value (bytes 14-15)
    data[14] = 0xFF;
    data[15] = 0xFF;
    let result = decode_messages(&data);
    // Should not panic even with impossible message size
    let _ = result;
}
