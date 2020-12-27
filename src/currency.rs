mod iso;
pub use crate::{LocalFormat, Locale, MoneyError};
pub use iso::Iso;
use std::collections::HashMap;
use std::fmt;

lazy_static! {
    static ref CURRENCIES_BY_ALPHA_CODE: HashMap<String, IsoCurrency> =
        IsoCurrency::generate_currencies_by_alpha_code();
    static ref CURRENCIES_BY_NUM_CODE: HashMap<String, IsoCurrency> =
        IsoCurrency::generate_currencies_by_num_code();
}

pub trait CurrencyType: PartialEq + Eq + Copy {
    fn to_string(&self) -> String;

    fn exponent(&self) -> u32;

    fn iso_alpha_code(&self) -> &'static str;
}

pub trait FormattableCurrency {
    fn locale(&self) -> Locale;

    fn symbol(&self) -> &'static str;

    fn symbol_first(&self) -> bool;
}


/// A struct which represent a generic currency.
///
/// Currency stores metadata like locale, exponent and symbol. Operations on Currencies pass around references,
/// since they are unchanging.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Currency {
    pub code: &'static str,
    pub exponent: u32,
    pub minor_denomination: u32,
}

impl CurrencyType for Currency {
    fn to_string(&self) -> String {
        self.code.to_string()
    }

    fn exponent(&self) -> u32 {
        self.exponent
    }

    // TODO: Fix this method to be generic.
    fn iso_alpha_code(&self) -> &'static str {
        self.code
    }
}

impl Currency {
    /// Returns a Currency.
    pub fn new(
        code: &'static str,
        exponent: u32,
        minor_denomination: u32,
    ) -> Currency {
        Currency {
            code,
            exponent,
            minor_denomination,
        }
    }
}

/// A struct which represent an ISO-4127 currency.
///
/// IsoCurrency stores metadata like numeric code, full name and symbol. Operations on Currencies pass around references,
/// since they are unchanging.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct IsoCurrency {
    pub locale: Locale,
    pub exponent: u32,
    pub iso_alpha_code: &'static str,
    pub iso_numeric_code: &'static str,
    pub name: &'static str,
    pub symbol: &'static str,
    pub symbol_first: bool,
    pub minor_denomination: u32,
}

impl CurrencyType for IsoCurrency {
    fn to_string(&self) -> String {
        self.iso_alpha_code().to_string()
    }

    fn exponent(&self) -> u32 {
        self.exponent
    }

    fn iso_alpha_code(&self) -> &'static str {
        self.iso_alpha_code
    }
}


impl FormattableCurrency for IsoCurrency {
    fn locale(&self) -> Locale {
        self.locale
    }

    fn symbol(&self) -> &'static str {
        self.symbol
    }

    fn symbol_first(&self) -> bool {
        self.symbol_first
    }
}

impl fmt::Display for IsoCurrency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iso_alpha_code)
    }
}

impl IsoCurrency {
    /// Returns a Currency given an ISO-4217 currency code as an str.
    pub fn find(code: &str) -> Result<&'static IsoCurrency, MoneyError> {
        IsoCurrency::from_string(code.to_string())
    }

    /// Returns a Currency given a Currency enumeration.
    pub fn get(code: Iso) -> &'static IsoCurrency {
        &IsoCurrency::from_string(code.to_string()).unwrap()
    }

    /// Returns a Currency given an ISO-4217 currency code as a string.
    pub fn from_string(code: String) -> Result<&'static IsoCurrency, MoneyError> {
        if code.chars().all(char::is_alphabetic) {
            IsoCurrency::find_by_alpha_iso(code).ok_or(MoneyError::InvalidCurrency)
        } else if code.chars().all(char::is_numeric) {
            IsoCurrency::find_by_numeric_iso(code).ok_or(MoneyError::InvalidCurrency)
        } else {
            Err(MoneyError::InvalidCurrency)
        }
    }

    /// Returns a Currency given an alphabetic ISO-4217 currency code.
    pub fn find_by_alpha_iso(code: String) -> Option<&'static IsoCurrency> {
        match CURRENCIES_BY_ALPHA_CODE.get(&code.to_uppercase()) {
            Some(c) => Some(c),
            None => None,
        }
    }

    /// Returns a currency given a numeric ISO-4217 currency code.
    pub fn find_by_numeric_iso(code: String) -> Option<&'static IsoCurrency> {
        match CURRENCIES_BY_NUM_CODE.get(&code) {
            Some(c) => Some(c),
            None => None,
        }
    }

    /// Returns a Currency Hashmap, keyed by ISO alphabetic code.
    fn generate_currencies_by_alpha_code() -> HashMap<String, IsoCurrency> {
        let mut num_map: HashMap<String, IsoCurrency> = HashMap::new();

        for item in iso::ISO_CURRENCIES {
            let currency = iso::from_enum(item);
            num_map.insert(currency.iso_alpha_code.to_string(), currency);
        }
        num_map
    }

    /// Returns a Currency Hashmap, keyed by ISO numeric code.
    fn generate_currencies_by_num_code() -> HashMap<String, IsoCurrency> {
        let mut num_map: HashMap<String, IsoCurrency> = HashMap::new();
        for item in iso::ISO_CURRENCIES {
            let currency = iso::from_enum(item);
            num_map.insert(currency.iso_numeric_code.to_string(), currency);
        }
        num_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iso::Iso::*;
    #[test]
    fn currency_find_known_can_be_found() {
        let currency_by_alpha = IsoCurrency::find("USD").unwrap();
        assert_eq!(currency_by_alpha.iso_alpha_code, "USD");
        assert_eq!(currency_by_alpha.exponent, 2);
        assert_eq!(currency_by_alpha.symbol, "$");

        let currency_by_numeric = IsoCurrency::find("840").unwrap();
        assert_eq!(currency_by_alpha, currency_by_numeric);
    }

    #[test]
    fn currency_find_unknown_iso_codes_raise_invalid_currency_error() {
        assert_eq!(
            IsoCurrency::find("fake").unwrap_err(),
            MoneyError::InvalidCurrency,
        );

        assert_eq!(
            IsoCurrency::find("123").unwrap_err(),
            MoneyError::InvalidCurrency,
        );
    }

    #[test]
    fn currency_get() {
        assert_eq!(IsoCurrency::get(USD), IsoCurrency::find("USD").unwrap());
    }
}
