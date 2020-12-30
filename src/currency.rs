pub mod crypto;
mod crypto_currency;
pub mod iso;
mod iso_currency;
use crate::{Locale, MoneyError};
pub use crypto_currency::CryptoCurrency;
pub use iso_currency::IsoCurrency;

// TODO: Add documentations for the trait.
pub trait FormattableCurrency: PartialEq + Eq + Copy {
    fn to_string(&self) -> String;

    fn exponent(&self) -> u32;

    fn code(&self) -> &'static str;

    fn locale(&self) -> Locale;

    fn symbol(&self) -> &'static str;

    fn symbol_first(&self) -> bool;

    fn find(code: &str) -> Result<&'static Self, MoneyError>;
}

/// A struct which represent a generic currency.
///
/// Currency stores metadata like locale, exponent and symbol. Operations on Currencies pass around references,
/// since they are unchanging.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Currency {
    pub code: &'static str,
    pub exponent: u32,
    pub locale: Locale,
    pub minor_denomination: u64,
    pub symbol: &'static str,
    pub symbol_first: bool,
}

// TODO: How do you implement find for generic currencies?

// impl FormattableCurrency for Currency {
//     fn to_string(&self) -> String {
//         self.code.to_string()
//     }

//     fn exponent(&self) -> u32 {
//         self.exponent
//     }

//     fn code(&self) -> &'static str {
//         self.code
//     }

//     fn locale(&self) -> Locale {
//         self.locale
//     }

//     fn symbol(&self) -> &'static str {
//         self.symbol
//     }

//     fn symbol_first(&self) -> bool {
//         self.symbol_first
//     }

//     /// Returns a Currency given an ISO-4217 currency code as an str.
//     fn find(code: &str) -> Result<&'static Currency, MoneyError> {
//         IsoCurrency::from_identifier(code.to_string())
//     }
// }

impl Currency {
    /// Returns a Currency.
    pub fn new(
        code: &'static str,
        exponent: u32,
        locale: Locale,
        minor_denomination: u64,
        symbol: &'static str,
        symbol_first: bool,
    ) -> Currency {
        Currency {
            code,
            exponent,
            locale,
            minor_denomination,
            symbol,
            symbol_first,
        }
    }
}
