//! The Daily Update - Terminal news aggregator

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use config::settings::Config;
use config::setup::{display_banner, needs_setup, run_setup_wizard};
use data::db::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Load or create configuration
    let mut config = Config::load()?;

    // Run first-time setup if needed
    if needs_setup(&config) {
        display_banner();
        run_setup_wizard(&mut config)?;
    }

    // Check if we have API keys after setup
    if !config.has_api_keys() {
        eprintln!("Error: Missing API keys. Please set environment variables:");
        for key in config.missing_api_keys() {
            eprintln!("  export {}=\"your-key\"", key);
        }
        std::process::exit(1);
    }

    // Initialize database
    let db_path = Config::database_path()?;
    let db = Database::open(&db_path)?;

    // Create application state
    let mut app = App::new(config, db);

    // Start initial background fetch
    app.start_fetch();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Process any pending fetch updates
        app.process_fetch_updates();

        // Draw UI
        terminal.draw(|frame| {
            ui::layout::render(frame, app);
        })?;

        // Handle input with timeout for async updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle stock browser modal inputs first
                if app.stock_browser_open {
                    match key.code {
                        KeyCode::Esc => app.close_stock_browser(),
                        KeyCode::Enter => app.save_stock_browser(),
                        KeyCode::Up => app.stock_browser_prev(),
                        KeyCode::Down => app.stock_browser_next(),
                        KeyCode::Char(' ') => app.stock_browser_toggle(),
                        KeyCode::Char('j') if app.config.general.vim_mode => {
                            app.stock_browser_next()
                        }
                        KeyCode::Char('k') if app.config.general.vim_mode => {
                            app.stock_browser_prev()
                        }
                        KeyCode::Backspace => {
                            app.stock_browser.filter.pop();
                            app.stock_browser.selected = 0;
                        }
                        KeyCode::Char(c) if c.is_alphanumeric() => {
                            app.stock_browser.filter.push(c);
                            app.stock_browser.selected = 0;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Global keybindings
                match key.code {
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.quit()
                    }
                    KeyCode::Char('r') => app.start_fetch(),
                    KeyCode::Char('w') => app.toggle_weather(),
                    KeyCode::Char('s') => app.open_stock_browser(),
                    KeyCode::Tab => app.next_panel(),
                    KeyCode::BackTab => app.next_panel(), // Same as Tab for now

                    // Navigation
                    KeyCode::Up => app.news_prev(),
                    KeyCode::Down => app.news_next(),
                    KeyCode::Char('j') if app.config.general.vim_mode => app.news_next(),
                    KeyCode::Char('k') if app.config.general.vim_mode => app.news_prev(),

                    _ => {}
                }
            }
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
