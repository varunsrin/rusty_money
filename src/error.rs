use std::{error, fmt};

/// Standard Error type for this crate.
#[derive(Debug, PartialEq)]
pub enum MoneyError {
    InvalidCurrency,
    InvalidAmount,
    InvalidRatio,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MoneyError::InvalidCurrency => write!(f, "Currency was not valid"),
            MoneyError::InvalidAmount => write!(f, "Amount not parsable"),
            MoneyError::InvalidRatio => write!(f, "Ratio was not valid"),
        }
    }
}

impl error::Error for MoneyError {
    fn description(&self) -> &str {
        match *self {
            MoneyError::InvalidCurrency => "Currency was not valid",
            MoneyError::InvalidAmount => "Amount not pauseable",
            MoneyError::InvalidRatio => "Ratio was not valid",
        }
    }
}

impl From<std::num::ParseIntError> for MoneyError {
    fn from(_err: std::num::ParseIntError) -> MoneyError {
        MoneyError::InvalidAmount
    }
}
