//! Location extraction from news headlines

/// Known US cities for location extraction
const US_CITIES: &[(&str, &str)] = &[
    ("New York", "NY"),
    ("Los Angeles", "CA"),
    ("Chicago", "IL"),
    ("Houston", "TX"),
    ("Phoenix", "AZ"),
    ("Philadelphia", "PA"),
    ("San Antonio", "TX"),
    ("San Diego", "CA"),
    ("Dallas", "TX"),
    ("San Jose", "CA"),
    ("Austin", "TX"),
    ("Jacksonville", "FL"),
    ("San Francisco", "CA"),
    ("Seattle", "WA"),
    ("Denver", "CO"),
    ("Washington", "DC"),
    ("Boston", "MA"),
    ("Nashville", "TN"),
    ("Detroit", "MI"),
    ("Portland", "OR"),
    ("Miami", "FL"),
    ("Atlanta", "GA"),
    ("Las Vegas", "NV"),
    ("Minneapolis", "MN"),
    ("Cleveland", "OH"),
    ("Orlando", "FL"),
    ("Tampa", "FL"),
    ("Pittsburgh", "PA"),
    ("Cincinnati", "OH"),
    ("St. Louis", "MO"),
    ("Baltimore", "MD"),
    ("Kansas City", "MO"),
    ("Charlotte", "NC"),
    ("Raleigh", "NC"),
    ("Salt Lake City", "UT"),
    ("Sacramento", "CA"),
    ("Milwaukee", "WI"),
    ("Buffalo", "NY"),
    ("Richmond", "VA"),
    ("Louisville", "KY"),
    ("New Orleans", "LA"),
    ("Oklahoma City", "OK"),
    ("Hartford", "CT"),
    ("Memphis", "TN"),
    ("Honolulu", "HI"),
    ("Anchorage", "AK"),
];

/// Extract location from text (headline or description)
///
/// Returns the first matching city name, or None if no location found.
pub fn extract_location(text: &str) -> Option<String> {
    let text_lower = text.to_lowercase();

    for (city, state) in US_CITIES {
        if text_lower.contains(&city.to_lowercase()) {
            return Some(format!("{}, {}", city, state));
        }
    }

    None
}

/// Extract location from a news item (tries headline first, then description)
pub fn extract_location_from_news(headline: &str, description: Option<&str>) -> Option<String> {
    // Try headline first
    if let Some(loc) = extract_location(headline) {
        return Some(loc);
    }

    // Fall back to description
    if let Some(desc) = description {
        return extract_location(desc);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_headline_with_city_when_extracted_then_returns_city_state() {
        let headline = "Breaking: Major fire reported in San Francisco downtown";
        let result = extract_location(headline);
        assert_eq!(result, Some("San Francisco, CA".to_string()));
    }

    #[test]
    fn given_headline_without_city_when_extracted_then_returns_none() {
        let headline = "Global markets rally on economic news";
        let result = extract_location(headline);
        assert!(result.is_none());
    }

    #[test]
    fn given_headline_with_new_york_when_extracted_then_returns_ny() {
        let headline = "New York Mayor announces new transit plan";
        let result = extract_location(headline);
        assert_eq!(result, Some("New York, NY".to_string()));
    }

    #[test]
    fn given_lowercase_city_when_extracted_then_still_matches() {
        let headline = "Storm hits chicago area causing delays";
        let result = extract_location(headline);
        assert_eq!(result, Some("Chicago, IL".to_string()));
    }

    #[test]
    fn given_news_item_with_location_in_description_when_extracted_then_uses_description() {
        let headline = "Breaking news update";
        let description = Some("Officials in Miami report flooding");
        let result = extract_location_from_news(headline, description);
        assert_eq!(result, Some("Miami, FL".to_string()));
    }

    #[test]
    fn given_news_item_with_location_in_both_when_extracted_then_uses_headline() {
        let headline = "Seattle tech company announces layoffs";
        let description = Some("The company, based in Miami, is restructuring");
        let result = extract_location_from_news(headline, description);
        assert_eq!(result, Some("Seattle, WA".to_string()));
    }
}
