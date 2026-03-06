//! SQLite database operations

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

use super::models::{NewsItem, NewsStockMention, StockData, WatchlistItem, WeatherData};

/// Database wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;
        conn.execute_batch("PRAGMA foreign_keys = ON")?;

        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
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
                location TEXT,
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

            CREATE TABLE IF NOT EXISTS news_stock_mentions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                news_id INTEGER NOT NULL,
                symbol TEXT NOT NULL,
                mentioned_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (news_id) REFERENCES news(id) ON DELETE CASCADE,
                UNIQUE(news_id, symbol)
            );

            CREATE INDEX IF NOT EXISTS idx_news_published ON news(published_at);
            CREATE INDEX IF NOT EXISTS idx_weather_location ON weather(location);
            CREATE INDEX IF NOT EXISTS idx_stocks_symbol ON stocks(symbol);
            CREATE INDEX IF NOT EXISTS idx_mentions_news_id ON news_stock_mentions(news_id);
            CREATE INDEX IF NOT EXISTS idx_mentions_symbol ON news_stock_mentions(symbol);
            "#,
        )?;
        Ok(())
    }

    // === News Operations ===

    /// Insert a news item
    pub fn insert_news(&self, item: &NewsItem) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO news (headline, source, description, url, location, published_at, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.headline,
                item.source,
                item.description,
                item.url,
                item.location,
                item.published_at.to_rfc3339(),
                item.fetched_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get recent news items
    pub fn get_news(&self, limit: usize) -> Result<Vec<NewsItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, headline, source, description, url, location, published_at, fetched_at
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
                    location: row.get(5)?,
                    published_at: parse_datetime(row.get::<_, String>(6)?),
                    fetched_at: parse_datetime(row.get::<_, String>(7)?),
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

    /// Atomically replace all news items (clear + insert in a transaction)
    pub fn replace_news(&self, items: &[NewsItem]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM news", [])?;
        for item in items {
            tx.execute(
                "INSERT INTO news (headline, source, description, url, location, published_at, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    item.headline,
                    item.source,
                    item.description,
                    item.url,
                    item.location,
                    item.published_at.to_rfc3339(),
                    item.fetched_at.to_rfc3339(),
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Delete news older than the specified number of days
    pub fn prune_news_older_than_days(&self, days: u32) -> Result<()> {
        self.conn.execute(
            "DELETE FROM news WHERE published_at < strftime('%Y-%m-%dT%H:%M:%S+00:00', 'now', ?1)",
            [format!("-{} days", days)],
        )?;
        Ok(())
    }

    // === Weather Operations ===

    /// Insert or update weather for a location
    pub fn upsert_weather(&self, data: &WeatherData) -> Result<i64> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM weather WHERE location = ?1", [&data.location])?;
        tx.execute(
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
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
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
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM stocks WHERE symbol = ?1", [&data.symbol])?;
        tx.execute(
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
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
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
        let mut stmt = self
            .conn
            .prepare("SELECT id, symbol, display_order FROM watchlist ORDER BY display_order")?;

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
        let max_order: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(display_order), -1) FROM watchlist",
            [],
            |row| row.get(0),
        )?;

        self.conn.execute(
            "INSERT OR IGNORE INTO watchlist (symbol, display_order) VALUES (?1, ?2)",
            params![symbol, max_order + 1],
        )?;
        Ok(())
    }

    /// Remove symbol from watchlist
    pub fn remove_from_watchlist(&self, symbol: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM watchlist WHERE symbol = ?1", [symbol])?;
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

    // === News Stock Mention Operations ===

    /// Add a stock mention for a news item
    pub fn add_stock_mention(&self, news_id: i64, symbol: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO news_stock_mentions (news_id, symbol, mentioned_at)
             VALUES (?1, ?2, ?3)",
            params![news_id, symbol, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Get all mentions for a news item
    pub fn get_mentions_for_news(&self, news_id: i64) -> Result<Vec<NewsStockMention>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, news_id, symbol, mentioned_at
             FROM news_stock_mentions WHERE news_id = ?1",
        )?;

        let mentions = stmt
            .query_map([news_id], |row| {
                Ok(NewsStockMention {
                    id: Some(row.get(0)?),
                    news_id: row.get(1)?,
                    symbol: row.get(2)?,
                    mentioned_at: parse_datetime(row.get::<_, String>(3)?),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(mentions)
    }

    /// Get all news IDs that mention a symbol
    pub fn get_news_for_symbol(&self, symbol: &str) -> Result<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT news_id FROM news_stock_mentions WHERE symbol = ?1")?;

        let ids = stmt
            .query_map([symbol], |row| row.get(0))?
            .collect::<Result<Vec<i64>, _>>()?;

        Ok(ids)
    }

    /// Get unique symbols mentioned in recent news
    pub fn get_recently_mentioned_symbols(&self, limit: usize) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT symbol FROM news_stock_mentions
             ORDER BY mentioned_at DESC LIMIT ?1",
        )?;

        let symbols = stmt
            .query_map([limit], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(symbols)
    }
}

/// Parse RFC3339 datetime string
///
/// Falls back to UNIX epoch if parsing fails, ensuring corrupted records are treated
/// as stale and will be refreshed on the next fetch cycle.
fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|e| {
            eprintln!("Warning: corrupted timestamp '{}': {} - using epoch", s, e);
            DateTime::UNIX_EPOCH
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{
        create_test_news_item, create_test_stock_data, create_test_weather_data,
    };
    use chrono::Duration;

    // ============================================================
    // Database Initialization Tests
    // ============================================================

    #[test]
    fn given_new_database_when_created_then_schema_is_initialized() {
        let db = Database::in_memory().expect("Failed to create in-memory database");

        // Verify all tables exist by querying them
        let news = db.get_news(10).expect("News table should exist");
        assert!(news.is_empty());

        let watchlist = db.get_watchlist().expect("Watchlist table should exist");
        assert!(watchlist.is_empty());
    }

    // ============================================================
    // News Operations Tests
    // ============================================================

    #[test]
    fn given_empty_database_when_getting_news_then_returns_empty_list() {
        let db = Database::in_memory().unwrap();
        let news = db.get_news(10).unwrap();
        assert!(news.is_empty());
    }

    #[test]
    fn given_news_item_when_inserted_then_returns_row_id() {
        let db = Database::in_memory().unwrap();
        let item = create_test_news_item("Test headline");

        let id = db.insert_news(&item).unwrap();
        assert!(id > 0, "Insert should return a positive row ID");
    }

    #[test]
    fn given_news_item_when_inserted_then_can_be_retrieved() {
        let db = Database::in_memory().unwrap();
        let item = create_test_news_item("Test headline");

        db.insert_news(&item).unwrap();
        let news = db.get_news(10).unwrap();

        assert_eq!(news.len(), 1);
        assert_eq!(news[0].headline, "Test headline");
        assert!(news[0].id.is_some(), "Retrieved item should have an ID");
    }

    #[test]
    fn given_news_item_with_all_fields_when_retrieved_then_all_fields_preserved() {
        let db = Database::in_memory().unwrap();
        let item = NewsItem {
            id: None,
            headline: "Breaking News".to_string(),
            source: Some("Reuters".to_string()),
            description: Some("Important news description".to_string()),
            url: Some("https://reuters.com/article".to_string()),
            location: Some("New York".to_string()),
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        };

        db.insert_news(&item).unwrap();
        let news = db.get_news(1).unwrap();

        assert_eq!(news[0].headline, "Breaking News");
        assert_eq!(news[0].source, Some("Reuters".to_string()));
        assert_eq!(
            news[0].description,
            Some("Important news description".to_string())
        );
        assert_eq!(news[0].url, Some("https://reuters.com/article".to_string()));
        assert_eq!(news[0].location, Some("New York".to_string()));
    }

    #[test]
    fn given_news_item_with_null_optional_fields_when_retrieved_then_nulls_preserved() {
        let db = Database::in_memory().unwrap();
        let item = NewsItem {
            id: None,
            headline: "Minimal headline".to_string(),
            source: None,
            description: None,
            url: None,
            location: None,
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        };

        db.insert_news(&item).unwrap();
        let news = db.get_news(1).unwrap();

        assert_eq!(news[0].headline, "Minimal headline");
        assert!(news[0].source.is_none());
        assert!(news[0].description.is_none());
        assert!(news[0].url.is_none());
        assert!(news[0].location.is_none());
    }

    #[test]
    fn given_multiple_news_items_when_retrieved_then_ordered_by_published_at_desc() {
        let db = Database::in_memory().unwrap();
        let now = Utc::now();

        // Insert items in non-chronological order
        let old_item = NewsItem {
            id: None,
            headline: "Old news".to_string(),
            source: None,
            description: None,
            url: None,
            location: None,
            published_at: now - Duration::hours(2),
            fetched_at: now,
        };
        let new_item = NewsItem {
            id: None,
            headline: "New news".to_string(),
            source: None,
            description: None,
            url: None,
            location: None,
            published_at: now,
            fetched_at: now,
        };
        let middle_item = NewsItem {
            id: None,
            headline: "Middle news".to_string(),
            source: None,
            description: None,
            url: None,
            location: None,
            published_at: now - Duration::hours(1),
            fetched_at: now,
        };

        db.insert_news(&old_item).unwrap();
        db.insert_news(&new_item).unwrap();
        db.insert_news(&middle_item).unwrap();

        let news = db.get_news(10).unwrap();

        assert_eq!(news.len(), 3);
        assert_eq!(news[0].headline, "New news", "Most recent should be first");
        assert_eq!(news[1].headline, "Middle news");
        assert_eq!(news[2].headline, "Old news", "Oldest should be last");
    }

    #[test]
    fn given_multiple_news_items_when_limit_applied_then_returns_limited_count() {
        let db = Database::in_memory().unwrap();

        for i in 0..5 {
            let item = create_test_news_item(&format!("Headline {}", i));
            db.insert_news(&item).unwrap();
        }

        let news = db.get_news(3).unwrap();
        assert_eq!(news.len(), 3, "Should only return 3 items when limit is 3");
    }

    #[test]
    fn given_news_items_when_cleared_then_database_is_empty() {
        let db = Database::in_memory().unwrap();

        db.insert_news(&create_test_news_item("Item 1")).unwrap();
        db.insert_news(&create_test_news_item("Item 2")).unwrap();

        db.clear_news().unwrap();

        let news = db.get_news(10).unwrap();
        assert!(news.is_empty(), "Database should be empty after clear");
    }

    // ============================================================
    // Weather Operations Tests
    // ============================================================

    #[test]
    fn given_empty_database_when_getting_weather_then_returns_none() {
        let db = Database::in_memory().unwrap();
        let weather = db.get_weather("New York").unwrap();
        assert!(weather.is_none());
    }

    #[test]
    fn given_weather_data_when_upserted_then_can_be_retrieved() {
        let db = Database::in_memory().unwrap();
        let data = create_test_weather_data("New York");

        db.upsert_weather(&data).unwrap();
        let weather = db.get_weather("New York").unwrap();

        assert!(weather.is_some());
        let weather = weather.unwrap();
        assert_eq!(weather.location, "New York");
        assert_eq!(weather.temperature, Some(72.5));
    }

    #[test]
    fn given_weather_data_with_all_fields_when_retrieved_then_all_fields_preserved() {
        let db = Database::in_memory().unwrap();
        let data = WeatherData {
            id: None,
            location: "San Francisco".to_string(),
            temperature: Some(65.0),
            condition: Some("Partly Cloudy".to_string()),
            humidity: Some(75),
            wind_speed: Some(12.5),
            alert_title: Some("Wind Advisory".to_string()),
            alert_description: Some("High winds expected".to_string()),
            alert_severity: Some("moderate".to_string()),
            fetched_at: Utc::now(),
        };

        db.upsert_weather(&data).unwrap();
        let weather = db.get_weather("San Francisco").unwrap().unwrap();

        assert_eq!(weather.temperature, Some(65.0));
        assert_eq!(weather.condition, Some("Partly Cloudy".to_string()));
        assert_eq!(weather.humidity, Some(75));
        assert_eq!(weather.wind_speed, Some(12.5));
        assert_eq!(weather.alert_title, Some("Wind Advisory".to_string()));
        assert_eq!(
            weather.alert_description,
            Some("High winds expected".to_string())
        );
        assert_eq!(weather.alert_severity, Some("moderate".to_string()));
    }

    #[test]
    fn given_weather_data_when_upserted_twice_then_old_data_replaced() {
        let db = Database::in_memory().unwrap();

        // Insert initial weather
        let initial = WeatherData {
            id: None,
            location: "Chicago".to_string(),
            temperature: Some(50.0),
            condition: Some("Cloudy".to_string()),
            humidity: None,
            wind_speed: None,
            alert_title: None,
            alert_description: None,
            alert_severity: None,
            fetched_at: Utc::now() - Duration::hours(1),
        };
        db.upsert_weather(&initial).unwrap();

        // Update with new weather
        let updated = WeatherData {
            id: None,
            location: "Chicago".to_string(),
            temperature: Some(55.0),
            condition: Some("Sunny".to_string()),
            humidity: Some(40),
            wind_speed: None,
            alert_title: None,
            alert_description: None,
            alert_severity: None,
            fetched_at: Utc::now(),
        };
        db.upsert_weather(&updated).unwrap();

        let weather = db.get_weather("Chicago").unwrap().unwrap();
        assert_eq!(
            weather.temperature,
            Some(55.0),
            "Should have updated temperature"
        );
        assert_eq!(
            weather.condition,
            Some("Sunny".to_string()),
            "Should have updated condition"
        );
        assert_eq!(weather.humidity, Some(40), "Should have new humidity value");
    }

    #[test]
    fn given_weather_for_different_locations_when_queried_then_returns_correct_location() {
        let db = Database::in_memory().unwrap();

        db.upsert_weather(&create_test_weather_data("Boston"))
            .unwrap();
        db.upsert_weather(&create_test_weather_data("Miami"))
            .unwrap();

        let boston = db.get_weather("Boston").unwrap().unwrap();
        let miami = db.get_weather("Miami").unwrap().unwrap();
        let unknown = db.get_weather("Unknown").unwrap();

        assert_eq!(boston.location, "Boston");
        assert_eq!(miami.location, "Miami");
        assert!(unknown.is_none(), "Unknown location should return None");
    }

    // ============================================================
    // Stock Operations Tests
    // ============================================================

    #[test]
    fn given_empty_database_when_getting_stock_then_returns_none() {
        let db = Database::in_memory().unwrap();
        let stock = db.get_stock("AAPL").unwrap();
        assert!(stock.is_none());
    }

    #[test]
    fn given_stock_data_when_upserted_then_can_be_retrieved() {
        let db = Database::in_memory().unwrap();
        let data = create_test_stock_data("AAPL", "Apple Inc.");

        db.upsert_stock(&data).unwrap();
        let stock = db.get_stock("AAPL").unwrap();

        assert!(stock.is_some());
        let stock = stock.unwrap();
        assert_eq!(stock.symbol, "AAPL");
        assert_eq!(stock.name, Some("Apple Inc.".to_string()));
    }

    #[test]
    fn given_stock_data_with_all_fields_when_retrieved_then_all_fields_preserved() {
        let db = Database::in_memory().unwrap();
        let data = StockData {
            id: None,
            symbol: "GOOGL".to_string(),
            name: Some("Alphabet Inc.".to_string()),
            price: Some(175.50),
            change_percent: Some(2.35),
            fetched_at: Utc::now(),
        };

        db.upsert_stock(&data).unwrap();
        let stock = db.get_stock("GOOGL").unwrap().unwrap();

        assert_eq!(stock.symbol, "GOOGL");
        assert_eq!(stock.name, Some("Alphabet Inc.".to_string()));
        assert_eq!(stock.price, Some(175.50));
        assert_eq!(stock.change_percent, Some(2.35));
    }

    #[test]
    fn given_stock_data_when_upserted_twice_then_old_data_replaced() {
        let db = Database::in_memory().unwrap();

        let initial = StockData {
            id: None,
            symbol: "MSFT".to_string(),
            name: Some("Microsoft".to_string()),
            price: Some(400.0),
            change_percent: Some(1.0),
            fetched_at: Utc::now() - Duration::hours(1),
        };
        db.upsert_stock(&initial).unwrap();

        let updated = StockData {
            id: None,
            symbol: "MSFT".to_string(),
            name: Some("Microsoft Corp.".to_string()),
            price: Some(405.0),
            change_percent: Some(-0.5),
            fetched_at: Utc::now(),
        };
        db.upsert_stock(&updated).unwrap();

        let stock = db.get_stock("MSFT").unwrap().unwrap();
        assert_eq!(stock.price, Some(405.0), "Price should be updated");
        assert_eq!(
            stock.change_percent,
            Some(-0.5),
            "Change percent should be updated"
        );
        assert_eq!(
            stock.name,
            Some("Microsoft Corp.".to_string()),
            "Name should be updated"
        );
    }

    #[test]
    fn given_stock_with_negative_change_when_retrieved_then_negative_preserved() {
        let db = Database::in_memory().unwrap();
        let data = StockData {
            id: None,
            symbol: "TSLA".to_string(),
            name: Some("Tesla".to_string()),
            price: Some(250.0),
            change_percent: Some(-5.25),
            fetched_at: Utc::now(),
        };

        db.upsert_stock(&data).unwrap();
        let stock = db.get_stock("TSLA").unwrap().unwrap();

        assert_eq!(
            stock.change_percent,
            Some(-5.25),
            "Negative change should be preserved"
        );
    }

    // ============================================================
    // Watchlist Stock Integration Tests
    // ============================================================

    #[test]
    fn given_empty_watchlist_when_getting_watchlist_stocks_then_returns_empty() {
        let db = Database::in_memory().unwrap();

        // Add some stocks but not to watchlist
        db.upsert_stock(&create_test_stock_data("AAPL", "Apple"))
            .unwrap();

        let stocks = db.get_watchlist_stocks().unwrap();
        assert!(
            stocks.is_empty(),
            "No stocks should be returned if watchlist is empty"
        );
    }

    #[test]
    fn given_stocks_in_watchlist_when_getting_watchlist_stocks_then_returns_matching_stocks() {
        let db = Database::in_memory().unwrap();

        // Add stocks
        db.upsert_stock(&create_test_stock_data("AAPL", "Apple"))
            .unwrap();
        db.upsert_stock(&create_test_stock_data("GOOGL", "Alphabet"))
            .unwrap();
        db.upsert_stock(&create_test_stock_data("MSFT", "Microsoft"))
            .unwrap();

        // Add some to watchlist
        db.add_to_watchlist("AAPL").unwrap();
        db.add_to_watchlist("MSFT").unwrap();

        let stocks = db.get_watchlist_stocks().unwrap();

        assert_eq!(stocks.len(), 2);
        assert!(stocks.iter().any(|s| s.symbol == "AAPL"));
        assert!(stocks.iter().any(|s| s.symbol == "MSFT"));
        assert!(
            !stocks.iter().any(|s| s.symbol == "GOOGL"),
            "GOOGL should not be included"
        );
    }

    #[test]
    fn given_stocks_in_watchlist_when_getting_watchlist_stocks_then_ordered_by_display_order() {
        let db = Database::in_memory().unwrap();

        // Add stocks
        db.upsert_stock(&create_test_stock_data("SPY", "S&P 500"))
            .unwrap();
        db.upsert_stock(&create_test_stock_data("QQQ", "NASDAQ"))
            .unwrap();
        db.upsert_stock(&create_test_stock_data("DIA", "Dow Jones"))
            .unwrap();

        // Add to watchlist in specific order
        db.add_to_watchlist("QQQ").unwrap(); // Order 0
        db.add_to_watchlist("SPY").unwrap(); // Order 1
        db.add_to_watchlist("DIA").unwrap(); // Order 2

        let stocks = db.get_watchlist_stocks().unwrap();

        assert_eq!(stocks.len(), 3);
        assert_eq!(stocks[0].symbol, "QQQ", "QQQ should be first (added first)");
        assert_eq!(stocks[1].symbol, "SPY", "SPY should be second");
        assert_eq!(stocks[2].symbol, "DIA", "DIA should be third");
    }

    // ============================================================
    // Watchlist Operations Tests
    // ============================================================

    #[test]
    fn given_empty_watchlist_when_getting_watchlist_then_returns_empty() {
        let db = Database::in_memory().unwrap();
        let watchlist = db.get_watchlist().unwrap();
        assert!(watchlist.is_empty());
    }

    #[test]
    fn given_symbol_when_added_to_watchlist_then_appears_in_watchlist() {
        let db = Database::in_memory().unwrap();

        db.add_to_watchlist("SPY").unwrap();

        assert!(db.is_in_watchlist("SPY").unwrap());
        let watchlist = db.get_watchlist().unwrap();
        assert_eq!(watchlist.len(), 1);
        assert_eq!(watchlist[0].symbol, "SPY");
    }

    #[test]
    fn given_multiple_symbols_when_added_then_maintains_insertion_order() {
        let db = Database::in_memory().unwrap();

        db.add_to_watchlist("AAPL").unwrap();
        db.add_to_watchlist("GOOGL").unwrap();
        db.add_to_watchlist("MSFT").unwrap();

        let watchlist = db.get_watchlist().unwrap();

        assert_eq!(watchlist.len(), 3);
        assert_eq!(watchlist[0].symbol, "AAPL");
        assert_eq!(watchlist[0].display_order, 0);
        assert_eq!(watchlist[1].symbol, "GOOGL");
        assert_eq!(watchlist[1].display_order, 1);
        assert_eq!(watchlist[2].symbol, "MSFT");
        assert_eq!(watchlist[2].display_order, 2);
    }

    #[test]
    fn given_duplicate_symbol_when_added_to_watchlist_then_ignored() {
        let db = Database::in_memory().unwrap();

        db.add_to_watchlist("AAPL").unwrap();
        db.add_to_watchlist("AAPL").unwrap(); // Duplicate

        let watchlist = db.get_watchlist().unwrap();
        assert_eq!(watchlist.len(), 1, "Duplicate should be ignored");
    }

    #[test]
    fn given_symbol_in_watchlist_when_removed_then_no_longer_in_watchlist() {
        let db = Database::in_memory().unwrap();

        db.add_to_watchlist("SPY").unwrap();
        db.add_to_watchlist("QQQ").unwrap();

        db.remove_from_watchlist("SPY").unwrap();

        assert!(!db.is_in_watchlist("SPY").unwrap());
        assert!(db.is_in_watchlist("QQQ").unwrap());

        let watchlist = db.get_watchlist().unwrap();
        assert_eq!(watchlist.len(), 1);
    }

    #[test]
    fn given_nonexistent_symbol_when_removed_then_no_error() {
        let db = Database::in_memory().unwrap();

        // Should not error when removing a symbol that doesn't exist
        let result = db.remove_from_watchlist("NONEXISTENT");
        assert!(result.is_ok());
    }

    #[test]
    fn given_symbol_not_in_watchlist_when_checked_then_returns_false() {
        let db = Database::in_memory().unwrap();

        db.add_to_watchlist("AAPL").unwrap();

        assert!(!db.is_in_watchlist("GOOGL").unwrap());
        assert!(!db.is_in_watchlist("").unwrap());
    }

    // ============================================================
    // News Stock Mention Tests
    // ============================================================

    #[test]
    fn given_news_item_when_mention_added_then_can_be_retrieved() {
        let db = Database::in_memory().unwrap();
        let news = create_test_news_item("Apple announces new iPhone");
        let news_id = db.insert_news(&news).unwrap();

        db.add_stock_mention(news_id, "AAPL").unwrap();

        let mentions = db.get_mentions_for_news(news_id).unwrap();
        assert_eq!(mentions.len(), 1);
        assert_eq!(mentions[0].symbol, "AAPL");
    }

    #[test]
    fn given_duplicate_mention_when_added_then_ignored() {
        let db = Database::in_memory().unwrap();
        let news = create_test_news_item("Apple stock rises");
        let news_id = db.insert_news(&news).unwrap();

        db.add_stock_mention(news_id, "AAPL").unwrap();
        db.add_stock_mention(news_id, "AAPL").unwrap(); // Duplicate

        let mentions = db.get_mentions_for_news(news_id).unwrap();
        assert_eq!(mentions.len(), 1, "Duplicate should be ignored");
    }

    #[test]
    fn given_symbol_when_getting_news_mentions_then_returns_matching_news() {
        let db = Database::in_memory().unwrap();

        let apple_news = create_test_news_item("Apple earnings beat expectations");
        let google_news = create_test_news_item("Google AI breakthrough");
        let mixed_news = create_test_news_item("Tech giants Apple and Google compete");

        let apple_id = db.insert_news(&apple_news).unwrap();
        let google_id = db.insert_news(&google_news).unwrap();
        let mixed_id = db.insert_news(&mixed_news).unwrap();

        db.add_stock_mention(apple_id, "AAPL").unwrap();
        db.add_stock_mention(google_id, "GOOGL").unwrap();
        db.add_stock_mention(mixed_id, "AAPL").unwrap();
        db.add_stock_mention(mixed_id, "GOOGL").unwrap();

        let apple_mentions = db.get_news_for_symbol("AAPL").unwrap();
        assert_eq!(apple_mentions.len(), 2);
    }

    #[test]
    fn given_no_mentions_when_getting_unique_symbols_then_returns_empty() {
        let db = Database::in_memory().unwrap();
        let symbols = db.get_recently_mentioned_symbols(10).unwrap();
        assert!(symbols.is_empty());
    }

    #[test]
    fn given_mentions_when_getting_unique_symbols_then_returns_distinct_list() {
        let db = Database::in_memory().unwrap();

        let news1 = create_test_news_item("Apple news");
        let news2 = create_test_news_item("Google news");
        let news3 = create_test_news_item("More Apple news");

        let id1 = db.insert_news(&news1).unwrap();
        let id2 = db.insert_news(&news2).unwrap();
        let id3 = db.insert_news(&news3).unwrap();

        db.add_stock_mention(id1, "AAPL").unwrap();
        db.add_stock_mention(id2, "GOOGL").unwrap();
        db.add_stock_mention(id3, "AAPL").unwrap();

        let symbols = db.get_recently_mentioned_symbols(10).unwrap();
        assert_eq!(symbols.len(), 2);
        assert!(symbols.contains(&"AAPL".to_string()));
        assert!(symbols.contains(&"GOOGL".to_string()));
    }
}
