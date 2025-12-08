//! Company name to stock ticker mapping
//!
//! Provides a static mapping of well-known company names to their stock symbols.
//! Uses case-insensitive matching with word boundary detection.

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Static mapping of company names/keywords to stock symbols
static COMPANY_TICKERS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Major tech companies
    m.insert("apple", "AAPL");
    m.insert("iphone", "AAPL");
    m.insert("ipad", "AAPL");
    m.insert("mac", "AAPL");
    m.insert("google", "GOOGL");
    m.insert("alphabet", "GOOGL");
    m.insert("microsoft", "MSFT");
    m.insert("windows", "MSFT");
    m.insert("azure", "MSFT");
    m.insert("amazon", "AMZN");
    m.insert("aws", "AMZN");
    m.insert("meta", "META");
    m.insert("facebook", "META");
    m.insert("instagram", "META");
    m.insert("nvidia", "NVDA");
    m.insert("tesla", "TSLA");
    m.insert("netflix", "NFLX");

    // Financial
    m.insert("jpmorgan", "JPM");
    m.insert("jp morgan", "JPM");
    m.insert("goldman", "GS");
    m.insert("goldman sachs", "GS");
    m.insert("bank of america", "BAC");
    m.insert("wells fargo", "WFC");
    m.insert("visa", "V");
    m.insert("mastercard", "MA");
    m.insert("paypal", "PYPL");

    // Retail & Consumer
    m.insert("walmart", "WMT");
    m.insert("target", "TGT");
    m.insert("costco", "COST");
    m.insert("home depot", "HD");
    m.insert("starbucks", "SBUX");
    m.insert("mcdonald", "MCD");
    m.insert("coca-cola", "KO");
    m.insert("coca cola", "KO");
    m.insert("pepsi", "PEP");
    m.insert("nike", "NKE");
    m.insert("disney", "DIS");

    // Healthcare
    m.insert("pfizer", "PFE");
    m.insert("johnson & johnson", "JNJ");
    m.insert("unitedhealth", "UNH");
    m.insert("moderna", "MRNA");

    // Other major companies
    m.insert("boeing", "BA");
    m.insert("exxon", "XOM");
    m.insert("chevron", "CVX");
    m.insert("intel", "INTC");
    m.insert("amd", "AMD");
    m.insert("salesforce", "CRM");
    m.insert("oracle", "ORCL");
    m.insert("ibm", "IBM");
    m.insert("cisco", "CSCO");
    m.insert("uber", "UBER");
    m.insert("airbnb", "ABNB");
    m.insert("spotify", "SPOT");
    m.insert("zoom", "ZM");

    m
});

/// Default symbol when no companies are detected
pub const DEFAULT_SYMBOL: &str = "SPY";

/// Extract stock symbols from a headline
///
/// Returns a list of unique symbols found in the headline.
/// Returns vec!["SPY"] if no companies are detected.
pub fn extract_symbols(headline: &str) -> Vec<String> {
    let headline_lower = headline.to_lowercase();
    let mut symbols: Vec<String> = Vec::new();

    for (keyword, symbol) in COMPANY_TICKERS.iter() {
        // Check for word boundary match to avoid partial matches
        // e.g., "application" shouldn't match "apple"
        if contains_word(&headline_lower, keyword) {
            let sym = symbol.to_string();
            if !symbols.contains(&sym) {
                symbols.push(sym);
            }
        }
    }

    if symbols.is_empty() {
        vec![DEFAULT_SYMBOL.to_string()]
    } else {
        symbols
    }
}

/// Check if text contains a word (with word boundaries)
fn contains_word(text: &str, word: &str) -> bool {
    // Simple word boundary check using character inspection
    if let Some(byte_pos) = text.find(word) {
        // Convert byte position to character index for proper Unicode handling
        let char_pos = text[..byte_pos].chars().count();

        // Check character before the match
        let before_ok = char_pos == 0 || !text.chars().nth(char_pos - 1).unwrap_or(' ').is_alphanumeric();

        // Check character after the match
        let word_char_len = word.chars().count();
        let after_char_pos = char_pos + word_char_len;
        let after_ok = after_char_pos >= text.chars().count()
            || !text.chars().nth(after_char_pos).unwrap_or(' ').is_alphanumeric();

        before_ok && after_ok
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_headline_with_apple_when_extracted_then_returns_aapl() {
        let symbols = extract_symbols("Apple announces new iPhone 15");
        assert!(symbols.contains(&"AAPL".to_string()));
    }

    #[test]
    fn given_headline_with_multiple_companies_when_extracted_then_returns_all() {
        let symbols = extract_symbols("Apple and Google compete in AI race");
        assert!(symbols.contains(&"AAPL".to_string()));
        assert!(symbols.contains(&"GOOGL".to_string()));
    }

    #[test]
    fn given_headline_with_no_companies_when_extracted_then_returns_spy() {
        let symbols = extract_symbols("Stock market sees mixed trading");
        assert_eq!(symbols, vec!["SPY".to_string()]);
    }

    #[test]
    fn given_headline_with_product_name_when_extracted_then_returns_parent_company() {
        let symbols = extract_symbols("iPhone sales break records");
        assert!(symbols.contains(&"AAPL".to_string()));
    }

    #[test]
    fn given_headline_with_partial_match_when_extracted_then_no_false_positive() {
        // "application" contains "apple" but shouldn't match
        let symbols = extract_symbols("Mobile application downloads surge");
        assert_eq!(symbols, vec!["SPY".to_string()]);
    }

    #[test]
    fn given_case_insensitive_when_extracted_then_matches() {
        let symbols = extract_symbols("NVIDIA GPUs dominate AI market");
        assert!(symbols.contains(&"NVDA".to_string()));
    }

    #[test]
    fn given_duplicate_mentions_when_extracted_then_unique_list() {
        let symbols = extract_symbols("Apple iPhone and Apple Mac both updated");
        assert_eq!(symbols.iter().filter(|s| *s == "AAPL").count(), 1);
    }

    #[test]
    fn given_headline_with_unicode_when_extracted_then_matches() {
        let symbols = extract_symbols("café Apple reports earnings");
        assert!(symbols.contains(&"AAPL".to_string()));
    }
}
