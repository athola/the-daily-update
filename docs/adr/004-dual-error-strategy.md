# ADR-004: Dual Error Strategy with thiserror and anyhow

## Status
Accepted

## Context
Leaf crates expose public APIs where callers benefit from typed, matchable errors.
Orchestration crates mostly propagate errors upward and present them to the user,
where ergonomic error chaining matters more than exhaustive matching.

## Decision
- **API crate**: Use `thiserror` to define domain-specific error enums (e.g.,
  `NewsApiError`, `StocksApiError`, `WeatherApiError`) so callers can pattern-match.
- **All other crates** (data, config, app, fetch, daily-update): Use `anyhow::Result`
  for ergonomic error propagation with `.context()`.

## Consequences
### Positive
- API consumers can pattern-match on specific error variants for recovery logic
- Internal code avoids boilerplate `From` impls; `anyhow` handles conversion automatically
- Error messages carry contextual information through the `anyhow` chain

### Negative
- Two error libraries in the dependency tree; contributors must know which to use where
- Crossing the boundary (thiserror -> anyhow) is seamless, but the reverse requires
  downcasting

### Neutral
- This split aligns with the Rust ecosystem convention for library-vs-application error
  handling
