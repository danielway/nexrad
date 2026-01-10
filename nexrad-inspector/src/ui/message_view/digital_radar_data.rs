//! Digital Radar Data (Type 31) message parsing and display.

use nexrad_decode::messages::{decode_messages, digital_radar_data, MessageContents};

/// Parses and displays a Digital Radar Data (Type 31) message with full details.
pub fn parse_digital_radar_data(data: &[u8]) -> String {
    // Use decode_messages to get the fully parsed message
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode digital radar data: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::DigitalRadarData(data) => data,
        _ => return "Message is not digital radar data".to_string(),
    };

    let header = &msg.header;

    let datetime = header
        .date_time()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let radar_id = std::str::from_utf8(&header.radar_identifier)
        .unwrap_or("????")
        .trim_end_matches('\0');

    let mut output = String::new();

    // Header section
    output.push_str("=== Digital Radar Data (Type 31) ===\n\n");
    output.push_str(&format!("Radar ID: {}\n", radar_id));
    output.push_str(&format!("Date/Time: {}\n", datetime));
    output.push_str(&format!(
        "Azimuth: #{} at {:.2}\u{00b0}\n",
        header.azimuth_number,
        header.azimuth_angle.get()
    ));
    output.push_str(&format!(
        "Elevation: #{} at {:.2}\u{00b0}\n",
        header.elevation_number,
        header.elevation_angle.get()
    ));
    output.push_str(&format!("Radial Status: {:?}\n", header.radial_status()));
    output.push_str(&format!("Radial Length: {} bytes\n", header.radial_length));
    output.push_str(&format!(
        "Compression: {:?}\n",
        header.compression_indicator()
    ));
    output.push_str(&format!(
        "Azimuth Resolution: {} ({}\u{00b0})\n",
        header.azimuth_resolution_spacing,
        if header.azimuth_resolution_spacing == 1 {
            0.5
        } else {
            1.0
        }
    ));
    output.push_str(&format!("Data Block Count: {}\n", header.data_block_count));

    // Volume Data Block
    if let Some(ref vol) = msg.volume_data_block {
        output.push_str("\n--- Volume Data Block ---\n");
        output.push_str(&format!(
            "Location: {:.4}\u{00b0}, {:.4}\u{00b0}\n",
            vol.latitude.get(),
            vol.longitude.get()
        ));
        output.push_str(&format!(
            "Site Height: {} m (feedhorn: {} m)\n",
            vol.site_height.get(),
            vol.feedhorn_height.get()
        ));
        output.push_str(&format!(
            "VCP: {} ({:?})\n",
            vol.volume_coverage_pattern_number.get(),
            vol.volume_coverage_pattern()
        ));
        output.push_str(&format!(
            "Calibration Constant: {:.2} dB\n",
            vol.calibration_constant.get()
        ));
        output.push_str(&format!(
            "TX Power (H/V): {:.1}/{:.1} kW\n",
            vol.horizontal_shv_tx_power.get(),
            vol.vertical_shv_tx_power.get()
        ));
        output.push_str(&format!(
            "System ZDR: {:.2} dB\n",
            vol.system_differential_reflectivity.get()
        ));
        output.push_str(&format!(
            "Initial DP: {:.2}\u{00b0}\n",
            vol.initial_system_differential_phase.get()
        ));
        output.push_str(&format!(
            "Processing Status: {:?}\n",
            vol.processing_status()
        ));
        output.push_str(&format!(
            "Version: {}.{}\n",
            vol.major_version_number, vol.minor_version_number
        ));
    }

    // Radial Data Block
    if let Some(ref rad) = msg.radial_data_block {
        output.push_str("\n--- Radial Data Block ---\n");
        output.push_str(&format!(
            "Unambiguous Range: {:.1} km\n",
            rad.unambiguous_range.get() as f32 * 0.1
        ));
        output.push_str(&format!(
            "Nyquist Velocity: {:.2} m/s\n",
            rad.nyquist_velocity.get() as f32 * 0.01
        ));
        output.push_str(&format!(
            "Noise Level (H): {:.2} dBm\n",
            rad.horizontal_channel_noise_level.get()
        ));
        output.push_str(&format!(
            "Noise Level (V): {:.2} dBm\n",
            rad.vertical_channel_noise_level.get()
        ));
        output.push_str(&format!(
            "Calibration (H): {:.2} dBZ\n",
            rad.horizontal_channel_calibration_constant.get()
        ));
        output.push_str(&format!(
            "Calibration (V): {:.2} dBZ\n",
            rad.vertical_channel_calibration_constant.get()
        ));
    }

    // Elevation Data Block
    if let Some(ref elv) = msg.elevation_data_block {
        output.push_str("\n--- Elevation Data Block ---\n");
        output.push_str(&format!(
            "Atmospheric Attenuation: {:.4} dB/km\n",
            elv.atmos.get()
        ));
        output.push_str(&format!(
            "Calibration Constant: {:.2} dB\n",
            elv.calibration_constant.get()
        ));
    }

    // Moment Data Blocks
    let moment_blocks: Vec<(
        &str,
        &str,
        Option<&digital_radar_data::DataBlock<digital_radar_data::GenericDataBlock>>,
    )> = vec![
        ("REF", "Reflectivity", msg.reflectivity_data_block.as_ref()),
        ("VEL", "Velocity", msg.velocity_data_block.as_ref()),
        (
            "SW ",
            "Spectrum Width",
            msg.spectrum_width_data_block.as_ref(),
        ),
        (
            "ZDR",
            "Differential Reflectivity",
            msg.differential_reflectivity_data_block.as_ref(),
        ),
        (
            "PHI",
            "Differential Phase",
            msg.differential_phase_data_block.as_ref(),
        ),
        (
            "RHO",
            "Correlation Coefficient",
            msg.correlation_coefficient_data_block.as_ref(),
        ),
        (
            "CFP",
            "Specific Diff Phase",
            msg.specific_diff_phase_data_block.as_ref(),
        ),
    ];

    for (id, name, block_opt) in moment_blocks {
        if let Some(block) = block_opt {
            output.push_str(&format!("\n--- {} ({}) ---\n", name, id));
            output.push_str(&format!(
                "Gates: {}\n",
                block.header.number_of_data_moment_gates.get()
            ));
            output.push_str(&format!(
                "First Gate: {:.3} km\n",
                block.header.data_moment_range.get() as f32 * 0.001
            ));
            output.push_str(&format!(
                "Gate Interval: {:.3} km\n",
                block.header.data_moment_range_sample_interval.get() as f32 * 0.001
            ));
            output.push_str(&format!(
                "SNR Threshold: {:.3} dB\n",
                block.header.snr_threshold.get()
            ));
            output.push_str(&format!(
                "Scale/Offset: {:.2}/{:.2}\n",
                block.header.scale.get(),
                block.header.offset.get()
            ));
            output.push_str(&format!(
                "Word Size: {} bits\n",
                block.header.data_word_size
            ));
            output.push_str(&format!(
                "Control Flags: {:?}\n",
                block.header.control_flags()
            ));

            // Get decoded values and create ASCII visualization
            let decoded = block.decoded_values();
            let gate_count = decoded.len();

            if gate_count > 0 {
                output.push_str(&format!("\nData ({} gates):\n", gate_count));

                // Show ASCII visualization in rows
                let gates_per_row = 80;
                let ascii = scaled_values_to_ascii(&decoded, id);

                for (row_idx, chunk) in ascii.as_bytes().chunks(gates_per_row).enumerate() {
                    let start_gate = row_idx * gates_per_row;
                    let start_range = block.header.data_moment_range.get() as f32 * 0.001
                        + start_gate as f32
                            * block.header.data_moment_range_sample_interval.get() as f32
                            * 0.001;
                    output.push_str(&format!(
                        "{:5.1}km |{}|\n",
                        start_range,
                        std::str::from_utf8(chunk).unwrap_or("")
                    ));
                }

                // Legend
                output.push_str("\nLegend: ");
                match id {
                    "REF" => output.push_str(
                        "' '=<-30 '.'=-20 ':'=-10 '-'=0 '='=10 '+'=20 '*'=30 '#'=40 '%'=50 '@'=60+ dBZ",
                    ),
                    "VEL" => output.push_str(
                        "' '=below ' '..'-'=toward radar '='=zero '+'...'@'=away from radar '~'=folded",
                    ),
                    _ => output.push_str(
                        "' '=below threshold '~'=range folded, intensity: . : - = + * # % @",
                    ),
                }
                output.push('\n');

                // Statistics
                let mut value_count = 0;
                let mut below_count = 0;
                let mut folded_count = 0;
                let mut min_val = f32::MAX;
                let mut max_val = f32::MIN;
                let mut sum = 0.0f64;

                for val in &decoded {
                    match val {
                        digital_radar_data::ScaledMomentValue::Value(v) => {
                            value_count += 1;
                            sum += *v as f64;
                            min_val = min_val.min(*v);
                            max_val = max_val.max(*v);
                        }
                        digital_radar_data::ScaledMomentValue::BelowThreshold => below_count += 1,
                        digital_radar_data::ScaledMomentValue::RangeFolded => folded_count += 1,
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
                        min_val, max_val, avg
                    ));
                }
            }
        }
    }

    output
}

/// Converts a slice of ScaledMomentValue to a visual ASCII string representation.
/// Uses characters ordered by visual density to represent radar data values.
fn scaled_values_to_ascii(
    values: &[digital_radar_data::ScaledMomentValue],
    product: &str,
) -> String {
    // Characters ordered by increasing visual density
    const CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

    // Different scaling based on product type
    let (min_val, max_val): (f32, f32) = match product {
        "REF" => (-30.0, 75.0),  // dBZ range for reflectivity
        "VEL" => (-50.0, 50.0),  // m/s range for velocity
        "SW " => (0.0, 20.0),    // m/s range for spectrum width
        "ZDR" => (-8.0, 8.0),    // dB range for differential reflectivity
        "PHI" => (0.0, 360.0),   // degrees for differential phase
        "RHO" => (0.0, 1.05),    // unitless correlation coefficient
        "CFP" => (0.0, 20.0),    // deg/km for specific differential phase
        _ => (-30.0, 75.0),      // default
    };

    values
        .iter()
        .map(|v| match v {
            digital_radar_data::ScaledMomentValue::Value(val) => {
                // Normalize to 0.0-1.0 range, then map to character index
                let normalized = ((val - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
                let index = (normalized * (CHARS.len() - 1) as f32) as usize;
                CHARS[index]
            }
            digital_radar_data::ScaledMomentValue::BelowThreshold => ' ',
            digital_radar_data::ScaledMomentValue::RangeFolded => '~',
        })
        .collect()
}
