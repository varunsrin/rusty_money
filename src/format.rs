use crate::Money;

pub fn format_money(money: &Money, params: Params) -> String {
    // Round the decimal
    let mut decimal = *money.amount();
    if let Some(x) = params.rounding {
        decimal = decimal.round_dp(x);
    }

    // Format the Amount String
    let amount = format_amount(&format!("{}", decimal), &params);

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

/// Returns a formatted amount string, given the raw amount and params.rust_decimal
fn format_amount(raw_amount: &str, params: &Params) -> String {
    // Split amount into digits and exponent.
    let amount_split: Vec<&str> = raw_amount.split('.').collect();
    let mut amount_digits = amount_split[0].to_string();

    // Format the digits
    amount_digits.retain(|c| c != '-');
    amount_digits = format_digits(
        &amount_digits,
        &params.digit_separator,
        &params.separator_pattern,
    );
    let mut result = amount_digits;

    // Format the exponent, and add to digits
    if amount_split.len() == 2 {
        result.push(params.exponent_separator);
        result += amount_split[1];
    } else if amount_split.len() > 2 {
        panic!("More than 1 exponent separators when parsing Decimal")
    }

    result
}

/// Returns a formatted digit component, given the digit string, separator and pattern of separation.
fn format_digits(raw_digits: &str, separator: &char, pattern: &Vec<usize>) -> String {
    let mut digits = raw_digits.to_string();

    let mut current_position: usize = 0;
    for position in pattern.iter() {
        current_position += position;
        if digits.len() > current_position {
            digits.insert(digits.len() - current_position, *separator);
            current_position += 1;
        }
    }
    digits
}

#[derive(Debug, Clone)]
pub enum Position {
    Space,
    Amount,
    Code,
    Symbol,
    Sign,
}

#[derive(Debug, Clone)]
pub struct Params {
    pub digit_separator: char,
    pub exponent_separator: char,
    pub separator_pattern: Vec<usize>,
    pub positions: Vec<Position>,
    pub rounding: Option<u32>,
    pub symbol: Option<&'static str>,
    pub code: Option<&'static str>,
}

impl Default for Params {
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
    use crate::currency::Currency;
    use crate::Iso::*;

    #[test]
    fn format_position() {
        let money = Money::from_major(-1000, Currency::get(USD));

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
        assert_eq!("- $1,000 USD", format_money(&money, params));

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
        assert_eq!("USD 1,000$ -", format_money(&money, params));

        // Test that you can omit some, and it works fine.
        let params = Params {
            positions: vec![Position::Amount],
            ..Default::default()
        };
        assert_eq!("1,000", format_money(&money, params));

        let params = Params {
            symbol: Some("$"),
            positions: vec![Position::Symbol],
            ..Default::default()
        };
        assert_eq!("$", format_money(&money, params));

        // Missing Optionals Insert Nothing
        let params = Params {
            positions: vec![Position::Amount, Position::Symbol],
            ..Default::default()
        };
        assert_eq!("1,000", format_money(&money, params));
    }

    #[test]
    fn format_digit_separators_with_custom_separators() {
        let params = Params {
            digit_separator: '/',
            ..Default::default()
        };

        // For 1_000_000
        let money = Money::from_major(1_000_000, Currency::get(USD));
        assert_eq!("1/000/000", format_money(&money, params.clone()));

        // For 1_000
        let money = Money::from_major(1_000, Currency::get(USD));
        assert_eq!("1/000", format_money(&money, params.clone()));

        // For 0 Chars
        let money = Money::from_major(0, Currency::get(USD));
        assert_eq!("0", format_money(&money, params.clone()));
    }

    #[test]
    fn format_digit_separators_with_custom_sequences() {
        let params = Params {
            separator_pattern: vec![3, 2, 2],
            ..Default::default()
        };

        let money = Money::from_major(1_00_00_000, Currency::get(USD));
        assert_eq!("1,00,00,000", format_money(&money, params.clone()));

        let money = Money::from_major(1_00_000, Currency::get(USD));
        assert_eq!("1,00,000", format_money(&money, params.clone()));

        let money = Money::from_major(1_000, Currency::get(USD));
        assert_eq!("1,000", format_money(&money, params.clone()));

        // With a zero sequence
        let params = Params {
            separator_pattern: vec![0, 2],
            ..Default::default()
        };

        let money = Money::from_major(100, Currency::get(USD));
        assert_eq!("1,00,", format_money(&money, params.clone()));

        let money = Money::from_major(0, Currency::get(USD));
        assert_eq!("0,", format_money(&money, params.clone()));
    }

    // What if pattern includes a zero or negative number?

    #[test]
    fn format_rounding() {
        let money = Money::new(1000, Currency::get(USD)) / 3;

        // Rounding = Some (0)
        let params = Params {
            rounding: Some(0),
            ..Default::default()
        };
        assert_eq!("3", format_money(&money, params));

        // Rounding = Some(2)
        let params = Params {
            rounding: Some(2),
            ..Default::default()
        };
        assert_eq!("3.33", format_money(&money, params));

        // Rounding = None
        let params = Params {
            ..Default::default()
        };
        assert_eq!(
            "3.3333333333333333333333333333",
            format_money(&money, params)
        );
    }
}
