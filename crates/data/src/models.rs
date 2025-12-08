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
    /// Location associated with this news item (for weather linking)
    pub location: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // NewsItem Tests
    // ============================================================

    #[test]
    fn given_news_item_when_serialized_then_json_is_valid() {
        let item = NewsItem {
            id: Some(1),
            headline: "Test headline".to_string(),
            source: Some("Reuters".to_string()),
            description: Some("Description".to_string()),
            url: Some("https://example.com".to_string()),
            location: Some("New York".to_string()),
            published_at: chrono::Utc::now(),
            fetched_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&item).expect("Serialization should succeed");
        assert!(json.contains("Test headline"));
        assert!(json.contains("Reuters"));
        assert!(json.contains("New York"));
    }

    #[test]
    fn given_news_item_json_when_deserialized_then_struct_is_correct() {
        let json = r#"{
            "id": 1,
            "headline": "Breaking news",
            "source": "AP",
            "description": "News description",
            "url": "https://ap.com/article",
            "location": "Washington DC",
            "published_at": "2024-01-15T10:00:00Z",
            "fetched_at": "2024-01-15T10:05:00Z"
        }"#;

        let item: NewsItem = serde_json::from_str(json).expect("Deserialization should succeed");
        assert_eq!(item.headline, "Breaking news");
        assert_eq!(item.source, Some("AP".to_string()));
        assert_eq!(item.id, Some(1));
        assert_eq!(item.location, Some("Washington DC".to_string()));
    }

    #[test]
    fn given_news_item_with_null_fields_when_deserialized_then_none_values() {
        let json = r#"{
            "id": null,
            "headline": "Minimal news",
            "source": null,
            "description": null,
            "url": null,
            "location": null,
            "published_at": "2024-01-15T10:00:00Z",
            "fetched_at": "2024-01-15T10:05:00Z"
        }"#;

        let item: NewsItem = serde_json::from_str(json).expect("Deserialization should succeed");
        assert_eq!(item.headline, "Minimal news");
        assert!(item.id.is_none());
        assert!(item.source.is_none());
        assert!(item.description.is_none());
        assert!(item.url.is_none());
        assert!(item.location.is_none());
    }

    #[test]
    fn given_news_item_when_cloned_then_is_equal() {
        let item = NewsItem {
            id: Some(1),
            headline: "Test".to_string(),
            source: None,
            description: None,
            url: None,
            location: None,
            published_at: chrono::Utc::now(),
            fetched_at: chrono::Utc::now(),
        };

        let cloned = item.clone();
        assert_eq!(cloned.headline, item.headline);
        assert_eq!(cloned.id, item.id);
    }

    // ============================================================
    // WeatherData Tests
    // ============================================================

    #[test]
    fn given_weather_data_when_serialized_then_json_is_valid() {
        let data = WeatherData {
            id: Some(1),
            location: "New York".to_string(),
            temperature: Some(72.5),
            condition: Some("Sunny".to_string()),
            humidity: Some(50),
            wind_speed: Some(10.0),
            alert_title: None,
            alert_description: None,
            alert_severity: None,
            fetched_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&data).expect("Serialization should succeed");
        assert!(json.contains("New York"));
        assert!(json.contains("72.5"));
        assert!(json.contains("Sunny"));
    }

    #[test]
    fn given_weather_data_with_alerts_when_serialized_then_alerts_included() {
        let data = WeatherData {
            id: None,
            location: "Miami".to_string(),
            temperature: Some(85.0),
            condition: Some("Stormy".to_string()),
            humidity: Some(90),
            wind_speed: Some(45.0),
            alert_title: Some("Hurricane Warning".to_string()),
            alert_description: Some("Major hurricane approaching".to_string()),
            alert_severity: Some("extreme".to_string()),
            fetched_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&data).expect("Serialization should succeed");
        assert!(json.contains("Hurricane Warning"));
        assert!(json.contains("extreme"));
    }

    #[test]
    fn given_weather_json_when_deserialized_then_struct_is_correct() {
        let json = r#"{
            "id": 5,
            "location": "Chicago",
            "temperature": 45.5,
            "condition": "Cloudy",
            "humidity": 65,
            "wind_speed": 20.0,
            "alert_title": null,
            "alert_description": null,
            "alert_severity": null,
            "fetched_at": "2024-01-15T10:00:00Z"
        }"#;

        let data: WeatherData = serde_json::from_str(json).expect("Deserialization should succeed");
        assert_eq!(data.location, "Chicago");
        assert_eq!(data.temperature, Some(45.5));
        assert_eq!(data.humidity, Some(65));
    }

    // ============================================================
    // StockData Tests
    // ============================================================

    #[test]
    fn given_stock_data_when_serialized_then_json_is_valid() {
        let data = StockData {
            id: Some(1),
            symbol: "AAPL".to_string(),
            name: Some("Apple Inc.".to_string()),
            price: Some(175.50),
            change_percent: Some(2.35),
            fetched_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&data).expect("Serialization should succeed");
        assert!(json.contains("AAPL"));
        assert!(json.contains("Apple Inc."));
        assert!(json.contains("175.5"));
    }

    #[test]
    fn given_stock_data_with_negative_change_when_serialized_then_negative_preserved() {
        let data = StockData {
            id: None,
            symbol: "TSLA".to_string(),
            name: Some("Tesla".to_string()),
            price: Some(250.0),
            change_percent: Some(-5.25),
            fetched_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&data).expect("Serialization should succeed");
        assert!(json.contains("-5.25"));
    }

    #[test]
    fn given_stock_json_when_deserialized_then_struct_is_correct() {
        let json = r#"{
            "id": 10,
            "symbol": "GOOGL",
            "name": "Alphabet Inc.",
            "price": 140.25,
            "change_percent": -1.5,
            "fetched_at": "2024-01-15T15:30:00Z"
        }"#;

        let data: StockData = serde_json::from_str(json).expect("Deserialization should succeed");
        assert_eq!(data.symbol, "GOOGL");
        assert_eq!(data.name, Some("Alphabet Inc.".to_string()));
        assert_eq!(data.price, Some(140.25));
        assert_eq!(data.change_percent, Some(-1.5));
    }

    // ============================================================
    // WatchlistItem Tests
    // ============================================================

    #[test]
    fn given_watchlist_item_when_serialized_then_json_is_valid() {
        let item = WatchlistItem {
            id: Some(1),
            symbol: "SPY".to_string(),
            display_order: 0,
        };

        let json = serde_json::to_string(&item).expect("Serialization should succeed");
        assert!(json.contains("SPY"));
        assert!(json.contains("display_order"));
    }

    #[test]
    fn given_watchlist_json_when_deserialized_then_struct_is_correct() {
        let json = r#"{
            "id": 5,
            "symbol": "QQQ",
            "display_order": 2
        }"#;

        let item: WatchlistItem =
            serde_json::from_str(json).expect("Deserialization should succeed");
        assert_eq!(item.symbol, "QQQ");
        assert_eq!(item.display_order, 2);
        assert_eq!(item.id, Some(5));
    }

    // ============================================================
    // AvailableStock Tests
    // ============================================================

    #[test]
    fn given_available_stock_when_created_then_fields_accessible() {
        let stock = AvailableStock {
            symbol: "MSFT".to_string(),
            name: "Microsoft".to_string(),
            in_watchlist: true,
        };

        assert_eq!(stock.symbol, "MSFT");
        assert_eq!(stock.name, "Microsoft");
        assert!(stock.in_watchlist);
    }

    #[test]
    fn given_available_stock_when_cloned_then_is_equal() {
        let stock = AvailableStock {
            symbol: "AMZN".to_string(),
            name: "Amazon".to_string(),
            in_watchlist: false,
        };

        let cloned = stock.clone();
        assert_eq!(cloned.symbol, stock.symbol);
        assert_eq!(cloned.name, stock.name);
        assert_eq!(cloned.in_watchlist, stock.in_watchlist);
    }
}
