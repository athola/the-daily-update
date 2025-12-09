//! Application configuration

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub apis: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_location")]
    pub default_location: String,
    #[serde(default)]
    pub vim_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiConfig {
    pub news_api_key: Option<String>,
    pub weather_api_key: Option<String>,
    pub tiingo_api_key: Option<String>,
}

fn default_location() -> String {
    "New York,US".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_location: default_location(),
            vim_mode: false,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "dailyupdate", "daily-update")
            .context("Could not determine config directory")?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        Ok(config_dir.join("config.toml"))
    }

    /// Get the data directory path
    pub fn data_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "dailyupdate", "daily-update")
            .context("Could not determine data directory")?;

        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)?;

        Ok(data_dir.to_path_buf())
    }

    /// Get the database path
    pub fn database_path() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("data.db"))
    }

    /// Load config from file, falling back to defaults
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from {:?}", path))?;
            let mut config: Config =
                toml::from_str(&content).with_context(|| "Failed to parse config file")?;

            // Override with environment variables
            config.load_env_vars();
            Ok(config)
        } else {
            let mut config = Config::default();
            config.load_env_vars();
            Ok(config)
        }
    }

    /// Load API keys from environment variables
    fn load_env_vars(&mut self) {
        if let Ok(key) = std::env::var("NEWS_API_KEY") {
            self.apis.news_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("WEATHER_API_KEY") {
            self.apis.weather_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("TIINGO_API_KEY") {
            self.apis.tiingo_api_key = Some(key);
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Check if all required API keys are present
    pub fn has_api_keys(&self) -> bool {
        self.apis.news_api_key.is_some()
            && self.apis.weather_api_key.is_some()
            && self.apis.tiingo_api_key.is_some()
    }

    /// Get list of missing API keys
    pub fn missing_api_keys(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.apis.news_api_key.is_none() {
            missing.push("NEWS_API_KEY");
        }
        if self.apis.weather_api_key.is_none() {
            missing.push("WEATHER_API_KEY");
        }
        if self.apis.tiingo_api_key.is_none() {
            missing.push("TIINGO_API_KEY");
        }
        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Default Value Tests
    // ============================================================

    #[test]
    fn given_default_config_then_has_expected_general_settings() {
        let config = Config::default();

        assert_eq!(config.general.default_location, "New York,US");
        assert!(!config.general.vim_mode);
    }

    #[test]
    fn given_default_config_then_has_expected_ui_settings() {
        let config = Config::default();

        assert_eq!(config.ui.theme, "dark");
    }

    #[test]
    fn given_default_config_then_has_no_api_keys() {
        let config = Config::default();

        assert!(config.apis.news_api_key.is_none());
        assert!(config.apis.weather_api_key.is_none());
        assert!(config.apis.tiingo_api_key.is_none());
    }

    #[test]
    fn given_default_general_config_then_has_expected_values() {
        let general = GeneralConfig::default();

        assert_eq!(general.default_location, "New York,US");
        assert!(!general.vim_mode);
    }

    #[test]
    fn given_default_ui_config_then_has_dark_theme() {
        let ui = UiConfig::default();

        assert_eq!(ui.theme, "dark");
    }

    #[test]
    fn given_default_api_config_then_all_keys_are_none() {
        let api = ApiConfig::default();

        assert!(api.news_api_key.is_none());
        assert!(api.weather_api_key.is_none());
        assert!(api.tiingo_api_key.is_none());
    }

    // ============================================================
    // has_api_keys Tests
    // ============================================================

    #[test]
    fn given_no_api_keys_when_has_api_keys_then_returns_false() {
        let config = Config::default();

        assert!(!config.has_api_keys());
    }

    #[test]
    fn given_partial_api_keys_when_has_api_keys_then_returns_false() {
        let mut config = Config::default();
        config.apis.news_api_key = Some("key1".to_string());

        assert!(
            !config.has_api_keys(),
            "Should return false with only 1 key"
        );

        config.apis.weather_api_key = Some("key2".to_string());
        assert!(
            !config.has_api_keys(),
            "Should return false with only 2 keys"
        );
    }

    #[test]
    fn given_all_api_keys_when_has_api_keys_then_returns_true() {
        let mut config = Config::default();
        config.apis.news_api_key = Some("news-key".to_string());
        config.apis.weather_api_key = Some("weather-key".to_string());
        config.apis.tiingo_api_key = Some("tiingo-key".to_string());

        assert!(config.has_api_keys());
    }

    #[test]
    fn given_empty_string_api_keys_when_has_api_keys_then_returns_true() {
        // Note: Empty strings are still considered "present" - may want to change this behavior
        let mut config = Config::default();
        config.apis.news_api_key = Some(String::new());
        config.apis.weather_api_key = Some(String::new());
        config.apis.tiingo_api_key = Some(String::new());

        assert!(config.has_api_keys(), "Empty strings count as present");
    }

    // ============================================================
    // missing_api_keys Tests
    // ============================================================

    #[test]
    fn given_no_api_keys_when_missing_api_keys_then_returns_all_three() {
        let config = Config::default();
        let missing = config.missing_api_keys();

        assert_eq!(missing.len(), 3);
        assert!(missing.contains(&"NEWS_API_KEY"));
        assert!(missing.contains(&"WEATHER_API_KEY"));
        assert!(missing.contains(&"TIINGO_API_KEY"));
    }

    #[test]
    fn given_one_api_key_when_missing_api_keys_then_returns_two() {
        let mut config = Config::default();
        config.apis.news_api_key = Some("key".to_string());

        let missing = config.missing_api_keys();

        assert_eq!(missing.len(), 2);
        assert!(!missing.contains(&"NEWS_API_KEY"));
        assert!(missing.contains(&"WEATHER_API_KEY"));
        assert!(missing.contains(&"TIINGO_API_KEY"));
    }

    #[test]
    fn given_two_api_keys_when_missing_api_keys_then_returns_one() {
        let mut config = Config::default();
        config.apis.news_api_key = Some("key1".to_string());
        config.apis.weather_api_key = Some("key2".to_string());

        let missing = config.missing_api_keys();

        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&"TIINGO_API_KEY"));
    }

    #[test]
    fn given_all_api_keys_when_missing_api_keys_then_returns_empty() {
        let mut config = Config::default();
        config.apis.news_api_key = Some("key1".to_string());
        config.apis.weather_api_key = Some("key2".to_string());
        config.apis.tiingo_api_key = Some("key3".to_string());

        let missing = config.missing_api_keys();

        assert!(missing.is_empty());
    }

    // ============================================================
    // TOML Serialization Tests
    // ============================================================

    #[test]
    fn given_default_config_when_serialized_then_valid_toml() {
        let config = Config::default();

        let toml_str = toml::to_string(&config).expect("Serialization should succeed");

        assert!(toml_str.contains("default_location"));
        assert!(toml_str.contains("New York,US"));
        assert!(toml_str.contains("vim_mode"));
        assert!(toml_str.contains("theme"));
    }

    #[test]
    fn given_config_with_api_keys_when_serialized_then_keys_included() {
        let mut config = Config::default();
        config.apis.news_api_key = Some("my-news-key".to_string());
        config.apis.weather_api_key = Some("my-weather-key".to_string());

        let toml_str = toml::to_string(&config).expect("Serialization should succeed");

        assert!(toml_str.contains("my-news-key"));
        assert!(toml_str.contains("my-weather-key"));
    }

    #[test]
    fn given_toml_string_when_deserialized_then_config_is_correct() {
        let toml_str = r#"
            [general]
            default_location = "San Francisco, CA"
            vim_mode = true

            [ui]
            theme = "light"

            [apis]
            news_api_key = "test-news-key"
        "#;

        let config: Config = toml::from_str(toml_str).expect("Deserialization should succeed");

        assert_eq!(config.general.default_location, "San Francisco, CA");
        assert!(config.general.vim_mode);
        assert_eq!(config.ui.theme, "light");
        assert_eq!(config.apis.news_api_key, Some("test-news-key".to_string()));
        assert!(config.apis.weather_api_key.is_none());
    }

    #[test]
    fn given_partial_toml_when_deserialized_then_uses_defaults() {
        let toml_str = r#"
            [general]
            vim_mode = true
        "#;

        let config: Config = toml::from_str(toml_str).expect("Deserialization should succeed");

        // Specified value
        assert!(config.general.vim_mode);

        // Default values for unspecified fields
        assert_eq!(config.general.default_location, "New York,US");
        assert_eq!(config.ui.theme, "dark");
    }

    #[test]
    fn given_empty_toml_when_deserialized_then_uses_all_defaults() {
        let toml_str = "";

        let config: Config = toml::from_str(toml_str).expect("Deserialization should succeed");

        assert_eq!(config.general.default_location, "New York,US");
        assert!(!config.general.vim_mode);
        assert_eq!(config.ui.theme, "dark");
        assert!(config.apis.news_api_key.is_none());
    }

    #[test]
    fn given_config_when_serialized_and_deserialized_then_roundtrip_succeeds() {
        let mut original = Config::default();
        original.general.default_location = "Boston, MA".to_string();
        original.general.vim_mode = true;
        original.ui.theme = "solarized".to_string();
        original.apis.news_api_key = Some("roundtrip-key".to_string());

        let toml_str = toml::to_string(&original).expect("Serialization should succeed");
        let restored: Config = toml::from_str(&toml_str).expect("Deserialization should succeed");

        assert_eq!(
            restored.general.default_location,
            original.general.default_location
        );
        assert_eq!(restored.general.vim_mode, original.general.vim_mode);
        assert_eq!(restored.ui.theme, original.ui.theme);
        assert_eq!(restored.apis.news_api_key, original.apis.news_api_key);
    }

    // ============================================================
    // Clone Tests
    // ============================================================

    #[test]
    fn given_config_when_cloned_then_is_equal() {
        let mut config = Config::default();
        config.general.vim_mode = true;
        config.apis.news_api_key = Some("clone-test".to_string());

        let cloned = config.clone();

        assert_eq!(cloned.general.vim_mode, config.general.vim_mode);
        assert_eq!(
            cloned.general.default_location,
            config.general.default_location
        );
        assert_eq!(cloned.apis.news_api_key, config.apis.news_api_key);
    }

    // ============================================================
    // Path Tests (basic validation only - can't test actual paths)
    // ============================================================

    #[test]
    fn given_database_path_when_called_then_ends_with_data_db() {
        // Note: This test may fail in some environments where ProjectDirs can't determine paths
        if let Ok(path) = Config::database_path() {
            assert!(
                path.to_string_lossy().ends_with("data.db"),
                "Database path should end with data.db"
            );
        }
    }

    #[test]
    fn given_config_path_when_called_then_ends_with_config_toml() {
        // Note: This test may fail in some environments where ProjectDirs can't determine paths
        if let Ok(path) = Config::config_path() {
            assert!(
                path.to_string_lossy().ends_with("config.toml"),
                "Config path should end with config.toml"
            );
        }
    }
}
