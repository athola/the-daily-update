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

    // ============================================================
    // CacheConfig Tests
    // ============================================================

    #[test]
    fn given_default_cache_config_then_has_expected_thresholds() {
        let config = CacheConfig::default();

        assert_eq!(config.news_max_age, Duration::hours(1));
        assert_eq!(config.weather_max_age, Duration::minutes(30));
        assert_eq!(config.stocks_max_age, Duration::minutes(15));
    }

    #[test]
    fn given_custom_cache_config_then_values_preserved() {
        let config = CacheConfig {
            news_max_age: Duration::hours(2),
            weather_max_age: Duration::hours(1),
            stocks_max_age: Duration::minutes(5),
        };

        assert_eq!(config.news_max_age, Duration::hours(2));
        assert_eq!(config.weather_max_age, Duration::hours(1));
        assert_eq!(config.stocks_max_age, Duration::minutes(5));
    }

    // ============================================================
    // is_stale Tests
    // ============================================================

    #[test]
    fn given_old_timestamp_when_checked_then_is_stale() {
        let now = Utc::now();
        let old = now - Duration::hours(2);

        assert!(is_stale(old, Duration::hours(1)));
    }

    #[test]
    fn given_recent_timestamp_when_checked_then_not_stale() {
        let now = Utc::now();
        let recent = now - Duration::minutes(5);

        assert!(!is_stale(recent, Duration::hours(1)));
    }

    #[test]
    fn given_timestamp_at_boundary_when_checked_then_not_stale() {
        let now = Utc::now();
        // Exactly at the boundary (minus 1 second for safety)
        let boundary = now - Duration::hours(1) + Duration::seconds(1);

        assert!(!is_stale(boundary, Duration::hours(1)));
    }

    #[test]
    fn given_timestamp_just_past_boundary_when_checked_then_is_stale() {
        let now = Utc::now();
        // Just past the boundary
        let past_boundary = now - Duration::hours(1) - Duration::seconds(1);

        assert!(is_stale(past_boundary, Duration::hours(1)));
    }

    #[test]
    fn given_future_timestamp_when_checked_then_not_stale() {
        let now = Utc::now();
        let future = now + Duration::hours(1);

        assert!(!is_stale(future, Duration::hours(1)));
    }

    #[test]
    fn given_zero_max_age_when_checked_then_always_stale() {
        let now = Utc::now();
        let recent = now - Duration::seconds(1);

        // With zero max_age, even 1 second old is stale
        assert!(is_stale(recent, Duration::zero()));
    }

    #[test]
    fn given_current_timestamp_when_checked_then_not_stale() {
        let now = Utc::now();

        assert!(!is_stale(now, Duration::hours(1)));
    }

    // ============================================================
    // format_relative_time Tests
    // ============================================================

    #[test]
    fn given_just_now_timestamp_then_returns_just_now() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now), "just now");
    }

    #[test]
    fn given_30_seconds_ago_then_returns_just_now() {
        let now = Utc::now();
        assert_eq!(
            format_relative_time(now - Duration::seconds(30)),
            "just now"
        );
    }

    #[test]
    fn given_59_seconds_ago_then_returns_just_now() {
        let now = Utc::now();
        assert_eq!(
            format_relative_time(now - Duration::seconds(59)),
            "just now"
        );
    }

    #[test]
    fn given_1_minute_ago_then_returns_1m_ago() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now - Duration::minutes(1)), "1m ago");
    }

    #[test]
    fn given_minutes_ago_then_returns_correct_format() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now - Duration::minutes(5)), "5m ago");
        assert_eq!(format_relative_time(now - Duration::minutes(30)), "30m ago");
        assert_eq!(format_relative_time(now - Duration::minutes(59)), "59m ago");
    }

    #[test]
    fn given_1_hour_ago_then_returns_1h_ago() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now - Duration::hours(1)), "1h ago");
    }

    #[test]
    fn given_hours_ago_then_returns_correct_format() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now - Duration::hours(2)), "2h ago");
        assert_eq!(format_relative_time(now - Duration::hours(12)), "12h ago");
        assert_eq!(format_relative_time(now - Duration::hours(23)), "23h ago");
    }

    #[test]
    fn given_1_day_ago_then_returns_1d_ago() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now - Duration::days(1)), "1d ago");
    }

    #[test]
    fn given_days_ago_then_returns_correct_format() {
        let now = Utc::now();
        assert_eq!(format_relative_time(now - Duration::days(3)), "3d ago");
        assert_eq!(format_relative_time(now - Duration::days(7)), "7d ago");
        assert_eq!(format_relative_time(now - Duration::days(30)), "30d ago");
    }

    #[test]
    fn given_90_minutes_then_returns_hours_not_minutes() {
        let now = Utc::now();
        // 90 minutes should be displayed as 1h (truncated)
        assert_eq!(format_relative_time(now - Duration::minutes(90)), "1h ago");
    }

    #[test]
    fn given_25_hours_then_returns_days_not_hours() {
        let now = Utc::now();
        // 25 hours should be displayed as 1d (truncated)
        assert_eq!(format_relative_time(now - Duration::hours(25)), "1d ago");
    }

    // ============================================================
    // prune_old_data Tests (placeholder behavior)
    // ============================================================

    #[test]
    fn given_database_when_prune_called_then_no_error() {
        use super::super::db::Database;

        let db = Database::in_memory().unwrap();
        let result = prune_old_data(&db);

        assert!(result.is_ok(), "prune_old_data should not error");
    }
}
