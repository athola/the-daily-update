# ADR-006: Concurrent API Fetching with tokio::join!

## Status
Accepted

## Context
The application fetches data from three API sources (news, weather, stocks). Calling
them sequentially serializes network latency, resulting in a total wait time equal to
the sum of all three round trips.

## Decision
Fetch news first, as it may influence weather location selection. Then fetch weather
and stocks concurrently using `tokio::join!`:

```
news  ---|
         +---> weather ---|
                          +---> done
         +---> stocks  ---|
```

## Consequences
### Positive
- Total fetch latency is reduced to `latency(news) + max(latency(weather), latency(stocks))`
  instead of the sum of all three
- `tokio::join!` is straightforward and does not require manual task management

### Negative
- News must complete before weather and stocks begin, creating a sequential dependency
- If one concurrent branch fails, the other still runs to completion (wasted work in
  some failure scenarios)

### Neutral
- The dependency on news completing first is a domain requirement, not an architectural
  limitation
