//! Clutter Filter Map (Type 15) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays a Clutter Filter Map (Type 15) message with full details.
pub fn parse_clutter_filter_map(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode clutter filter map: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::ClutterFilterMap(data) => data,
        _ => return "Message is not a clutter filter map".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== Clutter Filter Map (Type 15) ===\n\n");

    // Header
    output.push_str("--- Header ---\n");
    if let Some(dt) = msg.date_time() {
        output.push_str(&format!(
            "Generation Date/Time: {}\n",
            dt.format("%Y-%m-%d %H:%M UTC")
        ));
    } else {
        output.push_str(&format!(
            "Generation Date: {} (days since epoch)\n",
            msg.map_generation_date()
        ));
        output.push_str(&format!(
            "Generation Time: {} (minutes past midnight)\n",
            msg.map_generation_time()
        ));
    }
    output.push_str(&format!(
        "Number of Elevation Segments: {}\n",
        msg.elevation_segment_count()
    ));

    // Elevation segments
    let segments = msg.elevation_segments();
    for segment in segments {
        output.push_str(&format!(
            "\n--- Elevation Segment {} ---\n",
            segment.elevation_segment_number()
        ));

        let azimuth_segments = segment.azimuth_segments();
        output.push_str(&format!("Azimuth Segments: {}\n", azimuth_segments.len()));

        // Summary statistics
        let mut total_zones = 0usize;
        let mut forced_filter = 0usize;
        let mut bypass_control = 0usize;
        let mut forced_bypass = 0usize;
        let mut unknown_op = 0usize;

        for az_seg in azimuth_segments {
            for zone in az_seg.range_zones() {
                total_zones += 1;
                match zone.op_code() {
                    nexrad_decode::messages::clutter_filter_map::OpCode::ForceFilter => {
                        forced_filter += 1
                    }
                    nexrad_decode::messages::clutter_filter_map::OpCode::BypassMapInControl => {
                        bypass_control += 1
                    }
                    nexrad_decode::messages::clutter_filter_map::OpCode::BypassFilter => {
                        forced_bypass += 1
                    }
                    nexrad_decode::messages::clutter_filter_map::OpCode::Unknown(_) => {
                        unknown_op += 1
                    }
                }
            }
        }

        output.push_str(&format!("Total Range Zones: {}\n", total_zones));
        output.push_str(&format!("  Forced Filtering: {}\n", forced_filter));
        output.push_str(&format!("  Bypass Map Control: {}\n", bypass_control));
        output.push_str(&format!("  Forced Bypass: {}\n", forced_bypass));
        if unknown_op > 0 {
            output.push_str(&format!("  Unknown Op Code: {}\n", unknown_op));
        }

        // Detail for first 20 azimuths with non-trivial zones
        output.push_str("\nAzimuth detail (first 20 with >1 zone):\n");
        let mut shown = 0;
        for az_seg in azimuth_segments {
            let zones = az_seg.range_zones();
            if zones.len() > 1 {
                output.push_str(&format!(
                    "  Az {:3}\u{00b0}: {} zones",
                    az_seg.azimuth_segment(),
                    zones.len()
                ));
                // Show zone details inline
                let zone_details: Vec<String> = zones
                    .iter()
                    .map(|z| format!("0-{} km {:?}", z.end_range(), z.op_code()))
                    .collect();
                output.push_str(&format!(" [{}]\n", zone_details.join(", ")));

                shown += 1;
                if shown >= 20 {
                    let remaining = azimuth_segments
                        .iter()
                        .filter(|a| a.range_zones().len() > 1)
                        .count()
                        - 20;
                    if remaining > 0 {
                        output.push_str(&format!("  ... ({} more azimuths)\n", remaining));
                    }
                    break;
                }
            }
        }
    }

    output
}
