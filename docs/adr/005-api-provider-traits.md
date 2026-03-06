# ADR-005: API Provider Traits for Dependency Inversion

## Status
Accepted

## Context
Concrete API clients (NewsApi, NWS, Tiingo) make real HTTP calls, which makes unit
testing slow, flaky, and dependent on API keys. The fetch and app crates should not
be tightly coupled to specific providers.

## Decision
Define async traits in the api crate:

- `NewsProvider` -- fetches headline articles
- `WeatherProvider` -- fetches forecasts and alerts
- `StocksProvider` -- fetches quotes and metadata

Concrete clients implement these traits. Orchestration code depends on trait objects
or generics rather than concrete types.

## Consequences
### Positive
- Unit tests can substitute mock implementations with predictable responses
- Satisfies the Dependency Inversion Principle; high-level modules depend on abstractions
- Swapping a provider (e.g., replacing NewsAPI with a different source) requires only
  a new trait implementation

### Negative
- Async traits require `async-trait` or `impl Future` return types, adding some syntax
  overhead
- Trait objects introduce dynamic dispatch; negligible cost for network-bound operations

### Neutral
- Provider traits also serve as living documentation of each API's expected contract
