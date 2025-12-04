//! Main layout composition

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use app::App;

use super::news_panel::render_news_panel;
use super::stock_browser::render_stock_browser;
use super::stocks_panel::render_stocks_panel;
use super::weather_widget::render_weather_widget;

/// Render the main layout
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main vertical split: News (top), Stocks (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // News panel
            Constraint::Percentage(45), // Stocks panel
            Constraint::Length(1),      // Status bar
        ])
        .split(area);

    // Render main panels
    render_news_panel(frame, app, chunks[0]);
    render_stocks_panel(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Render weather widget (floating, bottom-left of stocks area)
    render_weather_widget(frame, app, chunks[1]);

    // Render modal overlays if active
    if app.stock_browser_open {
        render_stock_browser(frame, app, area);
    }

    // Help overlay (if we add a help_open flag later)
    // render_help_overlay(frame, app, area);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    let mut spans = vec![];

    // Offline indicator
    if app.offline {
        spans.push(Span::styled(
            " OFFLINE ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ));
        spans.push(Span::raw(" "));
    }

    // Fetching indicator
    if app.fetching {
        spans.push(Span::styled(
            "Refreshing...",
            Style::default().fg(Color::Cyan),
        ));
        spans.push(Span::raw(" | "));
    }

    // Keybindings
    let keybinds = if app.config.general.vim_mode {
        "[j/k] Navigate  [Tab] Switch  [r] Refresh  [s] Stocks  [w] Weather  [?] Help  [q] Quit"
    } else {
        "[Up/Down] Navigate  [Tab] Switch  [r] Refresh  [s] Stocks  [w] Weather  [?] Help  [q] Quit"
    };

    spans.push(Span::styled(keybinds, Style::default().fg(Color::DarkGray)));

    let status = Paragraph::new(Line::from(spans));
    frame.render_widget(status, area);
}

/// Calculate the area for a centered modal
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
