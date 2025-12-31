mod app;
mod data;
mod ui;

use std::io;
use std::path::PathBuf;

use app::{App, AppResult};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

#[derive(Parser)]
#[command(name = "nexrad-inspector")]
#[command(author, version, about = "Interactive inspector for NEXRAD Archive II volume files")]
struct Cli {
    /// Path to the Archive II volume file to inspect
    #[arg(required = true)]
    file_path: PathBuf,
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(&cli.file_path)?;
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> AppResult<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Enter => app.enter(),
                    KeyCode::Esc | KeyCode::Backspace => app.back(),
                    KeyCode::Tab => app.toggle_view(),
                    KeyCode::Char('s') => app.save_message()?,
                    KeyCode::Char('?') => app.toggle_help(),
                    KeyCode::PageUp => app.page_up(),
                    KeyCode::PageDown => app.page_down(),
                    _ => {}
                }
            }
        }
    }
}
