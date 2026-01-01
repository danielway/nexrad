use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table};

use crate::app::{App, AwsStep, FsEntry, LocalBrowserState};

pub fn render_local(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref state) = app.local_browser {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let path_display = state
            .current_dir
            .to_string_lossy()
            .to_string();
        let header = Paragraph::new(format!("Current Directory: {}", path_display))
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        render_file_list(frame, state, chunks[1]);

        let help = Paragraph::new("↑/↓: Navigate | Enter: Select/Open | Esc: Back to menu | q: Quit")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[2]);
    }
}

fn render_file_list(frame: &mut Frame, state: &LocalBrowserState, area: Rect) {
    let items: Vec<ListItem> = state
        .entries
        .iter()
        .map(|entry| {
            let (icon, name) = match entry {
                FsEntry::ParentDir => ("  ..", "..".to_string()),
                FsEntry::Directory(path) => (
                    "[DIR]",
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                ),
                FsEntry::File(path) => (
                    "     ",
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                ),
            };

            let style = match entry {
                FsEntry::Directory(_) => Style::default().fg(Color::Cyan),
                FsEntry::ParentDir => Style::default().fg(Color::Yellow),
                FsEntry::File(_) => Style::default(),
            };

            ListItem::new(format!("{} {}", icon, name)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Files and Directories")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(state.selected_index));

    frame.render_stateful_widget(list, area, &mut list_state);
}

pub fn render_aws(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref state) = app.aws_browser {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let title = Paragraph::new("AWS Archive Query")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        match state.step {
            AwsStep::EnterSite => {
                render_site_input(frame, state, chunks[1]);
                let help = Paragraph::new("Enter radar site code (e.g., KDMX) | Enter: Continue | Esc: Back")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(help, chunks[2]);
            }
            AwsStep::EnterDate => {
                render_date_input(frame, state, chunks[1]);
                let help = Paragraph::new("Enter date (YYYY-MM-DD) | Enter: Continue | Esc: Back")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(help, chunks[2]);
            }
            AwsStep::SelectFile => {
                render_file_selection(frame, state, chunks[1]);
                let help = Paragraph::new("↑/↓: Navigate | Enter: Download | Esc: Back")
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(help, chunks[2]);
            }
        }
    }
}

fn render_site_input(frame: &mut Frame, state: &crate::app::AwsBrowserState, area: Rect) {
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(40),
        ])
        .split(area);

    let horizontal_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(vertical_center[1]);

    state.site_input.render(frame, horizontal_center[1]);
}

fn render_date_input(frame: &mut Frame, state: &crate::app::AwsBrowserState, area: Rect) {
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(40),
        ])
        .split(area);

    let horizontal_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(vertical_center[1]);

    state.date_input.render(frame, horizontal_center[1]);
}

fn render_file_selection(frame: &mut Frame, state: &crate::app::AwsBrowserState, area: Rect) {
    use chrono::Timelike;

    if state.files.is_empty() {
        let msg = Paragraph::new("No files found for this site and date.")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Files"));
        frame.render_widget(msg, area);
        return;
    }

    let rows: Vec<Row> = state
        .files
        .iter()
        .map(|identifier| {
            let name = identifier.name();
            let time = identifier
                .date_time()
                .map(|dt| format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second()))
                .unwrap_or_else(|| "Unknown".to_string());

            Row::new(vec![time, name.to_string()])
        })
        .collect();

    let widths = [Constraint::Length(10), Constraint::Min(30)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(format!("{} files available", state.files.len()))
                .borders(Borders::ALL),
        )
        .header(
            Row::new(vec!["Time", "Filename"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(state.selected_index));

    frame.render_stateful_widget(table, area, &mut table_state);
}
