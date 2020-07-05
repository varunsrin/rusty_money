use crate::{Money, Round};
use std::cmp::Ordering;

/// A helper struct to convert Money objects into human readable strings.
pub struct Formatter;

impl Formatter {
    /// Returns a formatted Money String given parameters and a Money object.  
    pub fn money(money: &Money, params: Params) -> String {
        let mut decimal = *money.amount();

        // Round the decimal
        if let Some(x) = params.rounding {
            decimal = *money.round(x, Round::HalfEven).amount();
        }

        // Format the Amount String
        let amount = Formatter::amount(&format!("{}", decimal), &params);

        // Position values in the Output String
        let mut result = String::new();
        for position in params.positions.iter() {
            match position {
                Position::Space => result.push_str(" "),
                Position::Amount => result.push_str(&amount),
                Position::Code => result.push_str(params.code.unwrap_or("")),
                Position::Symbol => result.push_str(params.symbol.unwrap_or("")),
                Position::Sign => result.push_str(if money.is_negative() { "-" } else { "" }),
            }
        }
        result
    }

    /// Returns a formatted amount String, given the raw amount and formatting parameters.
    fn amount(raw_amount: &str, params: &Params) -> String {
        // Split amount into digits and exponent.
        let amount_split: Vec<&str> = raw_amount.split('.').collect();
        let mut amount_digits = amount_split[0].to_string();

        // Format the digits
        amount_digits.retain(|c| c != '-');
        amount_digits = Formatter::digits(
            &amount_digits,
            params.digit_separator,
            &params.separator_pattern,
        );
        let mut result = amount_digits;

        // Format the exponent, and add to digits
        match amount_split.len().cmp(&2) {
            Ordering::Equal => {
                // Exponent found, concatenate to digits.
                result.push(params.exponent_separator);
                result += amount_split[1];
            }
            Ordering::Less => {
                // No exponent, do nothing.
            }
            Ordering::Greater => panic!("More than 1 exponent separators when parsing Decimal"),
        }

        result
    }

    /// Returns a formatted digit component, given the digit string, separator and pattern of separation.
    fn digits(raw_digits: &str, separator: char, pattern: &[usize]) -> String {
        let mut digits = raw_digits.to_string();

        let mut current_position: usize = 0;
        for position in pattern.iter() {
            current_position += position;
            if digits.len() > current_position {
                digits.insert(digits.len() - current_position, separator);
                current_position += 1;
            }
        }
        digits
    }
}

/// Enumerates different items which can be positioned in a Money string.
#[derive(Debug, Clone)]
pub enum Position {
    Space,
    Amount,
    Code,
    Symbol,
    Sign,
}

/// A struct which contains parameters consumer by `Formatter`.
#[derive(Debug, Clone)]
pub struct Params {
    /// The character that separates grouped digits (e.g. 1,000,000)
    pub digit_separator: char,
    /// The character that separates minor units from major units (e.g. 1,000.00)
    pub exponent_separator: char,
    /// The grouping pattern that is applied to digits / major units (e.g. 1,000,000 vs 1,00,000)
    pub separator_pattern: Vec<usize>,
    /// The relative positions of the elements in a currency string (e.g. -$1,000 vs $ -1,000)
    pub positions: Vec<Position>,
    /// The number of minor unit digits should remain after Round::HalfEven is applied.
    pub rounding: Option<u32>,
    /// The symbol of the currency (e.g. $)
    pub symbol: Option<&'static str>,
    /// The currency's ISO code (e.g. USD)
    pub code: Option<&'static str>,
}

impl Default for Params {
    /// Defines the default parameters to format a Money string.
    fn default() -> Params {
        Params {
            digit_separator: ',',
            exponent_separator: '.',
            separator_pattern: vec![3, 3, 3],
            positions: vec![Position::Sign, Position::Symbol, Position::Amount],
            rounding: None,
            symbol: None,
            code: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::IsoCurrency;
    use crate::Iso::*;

    #[test]
    fn format_position() {
        let money = Money::from_major(-1000, IsoCurrency::get(USD));

        // Test that you can position Space, Amount, Code, Symbol and Sign in different places
        let params = Params {
            symbol: Some("$"),
            code: Some("USD"),
            positions: vec![
                Position::Sign,
                Position::Space,
                Position::Symbol,
                Position::Amount,
                Position::Space,
                Position::Code,
            ],
            ..Default::default()
        };
        assert_eq!("- $1,000 USD", Formatter::money(&money, params));

        let params = Params {
            symbol: Some("$"),
            code: Some("USD"),
            positions: vec![
                Position::Code,
                Position::Space,
                Position::Amount,
                Position::Symbol,
                Position::Space,
                Position::Sign,
            ],
            ..Default::default()
        };
        assert_eq!("USD 1,000$ -", Formatter::money(&money, params));

        // Test that you can omit some, and it works fine.
        let params = Params {
            positions: vec![Position::Amount],
            ..Default::default()
        };
        assert_eq!("1,000", Formatter::money(&money, params));

        let params = Params {
            symbol: Some("$"),
            positions: vec![Position::Symbol],
            ..Default::default()
        };
        assert_eq!("$", Formatter::money(&money, params));

        // Missing Optionals Insert Nothing
        let params = Params {
            positions: vec![Position::Amount, Position::Symbol],
            ..Default::default()
        };
        assert_eq!("1,000", Formatter::money(&money, params));
    }

    #[test]
    fn format_digit_separators_with_custom_separators() {
        let params = Params {
            digit_separator: '/',
            ..Default::default()
        };

        // For 1_000_000
        let money = Money::from_major(1_000_000, IsoCurrency::get(USD));
        assert_eq!("1/000/000", Formatter::money(&money, params.clone()));

        // For 1_000
        let money = Money::from_major(1_000, IsoCurrency::get(USD));
        assert_eq!("1/000", Formatter::money(&money, params.clone()));

        // For 0 Chars
        let money = Money::from_major(0, IsoCurrency::get(USD));
        assert_eq!("0", Formatter::money(&money, params.clone()));
    }

    #[test]
    fn format_digit_separators_with_custom_sequences() {
        let params = Params {
            separator_pattern: vec![3, 2, 2],
            ..Default::default()
        };

        let money = Money::from_major(1_00_00_000, IsoCurrency::get(USD));
        assert_eq!("1,00,00,000", Formatter::money(&money, params.clone()));

        let money = Money::from_major(1_00_000, IsoCurrency::get(USD));
        assert_eq!("1,00,000", Formatter::money(&money, params.clone()));

        let money = Money::from_major(1_000, IsoCurrency::get(USD));
        assert_eq!("1,000", Formatter::money(&money, params.clone()));

        // With a zero sequence
        let params = Params {
            separator_pattern: vec![0, 2],
            ..Default::default()
        };

        let money = Money::from_major(100,IsoCurrency::get(USD));
        assert_eq!("1,00,", Formatter::money(&money, params.clone()));

        let money = Money::from_major(0, IsoCurrency::get(USD));
        assert_eq!("0,", Formatter::money(&money, params.clone()));
    }

    // What if pattern includes a zero or negative number?

    #[test]
    fn format_rounding() {
        let money = Money::new(1000, IsoCurrency::get(USD)) / 3;

        // Rounding = Some (0)
        let params = Params {
            rounding: Some(0),
            ..Default::default()
        };
        assert_eq!("3", Formatter::money(&money, params));

        // Rounding = Some(2)
        let params = Params {
            rounding: Some(2),
            ..Default::default()
        };
        assert_eq!("3.33", Formatter::money(&money, params));

        // Rounding = None
        let params = Params {
            ..Default::default()
        };
        assert_eq!(
            "3.3333333333333333333333333333",
            Formatter::money(&money, params)
        );
    }
}
