//! Background data fetching

use std::sync::Arc;
use tokio::sync::mpsc;

use api::location::extract_location_from_news;
use api::news::NewsClient;
use api::stocks::StocksClient;
use api::weather::WeatherClient;
use api::{NewsProvider, StocksProvider, WeatherProvider};
use chrono::NaiveDate;
use config::settings::Config;
use data::db::Database;
use data::models::{NewsItem, StockData, WeatherData, WeatherSource};

const MAX_HEADLINES_PER_FETCH: u32 = 10;

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

/// Source of a fetch error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchSource {
    News,
    Weather,
    Stocks,
}

/// Fetch error with context
#[derive(Debug, Clone)]
pub struct FetchError {
    pub source: FetchSource,
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
        let news_client = config.apis.news_api_key.as_ref().and_then(|key| {
            key.parse::<NewsClient>()
                .map_err(|e| eprintln!("Warning: failed to create news client: {}", e))
                .ok()
        });

        let weather_client = config.apis.weather_api_key.as_ref().and_then(|key| {
            key.parse::<WeatherClient>()
                .map_err(|e| eprintln!("Warning: failed to create weather client: {}", e))
                .ok()
        });

        let stocks_client = config.apis.tiingo_api_key.as_ref().and_then(|key| {
            key.parse::<StocksClient>()
                .map_err(|e| eprintln!("Warning: failed to create stocks client: {}", e))
                .ok()
        });

        Self {
            config,
            news_client,
            weather_client,
            stocks_client,
        }
    }

    /// Store news items in the database using spawn_blocking
    async fn store_news(db: &Arc<std::sync::Mutex<Database>>, items: &[NewsItem]) {
        let db = Arc::clone(db);
        let items = items.to_vec();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(db) = db.lock() {
                if let Err(e) = db.replace_news(&items) {
                    eprintln!("Warning: failed to store news in database: {}", e);
                }
            }
        })
        .await;
    }

    /// Store weather data in the database using spawn_blocking
    async fn store_weather(db: &Arc<std::sync::Mutex<Database>>, data: &WeatherData) {
        let db = Arc::clone(db);
        let data = data.clone();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(db) = db.lock() {
                if let Err(e) = db.upsert_weather(&data) {
                    eprintln!("Warning: failed to store weather in database: {}", e);
                }
            }
        })
        .await;
    }

    /// Store stock data in the database using spawn_blocking
    async fn store_stocks(db: &Arc<std::sync::Mutex<Database>>, stocks: &[StockData]) {
        let db = Arc::clone(db);
        let stocks = stocks.to_vec();
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(db) = db.lock() {
                for stock in &stocks {
                    if let Err(e) = db.upsert_stock(stock) {
                        eprintln!(
                            "Warning: failed to store stock {} in database: {}",
                            stock.symbol, e
                        );
                    }
                }
            }
        })
        .await;
    }

    /// Fetch weather data and send result via channel
    async fn fetch_weather_task(
        client: &(impl WeatherProvider + Send + Sync),
        location: String,
        source: WeatherSource,
        db: &Arc<std::sync::Mutex<Database>>,
        tx: &mpsc::Sender<FetchUpdate>,
    ) {
        match client.fetch_weather(&location).await {
            Ok(data) => {
                Self::store_weather(db, &data).await;
                let _ = tx.send(FetchUpdate::WeatherUpdated(data, source)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(FetchUpdate::Error(FetchError {
                        source: FetchSource::Weather,
                        message: e.to_string(),
                    }))
                    .await;
            }
        }
    }

    /// Fetch stock data and send result via channel
    async fn fetch_stocks_task(
        client: &(impl StocksProvider + Send + Sync),
        symbols: Vec<String>,
        db: &Arc<std::sync::Mutex<Database>>,
        tx: &mpsc::Sender<FetchUpdate>,
    ) {
        match client.fetch_stocks(&symbols).await {
            Ok(stocks) => {
                Self::store_stocks(db, &stocks).await;
                let _ = tx.send(FetchUpdate::StocksUpdated(stocks)).await;
            }
            Err(e) => {
                let _ = tx
                    .send(FetchUpdate::Error(FetchError {
                        source: FetchSource::Stocks,
                        message: e.to_string(),
                    }))
                    .await;
            }
        }
    }

    /// Start background fetch, sending updates through channel.
    /// News is fetched first (needed for weather location extraction),
    /// then weather and stocks are fetched concurrently.
    pub async fn fetch_all(
        &self,
        db: Arc<std::sync::Mutex<Database>>,
        watchlist: Vec<String>,
        from_date: Option<NaiveDate>,
        tx: mpsc::Sender<FetchUpdate>,
    ) {
        // Phase 1: Fetch news (needed for weather location extraction)
        let fetched_news = if let Some(client) = &self.news_client {
            match client
                .fetch_top_headlines_with_date(
                    Some("us"),
                    None,
                    Some(MAX_HEADLINES_PER_FETCH),
                    from_date,
                )
                .await
            {
                Ok(items) => {
                    Self::store_news(&db, &items).await;
                    if tx
                        .send(FetchUpdate::NewsUpdated(items.clone()))
                        .await
                        .is_err()
                    {
                        return;
                    }
                    Some(items)
                }
                Err(e) => {
                    if tx
                        .send(FetchUpdate::Error(FetchError {
                            source: FetchSource::News,
                            message: e.to_string(),
                        }))
                        .await
                        .is_err()
                    {
                        return;
                    }
                    None
                }
            }
        } else {
            None
        };

        // Phase 2: Fetch weather and stocks concurrently
        // Determine weather location from news context
        let weather_params = self.weather_client.as_ref().map(|client| {
            let (location, source) = {
                let news_location = fetched_news
                    .as_ref()
                    .and_then(|items| items.first())
                    .and_then(|item| {
                        extract_location_from_news(
                            &item.headline,
                            item.description.as_deref(),
                            item.source.as_deref(),
                        )
                    });

                match news_location {
                    Some(loc) => (loc, WeatherSource::NewsContext),
                    None => (
                        self.config.general.default_location.clone(),
                        WeatherSource::Default,
                    ),
                }
            };
            (client, location, source)
        });

        let stock_params = self.stocks_client.as_ref().map(|client| {
            let symbols = if watchlist.is_empty() {
                StocksClient::default_watchlist()
            } else {
                watchlist
            };
            (client, symbols)
        });

        // Run weather and stocks fetches concurrently
        match (weather_params, stock_params) {
            (Some((weather_client, location, source)), Some((stocks_client, symbols))) => {
                tokio::join!(
                    Self::fetch_weather_task(weather_client, location, source, &db, &tx),
                    Self::fetch_stocks_task(stocks_client, symbols, &db, &tx)
                );
            }
            (Some((weather_client, location, source)), None) => {
                Self::fetch_weather_task(weather_client, location, source, &db, &tx).await;
            }
            (None, Some((stocks_client, symbols))) => {
                Self::fetch_stocks_task(stocks_client, symbols, &db, &tx).await;
            }
            (None, None) => {}
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
            Self::fetch_stocks_task(client, symbols, &db, &tx).await;
        }

        let _ = tx.send(FetchUpdate::Complete).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Helper to create a test NewsItem
    fn create_news_item(headline: &str, description: Option<&str>) -> NewsItem {
        create_news_item_with_source(headline, description, "Test Source")
    }

    /// Helper to create a test NewsItem with a specific source
    fn create_news_item_with_source(
        headline: &str,
        description: Option<&str>,
        source: &str,
    ) -> NewsItem {
        NewsItem {
            id: None,
            headline: headline.to_string(),
            source: Some(source.to_string()),
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

    #[test]
    fn given_news_headline_with_san_francisco_when_extracted_then_returns_san_francisco() {
        let item = create_news_item("Breaking: Fire in San Francisco downtown", None);
        let location = extract_location_from_news(
            &item.headline,
            item.description.as_deref(),
            item.source.as_deref(),
        );
        assert_eq!(location, Some("San Francisco, CA".to_string()));
    }

    #[test]
    fn given_news_headline_without_city_but_description_with_city_when_extracted_then_uses_description(
    ) {
        let item = create_news_item(
            "Major storm approaching",
            Some("Officials in Miami are preparing for severe weather"),
        );
        let location = extract_location_from_news(
            &item.headline,
            item.description.as_deref(),
            item.source.as_deref(),
        );
        assert_eq!(location, Some("Miami, FL".to_string()));
    }

    #[test]
    fn given_news_without_any_city_when_extracted_then_returns_none() {
        let item = create_news_item("Global markets rally on economic news", None);
        let location = extract_location_from_news(
            &item.headline,
            item.description.as_deref(),
            item.source.as_deref(),
        );
        assert_eq!(location, None);
    }

    #[test]
    fn given_empty_news_list_when_determining_location_then_returns_none() {
        let news_items: Vec<NewsItem> = vec![];
        let location = news_items.first().and_then(|item| {
            extract_location_from_news(
                &item.headline,
                item.description.as_deref(),
                item.source.as_deref(),
            )
        });
        assert_eq!(location, None);
    }

    #[test]
    fn given_config_with_default_location_when_no_news_location_then_uses_default() {
        let config = create_test_config("New York, NY");
        let news_items: Vec<NewsItem> = vec![];

        let (location, source) = {
            let news_location = news_items.first().and_then(|item| {
                extract_location_from_news(
                    &item.headline,
                    item.description.as_deref(),
                    item.source.as_deref(),
                )
            });

            match news_location {
                Some(loc) => (loc, WeatherSource::NewsContext),
                None => (
                    config.general.default_location.clone(),
                    WeatherSource::Default,
                ),
            }
        };

        assert_eq!(location, "New York, NY");
        assert_eq!(source, WeatherSource::Default);
    }

    #[test]
    fn given_news_with_location_when_determining_source_then_uses_news_context() {
        let config = create_test_config("Boston, MA");
        let news_items = [create_news_item(
            "Seattle tech company announces layoffs",
            None,
        )];

        let (location, source) = {
            let news_location = news_items.first().and_then(|item| {
                extract_location_from_news(
                    &item.headline,
                    item.description.as_deref(),
                    item.source.as_deref(),
                )
            });

            match news_location {
                Some(loc) => (loc, WeatherSource::NewsContext),
                None => (
                    config.general.default_location.clone(),
                    WeatherSource::Default,
                ),
            }
        };

        assert_eq!(location, "Seattle, WA");
        assert_eq!(source, WeatherSource::NewsContext);
    }

    #[test]
    fn given_multiple_news_items_when_extracting_location_then_uses_first_item_only() {
        let news_items = [
            create_news_item("Boston Marathon attracts thousands", None),
            create_news_item("Seattle sees record rainfall", None),
            create_news_item("Miami beaches crowded for holiday", None),
        ];

        let location = news_items.first().and_then(|item| {
            extract_location_from_news(
                &item.headline,
                item.description.as_deref(),
                item.source.as_deref(),
            )
        });

        assert_eq!(location, Some("Boston, MA".to_string()));
    }

    #[test]
    fn given_news_with_headline_and_description_locations_when_extracted_then_headline_wins() {
        let item = create_news_item(
            "Denver airport delays continue",
            Some("Officials in Phoenix report no issues"),
        );

        let location = extract_location_from_news(
            &item.headline,
            item.description.as_deref(),
            item.source.as_deref(),
        );

        assert_eq!(location, Some("Denver, CO".to_string()));
    }

    #[test]
    fn given_known_source_when_extracting_location_then_uses_source_hq() {
        let item =
            create_news_item_with_source("San Francisco startup raises $10M", None, "Bloomberg");

        let location = extract_location_from_news(
            &item.headline,
            item.description.as_deref(),
            item.source.as_deref(),
        );

        assert_eq!(location, Some("New York, NY".to_string()));
    }

    #[test]
    fn given_bbc_source_when_extracting_location_then_uses_london() {
        let item = create_news_item_with_source("Global economy shows recovery", None, "BBC News");

        let location = extract_location_from_news(
            &item.headline,
            item.description.as_deref(),
            item.source.as_deref(),
        );

        assert_eq!(location, Some("London, UK".to_string()));
    }

    #[test]
    fn given_cnn_source_when_extracting_location_then_uses_atlanta() {
        let item = create_news_item_with_source("Breaking news update", None, "CNN");

        let location = extract_location_from_news(
            &item.headline,
            item.description.as_deref(),
            item.source.as_deref(),
        );

        assert_eq!(location, Some("Atlanta, GA".to_string()));
    }

    #[test]
    fn given_fetch_source_variants_when_compared_then_equal_to_themselves() {
        assert_eq!(FetchSource::News, FetchSource::News);
        assert_eq!(FetchSource::Weather, FetchSource::Weather);
        assert_eq!(FetchSource::Stocks, FetchSource::Stocks);
    }

    #[test]
    fn given_different_fetch_source_variants_when_compared_then_not_equal() {
        assert_ne!(FetchSource::News, FetchSource::Weather);
        assert_ne!(FetchSource::Weather, FetchSource::Stocks);
        assert_ne!(FetchSource::Stocks, FetchSource::News);
    }

    #[test]
    fn given_fetch_error_when_created_then_has_source_and_message() {
        let error = FetchError {
            source: FetchSource::Weather,
            message: "API timeout".to_string(),
        };

        assert_eq!(error.source, FetchSource::Weather);
        assert_eq!(error.message, "API timeout");
    }

    #[test]
    fn given_fetch_errors_for_each_source_when_matched_then_routes_correctly() {
        let sources = [FetchSource::News, FetchSource::Weather, FetchSource::Stocks];

        for source in &sources {
            let error = FetchError {
                source: source.clone(),
                message: "test error".to_string(),
            };

            match error.source {
                FetchSource::News => assert_eq!(source, &FetchSource::News),
                FetchSource::Weather => assert_eq!(source, &FetchSource::Weather),
                FetchSource::Stocks => assert_eq!(source, &FetchSource::Stocks),
            }
        }
    }

    #[test]
    fn given_fetch_update_variants_when_matched_then_correct_types() {
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
