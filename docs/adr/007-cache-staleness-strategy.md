# ADR-007: Cache Staleness Strategy and Data Pruning

## Status
Accepted

## Context
Without staleness checks, every application launch triggers API calls regardless of
whether cached data is still fresh. Without pruning, the SQLite database grows
unboundedly over time.

## Decision
Introduce `CacheConfig` with per-domain staleness thresholds:

- **News**: 1 hour
- **Weather**: 30 minutes
- **Stocks**: 15 minutes

Before fetching, check the age of cached data. If data is within its staleness window,
serve from cache and skip the API call. On startup, run `prune_old_data` to delete
records older than 7 days.

## Consequences
### Positive
- Reduces unnecessary API calls, preserving rate-limit quota
- Bounded database size due to periodic pruning
- Faster startup when cached data is still fresh

### Negative
- Users may see slightly stale data until the next fetch cycle
- Staleness thresholds are static; changing them requires a configuration update

### Neutral
- The 7-day retention window is a reasonable default for a daily news aggregator
- Pruning on startup is simple and avoids the need for a background cleanup task
