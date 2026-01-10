//! Volume Coverage Pattern (Type 5) message parsing and display.

use nexrad_decode::messages::{decode_messages, volume_coverage_pattern, MessageContents};

/// Parses and displays a Volume Coverage Pattern (Type 5) message with full details.
pub fn parse_volume_coverage_pattern(data: &[u8]) -> String {
    // Try to use decode_messages for full parsing with elevation data
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(_) => {
            // Fall back to header-only parsing
            return parse_header_only(data);
        }
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return parse_header_only(data),
    };

    let msg = match message.contents() {
        MessageContents::VolumeCoveragePattern(vcp) => vcp,
        _ => return parse_header_only(data),
    };

    let header = &msg.header;

    let mut output = String::new();

    output.push_str("=== Volume Coverage Pattern (Type 5) ===\n\n");

    // Header Section
    output.push_str("--- Header ---\n");
    output.push_str(&format!("Pattern Number: {}\n", header.pattern_number));
    output.push_str(&format!("Pattern Type: {:?}\n", header.pattern_type()));
    output.push_str(&format!(
        "Number of Elevation Cuts: {}\n",
        header.number_of_elevation_cuts
    ));
    output.push_str(&format!("Version: {}\n", header.version));
    output.push_str(&format!(
        "Clutter Map Group: {}\n",
        header.clutter_map_group_number
    ));

    // Doppler velocity resolution
    if let Some(vel) = header.doppler_velocity_resolution_meters_per_second() {
        output.push_str(&format!("Doppler Velocity Resolution: {:.2} m/s\n", vel));
    } else {
        output.push_str(&format!(
            "Doppler Velocity Resolution: {} (raw)\n",
            header.doppler_velocity_resolution
        ));
    }

    output.push_str(&format!("Pulse Width: {:?}\n", header.pulse_width()));
    output.push_str(&format!(
        "Message Size: {} half-words\n",
        header.message_size
    ));

    // VCP Sequencing
    output.push_str("\n--- VCP Sequencing ---\n");
    output.push_str(&format!(
        "Sequence Active: {}\n",
        if header.vcp_sequencing_sequence_active() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Truncated VCP: {}\n",
        if header.vcp_sequencing_truncated_vcp() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Number of Elevations: {}\n",
        header.vcp_sequencing_number_of_elevations()
    ));
    output.push_str(&format!(
        "Max SAILS Cuts: {}\n",
        header.vcp_sequencing_maximum_sails_cuts()
    ));

    // VCP Supplemental Data
    output.push_str("\n--- Supplemental Data ---\n");
    output.push_str(&format!(
        "SAILS VCP: {} ({} cuts)\n",
        if header.vcp_supplemental_data_sails_vcp() {
            "Yes"
        } else {
            "No"
        },
        header.vcp_supplemental_data_number_sails_cuts()
    ));
    output.push_str(&format!(
        "MRLE VCP: {} ({} cuts)\n",
        if header.vcp_supplemental_data_mrle_vcp() {
            "Yes"
        } else {
            "No"
        },
        header.vcp_supplemental_data_number_mrle_cuts()
    ));
    output.push_str(&format!(
        "MPDA VCP: {}\n",
        if header.vcp_supplemental_data_mpda_vcp() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Base Tilt VCP: {} ({} tilts)\n",
        if header.vcp_supplemental_data_base_tilt_vcp() {
            "Yes"
        } else {
            "No"
        },
        header.vcp_supplemental_data_base_tilts()
    ));

    // Elevation Cuts
    output.push_str(&format!(
        "\n--- Elevation Cuts ({}) ---\n",
        msg.elevations.len()
    ));

    for (i, elev) in msg.elevations.iter().enumerate() {
        output.push_str(&format!(
            "\n[Cut {}] {:.2}\u{00b0}\n",
            i + 1,
            elev.elevation_angle_degrees()
        ));
        output.push_str(&format!(
            "  Channel Config: {:?}\n",
            elev.channel_configuration()
        ));
        output.push_str(&format!("  Waveform: {:?}\n", elev.waveform_type()));
        output.push_str(&format!(
            "  Azimuth Rate: {:.2}\u{00b0}/s\n",
            elev.azimuth_rate_degrees_per_second()
        ));

        // Super Resolution
        let sr_az = elev.super_resolution_control_half_degree_azimuth();
        let sr_ref = elev.super_resolution_control_quarter_km_reflectivity();
        let sr_dop = elev.super_resolution_control_doppler_to_300km();
        let sr_dp = elev.super_resolution_control_dual_polarization_to_300km();
        if sr_az || sr_ref || sr_dop || sr_dp {
            output.push_str("  Super Resolution: ");
            let mut sr_flags = Vec::new();
            if sr_az {
                sr_flags.push("0.5\u{00b0} Az");
            }
            if sr_ref {
                sr_flags.push("0.25km REF");
            }
            if sr_dop {
                sr_flags.push("DOP 300km");
            }
            if sr_dp {
                sr_flags.push("DP 300km");
            }
            output.push_str(&sr_flags.join(", "));
            output.push('\n');
        }

        // Thresholds
        output.push_str(&format!(
            "  Thresholds: REF={:.1} VEL={:.1} SW={:.1}\n",
            elev.reflectivity_threshold(),
            elev.velocity_threshold(),
            elev.spectrum_width_threshold()
        ));
        output.push_str(&format!(
            "              ZDR={:.1} PHI={:.1} RHO={:.1}\n",
            elev.differential_reflectivity_threshold(),
            elev.differential_phase_threshold(),
            elev.correlation_coefficient_threshold()
        ));

        // Sector edges
        let s1 = elev.sector_1_edge_angle_degrees();
        let s2 = elev.sector_2_edge_angle_degrees();
        let s3 = elev.sector_3_edge_angle_degrees();
        if s1 != 0.0 || s2 != 0.0 || s3 != 0.0 {
            output.push_str(&format!(
                "  Sector Edges: S1={:.1}\u{00b0} S2={:.1}\u{00b0} S3={:.1}\u{00b0}\n",
                s1, s2, s3
            ));
        }

        // EBC angle
        let ebc = elev.ebc_angle_degrees();
        if ebc != 0.0 {
            output.push_str(&format!("  EBC Angle: {:.1}\u{00b0}\n", ebc));
        }

        // Special cut types
        let mut special = Vec::new();
        if elev.supplemental_data_sails_cut() {
            special.push(format!(
                "SAILS(seq={})",
                elev.supplemental_data_sails_sequence_number()
            ));
        }
        if elev.supplemental_data_mrle_cut() {
            special.push(format!(
                "MRLE(seq={})",
                elev.supplemental_data_mrle_sequence_number()
            ));
        }
        if elev.supplemental_data_mpda_cut() {
            special.push("MPDA".to_string());
        }
        if elev.supplemental_data_base_tilt_cut() {
            special.push("Base Tilt".to_string());
        }
        if !special.is_empty() {
            output.push_str(&format!("  Special: {}\n", special.join(", ")));
        }
    }

    output
}

/// Fallback parser using only the header when full decode fails.
fn parse_header_only(data: &[u8]) -> String {
    use zerocopy::FromBytes;

    let header = match volume_coverage_pattern::Header::ref_from_prefix(data) {
        Ok((h, _)) => h,
        Err(_) => return "Failed to parse Volume Coverage Pattern header".to_string(),
    };

    format!(
        "=== Volume Coverage Pattern (Type 5) ===\n\n\
         Message Size: {} half-words\n\
         Pattern Type: {:?}\n\
         Pattern Number: {}\n\
         Number of Elevation Cuts: {}\n\
         Version: {}\n\
         Clutter Map Group: {}\n\
         Doppler Velocity Resolution: {}\n\
         Pulse Width: {:?}\n\
         VCP Sequencing: 0x{:04X}\n\
         VCP Supplemental Data: 0x{:04X}\n\n\
         (Elevation cut details not available - decode failed)",
        header.message_size,
        header.pattern_type(),
        header.pattern_number,
        header.number_of_elevation_cuts,
        header.version,
        header.clutter_map_group_number,
        header.doppler_velocity_resolution,
        header.pulse_width(),
        header.vcp_sequencing.get(),
        header.vcp_supplemental_data.get(),
    )
}
