//! Floating weather widget

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use app::App;
use data::cache::format_relative_time;

/// Render the weather widget (floating in bottom-left of parent area)
pub fn render_weather_widget(frame: &mut Frame, app: &App, parent_area: Rect) {
    if app.weather_expanded {
        render_expanded_weather(frame, app, parent_area);
    } else {
        render_collapsed_weather(frame, app, parent_area);
    }
}

/// Render collapsed weather widget (small floating box)
fn render_collapsed_weather(frame: &mut Frame, app: &App, parent_area: Rect) {
    // Position in bottom-left, small size
    let width = 22;
    let height = 3;

    let x = parent_area.x + 2;
    let y = parent_area.y + parent_area.height.saturating_sub(height + 1);

    let area = Rect::new(x, y, width.min(parent_area.width - 4), height);

    // Clear the area behind the widget
    frame.render_widget(Clear, area);

    let content = match &app.weather {
        Some(weather) => {
            let temp = weather
                .temperature
                .map(|t| format!("{:.0}°F", t))
                .unwrap_or_else(|| "N/A".to_string());

            let icon = get_weather_icon(weather.condition.as_deref());

            // Check for alerts
            let alert_indicator = if weather.alert_title.is_some() {
                Span::styled(" ⚠", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            };

            // Context indicator for news-based location
            let context_indicator =
                if app.weather_source == data::models::WeatherSource::NewsContext {
                    Span::styled(" [news]", Style::default().fg(Color::Magenta))
                } else {
                    Span::raw("")
                };

            vec![Line::from(vec![
                Span::raw(icon),
                Span::raw(" "),
                Span::styled(&weather.location, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(temp, Style::default().fg(Color::Cyan)),
                alert_indicator,
                context_indicator,
            ])]
        }
        None => {
            vec![Line::from(vec![Span::styled(
                "Weather unavailable",
                Style::default().fg(Color::DarkGray),
            )])]
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled("[w]", Style::default().fg(Color::DarkGray)));

    let widget = Paragraph::new(content).block(block);
    frame.render_widget(widget, area);
}

/// Render expanded weather panel (full width)
fn render_expanded_weather(frame: &mut Frame, app: &App, parent_area: Rect) {
    // Full width panel at bottom of parent area
    let height = 8;
    let y = parent_area.y + parent_area.height.saturating_sub(height);
    let area = Rect::new(parent_area.x, y, parent_area.width, height);

    // Clear the area
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" WEATHER ", Style::default().fg(Color::Cyan)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &app.weather {
        Some(weather) => {
            let temp = weather
                .temperature
                .map(|t| format!("{:.0}°F", t))
                .unwrap_or_else(|| "N/A".to_string());

            let humidity = weather
                .humidity
                .map(|h| format!("{}%", h))
                .unwrap_or_else(|| "N/A".to_string());

            let wind = weather
                .wind_speed
                .map(|w| format!("{:.1} mph", w))
                .unwrap_or_else(|| "N/A".to_string());

            let condition = weather.condition.as_deref().unwrap_or("Unknown");
            let icon = get_weather_icon(Some(condition));
            let updated = format_relative_time(weather.fetched_at);

            let context_label = if app.weather_source == data::models::WeatherSource::NewsContext {
                " (from news context)"
            } else {
                ""
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::raw(icon),
                    Span::raw(" "),
                    Span::styled(
                        &weather.location,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(context_label, Style::default().fg(Color::Magenta)),
                    Span::raw(" - "),
                    Span::raw(condition),
                ]),
                Line::from(vec![
                    Span::raw("Temperature: "),
                    Span::styled(temp, Style::default().fg(Color::Cyan)),
                    Span::raw("  │  Humidity: "),
                    Span::styled(humidity, Style::default().fg(Color::Blue)),
                    Span::raw("  │  Wind: "),
                    Span::styled(wind, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![Span::styled(
                    format!("Updated: {}", updated),
                    Style::default().fg(Color::DarkGray),
                )]),
            ];

            // Add alert if present
            if let Some(alert_title) = &weather.alert_title {
                lines.push(Line::from(vec![]));
                lines.push(Line::from(vec![Span::styled(
                    format!("⚠ ALERT: {}", alert_title),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
                if let Some(desc) = &weather.alert_description {
                    // Truncate long descriptions
                    let desc_short: String = desc.chars().take(80).collect();
                    lines.push(Line::from(vec![Span::styled(
                        desc_short,
                        Style::default().fg(Color::Yellow),
                    )]));
                }
            }

            let content = Paragraph::new(lines);
            frame.render_widget(content, inner);
        }
        None => {
            let msg = Paragraph::new("Weather data unavailable. Press 'r' to refresh.")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, inner);
        }
    }
}

/// Get weather icon based on condition
fn get_weather_icon(condition: Option<&str>) -> &'static str {
    match condition.map(|s| s.to_lowercase()).as_deref() {
        Some(c) if c.contains("clear") || c.contains("sun") => "☀",
        Some(c) if c.contains("cloud") => "☁",
        Some(c) if c.contains("rain") || c.contains("drizzle") => "🌧",
        Some(c) if c.contains("thunder") || c.contains("storm") => "⛈",
        Some(c) if c.contains("snow") => "❄",
        Some(c) if c.contains("fog") || c.contains("mist") => "🌫",
        _ => "🌡",
    }
}
