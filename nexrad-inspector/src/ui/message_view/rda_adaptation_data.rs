//! RDA Adaptation Data (Type 18) message parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Helper to format an `Option<f32>` field.
fn fmt_f32(label: &str, val: Option<f32>, unit: &str) -> String {
    match val {
        Some(v) => format!("{}: {:.4} {}\n", label, v, unit),
        None => format!("{}: N/A\n", label),
    }
}

/// Helper to format an `Option<u32>` field.
fn fmt_u32(label: &str, val: Option<u32>, unit: &str) -> String {
    match val {
        Some(v) => format!("{}: {} {}\n", label, v, unit),
        None => format!("{}: N/A\n", label),
    }
}

/// Helper to format an `Option<i32>` field.
fn fmt_i32(label: &str, val: Option<i32>, unit: &str) -> String {
    match val {
        Some(v) => format!("{}: {} {}\n", label, v, unit),
        None => format!("{}: N/A\n", label),
    }
}

/// Helper to format an `Option<String>` field.
fn fmt_str(label: &str, val: Option<String>) -> String {
    match val {
        Some(v) => format!("{}: {}\n", label, v),
        None => format!("{}: N/A\n", label),
    }
}

/// Helper to format an `Option<f64>` field.
fn fmt_f64(label: &str, val: Option<f64>, unit: &str) -> String {
    match val {
        Some(v) => format!("{}: {:.6} {}\n", label, v, unit),
        None => format!("{}: N/A\n", label),
    }
}

/// Parses and displays an RDA Adaptation Data (Type 18) message with full details.
pub fn parse_rda_adaptation_data(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode RDA adaptation data: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::RDAAdaptationData(data) => data,
        _ => return "Message is not RDA adaptation data".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== RDA Adaptation Data (Type 18) ===\n\n");
    output.push_str(&format!("Total Size: {} bytes\n", msg.total_size()));

    // Identity
    output.push_str("\n--- Identity ---\n");
    output.push_str(&fmt_str("File Name", msg.adap_file_name()));
    output.push_str(&fmt_str("Format", msg.adap_format()));
    output.push_str(&fmt_str("Revision", msg.adap_revision()));
    output.push_str(&fmt_str("Date", msg.adap_date()));
    output.push_str(&fmt_str("Time", msg.adap_time()));

    // Site Location
    output.push_str("\n--- Site Location ---\n");
    output.push_str(&fmt_str("Site Name", msg.site_name()));
    output.push_str(&fmt_f64("Latitude", msg.site_latitude(), "\u{00b0}"));
    output.push_str(&fmt_f64("Longitude", msg.site_longitude(), "\u{00b0}"));
    output.push_str(&fmt_u32("Latitude Degrees", msg.slatdeg(), "\u{00b0}"));
    output.push_str(&fmt_u32("Latitude Minutes", msg.slatmin(), "'"));
    output.push_str(&fmt_f32("Latitude Seconds", msg.slatsec(), "\""));
    output.push_str(&fmt_str("Latitude Direction", msg.slatdir()));
    output.push_str(&fmt_u32("Longitude Degrees", msg.slondeg(), "\u{00b0}"));
    output.push_str(&fmt_u32("Longitude Minutes", msg.slonmin(), "'"));
    output.push_str(&fmt_f32("Longitude Seconds", msg.slonsec(), "\""));
    output.push_str(&fmt_str("Longitude Direction", msg.slondir()));
    output.push_str(&fmt_u32("Ground Height", msg.site_ground_height(), "m"));
    output.push_str(&fmt_u32("Radar Height", msg.site_radar_height(), "m"));

    // Antenna/Pedestal
    output.push_str("\n--- Antenna/Pedestal ---\n");
    output.push_str(&fmt_f32(
        "Lower Pre-Limit",
        msg.lower_pre_limit(),
        "\u{00b0}",
    ));
    output.push_str(&fmt_f32(
        "Upper Pre-Limit",
        msg.upper_pre_limit(),
        "\u{00b0}",
    ));
    output.push_str(&fmt_f32(
        "Lower Dead Limit",
        msg.lower_dead_limit(),
        "\u{00b0}",
    ));
    output.push_str(&fmt_f32(
        "Upper Dead Limit",
        msg.upper_dead_limit(),
        "\u{00b0}",
    ));
    output.push_str(&fmt_f32("Az Encoder Latency", msg.az_lat(), "s"));
    output.push_str(&fmt_f32("El Encoder Latency", msg.el_lat(), "s"));
    output.push_str(&fmt_f32("Park Azimuth", msg.parkaz(), "\u{00b0}"));
    output.push_str(&fmt_f32("Park Elevation", msg.parkel(), "\u{00b0}"));
    output.push_str(&fmt_f32(
        "Az Correction Factor",
        msg.az_correction_factor(),
        "\u{00b0}",
    ));
    output.push_str(&fmt_f32(
        "El Correction Factor",
        msg.el_correction_factor(),
        "\u{00b0}",
    ));
    output.push_str(&fmt_i32(
        "Manual Setup El Min",
        msg.ant_manual_setup_iel_min(),
        "",
    ));
    output.push_str(&fmt_u32(
        "Manual Setup El Max",
        msg.ant_manual_setup_iel_max(),
        "",
    ));
    output.push_str(&fmt_u32(
        "Manual Setup Az Vel Max",
        msg.ant_manual_setup_fazvelmax(),
        "",
    ));
    output.push_str(&fmt_u32(
        "Manual Setup El Vel Max",
        msg.ant_manual_setup_felvelmax(),
        "",
    ));
    output.push_str(&fmt_f32(
        "Az Pos Sustain Drive",
        msg.az_pos_sustain_drive(),
        "",
    ));
    output.push_str(&fmt_f32(
        "Az Neg Sustain Drive",
        msg.az_neg_sustain_drive(),
        "",
    ));
    output.push_str(&fmt_f32("Az Inertia", msg.az_inertia(), ""));
    output.push_str(&fmt_f32("El Inertia", msg.el_inertia(), ""));
    output.push_str(&fmt_f32("Az Stow Angle", msg.az_stow_angle(), "\u{00b0}"));
    output.push_str(&fmt_f32("El Stow Angle", msg.el_stow_angle(), "\u{00b0}"));
    output.push_str(&fmt_f32("Antenna Gain", msg.antenna_gain(), "dB"));

    // Temperature Limits
    output.push_str("\n--- Temperature Limits ---\n");
    output.push_str(&fmt_f32(
        "Min Shelter Temp",
        msg.a_min_shelter_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Max Shelter Temp",
        msg.a_max_shelter_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Min Shelter AC Temp Diff",
        msg.a_min_shelter_ac_temp_diff(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Max Transmitter Air Temp",
        msg.a_max_xmtr_air_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Max Radome Temp",
        msg.a_max_rad_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Max Radome Temp Rise",
        msg.a_max_rad_temp_rise(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Min Shelter Temp Warn",
        msg.a_min_shelter_temp_warn(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Min Gen Room Temp",
        msg.a_min_gen_room_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Max Gen Room Temp",
        msg.a_max_gen_room_temp(),
        "\u{00b0}C",
    ));

    // Generator/SPIP/Channel Config
    output.push_str("\n--- Generator/SPIP/Channel Config ---\n");
    output.push_str(&fmt_f32("SPIP 5V Reg Limit", msg.spip_5v_reg_lim(), "V"));
    output.push_str(&fmt_f32("SPIP 15V Reg Limit", msg.spip_15v_reg_lim(), "V"));
    output.push_str(&fmt_str("RPG Co-Located", msg.rpg_co_located()));
    output.push_str(&fmt_str(
        "Spec Filter Installed",
        msg.spec_filter_installed(),
    ));
    output.push_str(&fmt_str("TPS Installed", msg.tps_installed()));
    output.push_str(&fmt_str("RMS Installed", msg.rms_installed()));
    output.push_str(&fmt_u32("HVDL Test Interval", msg.a_hvdl_tst_int(), "min"));
    output.push_str(&fmt_u32("RPG LT Interval", msg.a_rpg_lt_int(), "min"));
    output.push_str(&fmt_u32(
        "Min Stab Util Power Time",
        msg.a_min_stab_util_pwr_time(),
        "min",
    ));
    output.push_str(&fmt_u32(
        "Gen Auto Exercise Interval",
        msg.a_gen_auto_exer_interval(),
        "hours",
    ));
    output.push_str(&fmt_u32(
        "Util Pwr Switch Req Interval",
        msg.a_util_pwr_sw_req_interval(),
        "min",
    ));
    output.push_str(&fmt_f32("Low Fuel Level", msg.a_low_fuel_level(), "%"));
    output.push_str(&fmt_u32(
        "Config Channel Number",
        msg.config_chan_number(),
        "",
    ));
    output.push_str(&fmt_u32(
        "Redundant Channel Config",
        msg.redundant_chan_config(),
        "",
    ));

    // RF Path Losses
    output.push_str("\n--- RF Path Losses ---\n");
    output.push_str(&fmt_f32(
        "V IF Heliax Loss",
        msg.path_loss_vertical_if_heliax(),
        "dB",
    ));
    output.push_str(&fmt_f32(
        "H IF Heliax Loss",
        msg.path_loss_horizontal_if_heliax(),
        "dB",
    ));
    output.push_str(&fmt_f32("Loss 2A9A9", msg.path_loss_2a9a9(), "dB"));
    output.push_str(&fmt_f32(
        "H Coupler XMT Loss",
        msg.h_coupler_xmt_loss(),
        "dB",
    ));
    output.push_str(&fmt_f32("Loss WG02", msg.path_loss_wg02(), "dB"));
    output.push_str(&fmt_f32(
        "Loss WG Klystron",
        msg.path_loss_waveguide_klystron(),
        "dB",
    ));
    output.push_str(&fmt_f32("Loss WG06", msg.path_loss_wg06(), "dB"));
    output.push_str(&fmt_f32("Loss WG04", msg.path_loss_wg04(), "dB"));
    output.push_str(&fmt_f32("Loss A6", msg.path_loss_a6(), "dB"));
    output.push_str(&fmt_f32("H Coupler CW Loss", msg.h_coupler_cw_loss(), "dB"));
    output.push_str(&fmt_f32(
        "V Coupler XMT Loss",
        msg.v_coupler_xmt_loss(),
        "dB",
    ));
    output.push_str(&fmt_f32("V Coupler CW Loss", msg.v_coupler_cw_loss(), "dB"));
    output.push_str(&fmt_f32("Power Sense Bias", msg.pwr_sense_bias(), "dB"));

    // Transmitter
    output.push_str("\n--- Transmitter ---\n");
    output.push_str(&fmt_u32("TX Frequency", msg.tfreq_mhz(), "MHz"));
    output.push_str(&fmt_f32("Base Data TCN", msg.base_data_tcn(), ""));
    output.push_str(&fmt_f32("Refl Data TOVER", msg.refl_data_tover(), ""));
    output.push_str(&fmt_f32("Target H dBZ0 LP", msg.tar_h_dbz0_lp(), "dBZ"));
    output.push_str(&fmt_f32("Target V dBZ0 LP", msg.tar_v_dbz0_lp(), "dBZ"));
    output.push_str(&fmt_f32("Target H dBZ0 SP", msg.tar_h_dbz0_sp(), "dBZ"));
    output.push_str(&fmt_f32("Target V dBZ0 SP", msg.tar_v_dbz0_sp(), "dBZ"));
    output.push_str(&fmt_u32("Init PHI DP", msg.init_phi_dp(), "\u{00b0}"));
    output.push_str(&fmt_u32(
        "Norm Init PHI DP",
        msg.norm_init_phi_dp(),
        "\u{00b0}",
    ));
    output.push_str(&fmt_f32("LX LP", msg.lx_lp(), "dB"));
    output.push_str(&fmt_f32("LX SP", msg.lx_sp(), "dB"));
    output.push_str(&fmt_f32("Meteor Parameter", msg.meteor_param(), ""));
    output.push_str(&fmt_f32("Vel Degrade Limit", msg.vel_degrad_limit(), ""));
    output.push_str(&fmt_f32("Width Degrade Limit", msg.wth_degrad_limit(), ""));
    output.push_str(&fmt_f32(
        "H Noise Temp Degrade Limit",
        msg.h_noisetemp_dgrad_limit(),
        "K",
    ));
    output.push_str(&fmt_u32("H Min Noise Temp", msg.h_min_noisetemp(), "K"));
    output.push_str(&fmt_f32(
        "V Noise Temp Degrade Limit",
        msg.v_noisetemp_dgrad_limit(),
        "K",
    ));
    output.push_str(&fmt_u32("V Min Noise Temp", msg.v_min_noisetemp(), "K"));
    output.push_str(&fmt_f32(
        "Klystron Degrade Limit",
        msg.kly_degrade_limit(),
        "",
    ));
    output.push_str(&fmt_f32(
        "XMTR Peak Power High Limit",
        msg.xmtr_peak_pwr_high_limit(),
        "kW",
    ));
    output.push_str(&fmt_f32(
        "XMTR Peak Power Low Limit",
        msg.xmtr_peak_pwr_low_limit(),
        "kW",
    ));
    output.push_str(&fmt_f32(
        "H dBZ0 Delta Limit",
        msg.h_dbz0_delta_limit(),
        "dB",
    ));
    output.push_str(&fmt_f32(
        "V dBZ0 Delta Limit",
        msg.v_dbz0_delta_limit(),
        "dB",
    ));
    output.push_str(&fmt_u32("Delta PRF", msg.deltaprf(), ""));
    output.push_str(&fmt_u32("Tau SP", msg.tau_sp(), "ns"));
    output.push_str(&fmt_u32("Tau LP", msg.tau_lp(), "ns"));
    output.push_str(&fmt_u32("NC Dead Value", msg.nc_dead_value(), ""));
    output.push_str(&fmt_u32("Tau RF SP", msg.tau_rf_sp(), "ns"));
    output.push_str(&fmt_u32("Tau RF LP", msg.tau_rf_lp(), "ns"));
    output.push_str(&fmt_f32(
        "XMTR Pwr Meter Scale",
        msg.xmtr_pwr_mtr_scale(),
        "",
    ));

    // Receiver/Noise
    output.push_str("\n--- Receiver/Noise ---\n");
    output.push_str(&fmt_f32("TS COHO", msg.ts_coho(), "dBm"));
    output.push_str(&fmt_f32("H TS CW", msg.h_ts_cw(), "dBm"));
    output.push_str(&fmt_f32("V TS CW", msg.v_ts_cw(), "dBm"));
    output.push_str(&fmt_f32("TS STALO", msg.ts_stalo(), "dBm"));
    output.push_str(&fmt_f32("AME H Noise ENR", msg.ame_h_noise_enr(), "dB"));
    output.push_str(&fmt_f32("AME V Noise ENR", msg.ame_v_noise_enr(), "dB"));
    output.push_str(&fmt_f32("H Noise Long", msg.h_noise_long(), "dBm"));
    output.push_str(&fmt_f32("H Noise Short", msg.h_noise_short(), "dBm"));
    output.push_str(&fmt_f32("H Noise Tolerance", msg.h_noise_tolerance(), "dB"));
    output.push_str(&fmt_f32("V Noise Long", msg.v_noise_long(), "dBm"));
    output.push_str(&fmt_f32("V Noise Short", msg.v_noise_short(), "dBm"));
    output.push_str(&fmt_f32("V Noise Tolerance", msg.v_noise_tolerance(), "dB"));
    output.push_str(&fmt_f32("Min H Dynamic Range", msg.min_h_dyn_range(), "dB"));
    output.push_str(&fmt_f32("Min V Dynamic Range", msg.min_v_dyn_range(), "dB"));
    output.push_str(&fmt_f32("Antenna Noise Temp", msg.ant_noise_temp(), "K"));
    output.push_str(&fmt_f64(
        "Dig Receiver Clock Freq",
        msg.dig_rcvr_clock_freq(),
        "Hz",
    ));
    output.push_str(&fmt_f64("COHO Frequency", msg.coho_freq(), "Hz"));

    // Calibration Thresholds
    output.push_str("\n--- Calibration Thresholds ---\n");
    output.push_str(&fmt_f32("Threshold 1", msg.threshold1(), ""));
    output.push_str(&fmt_f32("Threshold 2", msg.threshold2(), ""));
    output.push_str(&fmt_f32(
        "Clutter Supp Degrade Limit",
        msg.clut_supp_dgrad_lim(),
        "dB",
    ));
    output.push_str(&fmt_f32("Range 0 Value", msg.range0_value(), ""));
    output.push_str(&fmt_f32(
        "ZDR Offset Degrade Limit",
        msg.zdr_offset_dgrad_lim(),
        "dB",
    ));
    output.push_str(&fmt_f32(
        "Baseline ZDR Offset",
        msg.baseline_zdr_offset(),
        "dB",
    ));
    output.push_str(&fmt_f32("ZDR Data TOVER", msg.zdr_data_tover(), ""));
    output.push_str(&fmt_f32("PHI Data TOVER", msg.phi_data_tover(), ""));
    output.push_str(&fmt_f32("RHO Data TOVER", msg.rho_data_tover(), ""));
    output.push_str(&fmt_f32(
        "STALO Power Degrade Limit",
        msg.stalo_power_dgrad_limit(),
        "dBm",
    ));
    output.push_str(&fmt_f32(
        "STALO Power Maint Limit",
        msg.stalo_power_maint_limit(),
        "dBm",
    ));

    // Power Sense
    output.push_str("\n--- Power Sense ---\n");
    output.push_str(&fmt_f32("Min H Power Sense", msg.min_h_pwr_sense(), "dBm"));
    output.push_str(&fmt_f32("Min V Power Sense", msg.min_v_pwr_sense(), "dBm"));
    output.push_str(&fmt_f32(
        "H Power Sense Offset",
        msg.h_pwr_sense_offset(),
        "dB",
    ));
    output.push_str(&fmt_f32(
        "V Power Sense Offset",
        msg.v_pwr_sense_offset(),
        "dB",
    ));
    output.push_str(&fmt_f32("PS Gain Ref", msg.ps_gain_ref(), ""));
    output.push_str(&fmt_f32(
        "RF Pallet Broad Loss",
        msg.rf_pallet_broad_loss(),
        "dB",
    ));
    output.push_str(&fmt_f32("Power Meter Zero", msg.power_meter_zero(), "V"));
    output.push_str(&fmt_f32("TXB Baseline", msg.txb_baseline(), ""));
    output.push_str(&fmt_f32("TXB Alarm Threshold", msg.txb_alarm_thresh(), ""));

    // AME Limits
    output.push_str("\n--- AME Limits ---\n");
    output.push_str(&fmt_f32("AME PS Tolerance", msg.ame_ps_tolerance(), "%"));
    output.push_str(&fmt_f32("AME Max Temp", msg.ame_max_temp(), "\u{00b0}C"));
    output.push_str(&fmt_f32("AME Min Temp", msg.ame_min_temp(), "\u{00b0}C"));
    output.push_str(&fmt_f32(
        "Receiver Module Max Temp",
        msg.rcvr_mod_max_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "Receiver Module Min Temp",
        msg.rcvr_mod_min_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "BITE Module Max Temp",
        msg.bite_mod_max_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "BITE Module Min Temp",
        msg.bite_mod_min_temp(),
        "\u{00b0}C",
    ));
    output.push_str(&fmt_f32(
        "AME Current Tolerance",
        msg.ame_current_tolerance(),
        "A",
    ));
    output.push_str(&fmt_f32(
        "TR Limit Degrade Limit",
        msg.tr_limit_dgrad_limit(),
        "",
    ));
    output.push_str(&fmt_f32(
        "TR Limit Fail Limit",
        msg.tr_limit_fail_limit(),
        "",
    ));

    // Polarization Config
    output.push_str("\n--- Polarization Config ---\n");
    output.push_str(&fmt_u32(
        "Default Polarization",
        msg.default_polarization(),
        "",
    ));
    output.push_str(&fmt_u32(
        "H Only Polarization",
        msg.h_only_polarization(),
        "",
    ));
    output.push_str(&fmt_u32(
        "V Only Polarization",
        msg.v_only_polarization(),
        "",
    ));
    output.push_str(&fmt_str("RFP Stepper Enabled", msg.rfp_stepper_enabled()));

    // Doppler Range/Segments
    output.push_str("\n--- Doppler Range/Segments ---\n");
    output.push_str(&fmt_f32(
        "Doppler Range Start",
        msg.doppler_range_start(),
        "km",
    ));
    output.push_str(&fmt_u32("Max El Index", msg.max_el_index(), ""));
    output.push_str(&fmt_f32("Seg1 Limit", msg.seg1lim(), "km"));
    output.push_str(&fmt_f32("Seg2 Limit", msg.seg2lim(), "km"));
    output.push_str(&fmt_f32("Seg3 Limit", msg.seg3lim(), "km"));
    output.push_str(&fmt_f32("Seg4 Limit", msg.seg4lim(), "km"));
    output.push_str(&fmt_u32("Nbr El Segments", msg.nbr_el_segments(), ""));

    // Misc
    output.push_str("\n--- Misc ---\n");
    output.push_str(&fmt_f32("Sun Bias", msg.sun_bias(), "dB"));

    // Fuel level conversion
    let fuel_conv = msg.a_fuel_conv();
    if !fuel_conv.is_empty() {
        output.push_str("\n--- Fuel Level Conversion ---\n");
        for (i, val) in fuel_conv.iter().enumerate() {
            output.push_str(&format!("  {}%: {:.2}\n", i * 10, val));
        }
    }

    output
}
