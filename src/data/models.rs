//! Data models for news, weather, stocks, and watchlist

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A news headline from NewsAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: Option<i64>,
    pub headline: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub published_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
}

/// Weather data from OpenWeatherMap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub id: Option<i64>,
    pub location: String,
    pub temperature: Option<f64>,
    pub condition: Option<String>,
    pub humidity: Option<i32>,
    pub wind_speed: Option<f64>,
    pub alert_title: Option<String>,
    pub alert_description: Option<String>,
    pub alert_severity: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

/// Stock/index data from Tiingo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    pub id: Option<i64>,
    pub symbol: String,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub change_percent: Option<f64>,
    pub fetched_at: DateTime<Utc>,
}

/// A watchlist item (user's saved stocks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: Option<i64>,
    pub symbol: String,
    pub display_order: i32,
}

/// Available stock for the stock browser
#[derive(Debug, Clone)]
pub struct AvailableStock {
    pub symbol: String,
    pub name: String,
    pub in_watchlist: bool,
}
