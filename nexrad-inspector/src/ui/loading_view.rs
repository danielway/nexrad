use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

pub fn render(frame: &mut Frame, message: &str, spinner_frame: usize, area: Rect) {
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Length(5),
            Constraint::Percentage(45),
        ])
        .split(area);

    let horizontal_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical_center[1]);

    let spinner = SPINNER_FRAMES[spinner_frame % SPINNER_FRAMES.len()];
    let text = format!("{} {}", spinner, message);

    let paragraph = Paragraph::new(text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Loading")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(paragraph, horizontal_center[1]);
}
