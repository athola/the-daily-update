//! Tiingo API client for stock data

use chrono::Utc;
use data::models::StockData;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::ApiKey;

const TIINGO_API_BASE: &str = "https://api.tiingo.com/iex";

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

/// Stock name mappings for common symbols
fn get_stock_name(symbol: &str) -> String {
    match symbol.to_uppercase().as_str() {
        "SPY" => "S&P 500 ETF".to_string(),
        "QQQ" => "NASDAQ ETF".to_string(),
        "DIA" => "Dow Jones ETF".to_string(),
        "AAPL" => "Apple Inc.".to_string(),
        "GOOGL" | "GOOG" => "Alphabet Inc.".to_string(),
        "MSFT" => "Microsoft Corp.".to_string(),
        "AMZN" => "Amazon.com".to_string(),
        "TSLA" => "Tesla Inc.".to_string(),
        "META" => "Meta Platforms".to_string(),
        "NVDA" => "NVIDIA Corp.".to_string(),
        _ => symbol.to_uppercase(),
    }
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

    /// Fetch stock data for multiple symbols
    pub async fn fetch_stocks(&self, symbols: &[String]) -> Result<Vec<StockData>, StocksApiError> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let tickers = symbols.join(",");
        let url = format!(
            "{}?tickers={}&token={}",
            TIINGO_API_BASE,
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
        let stocks: Vec<StockData> = body
            .into_iter()
            .map(|item| {
                let change_percent = match (item.last_price, item.prev_close) {
                    (Some(last), Some(prev)) if prev > 0.0 => Some(((last - prev) / prev) * 100.0),
                    _ => None,
                };

                StockData {
                    id: None,
                    symbol: item.ticker.to_uppercase(),
                    name: Some(get_stock_name(&item.ticker)),
                    price: item.last_price,
                    change_percent,
                    fetched_at: now,
                }
            })
            .collect();

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
    fn test_stock_name_mapping() {
        assert_eq!(get_stock_name("SPY"), "S&P 500 ETF");
        assert_eq!(get_stock_name("aapl"), "Apple Inc.");
        assert_eq!(get_stock_name("UNKNOWN"), "UNKNOWN");
    }

    #[test]
    fn test_default_watchlist() {
        let watchlist = StocksClient::default_watchlist();
        assert!(watchlist.contains(&"SPY".to_string()));
        assert_eq!(watchlist.len(), 3);
    }
}
