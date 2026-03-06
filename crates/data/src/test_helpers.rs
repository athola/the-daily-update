//! Shared test helper functions for creating sample data.
//!
//! Available when the `test-helpers` feature is enabled or during testing.
//! Downstream crates can use these by adding `data` as a dev-dependency
//! with the `test-helpers` feature.

use chrono::Utc;

use crate::models::{NewsItem, StockData, WeatherData};

/// Create a test [`NewsItem`] with only a headline; other fields use sensible defaults.
pub fn create_test_news_item(headline: &str) -> NewsItem {
    NewsItem {
        id: None,
        headline: headline.to_string(),
        source: Some("Test Source".to_string()),
        description: Some("Test description".to_string()),
        url: Some("https://example.com".to_string()),
        location: None,
        published_at: Utc::now(),
        fetched_at: Utc::now(),
    }
}

/// Create a test [`NewsItem`] with a specified source.
pub fn create_test_news_item_with_source(headline: &str, source: &str) -> NewsItem {
    NewsItem {
        id: None,
        headline: headline.to_string(),
        source: Some(source.to_string()),
        description: None,
        url: None,
        location: None,
        published_at: Utc::now(),
        fetched_at: Utc::now(),
    }
}

/// Create a test [`WeatherData`] with only a location; other fields use sensible defaults.
pub fn create_test_weather_data(location: &str) -> WeatherData {
    WeatherData {
        id: None,
        location: location.to_string(),
        temperature: Some(72.5),
        condition: Some("Sunny".to_string()),
        humidity: Some(50),
        wind_speed: Some(10.0),
        alert_title: None,
        alert_description: None,
        alert_severity: None,
        fetched_at: Utc::now(),
    }
}

/// Create a test [`WeatherData`] with specified temperature and condition.
pub fn create_test_weather_data_full(location: &str, temp: f64, condition: &str) -> WeatherData {
    WeatherData {
        id: None,
        location: location.to_string(),
        temperature: Some(temp),
        condition: Some(condition.to_string()),
        humidity: Some(50),
        wind_speed: Some(10.0),
        alert_title: None,
        alert_description: None,
        alert_severity: None,
        fetched_at: Utc::now(),
    }
}

/// Create a test [`StockData`] with a symbol and name; price and change use sensible defaults.
pub fn create_test_stock_data(symbol: &str, name: &str) -> StockData {
    StockData {
        id: None,
        symbol: symbol.to_string(),
        name: Some(name.to_string()),
        price: Some(150.0),
        change_percent: Some(1.5),
        fetched_at: Utc::now(),
    }
}

/// Create a test [`StockData`] with specified price and change percent.
pub fn create_test_stock_data_full(symbol: &str, price: f64, change: f64) -> StockData {
    StockData {
        id: None,
        symbol: symbol.to_string(),
        name: Some(format!("{} Inc.", symbol)),
        price: Some(price),
        change_percent: Some(change),
        fetched_at: Utc::now(),
    }
}
