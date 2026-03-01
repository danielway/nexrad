//! Console Message (Types 4, 10) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays a Console Message (Type 4 or 10) with full details.
pub fn parse_console_message(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode console message: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::ConsoleMessage(data) => data,
        _ => return "Message is not a console message".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== Console Message ===\n\n");
    output.push_str(&format!("Message Size: {} bytes\n", msg.message_size()));

    output.push_str("\n--- Text ---\n");
    match msg.text() {
        Some(text) => {
            output.push_str(text);
            output.push('\n');
        }
        None => {
            let bytes = msg.text_bytes();
            output.push_str(&format!("(Non-UTF8 text, {} bytes)\n", bytes.len()));
            // Show hex dump of first 256 bytes
            for (i, chunk) in bytes.chunks(16).take(16).enumerate() {
                output.push_str(&format!("{:04X}  ", i * 16));
                for byte in chunk {
                    output.push_str(&format!("{:02X} ", byte));
                }
                output.push('\n');
            }
            if bytes.len() > 256 {
                output.push_str("...\n");
            }
        }
    }

    output
}
