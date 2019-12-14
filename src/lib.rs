//! Handle money and currency conversions.
//!
//! Money lets you handle currencies in Rust easily  and takes care of rounding, currency tracking
//! and parsing monetary symbols according to ISO standards.
//!
//! # Use
//!
//! The easiest way to create Money is by using the flexible money! macro:
//!
//! ```edition2018
//! money = money!("-200.00", "USD");
//! money = money!(-200, "USD");
//! ```
//!
//! Money handles rounding for you based on the properties of the currency:    
//!
//! ```edition2018
//! money = money!("-200.009", "USD");
//! println!("{:?}", money) // -200.01 USD
//!
//! TODO - show a currency with different exponent
//! ```
//!   
//! You can perform basic operations on money like:
//!
//! ```edition2018
//! hundred = money!("100", "USD");
//! thousand = money!("1000", "USD")
//! println!("{:?}", hundred + thousand)     // 1000 USD
//! println!("{:?}", thousand > hundred)     // false
//! println!("{:?}", thousand.is_positive()) // true
//! ```
//!
//! Currency is still a work in progress, but has hardcoded values for USD and GBP.

use rust_decimal::Decimal;
use rust_decimal_macros::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::str::FromStr;
#[macro_use]
extern crate lazy_static;

// TODO
// Move config data into json.
// Currencies - only pass around references, don't go creating new ones.

lazy_static! {
    static ref CURRENCIES: HashMap<String, Currency> = serde_json::from_str(DATA).unwrap();
}

const DATA: &str = r#"{
    "usd": {
        "exponent": 2,
        "name": "USD"
    },
    "gbp": {
        "exponent": 2,
        "name": "GBP"
    },
    "zbd": {
        "exponent": 3,
        "name": "GBP"
    }
}"#;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct Currency {
    name: &'static str,
    exponent: u32,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Currency {
    pub fn new(name: String) -> Currency {
        match CURRENCIES.get(&name.to_lowercase()) {
            Some(c) => *c,
            None => panic!("{} is not a known currency", name), //TODO - more helpful message
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Money {
    amount: Decimal,
    currency: Currency,
}

impl Add for Money {
    type Output = Money;
    fn add(self, other: Money) -> Money {
        Money::new(self.amount + other.amount, self.currency)
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, other: Money) -> Money {
        Money::new(self.amount - other.amount, self.currency)
    }
}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Money) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Money) -> Ordering {
        if self.currency != other.currency {
            panic!();
        }
        self.amount.cmp(&other.amount)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            amount: self.amount + other.amount,
            currency: self.currency,
        };
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            amount: self.amount - other.amount,
            currency: self.currency,
        };
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

#[macro_export]
macro_rules! money {
    ($x:expr, $y:expr) => {
        Money::from_string($x.to_string(), $y.to_string());
    };
}

impl Money {
    pub fn new(amount: Decimal, currency: Currency) -> Money {
        Money {
            amount: amount.round_dp(currency.exponent),
            currency,
        }
    }

    pub fn from_string(amount: String, currency: String) -> Money {
        // TODO fetch these values from the current metadata when implemented.
        let separator: char = ',';
        let delimiter: char = '.';
        let currency = Currency::new(currency);

        let amount_parts: Vec<&str> = amount.split(delimiter).collect();

        fn panic_unless_integer(value: &str) {
            match i32::from_str(value) {
                Ok(_) => (),
                // TODO update to match the right error cases
                Err(_) => panic!("Could not parse"),
            }
        }

        let mut parsed_decimal = amount_parts[0].replace(separator, "");
        panic_unless_integer(&parsed_decimal);

        if amount_parts.len() == 1 {
            parsed_decimal += ".";
            for _ in 0..currency.exponent {
                parsed_decimal += "0";
            }
        } else if amount_parts.len() == 2 {
            panic_unless_integer(&amount_parts[1]);
            parsed_decimal = parsed_decimal + "." + amount_parts[1];
        } else {
            panic!()
        }

        let decimal = Decimal::from_str(&parsed_decimal)
            .unwrap()
            .round_dp(currency.exponent);
        Money::new(decimal, currency)
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn currency(&self) -> &str {
        &self.currency.name
    }

    pub fn allocate_to(&self, number: i32) -> Vec<Money> {
        let ratios: Vec<i32> = (0..number).map(|_| 1).collect();
        self.allocate(ratios)
    }

    pub fn is_zero(&self) -> bool {
        self.amount == dec!(0.0)
    }

    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive() && self.amount != dec!(0.0)
    }

    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative() && self.amount != dec!(0.0)
    }

    pub fn allocate(&self, ratios: Vec<i32>) -> Vec<Money> {
        if ratios.is_empty() {
            panic!();
        }

        let ratios_dec: Vec<Decimal> = ratios
            .iter()
            .map(|x| Decimal::from_str(&x.to_string()).unwrap().round_dp(0))
            .collect();

        let mut remainder = self.amount;
        let ratio_total: Decimal = ratios_dec.iter().fold(dec!(0.0), |acc, x| acc + x);

        let mut allocations: Vec<Money> = Vec::new();

        for ratio in ratios_dec {
            if ratio <= dec!(0.0) {
                panic!("Ratio was zero or negative, should be positive");
            }

            let share = (self.amount * ratio / ratio_total).floor();

            allocations.push(Money::new(share, self.currency));
            remainder -= share;
        }

        if remainder < dec!(0.0) {
            panic!("Remainder was negative, should be 0 or positive");
        }

        if remainder - remainder.floor() != dec!(0.0) {
            panic!("Remainder is not an integer, should be an integer");
        }

        let mut i = 0;
        while remainder > dec!(0.0) {
            allocations[i as usize].amount += dec!(1.0);
            remainder -= dec!(1.0);
            i += 1;
        }
        allocations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //
    // Currency Tests
    //
    #[test]
    fn currency_known_can_be_created() {
        let c = Currency::new("USD".to_string());
        assert_eq!(c.name, "USD");
        assert_eq!(c.exponent, 2);
    }

    #[test]
    #[should_panic]
    fn currency_unknown_raises_error() {
        Currency::new("fake".to_string());
    }

    //
    // Money Tests
    //
    #[test]
    fn money_rounds_exponent() {
        // 19.999 rounds to 20 for USD
        let money = Money::new(dec!(19.9999), Currency::new("USD".to_string()));
        let expected_money = Money::new(Decimal::new(20, 0), Currency::new("USD".to_string()));
        assert_eq!(money, expected_money);
        let expected_string = "20.00";
        let actual_string = money.amount().to_string();
        assert_eq!(actual_string, expected_string);

        // 29.111 rounds to 29.11 for USD
        let money = Money::new(dec!(29.111), Currency::new("USD".to_string()));
        let expected_money = Money::new(dec!(29.11), Currency::new("USD".to_string()));
        assert_eq!(money, expected_money);
        let expected_string = "29.11";
        assert_eq!(money.amount().to_string(), expected_string);

        // 39.1155 rounds to 39.116 for USD
        let money = Money::new(dec!(39.1155), Currency::new("ZBD".to_string()));
        let expected_money = Money::new(dec!(39.116), Currency::new("ZBD".to_string()));
        assert_eq!(money, expected_money);
        let expected_string = "39.116";
        assert_eq!(money.amount().to_string(), expected_string);
    }

    #[test]
    fn money_from_string_parses_correctly() {
        let expected_money = Money::new(Decimal::new(2999, 2), Currency::new("GBP".to_string()));
        let money = Money::from_string("29.99".to_string(), "GBP".to_string());
        assert_eq!(money, expected_money);
    }

    #[test]
    fn money_from_string_parses_signs() {
        let expected_money = Money::new(Decimal::new(-3, 0), Currency::new("GBP".to_string()));
        let money = Money::from_string("-3".to_string(), "GBP".to_string());
        assert_eq!(money, expected_money);

        let expected_money = Money::new(Decimal::new(3, 0), Currency::new("GBP".to_string()));
        let money = Money::from_string("+3".to_string(), "GBP".to_string());
        assert_eq!(money, expected_money);
    }

    #[test]
    fn money_from_string_rounds_exponent() {
        // 19.999 rounds to 20 for USD
        let expected_money = Money::new(Decimal::new(20, 0), Currency::new("USD".to_string()));
        let money = Money::from_string("19.9999".to_string(), "USD".to_string());
        assert_eq!(money, expected_money);

        // 29.111 rounds to 29.11 for USD
        let expected_money = Money::new(Decimal::new(2911, 2), Currency::new("USD".to_string()));
        let money = Money::from_string("29.111".to_string(), "USD".to_string());
        assert_eq!(money, expected_money);

        // 39.1155 rounds to 39.116 for ZBD
        let expected_money = Money::new(dec!(39.116), Currency::new("ZBD".to_string()));
        let money = Money::from_string("39.1155".to_string(), "ZBD".to_string());
        assert_eq!(money, expected_money);
    }

    #[test]
    fn money_from_string_ignores_separators() {
        let expected_money = Money::new(Decimal::new(1000000, 0), Currency::new("GBP".to_string()));
        let money = Money::from_string("1,000,000".to_string(), "GBP".to_string());
        assert_eq!(money, expected_money);
    }

    #[test]
    #[should_panic]
    fn money_from_string_panics_if_delimiter_preceeds_separator() {
        Money::from_string("1.0000,000".to_string(), "GBP".to_string());
    }

    #[test]
    #[should_panic]
    fn money_from_string_panics_if_multiple_delimiters() {
        Money::from_string("1.0000.000".to_string(), "GBP".to_string());
    }

    #[test]
    #[should_panic]
    fn money_from_string_panics_if_unrecognized_character() {
        Money::from_string("1.0000!000".to_string(), "GBP".to_string());
    }

    #[test]
    #[should_panic]
    fn money_from_string_panics_if_only_separator() {
        Money::from_string(",".to_string(), "GBP".to_string());
    }

    #[test]
    #[should_panic]
    fn money_from_string_panics_if_no_digits() {
        Money::from_string(".".to_string(), "GBP".to_string());
    }

    #[test]
    #[should_panic]
    fn money_from_string_panics_if_only_separators_and_delimiters() {
        Money::from_string(",,.".to_string(), "GBP".to_string());
    }

    #[test]
    fn money_ops() {
        // Addition
        assert_eq!(money!(2, "USD"), money!(1, "USD") + money!(1, "USD"));
        // Subtraction
        assert_eq!(money!(0, "USD"), money!(1, "USD") - money!(1, "USD"));
        // Greater Than
        assert_eq!(true, money!(2, "USD") > money!(1, "USD"));
        // Less Than
        assert_eq!(false, money!(2, "USD") < money!(1, "USD"));
        // Equals
        assert_eq!(true, money!(1, "USD") == money!(1, "USD"));
        assert_eq!(false, money!(1, "USD") == money!(1, "GBP"));
        // is positive
        assert_eq!(true, money!(1, "USD").is_positive());
        assert_eq!(false, money!(0, "USD").is_positive());
        assert_eq!(false, money!(-1, "USD").is_positive());

        // is zero
        assert_eq!(true, money!(0, "USD").is_zero());
        assert_eq!(false, money!(1, "USD").is_zero());
        assert_eq!(false, money!(-1, "USD").is_zero());

        // is negative
        assert_eq!(true, money!(-1, "USD").is_negative());
        assert_eq!(false, money!(1, "USD").is_negative());
        assert_eq!(false, money!(0, "USD").is_negative());
    }

    #[test]
    #[should_panic]
    fn money_ops_greater_than_panics_on_different_currencies() {
        assert!(money!(1, "USD") < money!(1, "GBP"));
    }

    #[test]
    #[should_panic]
    fn money_ops_less_than_panics_on_different_currencies() {
        assert!(money!(1, "USD") < money!(1, "GBP"));
    }

    #[test]
    fn money_allocate() {
        let money = money!(11, "USD");
        let allocs = money.allocate(vec![1, 1, 1]);
        let expected_results = vec![money!(4, "USD"), money!(4, "USD"), money!(3, "USD")];
        assert_eq!(expected_results, allocs);
    }

    #[test]
    #[should_panic]
    fn money_allocate_panics_if_empty() {
        money!(1, "USD").allocate(Vec::new());
    }

    #[test]
    #[should_panic]
    fn money_allocate_panics_any_ratio_is_zero() {
        money!(1, "USD").allocate(vec![1, 0]);
    }

    #[test]
    fn money_allocate_to() {
        let money = money!(11, "USD");
        let allocs = money.allocate_to(3);
        let expected_results = vec![money!(4, "USD"), money!(4, "USD"), money!(3, "USD")];
        assert_eq!(expected_results, allocs);
    }

    #[test]
    #[should_panic]
    fn money_allocate_to_panics_if_zero() {
        money!(1, "USD").allocate_to(0);
    }
}
