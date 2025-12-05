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
        self.conn
            .execute("DELETE FROM weather WHERE location = ?1", [&data.location])?;

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
        self.conn
            .execute("DELETE FROM stocks WHERE symbol = ?1", [&data.symbol])?;

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
}

/// Parse RFC3339 datetime string
///
/// Falls back to current time if parsing fails. This is intentional for graceful degradation
/// when reading potentially corrupted database records.
fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| {
            // Fallback to current time for corrupted timestamps
            Utc::now()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
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
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        };

        db.insert_news(&item).unwrap();
        let news = db.get_news(1).unwrap();

        assert_eq!(news[0].headline, "Minimal headline");
        assert!(news[0].source.is_none());
        assert!(news[0].description.is_none());
        assert!(news[0].url.is_none());
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
            published_at: now - Duration::hours(2),
            fetched_at: now,
        };
        let new_item = NewsItem {
            id: None,
            headline: "New news".to_string(),
            source: None,
            description: None,
            url: None,
            published_at: now,
            fetched_at: now,
        };
        let middle_item = NewsItem {
            id: None,
            headline: "Middle news".to_string(),
            source: None,
            description: None,
            url: None,
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
    // Helper Functions
    // ============================================================

    fn create_test_news_item(headline: &str) -> NewsItem {
        NewsItem {
            id: None,
            headline: headline.to_string(),
            source: Some("Test Source".to_string()),
            description: Some("Test description".to_string()),
            url: Some("https://example.com".to_string()),
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        }
    }

    fn create_test_weather_data(location: &str) -> WeatherData {
        WeatherData {
            id: None,
            location: location.to_string(),
            temperature: Some(72.5),
            condition: Some("Sunny".to_string()),
            humidity: Some(50),
            wind_speed: Some(10.0),
            alert_title: None,
            alert_description: None,
            alert_severity: None,
            fetched_at: Utc::now(),
        }
    }

    fn create_test_stock_data(symbol: &str, name: &str) -> StockData {
        StockData {
            id: None,
            symbol: symbol.to_string(),
            name: Some(name.to_string()),
            price: Some(150.0),
            change_percent: Some(1.5),
            fetched_at: Utc::now(),
        }
    }
}
