//! RDA Status Data (Type 2) message parsing and display.

use nexrad_decode::messages::{decode_messages, rda_status_data, MessageContents};

/// Parses and displays an RDA Status Data (Type 2) message with full details.
pub fn parse_rda_status_data(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(_) => return "Failed to decode RDA Status Data".to_string(),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let message = match message.contents() {
        MessageContents::RDAStatusData(data) => data,
        _ => return "Message is not RDA Status Data".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== RDA Status Data (Type 2) ===\n\n");

    // System Status Section
    output.push_str("--- System Status ---\n");
    output.push_str(&format!("RDA Status: {:?}\n", message.rda_status()));
    output.push_str(&format!(
        "Operability: {:?}\n",
        message.operability_status()
    ));
    output.push_str(&format!("Control Status: {:?}\n", message.control_status()));
    output.push_str(&format!(
        "Operational Mode: {:?}\n",
        message.operational_mode()
    ));
    output.push_str(&format!(
        "Control Authorization: {:?}\n",
        message.rda_control_authorization()
    ));
    output.push_str(&format!(
        "Controlling Channel: {}\n",
        if message.controlling_channel() {
            "Yes"
        } else {
            "No"
        }
    ));

    // Power & Calibration Section
    output.push_str("\n--- Power & Calibration ---\n");
    output.push_str(&format!(
        "Avg TX Power: {} watts\n",
        message.average_transmitter_power()
    ));
    output.push_str(&format!(
        "Aux Power Generator: {:?}\n",
        message.auxiliary_power_generator_state()
    ));
    output.push_str(&format!(
        "TPS Status: {:?}\n",
        message.transition_power_source_status()
    ));
    output.push_str(&format!(
        "Horiz Reflectivity Cal: {:.2} dB\n",
        message.horizontal_reflectivity_calibration_correction()
    ));
    output.push_str(&format!(
        "Vert Reflectivity Cal: {:.2} dB\n",
        message.raw_vertical_reflectivity_calibration_correction() as f32 / 100.0
    ));

    // Volume Coverage Pattern Section
    output.push_str("\n--- Volume Coverage Pattern ---\n");
    if let Some(vcp) = message.volume_coverage_pattern() {
        output.push_str(&format!(
            "VCP Number: {} ({})\n",
            vcp.number(),
            if vcp.remote() { "remote" } else { "local" }
        ));
    } else {
        output.push_str("VCP: None\n");
    }

    // Data Transmission Section
    output.push_str("\n--- Data Transmission ---\n");
    let tx = message.data_transmission_enabled();
    output.push_str(&format!(
        "TX Enabled: REF={} VEL={} SW={}\n",
        if tx.reflectivity() { "Yes" } else { "No" },
        if tx.velocity() { "Yes" } else { "No" },
        if tx.spectrum_width() { "Yes" } else { "No" }
    ));

    // Features Section
    output.push_str("\n--- Features ---\n");
    output.push_str(&format!(
        "Super Resolution: {:?}\n",
        message.super_resolution_status()
    ));
    output.push_str(&format!(
        "Clutter Mitigation: {:?}\n",
        message.clutter_mitigation_decision_status()
    ));
    output.push_str(&format!(
        "Spot Blanking: {:?}\n",
        message.spot_blanking_status()
    ));

    // Scan & Data Flags Section
    output.push_str("\n--- Scan & Data Flags ---\n");
    let flags = message.rda_scan_and_data_flags();
    output.push_str(&format!(
        "AVSET: {}\n",
        if flags.avset_enabled() {
            "Enabled"
        } else {
            "Disabled"
        }
    ));
    output.push_str(&format!(
        "EBC: {}\n",
        if flags.ebc_enabled() {
            "Enabled"
        } else {
            "Disabled"
        }
    ));
    output.push_str(&format!(
        "RDA Log Data: {}\n",
        if flags.rda_log_data_enabled() {
            "Enabled"
        } else {
            "Disabled"
        }
    ));
    output.push_str(&format!(
        "Time Series Recording: {}\n",
        if flags.time_series_data_recording_enabled() {
            "Enabled"
        } else {
            "Disabled"
        }
    ));

    // Control Section
    output.push_str("\n--- Control ---\n");
    output.push_str(&format!(
        "RMS Control: {:?}\n",
        message.rms_control_status()
    ));
    output.push_str(&format!(
        "Performance Check: {:?}\n",
        message.performance_check_status()
    ));
    if let Some(ack) = message.command_acknowledgement() {
        output.push_str(&format!("Command Ack: {:?}\n", ack));
    } else {
        output.push_str("Command Ack: None\n");
    }

    // Map Generation Times Section
    output.push_str("\n--- Map Generation Times ---\n");
    if let Some(dt) = message.bypass_map_generation_date_time() {
        output.push_str(&format!(
            "Bypass Map: {}\n",
            dt.format("%Y-%m-%d %H:%M UTC")
        ));
    } else {
        output.push_str("Bypass Map: Not set\n");
    }
    if let Some(dt) = message.clutter_filter_map_generation_date_time() {
        output.push_str(&format!(
            "Clutter Filter Map: {}\n",
            dt.format("%Y-%m-%d %H:%M UTC")
        ));
    } else {
        output.push_str("Clutter Filter Map: Not set\n");
    }

    // Alarms Section
    output.push_str("\n--- Alarms ---\n");
    let alarm_summary = message.rda_alarm_summary();
    output.push_str(&format!(
        "Alarm Categories: {}\n",
        format_alarm_summary(&alarm_summary)
    ));

    let alarms = message.alarm_messages();
    if alarms.is_empty() {
        output.push_str("Active Alarms: None\n");
    } else {
        output.push_str(&format!("Active Alarms ({}):\n", alarms.len()));
        for alarm in &alarms {
            let device = alarm
                .device()
                .map(|d| format!("{:?}", d))
                .unwrap_or_else(|| "Unknown".to_string());
            output.push_str(&format!(
                "  [{}] Code {}: {}\n",
                device,
                alarm.code(),
                alarm.message()
            ));
        }
    }

    // Build & Version Section
    output.push_str("\n--- Build & Version ---\n");
    output.push_str(&format!(
        "RDA Build Number: {:.1}\n",
        message.rda_build_number()
    ));
    output.push_str(&format!(
        "Status Message Version: {}\n",
        message.status_version()
    ));
    output.push_str(&format!(
        "Signal Processor Options: 0x{:04X}\n",
        message.raw_signal_processor_options()
    ));

    output
}

/// Formats the alarm summary flags into a readable string.
fn format_alarm_summary(summary: &rda_status_data::alarm::Summary) -> String {
    let mut categories = Vec::new();

    if summary.tower_utilities() {
        categories.push("Tower/Utilities");
    }
    if summary.pedestal() {
        categories.push("Pedestal");
    }
    if summary.transmitter() {
        categories.push("Transmitter");
    }
    if summary.receiver() {
        categories.push("Receiver");
    }
    if summary.rda_control() {
        categories.push("RDA Control");
    }
    if summary.communication() {
        categories.push("Communication");
    }
    if summary.signal_processor() {
        categories.push("Signal Processor");
    }

    if categories.is_empty() {
        "None".to_string()
    } else {
        categories.join(", ")
    }
}
