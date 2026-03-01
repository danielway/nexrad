//! RDA PRF Data (Type 32) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays an RDA PRF Data (Type 32) message with full details.
pub fn parse_rda_prf_data(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode RDA PRF data: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::RDAPRFData(data) => data,
        _ => return "Message is not RDA PRF data".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== RDA PRF Data (Type 32) ===\n\n");
    output.push_str(&format!(
        "Number of Waveforms: {}\n",
        msg.number_of_waveforms()
    ));

    let waveform_data = msg.waveform_prf_data();
    output.push_str(&format!("Waveform Entries: {}\n", waveform_data.len()));

    for (i, wf) in waveform_data.iter().enumerate() {
        output.push_str(&format!(
            "\n--- Waveform {} (Type {}) ---\n",
            i + 1,
            wf.waveform_type()
        ));

        if wf.prf_values().is_empty() {
            output.push_str("PRF Values: (none)\n");
        } else {
            output.push_str(&format!("PRF Values ({}):\n", wf.prf_values().len()));
            for (j, prf) in wf.prf_values().iter().enumerate() {
                output.push_str(&format!("  PRF {}: {} Hz\n", j + 1, prf));
            }
        }
    }

    output
}
