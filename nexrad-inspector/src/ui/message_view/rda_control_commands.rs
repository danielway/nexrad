//! RDA Control Commands (Type 6) parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays an RDA Control Commands (Type 6) message with full details.
pub fn parse_rda_control_commands(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode RDA control commands: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::RDAControlCommands(data) => data,
        _ => return "Message is not RDA control commands".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== RDA Control Commands (Type 6) ===\n\n");

    output.push_str("--- State & Mode ---\n");
    let state = msg.rda_state_command();
    output.push_str(&format!(
        "RDA State Command: {} ({})\n",
        state,
        match state {
            0 => "No change",
            32769 => "Stand-by",
            32772 => "Operate",
            32776 => "Restart",
            _ => "Unknown",
        }
    ));

    let auth = msg.rda_control_authorization();
    output.push_str(&format!(
        "Control Authorization: {} ({})\n",
        auth,
        match auth {
            0 => "No action",
            32770 => "Request remote control",
            _ => "Unknown",
        }
    ));

    output.push_str(&format!(
        "Restart VCP/Elevation Cut: {}\n",
        msg.restart_vcp_or_elevation_cut()
    ));

    output.push_str(&format!(
        "Select Local VCP Number: {}\n",
        msg.select_local_vcp_number()
    ));

    output.push_str("\n--- Features ---\n");
    let sr = msg.super_resolution_control();
    output.push_str(&format!(
        "Super Resolution: {} ({})\n",
        sr,
        match sr {
            0 => "No change",
            2 => "Enable",
            4 => "Disable",
            _ => "Unknown",
        }
    ));

    let cmd = msg.clutter_mitigation_decision_control();
    output.push_str(&format!(
        "Clutter Mitigation Decision: {} ({})\n",
        cmd,
        match cmd {
            0 => "No change",
            2 => "Enable",
            4 => "Disable",
            _ => "Unknown",
        }
    ));

    let avset = msg.avset_control();
    output.push_str(&format!(
        "AVSET: {} ({})\n",
        avset,
        match avset {
            0 => "No change",
            2 => "Enable",
            4 => "Disable",
            _ => "Unknown",
        }
    ));

    let spot = msg.spot_blanking();
    output.push_str(&format!(
        "Spot Blanking: {} ({})\n",
        spot,
        match spot {
            0 => "No change",
            2 => "Enable",
            4 => "Disable",
            _ => "Unknown",
        }
    ));

    output.push_str("\n--- Auxiliary & Calibration ---\n");
    output.push_str(&format!(
        "Aux Power Generator Control: {}\n",
        msg.auxiliary_power_generator_control()
    ));
    output.push_str(&format!(
        "Channel Control Command: {}\n",
        msg.channel_control_command()
    ));
    output.push_str(&format!(
        "Performance Check Control: {}\n",
        msg.performance_check_control()
    ));
    output.push_str(&format!("ZDR Bias Estimate: {}\n", msg.zdr_bias_estimate()));

    output.push_str("\n--- Logging ---\n");
    let log = msg.rda_log_command();
    output.push_str(&format!(
        "RDA Log Command: {} ({})\n",
        log,
        match log {
            0 => "No change",
            1 => "Enable",
            2 => "Disable",
            _ => "Unknown",
        }
    ));

    output
}
