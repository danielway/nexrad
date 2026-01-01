use nexrad_decode::messages::{
    digital_radar_data, rda_status_data, volume_coverage_pattern, MessageType,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};
use zerocopy::FromBytes;

use crate::app::{App, MessageTab};
use crate::ui::hex_view;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    // Get current message - extract indices first to avoid borrow issues
    let record_index = app.selected_record;
    let message_index = app.selected_message;

    let message_info = match app.get_messages(record_index) {
        Ok(msgs) => msgs.get(message_index).cloned(),
        Err(_) => None,
    };

    let Some(message_info) = message_info else {
        let error = Paragraph::new("No message selected")
            .block(Block::default().title(" Message ").borders(Borders::ALL));
        frame.render_widget(error, area);
        return;
    };

    // Layout: header info + tabs + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Header info
            Constraint::Length(3), // Tabs
            Constraint::Min(1),    // Content
        ])
        .split(area);

    render_header_info(frame, &message_info.data, chunks[0]);
    render_tabs(frame, app, chunks[1]);

    match app.message_tab {
        MessageTab::Hex => {
            hex_view::render(frame, &message_info.data, app.hex_scroll, chunks[2]);
        }
        MessageTab::Parsed => {
            render_parsed_view(frame, &message_info.data, app.parsed_scroll, chunks[2]);
        }
    }
}

fn render_header_info(frame: &mut Frame, data: &[u8], area: Rect) {
    let header_info = App::get_message_header(data);

    let info_text = if let Some(hdr) = header_info {
        let msg_type = hdr.message_type();
        let datetime = hdr
            .date_time()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let segment_info = if hdr.segmented() {
            format!(
                "Segment {}/{} ({} bytes each)",
                hdr.segment_number().unwrap_or(0),
                hdr.segment_count().unwrap_or(0),
                hdr.message_size_bytes()
            )
        } else {
            format!("Variable-length ({} bytes)", hdr.message_size_bytes())
        };

        format!(
            "Type: {} ({:?})\n\
             Sequence: {}\n\
             DateTime: {}\n\
             {}",
            hdr.message_type, msg_type, hdr.sequence_number, datetime, segment_info
        )
    } else {
        "Unable to parse message header".to_string()
    };

    let info = Paragraph::new(info_text).block(
        Block::default()
            .title(" Message Header ")
            .borders(Borders::ALL),
    );
    frame.render_widget(info, area);
}

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Hex", "Parsed"];
    let selected = match app.message_tab {
        MessageTab::Hex => 0,
        MessageTab::Parsed => 1,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .select(selected)
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    frame.render_widget(tabs, area);
}

fn render_parsed_view(frame: &mut Frame, data: &[u8], scroll: usize, area: Rect) {
    let header = App::get_message_header(data);
    let Some(header) = header else {
        let error = Paragraph::new("Unable to parse message header")
            .block(Block::default().title(" Parsed ").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error, area);
        return;
    };

    let msg_type = header.message_type();
    let content_offset = std::mem::size_of::<nexrad_decode::messages::MessageHeader>();

    // Parse based on message type
    let parsed_text = match msg_type {
        MessageType::RDADigitalRadarDataGenericFormat => {
            parse_digital_radar_data(&data[content_offset..])
        }
        MessageType::RDAStatusData => parse_rda_status_data(&data[content_offset..]),
        MessageType::RDAVolumeCoveragePattern => {
            parse_volume_coverage_pattern(&data[content_offset..])
        }
        _ => parse_common_header_only(header),
    };

    // Handle scrolling for parsed text
    let lines: Vec<&str> = parsed_text.lines().collect();
    let visible_height = (area.height as usize).saturating_sub(2);
    let max_scroll = lines.len().saturating_sub(visible_height);
    let effective_scroll = scroll.min(max_scroll);

    let visible_text: String = lines
        .iter()
        .skip(effective_scroll)
        .take(visible_height)
        .copied()
        .collect::<Vec<&str>>()
        .join("\n");

    let title = format!(
        " Parsed View (line {}/{}) ",
        effective_scroll + 1,
        lines.len().max(1)
    );

    let paragraph = Paragraph::new(visible_text)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn parse_common_header_only(header: &nexrad_decode::messages::MessageHeader) -> String {
    let msg_type = header.message_type();
    let datetime = header
        .date_time()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let channel = header.rda_redundant_channel();

    format!(
        "=== Common Message Header ===\n\n\
         Message Type: {} ({:?})\n\
         Sequence Number: {}\n\
         Date/Time: {}\n\
         Redundant Channel: {:?}\n\
         Segment Size: {} half-words\n\
         Segmented: {}\n\
         Message Size: {} bytes\n\n\
         (No additional parsing available for this message type)",
        header.message_type,
        msg_type,
        header.sequence_number,
        datetime,
        channel,
        header.segment_size,
        header.segmented(),
        header.message_size_bytes()
    )
}

fn parse_digital_radar_data(data: &[u8]) -> String {
    // Try to parse the header (Header is re-exported from raw module)
    let header = match digital_radar_data::Header::ref_from_prefix(data) {
        Ok((h, _)) => h,
        Err(_) => return "Failed to parse Digital Radar Data header".to_string(),
    };

    let datetime = header
        .date_time()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let radar_id = std::str::from_utf8(&header.radar_identifier)
        .unwrap_or("????")
        .trim_end_matches('\0');

    format!(
        "=== Digital Radar Data (Type 31) ===\n\n\
         Radar ID: {}\n\
         Date/Time: {}\n\
         Azimuth Number: {}\n\
         Azimuth Angle: {:.2} deg\n\
         Elevation Number: {}\n\
         Elevation Angle: {:.2} deg\n\
         Radial Status: {}\n\
         Radial Length: {} bytes\n\
         Compression: {}\n\
         Azimuth Resolution: {}\n\
         Cut Sector: {}\n\
         Spot Blanking: {}\n\
         Azimuth Indexing: {}\n\
         Data Block Count: {}",
        radar_id,
        datetime,
        header.azimuth_number,
        header.azimuth_angle.get(),
        header.elevation_number,
        header.elevation_angle.get(),
        header.radial_status,
        header.radial_length,
        header.compression_indicator,
        header.azimuth_resolution_spacing,
        header.cut_sector_number,
        header.radial_spot_blanking_status,
        header.azimuth_indexing_mode,
        header.data_block_count,
    )
}

fn parse_rda_status_data(data: &[u8]) -> String {
    // Message is directly in rda_status_data module (not raw)
    let message = match rda_status_data::Message::ref_from_prefix(data) {
        Ok((m, _)) => m,
        Err(_) => return "Failed to parse RDA Status Data".to_string(),
    };

    format!(
        "=== RDA Status Data (Type 2) ===\n\n\
         RDA Status: {}\n\
         Operability Status: {}\n\
         Control Status: {}\n\
         Aux Power Gen State: {}\n\
         Avg TX Power: {} watts\n\
         Horiz Reflectivity Cal: {}\n\
         Data TX Enabled: {}\n\
         Volume Coverage Pattern: {}\n\
         RDA Control Auth: {}\n\
         RDA Build Number: {}\n\
         Operational Mode: {}\n\
         Super Resolution: {}\n\
         Clutter Mitigation: {}\n\
         RDA Alarm Summary: {}\n\
         Command Ack: {}\n\
         Channel Control: {}\n\
         Spot Blanking: {}\n\
         Vert Reflectivity Cal: {}\n\
         TPS Status: {}\n\
         RMS Control: {}\n\
         Perf Check Status: {}\n\
         Status Version: {}",
        message.rda_status,
        message.operability_status,
        message.control_status,
        message.auxiliary_power_generator_state,
        message.average_transmitter_power,
        message.horizontal_reflectivity_calibration_correction,
        message.data_transmission_enabled,
        message.volume_coverage_pattern,
        message.rda_control_authorization,
        message.rda_build_number,
        message.operational_mode,
        message.super_resolution_status,
        message.clutter_mitigation_decision_status,
        message.rda_alarm_summary,
        message.command_acknowledgement,
        message.channel_control_status,
        message.spot_blanking_status,
        message.vertical_reflectivity_calibration_correction,
        message.transition_power_source_status,
        message.rms_control_status,
        message.performance_check_status,
        message.status_version,
    )
}

fn parse_volume_coverage_pattern(data: &[u8]) -> String {
    // Header is re-exported from raw module
    let header = match volume_coverage_pattern::Header::ref_from_prefix(data) {
        Ok((h, _)) => h,
        Err(_) => return "Failed to parse Volume Coverage Pattern header".to_string(),
    };

    format!(
        "=== Volume Coverage Pattern (Type 5) ===\n\n\
         Message Size: {} half-words\n\
         Pattern Type: {}\n\
         Pattern Number: {}\n\
         Number of Elevation Cuts: {}\n\
         Version: {}\n\
         Clutter Map Group: {}\n\
         Doppler Velocity Resolution: {}\n\
         Pulse Width: {}\n\
         VCP Sequencing: {}\n\
         VCP Supplemental Data: {}\n\n\
         (Elevation cut details not shown - {} cuts total)",
        header.message_size,
        header.pattern_type,
        header.pattern_number,
        header.number_of_elevation_cuts,
        header.version,
        header.clutter_map_group_number,
        header.doppler_velocity_resolution,
        header.pulse_width,
        header.vcp_sequencing,
        header.vcp_supplemental_data,
        header.number_of_elevation_cuts,
    )
}
