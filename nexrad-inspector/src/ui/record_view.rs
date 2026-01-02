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
    let messages = match app.get_messages(app.selected_record) {
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
                let type_str = format!("{} ({})", hdr.message_type, format_message_type(&msg_type));

                let datetime_str = hdr
                    .date_time()
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "-".to_string());

                let segment_str = if hdr.segmented() {
                    format!(
                        "{}/{}",
                        hdr.segment_number().unwrap_or(0),
                        hdr.segment_count().unwrap_or(0)
                    )
                } else {
                    format!("var ({}B)", hdr.message_size_bytes())
                };

                (type_str, datetime_str, segment_str)
            } else {
                ("?".to_string(), "-".to_string(), "-".to_string())
            };

            let cells = vec![
                Cell::from(format!("{}", msg.index)),
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
        Constraint::Length(5),
        Constraint::Length(28),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(14),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" Messages ({}) ", messages.len()))
                .borders(Borders::ALL),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = TableState::default();
    state.select(Some(app.selected_message));

    frame.render_stateful_widget(table, area, &mut state);
}

fn format_message_type(msg_type: &MessageType) -> &'static str {
    match msg_type {
        MessageType::RDADigitalRadarData => "RDA Digital Radar",
        MessageType::RDAStatusData => "RDA Status",
        MessageType::RDAPerformanceMaintenanceData => "RDA Perf/Maint",
        MessageType::RDAConsoleMessage => "RDA Console",
        MessageType::RDAVolumeCoveragePattern => "RDA VCP",
        MessageType::RDAControlCommands => "RDA Control",
        MessageType::RPGVolumeCoveragePattern => "RPG VCP",
        MessageType::RPGClutterCensorZones => "RPG Clutter",
        MessageType::RPGRequestForData => "RPG Request",
        MessageType::RPGConsoleMessage => "RPG Console",
        MessageType::RDALoopBackTest => "RDA Loopback",
        MessageType::RPGLoopBackTest => "RPG Loopback",
        MessageType::RDAClutterFilterBypassMap => "RDA Clutter Bypass",
        MessageType::Spare1 => "Spare",
        MessageType::RDAClutterFilterMap => "RDA Clutter Map",
        MessageType::ReservedFAARMSOnly1 => "Reserved FAARMS",
        MessageType::ReservedFAARMSOnly2 => "Reserved FAARMS",
        MessageType::RDAAdaptationData => "RDA Adaptation",
        MessageType::Reserved1 => "Reserved",
        MessageType::Reserved2 => "Reserved",
        MessageType::Reserved3 => "Reserved",
        MessageType::Reserved4 => "Reserved",
        MessageType::ReservedFAARMSOnly3 => "Reserved FAARMS",
        MessageType::ReservedFAARMSOnly4 => "Reserved FAARMS",
        MessageType::ReservedFAARMSOnly5 => "Reserved FAARMS",
        MessageType::Reserved5 => "Reserved",
        MessageType::RDADigitalRadarDataGenericFormat => "Digital Radar Data",
        MessageType::RDAPRFData => "RDA PRF",
        MessageType::RDALogData => "RDA Log",
        MessageType::Unknown(_) => "Unknown",
    }
}
