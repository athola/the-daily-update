# ADR-002: Synchronous SQLite with Arc<Mutex<Database>>

## Status
Accepted

## Context
The application needs shared database access from both the synchronous main TUI loop
and asynchronous background fetch tasks. Async SQLite wrappers exist but add complexity
and an extra dependency. rusqlite is mature, well-documented, and synchronous.

## Decision
Use `rusqlite::Connection` wrapped in `Arc<std::sync::Mutex<...>>` as the shared
database handle. Async code accesses the database through `tokio::task::spawn_blocking`
to avoid blocking the Tokio runtime.

## Consequences
### Positive
- Simple mental model: one mutex, one connection, no async state machines for DB access
- No dependency on async SQLite crates (e.g., sqlx, tokio-rusqlite)
- Synchronous TUI code can lock the mutex directly without an async runtime

### Negative
- Every async DB call requires `spawn_blocking`, adding minor boilerplate
- A long-running DB operation holds the mutex, blocking other accessors
- `std::sync::Mutex` cannot be held across `.await` points; must be careful in async code

### Neutral
- SQLite itself serializes writes internally, so a single-connection model is acceptable
  for this workload
