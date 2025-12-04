//! Cache management for data freshness

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

use super::db::Database;

/// Cache staleness thresholds
pub struct CacheConfig {
    /// News considered stale after this duration
    pub news_max_age: Duration,
    /// Weather considered stale after this duration
    pub weather_max_age: Duration,
    /// Stocks considered stale after this duration
    pub stocks_max_age: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            news_max_age: Duration::hours(1),
            weather_max_age: Duration::minutes(30),
            stocks_max_age: Duration::minutes(15),
        }
    }
}

/// Check if a timestamp is stale based on max age
pub fn is_stale(fetched_at: DateTime<Utc>, max_age: Duration) -> bool {
    Utc::now() - fetched_at > max_age
}

/// Format relative time for display
pub fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now - dt;

    if diff < Duration::minutes(1) {
        "just now".to_string()
    } else if diff < Duration::hours(1) {
        let mins = diff.num_minutes();
        format!("{}m ago", mins)
    } else if diff < Duration::hours(24) {
        let hours = diff.num_hours();
        format!("{}h ago", hours)
    } else {
        let days = diff.num_days();
        format!("{}d ago", days)
    }
}

/// Prune old data from database (keep last 7 days)
pub fn prune_old_data(db: &Database) -> Result<()> {
    // Note: This would need additional methods on Database
    // For now, we'll just clear news older than 7 days via direct SQL
    // This is a placeholder for future implementation
    let _ = db;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stale() {
        let now = Utc::now();
        let old = now - Duration::hours(2);
        let recent = now - Duration::minutes(5);

        assert!(is_stale(old, Duration::hours(1)));
        assert!(!is_stale(recent, Duration::hours(1)));
    }

    #[test]
    fn test_format_relative_time() {
        let now = Utc::now();

        assert_eq!(format_relative_time(now), "just now");
        assert_eq!(format_relative_time(now - Duration::minutes(5)), "5m ago");
        assert_eq!(format_relative_time(now - Duration::hours(2)), "2h ago");
        assert_eq!(format_relative_time(now - Duration::days(3)), "3d ago");
    }
}
