use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new("NEXRAD Inspector")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    let menu_items = vec![
        ListItem::new("Open Local File").style(Style::default()),
        ListItem::new("Query AWS Archive").style(Style::default()),
    ];

    let menu = List::new(menu_items)
        .block(
            Block::default()
                .title("Select an option")
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.menu_selected));

    frame.render_stateful_widget(menu, chunks[1], &mut state);

    let help = Paragraph::new("↑/↓: Navigate | Enter: Select | q: Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}
