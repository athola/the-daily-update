//! News headlines panel

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use app::{App, Panel};
use data::cache::format_relative_time;

/// Render the news panel
pub fn render_news_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::News;

    // Build title with status
    let mut title_parts = vec![Span::raw(" NEWS ")];

    if app.fetching {
        title_parts.push(Span::styled(
            "[Refreshing...] ",
            Style::default().fg(Color::Cyan),
        ));
    }

    if let Some(err) = &app.errors.news {
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

    // Build list items
    let items: Vec<ListItem> = if app.news.is_empty() {
        vec![ListItem::new(Line::from(vec![Span::styled(
            "  No news available. Press 'r' to refresh.",
            Style::default().fg(Color::DarkGray),
        )]))]
    } else {
        app.news
            .iter()
            .map(|item| {
                let source = item.source.as_deref().unwrap_or("Unknown");
                let time = format_relative_time(item.published_at);

                let line = Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(&item.headline, Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(
                        format!("({}, {})", source, time),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect()
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.news_selected));

    frame.render_stateful_widget(list, area, &mut state);
}
