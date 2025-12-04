# The Daily Update - Design Document

## 1. Overview & Architecture

### Purpose
A terminal-based daily news aggregator that displays news headlines alongside weather and stock market data. Built with Rust for performance and reliability.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Terminal UI (ratatui)                │
├─────────────────────────────────────────────────────────┤
│                    Application Layer                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ News Module │  │Stock Module │  │ Weather Module  │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────┤
│                    Data Layer                           │
│  ┌─────────────────────┐  ┌─────────────────────────┐   │
│  │   SQLite (rusqlite) │  │  Background Fetcher     │   │
│  └─────────────────────┘  └─────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│                    External APIs                        │
│     NewsAPI    │    OpenWeatherMap    │    Tiingo       │
└─────────────────────────────────────────────────────────┘
```

### Key Principles
- **Instant startup:** UI loads immediately with cached data
- **Background refresh:** Fresh data fetched asynchronously on startup
- **Graceful degradation:** Each panel handles its own errors independently
- **Offline capable:** Shows cached data when network unavailable

### Rust Crates (0.1.0)
| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI framework |
| `tokio` | Async runtime for background fetching |
| `rusqlite` | SQLite bindings |
| `reqwest` | HTTP client for APIs |
| `serde` / `serde_json` | JSON serialization |
| `toml` | Config file parsing |
| `directories` | XDG-compliant paths |
| `chrono` | Date/time handling |
| `crossterm` | Terminal backend for ratatui |

---

## 2. Data Model & Database

### SQLite Schema

```sql
-- News headlines from NewsAPI
CREATE TABLE news (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    headline TEXT NOT NULL,
    source TEXT,
    description TEXT,
    url TEXT,
    published_at DATETIME NOT NULL,
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Weather data from OpenWeatherMap
CREATE TABLE weather (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    location TEXT NOT NULL,
    temperature REAL,
    condition TEXT,
    humidity INTEGER,
    wind_speed REAL,
    alert_title TEXT,
    alert_description TEXT,
    alert_severity TEXT,
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Stock/index data from Tiingo
CREATE TABLE stocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    name TEXT,
    price REAL,
    change_percent REAL,
    fetched_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- User's watchlist (persisted across sessions)
CREATE TABLE watchlist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT UNIQUE NOT NULL,
    display_order INTEGER DEFAULT 0
);
```

### Data Flow
1. App starts → Load cached data from SQLite → Render UI immediately
2. Background task fetches fresh data from APIs
3. New data written to SQLite tables
4. UI receives notification → Re-renders with fresh data

### Storage Locations
- **Database:** `~/.local/share/daily-update/data.db`
- **Config:** `~/.config/daily-update/config.toml`

### Cache Management
- Data persists across sessions
- Old entries pruned on startup (keep last 7 days)
- `fetched_at` timestamp enables staleness indicators

---

## 3. UI Layout & Components

### Main Layout

```
┌─────────────────────────────────────────────────────────┐
│  NEWS                                    [Refreshing...] │
│─────────────────────────────────────────────────────────│
│  • Tesla announces new factory plans      (Reuters, 2h) │
│  • Fed signals rate decision coming       (WSJ, 3h)     │
│  • Tech stocks rally on earnings          (CNBC, 4h)    │
│  • Climate summit reaches agreement       (AP, 5h)      │
│  • Healthcare bill advances in Senate     (NYT, 6h)     │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  STOCKS                              Last updated: 10m  │
│─────────────────────────────────────────────────────────│
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐        │
│  │ S&P 500     │ │ NASDAQ      │ │ DOW         │        │
│  │ 4,567.89    │ │ 14,234.56   │ │ 34,567.12   │        │
│  │ +1.2%  ▲    │ │ +0.8%  ▲    │ │ -0.3%  ▼    │        │
│  └─────────────┘ └─────────────┘ └─────────────┘        │
│                                                         │
│  ┌──────────────────┐                                   │
│  │ ☀ NYC 72°F  [w]  │                                   │
│  └──────────────────┘                                   │
└─────────────────────────────────────────────────────────┘
  [↑↓] Navigate  [r] Refresh  [s] Stocks  [w] Weather  [?] Help  [q] Quit
```

### Components

**News Panel (Top)**
- Scrollable list of headlines
- Format: `• {headline} ({source}, {relative_time})`
- Selected item highlighted
- Header shows refresh status

**Stocks Panel (Middle)**
- Fixed subpanels for each watchlist item
- Each shows: symbol, price, percent change, direction indicator
- "Last updated" timestamp

**Weather Widget (Floating, Bottom-Left)**
- Collapsed: `☀ {location} {temp}°F [w]`
- Shows alert indicator if active: `⚠ Winter Storm Warning`
- Expanded (press `w`): Full-width panel with humidity, wind, conditions, full alert details
- Fixed position, overlays stock panel

**Status Bar (Bottom)**
- Keybind hints
- Context-sensitive based on active panel

### Stock Browser Modal Pane

Overlays the UI as a centered modal. Dismissed via Enter (submit changes) or Esc (cancel without saving).

```
┌─────────────── Add to Watchlist ───────────────┐
│  Search: AAPL_                                 │
│                                                │
│  [x] SPY    - S&P 500 ETF                     │
│  [x] QQQ    - NASDAQ ETF                      │
│  [ ] AAPL   - Apple Inc.                      │
│  [ ] GOOGL  - Alphabet Inc.                   │
│  [ ] MSFT   - Microsoft Corp.                 │
│                                                │
│  [↑↓] Navigate  [Space] Toggle                │
│  [Enter] Save   [Esc] Cancel                  │
└────────────────────────────────────────────────┘
```

---

## 4. Configuration

### First-Run Flow
1. App starts, checks for existing config
2. If none: Check environment for API keys
3. If keys found: Import them, prompt for preferences
4. If keys missing: Interactive prompts to enter them
5. Write `config.toml` with preferences only (keys via env vars)
6. Display security best practices

### Config File (`~/.config/daily-update/config.toml`)

```toml
# The Daily Update Configuration

[general]
# Default location for weather (used when news doesn't specify)
default_location = "New York, NY"

# Enable vim-style keybindings (j/k/h/l navigation)
vim_mode = false

[ui]
# Theme (future: light, dark, system)
theme = "dark"

[apis]
# API keys: Recommended to set via environment variables:
#   export NEWS_API_KEY="your-key"
#   export WEATHER_API_KEY="your-key"
#   export TIINGO_API_KEY="your-key"
#
# Storing keys in this file is NOT recommended for shared configs.
# Uncomment below only for personal, non-shared configurations:
# news_api_key = ""
# weather_api_key = ""
# tiingo_api_key = ""
```

### Environment Variables
| Variable | Purpose |
|----------|---------|
| `NEWS_API_KEY` | NewsAPI access |
| `WEATHER_API_KEY` | OpenWeatherMap access |
| `TIINGO_API_KEY` | Tiingo stock data access |

### Security Guidance (Shown at First Run)
```
╭─────────────────── API Key Security ───────────────────╮
│                                                        │
│  Your API keys grant access to paid services.          │
│  Keep them secure:                                     │
│                                                        │
│  ✓ Use environment variables (recommended)             │
│    export NEWS_API_KEY="your-key"                      │
│                                                        │
│  ✓ Or use a .env file (add to .gitignore)             │
│                                                        │
│  ✗ Avoid committing keys to version control            │
│  ✗ Don't share config files containing keys            │
│                                                        │
╰────────────────────────────────────────────────────────╯
```

---

## 5. Error Handling & Offline Mode

### Per-Panel Error States

Each panel handles failures independently:

```
┌─────────────────────────────────────────────────────────┐
│  NEWS                                                   │
│─────────────────────────────────────────────────────────│
│  ⚠ Unable to fetch news - showing cached data (2h old)  │
│                                                         │
│  • Tesla announces new factory plans      (Reuters, 2h) │
│  ...                                                    │
├─────────────────────────────────────────────────────────┤
│  STOCKS                                    ⚠ Stale      │
│─────────────────────────────────────────────────────────│
│  ┌─────────────┐ ┌─────────────┐                        │
│  │ S&P 500     │ │ ⚠ Error     │  ← Individual failure  │
│  │ 4,567.89    │ │ NASDAQ      │                        │
│  └─────────────┘ └─────────────┘                        │
```

### Offline Detection

```
┌─────────────────────────────────────────────────────────┐
│  ⚡ OFFLINE - Showing cached data                       │
│─────────────────────────────────────────────────────────│
```

- Detect network unavailability on fetch failure
- Global banner when offline
- All panels show cached data with timestamps
- Auto-recover when network returns

### Error Types & Responses

| Error | Response |
|-------|----------|
| Network unreachable | Show offline banner, use cache |
| API rate limited | Show warning, use cache |
| API key invalid | Show setup prompt, disable panel |
| No cached data | Show empty state with instructions |
| Parse error | Log error, show "Data unavailable" |

---

## 6. Keybindings

### Default Mode (Arrow Keys)

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate within panel |
| `Tab` | Switch to next panel |
| `Shift+Tab` | Switch to previous panel |
| `Enter` | Select / Confirm / Save (in modal) |
| `Space` | Toggle (in stock browser) |
| `r` | Manual refresh all data |
| `w` | Toggle weather widget expand/collapse |
| `s` | Open stock browser modal |
| `?` | Show help overlay |
| `q` | Quit application |
| `Esc` | Close modal / Cancel without saving |

### Vim Mode (Optional)

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate within panel |
| `h` / `l` | Switch panels |
| `g` | Go to top of list |
| `G` | Go to bottom of list |
| `/` | Search (future) |

### Context-Sensitive Keys

**In Stock Browser Modal:**
- `Space` toggles watchlist membership
- `Enter` saves changes and closes
- `Esc` cancels without saving
- Typing filters the list

**Weather Expanded:**
- `w` or `Esc` collapses back to widget

---

## 7. Project Structure

```
the-daily-update/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI args
│   ├── app.rs               # Application state & event loop
│   ├── ui/
│   │   ├── mod.rs           # UI module exports
│   │   ├── layout.rs        # Main layout composition
│   │   ├── news_panel.rs    # News headlines component
│   │   ├── stocks_panel.rs  # Stocks display component
│   │   ├── weather_widget.rs # Floating weather widget
│   │   ├── stock_browser.rs # Watchlist management modal
│   │   └── help_overlay.rs  # Help screen
│   ├── data/
│   │   ├── mod.rs           # Data module exports
│   │   ├── db.rs            # SQLite operations
│   │   ├── models.rs        # Data structures
│   │   └── cache.rs         # Cache management
│   ├── api/
│   │   ├── mod.rs           # API module exports
│   │   ├── news.rs          # NewsAPI client
│   │   ├── weather.rs       # OpenWeatherMap client
│   │   └── stocks.rs        # Tiingo client
│   ├── config/
│   │   ├── mod.rs           # Config module exports
│   │   ├── settings.rs      # Config structs
│   │   └── setup.rs         # First-run wizard
│   └── fetch/
│       ├── mod.rs           # Fetch module exports
│       └── background.rs    # Background refresh task
├── docs/
│   └── plans/
│       └── 2024-12-04-daily-update-design.md
└── README.md
```

---

## 8. Roadmap

### Version 0.1.0 (MVP)
- [ ] SQLite with file persistence
- [ ] Instant startup with cached data
- [ ] Background fetch on startup
- [ ] Manual refresh command (`r`)
- [ ] Interactive TUI with horizontal split layout
- [ ] News panel: headline + source + relative time
- [ ] Stocks panel with watchlist subpanels
- [ ] Stock browser modal for watchlist management
- [ ] Floating weather widget (collapsible) with alert support
- [ ] Arrow-key navigation (default)
- [ ] Optional vim mode
- [ ] Config file + env var support
- [ ] First-run setup wizard with security guidance
- [ ] Graceful degradation per panel
- [ ] Offline mode with cached data
- [ ] Default location + S&P 500 fallback

### Version 0.2.0
- [ ] Date cycling (Today → Yesterday → ...)
- [ ] Preset date filters (Today / Yesterday / Past 3 days / Week)
- [ ] Expandable news details (description, open in browser)
- [ ] Audit binary size, evaluate smaller crates
- [ ] Resizable panes via keybinds

### Version 0.3.0
- [ ] Full date picker UI
- [ ] Movable/resizable stock subpanels

### Future
- [ ] LLM-assisted entity extraction via MCP (Claude + user's Anthropic key)
- [ ] Retry with exponential backoff
- [ ] React frontend
- [ ] Live update dashboard mode (auto-refresh display)

---

## 9. APIs Reference

### NewsAPI
- **Endpoint:** `https://newsapi.org/v2/top-headlines`
- **Params:** `country=us`, `pageSize=10`
- **Rate limit:** 100 requests/day (free tier)

### OpenWeatherMap
- **Endpoint:** `https://api.openweathermap.org/data/2.5/weather`
- **One Call API for alerts:** `https://api.openweathermap.org/data/3.0/onecall`
- **Params:** `q={city}`, `units=imperial`
- **Rate limit:** 60 requests/minute (free tier)

### Tiingo
- **Endpoint:** `https://api.tiingo.com/iex`
- **Params:** `tickers={symbols}`
- **Rate limit:** 500 requests/hour (free tier)
