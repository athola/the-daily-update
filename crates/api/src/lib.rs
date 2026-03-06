//! External API clients

pub mod company_mapping;
pub mod location;
pub mod news;
pub mod stocks;
pub mod weather;

use chrono::NaiveDate;
use data::models::{NewsItem, StockData, WeatherData};
use thiserror::Error;

/// Trait for news data providers
pub trait NewsProvider {
    /// Fetch top headlines, optionally filtered by date
    fn fetch_top_headlines_with_date(
        &self,
        country: Option<&str>,
        category: Option<&str>,
        page_size: Option<u32>,
        from_date: Option<NaiveDate>,
    ) -> impl std::future::Future<Output = Result<Vec<NewsItem>, news::NewsApiError>> + Send;
}

/// Trait for weather data providers
pub trait WeatherProvider {
    /// Fetch current weather for a location
    fn fetch_weather(
        &self,
        location: &str,
    ) -> impl std::future::Future<Output = Result<WeatherData, weather::WeatherApiError>> + Send;
}

/// Trait for stock data providers
pub trait StocksProvider {
    /// Fetch stock data for multiple symbols
    fn fetch_stocks(
        &self,
        symbols: &[String],
    ) -> impl std::future::Future<Output = Result<Vec<StockData>, stocks::StocksApiError>> + Send;
}

/// Error type for API key validation
#[derive(Debug, Error)]
pub enum ApiKeyError {
    #[error("API key cannot be empty")]
    Empty,
    #[error("API key contains invalid characters")]
    InvalidCharacters,
}

/// A validated API key wrapper
///
/// Provides type safety and validation for API keys used across
/// all API clients (news, weather, stocks).
#[derive(Debug, Clone)]
pub struct ApiKey(String);

impl std::str::FromStr for ApiKey {
    type Err = ApiKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = s.trim().to_string();

        if key.is_empty() {
            return Err(ApiKeyError::Empty);
        }

        // Basic validation: alphanumeric, hyphens, and underscores only
        if !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ApiKeyError::InvalidCharacters);
        }

        Ok(Self(key))
    }
}

impl ApiKey {
    /// Create an ApiKey without validation (for trusted sources like env vars)
    pub fn from_trusted(key: String) -> Self {
        Self(key)
    }

    /// Get the key as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the key appears to be a placeholder/test key
    pub fn is_placeholder(&self) -> bool {
        let lower = self.0.to_lowercase();
        lower.contains("your-key")
            || lower.contains("placeholder")
            || lower == "test"
            || lower == "demo"
    }
}

impl AsRef<str> for ApiKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Mask the key for security in logs/display
        // Use .chars() to avoid panicking on non-ASCII from from_trusted()
        let chars: Vec<char> = self.0.chars().collect();
        if chars.len() > 8 {
            let prefix: String = chars[..4].iter().collect();
            let suffix: String = chars[chars.len() - 4..].iter().collect();
            write!(f, "{}...{}", prefix, suffix)
        } else {
            write!(f, "****")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_api_key() {
        let key: Result<ApiKey, _> = "abc123-def456".parse();
        assert!(key.is_ok());
        assert_eq!(key.unwrap().as_str(), "abc123-def456");
    }

    #[test]
    fn test_empty_api_key() {
        let key: Result<ApiKey, _> = "".parse();
        assert!(matches!(key, Err(ApiKeyError::Empty)));
    }

    #[test]
    fn test_whitespace_only_key() {
        let key: Result<ApiKey, _> = "   ".parse();
        assert!(matches!(key, Err(ApiKeyError::Empty)));
    }

    #[test]
    fn test_placeholder_detection() {
        let key = ApiKey::from_trusted("your-key-here".to_string());
        assert!(key.is_placeholder());

        let real = ApiKey::from_trusted("abc123def456".to_string());
        assert!(!real.is_placeholder());
    }

    #[test]
    fn test_display_masks_key() {
        let key = ApiKey::from_trusted("1234567890abcdef".to_string());
        let display = format!("{}", key);
        assert!(!display.contains("567890abc"));
        assert!(display.contains("..."));
    }

    #[test]
    fn test_invalid_characters_rejected() {
        let key: Result<ApiKey, _> = "abc!@#def".parse();
        assert!(matches!(key, Err(ApiKeyError::InvalidCharacters)));
    }

    #[test]
    fn test_display_short_key_fully_masked() {
        let key = ApiKey::from_trusted("short".to_string());
        let display = format!("{}", key);
        assert_eq!(display, "****");
    }

    #[test]
    fn test_display_non_ascii_key_does_not_panic() {
        // from_trusted bypasses validation, so non-ASCII is possible
        let key = ApiKey::from_trusted("café1234résumé".to_string());
        let display = format!("{}", key);
        // Should mask without panicking on multi-byte char boundaries
        assert!(display.contains("..."));
    }
}
