//! Application state and main event loop

use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use config::settings::Config;
use data::db::Database;
use data::models::{AvailableStock, NewsItem, StockData, WeatherData};
use fetch::background::{BackgroundFetcher, FetchUpdate};
use api::stocks::StocksClient;

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
        let mut should_complete = false;

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
                        should_complete = true;
                    }
                }
            }
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
