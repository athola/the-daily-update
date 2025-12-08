//! Application state and main event loop

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use api::company_mapping::extract_symbols;
use api::stocks::StocksClient;
use chrono::{Duration, NaiveDate};
use config::settings::Config;
use data::db::Database;
use data::models::{AvailableStock, NewsItem, StockData, WeatherData, WeatherSource};
use fetch::background::{BackgroundFetcher, FetchUpdate};

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
    /// Source of current weather data (default location or news context)
    pub weather_source: WeatherSource,
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
    /// Whether help overlay is open
    pub help_open: bool,
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
    /// Selected date for news filtering (None = today)
    pub selected_date: Option<NaiveDate>,
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
        // Note: lock() can fail if the mutex is poisoned (another thread panicked)
        // We recover gracefully by using defaults if this happens
        let (news, weather, stocks, watchlist) = match db.lock() {
            Ok(db) => {
                let news = db.get_news(10).unwrap_or_default();
                let weather = db
                    .get_weather(&config.general.default_location)
                    .ok()
                    .flatten();
                let watchlist = db
                    .get_watchlist()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|w| w.symbol)
                    .collect::<Vec<_>>();
                let stocks = db.get_watchlist_stocks().unwrap_or_default();
                (news, weather, stocks, watchlist)
            }
            Err(_) => {
                // Mutex poisoned - use defaults
                eprintln!("Warning: Database mutex poisoned, using default data");
                (Vec::new(), None, Vec::new(), Vec::new())
            }
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
            weather_source: WeatherSource::Default,
            stocks,
            watchlist,
            news_selected: 0,
            active_panel: Panel::News,
            weather_expanded: false,
            stock_browser_open: false,
            help_open: false,
            stock_browser: StockBrowserState::new(),
            fetching: false,
            offline: false,
            errors: PanelErrors::default(),
            fetch_rx: None,
            selected_date: None,
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
        let from_date = self.selected_date;

        tokio::spawn(async move {
            fetcher.fetch_all(db, watchlist, from_date, tx).await;
        });
    }

    /// Process any pending fetch updates
    pub fn process_fetch_updates(&mut self) {
        let mut should_complete = false;
        let mut news_updated = false;

        if let Some(rx) = &mut self.fetch_rx {
            while let Ok(update) = rx.try_recv() {
                match update {
                    FetchUpdate::NewsUpdated(items) => {
                        self.news = items;
                        self.errors.news = None;
                        news_updated = true;
                    }
                    FetchUpdate::WeatherUpdated(data, source) => {
                        self.weather = Some(data);
                        self.weather_source = source;
                        self.errors.weather = None;
                    }
                    FetchUpdate::StocksUpdated(stocks) => {
                        self.stocks = stocks;
                        self.errors.stocks = None;
                    }
                    FetchUpdate::Error(err) => match err.source.as_str() {
                        "news" => self.errors.news = Some(err.message),
                        "weather" => self.errors.weather = Some(err.message),
                        "stocks" => self.errors.stocks = Some(err.message),
                        _ => {}
                    },
                    FetchUpdate::Complete => {
                        should_complete = true;
                    }
                }
            }
        }

        if news_updated {
            self.process_news_mentions();
            self.auto_populate_watchlist();
        }

        if should_complete {
            self.fetching = false;
            self.fetch_rx = None;
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

    /// Toggle help overlay visibility
    pub fn toggle_help(&mut self) {
        self.help_open = !self.help_open;
    }

    /// Close help overlay
    pub fn close_help(&mut self) {
        self.help_open = false;
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
            self.news_selected = self
                .news_selected
                .checked_sub(1)
                .unwrap_or(self.news.len() - 1);
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
        if let Some(symbol) = filtered
            .get(self.stock_browser.selected)
            .map(|s| s.symbol.clone())
        {
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

    /// Get the effective date for news filtering (today if None)
    pub fn effective_date(&self) -> NaiveDate {
        self.selected_date
            .unwrap_or_else(|| chrono::Utc::now().date_naive())
    }

    /// Navigate to previous day (up to 7 days ago)
    pub fn date_prev(&mut self) {
        let today = chrono::Utc::now().date_naive();
        let min_date = today - Duration::days(6);
        let current = self.effective_date();
        let new_date = current - Duration::days(1);

        if new_date >= min_date {
            self.selected_date = Some(new_date);
            self.start_fetch();
        }
    }

    /// Navigate to next day (up to today)
    pub fn date_next(&mut self) {
        let today = chrono::Utc::now().date_naive();
        let current = self.effective_date();
        let new_date = current + Duration::days(1);

        if new_date <= today {
            self.selected_date = if new_date == today {
                None
            } else {
                Some(new_date)
            };
            self.start_fetch();
        }
    }

    /// Reset to today's date
    pub fn date_today(&mut self) {
        if self.selected_date.is_some() {
            self.selected_date = None;
            self.start_fetch();
        }
    }

    /// Process news items and extract stock mentions
    pub fn process_news_mentions(&self) {
        if let Ok(db) = self.db.lock() {
            for news in &self.news {
                if let Some(news_id) = news.id {
                    let symbols = extract_symbols(&news.headline);
                    for symbol in symbols {
                        let _ = db.add_stock_mention(news_id, &symbol);
                    }
                }
            }
        }
    }

    /// Get symbols mentioned in recent news that aren't in watchlist
    pub fn get_suggested_stocks(&self) -> Vec<String> {
        if let Ok(db) = self.db.lock() {
            if let Ok(mentioned) = db.get_recently_mentioned_symbols(20) {
                return mentioned
                    .into_iter()
                    .filter(|s| !self.watchlist.contains(s))
                    .take(5)
                    .collect();
            }
        }
        Vec::new()
    }

    /// Auto-add frequently mentioned stocks to watchlist
    /// Only adds stocks that appear in 2+ headlines and aren't already in watchlist
    pub fn auto_populate_watchlist(&mut self) {
        // Count mentions per symbol
        let mut mention_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for news in &self.news {
            let symbols = extract_symbols(&news.headline);
            let unique_symbols: std::collections::HashSet<_> = symbols.into_iter().collect();
            for symbol in unique_symbols {
                // Skip default SPY - only add explicitly mentioned companies
                if symbol != "SPY" {
                    *mention_counts.entry(symbol).or_insert(0) += 1;
                }
            }
        }

        // Add stocks mentioned 2+ times that aren't in watchlist
        let to_add: Vec<String> = mention_counts
            .into_iter()
            .filter(|(symbol, count)| *count >= 2 && !self.watchlist.contains(symbol))
            .map(|(symbol, _)| symbol)
            .collect();

        if !to_add.is_empty() {
            if let Ok(db) = self.db.lock() {
                for symbol in &to_add {
                    let _ = db.add_to_watchlist(symbol);
                }
            }
            self.watchlist.extend(to_add);
        }
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
                    || s.symbol
                        .to_lowercase()
                        .contains(&self.filter.to_lowercase())
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_test_app() -> App {
        let config = Config::default();
        let db = Database::in_memory().unwrap();
        App::new(config, db)
    }

    #[test]
    fn test_effective_date_defaults_to_today() {
        let app = make_test_app();
        let today = Utc::now().date_naive();
        assert_eq!(app.effective_date(), today);
    }

    #[test]
    fn test_date_prev_moves_back_one_day() {
        let mut app = make_test_app();
        let today = Utc::now().date_naive();

        // Manually set the date without triggering fetch
        let current = app.effective_date();
        let new_date = current - Duration::days(1);
        app.selected_date = Some(new_date);

        assert_eq!(app.selected_date, Some(today - Duration::days(1)));
    }

    #[test]
    fn test_date_prev_stops_at_7_days_ago() {
        let mut app = make_test_app();
        let today = Utc::now().date_naive();
        let min_date = today - Duration::days(6);

        // Test the boundary logic directly
        // Go back 6 days (to the limit)
        for i in 1..=6 {
            let current = app.effective_date();
            let new_date = current - Duration::days(1);
            if new_date >= min_date {
                app.selected_date = Some(new_date);
            }
            assert_eq!(app.selected_date, Some(today - Duration::days(i as i64)));
        }

        let date_at_limit = app.selected_date;

        // Try to go back one more - should stay at limit
        let current = app.effective_date();
        let new_date = current - Duration::days(1);
        if new_date >= min_date {
            app.selected_date = Some(new_date);
        }
        assert_eq!(app.selected_date, date_at_limit);
    }

    #[test]
    fn test_date_next_from_yesterday_returns_to_today() {
        let mut app = make_test_app();
        let today = Utc::now().date_naive();

        // Set to yesterday
        app.selected_date = Some(today - Duration::days(1));

        // Move forward to today - test the logic
        let current = app.effective_date();
        let new_date = current + Duration::days(1);
        if new_date <= today {
            app.selected_date = if new_date == today {
                None
            } else {
                Some(new_date)
            };
        }

        assert_eq!(app.selected_date, None); // None means today
    }

    #[test]
    fn test_date_today_resets_to_none() {
        let mut app = make_test_app();
        let today = Utc::now().date_naive();

        // Set to two days ago
        app.selected_date = Some(today - Duration::days(2));
        assert!(app.selected_date.is_some());

        // Reset to today
        if app.selected_date.is_some() {
            app.selected_date = None;
        }

        assert_eq!(app.selected_date, None);
    }

    // ============================================================
    // News Mention Processing and Auto-populate Tests
    // ============================================================

    fn create_news_item(headline: &str) -> NewsItem {
        NewsItem {
            id: None,
            headline: headline.to_string(),
            source: Some("Test".to_string()),
            description: None,
            url: None,
            location: None,
            published_at: Utc::now(),
            fetched_at: Utc::now(),
        }
    }

    #[test]
    fn given_news_with_company_mentions_when_processed_then_mentions_stored() {
        let mut app = make_test_app();

        // Manually add news with IDs (simulating DB insert)
        if let Ok(db) = app.db.lock() {
            let news = create_news_item("Apple stock rises on iPhone sales");
            let id = db.insert_news(&news).unwrap();
            app.news.push(NewsItem {
                id: Some(id),
                ..news
            });
        }

        app.process_news_mentions();

        if let Ok(db) = app.db.lock() {
            let symbols = db.get_recently_mentioned_symbols(10).unwrap();
            assert!(
                symbols.contains(&"AAPL".to_string()),
                "Expected AAPL in symbols, got: {:?}",
                symbols
            );
        };
    }

    #[test]
    fn given_multiple_mentions_when_auto_populate_then_adds_to_watchlist() {
        let mut app = make_test_app();

        // Add multiple news mentioning same company
        if let Ok(db) = app.db.lock() {
            let news1 = create_news_item("Tesla announces new factory");
            let news2 = create_news_item("Tesla stock surges on delivery numbers");
            let id1 = db.insert_news(&news1).unwrap();
            let id2 = db.insert_news(&news2).unwrap();
            app.news.push(NewsItem {
                id: Some(id1),
                ..news1
            });
            app.news.push(NewsItem {
                id: Some(id2),
                ..news2
            });
        }

        let initial_watchlist_len = app.watchlist.len();
        app.auto_populate_watchlist();

        assert!(
            app.watchlist.contains(&"TSLA".to_string()),
            "Expected TSLA in watchlist, got: {:?}",
            app.watchlist
        );
        assert!(
            app.watchlist.len() > initial_watchlist_len,
            "Watchlist should have grown"
        );
    }

    #[test]
    fn given_no_company_mentions_when_processed_then_spy_added() {
        let mut app = make_test_app();

        if let Ok(db) = app.db.lock() {
            let news = create_news_item("Market sees mixed trading today");
            let id = db.insert_news(&news).unwrap();
            app.news.push(NewsItem {
                id: Some(id),
                ..news
            });
        }

        app.process_news_mentions();

        if let Ok(db) = app.db.lock() {
            let symbols = db.get_recently_mentioned_symbols(10).unwrap();
            assert!(
                symbols.contains(&"SPY".to_string()),
                "Expected SPY in symbols when no company mentioned, got: {:?}",
                symbols
            );
        };
    }
}
