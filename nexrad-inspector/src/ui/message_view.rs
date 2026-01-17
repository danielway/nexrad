//! Message view rendering for the inspector.
//!
//! This module handles the main message view layout and delegates
//! message-type-specific parsing to sub-modules.

mod common;
mod digital_radar_data;
mod rda_status_data;
mod volume_coverage_pattern;

use nexrad_decode::messages::{MessageHeader, MessageType};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};

use crate::app::{App, MessageInfo, MessageTab};
use crate::ui::hex_view::{self, HexRegion};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    // Get current message - extract indices first to avoid borrow issues
    let record_index = app.selected_record;
    let message_index = app.selected_message;

    let message_info = match app.get_displayed_messages(record_index) {
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

    render_header_info(frame, &message_info, chunks[0]);
    render_tabs(frame, app, chunks[1]);

    match app.message_tab {
        MessageTab::Hex => {
            let regions = compute_hex_regions(&message_info);
            hex_view::render(
                frame,
                &message_info.data,
                app.hex_scroll,
                chunks[2],
                &regions,
            );
        }
        MessageTab::Parsed => {
            render_parsed_view(frame, &message_info.data, app.parsed_scroll, chunks[2]);
        }
    }
}

/// Compute hex regions for header/payload visualization
fn compute_hex_regions(msg_info: &MessageInfo) -> Vec<HexRegion> {
    let mut regions = Vec::new();
    let header_size = std::mem::size_of::<MessageHeader>();

    if msg_info.segment_count > 1 {
        // Multi-segment message: each segment is 2432 bytes (header + payload + padding)
        const SEGMENT_SIZE: usize = 2432;
        let payload_size = SEGMENT_SIZE - header_size;

        for seg_idx in 0..msg_info.segment_count {
            let seg_start = seg_idx * SEGMENT_SIZE;

            // Header region
            regions.push(HexRegion::header(seg_start, header_size));

            // Payload region
            regions.push(HexRegion::payload(seg_start + header_size, payload_size));
        }
    } else {
        // Single message (segmented with count=1, or variable-length)
        // Header region
        regions.push(HexRegion::header(0, header_size));

        // Payload region (rest of the data)
        if msg_info.size > header_size {
            regions.push(HexRegion::payload(header_size, msg_info.size - header_size));
        }
    }

    regions
}

fn render_header_info(frame: &mut Frame, msg_info: &MessageInfo, area: Rect) {
    let header_info = App::get_message_header(&msg_info.data);

    let info_text = if let Some(hdr) = header_info {
        let msg_type = hdr.message_type();
        let datetime = hdr
            .date_time()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Show segment info based on whether this is a combined segmented message
        let segment_info = if msg_info.segment_count > 1 {
            // Combined segmented message - show total segments and combined size
            let index_range = format!(
                "{}-{}",
                msg_info.raw_indices.first().unwrap_or(&0),
                msg_info.raw_indices.last().unwrap_or(&0)
            );
            format!(
                "{} segments (indices {}, {} bytes total)",
                msg_info.segment_count, index_range, msg_info.size
            )
        } else if hdr.segmented() {
            // Single fixed-length segment
            format!("Fixed-length ({} bytes)", hdr.message_size_bytes())
        } else {
            // Variable-length message
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

    // Parse based on message type - delegate to specialized parsers
    let parsed_text = match msg_type {
        MessageType::RDADigitalRadarDataGenericFormat => {
            // Pass full data so decode_messages can work
            digital_radar_data::parse_digital_radar_data(data)
        }
        MessageType::RDAStatusData => {
            rda_status_data::parse_rda_status_data(&data[content_offset..])
        }
        MessageType::RDAVolumeCoveragePattern => {
            // Pass full data for full decode with elevation cuts
            volume_coverage_pattern::parse_volume_coverage_pattern(data)
        }
        _ => common::parse_common_header_only(header),
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
