//! External API clients

pub mod company_mapping;
pub mod location;
pub mod news;
pub mod stocks;
pub mod weather;

use thiserror::Error;

/// Error type for API key validation
#[derive(Debug, Error)]
pub enum ApiKeyError {
    #[error("API key cannot be empty")]
    Empty,
    #[error("API key contains invalid characters")]
    InvalidCharacters,
}

/// A validated API key wrapper
///
/// Provides type safety and validation for API keys used across
/// all API clients (news, weather, stocks).
#[derive(Debug, Clone)]
pub struct ApiKey(String);

impl ApiKey {
    /// Create a new ApiKey from a string, validating it
    pub fn new(key: String) -> Result<Self, ApiKeyError> {
        let key = key.trim().to_string();

        if key.is_empty() {
            return Err(ApiKeyError::Empty);
        }

        // Basic validation: alphanumeric, hyphens, and underscores only
        if !key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ApiKeyError::InvalidCharacters);
        }

        Ok(Self(key))
    }

    /// Create an ApiKey without validation (for trusted sources like env vars)
    pub fn from_trusted(key: String) -> Self {
        Self(key)
    }

    /// Get the key as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if the key appears to be a placeholder/test key
    pub fn is_placeholder(&self) -> bool {
        let lower = self.0.to_lowercase();
        lower.contains("your-key")
            || lower.contains("placeholder")
            || lower == "test"
            || lower == "demo"
    }
}

impl AsRef<str> for ApiKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Mask the key for security in logs/display
        if self.0.len() > 8 {
            write!(f, "{}...{}", &self.0[..4], &self.0[self.0.len() - 4..])
        } else {
            write!(f, "****")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_api_key() {
        let key = ApiKey::new("abc123-def456".to_string());
        assert!(key.is_ok());
        assert_eq!(key.unwrap().as_str(), "abc123-def456");
    }

    #[test]
    fn test_empty_api_key() {
        let key = ApiKey::new("".to_string());
        assert!(matches!(key, Err(ApiKeyError::Empty)));
    }

    #[test]
    fn test_whitespace_only_key() {
        let key = ApiKey::new("   ".to_string());
        assert!(matches!(key, Err(ApiKeyError::Empty)));
    }

    #[test]
    fn test_placeholder_detection() {
        let key = ApiKey::from_trusted("your-key-here".to_string());
        assert!(key.is_placeholder());

        let real = ApiKey::from_trusted("abc123def456".to_string());
        assert!(!real.is_placeholder());
    }

    #[test]
    fn test_display_masks_key() {
        let key = ApiKey::from_trusted("1234567890abcdef".to_string());
        let display = format!("{}", key);
        assert!(!display.contains("567890abc"));
        assert!(display.contains("..."));
    }
}
