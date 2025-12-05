//! OpenWeatherMap client for fetching weather data

use chrono::Utc;
use data::models::WeatherData;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

const WEATHER_API_BASE_URL: &str = "https://api.openweathermap.org/data/2.5";

#[derive(Debug, Error)]
pub enum WeatherApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),
}

/// Response from OpenWeatherMap /data/2.5/weather endpoint
#[derive(Debug, Deserialize)]
pub struct WeatherApiResponse {
    pub main: WeatherMain,
    pub weather: Vec<WeatherCondition>,
    pub wind: WeatherWind,
    pub name: String,
}

/// Main weather metrics from API response
#[derive(Debug, Deserialize)]
pub struct WeatherMain {
    pub temp: f64,
    pub humidity: i32,
}

/// Weather condition from API response
#[derive(Debug, Deserialize)]
pub struct WeatherCondition {
    pub main: String,
    pub description: String,
}

/// Wind data from API response
#[derive(Debug, Deserialize)]
pub struct WeatherWind {
    pub speed: f64,
}

/// Client for interacting with OpenWeatherMap API
pub struct WeatherClient {
    api_key: String,
    client: Client,
}

impl WeatherClient {
    /// Create a new OpenWeatherMap client with the provided API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    /// Fetch current weather data for a location
    ///
    /// # Arguments
    /// * `location` - Location name (e.g., "New York", "London,uk")
    ///
    /// # Returns
    /// * `Ok(WeatherData)` - Weather data for the location
    /// * `Err(WeatherApiError)` - If the request fails or API returns an error
    pub async fn fetch_weather(&self, location: &str) -> Result<WeatherData, WeatherApiError> {
        let url = format!("{}/weather", WEATHER_API_BASE_URL);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("q", location),
                ("units", "imperial"),
                ("appid", &self.api_key),
            ])
            .send()
            .await?;

        // Check if the response is successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(WeatherApiError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let api_response: WeatherApiResponse = response.json().await?;

        // Convert API response to WeatherData
        let weather_data = WeatherData {
            id: None, // Will be set when saved to database
            location: api_response.name,
            temperature: Some(api_response.main.temp),
            condition: api_response.weather.first().map(|w| w.main.clone()),
            humidity: Some(api_response.main.humidity),
            wind_speed: Some(api_response.wind.speed),
            alert_title: None, // Alerts are not included in current weather endpoint
            alert_description: None,
            alert_severity: None,
            fetched_at: Utc::now(),
        };

        Ok(weather_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_client_creation() {
        let client = WeatherClient::new("test_api_key".to_string());
        assert_eq!(client.api_key, "test_api_key");
    }

    #[test]
    fn test_weather_api_response_deserialization() {
        let json = r#"{
            "main": {
                "temp": 72.5,
                "humidity": 65
            },
            "weather": [
                {
                    "main": "Clear",
                    "description": "clear sky"
                }
            ],
            "wind": {
                "speed": 10.5
            },
            "name": "New York"
        }"#;

        let response: WeatherApiResponse = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(response.main.temp, 72.5);
        assert_eq!(response.main.humidity, 65);
        assert_eq!(response.weather[0].main, "Clear");
        assert_eq!(response.wind.speed, 10.5);
        assert_eq!(response.name, "New York");
    }
}
