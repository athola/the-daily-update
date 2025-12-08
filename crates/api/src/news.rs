//! NewsAPI client for fetching top headlines

use chrono::{DateTime, NaiveDate, Utc};
use data::models::NewsItem;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::location::extract_location_from_news;
use crate::ApiKey;

const NEWS_API_BASE_URL: &str = "https://newsapi.org/v2";
const USER_AGENT: &str = "TheDailyUpdate/0.1.0 (https://github.com/athola/the-daily-update)";

#[derive(Debug, Error)]
pub enum NewsApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),
}

/// Response from NewsAPI /v2/top-headlines endpoint
#[derive(Debug, Deserialize)]
pub struct NewsApiResponse {
    pub status: String,
    #[serde(rename = "totalResults")]
    pub total_results: Option<u32>,
    pub articles: Option<Vec<NewsApiArticle>>,
    pub code: Option<String>,
    pub message: Option<String>,
}

/// Article from NewsAPI response
#[derive(Debug, Deserialize)]
pub struct NewsApiArticle {
    pub source: NewsApiSource,
    pub author: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    #[serde(rename = "urlToImage")]
    pub url_to_image: Option<String>,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    pub content: Option<String>,
}

/// Source information from NewsAPI
#[derive(Debug, Deserialize)]
pub struct NewsApiSource {
    pub id: Option<String>,
    pub name: String,
}

/// Client for interacting with NewsAPI
pub struct NewsClient {
    api_key: ApiKey,
    client: Client,
}

impl NewsClient {
    /// Create a new NewsAPI client with the provided API key
    pub fn new(api_key: ApiKey) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    /// Create a NewsClient from a raw string (for backwards compatibility)
    pub fn from_string(api_key: String) -> Self {
        Self::new(ApiKey::from_trusted(api_key))
    }

    /// Fetch top headlines from NewsAPI
    ///
    /// # Arguments
    /// * `country` - Optional country code (e.g., "us", "gb")
    /// * `category` - Optional category (e.g., "business", "technology")
    /// * `page_size` - Optional number of results to return (max 100)
    /// * `from_date` - Optional date to filter articles from (inclusive)
    pub async fn fetch_top_headlines(
        &self,
        country: Option<&str>,
        category: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<Vec<NewsItem>, NewsApiError> {
        self.fetch_top_headlines_with_date(country, category, page_size, None).await
    }

    /// Fetch top headlines with optional date filtering
    ///
    /// # Arguments
    /// * `country` - Optional country code (e.g., "us", "gb")
    /// * `category` - Optional category (e.g., "business", "technology")
    /// * `page_size` - Optional number of results to return (max 100)
    /// * `from_date` - Optional date to filter articles from (inclusive)
    pub async fn fetch_top_headlines_with_date(
        &self,
        country: Option<&str>,
        category: Option<&str>,
        page_size: Option<u32>,
        from_date: Option<NaiveDate>,
    ) -> Result<Vec<NewsItem>, NewsApiError> {
        let url = format!("{}/top-headlines", NEWS_API_BASE_URL);

        let mut request = self
            .client
            .get(&url)
            .header("X-Api-Key", self.api_key.as_str())
            .header("User-Agent", USER_AGENT);

        // Add query parameters
        if let Some(country) = country {
            request = request.query(&[("country", country)]);
        }
        if let Some(category) = category {
            request = request.query(&[("category", category)]);
        }
        if let Some(page_size) = page_size {
            request = request.query(&[("pageSize", page_size.to_string())]);
        }
        if let Some(date) = from_date {
            request = request.query(&[("from", date.format("%Y-%m-%d").to_string())]);
        }

        let response = request.send().await?;
        let api_response: NewsApiResponse = response.json().await?;

        // Check if the API returned an error
        if api_response.status != "ok" {
            let error_msg = api_response
                .message
                .unwrap_or_else(|| "Unknown API error".to_string());
            return Err(NewsApiError::ApiError(error_msg));
        }

        // Convert API articles to NewsItem
        let articles = api_response.articles.unwrap_or_default();
        let fetched_at = Utc::now();
        let mut news_items = Vec::new();

        for article in articles {
            // Parse the published_at timestamp
            let published_at = match DateTime::parse_from_rfc3339(&article.published_at) {
                Ok(dt) => dt.with_timezone(&Utc),
                Err(e) => {
                    // Skip articles with invalid timestamps
                    eprintln!(
                        "Failed to parse timestamp '{}': {}",
                        article.published_at, e
                    );
                    continue;
                }
            };

            // Extract location from headline and description
            let location = extract_location_from_news(
                &article.title,
                article.description.as_deref(),
            );

            news_items.push(NewsItem {
                id: None, // Will be set when saved to database
                headline: article.title,
                source: Some(article.source.name),
                description: article.description.clone(),
                url: Some(article.url),
                location,
                published_at,
                fetched_at,
            });
        }

        Ok(news_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_client_creation() {
        let client = NewsClient::from_string("test_api_key".to_string());
        assert_eq!(client.api_key.as_str(), "test_api_key");
    }

    #[test]
    fn test_news_client_with_api_key() {
        let api_key = ApiKey::from_trusted("my-news-key".to_string());
        let client = NewsClient::new(api_key);
        assert_eq!(client.api_key.as_str(), "my-news-key");
    }
}
