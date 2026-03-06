# The Daily Update - Makefile
# ============================
# Rust TUI application for news, weather, and stock aggregation
# Uses Cargo workspace for modular compilation

.PHONY: help lint test test-unit test-integration test-verbose test-crate \
        build release clean check fmt fmt-check run info all precommit ci \
        validate-makefile install

# Default target
.DEFAULT_GOAL := help

# Colors for output (if terminal supports it)
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

##@ General

help: ## Display this help message
	@awk 'BEGIN {FS = ":.*##"; printf "\n$(CYAN)Usage:$(RESET)\n  make $(GREEN)<target>$(RESET)\n"} \
		/^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-20s$(RESET) %s\n", $$1, $$2 } \
		/^##@/ { printf "\n$(YELLOW)%s$(RESET)\n", substr($$0, 5) }' $(MAKEFILE_LIST)

##@ Development

fmt: ## Format code using rustfmt (all workspace members)
	cargo fmt --all

fmt-check: ## Check formatting without applying changes (for CI)
	cargo fmt --all -- --check

check: ## Run cargo check for fast compilation validation
	cargo check --workspace

lint: ## Run clippy for linting with all warnings as errors
	cargo clippy --workspace --all-targets -- -D warnings

run: ## Run the application
	cargo run --package daily-update

##@ Testing

test: test-unit test-integration ## Run all tests (unit and integration)

test-unit: ## Run unit tests only (all workspace members)
	cargo test --workspace --lib

test-integration: ## Run integration tests only
	cargo test --workspace --test '*'

test-verbose: ## Run all tests with verbose output
	cargo test --workspace -- --nocapture

test-crate: ## Run tests for a specific crate (usage: make test-crate CRATE=data)
ifndef CRATE
	$(error CRATE is required. Usage: make test-crate CRATE=<crate-name>)
endif
	cargo test --package $(CRATE)

##@ Build

build: ## Build the project in debug mode
	cargo build --workspace

release: ## Build the project in release mode
	cargo build --workspace --release

install: release ## Install the release binary via cargo
	cargo install --path crates/daily-update

clean: ## Clean build artifacts
	cargo clean

##@ Demo & Features

info: ## Show feature keybindings for demos
	@echo "$(CYAN)The Daily Update - Feature Demo$(RESET)"
	@echo ""
	@echo "$(YELLOW)Date Navigation:$(RESET)"
	@echo "  $(GREEN)←/→$(RESET) or $(GREEN)h/l$(RESET)  Navigate dates (past 7 days)"
	@echo "  $(GREEN)d$(RESET)            Jump to today"
	@echo ""
	@echo "$(YELLOW)Navigation:$(RESET)"
	@echo "  $(GREEN)↑/↓$(RESET) or $(GREEN)j/k$(RESET)  Navigate news headlines"
	@echo "  $(GREEN)Tab$(RESET)          Switch panels"
	@echo ""
	@echo "$(YELLOW)Actions:$(RESET)"
	@echo "  $(GREEN)r$(RESET)            Refresh data"
	@echo "  $(GREEN)w$(RESET)            Toggle weather"
	@echo "  $(GREEN)s$(RESET)            Stock browser"
	@echo "  $(GREEN)?$(RESET)            Help overlay"
	@echo "  $(GREEN)q$(RESET)            Quit"

##@ Quality Assurance

all: ## Run full quality pipeline (format, lint, test, build)
	$(MAKE) fmt
	$(MAKE) lint
	$(MAKE) test
	$(MAKE) build

ci: fmt-check lint test build ## Full CI pipeline (non-mutating format check)
	@echo "$(GREEN)CI checks passed!$(RESET)"

precommit: lint test ## Run pre-commit checks (lint and test)
	@echo "$(GREEN)Pre-commit checks passed!$(RESET)"

validate-makefile: ## Validate Makefile syntax
	@$(MAKE) -n help > /dev/null 2>&1 && echo "$(GREEN)Makefile syntax is valid$(RESET)" || (echo "$(YELLOW)Makefile syntax error$(RESET)" && exit 1)
