use std::{error, fmt};

#[derive(Debug, PartialEq)]
pub enum CurrencyError {
    InvalidCurrency,
    InvalidAmount,
    InvalidRatio,
}

impl fmt::Display for CurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CurrencyError::InvalidCurrency => write!(f, "No currencies were found"), //TODO - Update Text
            CurrencyError::InvalidAmount => write!(f, "Amount was not valid"), //TODO - Update Text
            CurrencyError::InvalidRatio => write!(f, "Ratio was not valid"),   //TODO - Update Text
        }
    }
}

impl error::Error for CurrencyError {
    fn description(&self) -> &str {
        match *self {
            CurrencyError::InvalidCurrency => "No Currency Found",
            CurrencyError::InvalidAmount => "Amount not parseable",
            CurrencyError::InvalidRatio => "Ratio not valid",
        }
    }
}

impl From<std::num::ParseIntError> for CurrencyError {
    fn from(_err: std::num::ParseIntError) -> CurrencyError {
        CurrencyError::InvalidAmount
    }
}
