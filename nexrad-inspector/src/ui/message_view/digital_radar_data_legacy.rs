//! Digital Radar Data Legacy (Type 1) message parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays a Digital Radar Data Legacy (Type 1) message with full details.
pub fn parse_digital_radar_data_legacy(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode legacy digital radar data: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::DigitalRadarDataLegacy(data) => data,
        _ => return "Message is not legacy digital radar data".to_string(),
    };

    let datetime = msg
        .date_time()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let mut output = String::new();

    output.push_str("=== Digital Radar Data Legacy (Type 1) ===\n\n");

    // Header section
    output.push_str("--- Radial Header ---\n");
    output.push_str(&format!("Date/Time: {}\n", datetime));
    output.push_str(&format!(
        "Azimuth: #{} at {:.2}\u{00b0}\n",
        msg.azimuth_number(),
        msg.azimuth_angle()
    ));
    output.push_str(&format!(
        "Elevation: #{} at {:.2}\u{00b0}\n",
        msg.elevation_number(),
        msg.elevation_angle()
    ));

    let status = msg.radial_status();
    output.push_str(&format!(
        "Radial Status: {} ({})\n",
        status,
        match status {
            0 => "Start of new elevation",
            1 => "Intermediate radial",
            2 => "End of elevation",
            3 => "Beginning of volume scan",
            4 => "End of volume scan",
            _ => "Unknown",
        }
    ));

    output.push_str(&format!("VCP Number: {}\n", msg.vcp_number()));
    output.push_str(&format!(
        "Unambiguous Range: {:.1} km\n",
        msg.unambiguous_range_km()
    ));
    output.push_str(&format!(
        "Calibration Constant: {:.2} dB\n",
        msg.calibration_constant()
    ));
    output.push_str(&format!(
        "Doppler Velocity Resolution: {:.1} m/s\n",
        msg.doppler_velocity_resolution()
    ));

    // Gate configuration
    output.push_str("\n--- Gate Configuration ---\n");
    output.push_str(&format!(
        "Surveillance Gates: {} (interval: {} m, first gate: {} m)\n",
        msg.num_surveillance_gates(),
        msg.surveillance_gate_interval(),
        msg.surveillance_first_gate_range()
    ));
    output.push_str(&format!(
        "Doppler Gates: {} (interval: {} m, first gate: {} m)\n",
        msg.num_doppler_gates(),
        msg.doppler_gate_interval(),
        msg.doppler_first_gate_range()
    ));

    // Reflectivity data
    if let Some(gates) = msg.reflectivity_gates() {
        output.push_str(&format!("\n--- Reflectivity ({} gates) ---\n", gates.len()));
        render_legacy_gates(
            &mut output,
            gates,
            "REF",
            msg.surveillance_first_gate_range() as f32,
            msg.surveillance_gate_interval() as f32,
        );
    } else {
        output.push_str("\n--- Reflectivity ---\n");
        output.push_str("Not present in this radial\n");
    }

    // Velocity data
    if let Some(gates) = msg.velocity_gates() {
        output.push_str(&format!("\n--- Velocity ({} gates) ---\n", gates.len()));
        render_legacy_gates(
            &mut output,
            gates,
            "VEL",
            msg.doppler_first_gate_range() as f32,
            msg.doppler_gate_interval() as f32,
        );
    } else {
        output.push_str("\n--- Velocity ---\n");
        output.push_str("Not present in this radial\n");
    }

    // Spectrum Width data
    if let Some(gates) = msg.spectrum_width_gates() {
        output.push_str(&format!(
            "\n--- Spectrum Width ({} gates) ---\n",
            gates.len()
        ));
        render_legacy_gates(
            &mut output,
            gates,
            "SW",
            msg.doppler_first_gate_range() as f32,
            msg.doppler_gate_interval() as f32,
        );
    } else {
        output.push_str("\n--- Spectrum Width ---\n");
        output.push_str("Not present in this radial\n");
    }

    output
}

/// Renders a legacy gate data block with ASCII visualization and statistics.
///
/// Legacy gate encoding (1 byte per gate):
/// - 0 = Below Threshold
/// - 1 = Range Folded
/// - 2..=255 = Scaled physical value
fn render_legacy_gates(
    output: &mut String,
    gates: &[u8],
    product: &str,
    first_gate_range: f32,
    gate_interval: f32,
) {
    // Characters ordered by increasing visual density
    const CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

    let (min_val, max_val, decode): (f32, f32, fn(u8) -> f32) = match product {
        "REF" => (-32.0, 94.5, |v| (v as f32 - 2.0) / 2.0 - 32.0),
        "VEL" => (-63.5, 63.5, |v| (v as f32 - 2.0) * 0.5 - 63.5),
        "SW" => (-63.5, 63.5, |v| (v as f32 - 2.0) * 0.5 - 63.5),
        _ => (-32.0, 94.5, |v| (v as f32 - 2.0) / 2.0 - 32.0),
    };

    // ASCII visualization
    let ascii: String = gates
        .iter()
        .map(|&v| match v {
            0 => ' ', // Below threshold
            1 => '~', // Range folded
            _ => {
                let decoded = decode(v);
                let normalized = ((decoded - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
                let index = (normalized * (CHARS.len() - 1) as f32) as usize;
                CHARS[index]
            }
        })
        .collect();

    let gates_per_row = 80;
    for (row_idx, chunk) in ascii.as_bytes().chunks(gates_per_row).enumerate() {
        let start_gate = row_idx * gates_per_row;
        let start_range = first_gate_range * 0.001 + start_gate as f32 * gate_interval * 0.001;
        output.push_str(&format!(
            "{:5.1}km |{}|\n",
            start_range,
            std::str::from_utf8(chunk).unwrap_or("")
        ));
    }

    // Legend
    output.push_str("\nLegend: ");
    match product {
        "REF" => {
            output.push_str("' '=below '~'=folded, intensity: . : - = + * # % @ (-32 to 94.5 dBZ)")
        }
        "VEL" => output
            .push_str("' '=below '~'=folded, intensity: . : - = + * # % @ (-63.5 to 63.5 m/s)"),
        "SW" => output
            .push_str("' '=below '~'=folded, intensity: . : - = + * # % @ (-63.5 to 63.5 m/s)"),
        _ => {}
    }
    output.push('\n');

    // Statistics
    let mut value_count = 0u32;
    let mut below_count = 0u32;
    let mut folded_count = 0u32;
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    let mut sum = 0.0f64;

    for &v in gates {
        match v {
            0 => below_count += 1,
            1 => folded_count += 1,
            _ => {
                let decoded = decode(v);
                value_count += 1;
                sum += decoded as f64;
                min = min.min(decoded);
                max = max.max(decoded);
            }
        }
    }

    output.push_str(&format!(
        "\nStats: {} values, {} below threshold, {} range folded\n",
        value_count, below_count, folded_count
    ));
    if value_count > 0 {
        let avg = sum / value_count as f64;
        output.push_str(&format!(
            "Range: {:.2} to {:.2}, Avg: {:.2}\n",
            min, max, avg
        ));
    }
}
