use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::include_str;
use std::str::FromStr;

static CURRENCY_JSON: &str = include_str!("../config/currencies.json");

lazy_static! {
    static ref CURRENCIES_BY_ALPHA_CODE: HashMap<String, Currency> =
        serde_json::from_str(CURRENCY_JSON).unwrap();
    static ref CURRENCIES_BY_NUM_CODE: HashMap<String, Currency> =
        Currency::generate_currencies_by_num_code();
}

/// The `Currency` type, which stores metadata about an ISO-4127 currency.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Currency {
    pub digit_separator: char,
    digit_separator_sequence: &'static str,
    pub exponent: u32,
    pub exponent_separator: char,
    pub iso_alpha_code: &'static str,
    pub iso_numeric_code: &'static str,
    pub symbol: &'static str,
    pub symbol_first: bool,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iso_alpha_code)
    }
}

impl Currency {
    /// Finds a currency type given an ISO-4217 currency code.
    pub fn find(code: String) -> Currency {
        if code.chars().all(char::is_alphabetic) {
            Currency::find_by_alpha_iso(code)
        } else if code.chars().all(char::is_numeric) {
            Currency::find_by_numeric_iso(code)
        } else {
            panic!("{} is not a known currency", code);
        }
    }

    /// Finds a currency type given an alphabetic currency code.
    pub fn find_by_alpha_iso(code: String) -> Currency {
        match CURRENCIES_BY_ALPHA_CODE.get(&code.to_lowercase()) {
            Some(c) => *c,
            None => panic!("{} is not a known currency", code), //TODO - more helpful message
        }
    }

    /// Finds a currency type given a numeric ISO-4217 currency code.
    pub fn find_by_numeric_iso(code: String) -> Currency {
        match CURRENCIES_BY_NUM_CODE.get(&code.to_lowercase()) {
            Some(c) => *c,
            None => panic!("{} is not a known currency", code), //TODO - more helpful message
        }
    }

    /// Returns a vector indicating where digit separators should be applied for a given currency.  
    pub fn digit_separator_sequence(self) -> Vec<usize> {
        let v: Vec<&str> = self.digit_separator_sequence.split(", ").collect();
        v.iter().map(|x| usize::from_str(x).unwrap()).collect()
    }

    /// Returns a hashmap of currencies indexed by their ISO numeric code.
    fn generate_currencies_by_num_code() -> HashMap<String, Currency> {
        let mut num_map: HashMap<String, Currency> = HashMap::new();
        for (_k, v) in CURRENCIES_BY_ALPHA_CODE.iter() {
            num_map.insert(v.iso_numeric_code.to_string(), *v);
        }
        num_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn currency_known_can_be_found() {
        let currency_by_alpha = Currency::find("USD".to_string());
        assert_eq!(currency_by_alpha.iso_alpha_code, "USD");
        assert_eq!(currency_by_alpha.exponent, 2);
        assert_eq!(currency_by_alpha.symbol, "$");

        let currency_by_numeric = Currency::find("840".to_string());
        assert_eq!(currency_by_alpha, currency_by_numeric);
    }

    #[test]
    #[should_panic]
    fn currency_unknown_iso_alpha_code_raises_error() {
        Currency::find("fake".to_string());
    }

    #[test]
    #[should_panic]
    fn currency_unknown_iso_num_code_raises_error() {
        Currency::find("123".to_string());
    }
}
