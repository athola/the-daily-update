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
            )?;

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
