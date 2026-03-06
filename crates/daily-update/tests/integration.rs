//! Integration tests for The Daily Update
//!
//! These tests verify end-to-end workflows across multiple crates:
//! - App state management with data layer
//! - Configuration with database paths
//! - Full data lifecycle (create, store, retrieve)

use app::{App, Panel};
use chrono::Utc;
use config::settings::Config;
use data::cache::{format_relative_time, is_stale, CacheConfig};
use data::db::Database;
use data::models::{NewsItem, StockData, WeatherData};

// ============================================================
// Helper Functions
// ============================================================

fn test_config() -> Config {
    Config::default()
}

// ============================================================
// Smoke Tests
// ============================================================

#[test]
fn integration_smoke_test() {
    // Basic integration test to verify test infrastructure works
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(version, "0.1.0");
}

#[test]
fn given_all_crates_imported_then_they_compile_together() {
    // Verify all crate imports work together
    let db = Database::in_memory().unwrap();
    let config = test_config();
    let app = App::new(config, db);
    let cache_config = CacheConfig::default();

    // Verify each component initializes correctly
    assert!(app.is_running(), "App should be running after creation");
    assert!(
        cache_config.news_max_age.num_hours() >= 1,
        "Cache config should have valid thresholds"
    );
}

// ============================================================
// App + Data Integration Tests
// ============================================================

#[test]
fn given_app_and_database_when_managing_news_then_navigation_works() {
    // Scenario: App navigation behavior with news items
    let db = Database::in_memory().unwrap();

    // Insert 5 news items
    for i in 0..5 {
        let news = NewsItem {
            id: None,
            headline: format!("News item {}", i),
            source: Some("Test Source".to_string()),
            description: None,
            url: None,
            location: None,
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        };
        db.insert_news(&news).unwrap();
    }

    let config = test_config();
    let mut app = App::new(config, db);

    // Verify we loaded news from db
    assert_eq!(app.news.len(), 5, "Should have loaded 5 news items");
    assert_eq!(app.news_selected, 0, "Should start at first item");

    // Navigate through news
    app.news_next();
    assert_eq!(app.news_selected, 1);

    app.news_next();
    assert_eq!(app.news_selected, 2);

    app.news_prev();
    assert_eq!(app.news_selected, 1);
}

#[test]
fn given_app_on_stocks_panel_when_stocks_in_db_then_can_retrieve_watchlist() {
    // Scenario: Stock panel displays stocks from watchlist
    let db = Database::in_memory().unwrap();

    // Add stocks to watchlist
    db.add_to_watchlist("AAPL").unwrap();
    db.add_to_watchlist("GOOGL").unwrap();

    // Insert stock data
    let apple = StockData {
        id: None,
        symbol: "AAPL".to_string(),
        name: Some("Apple Inc.".to_string()),
        price: Some(175.50),
        change_percent: Some(2.5),
        fetched_at: Utc::now(),
    };
    db.upsert_stock(&apple).unwrap();

    let config = test_config();
    let mut app = App::new(config, db);

    // Navigate to stocks panel
    app.next_panel();
    assert_eq!(app.active_panel, Panel::Stocks);

    // Verify watchlist loaded
    assert!(!app.watchlist.is_empty(), "Watchlist should have stocks");
    assert!(
        app.watchlist.contains(&"AAPL".to_string()),
        "Apple should be in watchlist"
    );
}

#[test]
fn given_weather_widget_when_toggled_then_expands_and_collapses() {
    // Scenario: Weather widget can be toggled
    let db = Database::in_memory().unwrap();

    // Insert weather data
    let weather = WeatherData {
        id: None,
        location: "New York, NY".to_string(),
        temperature: Some(72.5),
        condition: Some("Sunny".to_string()),
        humidity: Some(45),
        wind_speed: Some(10.0),
        alert_title: None,
        alert_description: None,
        alert_severity: None,
        fetched_at: Utc::now(),
    };
    db.upsert_weather(&weather).unwrap();

    let config = test_config();
    let mut app = App::new(config, db);

    // Toggle weather expansion
    assert!(!app.weather_expanded, "Weather should start collapsed");
    app.toggle_weather();
    assert!(app.weather_expanded, "Weather should be expanded");
    app.toggle_weather();
    assert!(!app.weather_expanded, "Weather should be collapsed again");
}

// ============================================================
// Data Flow Integration Tests
// ============================================================

#[test]
fn given_fresh_database_when_full_data_flow_then_all_data_persists() {
    // Scenario: Complete data lifecycle - insert, query, update, verify
    let db = Database::in_memory().unwrap();

    // Step 1: Insert news
    let news = NewsItem {
        id: None,
        headline: "Integration test headline".to_string(),
        source: Some("Test Suite".to_string()),
        description: Some("Full integration test".to_string()),
        url: Some("https://test.example.com".to_string()),
        location: None,
        published_at: Utc::now(),
        fetched_at: Utc::now(),
    };
    db.insert_news(&news).unwrap();

    // Step 2: Insert weather
    let weather = WeatherData {
        id: None,
        location: "Integration City".to_string(),
        temperature: Some(65.0),
        condition: Some("Partly Cloudy".to_string()),
        humidity: Some(55),
        wind_speed: Some(12.0),
        alert_title: None,
        alert_description: None,
        alert_severity: None,
        fetched_at: Utc::now(),
    };
    db.upsert_weather(&weather).unwrap();

    // Step 3: Insert stocks and watchlist
    db.add_to_watchlist("MSFT").unwrap();
    let stock = StockData {
        id: None,
        symbol: "MSFT".to_string(),
        name: Some("Microsoft Corporation".to_string()),
        price: Some(378.25),
        change_percent: Some(-0.5),
        fetched_at: Utc::now(),
    };
    db.upsert_stock(&stock).unwrap();

    // Step 4: Verify all data persisted
    let retrieved_news = db.get_news(10).unwrap();
    assert_eq!(retrieved_news.len(), 1);
    assert_eq!(retrieved_news[0].headline, "Integration test headline");

    let retrieved_weather = db.get_weather("Integration City").unwrap().unwrap();
    assert_eq!(retrieved_weather.temperature, Some(65.0));

    let retrieved_stocks = db.get_watchlist_stocks().unwrap();
    assert_eq!(retrieved_stocks.len(), 1);
    assert_eq!(retrieved_stocks[0].symbol, "MSFT");
    assert_eq!(retrieved_stocks[0].price, Some(378.25));
}

#[test]
fn given_multiple_data_types_when_clearing_news_then_other_data_preserved() {
    // Scenario: Clearing news should not affect stocks or weather
    let db = Database::in_memory().unwrap();

    // Insert data of all types
    let news = NewsItem {
        id: None,
        headline: "Will be cleared".to_string(),
        source: None,
        description: None,
        url: None,
        location: None,
        published_at: Utc::now(),
        fetched_at: Utc::now(),
    };
    db.insert_news(&news).unwrap();

    let weather = WeatherData {
        id: None,
        location: "Preserved City".to_string(),
        temperature: Some(70.0),
        condition: None,
        humidity: None,
        wind_speed: None,
        alert_title: None,
        alert_description: None,
        alert_severity: None,
        fetched_at: Utc::now(),
    };
    db.upsert_weather(&weather).unwrap();

    db.add_to_watchlist("NVDA").unwrap();
    let stock = StockData {
        id: None,
        symbol: "NVDA".to_string(),
        name: Some("NVIDIA".to_string()),
        price: Some(450.0),
        change_percent: Some(3.0),
        fetched_at: Utc::now(),
    };
    db.upsert_stock(&stock).unwrap();

    // Clear only news
    db.clear_news().unwrap();

    // Verify news cleared
    let news_after = db.get_news(10).unwrap();
    assert!(news_after.is_empty(), "News should be cleared");

    // Verify other data preserved
    let weather_after = db.get_weather("Preserved City").unwrap();
    assert!(weather_after.is_some(), "Weather should be preserved");

    let stocks_after = db.get_watchlist_stocks().unwrap();
    assert_eq!(stocks_after.len(), 1, "Stock should be preserved");
}

// ============================================================
// Cache + Data Integration Tests
// ============================================================

#[test]
fn given_weather_data_when_checking_staleness_then_cache_config_applies() {
    // Scenario: Weather data staleness checked against cache thresholds
    let db = Database::in_memory().unwrap();
    let cache_config = CacheConfig::default();

    // Insert weather that was fetched now
    let fresh_weather = WeatherData {
        id: None,
        location: "Fresh City".to_string(),
        temperature: Some(80.0),
        condition: Some("Hot".to_string()),
        humidity: Some(30),
        wind_speed: Some(5.0),
        alert_title: None,
        alert_description: None,
        alert_severity: None,
        fetched_at: Utc::now(),
    };
    db.upsert_weather(&fresh_weather).unwrap();

    let retrieved = db.get_weather("Fresh City").unwrap().unwrap();

    // Fresh data should not be stale
    let is_data_stale = is_stale(retrieved.fetched_at, cache_config.weather_max_age);
    assert!(!is_data_stale, "Just-fetched weather should not be stale");
}

#[test]
fn given_stock_data_when_formatted_for_display_then_relative_time_correct() {
    // Scenario: Stock data timestamps formatted for UI display
    let db = Database::in_memory().unwrap();

    db.add_to_watchlist("AMZN").unwrap();
    let stock = StockData {
        id: None,
        symbol: "AMZN".to_string(),
        name: Some("Amazon".to_string()),
        price: Some(178.50),
        change_percent: Some(1.2),
        fetched_at: Utc::now(),
    };
    db.upsert_stock(&stock).unwrap();

    let stocks = db.get_watchlist_stocks().unwrap();
    assert_eq!(stocks.len(), 1);

    // Format the fetch time for display
    let display_time = format_relative_time(stocks[0].fetched_at);
    assert_eq!(display_time, "just now");
}

// ============================================================
// App State Transitions Integration Tests
// ============================================================

#[test]
fn given_app_with_stock_browser_when_toggling_then_state_changes() {
    // Scenario: Stock browser modal state management
    let db = Database::in_memory().unwrap();
    let config = test_config();
    let mut app = App::new(config, db);

    // Open stock browser
    assert!(!app.stock_browser_open, "Stock browser should start closed");
    app.open_stock_browser();
    assert!(app.stock_browser_open, "Stock browser should be open");

    // Close stock browser
    app.close_stock_browser();
    assert!(!app.stock_browser_open, "Stock browser should be closed");
}

#[test]
fn given_app_cycling_panels_when_full_cycle_then_returns_to_start() {
    // Scenario: Full panel cycle returns to starting panel
    let db = Database::in_memory().unwrap();
    let config = test_config();
    let mut app = App::new(config, db);

    let start_panel = app.active_panel;
    assert_eq!(start_panel, Panel::News);

    // Cycle through panels (News -> Stocks -> News)
    app.next_panel();
    assert_eq!(app.active_panel, Panel::Stocks);

    app.next_panel();
    assert_eq!(app.active_panel, Panel::News);

    assert_eq!(
        app.active_panel, start_panel,
        "Should return to starting panel after full cycle"
    );
}

// ============================================================
// Watchlist Management Integration Tests
// ============================================================

#[test]
fn given_watchlist_with_stocks_when_reordering_then_order_persists() {
    // Scenario: Watchlist ordering maintained across operations
    let db = Database::in_memory().unwrap();

    // Add stocks in specific order
    db.add_to_watchlist("SPY").unwrap();
    db.add_to_watchlist("QQQ").unwrap();
    db.add_to_watchlist("IWM").unwrap();

    // Get watchlist
    let watchlist = db.get_watchlist().unwrap();

    // Verify order (by display_order which is assigned at insertion)
    assert_eq!(watchlist[0].symbol, "SPY");
    assert_eq!(watchlist[1].symbol, "QQQ");
    assert_eq!(watchlist[2].symbol, "IWM");
}

#[test]
fn given_watchlist_when_stock_removed_and_readded_then_goes_to_end() {
    // Scenario: Re-adding a stock puts it at the end
    let db = Database::in_memory().unwrap();

    // Add stocks
    db.add_to_watchlist("FIRST").unwrap();
    db.add_to_watchlist("SECOND").unwrap();
    db.add_to_watchlist("THIRD").unwrap();

    // Remove first stock
    db.remove_from_watchlist("FIRST").unwrap();

    // Re-add first stock
    db.add_to_watchlist("FIRST").unwrap();

    // Get watchlist
    let watchlist = db.get_watchlist().unwrap();

    // FIRST should now be at the end
    assert_eq!(watchlist.len(), 3);
    assert_eq!(
        watchlist[2].symbol, "FIRST",
        "Re-added stock should be at end"
    );
}

// ============================================================
// Error Handling Integration Tests
// ============================================================

#[test]
fn given_database_when_getting_nonexistent_weather_then_returns_none() {
    // Scenario: Querying for non-existent data returns None, not error
    let db = Database::in_memory().unwrap();

    let result = db.get_weather("Nonexistent City").unwrap();
    assert!(
        result.is_none(),
        "Should return None for non-existent location"
    );
}

#[test]
fn given_empty_watchlist_when_getting_stocks_then_returns_empty_vec() {
    // Scenario: Empty watchlist returns empty vector, not error
    let db = Database::in_memory().unwrap();

    let stocks = db.get_watchlist_stocks().unwrap();
    assert!(
        stocks.is_empty(),
        "Empty watchlist should return empty vector"
    );
}

#[test]
fn given_app_at_first_item_when_navigating_prev_then_wraps_to_end() {
    // Scenario: Navigation wraps around
    let db = Database::in_memory().unwrap();

    // Insert news items
    for i in 0..3 {
        let news = NewsItem {
            id: None,
            headline: format!("Item {}", i),
            source: None,
            description: None,
            url: None,
            location: None,
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        };
        db.insert_news(&news).unwrap();
    }

    let config = test_config();
    let mut app = App::new(config, db);

    assert_eq!(app.news_selected, 0, "Should start at first item");

    // Go prev from first - should wrap to last
    app.news_prev();
    assert_eq!(
        app.news_selected, 2,
        "Should wrap to last item when going prev from first"
    );
}

// ============================================================
// Combined Workflow Integration Tests
// ============================================================

#[test]
fn given_full_app_session_when_simulating_user_workflow_then_all_operations_succeed() {
    // Scenario: Simulate a realistic user session
    let db = Database::in_memory().unwrap();

    // Add some news
    for i in 0..10 {
        let news = NewsItem {
            id: None,
            headline: format!("Headline {}", i),
            source: Some("Source".to_string()),
            description: None,
            url: None,
            location: None,
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        };
        db.insert_news(&news).unwrap();
    }

    // Add weather
    let weather = WeatherData {
        id: None,
        location: "New York, NY".to_string(),
        temperature: Some(75.0),
        condition: Some("Clear".to_string()),
        humidity: Some(40),
        wind_speed: Some(8.0),
        alert_title: None,
        alert_description: None,
        alert_severity: None,
        fetched_at: Utc::now(),
    };
    db.upsert_weather(&weather).unwrap();

    // Add stocks to watchlist
    db.add_to_watchlist("AAPL").unwrap();
    db.add_to_watchlist("GOOGL").unwrap();

    for symbol in &["AAPL", "GOOGL"] {
        let stock = StockData {
            id: None,
            symbol: symbol.to_string(),
            name: Some(format!("{} Inc.", symbol)),
            price: Some(150.0),
            change_percent: Some(1.5),
            fetched_at: Utc::now(),
        };
        db.upsert_stock(&stock).unwrap();
    }

    let config = test_config();
    let mut app = App::new(config, db);

    // 1. User starts on News panel, scrolls through headlines
    assert_eq!(app.active_panel, Panel::News);
    assert_eq!(app.news.len(), 10, "Should have loaded 10 news items");

    // Scroll through news
    app.news_next();
    app.news_next();
    assert_eq!(app.news_selected, 2);

    // 2. User switches to Stocks panel
    app.next_panel();
    assert_eq!(app.active_panel, Panel::Stocks);

    // User opens stock browser
    app.open_stock_browser();
    assert!(app.stock_browser_open);

    // Navigate in stock browser
    app.stock_browser_next();
    app.stock_browser_next();

    // User closes stock browser
    app.close_stock_browser();
    assert!(!app.stock_browser_open);

    // 3. User expands weather details
    app.toggle_weather();
    assert!(app.weather_expanded);

    // 4. User quits
    app.quit();
    assert!(!app.is_running());
}

// ============================================================
// Stock Browser Integration Tests
// ============================================================

#[test]
fn given_stock_browser_when_filtering_then_only_matching_shown() {
    // Scenario: Stock browser filter works correctly
    let db = Database::in_memory().unwrap();
    let config = test_config();
    let mut app = App::new(config, db);

    app.open_stock_browser();

    // Initially all stocks available
    let initial_count = app.stock_browser.filtered_stocks().len();
    assert!(initial_count > 0, "Should have available stocks");

    // Apply filter
    app.stock_browser.filter = "AAPL".to_string();
    let filtered = app.stock_browser.filtered_stocks();

    // Should only show Apple
    assert!(
        filtered.len() < initial_count,
        "Filter should reduce results"
    );
    assert!(
        filtered.iter().any(|s| s.symbol == "AAPL"),
        "Should include AAPL in filtered results"
    );
}

#[test]
fn given_stock_browser_when_toggling_then_updates_selection() {
    // Scenario: Toggling a stock updates its watchlist status
    let db = Database::in_memory().unwrap();
    let config = test_config();
    let mut app = App::new(config, db);

    app.open_stock_browser();

    // Find initial state of first stock
    let first_symbol = app.stock_browser.available[0].symbol.clone();
    let initially_in_watchlist = app.stock_browser.available[0].in_watchlist;

    // Toggle it
    app.stock_browser_toggle();

    // Check it changed
    let after_toggle = app
        .stock_browser
        .available
        .iter()
        .find(|s| s.symbol == first_symbol)
        .unwrap();
    assert_eq!(
        after_toggle.in_watchlist, !initially_in_watchlist,
        "Watchlist status should be toggled"
    );
}
