//!
//! Currency is still a work in progress, but has hardcoded values for USD and GBP.
//!
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use std::fs;

// Release TODO
// 1. Refactor lib.rs into separate files.
// 2. Import the 150 or so odd currencies with separator, delimiter and exponent.
// x-1. Clear out TODO's
// x. Update Docs

lazy_static! {
    static ref CURRENCY_CONFIG: String =
        fs::read_to_string("config/currencies.json".to_string()).unwrap();
    static ref CURRENCIES: HashMap<String, Currency> =
        serde_json::from_str(&CURRENCY_CONFIG).unwrap();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Currency {
    pub name: &'static str,
    pub exponent: u32,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Currency {
    pub fn find(name: String) -> Currency {
        match CURRENCIES.get(&name.to_lowercase()) {
            Some(c) => *c,
            None => panic!("{} is not a known currency", name), //TODO - more helpful message
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn currency_known_can_be_found() {
        let c = Currency::find("USD".to_string());
        assert_eq!(c.name, "USD");
        assert_eq!(c.exponent, 2);
    }

    #[test]
    #[should_panic]
    fn currency_unknown_raises_error() {
        Currency::find("fake".to_string());
    }
}
