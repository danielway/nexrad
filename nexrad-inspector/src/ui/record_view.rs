use nexrad_decode::messages::MessageType;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split into record info and message list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1)])
        .split(area);

    render_record_info(frame, app, chunks[0]);
    render_message_list(frame, app, chunks[1]);
}

fn render_record_info(frame: &mut Frame, app: &App, area: Rect) {
    let record_info = &app.records[app.selected_record];
    let decompressed_size = app.get_decompressed_size(app.selected_record).unwrap_or(0);

    let info_text = format!(
        "Record #{}\n\
         Original Size: {} bytes\n\
         Decompressed Size: {} bytes",
        record_info.index, record_info.size, decompressed_size
    );

    let info = Paragraph::new(info_text)
        .block(
            Block::default()
                .title(" Record Info ")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    frame.render_widget(info, area);
}

fn render_message_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let messages = match app.get_displayed_messages(app.selected_record) {
        Ok(msgs) => msgs,
        Err(e) => {
            let error = Paragraph::new(format!("Error loading messages: {}", e))
                .block(Block::default().title(" Messages ").borders(Borders::ALL))
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error, area);
            return;
        }
    };

    let header_cells = ["#", "Type", "Offset", "Size", "DateTime", "Segments"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = messages
        .iter()
        .map(|msg| {
            let header_info = App::get_message_header(&msg.data);

            let (type_str, datetime_str, segment_str) = if let Some(hdr) = header_info {
                let msg_type = hdr.message_type();
                let type_str = format!(
                    "{} ({})",
                    hdr.message_type_raw(),
                    format_message_type(&msg_type)
                );

                let datetime_str = hdr
                    .date_time()
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "-".to_string());

                // For combined segmented messages, show segment count
                // For single messages, show "var" for variable-length or "1" for fixed
                let segment_str = if msg.segment_count > 1 {
                    format!("{} segments", msg.segment_count)
                } else if hdr.is_fixed_frame() {
                    "1".to_string()
                } else {
                    format!("var ({}B)", hdr.message_size_bytes())
                };

                (type_str, datetime_str, segment_str)
            } else {
                ("?".to_string(), "-".to_string(), "-".to_string())
            };

            // Show index range for segmented messages (e.g., "253-257")
            let index_str = if msg.raw_indices.len() > 1 {
                format!(
                    "{}-{}",
                    msg.raw_indices.first().unwrap_or(&0),
                    msg.raw_indices.last().unwrap_or(&0)
                )
            } else {
                format!("{}", msg.raw_indices.first().unwrap_or(&msg.index))
            };

            let cells = vec![
                Cell::from(index_str),
                Cell::from(type_str),
                Cell::from(format!("0x{:06X}", msg.offset)),
                Cell::from(format!("{}", msg.size)),
                Cell::from(datetime_str),
                Cell::from(segment_str),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(9), // Index (can be "253-257" for segmented)
        Constraint::Length(28),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(14),
    ];

    let title = if app.filter_empty_unknown {
        format!(" Messages ({}) [filtered] ", messages.len())
    } else {
        format!(" Messages ({}) ", messages.len())
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title(title).borders(Borders::ALL))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = TableState::default();
    state.select(Some(app.selected_message));

    frame.render_stateful_widget(table, area, &mut state);
}

fn format_message_type(msg_type: &MessageType) -> &'static str {
    match msg_type {
        MessageType::DigitalRadarDataLegacy => "RDA Digital Radar",
        MessageType::RdaStatusData => "RDA Status",
        MessageType::PerformanceMaintenanceData => "RDA Perf/Maint",
        MessageType::RdaConsoleMessage => "RDA Console",
        MessageType::RdaVolumeCoveragePattern => "RDA VCP",
        MessageType::RdaControlCommands => "RDA Control",
        MessageType::RpgVolumeCoveragePattern => "RPG VCP",
        MessageType::ClutterCensorZones => "RPG Clutter",
        MessageType::RequestForData => "RPG Request",
        MessageType::RpgConsoleMessage => "RPG Console",
        MessageType::RdaLoopbackTest => "RDA Loopback",
        MessageType::RpgLoopbackTest => "RPG Loopback",
        MessageType::ClutterFilterBypassMap => "RDA Clutter Bypass",
        MessageType::ClutterFilterMap => "RDA Clutter Map",
        MessageType::RdaAdaptationData => "RDA Adaptation",
        MessageType::ModelData => "Model Data",
        MessageType::Reserved(_) => "Reserved",
        MessageType::DigitalRadarData => "Digital Radar Data",
        MessageType::RdaPrfData => "RDA PRF",
        MessageType::RdaLogData => "RDA Log",
        MessageType::Unknown(_) => "Unknown",
        _ => "Unsupported",
    }
}
