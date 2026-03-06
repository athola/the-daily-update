# ADR-001: Workspace Crate Decomposition

## Status
Accepted

## Context
The Daily Update is a TUI application that aggregates news, weather, and stock data.
A monolithic single-crate design would couple presentation, networking, persistence,
and configuration concerns together, making the codebase harder to test and evolve.

## Decision
Decompose the project into a Cargo workspace with 7 crates arranged as a directed
acyclic graph:

- **config** (leaf) -- settings, setup wizard, environment handling
- **data** (leaf) -- SQLite schema, cache logic, persistence types
- **api** (depends on data) -- provider traits and concrete API clients
- **fetch** (depends on api, data, config) -- background orchestration of API calls
- **app** (depends on fetch, data, config) -- application state machine
- **ui** (depends on app) -- Ratatui-based TUI rendering
- **daily-update** (binary, depends on ui, app) -- entry point

## Consequences
### Positive
- Clean DAG ensures no circular dependencies
- Independent crates compile in parallel, improving build times
- Clear ownership boundaries make code review and refactoring easier
- Leaf crates (config, data) can be tested in isolation

### Negative
- Seven `Cargo.toml` files to maintain; dependency version bumps touch multiple files
- New contributors must understand the crate graph before making cross-cutting changes

### Neutral
- Workspace-level `Cargo.lock` ensures reproducible builds across all crates
