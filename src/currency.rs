pub mod crypto;
mod crypto_currency;
pub mod iso;
mod iso_currency;
use crate::{Locale, MoneyError};
pub use crypto_currency::CryptoCurrency;
pub use iso_currency::IsoCurrency;

/// The `FormattableCurrency` trait
///
/// This trait is required for a Currency implementation to be accepted by Money as valid input.
pub trait FormattableCurrency: PartialEq + Eq + Copy {
    fn to_string(&self) -> String;

    fn exponent(&self) -> u32;

    fn code(&self) -> &'static str;

    fn locale(&self) -> Locale;

    fn symbol(&self) -> &'static str;

    fn symbol_first(&self) -> bool;

    fn find(code: &str) -> Result<&'static Self, MoneyError>;
}
