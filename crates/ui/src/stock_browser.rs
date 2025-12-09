//! Stock browser modal

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use app::App;

use super::layout::centered_rect;

/// Render the stock browser modal
pub fn render_stock_browser(frame: &mut Frame, app: &App, area: Rect) {
    // Calculate modal area (50% width, 60% height)
    let modal_area = centered_rect(50, 60, area);

    // Clear the background
    frame.render_widget(Clear, modal_area);

    // Main modal block
    let block = Block::default()
        .title(" Add to Watchlist ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block.clone(), modal_area);

    // Get inner area for content
    let inner = block.inner(modal_area);

    // Split into sections: filter input (top), list (middle), help (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter input area
            Constraint::Min(5),    // Stock list
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Render filter input
    render_filter_input(frame, app, chunks[0]);

    // Render stock list
    render_stock_list(frame, app, chunks[1]);

    // Render help text
    render_help(frame, chunks[2]);
}

/// Render the filter input box
fn render_filter_input(frame: &mut Frame, app: &App, area: Rect) {
    let filter_text = if app.stock_browser.filter.is_empty() {
        "Type to filter..."
    } else {
        &app.stock_browser.filter
    };

    let filter_style = if app.stock_browser.filter.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let filter_block = Block::default()
        .title(" Filter ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let filter_paragraph = Paragraph::new(filter_text)
        .style(filter_style)
        .block(filter_block);

    frame.render_widget(filter_paragraph, area);
}

/// Render the stock list
fn render_stock_list(frame: &mut Frame, app: &App, area: Rect) {
    let filtered_stocks = app.stock_browser.filtered_stocks();

    if filtered_stocks.is_empty() {
        let empty_msg = Paragraph::new("No stocks match your filter")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, area);
        return;
    }

    let items: Vec<ListItem> = filtered_stocks
        .iter()
        .enumerate()
        .map(|(i, stock)| {
            // Determine checkbox and colors
            let (checkbox, checkbox_color) = if stock.in_watchlist {
                ("[x]", Color::Green)
            } else {
                ("[ ]", Color::DarkGray)
            };

            // Build the line with checkbox, symbol, and name
            let line = Line::from(vec![
                Span::styled(checkbox, Style::default().fg(checkbox_color)),
                Span::raw(" "),
                Span::styled(
                    format!("{:6}", stock.symbol),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" "),
                Span::styled(&stock.name, Style::default().fg(Color::White)),
            ]);

            let mut item = ListItem::new(line);

            // Highlight selected item
            if i == app.stock_browser.selected {
                item = item.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }

            item
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}

/// Render help text
fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = Line::from(vec![
        Span::styled("[Up/Down]", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate  "),
        Span::styled("[Space]", Style::default().fg(Color::Cyan)),
        Span::raw(" Toggle  "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" Save  "),
        Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
        Span::raw(" Cancel"),
    ]);

    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, area);
}
