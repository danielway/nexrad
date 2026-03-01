//! Performance/Maintenance Data (Type 3) message parsing and display.

use nexrad_decode::messages::{decode_messages, MessageContents};

/// Parses and displays a Performance/Maintenance Data (Type 3) message with full details.
pub fn parse_performance_maintenance_data(data: &[u8]) -> String {
    let messages = match decode_messages(data) {
        Ok(m) => m,
        Err(e) => return format!("Failed to decode performance/maintenance data: {:?}", e),
    };

    let message = match messages.first() {
        Some(m) => m,
        None => return "No messages decoded".to_string(),
    };

    let msg = match message.contents() {
        MessageContents::PerformanceMaintenanceData(data) => data,
        _ => return "Message is not performance/maintenance data".to_string(),
    };

    let mut output = String::new();

    output.push_str("=== Performance/Maintenance Data (Type 3) ===\n\n");
    output.push_str(&format!("Version: {}\n", msg.version()));

    // Communications
    output.push_str("\n--- Communications ---\n");
    output.push_str(&format!(
        "Loop Back Test Status: {} ({})\n",
        msg.loop_back_test_status(),
        match msg.loop_back_test_status() {
            0 => "Pass",
            1 => "Fail",
            2 => "Timeout",
            3 => "Not Tested",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!("T1 Output Frames: {}\n", msg.t1_output_frames()));
    output.push_str(&format!("T1 Input Frames: {}\n", msg.t1_input_frames()));
    output.push_str(&format!(
        "Router Memory Used: {} bytes\n",
        msg.router_memory_used()
    ));
    output.push_str(&format!(
        "Router Memory Free: {} bytes\n",
        msg.router_memory_free()
    ));
    output.push_str(&format!(
        "Router Memory Utilization: {}%\n",
        msg.router_memory_utilization()
    ));
    output.push_str(&format!(
        "Route to RPG: {} ({})\n",
        msg.route_to_rpg(),
        match msg.route_to_rpg() {
            0 => "Normal",
            1 => "Backup in Use",
            2 => "Down Failure",
            3 => "Backup Commanded Down",
            4 => "Not Installed",
            _ => "Unknown",
        }
    ));

    let port_status = |v: u16| match v {
        1 => "Up",
        2 => "Down",
        3 => "Test",
        _ => "Unknown",
    };
    output.push_str(&format!(
        "T1 Port Status: {} ({})\n",
        msg.t1_port_status(),
        port_status(msg.t1_port_status())
    ));
    output.push_str(&format!(
        "Router Dedicated Ethernet: {} ({})\n",
        msg.router_dedicated_ethernet_port_status(),
        port_status(msg.router_dedicated_ethernet_port_status())
    ));
    output.push_str(&format!(
        "Router Commercial Ethernet: {} ({})\n",
        msg.router_commercial_ethernet_port_status(),
        port_status(msg.router_commercial_ethernet_port_status())
    ));

    output.push_str("\n  CSU 24-Hour Statistics:\n");
    output.push_str(&format!(
        "    Errored Seconds: {}\n",
        msg.csu_24hr_errored_seconds()
    ));
    output.push_str(&format!(
        "    Severely Errored Seconds: {}\n",
        msg.csu_24hr_severely_errored_seconds()
    ));
    output.push_str(&format!(
        "    Severely Errored Framing Seconds: {}\n",
        msg.csu_24hr_severely_errored_framing_seconds()
    ));
    output.push_str(&format!(
        "    Unavailable Seconds: {}\n",
        msg.csu_24hr_unavailable_seconds()
    ));
    output.push_str(&format!(
        "    Controlled Slip Seconds: {}\n",
        msg.csu_24hr_controlled_slip_seconds()
    ));
    output.push_str(&format!(
        "    Path Coding Violations: {}\n",
        msg.csu_24hr_path_coding_violations()
    ));
    output.push_str(&format!(
        "    Line Errored Seconds: {}\n",
        msg.csu_24hr_line_errored_seconds()
    ));
    output.push_str(&format!(
        "    Bursty Errored Seconds: {}\n",
        msg.csu_24hr_bursty_errored_seconds()
    ));
    output.push_str(&format!(
        "    Degraded Minutes: {}\n",
        msg.csu_24hr_degraded_minutes()
    ));

    output.push_str(&format!(
        "LAN Switch CPU Utilization: {}%\n",
        msg.lan_switch_cpu_utilization()
    ));
    output.push_str(&format!(
        "LAN Switch Memory Utilization: {}%\n",
        msg.lan_switch_memory_utilization()
    ));
    output.push_str(&format!(
        "IFDR Chassis Temp: {}\u{00b0}C\n",
        msg.ifdr_chassis_temperature()
    ));
    output.push_str(&format!(
        "IFDR FPGA Temp: {}\u{00b0}C\n",
        msg.ifdr_fpga_temperature()
    ));
    output.push_str(&format!(
        "NTP Status: {} ({})\n",
        msg.ntp_status(),
        match msg.ntp_status() {
            0 => "OK",
            1 => "Fail",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "IPC Status: {} ({})\n",
        msg.ipc_status(),
        match msg.ipc_status() {
            0 => "OK",
            1 => "Fail",
            2 => "N/A",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Commanded Channel Control: {} ({})\n",
        msg.commanded_channel_control(),
        match msg.commanded_channel_control() {
            0 => "N/A",
            1 => "Channel 1",
            2 => "Channel 2",
            _ => "Unknown",
        }
    ));

    // AME
    output.push_str("\n--- AME ---\n");
    output.push_str(&format!(
        "Polarization: {} ({})\n",
        msg.polarization(),
        match msg.polarization() {
            0 => "H Only",
            1 => "H+V",
            2 => "V Only",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "AME Internal Temp: {:.1}\u{00b0}C\n",
        msg.ame_internal_temperature()
    ));
    output.push_str(&format!(
        "AME Receiver Module Temp: {:.1}\u{00b0}C\n",
        msg.ame_receiver_module_temperature()
    ));
    output.push_str(&format!(
        "AME BITE/CAL Module Temp: {:.1}\u{00b0}C\n",
        msg.ame_bite_cal_module_temperature()
    ));
    output.push_str(&format!(
        "AME Peltier PWM: {}%\n",
        msg.ame_peltier_pulse_width_modulation()
    ));
    output.push_str(&format!(
        "AME Peltier Status: {} ({})\n",
        msg.ame_peltier_status(),
        match msg.ame_peltier_status() {
            0 => "OFF",
            1 => "ON",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "AME A/D Converter: {} ({})\n",
        msg.ame_ad_converter_status(),
        match msg.ame_ad_converter_status() {
            0 => "OK",
            1 => "FAIL",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "AME State: {} ({})\n",
        msg.ame_state(),
        match msg.ame_state() {
            0 => "START",
            1 => "RUNNING",
            2 => "FLASH",
            3 => "ERROR",
            _ => "Unknown",
        }
    ));

    output.push_str("\n  AME Power Supply Voltages:\n");
    output.push_str(&format!("    3.3V: {:.3} V\n", msg.ame_3_3v_ps_voltage()));
    output.push_str(&format!("    5V: {:.3} V\n", msg.ame_5v_ps_voltage()));
    output.push_str(&format!("    6.5V: {:.3} V\n", msg.ame_6_5v_ps_voltage()));
    output.push_str(&format!("    15V: {:.3} V\n", msg.ame_15v_ps_voltage()));
    output.push_str(&format!("    48V: {:.3} V\n", msg.ame_48v_ps_voltage()));
    output.push_str(&format!(
        "    STALO Power: {:.3} V\n",
        msg.ame_stalo_power()
    ));

    output.push_str(&format!(
        "Peltier Current: {:.3} A\n",
        msg.peltier_current()
    ));
    output.push_str(&format!(
        "ADC Cal Reference Voltage: {:.3} V\n",
        msg.adc_calibration_reference_voltage()
    ));
    output.push_str(&format!(
        "AME Mode: {} ({})\n",
        msg.ame_mode(),
        match msg.ame_mode() {
            0 => "READY",
            1 => "MAINTENANCE",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "AME Peltier Mode: {} ({})\n",
        msg.ame_peltier_mode(),
        match msg.ame_peltier_mode() {
            0 => "COOL",
            1 => "HEAT",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "AME Peltier Inside Fan: {:.3} A\n",
        msg.ame_peltier_inside_fan_current()
    ));
    output.push_str(&format!(
        "AME Peltier Outside Fan: {:.3} A\n",
        msg.ame_peltier_outside_fan_current()
    ));
    output.push_str(&format!(
        "H TR Limiter Voltage: {:.3} V\n",
        msg.horizontal_tr_limiter_voltage()
    ));
    output.push_str(&format!(
        "V TR Limiter Voltage: {:.3} V\n",
        msg.vertical_tr_limiter_voltage()
    ));
    output.push_str(&format!(
        "ADC Cal Offset Voltage: {:.3} mV\n",
        msg.adc_calibration_offset_voltage()
    ));
    output.push_str(&format!(
        "ADC Cal Gain Correction: {:.6}\n",
        msg.adc_calibration_gain_correction()
    ));
    output.push_str(&format!(
        "RCP Status: {} ({})\n",
        msg.rcp_status(),
        match msg.rcp_status() {
            0 => "OK",
            1 => "NOT OK",
            _ => "Unknown",
        }
    ));

    let rcp_str = msg.rcp_string();
    let rcp_text = String::from_utf8_lossy(rcp_str);
    output.push_str(&format!(
        "RCP String: {}\n",
        rcp_text.trim_end_matches('\0')
    ));
    output.push_str(&format!(
        "SPIP Power Buttons: 0x{:04X}\n",
        msg.spip_power_buttons()
    ));

    // Power
    output.push_str("\n--- Power ---\n");
    output.push_str(&format!(
        "Master Power Admin Load: {:.2} A\n",
        msg.master_power_administrator_load()
    ));
    output.push_str(&format!(
        "Expansion Power Admin Load: {:.2} A\n",
        msg.expansion_power_administrator_load()
    ));

    // Transmitter
    output.push_str("\n--- Transmitter ---\n");
    output.push_str(&format!(
        "Power Supplies: +5V={} +15V={} +28V={} -15V={} +45V={}\n",
        msg.plus_5_vdc_ps(),
        msg.plus_15_vdc_ps(),
        msg.plus_28_vdc_ps(),
        msg.minus_15_vdc_ps(),
        msg.plus_45_vdc_ps()
    ));
    output.push_str(&format!(
        "Filament PS Voltage: {}\n",
        msg.filament_ps_voltage()
    ));
    output.push_str(&format!(
        "Vacuum Pump PS Voltage: {}\n",
        msg.vacuum_pump_ps_voltage()
    ));
    output.push_str(&format!(
        "Focus Coil PS Voltage: {}\n",
        msg.focus_coil_ps_voltage()
    ));
    output.push_str(&format!(
        "Filament PS: {} ({})\n",
        msg.filament_ps(),
        match msg.filament_ps() {
            0 => "On",
            1 => "Off",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Klystron Warmup: {} ({})\n",
        msg.klystron_warmup(),
        match msg.klystron_warmup() {
            0 => "Normal",
            1 => "Preheat",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Transmitter Available: {} ({})\n",
        msg.transmitter_available(),
        match msg.transmitter_available() {
            0 => "Yes",
            1 => "No",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "WG Switch Position: {} ({})\n",
        msg.wg_switch_position(),
        match msg.wg_switch_position() {
            0 => "Antenna",
            1 => "Dummy Load",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "WG/PFN Transfer Interlock: {} ({})\n",
        msg.wg_pfn_transfer_interlock(),
        match msg.wg_pfn_transfer_interlock() {
            0 => "OK",
            1 => "Open",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Maintenance Mode: {} ({})\n",
        msg.maintenance_mode(),
        match msg.maintenance_mode() {
            0 => "No",
            1 => "Yes",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Maintenance Required: {} ({})\n",
        msg.maintenance_required(),
        match msg.maintenance_required() {
            0 => "No",
            1 => "Required",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "PFN Switch: {} ({})\n",
        msg.pfn_switch_position(),
        match msg.pfn_switch_position() {
            0 => "Short Pulse",
            1 => "Long Pulse",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Modulator Overload: {}\n",
        msg.modulator_overload()
    ));
    output.push_str(&format!(
        "Modulator Inv Current: {}\n",
        msg.modulator_inv_current()
    ));
    output.push_str(&format!(
        "Modulator Switch Fail: {}\n",
        msg.modulator_switch_fail()
    ));
    output.push_str(&format!(
        "Main Power Voltage: {}\n",
        msg.main_power_voltage()
    ));
    output.push_str(&format!(
        "Charging System Fail: {}\n",
        msg.charging_system_fail()
    ));
    output.push_str(&format!(
        "Inverse Diode Current: {}\n",
        msg.inverse_diode_current()
    ));
    output.push_str(&format!("Trigger Amplifier: {}\n", msg.trigger_amplifier()));
    output.push_str(&format!(
        "Circulator Temperature: {}\n",
        msg.circulator_temperature()
    ));
    output.push_str(&format!(
        "Spectrum Filter Pressure: {}\n",
        msg.spectrum_filter_pressure()
    ));
    output.push_str(&format!("WG Arc/VSWR: {}\n", msg.wg_arc_vswr()));
    output.push_str(&format!("Cabinet Interlock: {}\n", msg.cabinet_interlock()));
    output.push_str(&format!(
        "Cabinet Air Temp: {}\n",
        msg.cabinet_air_temperature()
    ));
    output.push_str(&format!("Cabinet Airflow: {}\n", msg.cabinet_airflow()));
    output.push_str(&format!("Klystron Current: {}\n", msg.klystron_current()));
    output.push_str(&format!(
        "Klystron Filament Current: {}\n",
        msg.klystron_filament_current()
    ));
    output.push_str(&format!(
        "Klystron Vacion Current: {}\n",
        msg.klystron_vacion_current()
    ));
    output.push_str(&format!(
        "Klystron Air Temp: {}\n",
        msg.klystron_air_temperature()
    ));
    output.push_str(&format!("Klystron Airflow: {}\n", msg.klystron_airflow()));
    output.push_str(&format!(
        "Modulator Switch Maintenance: {}\n",
        msg.modulator_switch_maintenance()
    ));
    output.push_str(&format!(
        "Post Charge Regulator Maintenance: {}\n",
        msg.post_charge_regulator_maintenance()
    ));
    output.push_str(&format!(
        "WG Pressure/Humidity: {}\n",
        msg.wg_pressure_humidity()
    ));
    output.push_str(&format!(
        "Transmitter Overvoltage: {}\n",
        msg.transmitter_overvoltage()
    ));
    output.push_str(&format!(
        "Transmitter Overcurrent: {}\n",
        msg.transmitter_overcurrent()
    ));
    output.push_str(&format!(
        "Focus Coil Current: {}\n",
        msg.focus_coil_current()
    ));
    output.push_str(&format!(
        "Focus Coil Airflow: {}\n",
        msg.focus_coil_airflow()
    ));
    output.push_str(&format!("Oil Temperature: {}\n", msg.oil_temperature()));
    output.push_str(&format!("PRF Limit: {}\n", msg.prf_limit()));
    output.push_str(&format!(
        "Transmitter Oil Level: {}\n",
        msg.transmitter_oil_level()
    ));
    output.push_str(&format!(
        "Transmitter Battery Charging: {} ({})\n",
        msg.transmitter_battery_charging(),
        match msg.transmitter_battery_charging() {
            0 => "Yes",
            1 => "No",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "High Voltage Status: {} ({})\n",
        msg.high_voltage_status(),
        match msg.high_voltage_status() {
            0 => "On",
            1 => "Off",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Transmitter Recycling Summary: {} ({})\n",
        msg.transmitter_recycling_summary(),
        match msg.transmitter_recycling_summary() {
            0 => "Normal",
            1 => "Recycling",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Transmitter Inoperable: {}\n",
        msg.transmitter_inoperable()
    ));
    output.push_str(&format!(
        "Transmitter Air Filter: {} ({})\n",
        msg.transmitter_air_filter(),
        match msg.transmitter_air_filter() {
            0 => "Dirty",
            1 => "OK",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "XMTR SPIP Interface: {}\n",
        msg.xmtr_spip_interface()
    ));
    output.push_str(&format!(
        "Transmitter Summary Status: {} ({})\n",
        msg.transmitter_summary_status(),
        match msg.transmitter_summary_status() {
            0 => "Ready",
            1 => "Alarm",
            2 => "Maintenance",
            3 => "Recycle",
            4 => "Preheat",
            _ => "Unknown",
        }
    ));

    output.push_str("\n  Transmitter Power:\n");
    output.push_str(&format!(
        "    RF Power Sensor: {:.2} mW\n",
        msg.transmitter_rf_power_sensor()
    ));
    output.push_str(&format!(
        "    H Peak Power: {:.2} kW\n",
        msg.horizontal_xmtr_peak_power()
    ));
    output.push_str(&format!(
        "    Peak Power: {:.2} kW\n",
        msg.xmtr_peak_power()
    ));
    output.push_str(&format!(
        "    V Peak Power: {:.2} kW\n",
        msg.vertical_xmtr_peak_power()
    ));
    output.push_str(&format!(
        "    RF Avg Power: {:.2} W\n",
        msg.xmtr_rf_avg_power()
    ));
    output.push_str(&format!(
        "    Recycle Count: {}\n",
        msg.xmtr_recycle_count()
    ));
    output.push_str(&format!(
        "    Receiver Bias: {:.2} dB\n",
        msg.receiver_bias_measurement()
    ));
    output.push_str(&format!(
        "    Transmit Imbalance: {:.2} dB\n",
        msg.transmit_imbalance()
    ));
    output.push_str(&format!(
        "    Power Meter Zero: {:.3} V\n",
        msg.xmtr_power_meter_zero()
    ));

    let ztb = msg.zero_test_bits();
    let otb = msg.one_test_bits();
    output.push_str(&format!("\n  Zero Test Bits: {:?}\n", ztb));
    output.push_str(&format!("  One Test Bits: {:?}\n", otb));

    // Tower/Utilities
    output.push_str("\n--- Tower/Utilities ---\n");
    output.push_str(&format!(
        "AC Unit 1 Compressor Shut Off: {}\n",
        msg.ac_unit_1_compressor_shut_off()
    ));
    output.push_str(&format!(
        "AC Unit 2 Compressor Shut Off: {}\n",
        msg.ac_unit_2_compressor_shut_off()
    ));
    output.push_str(&format!(
        "Generator Maint Required: {}\n",
        msg.generator_maintenance_required()
    ));
    output.push_str(&format!(
        "Generator Battery Voltage: {}\n",
        msg.generator_battery_voltage()
    ));
    output.push_str(&format!("Generator Engine: {}\n", msg.generator_engine()));
    output.push_str(&format!(
        "Generator Volt/Frequency: {}\n",
        msg.generator_volt_frequency()
    ));
    output.push_str(&format!(
        "Power Source: {} ({})\n",
        msg.power_source(),
        match msg.power_source() {
            0 => "Utility",
            1 => "Generator",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Transitional Power Source: {} ({})\n",
        msg.transitional_power_source(),
        match msg.transitional_power_source() {
            0 => "OK",
            1 => "Off",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Generator Auto Run Off Switch: {} ({})\n",
        msg.generator_auto_run_off_switch(),
        match msg.generator_auto_run_off_switch() {
            0 => "Manual",
            1 => "Auto",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Aircraft Hazard Lighting: {}\n",
        msg.aircraft_hazard_lighting()
    ));
    output.push_str(&format!(
        "Equipment Shelter Fire Detection: {}\n",
        msg.equipment_shelter_fire_detection_system()
    ));
    output.push_str(&format!(
        "Equipment Shelter Fire/Smoke: {}\n",
        msg.equipment_shelter_fire_smoke()
    ));
    output.push_str(&format!(
        "Generator Shelter Fire/Smoke: {}\n",
        msg.generator_shelter_fire_smoke()
    ));
    output.push_str(&format!(
        "Utility Voltage/Frequency: {}\n",
        msg.utility_voltage_frequency()
    ));
    output.push_str(&format!(
        "Site Security Alarm: {}\n",
        msg.site_security_alarm()
    ));
    output.push_str(&format!(
        "Security Equipment: {}\n",
        msg.security_equipment()
    ));
    output.push_str(&format!("Security System: {}\n", msg.security_system()));
    output.push_str(&format!(
        "Receiver Connected to Antenna: {} ({})\n",
        msg.receiver_connected_to_antenna(),
        match msg.receiver_connected_to_antenna() {
            0 => "Connected",
            1 => "Not Connected",
            2 => "N/A",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Radome Hatch: {} ({})\n",
        msg.radome_hatch(),
        match msg.radome_hatch() {
            0 => "Open",
            1 => "Closed",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "AC Unit 1 Filter Dirty: {}\n",
        msg.ac_unit_1_filter_dirty()
    ));
    output.push_str(&format!(
        "AC Unit 2 Filter Dirty: {}\n",
        msg.ac_unit_2_filter_dirty()
    ));

    output.push_str("\n  Temperatures:\n");
    output.push_str(&format!(
        "    Equipment Shelter: {:.1}\u{00b0}C\n",
        msg.equipment_shelter_temperature()
    ));
    output.push_str(&format!(
        "    Outside Ambient: {:.1}\u{00b0}C\n",
        msg.outside_ambient_temperature()
    ));
    output.push_str(&format!(
        "    Transmitter Leaving Air: {:.1}\u{00b0}C\n",
        msg.transmitter_leaving_air_temp()
    ));
    output.push_str(&format!(
        "    AC Unit 1 Discharge: {:.1}\u{00b0}C\n",
        msg.ac_unit_1_discharge_air_temp()
    ));
    output.push_str(&format!(
        "    Generator Shelter: {:.1}\u{00b0}C\n",
        msg.generator_shelter_temperature()
    ));
    output.push_str(&format!(
        "    Radome Air: {:.1}\u{00b0}C\n",
        msg.radome_air_temperature()
    ));
    output.push_str(&format!(
        "    AC Unit 2 Discharge: {:.1}\u{00b0}C\n",
        msg.ac_unit_2_discharge_air_temp()
    ));

    output.push_str(&format!("SPIP +15V PS: {:.3} V\n", msg.spip_15v_ps()));
    output.push_str(&format!("SPIP -15V PS: {:.3} V\n", msg.spip_neg_15v_ps()));
    output.push_str(&format!(
        "SPIP 28V PS Status: {} ({})\n",
        msg.spip_28v_ps_status(),
        match msg.spip_28v_ps_status() {
            0 => "Fail",
            1 => "OK",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!("SPIP 5V PS: {:.3} V\n", msg.spip_5v_ps()));
    output.push_str(&format!(
        "Converted Generator Fuel Level: {}%\n",
        msg.converted_generator_fuel_level()
    ));

    // Antenna/Pedestal
    output.push_str("\n--- Antenna/Pedestal ---\n");
    output.push_str(&format!(
        "Elevation +Dead Limit: {}\n",
        msg.elevation_plus_dead_limit()
    ));
    output.push_str(&format!(
        "+150V Overvoltage: {}\n",
        msg.plus_150v_overvoltage()
    ));
    output.push_str(&format!(
        "+150V Undervoltage: {}\n",
        msg.plus_150v_undervoltage()
    ));
    output.push_str(&format!(
        "Elevation Servo Amp Inhibit: {}\n",
        msg.elevation_servo_amp_inhibit()
    ));
    output.push_str(&format!(
        "Elevation Servo Amp Short Circuit: {}\n",
        msg.elevation_servo_amp_short_circuit()
    ));
    output.push_str(&format!(
        "Elevation Servo Amp Overtemp: {}\n",
        msg.elevation_servo_amp_overtemp()
    ));
    output.push_str(&format!(
        "Elevation Motor Overtemp: {}\n",
        msg.elevation_motor_overtemp()
    ));
    output.push_str(&format!(
        "Elevation Stow Pin: {}\n",
        msg.elevation_stow_pin()
    ));
    output.push_str(&format!(
        "Elevation Housing 5V PS: {}\n",
        msg.elevation_housing_5v_ps()
    ));
    output.push_str(&format!(
        "Elevation -Dead Limit: {}\n",
        msg.elevation_minus_dead_limit()
    ));
    output.push_str(&format!(
        "Elevation +Normal Limit: {}\n",
        msg.elevation_plus_normal_limit()
    ));
    output.push_str(&format!(
        "Elevation -Normal Limit: {}\n",
        msg.elevation_minus_normal_limit()
    ));
    output.push_str(&format!(
        "Elevation Encoder Light: {}\n",
        msg.elevation_encoder_light()
    ));
    output.push_str(&format!(
        "Elevation Gearbox Oil: {}\n",
        msg.elevation_gearbox_oil()
    ));
    output.push_str(&format!(
        "Elevation Handwheel: {}\n",
        msg.elevation_handwheel()
    ));
    output.push_str(&format!("Elevation Amp PS: {}\n", msg.elevation_amp_ps()));
    output.push_str(&format!(
        "Azimuth Servo Amp Inhibit: {}\n",
        msg.azimuth_servo_amp_inhibit()
    ));
    output.push_str(&format!(
        "Azimuth Servo Amp Short Circuit: {}\n",
        msg.azimuth_servo_amp_short_circuit()
    ));
    output.push_str(&format!(
        "Azimuth Servo Amp Overtemp: {}\n",
        msg.azimuth_servo_amp_overtemp()
    ));
    output.push_str(&format!(
        "Azimuth Motor Overtemp: {}\n",
        msg.azimuth_motor_overtemp()
    ));
    output.push_str(&format!("Azimuth Stow Pin: {}\n", msg.azimuth_stow_pin()));
    output.push_str(&format!(
        "Azimuth Housing 5V PS: {}\n",
        msg.azimuth_housing_5v_ps()
    ));
    output.push_str(&format!(
        "Azimuth Encoder Light: {}\n",
        msg.azimuth_encoder_light()
    ));
    output.push_str(&format!(
        "Azimuth Gearbox Oil: {}\n",
        msg.azimuth_gearbox_oil()
    ));
    output.push_str(&format!(
        "Azimuth Bull Gear Oil: {}\n",
        msg.azimuth_bull_gear_oil()
    ));
    output.push_str(&format!("Azimuth Handwheel: {}\n", msg.azimuth_handwheel()));
    output.push_str(&format!(
        "Azimuth Servo Amp PS: {}\n",
        msg.azimuth_servo_amp_ps()
    ));
    output.push_str(&format!(
        "Servo: {} ({})\n",
        msg.servo(),
        match msg.servo() {
            0 => "On",
            1 => "Off",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!(
        "Pedestal Interlock Switch: {} ({})\n",
        msg.pedestal_interlock_switch(),
        match msg.pedestal_interlock_switch() {
            0 => "Operational",
            1 => "Safe",
            _ => "Unknown",
        }
    ));

    // RF Generator/Receiver
    output.push_str("\n--- RF Generator/Receiver ---\n");
    output.push_str(&format!("COHO/Clock: {}\n", msg.coho_clock()));
    output.push_str(&format!(
        "RF Gen Freq Select Oscillator: {}\n",
        msg.rf_generator_frequency_select_oscillator()
    ));
    output.push_str(&format!(
        "RF Gen RF STALO: {}\n",
        msg.rf_generator_rf_stalo()
    ));
    output.push_str(&format!(
        "RF Gen Phase-Shifted COHO: {}\n",
        msg.rf_generator_phase_shifted_coho()
    ));
    output.push_str(&format!(
        "Receiver PS: +9V={} +5V={} +/-18V={} -9V={}\n",
        msg.plus_9v_receiver_ps(),
        msg.plus_5v_receiver_ps(),
        msg.plus_or_minus_18v_receiver_ps(),
        msg.minus_9v_receiver_ps()
    ));
    output.push_str(&format!(
        "+5V Single Channel RDAIU PS: {}\n",
        msg.plus_5v_single_channel_rdaiu_ps()
    ));

    output.push_str("\n  Noise Levels:\n");
    output.push_str(&format!(
        "    H Short Pulse: {:.2} dBm\n",
        msg.horizontal_short_pulse_noise()
    ));
    output.push_str(&format!(
        "    H Long Pulse: {:.2} dBm\n",
        msg.horizontal_long_pulse_noise()
    ));
    output.push_str(&format!(
        "    H Noise Temp: {:.2} K\n",
        msg.horizontal_noise_temperature()
    ));
    output.push_str(&format!(
        "    V Short Pulse: {:.2} dBm\n",
        msg.vertical_short_pulse_noise()
    ));
    output.push_str(&format!(
        "    V Long Pulse: {:.2} dBm\n",
        msg.vertical_long_pulse_noise()
    ));
    output.push_str(&format!(
        "    V Noise Temp: {:.2} K\n",
        msg.vertical_noise_temperature()
    ));

    // Calibration
    output.push_str("\n--- Calibration ---\n");
    output.push_str(&format!("H Linearity: {:.4}\n", msg.horizontal_linearity()));
    output.push_str(&format!(
        "H Dynamic Range: {:.2} dB\n",
        msg.horizontal_dynamic_range()
    ));
    output.push_str(&format!(
        "H Delta dBZ0: {:.2} dB\n",
        msg.horizontal_delta_dbz0()
    ));
    output.push_str(&format!(
        "V Delta dBZ0: {:.2} dB\n",
        msg.vertical_delta_dbz0()
    ));
    output.push_str(&format!(
        "KD Peak Measured: {:.2} dBm\n",
        msg.kd_peak_measured()
    ));
    output.push_str(&format!(
        "Short Pulse H dBZ0: {:.2} dBZ\n",
        msg.short_pulse_horizontal_dbz0()
    ));
    output.push_str(&format!(
        "Long Pulse H dBZ0: {:.2} dBZ\n",
        msg.long_pulse_horizontal_dbz0()
    ));
    output.push_str(&format!(
        "Velocity Processed: {} ({})\n",
        msg.velocity_processed(),
        match msg.velocity_processed() {
            0 => "Good",
            1 => "Fail",
            _ => "Unknown",
        }
    ));
    output.push_str(&format!("Width Processed: {}\n", msg.width_processed()));
    output.push_str(&format!("Velocity RF Gen: {}\n", msg.velocity_rf_gen()));
    output.push_str(&format!("Width RF Gen: {}\n", msg.width_rf_gen()));
    output.push_str(&format!("H I0: {:.2} dBm\n", msg.horizontal_i0()));
    output.push_str(&format!("V I0: {:.2} dBm\n", msg.vertical_i0()));
    output.push_str(&format!(
        "V Dynamic Range: {:.2} dB\n",
        msg.vertical_dynamic_range()
    ));
    output.push_str(&format!(
        "Short Pulse V dBZ0: {:.2} dBZ\n",
        msg.short_pulse_vertical_dbz0()
    ));
    output.push_str(&format!(
        "Long Pulse V dBZ0: {:.2} dBZ\n",
        msg.long_pulse_vertical_dbz0()
    ));
    output.push_str(&format!(
        "H Power Sense: {:.2} dBm\n",
        msg.horizontal_power_sense()
    ));
    output.push_str(&format!(
        "V Power Sense: {:.2} dBm\n",
        msg.vertical_power_sense()
    ));
    output.push_str(&format!("ZDR Offset: {:.2} dB\n", msg.zdr_offset()));
    output.push_str(&format!(
        "Clutter Suppression Delta: {:.2} dB\n",
        msg.clutter_suppression_delta()
    ));
    output.push_str(&format!(
        "Clutter Suppression Unfiltered Power: {:.2} dBZ\n",
        msg.clutter_suppression_unfiltered_power()
    ));
    output.push_str(&format!(
        "Clutter Suppression Filtered Power: {:.2} dBZ\n",
        msg.clutter_suppression_filtered_power()
    ));
    output.push_str(&format!("V Linearity: {:.4}\n", msg.vertical_linearity()));

    // File Status
    output.push_str("\n--- File Status ---\n");
    output.push_str(&format!(
        "State File: R={} W={}\n",
        msg.state_file_read_status(),
        msg.state_file_write_status()
    ));
    output.push_str(&format!(
        "Bypass Map File: R={} W={}\n",
        msg.bypass_map_file_read_status(),
        msg.bypass_map_file_write_status()
    ));
    output.push_str(&format!(
        "Current Adaptation File: R={} W={}\n",
        msg.current_adaptation_file_read_status(),
        msg.current_adaptation_file_write_status()
    ));
    output.push_str(&format!(
        "Censor Zone File: R={} W={}\n",
        msg.censor_zone_file_read_status(),
        msg.censor_zone_file_write_status()
    ));
    output.push_str(&format!(
        "Remote VCP File: R={} W={}\n",
        msg.remote_vcp_file_read_status(),
        msg.remote_vcp_file_write_status()
    ));
    output.push_str(&format!(
        "Baseline Adaptation File: R={}\n",
        msg.baseline_adaptation_file_read_status()
    ));
    output.push_str(&format!(
        "PRF Sets: R=0x{:04X}\n",
        msg.read_status_of_prf_sets()
    ));
    output.push_str(&format!(
        "Clutter Filter Map File: R={} W={}\n",
        msg.clutter_filter_map_file_read_status(),
        msg.clutter_filter_map_file_write_status()
    ));
    output.push_str(&format!(
        "General Disk I/O Error: {}\n",
        msg.general_disk_io_error()
    ));

    // RSP/CPU Status
    output.push_str("\n--- RSP/CPU Status ---\n");
    output.push_str(&format!("RSP Status: 0x{:04X}\n", msg.rsp_status()));
    let cpu_temps = msg.cpu_temperatures();
    output.push_str(&format!(
        "CPU Temperatures: CPU1={}\u{00b0}C CPU2={}\u{00b0}C\n",
        cpu_temps & 0xFF,
        (cpu_temps >> 8) & 0xFF
    ));
    output.push_str(&format!(
        "RSP Motherboard Power: {} W\n",
        msg.rsp_motherboard_power()
    ));

    // Device Status
    output.push_str("\n--- Device Status ---\n");
    output.push_str(&format!("SPIP Comm Status: {}\n", msg.spip_comm_status()));
    output.push_str(&format!("HCI Comm Status: {}\n", msg.hci_comm_status()));
    output.push_str(&format!(
        "Signal Processor Command Status: {}\n",
        msg.signal_processor_command_status()
    ));
    output.push_str(&format!(
        "AME Comm Status: {}\n",
        msg.ame_communication_status()
    ));
    output.push_str(&format!("RMS Link Status: {}\n", msg.rms_link_status()));
    output.push_str(&format!("RPG Link Status: {}\n", msg.rpg_link_status()));
    output.push_str(&format!(
        "Interpanel Link Status: {}\n",
        msg.interpanel_link_status()
    ));
    output.push_str(&format!(
        "Performance Check Time: {}\n",
        msg.performance_check_time()
    ));

    output
}
