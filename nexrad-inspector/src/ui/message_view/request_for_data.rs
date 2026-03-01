//! Request for Data (Type 9) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays a Request for Data (Type 9) message with full details.
pub fn parse_request_for_data(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode request for data: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::RequestForData(data) => data,
        _ => return "Message is not a request for data".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== Request for Data (Type 9) ===\n\n");
    output.push_str(&format!(
        "Raw Request Type: 0x{:04X}\n\n",
        msg.raw_data_request_type()
    ));

    output.push_str("--- Requested Data ---\n");
    output.push_str(&format!(
        "RDA Status: {}\n",
        if msg.requests_rda_status() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Performance/Maintenance Data: {}\n",
        if msg.requests_performance_maintenance_data() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Clutter Filter Bypass Map: {}\n",
        if msg.requests_clutter_filter_bypass_map() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Clutter Filter Map: {}\n",
        if msg.requests_clutter_filter_map() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Adaptation Data: {}\n",
        if msg.requests_adaptation_data() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Volume Coverage Pattern: {}\n",
        if msg.requests_volume_coverage_pattern() {
            "Yes"
        } else {
            "No"
        }
    ));

    output
}
