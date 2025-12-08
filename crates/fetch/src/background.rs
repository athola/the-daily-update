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
        // Fetch news
        if let Some(client) = &self.news_client {
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

        // Fetch weather - try to use location from first news item
        if let Some(client) = &self.weather_client {
            // Determine location: try news context first, fall back to default
            let (location, source) = {
                // Try to get location from first news item
                let news_location = if let Ok(db_guard) = db.lock() {
                    db_guard.get_news(1).ok().and_then(|items| {
                        items.first().and_then(|item| {
                            extract_location_from_news(&item.headline, item.description.as_deref())
                        })
                    })
                } else {
                    None
                };

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
