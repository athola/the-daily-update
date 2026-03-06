//! OpenWeatherMap client for fetching weather data

use chrono::Utc;
use data::models::WeatherData;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::ApiKey;

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
    pub wind: Option<WeatherWind>,
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
    api_key: ApiKey,
    client: Client,
}

/// Normalize location string for OpenWeatherMap API
///
/// OpenWeatherMap rejects US state abbreviations in location strings.
/// This function converts "City, STATE" to "City,US" format.
///
/// # Arguments
/// * `location` - Original location string
///
/// # Returns
/// Normalized location string suitable for OpenWeatherMap API
///
/// # Examples
/// ```
/// use api::weather::normalize_location;
///
/// assert_eq!(normalize_location("Austin, TX"), "Austin,US");
/// assert_eq!(normalize_location("New York, NY"), "New York,US");
/// assert_eq!(normalize_location("London, UK"), "London, UK");
/// assert_eq!(normalize_location("Paris"), "Paris");
/// assert_eq!(normalize_location("Boston,US"), "Boston,US");
/// ```
pub fn normalize_location(location: &str) -> String {
    // List of US state abbreviations
    const US_STATES: &[&str] = &[
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
        "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT",
        "VA", "WA", "WV", "WI", "WY", "DC",
    ];

    // Check if location contains a comma
    // Use rfind to handle city names with commas (e.g., "Washington, D.C., DC")
    if let Some(comma_pos) = location.rfind(',') {
        let (city_part, state_part) = location.split_at(comma_pos);

        // Guard against empty state part after comma
        if state_part.len() <= 1 {
            return location.to_string();
        }

        let state_trimmed = state_part[1..].trim().to_uppercase();

        // Check if it's a US state abbreviation
        if US_STATES.contains(&state_trimmed.as_str()) {
            return format!("{},US", city_part.trim());
        }
    }

    // Return original location if no transformation needed
    location.to_string()
}

/// NWS alerts API response
#[derive(Debug, Deserialize)]
struct NwsAlertsResponse {
    features: Vec<NwsAlertFeature>,
}

/// NWS alert feature
#[derive(Debug, Deserialize)]
struct NwsAlertFeature {
    properties: NwsAlertProperties,
}

/// NWS alert properties
#[derive(Debug, Deserialize)]
struct NwsAlertProperties {
    event: Option<String>,
    headline: Option<String>,
    description: Option<String>,
    severity: Option<String>,
}

/// OpenWeatherMap geocoding response (for lat/lon lookup)
#[derive(Debug, Deserialize)]
struct GeocodingResponse {
    lat: f64,
    lon: f64,
}

const NWS_API_BASE: &str = "https://api.weather.gov";
const NWS_USER_AGENT: &str = concat!("TheDailyUpdate/", env!("CARGO_PKG_VERSION"), " (https://github.com/athola/the-daily-update)");
const OWM_GEO_BASE: &str = "https://api.openweathermap.org/geo/1.0";

impl WeatherClient {
    /// Create a new OpenWeatherMap client with the provided API key
    pub fn new(api_key: ApiKey) -> Result<Self, reqwest::Error> {
        Ok(Self {
            api_key,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }

    /// Geocode a location string to lat/lon using OpenWeatherMap geocoding API
    async fn geocode(&self, location: &str) -> Result<Option<(f64, f64)>, WeatherApiError> {
        let url = format!("{}/direct", OWM_GEO_BASE);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("q", location),
                ("limit", "1"),
                ("appid", self.api_key.as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let results: Vec<GeocodingResponse> = response
            .json()
            .await
            .map_err(|e| WeatherApiError::ParseError(e.to_string()))?;

        Ok(results.first().map(|r| (r.lat, r.lon)))
    }

    /// Fetch weather alerts from NWS API using lat/lon coordinates
    async fn fetch_nws_alerts(&self, lat: f64, lon: f64) -> Option<(String, String, String)> {
        let url = format!("{}/alerts/active?point={:.4},{:.4}", NWS_API_BASE, lat, lon);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", NWS_USER_AGENT)
            .header("Accept", "application/geo+json")
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let alerts: NwsAlertsResponse = response.json().await.ok()?;

        // Return the most severe/recent alert
        alerts.features.into_iter().next().map(|f| {
            let props = f.properties;
            let title = props
                .event
                .or(props.headline)
                .unwrap_or_else(|| "Weather Alert".to_string());
            let description = props.description.unwrap_or_default();
            let severity = props.severity.unwrap_or_else(|| "Unknown".to_string());
            (title, description, severity)
        })
    }
}

impl std::str::FromStr for WeatherClient {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(api_key: &str) -> Result<Self, Self::Err> {
        let key: ApiKey = api_key.parse()?;
        Ok(Self::new(key)?)
    }
}

impl crate::WeatherProvider for WeatherClient {
    /// Fetch current weather data for a location
    ///
    /// Fetches weather from OpenWeatherMap and alerts from NWS (for US locations).
    async fn fetch_weather(&self, location: &str) -> Result<WeatherData, WeatherApiError> {
        let url = format!("{}/weather", WEATHER_API_BASE_URL);

        // Normalize location to handle US state abbreviations
        let normalized_location = normalize_location(location);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("q", normalized_location.as_str()),
                ("units", "imperial"),
                ("appid", self.api_key.as_str()),
            ])
            .send()
            .await?;

        // Check if the response is successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<body unreadable: {}>", e));
            return Err(WeatherApiError::ApiError(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let api_response: WeatherApiResponse = response.json().await?;

        // Try to fetch alerts from NWS using geocoding
        let alert = match self.geocode(&normalized_location).await {
            Ok(Some((lat, lon))) => self.fetch_nws_alerts(lat, lon).await,
            _ => None,
        };
        let (alert_title, alert_description, alert_severity) = match alert {
            Some((title, desc, severity)) => (Some(title), Some(desc), Some(severity)),
            None => (None, None, None),
        };

        // Convert API response to WeatherData
        let weather_data = WeatherData {
            id: None,
            location: api_response.name,
            temperature: Some(api_response.main.temp),
            condition: api_response.weather.first().map(|w| w.main.clone()),
            humidity: Some(api_response.main.humidity),
            wind_speed: api_response.wind.map(|w| w.speed),
            alert_title,
            alert_description,
            alert_severity,
            fetched_at: Utc::now(),
        };

        Ok(weather_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Location Normalization Tests
    // ============================================================

    #[test]
    fn given_austin_tx_when_normalized_then_converts_to_austin_us() {
        assert_eq!(normalize_location("Austin, TX"), "Austin,US");
    }

    #[test]
    fn given_new_york_ny_when_normalized_then_converts_to_new_york_us() {
        assert_eq!(normalize_location("New York, NY"), "New York,US");
    }

    #[test]
    fn given_san_francisco_ca_when_normalized_then_converts_to_san_francisco_us() {
        assert_eq!(normalize_location("San Francisco, CA"), "San Francisco,US");
    }

    #[test]
    fn given_miami_fl_when_normalized_then_converts_to_miami_us() {
        assert_eq!(normalize_location("Miami, FL"), "Miami,US");
    }

    #[test]
    fn given_seattle_wa_when_normalized_then_converts_to_seattle_us() {
        assert_eq!(normalize_location("Seattle, WA"), "Seattle,US");
    }

    #[test]
    fn given_washington_dc_when_normalized_then_converts_to_washington_us() {
        assert_eq!(normalize_location("Washington, DC"), "Washington,US");
    }

    #[test]
    fn given_lowercase_state_when_normalized_then_converts_to_city_us() {
        assert_eq!(normalize_location("Boston, ma"), "Boston,US");
    }

    #[test]
    fn given_mixed_case_state_when_normalized_then_converts_to_city_us() {
        assert_eq!(normalize_location("Denver, Co"), "Denver,US");
    }

    #[test]
    fn given_no_space_after_comma_when_normalized_then_converts_correctly() {
        assert_eq!(normalize_location("Portland,OR"), "Portland,US");
    }

    #[test]
    fn given_extra_spaces_when_normalized_then_trims_correctly() {
        assert_eq!(normalize_location("Chicago,  IL  "), "Chicago,US");
    }

    #[test]
    fn given_city_only_when_normalized_then_returns_unchanged() {
        assert_eq!(normalize_location("London"), "London");
    }

    #[test]
    fn given_city_with_country_code_when_normalized_then_returns_unchanged() {
        assert_eq!(normalize_location("London,UK"), "London,UK");
    }

    #[test]
    fn given_city_with_full_country_name_when_normalized_then_returns_unchanged() {
        assert_eq!(normalize_location("Paris, France"), "Paris, France");
    }

    #[test]
    fn given_already_us_format_when_normalized_then_returns_unchanged() {
        assert_eq!(normalize_location("Boston,US"), "Boston,US");
    }

    #[test]
    fn given_international_location_when_normalized_then_returns_unchanged() {
        assert_eq!(normalize_location("Tokyo, Japan"), "Tokyo, Japan");
    }

    #[test]
    fn given_non_state_abbreviation_when_normalized_then_returns_unchanged() {
        assert_eq!(normalize_location("City, AB"), "City, AB");
    }

    #[test]
    fn given_empty_string_when_normalized_then_returns_empty() {
        assert_eq!(normalize_location(""), "");
    }

    #[test]
    fn given_trailing_comma_when_normalized_then_returns_unchanged() {
        assert_eq!(normalize_location("City,"), "City,");
    }

    #[test]
    fn given_just_comma_when_normalized_then_returns_unchanged() {
        assert_eq!(normalize_location(","), ",");
    }

    #[test]
    fn given_multi_comma_location_when_normalized_then_handles_correctly() {
        // DC after "D.C.," should be recognized as state abbreviation
        assert_eq!(
            normalize_location("Washington, D.C., DC"),
            "Washington, D.C.,US"
        );
    }

    #[test]
    fn given_multi_part_international_when_normalized_then_returns_unchanged() {
        assert_eq!(
            normalize_location("City, Province, Canada"),
            "City, Province, Canada"
        );
    }

    #[test]
    fn given_all_us_states_when_normalized_then_all_convert_to_us() {
        let test_cases = vec![
            ("City, AL", "City,US"),
            ("City, AK", "City,US"),
            ("City, AZ", "City,US"),
            ("City, AR", "City,US"),
            ("City, CA", "City,US"),
            ("City, CO", "City,US"),
            ("City, CT", "City,US"),
            ("City, DE", "City,US"),
            ("City, FL", "City,US"),
            ("City, GA", "City,US"),
            ("City, HI", "City,US"),
            ("City, ID", "City,US"),
            ("City, IL", "City,US"),
            ("City, IN", "City,US"),
            ("City, IA", "City,US"),
            ("City, KS", "City,US"),
            ("City, KY", "City,US"),
            ("City, LA", "City,US"),
            ("City, ME", "City,US"),
            ("City, MD", "City,US"),
            ("City, MA", "City,US"),
            ("City, MI", "City,US"),
            ("City, MN", "City,US"),
            ("City, MS", "City,US"),
            ("City, MO", "City,US"),
            ("City, MT", "City,US"),
            ("City, NE", "City,US"),
            ("City, NV", "City,US"),
            ("City, NH", "City,US"),
            ("City, NJ", "City,US"),
            ("City, NM", "City,US"),
            ("City, NY", "City,US"),
            ("City, NC", "City,US"),
            ("City, ND", "City,US"),
            ("City, OH", "City,US"),
            ("City, OK", "City,US"),
            ("City, OR", "City,US"),
            ("City, PA", "City,US"),
            ("City, RI", "City,US"),
            ("City, SC", "City,US"),
            ("City, SD", "City,US"),
            ("City, TN", "City,US"),
            ("City, TX", "City,US"),
            ("City, UT", "City,US"),
            ("City, VT", "City,US"),
            ("City, VA", "City,US"),
            ("City, WA", "City,US"),
            ("City, WV", "City,US"),
            ("City, WI", "City,US"),
            ("City, WY", "City,US"),
            ("City, DC", "City,US"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                normalize_location(input),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }

    // ============================================================
    // WeatherClient Tests
    // ============================================================

    #[test]
    fn test_weather_client_creation() {
        let client: WeatherClient = "test_api_key".parse().unwrap();
        assert_eq!(client.api_key.as_str(), "test_api_key");
    }

    #[test]
    fn test_weather_client_with_api_key() {
        let api_key = ApiKey::from_trusted("my-weather-key".to_string());
        let client = WeatherClient::new(api_key).unwrap();
        assert_eq!(client.api_key.as_str(), "my-weather-key");
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
        assert_eq!(response.wind.unwrap().speed, 10.5);
        assert_eq!(response.name, "New York");
    }

    #[test]
    fn test_weather_api_response_without_wind() {
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
            "name": "New York"
        }"#;

        let response: WeatherApiResponse =
            serde_json::from_str(json).expect("Should deserialize without wind");
        assert!(response.wind.is_none());
    }

    // ============================================================
    // NWS Alert Deserialization Tests
    // ============================================================

    #[test]
    fn test_nws_alerts_response_deserialization() {
        let json = r#"{
            "features": [
                {
                    "properties": {
                        "event": "Winter Storm Warning",
                        "headline": "Winter Storm Warning issued",
                        "description": "Heavy snow expected",
                        "severity": "Severe"
                    }
                }
            ]
        }"#;

        let response: NwsAlertsResponse =
            serde_json::from_str(json).expect("Should deserialize NWS alerts");
        assert_eq!(response.features.len(), 1);
        assert_eq!(
            response.features[0].properties.event,
            Some("Winter Storm Warning".to_string())
        );
        assert_eq!(
            response.features[0].properties.severity,
            Some("Severe".to_string())
        );
    }

    #[test]
    fn test_nws_alerts_empty_response() {
        let json = r#"{"features": []}"#;

        let response: NwsAlertsResponse =
            serde_json::from_str(json).expect("Should deserialize empty NWS alerts");
        assert!(response.features.is_empty());
    }

    #[test]
    fn test_nws_alerts_with_null_fields() {
        let json = r#"{
            "features": [
                {
                    "properties": {
                        "event": null,
                        "headline": "Some headline",
                        "description": null,
                        "severity": null
                    }
                }
            ]
        }"#;

        let response: NwsAlertsResponse =
            serde_json::from_str(json).expect("Should deserialize NWS alerts with nulls");
        assert!(response.features[0].properties.event.is_none());
        assert_eq!(
            response.features[0].properties.headline,
            Some("Some headline".to_string())
        );
    }

    #[test]
    fn test_geocoding_response_deserialization() {
        let json = r#"[{"lat": 40.7128, "lon": -74.0060}]"#;

        let results: Vec<GeocodingResponse> =
            serde_json::from_str(json).expect("Should deserialize geocoding response");
        assert_eq!(results.len(), 1);
        assert!((results[0].lat - 40.7128).abs() < 0.001);
        assert!((results[0].lon - (-74.0060)).abs() < 0.001);
    }
}
