mod iso_currencies;
use crate::CurrencyError;
use iso_currencies::ISO_CURRENCIES;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

lazy_static! {
    static ref CURRENCIES_BY_ALPHA_CODE: HashMap<String, Currency> =
        Currency::generate_currencies_by_alpha_code();
    static ref CURRENCIES_BY_NUM_CODE: HashMap<String, Currency> =
        Currency::generate_currencies_by_num_code();
}

/// The `Currency` type, which stores metadata about an ISO-4127 currency.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    /// Returns a Currency given an ISO-4217 currency code as an str.
    pub fn find(code: &str) -> Result<Currency, CurrencyError> {
        Currency::from_string(code.to_string())
    }

    /// Returns a Currency given an ISO-4217 currency code as a string.
    pub fn from_string(code: String) -> Result<Currency, CurrencyError> {
        if code.chars().all(char::is_alphabetic) {
            Currency::find_by_alpha_iso(code).ok_or(CurrencyError::InvalidCurrency)
        } else if code.chars().all(char::is_numeric) {
            Currency::find_by_numeric_iso(code).ok_or(CurrencyError::InvalidCurrency)
        } else {
            Err(CurrencyError::InvalidCurrency)
        }
    }

    /// Returns a Currency given an alphabetic ISO-4217 currency code.
    pub fn find_by_alpha_iso(code: String) -> Option<Currency> {
        match CURRENCIES_BY_ALPHA_CODE.get(&code.to_uppercase()) {
            Some(c) => Some(*c),
            None => None,
        }
    }

    /// Returns a currency given a numeric ISO-4217 currency code.
    pub fn find_by_numeric_iso(code: String) -> Option<Currency> {
        match CURRENCIES_BY_NUM_CODE.get(&code) {
            Some(c) => Some(*c),
            None => None,
        }
    }

    /// Returns a vector indicating where digit separators should be applied for a given currency.  
    pub fn digit_separator_sequence(self) -> Vec<usize> {
        let v: Vec<&str> = self.digit_separator_sequence.split(", ").collect();
        v.iter().map(|x| usize::from_str(x).unwrap()).collect()
    }

    /// Returns a Currency Hashmap, keyed by ISO alphabetic code.
    fn generate_currencies_by_alpha_code() -> HashMap<String, Currency> {
        let mut num_map: HashMap<String, Currency> = HashMap::new();
        for c in ISO_CURRENCIES.iter() {
            num_map.insert(c.iso_alpha_code.to_string(), *c);
        }
        num_map
    }

    /// Returns a Currency Hashmap, keyed by ISO numeric code.
    fn generate_currencies_by_num_code() -> HashMap<String, Currency> {
        let mut num_map: HashMap<String, Currency> = HashMap::new();
        for c in ISO_CURRENCIES.iter() {
            num_map.insert(c.iso_numeric_code.to_string(), *c);
        }
        num_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn currency_known_can_be_found() {
        let currency_by_alpha = Currency::find("USD").unwrap();
        assert_eq!(currency_by_alpha.iso_alpha_code, "USD");
        assert_eq!(currency_by_alpha.exponent, 2);
        assert_eq!(currency_by_alpha.symbol, "$");

        let currency_by_numeric = Currency::find("840").unwrap();
        assert_eq!(currency_by_alpha, currency_by_numeric);
    }

    #[test]
    fn currency_unknown_iso_codes_raise_invalid_currency_error() {
        assert_eq!(
            Currency::find("fake").unwrap_err(),
            CurrencyError::InvalidCurrency,
        );

        assert_eq!(
            Currency::find("123").unwrap_err(),
            CurrencyError::InvalidCurrency,
        );
    }
}
