use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Split into header info and record list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(1)])
        .split(area);

    render_header_info(frame, app, chunks[0]);
    render_record_list(frame, app, chunks[1]);
}

fn render_header_info(frame: &mut Frame, app: &App, area: Rect) {
    let header = &app.header;

    let tape_filename = header
        .tape_filename()
        .unwrap_or_else(|| "Unknown".to_string());
    let extension = header
        .extension_number()
        .unwrap_or_else(|| "???".to_string());
    let datetime = header
        .date_time()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let icao = header.icao_of_radar().unwrap_or_else(|| "????".to_string());

    let info_text = format!(
        "File: {}\n\
         Tape Filename: {}\n\
         Extension: {}\n\
         Date/Time: {}\n\
         Radar ICAO: {}",
        app.file_path.display(),
        tape_filename,
        extension,
        datetime,
        icao
    );

    let info = Paragraph::new(info_text)
        .block(
            Block::default()
                .title(" Volume Header ")
                .borders(Borders::ALL),
        )
        .style(Style::default());

    frame.render_widget(info, area);
}

fn render_record_list(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["#", "Status", "Compressed", "Decompressed"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .records
        .iter()
        .map(|record| {
            let is_cached = app.is_record_decompressed(record.index);
            let decompressed_size = app.get_decompressed_size(record.index);

            let (status, status_style) = if !record.compressed {
                ("Raw", Style::default().fg(Color::Blue))
            } else if is_cached {
                ("Cached", Style::default().fg(Color::Green))
            } else {
                ("Compressed", Style::default().fg(Color::Yellow))
            };

            let decompressed_str = decompressed_size
                .map(|s| format!("{}", s))
                .unwrap_or_else(|| "-".to_string());

            let cells = vec![
                Cell::from(format!("{}", record.index)),
                Cell::from(status).style(status_style),
                Cell::from(format!("{}", record.size)),
                Cell::from(decompressed_str),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(" Records ({}) - d:decompress ", app.records.len()))
                .borders(Borders::ALL),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = TableState::default();
    state.select(Some(app.selected_record));

    frame.render_stateful_widget(table, area, &mut state);
}
