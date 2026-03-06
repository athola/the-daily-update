//! Tiingo API client for stock data

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;
use data::models::StockData;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::ApiKey;

const TIINGO_IEX_BASE: &str = "https://api.tiingo.com/iex";
const TIINGO_META_BASE: &str = "https://api.tiingo.com/tiingo/daily";

#[derive(Debug, Error)]
pub enum StocksApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),
}

/// Tiingo IEX response
#[derive(Debug, Deserialize)]
struct TiingoResponse {
    ticker: String,
    #[serde(rename = "tngoLast")]
    last_price: Option<f64>,
    #[serde(rename = "prevClose")]
    prev_close: Option<f64>,
}

/// Tiingo daily meta response (for company name)
#[derive(Debug, Deserialize)]
struct TiingoMetaResponse {
    #[allow(dead_code)]
    ticker: String,
    name: Option<String>,
}

/// Tiingo API client
pub struct StocksClient {
    api_key: ApiKey,
    client: Client,
    name_cache: Mutex<HashMap<String, String>>,
}

impl StocksClient {
    /// Create a new Tiingo client
    pub fn new(api_key: ApiKey) -> Result<Self, reqwest::Error> {
        Ok(Self {
            api_key,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
            name_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Look up the company name for a ticker, checking cache first
    fn get_cached_name(&self, symbol: &str) -> Option<String> {
        self.name_cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(symbol).cloned())
    }

    /// Store a company name in the cache
    fn cache_name(&self, symbol: &str, name: &str) {
        if let Ok(mut cache) = self.name_cache.lock() {
            cache.insert(symbol.to_string(), name.to_string());
        }
    }

    /// Fetch company name from Tiingo meta endpoint for a single ticker
    async fn fetch_company_name(&self, symbol: &str) -> Result<Option<String>, StocksApiError> {
        let url = format!("{}/{}", TIINGO_META_BASE, symbol.to_lowercase());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_key.as_str()))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let meta: TiingoMetaResponse = response
            .json()
            .await
            .map_err(|e| StocksApiError::ParseError(e.to_string()))?;

        Ok(meta.name)
    }

    /// Resolve the company name for a symbol: cache -> API -> fallback to symbol
    async fn resolve_name(&self, symbol: &str) -> String {
        // Check cache first
        if let Some(name) = self.get_cached_name(symbol) {
            return name;
        }

        // Try fetching from API
        if let Ok(Some(name)) = self.fetch_company_name(symbol).await {
            if !name.is_empty() {
                self.cache_name(symbol, &name);
                return name;
            }
        }

        // Fall back to symbol itself
        symbol.to_uppercase()
    }

    /// Get default watchlist symbols
    pub fn default_watchlist() -> Vec<String> {
        vec!["SPY".to_string(), "QQQ".to_string(), "DIA".to_string()]
    }

    /// Get available stocks for the browser
    pub fn available_stocks() -> Vec<(String, String)> {
        vec![
            ("SPY".to_string(), "S&P 500 ETF".to_string()),
            ("QQQ".to_string(), "NASDAQ ETF".to_string()),
            ("DIA".to_string(), "Dow Jones ETF".to_string()),
            ("AAPL".to_string(), "Apple Inc.".to_string()),
            ("GOOGL".to_string(), "Alphabet Inc.".to_string()),
            ("MSFT".to_string(), "Microsoft Corp.".to_string()),
            ("AMZN".to_string(), "Amazon.com".to_string()),
            ("TSLA".to_string(), "Tesla Inc.".to_string()),
            ("META".to_string(), "Meta Platforms".to_string()),
            ("NVDA".to_string(), "NVIDIA Corp.".to_string()),
        ]
    }
}

impl std::str::FromStr for StocksClient {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(api_key: &str) -> Result<Self, Self::Err> {
        let key: ApiKey = api_key.parse()?;
        Ok(Self::new(key)?)
    }
}

impl crate::StocksProvider for StocksClient {
    /// Fetch stock data for multiple symbols
    async fn fetch_stocks(&self, symbols: &[String]) -> Result<Vec<StockData>, StocksApiError> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let tickers = symbols.join(",");
        let url = format!("{}?tickers={}", TIINGO_IEX_BASE, tickers);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_key.as_str()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<body unreadable: {}>", e));
            return Err(StocksApiError::ApiError(format!(
                "Tiingo error: {} - {}",
                status, body
            )));
        }

        let body: Vec<TiingoResponse> = response
            .json()
            .await
            .map_err(|e| StocksApiError::ParseError(e.to_string()))?;

        let now = Utc::now();
        let mut stocks = Vec::new();

        for item in body {
            let symbol = item.ticker.to_uppercase();
            let name = self.resolve_name(&symbol).await;
            let change_percent = match (item.last_price, item.prev_close) {
                (Some(last), Some(prev)) if prev > 0.0 => Some(((last - prev) / prev) * 100.0),
                _ => None,
            };

            stocks.push(StockData {
                id: None,
                symbol,
                name: Some(name),
                price: item.last_price,
                change_percent,
                fetched_at: now,
            });
        }

        Ok(stocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_client() -> StocksClient {
        "test_key".parse::<StocksClient>().unwrap()
    }

    #[test]
    fn test_name_cache_stores_and_retrieves() {
        let client = make_test_client();
        client.cache_name("TEST_SYM", "Test Company");
        assert_eq!(
            client.get_cached_name("TEST_SYM"),
            Some("Test Company".to_string())
        );
    }

    #[test]
    fn test_name_cache_returns_none_for_unknown() {
        let client = make_test_client();
        assert!(client.get_cached_name("NONEXISTENT_XYZ_999").is_none());
    }

    #[test]
    fn test_name_cache_overwrites_existing() {
        let client = make_test_client();
        client.cache_name("OVERWRITE_SYM", "Old Name");
        client.cache_name("OVERWRITE_SYM", "New Name");
        assert_eq!(
            client.get_cached_name("OVERWRITE_SYM"),
            Some("New Name".to_string())
        );
    }

    #[test]
    fn test_default_watchlist() {
        let watchlist = StocksClient::default_watchlist();
        assert!(watchlist.contains(&"SPY".to_string()));
        assert_eq!(watchlist.len(), 3);
    }

    #[test]
    fn test_available_stocks_contains_major_etfs() {
        let stocks = StocksClient::available_stocks();
        let symbols: Vec<&str> = stocks.iter().map(|(s, _)| s.as_str()).collect();
        assert!(symbols.contains(&"SPY"));
        assert!(symbols.contains(&"QQQ"));
        assert!(symbols.contains(&"DIA"));
    }
}
