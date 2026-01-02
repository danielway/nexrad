/// Contains summary information from the Volume Coverage Pattern message
#[derive(Clone, PartialEq, Debug)]
pub struct VCPInfo {
    pub pattern_number: u16,
    pub version: u8,
    pub number_of_elevation_cuts: u16,
    pub pulse_width: String,
    pub doppler_velocity_resolution: Option<f64>,
    pub vcp_features: Vec<String>,
    pub elevations: Vec<VCPElevationInfo>,
}

/// Summary information about a single elevation cut in a VCP
#[derive(Clone, PartialEq, Debug)]
pub struct VCPElevationInfo {
    pub elevation_angle: f64,
    pub channel_configuration: String,
    pub waveform_type: String,
    pub azimuth_rate: f64,
    pub super_resolution_features: Vec<String>,
    pub special_cut_type: Option<String>,
}

/// Helper function to extract Volume Coverage Pattern information from a message
pub fn extract_vcp_info(message: &crate::messages::volume_coverage_pattern::Message) -> VCPInfo {
    let mut vcp_features = Vec::new();

    if message.header.vcp_supplemental_data_sails_vcp() {
        let sails_cuts = message.header.vcp_supplemental_data_number_sails_cuts();
        vcp_features.push(format!("SAILS ({sails_cuts} cuts)"));
    }

    if message.header.vcp_supplemental_data_mrle_vcp() {
        let mrle_cuts = message.header.vcp_supplemental_data_number_mrle_cuts();
        vcp_features.push(format!("MRLE ({mrle_cuts} cuts)"));
    }

    if message.header.vcp_supplemental_data_mpda_vcp() {
        vcp_features.push("MPDA".to_string());
    }

    if message.header.vcp_supplemental_data_base_tilt_vcp() {
        let base_tilts = message.header.vcp_supplemental_data_base_tilts();
        vcp_features.push(format!("Base tilts ({base_tilts} cuts)"));
    }

    if message.header.vcp_sequencing_sequence_active() {
        vcp_features.push("VCP sequence active".to_string());
    }

    if message.header.vcp_sequencing_truncated_vcp() {
        vcp_features.push("Truncated VCP".to_string());
    }

    let mut elevations = Vec::new();
    for elev in message.elevations.iter() {
        let mut super_res_features = Vec::new();
        if elev.super_resolution_control_half_degree_azimuth() {
            super_res_features.push("0.5° azimuth".to_string());
        }
        if elev.super_resolution_control_quarter_km_reflectivity() {
            super_res_features.push("0.25 km reflectivity".to_string());
        }
        if elev.super_resolution_control_doppler_to_300km() {
            super_res_features.push("Doppler to 300 km".to_string());
        }
        if elev.super_resolution_control_dual_polarization_to_300km() {
            super_res_features.push("Dual pol to 300 km".to_string());
        }

        // Determine special cut type
        let mut special_cut_type = None;
        if elev.supplemental_data_sails_cut() {
            let seq = elev.supplemental_data_sails_sequence_number();
            special_cut_type = Some(format!("SAILS {seq}"));
        } else if elev.supplemental_data_mrle_cut() {
            let seq = elev.supplemental_data_mrle_sequence_number();
            special_cut_type = Some(format!("MRLE {seq}"));
        } else if elev.supplemental_data_mpda_cut() {
            special_cut_type = Some("MPDA".to_string());
        } else if elev.supplemental_data_base_tilt_cut() {
            special_cut_type = Some("Base tilt".to_string());
        }

        elevations.push(VCPElevationInfo {
            elevation_angle: elev.elevation_angle_degrees(),
            channel_configuration: format!("{:?}", elev.channel_configuration()),
            waveform_type: format!("{:?}", elev.waveform_type()),
            azimuth_rate: elev.azimuth_rate_degrees_per_second(),
            super_resolution_features: super_res_features,
            special_cut_type,
        });
    }

    VCPInfo {
        pattern_number: message.header.pattern_number.get(),
        version: message.header.version,
        number_of_elevation_cuts: message.header.number_of_elevation_cuts.get(),
        pulse_width: format!("{:?}", message.header.pulse_width()),
        doppler_velocity_resolution: message
            .header
            .doppler_velocity_resolution_meters_per_second(),
        vcp_features,
        elevations,
    }
}
