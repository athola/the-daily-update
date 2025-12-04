//! Application configuration

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub news_api_key: Option<String>,
    pub weather_api_key: Option<String>,
    pub tiingo_api_key: Option<String>,
}

fn default_location() -> String {
    "New York, NY".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UiConfig::default(),
            apis: ApiConfig::default(),
        }
    }
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

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            news_api_key: None,
            weather_api_key: None,
            tiingo_api_key: None,
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
            let mut config: Config = toml::from_str(&content)
                .with_context(|| "Failed to parse config file")?;

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

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.default_location, "New York, NY");
        assert!(!config.general.vim_mode);
        assert_eq!(config.ui.theme, "dark");
    }

    #[test]
    fn test_missing_api_keys() {
        let config = Config::default();
        let missing = config.missing_api_keys();
        assert_eq!(missing.len(), 3);
        assert!(missing.contains(&"NEWS_API_KEY"));
    }
}
