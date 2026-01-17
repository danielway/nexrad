use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Type of byte region for visual distinction in hex view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    /// Message header bytes
    Header,
    /// Message payload/content bytes
    Payload,
}

/// A region of bytes with a specific type for styling
#[derive(Debug, Clone)]
pub struct HexRegion {
    /// Starting byte offset
    pub start: usize,
    /// Length in bytes
    pub len: usize,
    /// Type of region
    pub region_type: RegionType,
}

impl HexRegion {
    pub fn header(start: usize, len: usize) -> Self {
        Self {
            start,
            len,
            region_type: RegionType::Header,
        }
    }

    pub fn payload(start: usize, len: usize) -> Self {
        Self {
            start,
            len,
            region_type: RegionType::Payload,
        }
    }

    /// Check if this region contains the given byte offset
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.start + self.len
    }
}

/// Find the region containing a given byte offset
fn find_region(regions: &[HexRegion], offset: usize) -> Option<&HexRegion> {
    regions.iter().find(|r| r.contains(offset))
}

/// Check if a region starts within this row (16-byte range starting at row_offset)
fn region_starting_in_row(regions: &[HexRegion], row_offset: usize) -> Option<&HexRegion> {
    let row_end = row_offset + 16;
    regions
        .iter()
        .find(|r| r.start >= row_offset && r.start < row_end)
}

/// Render a hex dump with region highlighting
pub fn render(frame: &mut Frame, data: &[u8], scroll: usize, area: Rect, regions: &[HexRegion]) {
    let bytes_per_row = 16;
    let total_rows = data.len().div_ceil(bytes_per_row);

    // Calculate visible rows based on area height (minus 2 for borders)
    let visible_rows = (area.height as usize).saturating_sub(2);

    // Clamp scroll to valid range
    let max_scroll = total_rows.saturating_sub(visible_rows);
    let effective_scroll = scroll.min(max_scroll);

    let mut lines: Vec<Line> = Vec::new();

    for row_idx in effective_scroll..(effective_scroll + visible_rows).min(total_rows) {
        let row_start_offset = row_idx * bytes_per_row;
        let row_data = &data[row_start_offset..data.len().min(row_start_offset + bytes_per_row)];

        let mut spans = Vec::new();

        // Check if a region starts within this row and add marker
        let region_marker = if let Some(region) = region_starting_in_row(regions, row_start_offset)
        {
            match region.region_type {
                RegionType::Header => "H",
                RegionType::Payload => "P",
            }
        } else {
            " "
        };

        // Region marker + Offset
        spans.push(Span::styled(
            format!("{} {:08X}  ", region_marker, row_start_offset),
            Style::default().fg(Color::DarkGray),
        ));

        // Hex bytes with region-aware styling
        for (i, byte) in row_data.iter().enumerate() {
            let byte_offset = row_start_offset + i;
            let style = get_byte_style_with_region(*byte, find_region(regions, byte_offset));
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

        // ASCII representation with region-aware styling
        spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));
        for (i, byte) in row_data.iter().enumerate() {
            let byte_offset = row_start_offset + i;
            let ch = if *byte >= 0x20 && *byte < 0x7F {
                *byte as char
            } else {
                '.'
            };
            let region = find_region(regions, byte_offset);
            let style = get_ascii_style_with_region(*byte, region);
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

/// Get style for a byte based on its value and region
fn get_byte_style_with_region(byte: u8, region: Option<&HexRegion>) -> Style {
    // Base color from byte value
    let base_color = match byte {
        0x00 => Color::DarkGray,
        0x01..=0x1F => Color::Yellow, // Control characters
        0x20..=0x7E => Color::Green,  // Printable ASCII
        0x7F => Color::Yellow,        // DEL
        0x80..=0xFF => Color::Cyan,   // Extended/High bytes
    };

    // Apply region-based styling
    match region.map(|r| r.region_type) {
        Some(RegionType::Header) => {
            // Headers get a magenta tint and bold
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD)
        }
        Some(RegionType::Payload) => {
            // Payload keeps base color
            Style::default().fg(base_color)
        }
        None => {
            // No region info, use base color
            Style::default().fg(base_color)
        }
    }
}

/// Get style for ASCII representation based on byte value and region
fn get_ascii_style_with_region(byte: u8, region: Option<&HexRegion>) -> Style {
    let is_printable = (0x20..0x7F).contains(&byte);

    match region.map(|r| r.region_type) {
        Some(RegionType::Header) => {
            if is_printable {
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            }
        }
        Some(RegionType::Payload) | None => {
            if is_printable {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            }
        }
    }
}
