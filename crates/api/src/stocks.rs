//! Tiingo API client for stock data

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use data::models::StockData;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::ApiKey;

const TIINGO_IEX_BASE: &str = "https://api.tiingo.com/iex";
const TIINGO_META_BASE: &str = "https://api.tiingo.com/tiingo/daily";

/// In-memory cache for company names (ticker -> name)
static NAME_CACHE: Lazy<Arc<Mutex<HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

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
}

impl StocksClient {
    /// Create a new Tiingo client
    pub fn new(api_key: ApiKey) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    /// Create a StocksClient from a raw string (for backwards compatibility)
    pub fn from_string(api_key: String) -> Self {
        Self::new(ApiKey::from_trusted(api_key))
    }

    /// Look up the company name for a ticker, checking cache first
    fn get_cached_name(symbol: &str) -> Option<String> {
        NAME_CACHE
            .lock()
            .ok()
            .and_then(|cache| cache.get(symbol).cloned())
    }

    /// Store a company name in the cache
    fn cache_name(symbol: &str, name: &str) {
        if let Ok(mut cache) = NAME_CACHE.lock() {
            cache.insert(symbol.to_string(), name.to_string());
        }
    }

    /// Fetch company name from Tiingo meta endpoint for a single ticker
    async fn fetch_company_name(&self, symbol: &str) -> Result<Option<String>, StocksApiError> {
        let url = format!(
            "{}/{}?token={}",
            TIINGO_META_BASE,
            symbol.to_lowercase(),
            self.api_key.as_str()
        );

        let response = self.client.get(&url).send().await?;

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
        if let Some(name) = Self::get_cached_name(symbol) {
            return name;
        }

        // Try fetching from API
        if let Ok(Some(name)) = self.fetch_company_name(symbol).await {
            if !name.is_empty() {
                Self::cache_name(symbol, &name);
                return name;
            }
        }

        // Fall back to symbol itself
        symbol.to_uppercase()
    }

    /// Fetch stock data for multiple symbols
    pub async fn fetch_stocks(&self, symbols: &[String]) -> Result<Vec<StockData>, StocksApiError> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let tickers = symbols.join(",");
        let url = format!(
            "{}?tickers={}&token={}",
            TIINGO_IEX_BASE,
            tickers,
            self.api_key.as_str()
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
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

    /// Fetch data for a single stock
    pub async fn fetch_stock(&self, symbol: &str) -> Result<Option<StockData>, StocksApiError> {
        let stocks = self.fetch_stocks(&[symbol.to_string()]).await?;
        Ok(stocks.into_iter().next())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_cache_stores_and_retrieves() {
        StocksClient::cache_name("TEST_SYM", "Test Company");
        assert_eq!(
            StocksClient::get_cached_name("TEST_SYM"),
            Some("Test Company".to_string())
        );
    }

    #[test]
    fn test_name_cache_returns_none_for_unknown() {
        assert!(StocksClient::get_cached_name("NONEXISTENT_XYZ_999").is_none());
    }

    #[test]
    fn test_name_cache_overwrites_existing() {
        StocksClient::cache_name("OVERWRITE_SYM", "Old Name");
        StocksClient::cache_name("OVERWRITE_SYM", "New Name");
        assert_eq!(
            StocksClient::get_cached_name("OVERWRITE_SYM"),
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
