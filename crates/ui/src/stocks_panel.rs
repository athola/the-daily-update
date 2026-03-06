//! Stocks watchlist panel

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use app::{App, Panel};
use data::cache::format_relative_time;
use data::models::StockData;

const MAX_STOCK_CARDS: usize = 6;

/// Render the stocks panel
pub fn render_stocks_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Stocks;

    // Build title with status
    let mut title_parts = vec![Span::raw(" STOCKS ")];

    if let Some(first_stock) = app.stocks.first() {
        title_parts.push(Span::styled(
            format!(
                "Last updated: {} ",
                format_relative_time(first_stock.fetched_at)
            ),
            Style::default().fg(Color::DarkGray),
        ));
    }

    if let Some(err) = &app.errors.stocks {
        title_parts.push(Span::styled(
            format!("⚠ {} ", err),
            Style::default().fg(Color::Yellow),
        ));
    }

    let block = Block::default()
        .title(Line::from(title_parts))
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    frame.render_widget(block, area);

    // Inner area for stock cards
    let inner = Block::default().inner(area);

    if app.stocks.is_empty() {
        let msg = Paragraph::new("No stocks in watchlist. Press 's' to add some.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    // Calculate card layout - horizontal row of cards
    let card_count = app.stocks.len().min(MAX_STOCK_CARDS);
    let base = 100u16 / card_count as u16;
    let remainder = 100u16 % card_count as u16;
    let constraints: Vec<Constraint> = (0..card_count)
        .map(|i| {
            let pct = if i == card_count - 1 {
                base + remainder
            } else {
                base
            };
            Constraint::Percentage(pct)
        })
        .collect();

    let card_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .margin(1)
        .split(inner);

    // Render each stock card
    for (i, stock) in app.stocks.iter().take(MAX_STOCK_CARDS).enumerate() {
        render_stock_card(frame, stock, card_areas[i]);
    }
}

/// Render a single stock card
fn render_stock_card(frame: &mut Frame, stock: &StockData, area: Rect) {
    let price = stock
        .price
        .map(|p| format!("{:.2}", p))
        .unwrap_or_else(|| "N/A".to_string());

    let (change_str, change_color, arrow) = match stock.change_percent {
        Some(pct) if pct > 0.0 => (format!("+{:.2}%", pct), Color::Green, "▲"),
        Some(pct) if pct < 0.0 => (format!("{:.2}%", pct), Color::Red, "▼"),
        Some(_) => ("0.00%".to_string(), Color::DarkGray, "─"),
        None => ("N/A".to_string(), Color::DarkGray, " "),
    };

    let card = Block::default()
        .title(Span::styled(
            &stock.symbol,
            Style::default().fg(Color::White),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(card.clone(), area);

    let inner = card.inner(area);

    // Only render content if we have enough space
    if inner.height >= 2 {
        let content = vec![
            Line::from(vec![Span::styled(price, Style::default().fg(Color::White))]),
            Line::from(vec![
                Span::styled(change_str, Style::default().fg(change_color)),
                Span::raw(" "),
                Span::styled(arrow, Style::default().fg(change_color)),
            ]),
        ];

        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, inner);
    }
}
