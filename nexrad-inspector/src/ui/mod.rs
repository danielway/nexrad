mod browser_view;
mod file_view;
mod hex_view;
mod loading_view;
mod menu_view;
mod message_view;
mod record_view;
pub mod text_input;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::{App, AppMode, View};

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

    // Render the appropriate view based on mode
    match app.mode {
        AppMode::Menu => {
            menu_view::render(frame, app, content_area);
        }
        AppMode::LocalBrowser => {
            browser_view::render_local(frame, app, content_area);
        }
        AppMode::AwsBrowser => {
            browser_view::render_aws(frame, app, content_area);
        }
        AppMode::Loading => {
            loading_view::render(frame, &app.loading_message, app.spinner_frame, content_area);
        }
        AppMode::Inspector => match app.view {
            View::File => file_view::render(frame, app, content_area),
            View::Record => record_view::render(frame, app, content_area),
            View::Message => message_view::render(frame, app, content_area),
        },
    }

    // Render status bar
    render_status_bar(frame, app, status_area);

    // Render error overlay if present
    if app.error.is_some() {
        render_error_overlay(frame, app, area);
    }

    // Render help overlay if shown
    if app.show_help {
        render_help_overlay(frame, area);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        let nav_hint = match app.mode {
            AppMode::Menu => "↑/↓:navigate  Enter:select  q:quit",
            AppMode::LocalBrowser => "↑/↓:navigate  Enter:select  Esc:back  q:quit",
            AppMode::AwsBrowser => "Type to enter text  Enter:continue  Esc:back  q:quit",
            AppMode::Loading => "Loading...",
            AppMode::Inspector => match app.view {
                View::File => "q:quit  Enter:open  d:decompress  D:decompress all  s:save  ?:help",
                View::Record => "q:quit  Enter:open message  s:save record  Esc:back  ?:help",
                View::Message => "q:quit  Tab:hex/parsed  s:save message  Esc:back  ?:help",
            },
        };
        nav_hint.to_string()
    };

    let status = Paragraph::new(status_text).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(status, area);
}

fn render_error_overlay(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref error) = app.error {
        let popup_width = 60;
        let popup_height = 7;
        let popup_area = centered_rect(popup_width, popup_height, area);

        frame.render_widget(Clear, popup_area);

        let error_text = format!("Error:\n\n{}\n\nPress Enter or Esc to dismiss", error);

        let error_widget = Paragraph::new(error_text)
            .block(
                Block::default()
                    .title(" Error ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(Color::Black).fg(Color::Red));

        frame.render_widget(error_widget, popup_area);
    }
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Center the help popup
    let popup_width = 60;
    let popup_height = 21;
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
  D               Decompress all records (bulk)
  s               Save selected record (compressed or
                  decompressed based on current state)

Record View:
  s               Save selected record

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
