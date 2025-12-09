//! Location extraction from news headlines and sources

/// Known news source headquarters locations
///
/// Maps news source names (as returned by NewsAPI) to their headquarters cities.
/// This provides more accurate location data than parsing headlines.
const SOURCE_LOCATIONS: &[(&str, &str)] = &[
    // Major US News Sources
    ("Bloomberg", "New York, NY"),
    ("Reuters", "New York, NY"),
    ("Associated Press", "New York, NY"),
    ("The New York Times", "New York, NY"),
    ("New York Post", "New York, NY"),
    ("The Wall Street Journal", "New York, NY"),
    ("NBC News", "New York, NY"),
    ("ABC News", "New York, NY"),
    ("CBS News", "New York, NY"),
    ("Fox News", "New York, NY"),
    ("CNBC", "New York, NY"),
    ("CNN", "Atlanta, GA"),
    ("The Washington Post", "Washington, DC"),
    ("Politico", "Washington, DC"),
    ("NPR", "Washington, DC"),
    ("USA Today", "Washington, DC"),
    ("Los Angeles Times", "Los Angeles, CA"),
    ("The Hollywood Reporter", "Los Angeles, CA"),
    ("Variety", "Los Angeles, CA"),
    ("San Francisco Chronicle", "San Francisco, CA"),
    ("TechCrunch", "San Francisco, CA"),
    ("Wired", "San Francisco, CA"),
    ("The Verge", "New York, NY"),
    ("Ars Technica", "San Francisco, CA"),
    ("Chicago Tribune", "Chicago, IL"),
    ("Chicago Sun-Times", "Chicago, IL"),
    ("The Boston Globe", "Boston, MA"),
    ("The Miami Herald", "Miami, FL"),
    ("The Seattle Times", "Seattle, WA"),
    ("The Denver Post", "Denver, CO"),
    ("Houston Chronicle", "Houston, TX"),
    ("The Dallas Morning News", "Dallas, TX"),
    ("Detroit Free Press", "Detroit, MI"),
    ("The Philadelphia Inquirer", "Philadelphia, PA"),
    ("The Arizona Republic", "Phoenix, AZ"),
    ("Star Tribune", "Minneapolis, MN"),
    // International Sources (major cities)
    ("BBC News", "London, UK"),
    ("BBC", "London, UK"),
    ("The Guardian", "London, UK"),
    ("Financial Times", "London, UK"),
    ("The Independent", "London, UK"),
    ("Daily Mail", "London, UK"),
    ("Sky News", "London, UK"),
    ("The Times", "London, UK"),
    ("The Telegraph", "London, UK"),
    ("Al Jazeera English", "Doha, Qatar"),
    ("Al Jazeera", "Doha, Qatar"),
    ("Deutsche Welle", "Berlin, Germany"),
    ("France 24", "Paris, France"),
    ("The Globe and Mail", "Toronto, Canada"),
    ("CBC News", "Toronto, Canada"),
    ("The Sydney Morning Herald", "Sydney, Australia"),
    ("ABC News (AU)", "Sydney, Australia"),
    ("South China Morning Post", "Hong Kong"),
    ("The Japan Times", "Tokyo, Japan"),
    ("The Times of India", "Mumbai, India"),
    ("Hindustan Times", "New Delhi, India"),
];

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

/// Look up location for a known news source by name
///
/// Returns the headquarters city for known news sources.
pub fn get_source_location(source_name: &str) -> Option<String> {
    let source_lower = source_name.to_lowercase();

    for (source, location) in SOURCE_LOCATIONS {
        if source_lower == source.to_lowercase() {
            return Some(location.to_string());
        }
    }

    None
}

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

/// Extract location from a news item using layered approach:
/// 1. Source headquarters (most reliable)
/// 2. City mentioned in headline
/// 3. City mentioned in description
pub fn extract_location_from_news(
    headline: &str,
    description: Option<&str>,
    source: Option<&str>,
) -> Option<String> {
    // Try source location first (most reliable)
    if let Some(src) = source {
        if let Some(loc) = get_source_location(src) {
            return Some(loc);
        }
    }

    // Try headline
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

    // ============================================================
    // Source Location Tests
    // ============================================================

    #[test]
    fn given_known_source_when_lookup_then_returns_location() {
        assert_eq!(
            get_source_location("Bloomberg"),
            Some("New York, NY".to_string())
        );
        assert_eq!(get_source_location("CNN"), Some("Atlanta, GA".to_string()));
        assert_eq!(
            get_source_location("BBC News"),
            Some("London, UK".to_string())
        );
    }

    #[test]
    fn given_source_with_different_case_when_lookup_then_still_matches() {
        assert_eq!(
            get_source_location("BLOOMBERG"),
            Some("New York, NY".to_string())
        );
        assert_eq!(
            get_source_location("bbc news"),
            Some("London, UK".to_string())
        );
    }

    #[test]
    fn given_unknown_source_when_lookup_then_returns_none() {
        assert!(get_source_location("Random Blog").is_none());
        assert!(get_source_location("Unknown Source").is_none());
    }

    // ============================================================
    // Text Location Extraction Tests
    // ============================================================

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

    // ============================================================
    // Combined Location Extraction Tests (with source priority)
    // ============================================================

    #[test]
    fn given_known_source_when_extracting_then_uses_source_location() {
        let headline = "Tech stocks surge on earnings reports";
        let description = Some("Markets react positively");
        let source = Some("Bloomberg");
        let result = extract_location_from_news(headline, description, source);
        assert_eq!(result, Some("New York, NY".to_string()));
    }

    #[test]
    fn given_known_source_and_city_in_headline_when_extracting_then_prefers_source() {
        // Source location takes priority over headline city
        let headline = "San Francisco startup raises funding";
        let description = None;
        let source = Some("Bloomberg");
        let result = extract_location_from_news(headline, description, source);
        assert_eq!(result, Some("New York, NY".to_string()));
    }

    #[test]
    fn given_unknown_source_and_city_in_headline_when_extracting_then_uses_headline() {
        let headline = "San Francisco startup raises funding";
        let description = None;
        let source = Some("Unknown Tech Blog");
        let result = extract_location_from_news(headline, description, source);
        assert_eq!(result, Some("San Francisco, CA".to_string()));
    }

    #[test]
    fn given_no_source_and_city_in_headline_when_extracting_then_uses_headline() {
        let headline = "Seattle tech company announces layoffs";
        let description = None;
        let result = extract_location_from_news(headline, description, None);
        assert_eq!(result, Some("Seattle, WA".to_string()));
    }

    #[test]
    fn given_news_item_with_location_in_description_when_extracted_then_uses_description() {
        let headline = "Breaking news update";
        let description = Some("Officials in Miami report flooding");
        let result = extract_location_from_news(headline, description, None);
        assert_eq!(result, Some("Miami, FL".to_string()));
    }

    #[test]
    fn given_no_source_location_in_both_when_extracted_then_uses_headline() {
        let headline = "Seattle tech company announces layoffs";
        let description = Some("The company, based in Miami, is restructuring");
        let result = extract_location_from_news(headline, description, None);
        assert_eq!(result, Some("Seattle, WA".to_string()));
    }

    #[test]
    fn given_bbc_source_when_extracting_then_returns_london() {
        let headline = "Global economy shows signs of recovery";
        let result = extract_location_from_news(headline, None, Some("BBC News"));
        assert_eq!(result, Some("London, UK".to_string()));
    }
}
