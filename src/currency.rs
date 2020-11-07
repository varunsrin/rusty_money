mod iso;
pub use crate::{LocalFormat, Locale, MoneyError};
pub use iso::Iso;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

lazy_static! {
    static ref CURRENCIES_BY_ALPHA_CODE: HashMap<String, Currency> =
        Currency::generate_currencies_by_alpha_code();
    static ref CURRENCIES_BY_NUM_CODE: HashMap<String, Currency> =
        Currency::generate_currencies_by_num_code();
}

/// A struct which represent an ISO-4127 currency.
///
/// Currency stores metadata like numeric code, full name and symbol. Operations on Currencies pass around references,
/// since they are unchanging.
#[derive(Debug, PartialEq, Eq)]
pub struct Currency {
    pub locale: Locale,
    pub exponent: u32,
    pub iso_alpha_code: &'static str,
    pub iso_numeric_code: &'static str,
    pub name: &'static str,
    pub symbol: &'static str,
    pub symbol_first: bool,
    pub minor_denomination: u32,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iso_alpha_code)
    }
}

impl Currency {
    /// Returns a Currency given an ISO-4217 currency code as an str.
    pub fn find(code: &str) -> Result<&'static Currency, MoneyError> {
        FromStr::from_str(code)
    }

    /// Returns a Currency given a Currency enumeration.
    pub fn get(code: Iso) -> &'static Currency {
        From::from(code)
    }

    /// Returns a Currency given an alphabetic ISO-4217 currency code.
    pub fn find_by_alpha_iso(code: &str) -> Option<&'static Currency> {
        match CURRENCIES_BY_ALPHA_CODE.get(&code.to_uppercase()) {
            Some(c) => Some(c),
            None => None,
        }
    }

    /// Returns a currency given a numeric ISO-4217 currency code.
    pub fn find_by_numeric_iso(code: &str) -> Option<&'static Currency> {
        match CURRENCIES_BY_NUM_CODE.get(code) {
            Some(c) => Some(c),
            None => None,
        }
    }

    /// Returns a Currency Hashmap, keyed by ISO alphabetic code.
    fn generate_currencies_by_alpha_code() -> HashMap<String, Currency> {
        let mut num_map: HashMap<String, Currency> = HashMap::new();

        for item in iso::ISO_CURRENCIES {
            let currency = iso::from_enum(item);
            num_map.insert(currency.iso_alpha_code.to_string(), currency);
        }
        num_map
    }

    /// Returns a Currency Hashmap, keyed by ISO numeric code.
    fn generate_currencies_by_num_code() -> HashMap<String, Currency> {
        let mut num_map: HashMap<String, Currency> = HashMap::new();
        for item in iso::ISO_CURRENCIES {
            let currency = iso::from_enum(item);
            num_map.insert(currency.iso_numeric_code.to_string(), currency);
        }
        num_map
    }
}

/// Returns a Currency given an ISO-4217 currency code as an str.
impl FromStr for &'static Currency {
    type Err = MoneyError;
    fn from_str(code: &str) -> Result<Self, Self::Err> {
        if code.chars().all(char::is_alphabetic) {
            Currency::find_by_alpha_iso(code).ok_or(MoneyError::InvalidCurrency)
        } else if code.chars().all(char::is_numeric) {
            Currency::find_by_numeric_iso(code).ok_or(MoneyError::InvalidCurrency)
        } else {
            Err(MoneyError::InvalidCurrency)
        }
    }
}

/// Returns a Currency given an ISO-4217 currency code as a string.
impl std::convert::From<Iso> for &'static Currency {
    fn from(iso_code: Iso) -> Self {
        FromStr::from_str(&iso_code.to_string()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iso::Iso::*;
    #[test]
    fn currency_find_known_can_be_found() {
        let currency_by_alpha = Currency::find("USD").unwrap();
        assert_eq!(currency_by_alpha.iso_alpha_code, "USD");
        assert_eq!(currency_by_alpha.exponent, 2);
        assert_eq!(currency_by_alpha.symbol, "$");

        let currency_by_numeric = Currency::find("840").unwrap();
        assert_eq!(currency_by_alpha, currency_by_numeric);
    }

    #[test]
    fn currency_find_unknown_iso_codes_raise_invalid_currency_error() {
        assert_eq!(
            Currency::find("fake").unwrap_err(),
            MoneyError::InvalidCurrency,
        );

        assert_eq!(
            Currency::find("123").unwrap_err(),
            MoneyError::InvalidCurrency,
        );
    }

    #[test]
    fn currency_get() {
        assert_eq!(Currency::get(USD), Currency::find("USD").unwrap());
    }
}
