mod file_view;
mod hex_view;
mod message_view;
mod record_view;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::{App, View};

/// Render the UI based on current app state
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Main layout: content area + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let content_area = chunks[0];
    let status_area = chunks[1];

    // Render the appropriate view
    match app.view {
        View::File => file_view::render(frame, app, content_area),
        View::Record => record_view::render(frame, app, content_area),
        View::Message => message_view::render(frame, app, content_area),
    }

    // Render status bar
    render_status_bar(frame, app, status_area);

    // Render help overlay if shown
    if app.show_help {
        render_help_overlay(frame, area);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        let nav_hint = match app.view {
            View::File => "q:quit  Enter:open record  d:decompress  ?:help",
            View::Record => "q:quit  Enter:open message  Esc:back  ?:help",
            View::Message => "q:quit  Tab:hex/parsed  s:save  Esc:back  ?:help",
        };
        nav_hint.to_string()
    };

    let status = Paragraph::new(status_text).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Center the help popup
    let popup_width = 50;
    let popup_height = 17;
    let popup_area = centered_rect(popup_width, popup_height, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let help_text = r#"NEXRAD Inspector - Help

Navigation:
  Up/Down, j/k    Navigate list items
  Enter           Drill into selected item
  Esc/Backspace   Go back to previous view
  PageUp/PageDown Scroll by 10 items

File View:
  d               Decompress selected record

Message View:
  Tab             Toggle hex/parsed view
  s               Save message to file

General:
  ?               Toggle this help
  q               Quit"#;

    let help = Paragraph::new(help_text)
        .block(Block::default().title(" Help ").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(Color::Black));

    frame.render_widget(help, popup_area);
}

/// Helper to create a centered rectangle
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
