# ADR-003: Background Fetching via tokio::spawn and mpsc Channels

## Status
Accepted

## Context
The TUI must remain responsive while data is fetched from remote APIs. Performing
HTTP requests on the main thread would freeze the interface for seconds at a time.

## Decision
Spawn background tasks with `tokio::spawn` to perform API fetches. Results (or errors)
are sent back to the main loop through a `tokio::sync::mpsc` channel. The main loop
calls `try_recv` on each tick to process any available messages without blocking.

## Consequences
### Positive
- TUI rendering never blocks on network I/O
- Clean producer/consumer separation between fetch logic and state management
- Natural backpressure: bounded channel capacity prevents unbounded memory growth
- Multiple fetches can run concurrently without additional synchronization

### Negative
- Adds indirection; data flows through a channel rather than being returned directly
- Error handling is deferred to the receiver side, making the control flow less obvious

### Neutral
- The mpsc pattern is idiomatic Tokio and well-understood by Rust async developers
