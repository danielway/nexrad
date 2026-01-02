use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextInputResult {
    Updated,
    Submitted,
    Cancelled,
    Ignored,
}

#[derive(Debug, Clone)]
pub struct TextInput {
    pub value: String,
    pub cursor: usize,
    pub label: String,
    pub placeholder: String,
}

impl TextInput {
    pub fn new(label: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            label: label.into(),
            placeholder: placeholder.into(),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> TextInputResult {
        match key {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += 1;
                TextInputResult::Updated
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                    TextInputResult::Updated
                } else {
                    TextInputResult::Ignored
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                    TextInputResult::Updated
                } else {
                    TextInputResult::Ignored
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    TextInputResult::Updated
                } else {
                    TextInputResult::Ignored
                }
            }
            KeyCode::Right => {
                if self.cursor < self.value.len() {
                    self.cursor += 1;
                    TextInputResult::Updated
                } else {
                    TextInputResult::Ignored
                }
            }
            KeyCode::Home => {
                self.cursor = 0;
                TextInputResult::Updated
            }
            KeyCode::End => {
                self.cursor = self.value.len();
                TextInputResult::Updated
            }
            KeyCode::Enter => TextInputResult::Submitted,
            KeyCode::Esc => TextInputResult::Cancelled,
            _ => TextInputResult::Ignored,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(self.label.clone())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let display_text = if self.value.is_empty() {
            format!("{} ", self.placeholder)
        } else {
            format!("{} ", self.value)
        };

        let style = if self.value.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(display_text.clone()).style(style);
        frame.render_widget(paragraph, inner);

        if self.cursor <= display_text.len() {
            let cursor_x = inner.x + self.cursor as u16;
            let cursor_y = inner.y;
            if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
                frame.set_cursor_position(Position::new(cursor_x, cursor_y));
            }
        }
    }
}
