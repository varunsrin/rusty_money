use crate::currency::crypto;
use crate::currency::FormattableCurrency;
use crate::{Locale, MoneyError};
use std::fmt;

/// Represents a single Crypto Currency (e.g. BTC).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CryptoCurrency {
    pub code: &'static str,
    pub exponent: u32,
    pub locale: Locale,
    pub minor_units: u64,
    pub name: &'static str,
    pub symbol: &'static str,
    pub symbol_first: bool,
}

impl FormattableCurrency for CryptoCurrency {
    fn to_string(&self) -> String {
        self.code().to_string()
    }

    fn exponent(&self) -> u32 {
        self.exponent
    }

    fn code(&self) -> &'static str {
        self.code
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
}

impl fmt::Display for CryptoCurrency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl CryptoCurrency {
    /// Returns a CryptoCurrency given a currency code as a &str.
    pub fn find(code: &str) -> Result<&'static CryptoCurrency, MoneyError> {
        crypto::find_by_code(code).ok_or(MoneyError::InvalidCurrency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_returns_known_currencies() {
        let currency_by_code = CryptoCurrency::find("BTC").unwrap();
        assert_eq!(currency_by_code.code, "BTC");
        assert_eq!(currency_by_code.exponent, 8);
        assert_eq!(currency_by_code.symbol, "₿");
    }

    #[test]
    fn find_raises_invalid_currency_error_on_unknown_currency() {
        assert_eq!(
            CryptoCurrency::find("fake").unwrap_err(),
            MoneyError::InvalidCurrency,
        );
    }

    #[test]
    fn currency_can_be_accessed_by_reference() {
        assert_eq!(crypto::ETH.code, "ETH");
        assert_eq!(crypto::ETH.exponent, 18);
        assert_eq!(crypto::ETH.symbol, "ETH");
    }

    #[test]
    fn find_and_reference_point_to_same() {
        assert_eq!(crypto::BTC, CryptoCurrency::find("BTC").unwrap());
    }
}
