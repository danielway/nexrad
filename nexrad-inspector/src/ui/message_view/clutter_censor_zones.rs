//! Clutter Censor Zones (Type 8) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays a Clutter Censor Zones (Type 8) message with full details.
pub fn parse_clutter_censor_zones(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode clutter censor zones: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::ClutterCensorZones(data) => data,
        _ => return "Message is not clutter censor zones".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== Clutter Censor Zones (Type 8) ===\n\n");
    output.push_str(&format!(
        "Override Region Count: {}\n",
        msg.override_region_count()
    ));

    let regions = msg.regions();
    if regions.is_empty() {
        output.push_str("\nNo override regions defined.\n");
    } else {
        output.push_str(&format!("\n--- Override Regions ({}) ---\n", regions.len()));
        for (i, region) in regions.iter().enumerate() {
            output.push_str(&format!("\n[Region {}]\n", i + 1));
            output.push_str(&format!(
                "  Range: {}-{} km\n",
                region.start_range, region.stop_range
            ));
            output.push_str(&format!(
                "  Azimuth: {}\u{00b0}-{}\u{00b0}\n",
                region.start_azimuth, region.stop_azimuth
            ));
            output.push_str(&format!(
                "  Elevation Segment: {}\n",
                region.elevation_segment_number
            ));

            let op_code = region.operator_select_code & 0xFFFF;
            output.push_str(&format!(
                "  Operator Select Code: {} ({})\n",
                op_code,
                match op_code {
                    0 => "Bypass filter forced",
                    1 => "Bypass map in control",
                    2 => "Clutter filtering forced",
                    _ => "Unknown",
                }
            ));
        }
    }

    output
}
