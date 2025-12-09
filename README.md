# The Daily Update

[![Built with Ratatui](https://img.shields.io/badge/Built_With-Ratatui-blue?logo=rust)](https://ratatui.rs/)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

A terminal-based daily news aggregator that displays news headlines alongside weather and stock market data. Built with Rust for performance and reliability.

## Features

- **Instant startup** with cached data from previous sessions
- **Background refresh** fetches fresh data asynchronously on startup
- **News headlines** from NewsAPI with source and relative timestamps
- **Stock watchlist** with customizable symbols via modal browser
- **Weather widget** (collapsible) with temperature, conditions, and alerts
- **Offline capable** - shows cached data when network unavailable
- **Graceful degradation** - each panel handles errors independently

## Installation

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- API keys from:
  - [NewsAPI](https://newsapi.org/) (free tier: 100 requests/day)
  - [OpenWeatherMap](https://openweathermap.org/api) (free tier: 60 requests/minute)
  - [Tiingo](https://www.tiingo.com/) (free tier: 500 requests/hour)

### Build from Source

```bash
git clone https://github.com/yourusername/the-daily-update.git
cd the-daily-update
cargo build --release
```

The binary will be at `target/release/daily-update`.

## Configuration

### API Keys (Required)

Set your API keys via environment variables (recommended):

```bash
export NEWS_API_KEY="your-newsapi-key"
export WEATHER_API_KEY="your-openweathermap-key"
export TIINGO_API_KEY="your-tiingo-key"
```

Add these to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) for persistence.

### First Run

On first launch, the app will:
1. Check for API keys in environment variables
2. Prompt for any missing keys
3. Ask for your default location (for weather)
4. Display security best practices

### Config File

Preferences are stored in `~/.config/daily-update/config.toml`:

```toml
[general]
default_location = "New York, NY"
vim_mode = false

[ui]
theme = "dark"
```

### Data Storage

- Database: `~/.local/share/daily-update/data.db`
- Config: `~/.config/daily-update/config.toml`

## Usage

```bash
./target/release/daily-update
```

Or after adding to PATH:
```bash
daily-update
```

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `Up/Down` | Navigate within panel |
| `Tab` | Switch to next panel |
| `Shift+Tab` | Switch to previous panel |

### Actions

| Key | Action |
|-----|--------|
| `r` | Refresh all data |
| `w` | Toggle weather widget expand/collapse |
| `s` | Open stock browser modal |
| `?` | Show help overlay |
| `q` | Quit application |

### Stock Browser Modal

| Key | Action |
|-----|--------|
| `Up/Down` | Navigate stocks |
| `Space` | Toggle stock in watchlist |
| `Enter` | Save changes |
| `Esc` | Cancel without saving |
| Type | Filter stocks by symbol |

### Vim Mode (Optional)

Enable in config with `vim_mode = true`:

| Key | Action |
|-----|--------|
| `j/k` | Navigate within panel |
| `h/l` | Switch panels |
| `g` | Go to top |
| `G` | Go to bottom |

## Architecture

```
the-daily-update/
├── crates/
│   ├── daily-update/   # Main binary
│   ├── app/            # Application state
│   ├── ui/             # TUI components (ratatui)
│   ├── data/           # Models, SQLite database, cache
│   ├── api/            # API clients (news, weather, stocks)
│   ├── config/         # Configuration and setup wizard
│   └── fetch/          # Background data fetcher
└── docs/plans/         # Design documents
```

The project uses a Cargo workspace for modular compilation. Each crate compiles independently, reducing rebuild times during development.

## Development

### Prerequisites

- Rust 1.70+
- (Optional) [pre-commit](https://pre-commit.com/) for automated checks

### Makefile Targets

Run `make help` to see all available targets:

```bash
make fmt          # Format code
make lint         # Run clippy with strict warnings
make test         # Run all tests (unit and integration)
make test-unit    # Run unit tests only
make build        # Build in debug mode
make release      # Build in release mode
make run          # Run the application
make precommit    # Run lint and test (CI validation)
```

### Running Tests

Unit tests cover each crate's internal logic:

```bash
make test-unit
```

Integration tests verify cross-crate workflows:

```bash
make test-integration
```

Run both with:

```bash
make test
```

### Pre-commit Hooks

Install pre-commit hooks to run validation before each commit:

```bash
pre-commit install
```

The hooks run rustfmt, clippy, and tests automatically.

## License

MIT
