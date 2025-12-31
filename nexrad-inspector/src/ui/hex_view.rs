use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Render a hex dump of binary data
pub fn render(frame: &mut Frame, data: &[u8], scroll: usize, area: Rect) {
    let bytes_per_row = 16;
    let total_rows = (data.len() + bytes_per_row - 1) / bytes_per_row;

    // Calculate visible rows based on area height (minus 2 for borders)
    let visible_rows = (area.height as usize).saturating_sub(2);

    // Clamp scroll to valid range
    let max_scroll = total_rows.saturating_sub(visible_rows);
    let effective_scroll = scroll.min(max_scroll);

    let mut lines: Vec<Line> = Vec::new();

    for row_idx in effective_scroll..(effective_scroll + visible_rows).min(total_rows) {
        let offset = row_idx * bytes_per_row;
        let row_data = &data[offset..data.len().min(offset + bytes_per_row)];

        let mut spans = Vec::new();

        // Offset
        spans.push(Span::styled(
            format!("{:08X}  ", offset),
            Style::default().fg(Color::DarkGray),
        ));

        // Hex bytes
        for (i, byte) in row_data.iter().enumerate() {
            let style = get_byte_style(*byte);
            spans.push(Span::styled(format!("{:02X} ", byte), style));

            // Add extra space in middle
            if i == 7 {
                spans.push(Span::raw(" "));
            }
        }

        // Pad if row is not complete
        for i in row_data.len()..bytes_per_row {
            spans.push(Span::raw("   "));
            if i == 7 {
                spans.push(Span::raw(" "));
            }
        }

        spans.push(Span::raw(" "));

        // ASCII representation
        spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));
        for byte in row_data {
            let ch = if *byte >= 0x20 && *byte < 0x7F {
                *byte as char
            } else {
                '.'
            };
            let style = if *byte >= 0x20 && *byte < 0x7F {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(ch.to_string(), style));
        }

        // Pad ASCII if row is not complete
        for _ in row_data.len()..bytes_per_row {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));

        lines.push(Line::from(spans));
    }

    let title = format!(
        " Hex View ({} bytes, row {}/{}) ",
        data.len(),
        effective_scroll + 1,
        total_rows
    );

    let paragraph =
        Paragraph::new(lines).block(Block::default().title(title).borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

/// Get style for a byte based on its value
fn get_byte_style(byte: u8) -> Style {
    match byte {
        0x00 => Style::default().fg(Color::DarkGray),
        0x01..=0x1F => Style::default().fg(Color::Yellow), // Control characters
        0x20..=0x7E => Style::default().fg(Color::Green),  // Printable ASCII
        0x7F => Style::default().fg(Color::Yellow),        // DEL
        0x80..=0xFF => Style::default().fg(Color::Cyan),   // Extended/High bytes
    }
}
