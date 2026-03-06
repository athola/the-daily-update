//! UI rendering tests for The Daily Update TUI.
//!
//! Uses Ratatui's TestBackend to verify that all panels render correctly
//! without needing a real terminal.

use ratatui::{backend::TestBackend, Terminal};

use app::{App, Panel};
use config::settings::Config;
use data::db::Database;
use data::test_helpers::{
    create_test_news_item_with_source, create_test_stock_data_full, create_test_weather_data_full,
};

// ── Test Helpers ──

/// Create a test terminal with a given width and height.
fn test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("Failed to create test terminal")
}

/// Extract the rendered text from the test terminal buffer.
fn render_to_string(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer().clone();
    let mut output = String::new();
    for y in 0..buffer.area.height {
        let mut line = String::new();
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            line.push_str(cell.symbol());
        }
        output.push_str(line.trim_end());
        output.push('\n');
    }
    output
}

/// Create a test App with default config and in-memory database.
fn make_test_app() -> App {
    let config = Config::default();
    let db = Database::in_memory().unwrap();
    App::new(config, db)
}

/// Create a test App pre-populated with sample data.
fn app_with_data() -> App {
    let mut app = make_test_app();
    app.news = vec![
        create_test_news_item_with_source("Tech stocks rally on earnings", "Reuters"),
        create_test_news_item_with_source("Fed holds rates steady", "AP"),
        create_test_news_item_with_source("Oil prices drop amid supply concerns", "Bloomberg"),
    ];
    app.stocks = vec![
        create_test_stock_data_full("SPY", 450.25, 1.5),
        create_test_stock_data_full("AAPL", 175.50, -0.75),
        create_test_stock_data_full("TSLA", 250.00, 3.2),
    ];
    app.weather = Some(create_test_weather_data_full("New York", 72.0, "Clear"));
    app
}

// ── News Panel Tests ──

#[test]
fn news_panel_renders_headlines() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Tech stocks rally on earnings"),
        "News panel should show first headline"
    );
    assert!(
        output.contains("Fed holds rates steady"),
        "News panel should show second headline"
    );
    assert!(
        output.contains("Oil prices drop"),
        "News panel should show third headline"
    );
}

#[test]
fn news_panel_shows_sources() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Reuters"),
        "News panel should show source name"
    );
}

#[test]
fn news_panel_shows_title() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(output.contains("NEWS"), "News panel should have NEWS title");
}

#[test]
fn news_panel_empty_state() {
    let mut terminal = test_terminal(120, 30);
    let app = make_test_app();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("No news available"),
        "Empty news panel should show placeholder message"
    );
}

#[test]
fn news_panel_shows_refreshing_indicator() {
    let mut terminal = test_terminal(120, 30);
    let mut app = app_with_data();
    app.fetching = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Refreshing"),
        "News panel should show refreshing indicator when fetching"
    );
}

#[test]
fn news_panel_shows_error() {
    let mut terminal = test_terminal(120, 30);
    let mut app = app_with_data();
    app.errors.news = Some("API rate limit exceeded".to_string());

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("API rate limit exceeded"),
        "News panel should show error message in title"
    );
}

// ── Stocks Panel Tests ──

#[test]
fn stocks_panel_renders_symbols() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("SPY"),
        "Stocks panel should show SPY symbol"
    );
    assert!(
        output.contains("AAPL"),
        "Stocks panel should show AAPL symbol"
    );
    assert!(
        output.contains("TSLA"),
        "Stocks panel should show TSLA symbol"
    );
}

#[test]
fn stocks_panel_shows_prices() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("450.25"),
        "Stocks panel should show SPY price"
    );
    assert!(
        output.contains("175.50"),
        "Stocks panel should show AAPL price"
    );
}

#[test]
fn stocks_panel_shows_positive_change() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("+1.50%"),
        "Should show positive change with + prefix"
    );
}

#[test]
fn stocks_panel_shows_negative_change() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(output.contains("-0.75%"), "Should show negative change");
}

#[test]
fn stocks_panel_shows_arrows() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("▲"),
        "Should show up arrow for positive change"
    );
    assert!(
        output.contains("▼"),
        "Should show down arrow for negative change"
    );
}

#[test]
fn stocks_panel_shows_title() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("STOCKS"),
        "Stocks panel should have STOCKS title"
    );
}

#[test]
fn stocks_panel_empty_state() {
    let mut terminal = test_terminal(120, 30);
    let mut app = make_test_app();
    app.stocks.clear();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("No stocks in watchlist"),
        "Empty stocks panel should show placeholder"
    );
}

#[test]
fn stocks_panel_shows_error() {
    let mut terminal = test_terminal(120, 30);
    let mut app = app_with_data();
    app.errors.stocks = Some("Tiingo API error".to_string());

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Tiingo API error"),
        "Stocks panel should show error in title"
    );
}

// ── Status Bar Tests ──

#[test]
fn status_bar_shows_today() {
    let mut terminal = test_terminal(120, 30);
    let app = make_test_app();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Today"),
        "Status bar should show 'Today' when no date selected"
    );
}

#[test]
fn status_bar_shows_keybindings() {
    let mut terminal = test_terminal(120, 30);
    let app = make_test_app();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Refresh"),
        "Status bar should show refresh hint"
    );
    assert!(output.contains("Help"), "Status bar should show help hint");
    assert!(output.contains("Quit"), "Status bar should show quit hint");
}

#[test]
fn status_bar_shows_offline_indicator() {
    let mut terminal = test_terminal(120, 30);
    let mut app = make_test_app();
    app.offline = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("OFFLINE"),
        "Status bar should show OFFLINE when offline"
    );
}

#[test]
fn status_bar_shows_vim_keybindings_in_vim_mode() {
    let mut terminal = test_terminal(120, 30);
    let mut app = make_test_app();
    app.config.general.vim_mode = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("h/l"),
        "Vim mode should show h/l for date navigation"
    );
    assert!(
        output.contains("j/k"),
        "Vim mode should show j/k for navigation"
    );
}

// ── Weather Widget Tests ──

#[test]
fn weather_collapsed_shows_location_and_temp() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("New York"),
        "Collapsed weather should show location"
    );
    assert!(
        output.contains("72"),
        "Collapsed weather should show temperature"
    );
}

#[test]
fn weather_collapsed_shows_icon() {
    let mut terminal = test_terminal(120, 30);
    let mut app = make_test_app();
    app.weather = Some(create_test_weather_data_full("Miami", 85.0, "Clear sky"));

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    // "Clear" condition should produce sun icon
    assert!(output.contains("☀"), "Clear weather should show sun icon");
}

#[test]
fn weather_unavailable_shows_message() {
    let mut terminal = test_terminal(120, 30);
    let mut app = make_test_app();
    app.weather = None;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Weather unavailable"),
        "Should show unavailable message when no weather data"
    );
}

#[test]
fn weather_expanded_shows_details() {
    let mut terminal = test_terminal(120, 30);
    let mut app = app_with_data();
    app.weather_expanded = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("WEATHER"),
        "Expanded weather should show WEATHER title"
    );
    assert!(
        output.contains("Temperature"),
        "Expanded weather should show Temperature label"
    );
    assert!(
        output.contains("Humidity"),
        "Expanded weather should show Humidity label"
    );
    assert!(
        output.contains("Wind"),
        "Expanded weather should show Wind label"
    );
}

#[test]
fn weather_expanded_shows_alert() {
    let mut terminal = test_terminal(120, 30);
    let mut app = make_test_app();
    let mut weather = create_test_weather_data_full("Miami", 85.0, "Stormy");
    weather.alert_title = Some("Hurricane Warning".to_string());
    weather.alert_description = Some("Major hurricane approaching".to_string());
    app.weather = Some(weather);
    app.weather_expanded = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Hurricane Warning"),
        "Expanded weather should show alert title"
    );
}

#[test]
fn weather_collapsed_shows_alert_indicator() {
    let mut terminal = test_terminal(120, 30);
    let mut app = make_test_app();
    let mut weather = create_test_weather_data_full("Miami", 85.0, "Stormy");
    weather.alert_title = Some("Tornado Warning".to_string());
    app.weather = Some(weather);

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("⚠"),
        "Collapsed weather with alert should show warning indicator"
    );
}

// ── Panel Focus Tests ──

#[test]
fn news_panel_has_active_border_by_default() {
    let mut terminal = test_terminal(120, 30);
    let app = make_test_app();

    assert_eq!(
        app.active_panel,
        Panel::News,
        "News should be active by default"
    );

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    // Verify rendering succeeds with News as active panel
    let output = render_to_string(&terminal);
    assert!(output.contains("NEWS"), "News panel should be rendered");
}

#[test]
fn tab_switches_active_panel() {
    let mut app = make_test_app();
    assert_eq!(app.active_panel, Panel::News);

    app.next_panel();
    assert_eq!(app.active_panel, Panel::Stocks);

    app.next_panel();
    assert_eq!(app.active_panel, Panel::News);
}

#[test]
fn both_panels_render_after_tab_switch() {
    let mut terminal = test_terminal(120, 30);
    let mut app = app_with_data();

    // Switch to stocks panel
    app.next_panel();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("NEWS"),
        "News panel should still be visible"
    );
    assert!(
        output.contains("STOCKS"),
        "Stocks panel should still be visible"
    );
}

// ── Help Overlay Tests ──

#[test]
fn help_overlay_renders_when_open() {
    let mut terminal = test_terminal(120, 40);
    let mut app = make_test_app();
    app.help_open = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Help - The Daily Update"),
        "Help overlay should show title"
    );
    assert!(
        output.contains("Navigation"),
        "Help overlay should show navigation section"
    );
    assert!(
        output.contains("Actions"),
        "Help overlay should show actions section"
    );
}

#[test]
fn help_overlay_shows_keybindings() {
    let mut terminal = test_terminal(120, 40);
    let mut app = make_test_app();
    app.help_open = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Refresh all data"),
        "Help should describe refresh action"
    );
    assert!(
        output.contains("Toggle weather"),
        "Help should describe weather toggle"
    );
    assert!(
        output.contains("Quit application"),
        "Help should describe quit action"
    );
}

#[test]
fn help_overlay_shows_vim_bindings_in_vim_mode() {
    let mut terminal = test_terminal(120, 40);
    let mut app = make_test_app();
    app.config.general.vim_mode = true;
    app.help_open = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("j / k"),
        "Help should show vim navigation keys in vim mode"
    );
}

#[test]
fn help_overlay_not_visible_when_closed() {
    let mut terminal = test_terminal(120, 40);
    let app = make_test_app();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        !output.contains("Help - The Daily Update"),
        "Help overlay should not be visible when help_open is false"
    );
}

// ── Stock Browser Modal Tests ──

#[test]
fn stock_browser_renders_when_open() {
    let mut terminal = test_terminal(120, 40);
    let mut app = make_test_app();
    app.open_stock_browser();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Add to Watchlist"),
        "Stock browser should show title"
    );
    assert!(
        output.contains("Filter"),
        "Stock browser should show filter input"
    );
}

#[test]
fn stock_browser_shows_stock_list() {
    let mut terminal = test_terminal(120, 40);
    let mut app = make_test_app();
    app.open_stock_browser();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    // Should show some of the available stocks
    assert!(
        output.contains("AAPL") || output.contains("MSFT") || output.contains("GOOGL"),
        "Stock browser should display available stocks"
    );
}

#[test]
fn stock_browser_shows_help_text() {
    // Use wider terminal so help text isn't truncated by the 50%-width modal
    let mut terminal = test_terminal(160, 40);
    let mut app = make_test_app();
    app.open_stock_browser();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Navigate"),
        "Stock browser should show navigation help"
    );
    assert!(
        output.contains("Toggle"),
        "Stock browser should show toggle help"
    );
    assert!(
        output.contains("Save"),
        "Stock browser should show save help"
    );
    assert!(
        output.contains("Cancel"),
        "Stock browser should show cancel help"
    );
}

#[test]
fn stock_browser_not_visible_when_closed() {
    let mut terminal = test_terminal(120, 40);
    let app = make_test_app();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        !output.contains("Add to Watchlist"),
        "Stock browser should not be visible when closed"
    );
}

// ── Terminal Size Tests ──

#[test]
fn renders_at_minimum_terminal_size() {
    let mut terminal = test_terminal(40, 15);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("NEWS"),
        "Should render NEWS panel title even at minimum size"
    );
}

#[test]
fn renders_at_large_terminal_size() {
    let mut terminal = test_terminal(200, 60);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(output.contains("NEWS"), "Large terminal should show NEWS");
    assert!(
        output.contains("STOCKS"),
        "Large terminal should show STOCKS"
    );
    assert!(
        output.contains("Tech stocks rally on earnings"),
        "Large terminal should show full headlines"
    );
}

// ── Full Dashboard Snapshot Tests ──

#[test]
fn full_dashboard_renders_all_components() {
    let mut terminal = test_terminal(120, 30);
    let app = app_with_data();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);

    // News panel
    assert!(output.contains("NEWS"), "Should have NEWS panel");
    assert!(
        output.contains("Tech stocks rally"),
        "Should have news headlines"
    );

    // Stocks panel
    assert!(output.contains("STOCKS"), "Should have STOCKS panel");
    assert!(output.contains("SPY"), "Should have stock symbols");

    // Status bar
    assert!(output.contains("Today"), "Should show today in status bar");

    // Weather widget
    assert!(
        output.contains("New York") || output.contains("[w]"),
        "Should show weather widget"
    );
}

#[test]
fn empty_dashboard_renders_gracefully() {
    let mut terminal = test_terminal(120, 30);
    let app = make_test_app();

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(output.contains("NEWS"), "Should still show NEWS title");
    assert!(
        output.contains("No news available"),
        "Should show empty news message"
    );
    assert!(
        output.contains("No stocks in watchlist"),
        "Should show empty stocks message"
    );
}

#[test]
fn dashboard_with_all_overlays() {
    let mut terminal = test_terminal(120, 40);
    let mut app = app_with_data();
    app.weather_expanded = true;
    app.help_open = true;

    terminal
        .draw(|f| ui::layout::render(f, &app))
        .expect("draw failed");

    let output = render_to_string(&terminal);
    assert!(
        output.contains("Help - The Daily Update"),
        "Help overlay should render on top"
    );
}
