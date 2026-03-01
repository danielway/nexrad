//! RDA Log Data (Type 33) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays an RDA Log Data (Type 33) message with full details.
pub fn parse_rda_log_data(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode RDA log data: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::RDALogData(data) => data,
        _ => return "Message is not RDA log data".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== RDA Log Data (Type 33) ===\n\n");

    output.push_str("--- Header ---\n");
    output.push_str(&format!("Version: {}\n", msg.version()));
    output.push_str(&format!("Identifier: {}\n", msg.identifier()));
    output.push_str(&format!("Data Version: {}\n", msg.data_version()));

    let comp_type = msg.compression_type();
    output.push_str(&format!(
        "Compression: {} ({})\n",
        comp_type,
        match comp_type {
            0 => "Uncompressed",
            1 => "GZIP",
            2 => "BZIP2",
            3 => "ZIP",
            _ => "Unknown",
        }
    ));

    output.push_str(&format!(
        "Compressed Size: {} bytes\n",
        msg.compressed_size()
    ));
    output.push_str(&format!(
        "Decompressed Size: {} bytes\n",
        msg.decompressed_size()
    ));

    if msg.decompressed_size() > 0 {
        let ratio = msg.compressed_size() as f64 / msg.decompressed_size() as f64 * 100.0;
        output.push_str(&format!("Compression Ratio: {:.1}%\n", ratio));
    }

    let log_data = msg.data();
    output.push_str(&format!(
        "\n--- Data Payload ({} bytes) ---\n",
        log_data.len()
    ));

    // Show first 512 bytes as hex
    let display_bytes = log_data.len().min(512);
    for (i, chunk) in log_data[..display_bytes].chunks(16).enumerate() {
        output.push_str(&format!("{:04X}  ", i * 16));
        for (j, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02X} ", byte));
            if j == 7 {
                output.push(' ');
            }
        }
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                output.push_str("   ");
                if j == 7 {
                    output.push(' ');
                }
            }
        }
        output.push(' ');
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                output.push(*byte as char);
            } else {
                output.push('.');
            }
        }
        output.push('\n');
    }
    if log_data.len() > 512 {
        output.push_str(&format!("... ({} more bytes)\n", log_data.len() - 512));
    }

    output
}
