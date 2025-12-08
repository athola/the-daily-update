//! Background data fetching

use std::sync::Arc;
use tokio::sync::mpsc;

use api::location::extract_location_from_news;
use api::news::NewsClient;
use api::stocks::StocksClient;
use api::weather::WeatherClient;
use chrono::NaiveDate;
use config::settings::Config;
use data::db::Database;
use data::models::{NewsItem, StockData, WeatherData, WeatherSource};

/// Messages sent from background fetcher to UI
#[derive(Debug, Clone)]
pub enum FetchUpdate {
    /// News data has been updated
    NewsUpdated(Vec<NewsItem>),
    /// Weather data has been updated with source context
    WeatherUpdated(WeatherData, WeatherSource),
    /// Stocks data has been updated
    StocksUpdated(Vec<StockData>),
    /// An error occurred during fetch
    Error(FetchError),
    /// All fetches complete
    Complete,
}

/// Fetch error with context
#[derive(Debug, Clone)]
pub struct FetchError {
    pub source: String,
    pub message: String,
}

/// Background fetcher that updates data asynchronously
pub struct BackgroundFetcher {
    config: Config,
    news_client: Option<NewsClient>,
    weather_client: Option<WeatherClient>,
    stocks_client: Option<StocksClient>,
}

impl BackgroundFetcher {
    /// Create a new background fetcher
    pub fn new(config: Config) -> Self {
        let news_client = config
            .apis
            .news_api_key
            .as_ref()
            .map(|key| NewsClient::from_string(key.clone()));

        let weather_client = config
            .apis
            .weather_api_key
            .as_ref()
            .map(|key| WeatherClient::from_string(key.clone()));

        let stocks_client = config
            .apis
            .tiingo_api_key
            .as_ref()
            .map(|key| StocksClient::from_string(key.clone()));

        Self {
            config,
            news_client,
            weather_client,
            stocks_client,
        }
    }

    /// Start background fetch, sending updates through channel
    pub async fn fetch_all(
        &self,
        db: Arc<std::sync::Mutex<Database>>,
        watchlist: Vec<String>,
        from_date: Option<NaiveDate>,
        tx: mpsc::Sender<FetchUpdate>,
    ) {
        // Fetch news and store for weather location extraction
        let fetched_news = if let Some(client) = &self.news_client {
            match client
                .fetch_top_headlines_with_date(Some("us"), None, Some(10), from_date)
                .await
            {
                Ok(items) => {
                    // Store in database
                    if let Ok(db) = db.lock() {
                        let _ = db.clear_news();
                        for item in &items {
                            let _ = db.insert_news(item);
                        }
                    }
                    let _ = tx.send(FetchUpdate::NewsUpdated(items.clone())).await;
                    Some(items)
                }
                Err(e) => {
                    let _ = tx
                        .send(FetchUpdate::Error(FetchError {
                            source: "news".to_string(),
                            message: e.to_string(),
                        }))
                        .await;
                    None
                }
            }
        } else {
            None
        };

        // Fetch weather - try to use location from first news item
        if let Some(client) = &self.weather_client {
            // Determine location: try news context first, fall back to default
            let (location, source) = {
                // Try to get location from freshly fetched news
                let news_location = fetched_news
                    .as_ref()
                    .and_then(|items| items.first())
                    .and_then(|item| {
                        extract_location_from_news(&item.headline, item.description.as_deref())
                    });

                match news_location {
                    Some(loc) => (loc, WeatherSource::NewsContext),
                    None => (
                        self.config.general.default_location.clone(),
                        WeatherSource::Default,
                    ),
                }
            };

            match client.fetch_weather(&location).await {
                Ok(data) => {
                    // Store in database
                    if let Ok(db) = db.lock() {
                        let _ = db.upsert_weather(&data);
                    }
                    let _ = tx.send(FetchUpdate::WeatherUpdated(data, source)).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(FetchUpdate::Error(FetchError {
                            source: "weather".to_string(),
                            message: e.to_string(),
                        }))
                        .await;
                }
            }
        }

        // Fetch stocks
        if let Some(client) = &self.stocks_client {
            let symbols = if watchlist.is_empty() {
                StocksClient::default_watchlist()
            } else {
                watchlist
            };

            match client.fetch_stocks(&symbols).await {
                Ok(stocks) => {
                    // Store in database
                    if let Ok(db) = db.lock() {
                        for stock in &stocks {
                            let _ = db.upsert_stock(stock);
                        }
                    }
                    let _ = tx.send(FetchUpdate::StocksUpdated(stocks)).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(FetchUpdate::Error(FetchError {
                            source: "stocks".to_string(),
                            message: e.to_string(),
                        }))
                        .await;
                }
            }
        }

        let _ = tx.send(FetchUpdate::Complete).await;
    }

    /// Fetch stocks only for specific symbols (used after auto-populate adds new stocks)
    pub async fn fetch_stocks_only(
        &self,
        symbols: Vec<String>,
        db: Arc<std::sync::Mutex<Database>>,
        tx: mpsc::Sender<FetchUpdate>,
    ) {
        if symbols.is_empty() {
            let _ = tx.send(FetchUpdate::Complete).await;
            return;
        }

        if let Some(client) = &self.stocks_client {
            match client.fetch_stocks(&symbols).await {
                Ok(stocks) => {
                    // Store in database
                    if let Ok(db) = db.lock() {
                        for stock in &stocks {
                            let _ = db.upsert_stock(stock);
                        }
                    }
                    let _ = tx.send(FetchUpdate::StocksUpdated(stocks)).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(FetchUpdate::Error(FetchError {
                            source: "stocks".to_string(),
                            message: e.to_string(),
                        }))
                        .await;
                }
            }
        }

        let _ = tx.send(FetchUpdate::Complete).await;
    }
}

/// Check if we appear to be offline
pub fn check_network() -> bool {
    // Simple check - try to resolve a known host
    // In production, could use a more sophisticated check
    std::net::ToSocketAddrs::to_socket_addrs(&("api.tiingo.com", 443)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Helper to create a test NewsItem
    fn create_news_item(headline: &str, description: Option<&str>) -> NewsItem {
        NewsItem {
            id: None,
            headline: headline.to_string(),
            source: Some("Test Source".to_string()),
            description: description.map(|s| s.to_string()),
            url: Some("https://example.com".to_string()),
            location: None,
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        }
    }

    /// Helper to create a test Config with default location
    fn create_test_config(default_location: &str) -> Config {
        let mut config = Config::default();
        config.general.default_location = default_location.to_string();
        config
    }

    // Integration test note: Full fetch_all integration testing would require
    // mock API clients. The fetch_all flow is:
    // 1. Fetch news and store in local variable
    // 2. Clone news items for channel, keep original for weather location
    // 3. Extract location from first news headline/description
    // 4. Use WeatherSource::NewsContext if found, else WeatherSource::Default
    //
    // This behavior is verified through:
    // - Unit tests below for location extraction logic
    // - Manual testing with demo/demo-info targets

    #[test]
    fn given_news_headline_with_san_francisco_when_extracted_then_returns_san_francisco() {
        // Test location extraction with news item
        let item = create_news_item("Breaking: Fire in San Francisco downtown", None);
        let location = extract_location_from_news(&item.headline, item.description.as_deref());
        assert_eq!(location, Some("San Francisco, CA".to_string()));
    }

    #[test]
    fn given_news_headline_without_city_but_description_with_city_when_extracted_then_uses_description() {
        // Test fallback to description
        let item = create_news_item(
            "Major storm approaching",
            Some("Officials in Miami are preparing for severe weather"),
        );
        let location = extract_location_from_news(&item.headline, item.description.as_deref());
        assert_eq!(location, Some("Miami, FL".to_string()));
    }

    #[test]
    fn given_news_without_any_city_when_extracted_then_returns_none() {
        // Test no location found scenario
        let item = create_news_item("Global markets rally on economic news", None);
        let location = extract_location_from_news(&item.headline, item.description.as_deref());
        assert_eq!(location, None);
    }

    #[test]
    fn given_empty_news_list_when_determining_location_then_returns_none() {
        // Test empty news scenario
        let news_items: Vec<NewsItem> = vec![];
        let location = news_items
            .first()
            .and_then(|item| extract_location_from_news(&item.headline, item.description.as_deref()));
        assert_eq!(location, None);
    }

    #[test]
    fn given_config_with_default_location_when_no_news_location_then_uses_default() {
        // Test default location fallback
        let config = create_test_config("New York, NY");
        let news_items: Vec<NewsItem> = vec![];

        let (location, source) = {
            let news_location = news_items
                .first()
                .and_then(|item| extract_location_from_news(&item.headline, item.description.as_deref()));

            match news_location {
                Some(loc) => (loc, WeatherSource::NewsContext),
                None => (config.general.default_location.clone(), WeatherSource::Default),
            }
        };

        assert_eq!(location, "New York, NY");
        assert_eq!(source, WeatherSource::Default);
    }

    #[test]
    fn given_news_with_location_when_determining_source_then_uses_news_context() {
        // Test WeatherSource::NewsContext is used when location found in news
        let config = create_test_config("Boston, MA");
        let news_items = [create_news_item("Seattle tech company announces layoffs", None)];

        let (location, source) = {
            let news_location = news_items
                .first()
                .and_then(|item| extract_location_from_news(&item.headline, item.description.as_deref()));

            match news_location {
                Some(loc) => (loc, WeatherSource::NewsContext),
                None => (config.general.default_location.clone(), WeatherSource::Default),
            }
        };

        assert_eq!(location, "Seattle, WA");
        assert_eq!(source, WeatherSource::NewsContext);
    }

    #[test]
    fn given_news_items_when_cloned_then_original_unchanged() {
        // Test that cloning news items for channel doesn't affect original
        let original = vec![
            create_news_item("Headline 1", Some("Description 1")),
            create_news_item("Headline 2", Some("Description 2")),
        ];

        let cloned = original.clone();

        // Verify clone is equal
        assert_eq!(original.len(), cloned.len());
        assert_eq!(original[0].headline, cloned[0].headline);
        assert_eq!(original[1].headline, cloned[1].headline);

        // Both can be used independently for location extraction
        let loc_from_original = extract_location_from_news(
            &original[0].headline,
            original[0].description.as_deref(),
        );
        let loc_from_cloned = extract_location_from_news(
            &cloned[0].headline,
            cloned[0].description.as_deref(),
        );
        assert_eq!(loc_from_original, loc_from_cloned);
    }

    #[test]
    fn given_multiple_news_items_when_extracting_location_then_uses_first_item_only() {
        // Test that only the first news item is used for location extraction
        let news_items = [
            create_news_item("Boston Marathon attracts thousands", None),
            create_news_item("Seattle sees record rainfall", None),
            create_news_item("Miami beaches crowded for holiday", None),
        ];

        // Simulate the logic in fetch_all
        let location = news_items
            .first()
            .and_then(|item| extract_location_from_news(&item.headline, item.description.as_deref()));

        // Should use Boston from first item, not Seattle or Miami
        assert_eq!(location, Some("Boston, MA".to_string()));
    }

    #[test]
    fn given_news_with_headline_and_description_locations_when_extracted_then_headline_wins() {
        // Test that headline takes precedence over description
        let item = create_news_item(
            "Denver airport delays continue",
            Some("Officials in Phoenix report no issues"),
        );

        let location = extract_location_from_news(&item.headline, item.description.as_deref());

        // Should use Denver from headline, not Phoenix from description
        assert_eq!(location, Some("Denver, CO".to_string()));
    }

    #[test]
    fn given_fetch_error_when_created_then_has_source_and_message() {
        // Test FetchError structure
        let error = FetchError {
            source: "weather".to_string(),
            message: "API timeout".to_string(),
        };

        assert_eq!(error.source, "weather");
        assert_eq!(error.message, "API timeout");
    }

    #[test]
    fn given_fetch_update_variants_when_matched_then_correct_types() {
        // Test FetchUpdate enum variants
        let news_update = FetchUpdate::NewsUpdated(vec![]);
        assert!(matches!(news_update, FetchUpdate::NewsUpdated(_)));

        let weather_data = WeatherData {
            id: None,
            location: "Test".to_string(),
            temperature: Some(72.0),
            condition: Some("Sunny".to_string()),
            humidity: Some(50),
            wind_speed: Some(10.0),
            alert_title: None,
            alert_description: None,
            alert_severity: None,
            fetched_at: Utc::now(),
        };
        let weather_update = FetchUpdate::WeatherUpdated(weather_data, WeatherSource::NewsContext);
        assert!(matches!(weather_update, FetchUpdate::WeatherUpdated(_, _)));

        let complete = FetchUpdate::Complete;
        assert!(matches!(complete, FetchUpdate::Complete));
    }
}
