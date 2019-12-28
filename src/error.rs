use std::{error, fmt};

#[derive(Debug, PartialEq)]
pub enum MoneyError {
    InvalidCurrency,
    InvalidAmount,
    InvalidRatio,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MoneyError::InvalidCurrency => write!(f, "No currencies were found"), //TODO - Update Text
            MoneyError::InvalidAmount => write!(f, "Amount was not valid"), //TODO - Update Text
            MoneyError::InvalidRatio => write!(f, "Ratio was not valid"),   //TODO - Update Text
        }
    }
}

impl error::Error for MoneyError {
    fn description(&self) -> &str {
        match *self {
            MoneyError::InvalidCurrency => "No Currency Found",
            MoneyError::InvalidAmount => "Amount not parseable",
            MoneyError::InvalidRatio => "Ratio not valid",
        }
    }
}

impl From<std::num::ParseIntError> for MoneyError {
    fn from(_err: std::num::ParseIntError) -> MoneyError {
        MoneyError::InvalidAmount
    }
}
