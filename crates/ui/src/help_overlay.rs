//! Help overlay modal

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use app::App;

use super::layout::centered_rect;

/// Render the help overlay
pub fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    // Calculate modal area (60% width, 70% height)
    let modal_area = centered_rect(60, 70, area);

    // Clear the background
    frame.render_widget(Clear, modal_area);

    // Main modal block
    let block = Block::default()
        .title(Span::styled(
            " Help - The Daily Update ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // Build keybinding list based on vim mode
    let keybinds = if app.config.general.vim_mode {
        vec![
            ("j / k", "Navigate up/down"),
            ("h / l", "Switch panels"),
            ("g", "Go to top"),
            ("G", "Go to bottom"),
        ]
    } else {
        vec![("↑ / ↓", "Navigate up/down"), ("Tab", "Switch panels")]
    };

    // Build help text lines
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![]),
    ];

    // Add navigation keybindings
    for (key, desc) in &keybinds {
        lines.push(Line::from(vec![
            Span::styled(format!("{:12}", key), Style::default().fg(Color::Cyan)),
            Span::raw(*desc),
        ]));
    }

    // Add action keybindings
    lines.extend(vec![
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Actions",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled("r           ", Style::default().fg(Color::Cyan)),
            Span::raw("Refresh all data"),
        ]),
        Line::from(vec![
            Span::styled("w           ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle weather panel"),
        ]),
        Line::from(vec![
            Span::styled("s           ", Style::default().fg(Color::Cyan)),
            Span::raw("Open stock browser"),
        ]),
        Line::from(vec![
            Span::styled("q           ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![]),
        Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )]),
    ]);

    let help = Paragraph::new(lines);
    frame.render_widget(help, inner);
}
