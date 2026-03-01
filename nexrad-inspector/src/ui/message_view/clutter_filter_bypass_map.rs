//! Clutter Filter Bypass Map (Type 13) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays a Clutter Filter Bypass Map (Type 13) message with full details.
pub fn parse_clutter_filter_bypass_map(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode clutter filter bypass map: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::ClutterFilterBypassMap(data) => data,
        _ => return "Message is not a clutter filter bypass map".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== Clutter Filter Bypass Map (Type 13) ===\n\n");

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
            msg.bypass_map_generation_date()
        ));
        output.push_str(&format!(
            "Generation Time: {} (minutes past midnight)\n",
            msg.bypass_map_generation_time()
        ));
    }
    output.push_str(&format!(
        "Number of Elevation Segments: {}\n",
        msg.number_of_elevation_segments()
    ));

    // Elevation segments with summary statistics
    let segments = msg.elevation_segments();
    output.push_str(&format!(
        "\n--- Elevation Segments ({}) ---\n",
        segments.len()
    ));

    for (i, segment) in segments.iter().enumerate() {
        output.push_str(&format!("\n[Segment {}]\n", segment.segment_number()));

        // Compute summary statistics across 360 radials x 512 range bins
        let mut bypass_count = 0u32;
        let mut filter_count = 0u32;
        let mut radials_with_bypass = 0u32;

        for radial in 0..360 {
            let mut radial_has_bypass = false;
            for bin in 0..512 {
                match segment.bypass_flag(radial, bin) {
                    Some(true) => {
                        bypass_count += 1;
                        radial_has_bypass = true;
                    }
                    Some(false) => {
                        filter_count += 1;
                    }
                    None => {}
                }
            }
            if radial_has_bypass {
                radials_with_bypass += 1;
            }
        }

        let total = bypass_count + filter_count;
        output.push_str(&format!(
            "  Total Bins: {} (360 radials x 512 range bins)\n",
            total
        ));
        output.push_str(&format!(
            "  Bypass Enabled: {} bins ({:.1}%)\n",
            bypass_count,
            if total > 0 {
                bypass_count as f64 / total as f64 * 100.0
            } else {
                0.0
            }
        ));
        output.push_str(&format!(
            "  Filter Active: {} bins ({:.1}%)\n",
            filter_count,
            if total > 0 {
                filter_count as f64 / total as f64 * 100.0
            } else {
                0.0
            }
        ));
        output.push_str(&format!(
            "  Radials with Bypass: {}/360\n",
            radials_with_bypass
        ));

        // Show per-radial summary for radials with bypass bins (first 20)
        if radials_with_bypass > 0 && i == 0 {
            output.push_str("\n  Radials with bypass bins (first 20):\n");
            let mut shown = 0;
            for radial in 0..360 {
                let bypass_in_radial: u32 = (0..512)
                    .filter(|&bin| segment.bypass_flag(radial, bin) == Some(true))
                    .count() as u32;
                if bypass_in_radial > 0 {
                    output.push_str(&format!(
                        "    Radial {:3}\u{00b0}: {} bypass bins\n",
                        radial, bypass_in_radial
                    ));
                    shown += 1;
                    if shown >= 20 {
                        if radials_with_bypass > 20 {
                            output.push_str(&format!(
                                "    ... ({} more radials)\n",
                                radials_with_bypass - 20
                            ));
                        }
                        break;
                    }
                }
            }
        }
    }

    output
}
