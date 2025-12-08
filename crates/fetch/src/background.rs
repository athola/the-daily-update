//! Background data fetching

use std::sync::Arc;
use tokio::sync::mpsc;

use api::news::NewsClient;
use api::stocks::StocksClient;
use api::weather::WeatherClient;
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
        tx: mpsc::Sender<FetchUpdate>,
    ) {
        // Fetch news
        if let Some(client) = &self.news_client {
            match client.fetch_top_headlines(Some("us"), None, Some(10)).await {
                Ok(items) => {
                    // Store in database
                    if let Ok(db) = db.lock() {
                        let _ = db.clear_news();
                        for item in &items {
                            let _ = db.insert_news(item);
                        }
                    }
                    let _ = tx.send(FetchUpdate::NewsUpdated(items)).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(FetchUpdate::Error(FetchError {
                            source: "news".to_string(),
                            message: e.to_string(),
                        }))
                        .await;
                }
            }
        }

        // Fetch weather
        if let Some(client) = &self.weather_client {
            let location = &self.config.general.default_location;
            match client.fetch_weather(location).await {
                Ok(data) => {
                    // Store in database
                    if let Ok(db) = db.lock() {
                        let _ = db.upsert_weather(&data);
                    }
                    let _ = tx.send(FetchUpdate::WeatherUpdated(data, WeatherSource::Default)).await;
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
}

/// Check if we appear to be offline
pub fn check_network() -> bool {
    // Simple check - try to resolve a known host
    // In production, could use a more sophisticated check
    std::net::ToSocketAddrs::to_socket_addrs(&("api.tiingo.com", 443)).is_ok()
}
