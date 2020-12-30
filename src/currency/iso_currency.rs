use crate::currency::iso;
use crate::currency::FormattableCurrency;
use crate::{Locale, MoneyError};
use std::fmt;

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

impl FormattableCurrency for IsoCurrency {
    fn to_string(&self) -> String {
        self.code().to_string()
    }

    fn exponent(&self) -> u32 {
        self.exponent
    }

    fn code(&self) -> &'static str {
        self.iso_alpha_code
    }

    fn locale(&self) -> Locale {
        self.locale
    }

    fn symbol(&self) -> &'static str {
        self.symbol
    }

    fn symbol_first(&self) -> bool {
        self.symbol_first
    }

    /// Returns a Currency given an ISO-4217 currency code as an str.
    fn find(code: &str) -> Result<&'static IsoCurrency, MoneyError> {
        if code.chars().all(char::is_alphabetic) {
            iso::find_by_alpha_code(code).ok_or(MoneyError::InvalidCurrency)
        } else if code.chars().all(char::is_numeric) {
            iso::find_by_num_code(code).ok_or(MoneyError::InvalidCurrency)
        } else {
            Err(MoneyError::InvalidCurrency)
        }
    }
}

impl fmt::Display for IsoCurrency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iso_alpha_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_returns_known_currencies() {
        let currency_by_alpha = IsoCurrency::find("USD").unwrap();
        assert_eq!(currency_by_alpha.iso_alpha_code, "USD");
        assert_eq!(currency_by_alpha.exponent, 2);
        assert_eq!(currency_by_alpha.symbol, "$");

        let currency_by_numeric = IsoCurrency::find("840").unwrap();
        assert_eq!(currency_by_alpha, currency_by_numeric);
    }

    #[test]
    fn find_raises_invalid_currency_error_on_unknown_currency() {
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
    fn currency_can_be_accessed_by_reference() {
        assert_eq!(iso::USD.iso_alpha_code, "USD");
        assert_eq!(iso::USD.exponent, 2);
        assert_eq!(iso::USD.symbol, "$");
    }

    #[test]
    fn find_and_reference_point_to_same() {
        assert_eq!(iso::USD, IsoCurrency::find("USD").unwrap());
    }
}
