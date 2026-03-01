//! Loopback Test (Types 11, 12) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays a Loopback Test (Type 11 or 12) message with full details.
pub fn parse_loopback_test(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode loopback test: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::LoopbackTest(data) => data,
        _ => return "Message is not a loopback test".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== Loopback Test ===\n\n");
    output.push_str(&format!(
        "Message Size: {} half-words\n",
        msg.message_size()
    ));

    let pattern = msg.bit_pattern();
    output.push_str(&format!("Bit Pattern: {} bytes\n", pattern.len()));

    output.push_str("\n--- Bit Pattern Data ---\n");
    for (i, chunk) in pattern.chunks(16).enumerate() {
        output.push_str(&format!("{:04X}  ", i * 16));
        for (j, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02X} ", byte));
            if j == 7 {
                output.push(' ');
            }
        }
        // Pad if short row
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                output.push_str("   ");
                if j == 7 {
                    output.push(' ');
                }
            }
        }
        output.push(' ');
        // ASCII
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                output.push(*byte as char);
            } else {
                output.push('.');
            }
        }
        output.push('\n');
    }

    output
}
