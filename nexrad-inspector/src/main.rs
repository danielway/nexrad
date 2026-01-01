mod app;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use app::{App, AppMode, AppResult, AwsStep};
use clap::Parser;
use crossterm::event::{self, poll, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

use ui::text_input::TextInputResult;

#[derive(Parser)]
#[command(name = "nexrad-inspector")]
#[command(
    author,
    version,
    about = "Interactive inspector for NEXRAD Archive II volume files"
)]
struct Cli {
    /// Path to the Archive II volume file to inspect (optional)
    #[arg(required = false)]
    file_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app based on CLI arguments
    let mut app = if let Some(path) = cli.file_path {
        App::new_with_file(&path)?
    } else {
        App::new_interactive()
    };

    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> AppResult<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        // Adjust timeout based on mode
        let timeout = if matches!(app.mode, AppMode::Loading) {
            Duration::from_millis(100)
        } else {
            Duration::from_millis(250)
        };

        // Poll for events with timeout
        if poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if handle_key_event(app, key).await? {
                        return Ok(()); // Quit
                    }
                }
            }
        }

        // Poll pending async operations
        app.poll_pending_operations().await?;

        // Update spinner if loading
        if matches!(app.mode, AppMode::Loading) {
            app.tick_spinner();
        }

        // Check quit flag
        if app.should_quit {
            return Ok(());
        }
    }
}

async fn handle_key_event(app: &mut App, key: event::KeyEvent) -> AppResult<bool> {
    // Dismiss error overlay
    if app.error.is_some() {
        if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
            app.dismiss_error();
        }
        return Ok(false);
    }

    // Global quit (except when typing in text inputs)
    if key.code == KeyCode::Char('q') && !is_text_input_active(app) {
        return Ok(true);
    }

    // Global help toggle (except when typing in text inputs)
    if key.code == KeyCode::Char('?') && !is_text_input_active(app) {
        app.toggle_help();
        return Ok(false);
    }

    // Mode-specific handling
    match app.mode {
        AppMode::Menu => handle_menu_keys(app, key).await,
        AppMode::LocalBrowser => handle_local_browser_keys(app, key).await,
        AppMode::AwsBrowser => handle_aws_browser_keys(app, key).await,
        AppMode::Loading => {
            // Ignore most keys during loading (could add cancellation here)
            Ok(false)
        }
        AppMode::Inspector => handle_inspector_keys(app, key),
    }
}

fn is_text_input_active(app: &App) -> bool {
    matches!(app.mode, AppMode::AwsBrowser) &&
        app.aws_browser.as_ref().map_or(false, |aws| {
            matches!(aws.step, AwsStep::EnterSite | AwsStep::EnterDate)
        })
}

async fn handle_menu_keys(app: &mut App, key: event::KeyEvent) -> AppResult<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.menu_selected > 0 {
                app.menu_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.menu_selected < 1 {
                app.menu_selected += 1;
            }
        }
        KeyCode::Enter => {
            match app.menu_selected {
                0 => app.init_local_browser(),
                1 => app.init_aws_browser(),
                _ => {}
            }
        }
        KeyCode::Esc => {
            // At top level, quit
            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}

async fn handle_local_browser_keys(app: &mut App, key: event::KeyEvent) -> AppResult<bool> {
    if let Some(ref mut state) = app.local_browser {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if state.selected_index > 0 {
                    state.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if state.selected_index < state.entries.len().saturating_sub(1) {
                    state.selected_index += 1;
                }
            }
            KeyCode::PageUp => {
                state.selected_index = state.selected_index.saturating_sub(10);
            }
            KeyCode::PageDown => {
                state.selected_index =
                    (state.selected_index + 10).min(state.entries.len().saturating_sub(1));
            }
            KeyCode::Enter => {
                app.local_browser_enter()?;
            }
            KeyCode::Esc | KeyCode::Backspace => {
                app.back();
            }
            _ => {}
        }
    }
    Ok(false)
}

async fn handle_aws_browser_keys(app: &mut App, key: event::KeyEvent) -> AppResult<bool> {
    if let Some(ref mut state) = app.aws_browser {
        match state.step {
            AwsStep::EnterSite => {
                match state.site_input.handle_key(key.code) {
                    TextInputResult::Submitted => {
                        if !state.site_input.value.is_empty() {
                            state.step = AwsStep::EnterDate;
                        }
                    }
                    TextInputResult::Cancelled => {
                        app.back();
                    }
                    _ => {}
                }
            }
            AwsStep::EnterDate => {
                match state.date_input.handle_key(key.code) {
                    TextInputResult::Submitted => {
                        if !state.date_input.value.is_empty() {
                            use chrono::NaiveDate;

                            let site = state.site_input.value.clone();
                            match NaiveDate::parse_from_str(&state.date_input.value, "%Y-%m-%d") {
                                Ok(date) => {
                                    app.start_aws_list(site, date);
                                }
                                Err(_) => {
                                    app.error = Some("Invalid date format. Use YYYY-MM-DD.".to_string());
                                }
                            }
                        }
                    }
                    TextInputResult::Cancelled => {
                        state.step = AwsStep::EnterSite;
                    }
                    _ => {}
                }
            }
            AwsStep::SelectFile => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if state.selected_index > 0 {
                            state.selected_index -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if state.selected_index < state.files.len().saturating_sub(1) {
                            state.selected_index += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        state.selected_index = state.selected_index.saturating_sub(10);
                    }
                    KeyCode::PageDown => {
                        state.selected_index =
                            (state.selected_index + 10).min(state.files.len().saturating_sub(1));
                    }
                    KeyCode::Enter => {
                        if let Some(identifier) = state.files.get(state.selected_index).cloned() {
                            app.start_aws_download(identifier);
                        }
                    }
                    KeyCode::Esc | KeyCode::Backspace => {
                        state.step = AwsStep::EnterDate;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}

fn handle_inspector_keys(app: &mut App, key: event::KeyEvent) -> AppResult<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.previous(),
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Enter => app.enter(),
        KeyCode::Esc | KeyCode::Backspace => app.back(),
        KeyCode::Tab => app.toggle_view(),
        KeyCode::Char('s') => app.save_message()?,
        KeyCode::Char('d') => app.decompress_selected(),
        KeyCode::PageUp => app.page_up(),
        KeyCode::PageDown => app.page_down(),
        _ => {}
    }
    Ok(false)
}
