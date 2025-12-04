# The Daily Update - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a terminal-based daily news aggregator with weather and stock data, featuring instant startup with cached data and background refresh.

**Architecture:** Layered Rust application with ratatui TUI, SQLite persistence via rusqlite, async API clients via reqwest/tokio, and TOML configuration. Data flows from APIs → background fetcher → SQLite → UI components.

**Tech Stack:** Rust, ratatui, tokio, rusqlite, reqwest, serde, toml, crossterm, chrono, directories

**Reference:** See `docs/plans/2024-12-04-daily-update-design.md` for full design details.

---

## Phase 1: Project Setup

### Task 1: Initialize Cargo Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Initialize the project**

Run:
```bash
cd /home/alext/the-daily-update
cargo init
```

Expected: Creates `Cargo.toml` and `src/main.rs`

**Step 2: Add dependencies to Cargo.toml**

Replace `Cargo.toml` with:

```toml
[package]
name = "daily-update"
version = "0.1.0"
edition = "2021"
description = "Terminal-based daily news aggregator with weather and stock data"
license = "MIT"

[dependencies]
# TUI
ratatui = "0.29"
crossterm = "0.28"

# Async runtime
tokio = { version = "1.41", features = ["full"] }

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
directories = "5.0"
thiserror = "2.0"
anyhow = "1.0"

[dev-dependencies]
tempfile = "3.14"
```

**Step 3: Create minimal main.rs**

Replace `src/main.rs` with:

```rust
fn main() {
    println!("The Daily Update - v0.1.0");
}
```

**Step 4: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds, dependencies download

**Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "chore: initialize Rust project with dependencies"
```

---

### Task 2: Create Project Directory Structure

**Files:**
- Create: `src/app.rs`
- Create: `src/ui/mod.rs`
- Create: `src/data/mod.rs`
- Create: `src/api/mod.rs`
- Create: `src/config/mod.rs`
- Create: `src/fetch/mod.rs`

**Step 1: Create directory structure**

Run:
```bash
mkdir -p src/ui src/data src/api src/config src/fetch
```

**Step 2: Create placeholder modules**

Create `src/app.rs`:
```rust
//! Application state and main event loop

pub struct App {
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        Self { running: true }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
```

Create `src/ui/mod.rs`:
```rust
//! UI components for the TUI

pub mod layout;
pub mod news_panel;
pub mod stocks_panel;
pub mod weather_widget;
pub mod stock_browser;
pub mod help_overlay;
```

Create `src/data/mod.rs`:
```rust
//! Data layer: database and models

pub mod db;
pub mod models;
pub mod cache;
```

Create `src/api/mod.rs`:
```rust
//! External API clients

pub mod news;
pub mod weather;
pub mod stocks;
```

Create `src/config/mod.rs`:
```rust
//! Configuration management

pub mod settings;
pub mod setup;
```

Create `src/fetch/mod.rs`:
```rust
//! Background data fetching

pub mod background;
```

**Step 3: Create placeholder submodules**

Create empty placeholder files:
```bash
touch src/ui/layout.rs src/ui/news_panel.rs src/ui/stocks_panel.rs
touch src/ui/weather_widget.rs src/ui/stock_browser.rs src/ui/help_overlay.rs
touch src/data/db.rs src/data/models.rs src/data/cache.rs
touch src/api/news.rs src/api/weather.rs src/api/stocks.rs
touch src/config/settings.rs src/config/setup.rs
touch src/fetch/background.rs
```

**Step 4: Update main.rs to declare modules**

Replace `src/main.rs` with:
```rust
mod app;
mod ui;
mod data;
mod api;
mod config;
mod fetch;

fn main() {
    println!("The Daily Update - v0.1.0");
}
```

**Step 5: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds (warnings about unused modules OK)

**Step 6: Commit**

```bash
git add src/
git commit -m "chore: create project directory structure with placeholder modules"
```

---

## Phase 2: Data Layer

### Task 3: Define Data Models

**Files:**
- Modify: `src/data/models.rs`
- Test: Manual compilation check

**Step 1: Write data model structs**

Replace `src/data/models.rs` with:
```rust
//! Data models for news, weather, stocks, and watchlist

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A news headline from NewsAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: Option<i64>,
    pub headline: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub published_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
}

/// Weather data from OpenWeatherMap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub id: Option<i64>,
    pub location: String,
    pub temperature: Option<f64>,
    pub condition: Option<String>,
    pub humidity: Option<i32>,
    pub wind_speed: Option<f64>,
    pub alert_title: Option<String>,
    pub alert_description: Option<String>,
    pub alert_severity: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

/// Stock/index data from Tiingo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    pub id: Option<i64>,
    pub symbol: String,
    pub name: Option<String>,
    pub price: Option<f64>,
    pub change_percent: Option<f64>,
    pub fetched_at: DateTime<Utc>,
}

/// A watchlist item (user's saved stocks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: Option<i64>,
    pub symbol: String,
    pub display_order: i32,
}

/// Available stock for the stock browser
#[derive(Debug, Clone)]
pub struct AvailableStock {
    pub symbol: String,
    pub name: String,
    pub in_watchlist: bool,
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/data/models.rs
git commit -m "feat(data): add data model structs for news, weather, stocks"
```

---

### Task 4: Implement Database Module

**Files:**
- Modify: `src/data/db.rs`

**Step 1: Write database initialization and schema**

Replace `src/data/db.rs` with:
```rust
//! SQLite database operations

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

use super::models::{NewsItem, StockData, WatchlistItem, WeatherData};

/// Database wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS news (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                headline TEXT NOT NULL,
                source TEXT,
                description TEXT,
                url TEXT,
                published_at TEXT NOT NULL,
                fetched_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS weather (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                location TEXT NOT NULL,
                temperature REAL,
                condition TEXT,
                humidity INTEGER,
                wind_speed REAL,
                alert_title TEXT,
                alert_description TEXT,
                alert_severity TEXT,
                fetched_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS stocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                name TEXT,
                price REAL,
                change_percent REAL,
                fetched_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS watchlist (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT UNIQUE NOT NULL,
                display_order INTEGER DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_news_published ON news(published_at);
            CREATE INDEX IF NOT EXISTS idx_weather_location ON weather(location);
            CREATE INDEX IF NOT EXISTS idx_stocks_symbol ON stocks(symbol);
            "#,
        )?;
        Ok(())
    }

    // === News Operations ===

    /// Insert a news item
    pub fn insert_news(&self, item: &NewsItem) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO news (headline, source, description, url, published_at, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.headline,
                item.source,
                item.description,
                item.url,
                item.published_at.to_rfc3339(),
                item.fetched_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get recent news items
    pub fn get_news(&self, limit: usize) -> Result<Vec<NewsItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, headline, source, description, url, published_at, fetched_at
             FROM news ORDER BY published_at DESC LIMIT ?1",
        )?;

        let items = stmt
            .query_map([limit], |row| {
                Ok(NewsItem {
                    id: Some(row.get(0)?),
                    headline: row.get(1)?,
                    source: row.get(2)?,
                    description: row.get(3)?,
                    url: row.get(4)?,
                    published_at: parse_datetime(row.get::<_, String>(5)?),
                    fetched_at: parse_datetime(row.get::<_, String>(6)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    /// Clear all news items
    pub fn clear_news(&self) -> Result<()> {
        self.conn.execute("DELETE FROM news", [])?;
        Ok(())
    }

    // === Weather Operations ===

    /// Insert or update weather for a location
    pub fn upsert_weather(&self, data: &WeatherData) -> Result<i64> {
        // Delete old weather for this location
        self.conn.execute(
            "DELETE FROM weather WHERE location = ?1",
            [&data.location],
        )?;

        self.conn.execute(
            "INSERT INTO weather (location, temperature, condition, humidity, wind_speed,
                                  alert_title, alert_description, alert_severity, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                data.location,
                data.temperature,
                data.condition,
                data.humidity,
                data.wind_speed,
                data.alert_title,
                data.alert_description,
                data.alert_severity,
                data.fetched_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get weather for a location
    pub fn get_weather(&self, location: &str) -> Result<Option<WeatherData>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, location, temperature, condition, humidity, wind_speed,
                    alert_title, alert_description, alert_severity, fetched_at
             FROM weather WHERE location = ?1 ORDER BY fetched_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query([location])?;
        if let Some(row) = rows.next()? {
            Ok(Some(WeatherData {
                id: Some(row.get(0)?),
                location: row.get(1)?,
                temperature: row.get(2)?,
                condition: row.get(3)?,
                humidity: row.get(4)?,
                wind_speed: row.get(5)?,
                alert_title: row.get(6)?,
                alert_description: row.get(7)?,
                alert_severity: row.get(8)?,
                fetched_at: parse_datetime(row.get::<_, String>(9)?),
            }))
        } else {
            Ok(None)
        }
    }

    // === Stock Operations ===

    /// Insert or update a stock
    pub fn upsert_stock(&self, data: &StockData) -> Result<i64> {
        self.conn.execute(
            "DELETE FROM stocks WHERE symbol = ?1",
            [&data.symbol],
        )?;

        self.conn.execute(
            "INSERT INTO stocks (symbol, name, price, change_percent, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                data.symbol,
                data.name,
                data.price,
                data.change_percent,
                data.fetched_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get stock by symbol
    pub fn get_stock(&self, symbol: &str) -> Result<Option<StockData>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, name, price, change_percent, fetched_at
             FROM stocks WHERE symbol = ?1 ORDER BY fetched_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query([symbol])?;
        if let Some(row) = rows.next()? {
            Ok(Some(StockData {
                id: Some(row.get(0)?),
                symbol: row.get(1)?,
                name: row.get(2)?,
                price: row.get(3)?,
                change_percent: row.get(4)?,
                fetched_at: parse_datetime(row.get::<_, String>(5)?),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all stocks in watchlist
    pub fn get_watchlist_stocks(&self) -> Result<Vec<StockData>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.symbol, s.name, s.price, s.change_percent, s.fetched_at
             FROM stocks s
             INNER JOIN watchlist w ON s.symbol = w.symbol
             ORDER BY w.display_order",
        )?;

        let items = stmt
            .query_map([], |row| {
                Ok(StockData {
                    id: Some(row.get(0)?),
                    symbol: row.get(1)?,
                    name: row.get(2)?,
                    price: row.get(3)?,
                    change_percent: row.get(4)?,
                    fetched_at: parse_datetime(row.get::<_, String>(5)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    // === Watchlist Operations ===

    /// Get all watchlist items
    pub fn get_watchlist(&self) -> Result<Vec<WatchlistItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, symbol, display_order FROM watchlist ORDER BY display_order",
        )?;

        let items = stmt
            .query_map([], |row| {
                Ok(WatchlistItem {
                    id: Some(row.get(0)?),
                    symbol: row.get(1)?,
                    display_order: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    /// Add symbol to watchlist
    pub fn add_to_watchlist(&self, symbol: &str) -> Result<()> {
        let max_order: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(display_order), -1) FROM watchlist",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        self.conn.execute(
            "INSERT OR IGNORE INTO watchlist (symbol, display_order) VALUES (?1, ?2)",
            params![symbol, max_order + 1],
        )?;
        Ok(())
    }

    /// Remove symbol from watchlist
    pub fn remove_from_watchlist(&self, symbol: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM watchlist WHERE symbol = ?1",
            [symbol],
        )?;
        Ok(())
    }

    /// Check if symbol is in watchlist
    pub fn is_in_watchlist(&self, symbol: &str) -> Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM watchlist WHERE symbol = ?1",
            [symbol],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

/// Parse RFC3339 datetime string
fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().expect("Failed to create in-memory database");
        let news = db.get_news(10).expect("Failed to get news");
        assert!(news.is_empty());
    }

    #[test]
    fn test_news_operations() {
        let db = Database::in_memory().unwrap();

        let item = NewsItem {
            id: None,
            headline: "Test headline".to_string(),
            source: Some("Test Source".to_string()),
            description: Some("Test description".to_string()),
            url: Some("https://example.com".to_string()),
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        };

        db.insert_news(&item).unwrap();
        let news = db.get_news(10).unwrap();
        assert_eq!(news.len(), 1);
        assert_eq!(news[0].headline, "Test headline");
    }

    #[test]
    fn test_watchlist_operations() {
        let db = Database::in_memory().unwrap();

        db.add_to_watchlist("SPY").unwrap();
        db.add_to_watchlist("QQQ").unwrap();

        assert!(db.is_in_watchlist("SPY").unwrap());
        assert!(db.is_in_watchlist("QQQ").unwrap());
        assert!(!db.is_in_watchlist("AAPL").unwrap());

        let watchlist = db.get_watchlist().unwrap();
        assert_eq!(watchlist.len(), 2);

        db.remove_from_watchlist("SPY").unwrap();
        assert!(!db.is_in_watchlist("SPY").unwrap());
    }
}
```

**Step 2: Run tests**

Run:
```bash
cargo test data::db
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add src/data/db.rs
git commit -m "feat(data): implement SQLite database with CRUD operations"
```

---

### Task 5: Implement Cache Management

**Files:**
- Modify: `src/data/cache.rs`

**Step 1: Write cache manager**

Replace `src/data/cache.rs` with:
```rust
//! Cache management for data freshness

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

use super::db::Database;

/// Cache staleness thresholds
pub struct CacheConfig {
    /// News considered stale after this duration
    pub news_max_age: Duration,
    /// Weather considered stale after this duration
    pub weather_max_age: Duration,
    /// Stocks considered stale after this duration
    pub stocks_max_age: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            news_max_age: Duration::hours(1),
            weather_max_age: Duration::minutes(30),
            stocks_max_age: Duration::minutes(15),
        }
    }
}

/// Check if a timestamp is stale based on max age
pub fn is_stale(fetched_at: DateTime<Utc>, max_age: Duration) -> bool {
    Utc::now() - fetched_at > max_age
}

/// Format relative time for display
pub fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now - dt;

    if diff < Duration::minutes(1) {
        "just now".to_string()
    } else if diff < Duration::hours(1) {
        let mins = diff.num_minutes();
        format!("{}m ago", mins)
    } else if diff < Duration::hours(24) {
        let hours = diff.num_hours();
        format!("{}h ago", hours)
    } else {
        let days = diff.num_days();
        format!("{}d ago", days)
    }
}

/// Prune old data from database (keep last 7 days)
pub fn prune_old_data(db: &Database) -> Result<()> {
    // Note: This would need additional methods on Database
    // For now, we'll just clear news older than 7 days via direct SQL
    // This is a placeholder for future implementation
    let _ = db;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stale() {
        let now = Utc::now();
        let old = now - Duration::hours(2);
        let recent = now - Duration::minutes(5);

        assert!(is_stale(old, Duration::hours(1)));
        assert!(!is_stale(recent, Duration::hours(1)));
    }

    #[test]
    fn test_format_relative_time() {
        let now = Utc::now();

        assert_eq!(format_relative_time(now), "just now");
        assert_eq!(format_relative_time(now - Duration::minutes(5)), "5m ago");
        assert_eq!(format_relative_time(now - Duration::hours(2)), "2h ago");
        assert_eq!(format_relative_time(now - Duration::days(3)), "3d ago");
    }
}
```

**Step 2: Run tests**

Run:
```bash
cargo test data::cache
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add src/data/cache.rs
git commit -m "feat(data): add cache management utilities"
```

---

## Phase 3: Configuration

### Task 6: Implement Configuration Settings

**Files:**
- Modify: `src/config/settings.rs`

**Step 1: Write config structs**

Replace `src/config/settings.rs` with:
```rust
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
```

**Step 2: Run tests**

Run:
```bash
cargo test config::settings
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add src/config/settings.rs
git commit -m "feat(config): implement configuration loading with env var support"
```

---

### Task 7: Implement First-Run Setup

**Files:**
- Modify: `src/config/setup.rs`

**Step 1: Write setup wizard**

Replace `src/config/setup.rs` with:
```rust
//! First-run setup wizard

use anyhow::Result;
use std::io::{self, Write};

use super::settings::Config;

/// Check if first-run setup is needed
pub fn needs_setup(config: &Config) -> bool {
    !config.has_api_keys()
}

/// Display security guidance
pub fn display_security_guidance() {
    println!();
    println!("╭─────────────────── API Key Security ───────────────────╮");
    println!("│                                                        │");
    println!("│  Your API keys grant access to paid services.          │");
    println!("│  Keep them secure:                                     │");
    println!("│                                                        │");
    println!("│  ✓ Use environment variables (recommended)             │");
    println!("│    export NEWS_API_KEY=\"your-key\"                      │");
    println!("│                                                        │");
    println!("│  ✓ Or use a .env file (add to .gitignore)             │");
    println!("│                                                        │");
    println!("│  ✗ Avoid committing keys to version control            │");
    println!("│  ✗ Don't share config files containing keys            │");
    println!("│                                                        │");
    println!("╰────────────────────────────────────────────────────────╯");
    println!();
}

/// Run interactive setup wizard
pub fn run_setup_wizard(config: &mut Config) -> Result<()> {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("           Welcome to The Daily Update!");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Check for existing env vars
    let missing = config.missing_api_keys();

    if missing.is_empty() {
        println!("✓ All API keys found in environment variables!");
    } else {
        println!("Missing API keys: {}", missing.join(", "));
        println!();
        println!("You can set them as environment variables:");
        for key in &missing {
            println!("  export {}=\"your-key-here\"", key);
        }
        println!();

        // Ask if user wants to enter them now
        print!("Would you like to enter them now? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            prompt_for_api_keys(config, &missing)?;
        }
    }

    // Prompt for default location
    println!();
    print!("Enter your default location for weather [{}]: ", config.general.default_location);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if !input.is_empty() {
        config.general.default_location = input.to_string();
    }

    // Ask about vim mode
    print!("Enable vim-style keybindings? (y/n) [n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    config.general.vim_mode = input.trim().to_lowercase() == "y";

    // Save config
    config.save()?;

    display_security_guidance();

    println!("Configuration saved to: {:?}", Config::config_path()?);
    println!();

    Ok(())
}

fn prompt_for_api_keys(config: &mut Config, missing: &[&str]) -> Result<()> {
    println!();
    println!("Note: Keys entered here will be saved to config file.");
    println!("For better security, use environment variables instead.");
    println!();

    for key in missing {
        print!("Enter {}: ", key);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let value = input.trim().to_string();

        if !value.is_empty() {
            match *key {
                "NEWS_API_KEY" => config.apis.news_api_key = Some(value),
                "WEATHER_API_KEY" => config.apis.weather_api_key = Some(value),
                "TIINGO_API_KEY" => config.apis.tiingo_api_key = Some(value),
                _ => {}
            }
        }
    }

    Ok(())
}

/// Display startup banner
pub fn display_banner() {
    println!();
    println!("╔═══════════════════════════════════════╗");
    println!("║       The Daily Update v0.1.0         ║");
    println!("╚═══════════════════════════════════════╝");
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/config/setup.rs
git commit -m "feat(config): implement first-run setup wizard"
```

---

## Phase 4: API Clients

### Task 8: Implement NewsAPI Client

**Files:**
- Modify: `src/api/news.rs`

**Step 1: Write NewsAPI client**

Replace `src/api/news.rs` with:
```rust
//! NewsAPI client

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::data::models::NewsItem;

const NEWS_API_BASE: &str = "https://newsapi.org/v2";

/// NewsAPI response structures
#[derive(Debug, Deserialize)]
struct NewsApiResponse {
    status: String,
    #[serde(default)]
    articles: Vec<NewsApiArticle>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsApiArticle {
    title: Option<String>,
    source: Option<NewsApiSource>,
    description: Option<String>,
    url: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NewsApiSource {
    name: Option<String>,
}

/// NewsAPI client
pub struct NewsClient {
    api_key: String,
    client: reqwest::Client,
}

impl NewsClient {
    /// Create a new NewsAPI client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch top headlines
    pub async fn fetch_top_headlines(&self, count: usize) -> Result<Vec<NewsItem>> {
        let url = format!(
            "{}/top-headlines?country=us&pageSize={}",
            NEWS_API_BASE, count
        );

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await
            .context("Failed to send request to NewsAPI")?;

        let status = response.status();
        let body = response
            .json::<NewsApiResponse>()
            .await
            .context("Failed to parse NewsAPI response")?;

        if body.status != "ok" {
            anyhow::bail!(
                "NewsAPI error ({}): {}",
                status,
                body.message.unwrap_or_default()
            );
        }

        let now = Utc::now();
        let items: Vec<NewsItem> = body
            .articles
            .into_iter()
            .filter_map(|article| {
                let headline = article.title?;
                if headline.is_empty() || headline == "[Removed]" {
                    return None;
                }

                let published_at = article
                    .published_at
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(now);

                Some(NewsItem {
                    id: None,
                    headline,
                    source: article.source.and_then(|s| s.name),
                    description: article.description,
                    url: article.url,
                    published_at,
                    fetched_at: now,
                })
            })
            .collect();

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = NewsClient::new("test-key".to_string());
        assert_eq!(client.api_key, "test-key");
    }
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/api/news.rs
git commit -m "feat(api): implement NewsAPI client"
```

---

### Task 9: Implement OpenWeatherMap Client

**Files:**
- Modify: `src/api/weather.rs`

**Step 1: Write weather client**

Replace `src/api/weather.rs` with:
```rust
//! OpenWeatherMap API client

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;

use crate::data::models::WeatherData;

const WEATHER_API_BASE: &str = "https://api.openweathermap.org/data/2.5";

/// OpenWeatherMap response structures
#[derive(Debug, Deserialize)]
struct WeatherApiResponse {
    name: Option<String>,
    main: Option<WeatherMain>,
    weather: Option<Vec<WeatherCondition>>,
    wind: Option<WeatherWind>,
}

#[derive(Debug, Deserialize)]
struct WeatherMain {
    temp: Option<f64>,
    humidity: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct WeatherCondition {
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WeatherWind {
    speed: Option<f64>,
}

/// OpenWeatherMap alerts response (from One Call API)
#[derive(Debug, Deserialize)]
struct AlertsResponse {
    alerts: Option<Vec<WeatherAlert>>,
}

#[derive(Debug, Deserialize)]
struct WeatherAlert {
    event: Option<String>,
    description: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// Weather API client
pub struct WeatherClient {
    api_key: String,
    client: reqwest::Client,
}

impl WeatherClient {
    /// Create a new weather client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch weather for a location
    pub async fn fetch_weather(&self, location: &str) -> Result<WeatherData> {
        let url = format!(
            "{}/weather?q={}&units=imperial&appid={}",
            WEATHER_API_BASE, location, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to OpenWeatherMap")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "OpenWeatherMap error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        let body: WeatherApiResponse = response
            .json()
            .await
            .context("Failed to parse weather response")?;

        let condition = body
            .weather
            .and_then(|w| w.into_iter().next())
            .and_then(|c| c.description);

        Ok(WeatherData {
            id: None,
            location: body.name.unwrap_or_else(|| location.to_string()),
            temperature: body.main.as_ref().and_then(|m| m.temp),
            condition,
            humidity: body.main.as_ref().and_then(|m| m.humidity),
            wind_speed: body.wind.and_then(|w| w.speed),
            alert_title: None,
            alert_description: None,
            alert_severity: None,
            fetched_at: Utc::now(),
        })
    }

    /// Fetch weather with alerts (requires lat/lon and One Call API subscription)
    /// For now, this is a placeholder - alerts require additional API setup
    pub async fn fetch_weather_with_alerts(
        &self,
        location: &str,
    ) -> Result<WeatherData> {
        // For MVP, just fetch basic weather
        // Alerts would require geocoding to get lat/lon, then One Call API
        self.fetch_weather(location).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = WeatherClient::new("test-key".to_string());
        assert_eq!(client.api_key, "test-key");
    }
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/api/weather.rs
git commit -m "feat(api): implement OpenWeatherMap client"
```

---

### Task 10: Implement Tiingo Client

**Files:**
- Modify: `src/api/stocks.rs`

**Step 1: Write Tiingo client**

Replace `src/api/stocks.rs` with:
```rust
//! Tiingo API client for stock data

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;

use crate::data::models::StockData;

const TIINGO_API_BASE: &str = "https://api.tiingo.com/iex";

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
    api_key: String,
    client: reqwest::Client,
}

impl StocksClient {
    /// Create a new Tiingo client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch stock data for multiple symbols
    pub async fn fetch_stocks(&self, symbols: &[String]) -> Result<Vec<StockData>> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let tickers = symbols.join(",");
        let url = format!("{}?tickers={}&token={}", TIINGO_API_BASE, tickers, self.api_key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to Tiingo")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Tiingo error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        let body: Vec<TiingoResponse> = response
            .json()
            .await
            .context("Failed to parse Tiingo response")?;

        let now = Utc::now();
        let stocks: Vec<StockData> = body
            .into_iter()
            .map(|item| {
                let change_percent = match (item.last_price, item.prev_close) {
                    (Some(last), Some(prev)) if prev > 0.0 => {
                        Some(((last - prev) / prev) * 100.0)
                    }
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
    pub async fn fetch_stock(&self, symbol: &str) -> Result<Option<StockData>> {
        let stocks = self.fetch_stocks(&[symbol.to_string()]).await?;
        Ok(stocks.into_iter().next())
    }

    /// Get default watchlist symbols
    pub fn default_watchlist() -> Vec<String> {
        vec![
            "SPY".to_string(),
            "QQQ".to_string(),
            "DIA".to_string(),
        ]
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
    }
}
```

**Step 2: Run tests**

Run:
```bash
cargo test api::stocks
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add src/api/stocks.rs
git commit -m "feat(api): implement Tiingo stocks client"
```

---

## Phase 5: Background Fetching

### Task 11: Implement Background Fetcher

**Files:**
- Modify: `src/fetch/background.rs`

**Step 1: Write background fetch orchestrator**

Replace `src/fetch/background.rs` with:
```rust
//! Background data fetching

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::api::news::NewsClient;
use crate::api::stocks::StocksClient;
use crate::api::weather::WeatherClient;
use crate::config::settings::Config;
use crate::data::db::Database;
use crate::data::models::{NewsItem, StockData, WeatherData};

/// Messages sent from background fetcher to UI
#[derive(Debug, Clone)]
pub enum FetchUpdate {
    /// News data has been updated
    NewsUpdated(Vec<NewsItem>),
    /// Weather data has been updated
    WeatherUpdated(WeatherData),
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
            .map(|key| NewsClient::new(key.clone()));

        let weather_client = config
            .apis
            .weather_api_key
            .as_ref()
            .map(|key| WeatherClient::new(key.clone()));

        let stocks_client = config
            .apis
            .tiingo_api_key
            .as_ref()
            .map(|key| StocksClient::new(key.clone()));

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
        tx: mpsc::Sender<FetchUpdate>,
    ) {
        // Fetch news
        if let Some(client) = &self.news_client {
            match client.fetch_top_headlines(10).await {
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

        // Fetch weather
        if let Some(client) = &self.weather_client {
            let location = &self.config.general.default_location;
            match client.fetch_weather(location).await {
                Ok(data) => {
                    // Store in database
                    if let Ok(db) = db.lock() {
                        let _ = db.upsert_weather(&data);
                    }
                    let _ = tx.send(FetchUpdate::WeatherUpdated(data)).await;
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
}

/// Check if we appear to be offline
pub fn check_network() -> bool {
    // Simple check - try to resolve a known host
    // In production, could use a more sophisticated check
    std::net::ToSocketAddrs::to_socket_addrs(&("api.tiingo.com", 443)).is_ok()
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/fetch/background.rs
git commit -m "feat(fetch): implement background data fetcher with channel updates"
```

---

## Phase 6: UI Components

### Task 12: Implement Application State

**Files:**
- Modify: `src/app.rs`

**Step 1: Write full application state**

Replace `src/app.rs` with:
```rust
//! Application state and main event loop

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::config::settings::Config;
use crate::data::db::Database;
use crate::data::models::{AvailableStock, NewsItem, StockData, WeatherData};
use crate::fetch::background::{BackgroundFetcher, FetchUpdate};
use crate::api::stocks::StocksClient;

/// Application state
pub struct App {
    /// Whether the app should keep running
    pub running: bool,
    /// Current configuration
    pub config: Config,
    /// Database connection
    pub db: Arc<Mutex<Database>>,
    /// Current news items
    pub news: Vec<NewsItem>,
    /// Current weather data
    pub weather: Option<WeatherData>,
    /// Current stock data (from watchlist)
    pub stocks: Vec<StockData>,
    /// User's watchlist symbols
    pub watchlist: Vec<String>,
    /// Selected news index
    pub news_selected: usize,
    /// Currently active panel
    pub active_panel: Panel,
    /// Whether weather widget is expanded
    pub weather_expanded: bool,
    /// Whether stock browser modal is open
    pub stock_browser_open: bool,
    /// Stock browser state
    pub stock_browser: StockBrowserState,
    /// Whether data is currently being fetched
    pub fetching: bool,
    /// Whether we're offline
    pub offline: bool,
    /// Current error messages per panel
    pub errors: PanelErrors,
    /// Channel receiver for fetch updates
    pub fetch_rx: Option<mpsc::Receiver<FetchUpdate>>,
}

/// Which panel is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    News,
    Stocks,
}

/// Stock browser modal state
#[derive(Debug, Clone)]
pub struct StockBrowserState {
    pub available: Vec<AvailableStock>,
    pub selected: usize,
    pub filter: String,
}

/// Error messages per panel
#[derive(Debug, Clone, Default)]
pub struct PanelErrors {
    pub news: Option<String>,
    pub weather: Option<String>,
    pub stocks: Option<String>,
}

impl App {
    /// Create a new application with the given config and database
    pub fn new(config: Config, db: Database) -> Self {
        let db = Arc::new(Mutex::new(db));

        // Load initial data from database
        let (news, weather, stocks, watchlist) = {
            let db = db.lock().unwrap();
            let news = db.get_news(10).unwrap_or_default();
            let weather = db.get_weather(&config.general.default_location).ok().flatten();
            let watchlist = db
                .get_watchlist()
                .unwrap_or_default()
                .into_iter()
                .map(|w| w.symbol)
                .collect::<Vec<_>>();
            let stocks = db.get_watchlist_stocks().unwrap_or_default();
            (news, weather, stocks, watchlist)
        };

        // Initialize watchlist with defaults if empty
        let watchlist = if watchlist.is_empty() {
            let defaults = StocksClient::default_watchlist();
            if let Ok(db) = db.lock() {
                for symbol in &defaults {
                    let _ = db.add_to_watchlist(symbol);
                }
            }
            defaults
        } else {
            watchlist
        };

        Self {
            running: true,
            config,
            db,
            news,
            weather,
            stocks,
            watchlist,
            news_selected: 0,
            active_panel: Panel::News,
            weather_expanded: false,
            stock_browser_open: false,
            stock_browser: StockBrowserState::new(),
            fetching: false,
            offline: false,
            errors: PanelErrors::default(),
            fetch_rx: None,
        }
    }

    /// Start background fetch
    pub fn start_fetch(&mut self) {
        self.fetching = true;
        self.errors = PanelErrors::default();

        let (tx, rx) = mpsc::channel(32);
        self.fetch_rx = Some(rx);

        let fetcher = BackgroundFetcher::new(self.config.clone());
        let db = Arc::clone(&self.db);
        let watchlist = self.watchlist.clone();

        tokio::spawn(async move {
            fetcher.fetch_all(db, watchlist, tx).await;
        });
    }

    /// Process any pending fetch updates
    pub fn process_fetch_updates(&mut self) {
        if let Some(rx) = &mut self.fetch_rx {
            while let Ok(update) = rx.try_recv() {
                match update {
                    FetchUpdate::NewsUpdated(items) => {
                        self.news = items;
                        self.errors.news = None;
                    }
                    FetchUpdate::WeatherUpdated(data) => {
                        self.weather = Some(data);
                        self.errors.weather = None;
                    }
                    FetchUpdate::StocksUpdated(stocks) => {
                        self.stocks = stocks;
                        self.errors.stocks = None;
                    }
                    FetchUpdate::Error(err) => {
                        match err.source.as_str() {
                            "news" => self.errors.news = Some(err.message),
                            "weather" => self.errors.weather = Some(err.message),
                            "stocks" => self.errors.stocks = Some(err.message),
                            _ => {}
                        }
                    }
                    FetchUpdate::Complete => {
                        self.fetching = false;
                        self.fetch_rx = None;
                    }
                }
            }
        }
    }

    /// Handle quit action
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Toggle weather widget expansion
    pub fn toggle_weather(&mut self) {
        self.weather_expanded = !self.weather_expanded;
    }

    /// Open stock browser modal
    pub fn open_stock_browser(&mut self) {
        self.stock_browser = StockBrowserState::new_with_watchlist(&self.watchlist);
        self.stock_browser_open = true;
    }

    /// Close stock browser without saving
    pub fn close_stock_browser(&mut self) {
        self.stock_browser_open = false;
    }

    /// Save stock browser selections and close
    pub fn save_stock_browser(&mut self) {
        // Update watchlist based on selections
        let new_watchlist: Vec<String> = self
            .stock_browser
            .available
            .iter()
            .filter(|s| s.in_watchlist)
            .map(|s| s.symbol.clone())
            .collect();

        // Update database
        if let Ok(db) = self.db.lock() {
            // Remove all current items
            for symbol in &self.watchlist {
                let _ = db.remove_from_watchlist(symbol);
            }
            // Add new selections
            for symbol in &new_watchlist {
                let _ = db.add_to_watchlist(symbol);
            }
        }

        self.watchlist = new_watchlist;
        self.stock_browser_open = false;

        // Trigger refresh to get new stock data
        self.start_fetch();
    }

    /// Navigate news selection
    pub fn news_next(&mut self) {
        if !self.news.is_empty() {
            self.news_selected = (self.news_selected + 1) % self.news.len();
        }
    }

    pub fn news_prev(&mut self) {
        if !self.news.is_empty() {
            self.news_selected = self.news_selected.checked_sub(1).unwrap_or(self.news.len() - 1);
        }
    }

    /// Navigate stock browser
    pub fn stock_browser_next(&mut self) {
        let filtered_len = self.stock_browser.filtered_stocks().len();
        if filtered_len > 0 {
            self.stock_browser.selected = (self.stock_browser.selected + 1) % filtered_len;
        }
    }

    pub fn stock_browser_prev(&mut self) {
        let filtered_len = self.stock_browser.filtered_stocks().len();
        if filtered_len > 0 {
            self.stock_browser.selected = self
                .stock_browser
                .selected
                .checked_sub(1)
                .unwrap_or(filtered_len - 1);
        }
    }

    /// Toggle selected stock in browser
    pub fn stock_browser_toggle(&mut self) {
        let filtered: Vec<_> = self.stock_browser.filtered_stocks();
        if let Some(symbol) = filtered.get(self.stock_browser.selected).map(|s| s.symbol.clone()) {
            if let Some(stock) = self
                .stock_browser
                .available
                .iter_mut()
                .find(|s| s.symbol == symbol)
            {
                stock.in_watchlist = !stock.in_watchlist;
            }
        }
    }

    /// Switch active panel
    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::News => Panel::Stocks,
            Panel::Stocks => Panel::News,
        };
    }
}

impl StockBrowserState {
    pub fn new() -> Self {
        let available = StocksClient::available_stocks()
            .into_iter()
            .map(|(symbol, name)| AvailableStock {
                symbol,
                name,
                in_watchlist: false,
            })
            .collect();

        Self {
            available,
            selected: 0,
            filter: String::new(),
        }
    }

    pub fn new_with_watchlist(watchlist: &[String]) -> Self {
        let available = StocksClient::available_stocks()
            .into_iter()
            .map(|(symbol, name)| AvailableStock {
                symbol: symbol.clone(),
                name,
                in_watchlist: watchlist.contains(&symbol),
            })
            .collect();

        Self {
            available,
            selected: 0,
            filter: String::new(),
        }
    }

    pub fn filtered_stocks(&self) -> Vec<&AvailableStock> {
        self.available
            .iter()
            .filter(|s| {
                self.filter.is_empty()
                    || s.symbol.to_lowercase().contains(&self.filter.to_lowercase())
                    || s.name.to_lowercase().contains(&self.filter.to_lowercase())
            })
            .collect()
    }
}

impl Default for App {
    fn default() -> Self {
        panic!("App must be created with App::new(config, db)")
    }
}

impl Default for StockBrowserState {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat(app): implement full application state with fetch integration"
```

---

### Task 13: Implement Main Layout

**Files:**
- Modify: `src/ui/layout.rs`

**Step 1: Write main layout**

Replace `src/ui/layout.rs` with:
```rust
//! Main layout composition

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::App;

use super::help_overlay::render_help_overlay;
use super::news_panel::render_news_panel;
use super::stock_browser::render_stock_browser;
use super::stocks_panel::render_stocks_panel;
use super::weather_widget::render_weather_widget;

/// Render the main layout
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main vertical split: News (top), Stocks (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // News panel
            Constraint::Percentage(45), // Stocks panel
            Constraint::Length(1),      // Status bar
        ])
        .split(area);

    // Render main panels
    render_news_panel(frame, app, chunks[0]);
    render_stocks_panel(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Render weather widget (floating, bottom-left of stocks area)
    render_weather_widget(frame, app, chunks[1]);

    // Render modal overlays if active
    if app.stock_browser_open {
        render_stock_browser(frame, app, area);
    }

    // Help overlay (if we add a help_open flag later)
    // render_help_overlay(frame, app, area);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    let mut spans = vec![];

    // Offline indicator
    if app.offline {
        spans.push(Span::styled(
            " OFFLINE ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ));
        spans.push(Span::raw(" "));
    }

    // Fetching indicator
    if app.fetching {
        spans.push(Span::styled(
            "Refreshing...",
            Style::default().fg(Color::Cyan),
        ));
        spans.push(Span::raw(" │ "));
    }

    // Keybindings
    let keybinds = if app.config.general.vim_mode {
        "[j/k] Navigate  [Tab] Switch  [r] Refresh  [s] Stocks  [w] Weather  [?] Help  [q] Quit"
    } else {
        "[↑↓] Navigate  [Tab] Switch  [r] Refresh  [s] Stocks  [w] Weather  [?] Help  [q] Quit"
    };

    spans.push(Span::styled(keybinds, Style::default().fg(Color::DarkGray)));

    let status = Paragraph::new(Line::from(spans));
    frame.render_widget(status, area);
}

/// Calculate the area for a centered modal
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds (warnings about unused imports OK for now)

**Step 3: Commit**

```bash
git add src/ui/layout.rs
git commit -m "feat(ui): implement main layout with status bar"
```

---

### Task 14: Implement News Panel

**Files:**
- Modify: `src/ui/news_panel.rs`

**Step 1: Write news panel component**

Replace `src/ui/news_panel.rs` with:
```rust
//! News headlines panel

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{App, Panel};
use crate::data::cache::format_relative_time;

/// Render the news panel
pub fn render_news_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::News;

    // Build title with status
    let mut title_parts = vec![Span::raw(" NEWS ")];

    if app.fetching {
        title_parts.push(Span::styled(
            "[Refreshing...] ",
            Style::default().fg(Color::Cyan),
        ));
    }

    if let Some(err) = &app.errors.news {
        title_parts.push(Span::styled(
            format!("⚠ {} ", err),
            Style::default().fg(Color::Yellow),
        ));
    }

    let block = Block::default()
        .title(Line::from(title_parts))
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    // Build list items
    let items: Vec<ListItem> = if app.news.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled(
                "  No news available. Press 'r' to refresh.",
                Style::default().fg(Color::DarkGray),
            ),
        ]))]
    } else {
        app.news
            .iter()
            .map(|item| {
                let source = item.source.as_deref().unwrap_or("Unknown");
                let time = format_relative_time(item.published_at);

                let line = Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(&item.headline, Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(
                        format!("({}, {})", source, time),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect()
    };

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.news_selected));

    frame.render_stateful_widget(list, area, &mut state);
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/ui/news_panel.rs
git commit -m "feat(ui): implement news panel with selection and status"
```

---

### Task 15: Implement Stocks Panel

**Files:**
- Modify: `src/ui/stocks_panel.rs`

**Step 1: Write stocks panel component**

Replace `src/ui/stocks_panel.rs` with:
```rust
//! Stocks panel with watchlist subpanels

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Panel};
use crate::data::cache::format_relative_time;

/// Render the stocks panel
pub fn render_stocks_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Stocks;

    // Build title with status
    let mut title_parts = vec![Span::raw(" STOCKS ")];

    if let Some(first_stock) = app.stocks.first() {
        title_parts.push(Span::styled(
            format!("Last updated: {} ", format_relative_time(first_stock.fetched_at)),
            Style::default().fg(Color::DarkGray),
        ));
    }

    if let Some(err) = &app.errors.stocks {
        title_parts.push(Span::styled(
            format!("⚠ {} ", err),
            Style::default().fg(Color::Yellow),
        ));
    }

    let block = Block::default()
        .title(Line::from(title_parts))
        .borders(Borders::ALL)
        .border_style(if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    frame.render_widget(block, area);

    // Inner area for stock cards
    let inner = Block::default().inner(area);

    if app.stocks.is_empty() {
        let msg = Paragraph::new("No stocks in watchlist. Press 's' to add some.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    // Calculate card layout - horizontal row of cards
    let card_count = app.stocks.len().min(6); // Max 6 cards
    let constraints: Vec<Constraint> = (0..card_count)
        .map(|_| Constraint::Percentage((100 / card_count) as u16))
        .collect();

    let card_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .margin(1)
        .split(inner);

    // Render each stock card
    for (i, stock) in app.stocks.iter().take(6).enumerate() {
        render_stock_card(frame, stock, card_areas[i]);
    }
}

/// Render a single stock card
fn render_stock_card(frame: &mut Frame, stock: &crate::data::models::StockData, area: Rect) {
    let price = stock.price.map(|p| format!("{:.2}", p)).unwrap_or_else(|| "N/A".to_string());

    let (change_str, change_color, arrow) = match stock.change_percent {
        Some(pct) if pct > 0.0 => (format!("+{:.2}%", pct), Color::Green, "▲"),
        Some(pct) if pct < 0.0 => (format!("{:.2}%", pct), Color::Red, "▼"),
        Some(_) => ("0.00%".to_string(), Color::DarkGray, "─"),
        None => ("N/A".to_string(), Color::DarkGray, " "),
    };

    let name = stock.name.as_deref().unwrap_or(&stock.symbol);

    let card = Block::default()
        .title(Span::styled(&stock.symbol, Style::default().fg(Color::White)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(card.clone(), area);

    let inner = card.inner(area);

    // Only render content if we have enough space
    if inner.height >= 2 {
        let content = vec![
            Line::from(vec![
                Span::styled(price, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled(change_str, Style::default().fg(change_color)),
                Span::raw(" "),
                Span::styled(arrow, Style::default().fg(change_color)),
            ]),
        ];

        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, inner);
    }
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/ui/stocks_panel.rs
git commit -m "feat(ui): implement stocks panel with watchlist cards"
```

---

### Task 16: Implement Weather Widget

**Files:**
- Modify: `src/ui/weather_widget.rs`

**Step 1: Write weather widget component**

Replace `src/ui/weather_widget.rs` with:
```rust
//! Floating weather widget

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::data::cache::format_relative_time;

/// Render the weather widget (floating in bottom-left of parent area)
pub fn render_weather_widget(frame: &mut Frame, app: &App, parent_area: Rect) {
    if app.weather_expanded {
        render_expanded_weather(frame, app, parent_area);
    } else {
        render_collapsed_weather(frame, app, parent_area);
    }
}

/// Render collapsed weather widget (small floating box)
fn render_collapsed_weather(frame: &mut Frame, app: &App, parent_area: Rect) {
    // Position in bottom-left, small size
    let width = 22;
    let height = 3;

    let x = parent_area.x + 2;
    let y = parent_area.y + parent_area.height.saturating_sub(height + 1);

    let area = Rect::new(x, y, width.min(parent_area.width - 4), height);

    // Clear the area behind the widget
    frame.render_widget(Clear, area);

    let content = match &app.weather {
        Some(weather) => {
            let temp = weather
                .temperature
                .map(|t| format!("{:.0}°F", t))
                .unwrap_or_else(|| "N/A".to_string());

            let icon = get_weather_icon(weather.condition.as_deref());

            // Check for alerts
            let alert_indicator = if weather.alert_title.is_some() {
                Span::styled(" ⚠", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            };

            vec![
                Line::from(vec![
                    Span::raw(icon),
                    Span::raw(" "),
                    Span::styled(&weather.location, Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(temp, Style::default().fg(Color::Cyan)),
                    alert_indicator,
                ]),
            ]
        }
        None => {
            vec![Line::from(vec![
                Span::styled("Weather unavailable", Style::default().fg(Color::DarkGray)),
            ])]
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled("[w]", Style::default().fg(Color::DarkGray)));

    let widget = Paragraph::new(content).block(block);
    frame.render_widget(widget, area);
}

/// Render expanded weather panel (full width)
fn render_expanded_weather(frame: &mut Frame, app: &App, parent_area: Rect) {
    // Full width panel at bottom of parent area
    let height = 8;
    let y = parent_area.y + parent_area.height.saturating_sub(height);
    let area = Rect::new(parent_area.x, y, parent_area.width, height);

    // Clear the area
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" WEATHER ", Style::default().fg(Color::Cyan)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &app.weather {
        Some(weather) => {
            let temp = weather
                .temperature
                .map(|t| format!("{:.0}°F", t))
                .unwrap_or_else(|| "N/A".to_string());

            let humidity = weather
                .humidity
                .map(|h| format!("{}%", h))
                .unwrap_or_else(|| "N/A".to_string());

            let wind = weather
                .wind_speed
                .map(|w| format!("{:.1} mph", w))
                .unwrap_or_else(|| "N/A".to_string());

            let condition = weather.condition.as_deref().unwrap_or("Unknown");
            let icon = get_weather_icon(Some(condition));
            let updated = format_relative_time(weather.fetched_at);

            let mut lines = vec![
                Line::from(vec![
                    Span::raw(icon),
                    Span::raw(" "),
                    Span::styled(&weather.location, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw(" - "),
                    Span::raw(condition),
                ]),
                Line::from(vec![
                    Span::raw("Temperature: "),
                    Span::styled(temp, Style::default().fg(Color::Cyan)),
                    Span::raw("  │  Humidity: "),
                    Span::styled(humidity, Style::default().fg(Color::Blue)),
                    Span::raw("  │  Wind: "),
                    Span::styled(wind, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("Updated: {}", updated),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ];

            // Add alert if present
            if let Some(alert_title) = &weather.alert_title {
                lines.push(Line::from(vec![]));
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("⚠ ALERT: {}", alert_title),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                ]));
                if let Some(desc) = &weather.alert_description {
                    // Truncate long descriptions
                    let desc_short: String = desc.chars().take(80).collect();
                    lines.push(Line::from(vec![
                        Span::styled(desc_short, Style::default().fg(Color::Yellow)),
                    ]));
                }
            }

            let content = Paragraph::new(lines);
            frame.render_widget(content, inner);
        }
        None => {
            let msg = Paragraph::new("Weather data unavailable. Press 'r' to refresh.")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, inner);
        }
    }
}

/// Get weather icon based on condition
fn get_weather_icon(condition: Option<&str>) -> &'static str {
    match condition.map(|s| s.to_lowercase()).as_deref() {
        Some(c) if c.contains("clear") || c.contains("sun") => "☀",
        Some(c) if c.contains("cloud") => "☁",
        Some(c) if c.contains("rain") || c.contains("drizzle") => "🌧",
        Some(c) if c.contains("thunder") || c.contains("storm") => "⛈",
        Some(c) if c.contains("snow") => "❄",
        Some(c) if c.contains("fog") || c.contains("mist") => "🌫",
        _ => "🌡",
    }
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/ui/weather_widget.rs
git commit -m "feat(ui): implement floating weather widget with expand/collapse"
```

---

### Task 17: Implement Stock Browser Modal

**Files:**
- Modify: `src/ui/stock_browser.rs`

**Step 1: Write stock browser modal**

Replace `src/ui/stock_browser.rs` with:
```rust
//! Stock browser modal for watchlist management

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;
use super::layout::centered_rect;

/// Render the stock browser modal
pub fn render_stock_browser(frame: &mut Frame, app: &App, area: Rect) {
    // Modal size: 50% width, 60% height
    let modal_area = centered_rect(50, 60, area);

    // Clear the area behind the modal
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(
            " Add to Watchlist ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block.clone(), modal_area);
    let inner = block.inner(modal_area);

    // Split inner area: filter input + list + help
    if inner.height < 6 {
        return; // Not enough space
    }

    let filter_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let list_area = Rect::new(inner.x, inner.y + 2, inner.width, inner.height.saturating_sub(4));
    let help_area = Rect::new(inner.x, inner.y + inner.height - 2, inner.width, 2);

    // Render filter input
    let filter_text = if app.stock_browser.filter.is_empty() {
        Span::styled("Search: (type to filter)", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(
            format!("Search: {}_", app.stock_browser.filter),
            Style::default().fg(Color::White),
        )
    };
    let filter_widget = Paragraph::new(Line::from(filter_text));
    frame.render_widget(filter_widget, filter_area);

    // Build list items
    let filtered_stocks = app.stock_browser.filtered_stocks();
    let items: Vec<ListItem> = filtered_stocks
        .iter()
        .map(|stock| {
            let checkbox = if stock.in_watchlist { "[x]" } else { "[ ]" };
            let line = Line::from(vec![
                Span::styled(
                    checkbox,
                    Style::default().fg(if stock.in_watchlist {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::raw(" "),
                Span::styled(&stock.symbol, Style::default().fg(Color::Cyan)),
                Span::raw(" - "),
                Span::styled(&stock.name, Style::default().fg(Color::White)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.stock_browser.selected));

    frame.render_stateful_widget(list, list_area, &mut state);

    // Render help text
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("[↑↓]", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled("[Space]", Style::default().fg(Color::Cyan)),
            Span::raw(" Toggle  "),
            Span::styled("[Enter]", Style::default().fg(Color::Green)),
            Span::raw(" Save  "),
            Span::styled("[Esc]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
    ]);
    frame.render_widget(help, help_area);
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/ui/stock_browser.rs
git commit -m "feat(ui): implement stock browser modal for watchlist management"
```

---

### Task 18: Implement Help Overlay

**Files:**
- Modify: `src/ui/help_overlay.rs`

**Step 1: Write help overlay**

Replace `src/ui/help_overlay.rs` with:
```rust
//! Help overlay

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use super::layout::centered_rect;

/// Render the help overlay
pub fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let modal_area = centered_rect(60, 70, area);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(
            " Help - The Daily Update ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let keybinds = if app.config.general.vim_mode {
        vec![
            ("j / k", "Navigate up/down"),
            ("h / l", "Switch panels"),
            ("g", "Go to top"),
            ("G", "Go to bottom"),
        ]
    } else {
        vec![
            ("↑ / ↓", "Navigate up/down"),
            ("Tab", "Switch panels"),
        ]
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![]),
    ];

    for (key, desc) in &keybinds {
        lines.push(Line::from(vec![
            Span::styled(format!("{:12}", key), Style::default().fg(Color::Cyan)),
            Span::raw(*desc),
        ]));
    }

    lines.extend(vec![
        Line::from(vec![]),
        Line::from(vec![
            Span::styled("Actions", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled("r           ", Style::default().fg(Color::Cyan)),
            Span::raw("Refresh all data"),
        ]),
        Line::from(vec![
            Span::styled("w           ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle weather panel"),
        ]),
        Line::from(vec![
            Span::styled("s           ", Style::default().fg(Color::Cyan)),
            Span::raw("Open stock browser"),
        ]),
        Line::from(vec![
            Span::styled("q           ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![]),
        Line::from(vec![
            Span::styled("Press any key to close", Style::default().fg(Color::DarkGray)),
        ]),
    ]);

    let help = Paragraph::new(lines);
    frame.render_widget(help, inner);
}
```

**Step 2: Verify build**

Run:
```bash
cargo build
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git add src/ui/help_overlay.rs
git commit -m "feat(ui): implement help overlay"
```

---

## Phase 7: Integration

### Task 19: Wire Everything Together in Main

**Files:**
- Modify: `src/main.rs`

**Step 1: Write full main entry point**

Replace `src/main.rs` with:
```rust
//! The Daily Update - Terminal news aggregator

mod app;
mod api;
mod config;
mod data;
mod fetch;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use config::settings::Config;
use config::setup::{display_banner, needs_setup, run_setup_wizard};
use data::db::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Load or create configuration
    let mut config = Config::load()?;

    // Run first-time setup if needed
    if needs_setup(&config) {
        display_banner();
        run_setup_wizard(&mut config)?;
    }

    // Check if we have API keys after setup
    if !config.has_api_keys() {
        eprintln!("Error: Missing API keys. Please set environment variables:");
        for key in config.missing_api_keys() {
            eprintln!("  export {}=\"your-key\"", key);
        }
        std::process::exit(1);
    }

    // Initialize database
    let db_path = Config::database_path()?;
    let db = Database::open(&db_path)?;

    // Create application state
    let mut app = App::new(config, db);

    // Start initial background fetch
    app.start_fetch();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Process any pending fetch updates
        app.process_fetch_updates();

        // Draw UI
        terminal.draw(|frame| {
            ui::layout::render(frame, app);
        })?;

        // Handle input with timeout for async updates
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle stock browser modal inputs first
                if app.stock_browser_open {
                    match key.code {
                        KeyCode::Esc => app.close_stock_browser(),
                        KeyCode::Enter => app.save_stock_browser(),
                        KeyCode::Up => app.stock_browser_prev(),
                        KeyCode::Down => app.stock_browser_next(),
                        KeyCode::Char(' ') => app.stock_browser_toggle(),
                        KeyCode::Char('j') if app.config.general.vim_mode => {
                            app.stock_browser_next()
                        }
                        KeyCode::Char('k') if app.config.general.vim_mode => {
                            app.stock_browser_prev()
                        }
                        KeyCode::Backspace => {
                            app.stock_browser.filter.pop();
                            app.stock_browser.selected = 0;
                        }
                        KeyCode::Char(c) if c.is_alphanumeric() => {
                            app.stock_browser.filter.push(c);
                            app.stock_browser.selected = 0;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Global keybindings
                match key.code {
                    KeyCode::Char('q') => app.quit(),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.quit()
                    }
                    KeyCode::Char('r') => app.start_fetch(),
                    KeyCode::Char('w') => app.toggle_weather(),
                    KeyCode::Char('s') => app.open_stock_browser(),
                    KeyCode::Tab => app.next_panel(),
                    KeyCode::BackTab => app.next_panel(), // Same as Tab for now

                    // Navigation
                    KeyCode::Up => app.news_prev(),
                    KeyCode::Down => app.news_next(),
                    KeyCode::Char('j') if app.config.general.vim_mode => app.news_next(),
                    KeyCode::Char('k') if app.config.general.vim_mode => app.news_prev(),

                    _ => {}
                }
            }
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
```

**Step 2: Run the application**

Run:
```bash
cargo run
```

Expected: Application starts (may show API key errors if not configured)

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire up main application with event loop and TUI"
```

---

### Task 20: Add .gitignore and Final Polish

**Files:**
- Create: `.gitignore`
- Create: `README.md`

**Step 1: Create .gitignore**

```bash
cat > /home/alext/the-daily-update/.gitignore << 'EOF'
# Build artifacts
/target/
Cargo.lock

# IDE
.idea/
.vscode/
*.swp
*.swo

# Environment
.env
.env.local

# OS
.DS_Store
Thumbs.db
EOF
```

**Step 2: Create README**

```bash
cat > /home/alext/the-daily-update/README.md << 'EOF'
# The Daily Update

A terminal-based daily news aggregator with weather and stock market data.

## Features

- Top 10 news headlines from NewsAPI
- Weather data with alert support from OpenWeatherMap
- Stock watchlist with real-time prices from Tiingo
- Instant startup with cached data
- Background refresh on startup
- Offline mode with graceful degradation

## Installation

```bash
cargo build --release
```

## Configuration

Set API keys as environment variables:

```bash
export NEWS_API_KEY="your-newsapi-key"
export WEATHER_API_KEY="your-openweathermap-key"
export TIINGO_API_KEY="your-tiingo-key"
```

Or run the app for first-time setup wizard.

## Usage

```bash
cargo run
```

### Keybindings

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate news |
| Tab | Switch panels |
| r | Refresh data |
| w | Toggle weather |
| s | Stock browser |
| q | Quit |

## License

MIT
EOF
```

**Step 3: Commit**

```bash
git add .gitignore README.md
git commit -m "docs: add README and .gitignore"
```

---

## Final Verification

### Task 21: Build and Test

**Step 1: Run full build**

```bash
cargo build --release
```

Expected: Build succeeds

**Step 2: Run tests**

```bash
cargo test
```

Expected: All tests pass

**Step 3: Run clippy**

```bash
cargo clippy
```

Expected: No errors (warnings OK for MVP)

**Step 4: Tag release**

```bash
git tag -a v0.1.0 -m "Initial release - MVP with TUI, news, weather, stocks"
```

---

## Summary

This implementation plan creates a complete 0.1.0 MVP with:

1. **Project setup** with all dependencies
2. **Data layer** with SQLite persistence and models
3. **Configuration** with env vars, TOML file, and first-run wizard
4. **API clients** for NewsAPI, OpenWeatherMap, and Tiingo
5. **Background fetcher** with async updates
6. **Full TUI** with news panel, stocks panel, floating weather widget, and stock browser modal
7. **Event loop** with keybinding support

Total: ~21 tasks, each 2-5 minutes, following TDD where applicable.

---

**Plan complete and saved to `docs/plans/2024-12-04-daily-update-implementation.md`.**

**Two execution options:**

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session in worktree with executing-plans, batch execution with checkpoints

**Which approach?**
