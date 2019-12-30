use crate::currency::*;
use crate::format::*;
use crate::MoneyError;
use rust_decimal::Decimal;
use rust_decimal_macros::*;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use std::str::FromStr;

/// The `Money` type, which contains an amount and a currency.
///
/// Money represents financial amounts through a Decimal (owned) and a Currency (refernce).
/// Operations on Money objects always create new instances of Money, with the exception
/// of `round()`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Money {
    amount: Decimal,
    currency: &'static Currency,
}

/// Create `Money` from an amount and an ISO currency code.
///
/// The amount can be provided as a string or an integer.
#[macro_export]
macro_rules! money {
    ($x:expr, $y:expr) => {
        Money::from_string($x.to_string(), $y.to_string()).unwrap();
    };
}

impl Add for Money {
    type Output = Money;
    fn add(self, other: Money) -> Money {
        Money::from_decimal(self.amount + other.amount, self.currency)
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

impl Sub for Money {
    type Output = Money;
    fn sub(self, other: Money) -> Money {
        Money::from_decimal(self.amount - other.amount, self.currency)
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

macro_rules! impl_mul_div {
    ($type:ty) => {
        impl Mul<$type> for Money {
            type Output = Money;

            fn mul(self, rhs: $type) -> Money {
                let rhs = Decimal::from_str(&rhs.to_string()).unwrap();
                Money::from_decimal(self.amount * rhs, self.currency)
            }
        }

        impl Mul<Money> for $type {
            type Output = Money;

            fn mul(self, rhs: Money) -> Money {
                let lhs = Decimal::from_str(&self.to_string()).unwrap();
                Money::from_decimal(rhs.amount * lhs, rhs.currency)
            }
        }

        impl MulAssign<$type> for Money {
            fn mul_assign(&mut self, rhs: $type) {
                let rhs = Decimal::from_str(&rhs.to_string()).unwrap();
                *self = Self {
                    amount: self.amount * rhs,
                    currency: self.currency,
                };
            }
        }

        impl Div<$type> for Money {
            type Output = Money;

            fn div(self, rhs: $type) -> Money {
                let rhs = Decimal::from_str(&rhs.to_string()).unwrap();
                Money::from_decimal(self.amount / rhs, self.currency)
            }
        }

        impl Div<Money> for $type {
            type Output = Money;

            fn div(self, rhs: Money) -> Money {
                let lhs = Decimal::from_str(&self.to_string()).unwrap();
                Money::from_decimal(lhs / rhs.amount, rhs.currency)
            }
        }

        impl DivAssign<$type> for Money {
            fn div_assign(&mut self, rhs: $type) {
                let rhs = Decimal::from_str(&rhs.to_string()).unwrap();
                *self = Self {
                    amount: self.amount / rhs,
                    currency: self.currency,
                };
            }
        }
    };
}

impl_mul_div!(isize);
impl_mul_div!(i8);
impl_mul_div!(i16);
impl_mul_div!(i32);
impl_mul_div!(i64);
impl_mul_div!(usize);
impl_mul_div!(u8);
impl_mul_div!(u16);
impl_mul_div!(u32);
impl_mul_div!(u64);

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

impl Money {
    /// Creates a Money object given an integer and a currency reference.
    ///
    /// The integer represents minor units of the currency (e.g. 1000 -> 10.00 in USD )
    pub fn new(amount: i64, currency: &'static Currency) -> Money {
        Money::from_minor(amount, currency)
    }

    /// Creates a Money object given an integer and a currency reference.
    ///
    /// The integer represents minor units of the currency (e.g. 1000 -> 10.00 in USD )
    pub fn from_minor(amount: i64, currency: &'static Currency) -> Money {
        let amount = Decimal::new(amount, currency.exponent);
        Money { amount, currency }
    }

    /// Creates a Money object given an integer and a currency reference.
    ///
    /// The integer represents major units of the currency (e.g. 1000 -> 1,000 in USD )
    pub fn from_major(amount: i64, currency: &'static Currency) -> Money {
        let amount = Decimal::new(amount, 0);
        Money { amount, currency }
    }

    /// Creates a Money object given a decimal amount and a currency reference.
    pub fn from_decimal(amount: Decimal, currency: &'static Currency) -> Money {
        Money { amount, currency }
    }

    /// Creates a Money object given an amount str and a currency str.
    ///
    /// Supports fuzzy amount strings like "100", "100.00" and "-100.00"
    pub fn from_str(amount: &str, currency: &str) -> Result<Money, MoneyError> {
        Money::from_string(amount.to_string(), currency.to_string())
    }

    /// Creates a Money object given an amount string and a currency string.
    ///
    /// Supports fuzzy amount strings like "100", "100.00" and "-100.00"
    pub fn from_string(amount: String, currency: String) -> Result<Money, MoneyError> {
        let currency = Currency::from_string(currency)?;
        let amount_parts: Vec<&str> = amount.split(currency.exponent_separator).collect();

        let mut parsed_decimal = amount_parts[0].replace(currency.digit_separator, "");
        i32::from_str(&parsed_decimal)?;

        if amount_parts.len() == 1 {
            parsed_decimal += ".";
            for _ in 0..currency.exponent {
                parsed_decimal += "0";
            }
        } else if amount_parts.len() == 2 {
            i32::from_str(&amount_parts[1])?;
            parsed_decimal = parsed_decimal + "." + amount_parts[1];
        } else {
            return Err(MoneyError::InvalidAmount);
        }

        let decimal = Decimal::from_str(&parsed_decimal).unwrap();
        Ok(Money::from_decimal(decimal, currency))
    }

    /// Returns a reference to the Decimal amount.
    pub fn amount(&self) -> &Decimal {
        &self.amount
    }

    /// Returns the currency type.
    pub fn currency(&self) -> &'static Currency {
        self.currency
    }

    /// Returns true if the amount is == 0.
    pub fn is_zero(&self) -> bool {
        self.amount == dec!(0.0)
    }

    /// Returns true if the amount is > 0.
    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive() && self.amount != dec!(0.0)
    }

    /// Returns true if the amount is < 0.
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative() && self.amount != dec!(0.0)
    }

    /// Divides money equally into n shares.
    ///
    /// If the divison cannot be applied perfectly, it allocates the remainder
    /// to some of the shares.
    pub fn allocate_to(&self, number: i32) -> Result<Vec<Money>, MoneyError> {
        let ratios: Vec<i32> = (0..number).map(|_| 1).collect();
        self.allocate(ratios)
    }

    /// Divides money into n shares according to a particular ratio.
    ///  
    /// If the divison cannot be applied perfectly, it allocates the remainder
    /// to some of the shares.
    pub fn allocate(&self, ratios: Vec<i32>) -> Result<Vec<Money>, MoneyError> {
        if ratios.is_empty() {
            return Err(MoneyError::InvalidRatio);
        }

        let ratios: Vec<Decimal> = ratios
            .iter()
            .map(|x| Decimal::from_str(&x.to_string()).unwrap())
            .collect();

        let mut remainder = self.amount;
        let ratio_total: Decimal = ratios.iter().fold(dec!(0.0), |acc, x| acc + x);

        let mut allocations: Vec<Money> = Vec::new();

        for ratio in ratios {
            if ratio <= dec!(0.0) {
                return Err(MoneyError::InvalidRatio);
            }

            let share = (self.amount * ratio / ratio_total).floor();

            allocations.push(Money::from_decimal(share, self.currency));
            remainder -= share;
        }

        if remainder < dec!(0.0) {
            panic!("Remainder was negative, should be 0 or positive");
        }

        if remainder - remainder.floor() != dec!(0.0) {
            panic!("Remainder is not an integer, should be an integer");
        }

        let mut i: usize = 0;
        while remainder > dec!(0.0) {
            allocations[i].amount += dec!(1.0);
            remainder -= dec!(1.0);
            i += 1;
        }
        Ok(allocations)
    }

    /// Rounds the amount down to the currency's exponent.
    pub fn round(&mut self) {
        self.amount = self.amount.round_dp(self.currency.exponent);
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let currency = self.currency;

        let mut format_params = Params {
            digit_separator: currency.digit_separator,
            exponent_separator: currency.exponent_separator,
            separator_pattern: currency.digit_separator_sequence(),
            rounding: Some(currency.exponent),
            symbol: Some(currency.symbol),
            code: Some(currency.iso_alpha_code),
            ..Default::default()
        };

        if currency.symbol_first {
            format_params.positions = vec![Position::Sign, Position::Symbol, Position::Amount];
            write!(f, "{}", format_money(self, format_params))
        } else {
            format_params.positions = vec![Position::Sign, Position::Amount, Position::Symbol];
            write!(f, "{}", format_money(self, format_params))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Iso::*;

    #[test]
    fn money_major_minor() {
        let major_usd = Money::from_major(10, Currency::get(USD));
        let minor_usd = Money::from_minor(1000, Currency::get(USD));
        let new_usd = Money::new(1000, Currency::get(USD));
        assert_eq!(major_usd, minor_usd);
        assert_eq!(major_usd, new_usd);
    }

    #[test]
    fn money_from_string_parses_correctly() {
        let expected_money = Money::new(2999, Currency::get(GBP));
        let money = Money::from_string("29.99".to_string(), "GBP".to_string()).unwrap();
        assert_eq!(money, expected_money);
    }

    #[test]
    fn money_from_string_parses_signs() {
        let expected_money = Money::new(-300, Currency::get(GBP));
        let money = Money::from_string("-3".to_string(), "GBP".to_string()).unwrap();
        assert_eq!(money, expected_money);

        let expected_money = Money::new(300, Currency::get(GBP));
        let money = Money::from_string("+3".to_string(), "GBP".to_string()).unwrap();
        assert_eq!(money, expected_money);
    }

    #[test]
    fn money_from_string_ignores_separators() {
        let expected_money = Money::new(100000000, Currency::get(GBP));
        let money = Money::from_string("1,000,000".to_string(), "GBP".to_string()).unwrap();
        assert_eq!(money, expected_money);
    }

    #[test]
    fn money_from_string_parse_errs() {
        // If the delimiter preceeds the separators
        let money = Money::from_string("1.0000,000".to_string(), "GBP".to_string());
        assert_eq!(money.unwrap_err(), MoneyError::InvalidAmount);

        // If there are multiple delimiters
        let money = Money::from_string("1.0000.000".to_string(), "GBP".to_string());
        assert_eq!(money.unwrap_err(), MoneyError::InvalidAmount);

        // If there is an unrecognized character
        let money = Money::from_string("1.0000!000".to_string(), "GBP".to_string());
        assert_eq!(money.unwrap_err(), MoneyError::InvalidAmount);

        // If there are no characters other than separators
        let exponent_separator_only = Money::from_string(",".to_string(), "GBP".to_string());
        let amount_separator_only = Money::from_string(".".to_string(), "GBP".to_string());
        let both_separators = Money::from_string(",,.".to_string(), "GBP".to_string());
        assert_eq!(
            exponent_separator_only.unwrap_err(),
            MoneyError::InvalidAmount
        );
        assert_eq!(
            amount_separator_only.unwrap_err(),
            MoneyError::InvalidAmount
        );
        assert_eq!(both_separators.unwrap_err(), MoneyError::InvalidAmount);
    }

    #[test]
    fn money_format_rounds_exponent() {
        // // 19.999 rounds to 20 for USD
        let money = money!("19.9999", "USD");
        assert_eq!("$20.00", format!("{}", money));

        // // 29.111 rounds to 29.11 for USD
        let money = money!("29.111", "USD");
        assert_eq!("$29.11", format!("{}", money));

        // // 39.1155 rounds to 39.116 for BHD
        let money = money!("39.1155", "BHD");
        assert_eq!("ب.د39.116", format!("{}", money));
    }

    #[test]
    fn money_addition_and_subtraction() {
        // Addition
        assert_eq!(money!(2, "USD"), money!(1, "USD") + money!(1, "USD"));
        // Subtraction
        assert_eq!(money!(0, "USD"), money!(1, "USD") - money!(1, "USD"));
    }

    #[test]
    fn money_multiplication_and_division() {
        // Multiplication
        assert_eq!(money!(2, "USD"), money!(1, "USD") * 2);
        assert_eq!(money!(2, "USD"), money!(-1, "USD") * -2);
        assert_eq!(money!(2, "USD"), -2 * money!(-1, "USD"));

        // Division
        assert_eq!(money!(2, "USD"), money!(4, "USD") / 2);
        assert_eq!(money!(2, "USD"), money!(-4, "USD") / -2);
        assert_eq!(money!("0.5", "USD"), -1 / money!(-2, "USD"));
        assert_eq!(money!("2.0", "USD"), money!(-2, "USD") / -1);

        //MulAssign
        let mut money = money!(1, "USD");
        money *= 2;
        assert_eq!(money!(2, "USD"), money);

        //DivAssign
        let mut money = money!(1, "USD");
        money /= -2;
        assert_eq!(money!("-0.5", "USD"), money);
    }

    #[test]
    fn money_comparison() {
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
        let allocs = money.allocate(vec![1, 1, 1]).unwrap();
        let expected_results = vec![money!(4, "USD"), money!(4, "USD"), money!(3, "USD")];
        assert_eq!(expected_results, allocs);

        // Error if the ratio vector is empty
        let monies = money!(1, "USD").allocate(Vec::new());
        assert_eq!(monies.unwrap_err(), MoneyError::InvalidRatio);

        // Error if any ratio is zero
        let monies = money!(1, "USD").allocate(vec![1, 0]);
        assert_eq!(monies.unwrap_err(), MoneyError::InvalidRatio);
    }

    #[test]
    fn money_allocate_to() {
        let money = money!(11, "USD");
        let monies = money.allocate_to(3).unwrap();
        let expected_results = vec![money!(4, "USD"), money!(4, "USD"), money!(3, "USD")];
        assert_eq!(expected_results, monies);

        let monies = money!(1, "USD").allocate_to(0);
        assert_eq!(monies.unwrap_err(), MoneyError::InvalidRatio);
    }

    #[test]
    fn money_fmt_separates_digits() {
        let usd = money!(0, "USD"); // Zero Dollars
        let expected_usd_fmt = "$0.00";
        assert_eq!(format!("{}", usd), expected_usd_fmt);

        let usd = money!(100000, "USD"); // - One Hundred Thousand Dollars
        let expected_usd_fmt = "$100,000.00";
        assert_eq!(format!("{}", usd), expected_usd_fmt);

        let usd = money!(-100000, "USD"); // - One Hundred Thousand Dollars
        let expected_usd_fmt = "-$100,000.00";
        assert_eq!(format!("{}", usd), expected_usd_fmt);

        let usd = money!(1000000000, "USD"); // 1 Billion Dollars
        let expected_usd_fmt = "$1,000,000,000.00";
        assert_eq!(format!("{}", usd), expected_usd_fmt);

        let inr = money!(100000, "INR"); // 1 Lakh Rupees
        let expected_inr_fmt = "₹1,00,000.00";
        assert_eq!(format!("{}", inr), expected_inr_fmt);

        let inr = money!(-10000000, "INR"); // - 1 Crore Rupees
        let expected_inr_fmt = "-₹1,00,00,000.00";
        assert_eq!(format!("{}", inr), expected_inr_fmt);
    }

    #[test]
    fn money_fmt_places_symbols_correctly() {
        let money = money!(0, "USD");
        let expected_fmt = "$0.00";
        assert_eq!(format!("{}", money), expected_fmt);

        let money = money!(0, "AED");
        let expected_fmt = "0.00د.إ";
        assert_eq!(format!("{}", money), expected_fmt);
    }

    #[test]
    fn money_fmt_uses_correct_separators() {
        let money = money!(1000, "EUR");
        let expected_fmt = "€1.000,00";
        assert_eq!(format!("{}", money), expected_fmt);
    }

    #[test]
    // Dividing 20 by 3 rounds to 6.67 in USD and 6.667 in BHD
    fn money_precision_and_rounding() {
        let expected_money = money!("6.67", "USD");
        let mut money = money!("20.00", "USD");
        money /= 3;
        money.round();
        assert_eq!(money, expected_money);

        let expected_money = money!("6.667", "BHD");
        let mut money = money!("20", "BHD");
        money /= 3;
        money.round();
        assert_eq!(money, expected_money);
    }
}
