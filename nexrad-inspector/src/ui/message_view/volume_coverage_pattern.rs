//! Volume Coverage Pattern (Type 5) message parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

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

    let header = msg.header();

    let mut output = String::new();

    output.push_str("=== Volume Coverage Pattern (Type 5) ===\n\n");

    // Header Section
    output.push_str("--- Header ---\n");
    output.push_str(&format!("Pattern Number: {}\n", header.pattern_number()));
    output.push_str(&format!("Pattern Type: {:?}\n", header.pattern_type()));
    output.push_str(&format!(
        "Number of Elevation Cuts: {}\n",
        header.number_of_elevation_cuts()
    ));
    output.push_str(&format!("Version: {}\n", header.version()));
    output.push_str(&format!(
        "Clutter Map Group: {}\n",
        header.clutter_map_group_number()
    ));

    // Doppler velocity resolution
    let vel = header.doppler_velocity_resolution();
    if vel > 0.0 {
        output.push_str(&format!("Doppler Velocity Resolution: {:.2} m/s\n", vel));
    } else {
        output.push_str(&format!(
            "Doppler Velocity Resolution: {} (raw)\n",
            header.doppler_velocity_resolution_raw()
        ));
    }

    output.push_str(&format!("Pulse Width: {:?}\n", header.pulse_width()));
    output.push_str(&format!(
        "Message Size: {} half-words\n",
        header.message_size()
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
        if header.vcp_sequencing_truncated() {
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
        header.vcp_sequencing_max_sails_cuts()
    ));

    // VCP Supplemental Data
    output.push_str("\n--- Supplemental Data ---\n");
    output.push_str(&format!(
        "SAILS VCP: {} ({} cuts)\n",
        if header.is_sails_vcp() {
            "Yes"
        } else {
            "No"
        },
        header.number_of_sails_cuts()
    ));
    output.push_str(&format!(
        "MRLE VCP: {} ({} cuts)\n",
        if header.is_mrle_vcp() {
            "Yes"
        } else {
            "No"
        },
        header.number_of_mrle_cuts()
    ));
    output.push_str(&format!(
        "MPDA VCP: {}\n",
        if header.is_mpda_vcp() {
            "Yes"
        } else {
            "No"
        }
    ));
    output.push_str(&format!(
        "Base Tilt VCP: {} ({} tilts)\n",
        if header.is_base_tilt_vcp() {
            "Yes"
        } else {
            "No"
        },
        header.number_of_base_tilts()
    ));

    // Elevation Cuts
    output.push_str(&format!(
        "\n--- Elevation Cuts ({}) ---\n",
        msg.elevations().len()
    ));

    for (i, elev) in msg.elevations().iter().enumerate() {
        output.push_str(&format!(
            "\n[Cut {}] {:.2}\u{00b0}\n",
            i + 1,
            elev.elevation_angle()
        ));
        output.push_str(&format!(
            "  Channel Config: {:?}\n",
            elev.channel_configuration()
        ));
        output.push_str(&format!("  Waveform: {:?}\n", elev.waveform_type()));
        output.push_str(&format!(
            "  Azimuth Rate: {:.2}\u{00b0}/s\n",
            elev.azimuth_rate()
        ));

        // Super Resolution
        let sr_az = elev.super_resolution_half_degree_azimuth();
        let sr_ref = elev.super_resolution_quarter_km_reflectivity();
        let sr_dop = elev.super_resolution_doppler_to_300km();
        let sr_dp = elev.super_resolution_dual_pol_to_300km();
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
        let s1 = elev.sector_1_edge_angle();
        let s2 = elev.sector_2_edge_angle();
        let s3 = elev.sector_3_edge_angle();
        if s1 != 0.0 || s2 != 0.0 || s3 != 0.0 {
            output.push_str(&format!(
                "  Sector Edges: S1={:.1}\u{00b0} S2={:.1}\u{00b0} S3={:.1}\u{00b0}\n",
                s1, s2, s3
            ));
        }

        // EBC angle
        let ebc = elev.ebc_angle();
        if ebc != 0.0 {
            output.push_str(&format!("  EBC Angle: {:.1}\u{00b0}\n", ebc));
        }

        // Special cut types
        let mut special = Vec::new();
        if elev.is_sails_cut() {
            special.push(format!(
                "SAILS(seq={})",
                elev.sails_sequence_number()
            ));
        }
        if elev.is_mrle_cut() {
            special.push(format!(
                "MRLE(seq={})",
                elev.mrle_sequence_number()
            ));
        }
        if elev.is_mpda_cut() {
            special.push("MPDA".to_string());
        }
        if elev.is_base_tilt_cut() {
            special.push("Base Tilt".to_string());
        }
        if !special.is_empty() {
            output.push_str(&format!("  Special: {}\n", special.join(", ")));
        }
    }

    output
}

/// Fallback parser using only the header when full decode fails.
fn parse_header_only(_data: &[u8]) -> String {
    "Failed to parse Volume Coverage Pattern message".to_string()
}
