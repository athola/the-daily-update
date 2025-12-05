# The Daily Update - Makefile
# ============================
# Rust TUI application for news, weather, and stock aggregation
# Uses Cargo workspace for modular compilation

.PHONY: help lint test test-unit test-integration build clean check fmt all run

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

check: ## Run cargo check for fast compilation validation
	cargo check --workspace

lint: ## Run clippy for linting with all warnings as errors
	cargo clippy --workspace --all-targets --all-features -- -D warnings

run: ## Run the application
	cargo run --package daily-update

##@ Testing

test: test-unit test-integration ## Run all tests (unit and integration)

test-unit: ## Run unit tests only (all workspace members)
	cargo test --workspace --lib

test-integration: ## Run integration tests only
	cargo test --package daily-update --test '*'

test-verbose: ## Run all tests with verbose output
	cargo test --workspace -- --nocapture

test-crate: ## Run tests for a specific crate (usage: make test-crate CRATE=data)
	cargo test --package $(CRATE)

##@ Build

build: ## Build the project in debug mode
	cargo build --workspace

release: ## Build the project in release mode
	cargo build --workspace --release

clean: ## Clean build artifacts
	cargo clean

##@ Quality Assurance

all: fmt lint test build ## Run full quality pipeline (format, lint, test, build)

precommit: lint test ## Run pre-commit checks (lint and test)
	@echo "$(GREEN)Pre-commit checks passed!$(RESET)"

validate-makefile: ## Validate Makefile syntax
	@make -n help > /dev/null 2>&1 && echo "$(GREEN)Makefile syntax is valid$(RESET)" || (echo "$(YELLOW)Makefile syntax error$(RESET)" && exit 1)
