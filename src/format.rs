use arrayvec::ArrayString;
use rust_decimal::Decimal;

use crate::currency::FormattableCurrency;
use crate::{Money, Round};
use std::fmt::{self, Write};

/// Converts Money objects into human readable strings.
pub struct Formatter;

impl Formatter {
    /// Returns a formatted Money String given parameters and a Money object.
    pub fn money<'a, T: FormattableCurrency>(money: &Money<'a, T>, params: Params<'_>) -> String {
        let mut result = String::with_capacity(32);
        Self::write_money(money, params, &mut result).expect("writing to a String cannot fail");
        result
    }

    pub(crate) fn write_money<T: FormattableCurrency, W: Write + ?Sized>(
        money: &Money<'_, T>,
        params: Params<'_>,
        output: &mut W,
    ) -> fmt::Result {
        let mut decimal = *money.amount();

        // Round the decimal and ensure it has the correct scale
        if let Some(x) = params.rounding {
            // Keep rescale within Decimal's supported range, even if a dependency
            // version accepts larger scales. The formatter buffer relies on this bound.
            let x = x.min(Decimal::MAX_SCALE);
            decimal = *money.round(x, Round::HalfEven).amount();
            decimal.rescale(x);
        }

        let raw_amount = Self::unsigned_decimal_text(decimal)?;
        let (digits, fraction) = raw_amount
            .split_once('.')
            .map_or((raw_amount.as_str(), None), |(digits, fraction)| {
                (digits, Some(fraction))
            });

        // Position values in the Output String
        for position in params.positions.iter() {
            match position {
                Position::Space => output.write_char(' ')?,
                Position::Amount => {
                    Self::write_digits(digits, &params, output)?;
                    if let Some(fraction) = fraction {
                        output.write_char(params.exponent_separator)?;
                        output.write_str(fraction)?;
                    }
                }
                Position::Code => output.write_str(params.code.unwrap_or(""))?,
                Position::Symbol => output.write_str(params.symbol.unwrap_or(""))?,
                Position::Sign => {
                    if money.is_negative() {
                        output.write_str("-")?;
                    } else {
                        output.write_str("")?;
                    }
                }
            }
        }
        Ok(())
    }

    fn write_digits<W: Write + ?Sized>(
        digits: &str,
        params: &Params<'_>,
        output: &mut W,
    ) -> fmt::Result {
        let mut grouped = 0;
        let mut count = 0;
        for &width in params.separator_pattern {
            if width >= digits.len() - grouped {
                break;
            }
            grouped += width;
            count += 1;
        }

        let mut start = digits.len() - grouped;
        output.write_str(&digits[..start])?;
        for &width in params.separator_pattern[..count].iter().rev() {
            output.write_char(params.digit_separator)?;
            output.write_str(&digits[start..start + width])?;
            start += width;
        }
        Ok(())
    }

    // Decimal has at most 29 coefficient digits and scale <= 28. Its unsigned
    // fixed-point representation therefore needs at most 30 bytes ("0." + 28
    // fractional digits). Both buffers have spare capacity; all writes remain
    // fallible. The sign is handled separately from the original Money amount.
    fn unsigned_decimal_text(decimal: Decimal) -> Result<ArrayString<32>, fmt::Error> {
        let mut coefficient = ArrayString::<32>::new();
        write!(&mut coefficient, "{}", decimal.mantissa().unsigned_abs())?;
        let scale = decimal.scale() as usize;
        if scale == 0 {
            return Ok(coefficient);
        }

        let mut result = ArrayString::<32>::new();
        if coefficient.len() > scale {
            let boundary = coefficient.len() - scale;
            result.write_str(&coefficient[..boundary])?;
            result.write_char('.')?;
            result.write_str(&coefficient[boundary..])?;
        } else {
            result.write_str("0.")?;
            for _ in coefficient.len()..scale {
                result.write_char('0')?;
            }
            result.write_str(&coefficient)?;
        }
        Ok(result)
    }
}

/// Items which must be positioned in a Money string.
#[derive(Debug, Clone)]
pub enum Position {
    Space,
    Amount,
    Code,
    Symbol,
    Sign,
}

/// Group of formatting parameters consumed by `Formatter`.
#[derive(Debug, Clone)]
pub struct Params<'a> {
    /// The character that separates grouped digits (e.g. 1,000,000)
    pub digit_separator: char,
    /// The character that separates minor units from major units (e.g. 1,000.00)
    pub exponent_separator: char,
    /// The grouping pattern that is applied to digits / major units (e.g. 1,000,000 vs 1,00,000)
    /// Steps count digits from right to left. A zero step repeats the current
    /// boundary; a leading zero places a separator after the final digit.
    pub separator_pattern: &'a [usize],
    /// The relative positions of the elements in a currency string (e.g. -$1,000 vs $ -1,000)
    pub positions: &'a [Position],
    /// The number of minor unit digits should remain after Round::HalfEven is applied.
    /// Values above Decimal::MAX_SCALE (28) are capped at that limit.
    pub rounding: Option<u32>,
    /// The symbol of the currency (e.g. $)
    pub symbol: Option<&'static str>,
    /// The currency's ISO code (e.g. USD)
    pub code: Option<&'static str>,
}

// Default patterns as static arrays for zero-allocation formatting
const DEFAULT_SEPARATOR_PATTERN: &[usize] = &[3, 3, 3];
const DEFAULT_POSITIONS: &[Position] = &[Position::Sign, Position::Symbol, Position::Amount];

impl Default for Params<'_> {
    /// Defines the default parameters to format a Money string.
    fn default() -> Self {
        Params {
            digit_separator: ',',
            exponent_separator: '.',
            separator_pattern: DEFAULT_SEPARATOR_PATTERN,
            positions: DEFAULT_POSITIONS,
            rounding: None,
            symbol: None,
            code: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::define_currency_set;

    define_currency_set!(
        test {
            USD: {
                code: "USD",
                exponent: 2,
                locale: EnUs,
                minor_units: 100,
                name: "USD",
                symbol: "$",
                symbol_first: true,
            }
        }
    );

    #[test]
    fn format_position() {
        let _usd = test::find("USD"); // Prevents unused code warnings from the defined module.

        let money = Money::from_major(-1000, test::USD);

        // Test that you can position eSpace, Amount, Code, Symbol and Sign in different places
        let params = Params {
            symbol: Some("$"),
            code: Some("USD"),
            positions: &[
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
            positions: &[
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
            positions: &[Position::Amount],
            ..Default::default()
        };
        assert_eq!("1,000", Formatter::money(&money, params));

        let params = Params {
            symbol: Some("$"),
            positions: &[Position::Symbol],
            ..Default::default()
        };
        assert_eq!("$", Formatter::money(&money, params));

        // Missing Optionals Insert Nothing
        let params = Params {
            positions: &[Position::Amount, Position::Symbol],
            ..Default::default()
        };
        assert_eq!("1,000", Formatter::money(&money, params));

        // Sign between symbol and amount
        let params = Params {
            symbol: Some("$"),
            positions: &[Position::Symbol, Position::Sign, Position::Amount],
            ..Default::default()
        };
        assert_eq!("$-1,000", Formatter::money(&money, params));
    }

    fn format_grouped_amount(amount: i64, separator: char, pattern: &[usize]) -> String {
        let money = Money::from_major(amount, test::USD);
        Formatter::money(
            &money,
            Params {
                digit_separator: separator,
                separator_pattern: pattern,
                ..Params::default()
            },
        )
    }

    #[test]
    fn grouping_handles_multibyte_separators() {
        for separator in [',', '\u{a0}', '\u{202f}', '💰'] {
            assert_eq!(
                format_grouped_amount(1234567, separator, &[3, 3, 3]),
                format!("1{separator}234{separator}567")
            );
            assert_eq!(
                format_grouped_amount(123456789, separator, &[3, 2, 2]),
                format!("12{separator}34{separator}56{separator}789")
            );
            assert_eq!(format_grouped_amount(123, separator, &[3, 3, 3]), "123");
        }
        let money = Money::from_major(-1234567, test::USD);
        let params = Params {
            digit_separator: '\u{202f}',
            symbol: Some("$"),
            rounding: Some(2),
            ..Params::default()
        };
        assert_eq!(
            Formatter::money(&money, params),
            "-$1\u{202f}234\u{202f}567.00"
        );
    }

    #[test]
    fn grouping_zero_steps_repeat_at_the_same_boundary() {
        for separator in [',', '\u{202f}', '💰'] {
            assert_eq!(
                format_grouped_amount(1234567, separator, &[0, 0, 3]),
                format!("1234{separator}567{separator}{separator}")
            );
            assert_eq!(
                format_grouped_amount(1234567, separator, &[3, 0, 3]),
                format!("1{separator}234{separator}{separator}567")
            );
        }
    }

    #[test]
    fn grouping_oversized_steps_stop_without_overflow() {
        assert_eq!(
            format_grouped_amount(1234567, ',', &[usize::MAX]),
            "1234567"
        );
        assert_eq!(
            format_grouped_amount(1234567, ',', &[3, usize::MAX, 1]),
            "1234,567"
        );
        assert_eq!(format_grouped_amount(1234567, '\u{202f}', &[]), "1234567");
    }

    #[test]
    fn format_digit_separators_with_custom_separators() {
        let params = Params {
            digit_separator: '/',
            ..Default::default()
        };

        // For 1_000_000
        let money = Money::from_major(1_000_000, test::USD);
        assert_eq!("1/000/000", Formatter::money(&money, params.clone()));

        // For 1_000
        let money = Money::from_major(1_000, test::USD);
        assert_eq!("1/000", Formatter::money(&money, params.clone()));

        // For 0 Chars
        let money = Money::from_major(0, test::USD);
        assert_eq!("0", Formatter::money(&money, params));

        // European style: swap digit and exponent separators
        let params = Params {
            rounding: Some(2),
            exponent_separator: ',',
            digit_separator: '.',
            ..Default::default()
        };
        let money = Money::from_minor(123456, test::USD);
        assert_eq!("1.234,56", Formatter::money(&money, params));
    }

    #[test]
    fn format_digit_separators_with_custom_sequences() {
        // Indian-style numbering: 3,2,2 pattern (e.g., 1,00,00,000)
        let params = Params {
            separator_pattern: &[3, 2, 2],
            ..Default::default()
        };

        let money = Money::from_major(10_000_000, test::USD);
        assert_eq!("1,00,00,000", Formatter::money(&money, params.clone()));

        let money = Money::from_major(100_000, test::USD);
        assert_eq!("1,00,000", Formatter::money(&money, params.clone()));

        let money = Money::from_major(1_000, test::USD);
        assert_eq!("1,000", Formatter::money(&money, params));
    }

    #[test]
    fn format_zero_amount() {
        let params = Params {
            symbol: Some("$"),
            positions: &[Position::Sign, Position::Symbol, Position::Amount],
            ..Default::default()
        };

        let money = Money::from_major(0, test::USD);
        // Zero should not have a sign
        assert_eq!("$0", Formatter::money(&money, params));
    }

    #[test]
    fn format_rounding() {
        let money = Money::from_minor(1000, test::USD).div(3).unwrap();

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

    #[test]
    fn format_preserves_rounding_carry_sign_and_scale() {
        for (amount, expected) in [
            ("999.995", "1,000.00"),
            ("-999.995", "-1,000.00"),
            ("-0.001", "-0.00"),
            ("0", "0.00"),
            ("1.5", "1.50"),
        ] {
            let decimal = amount.parse().unwrap();
            let money = Money::from_decimal(decimal, test::USD);
            let params = Params {
                rounding: Some(2),
                ..Default::default()
            };
            assert_eq!(Formatter::money(&money, params), expected);
            assert_eq!(money.amount().serialize(), decimal.serialize());
        }
    }

    #[test]
    fn format_caps_precision_at_decimal_maximum() {
        let money = Money::from_decimal(Decimal::from_parts(0, 0, 1, false, 22), test::USD);
        for digits in [28, 29, 31, 32, u32::MAX] {
            let params = Params {
                rounding: Some(digits),
                ..Default::default()
            };
            assert_eq!(
                Formatter::money(&money, params),
                "0.0018446744073709551616000000"
            );
        }
    }

    #[test]
    fn format_preserves_custom_grouping() {
        for (amount, pattern, separator, expected) in [
            (1_234_567_890, &[3, 2, 2][..], ',', "123,45,67,890"),
            (1_234, &[][..], ',', "1234"),
            (1_234, &[0, 3][..], ',', "1,234,"),
            (1_234, &[4, 3][..], ',', "1234"),
            (1_234_567, &[3, 3, 3][..], '\u{a0}', "1\u{a0}234\u{a0}567"),
        ] {
            let money = Money::from_major(amount, test::USD);
            let params = Params {
                separator_pattern: pattern,
                digit_separator: separator,
                ..Default::default()
            };
            // Preserve finite grouping patterns and correct multibyte grouping.
            assert_eq!(Formatter::money(&money, params), expected);
        }
    }

    #[test]
    fn format_preserves_repeated_amount_positions() {
        let money = Money::from_minor(-123_450, test::USD);
        let params = Params {
            positions: &[
                Position::Amount,
                Position::Space,
                Position::Sign,
                Position::Amount,
                Position::Symbol,
            ],
            symbol: Some("€"),
            exponent_separator: ',',
            digit_separator: '.',
            rounding: Some(2),
            ..Default::default()
        };
        assert_eq!(Formatter::money(&money, params), "1.234,50 -1.234,50€");
    }

    #[test]
    fn decimal_text_preserves_coefficient_boundaries_and_every_scale() {
        for coefficient in [
            0,
            1,
            9,
            10,
            99,
            100,
            u32::MAX as u128,
            u64::MAX as u128,
            (1u128 << 96) - 1,
        ] {
            for scale in 0..=28 {
                for negative in [false, true] {
                    let decimal = Decimal::from_parts(
                        coefficient as u32,
                        (coefficient >> 32) as u32,
                        (coefficient >> 64) as u32,
                        negative,
                        scale,
                    );
                    let expected = decimal.to_string();
                    assert_eq!(
                        Formatter::unsigned_decimal_text(decimal).unwrap().as_str(),
                        expected.trim_start_matches('-'),
                    );
                }
            }
        }
    }

    // Differential reference: keep the previous Decimal Display + string
    // insertion algorithm independent of the new integer/streaming path.
    fn reference_amount(decimal: Decimal, separator: char, pattern: &[usize]) -> String {
        let raw = decimal.to_string();
        let unsigned = raw.trim_start_matches('-');
        let (integer, fraction) = unsigned
            .split_once('.')
            .map_or((unsigned, None), |(i, f)| (i, Some(f)));
        let mut digits = integer.to_string();
        let mut position = 0usize;
        for &width in pattern {
            let Some(next) = position.checked_add(width) else {
                break;
            };
            position = next;
            if digits.len() > position {
                digits.insert(digits.len() - position, separator);
                position += separator.len_utf8();
            }
        }
        if let Some(fraction) = fraction {
            digits.push('.');
            digits.push_str(fraction);
        }
        digits
    }

    proptest::proptest! {
        #[test]
        fn streaming_matches_decimal_and_insertion_reference(
            words in proptest::array::uniform3(proptest::prelude::any::<u32>()),
            negative in proptest::prelude::any::<bool>(),
            scale in 0u32..=28,
            rounding in proptest::option::of(0u32..=32),
            separator in proptest::sample::select(vec![',', '.', '\u{a0}', '\u{202f}', '💰']),
            pattern in proptest::collection::vec(
                proptest::prop_oneof![0usize..=32, proptest::strategy::Just(usize::MAX)], 0..12),
        ) {
            let decimal = Decimal::from_parts(words[0], words[1], words[2], negative, scale);
            let money = Money::from_decimal(decimal, test::USD);
            let mut expected_decimal = decimal;
            if let Some(digits) = rounding {
                let digits = digits.min(Decimal::MAX_SCALE);
                expected_decimal = decimal.round_dp_with_strategy(digits, rust_decimal::RoundingStrategy::MidpointNearestEven);
                expected_decimal.rescale(digits);
            }
            let amount = reference_amount(expected_decimal, separator, &pattern);
            let sign = if money.is_negative() { "-" } else { "" };
            let params = Params {
                digit_separator: separator,
                separator_pattern: &pattern,
                rounding,
                positions: &[Position::Amount, Position::Space, Position::Sign, Position::Amount],
                ..Params::default()
            };
            proptest::prop_assert_eq!(Formatter::money(&money, params), format!("{amount} {sign}{amount}"));
        }
    }

    #[test]
    fn writing_propagates_output_errors() {
        struct FailingWriter {
            remaining: usize,
        }
        impl std::fmt::Write for FailingWriter {
            fn write_str(&mut self, text: &str) -> std::fmt::Result {
                if text.len() > self.remaining {
                    return Err(std::fmt::Error);
                }
                self.remaining -= text.len();
                Ok(())
            }
        }
        let money = Money::from_major(1_234_567, test::USD);
        // Fail immediately or after part of the grouped amount has been written.
        for separator in [',', '\u{202f}', '💰'] {
            for remaining in [0, 1, 4, 8] {
                let mut writer = FailingWriter { remaining };
                let params = Params {
                    digit_separator: separator,
                    ..Params::default()
                };
                assert!(Formatter::write_money(&money, params, &mut writer).is_err());
            }
        }
    }
}

/// Golden tests for format output stability.
/// These tests document expected Display output for real currencies.
/// If these change, it's a breaking change for users.
#[cfg(all(test, feature = "iso"))]
mod golden_tests {
    use crate::Money;
    use crate::iso;

    #[test]
    fn usd_format_golden() {
        // US Dollar: symbol first, comma digit separator, period decimal
        assert_eq!(format!("{}", Money::from_minor(0, iso::USD)), "$0.00");
        assert_eq!(format!("{}", Money::from_minor(1, iso::USD)), "$0.01");
        assert_eq!(format!("{}", Money::from_minor(100, iso::USD)), "$1.00");
        assert_eq!(
            format!("{}", Money::from_minor(123456, iso::USD)),
            "$1,234.56"
        );
        assert_eq!(
            format!("{}", Money::from_minor(123456789, iso::USD)),
            "$1,234,567.89"
        );
        // Negative amounts
        assert_eq!(format!("{}", Money::from_minor(-100, iso::USD)), "-$1.00");
        assert_eq!(
            format!("{}", Money::from_minor(-123456, iso::USD)),
            "-$1,234.56"
        );
    }

    #[test]
    fn eur_format_golden() {
        // Euro: European locale - period digit separator, comma decimal
        assert_eq!(format!("{}", Money::from_minor(0, iso::EUR)), "€0,00");
        assert_eq!(
            format!("{}", Money::from_minor(123456, iso::EUR)),
            "€1.234,56"
        );
        assert_eq!(
            format!("{}", Money::from_minor(-123456, iso::EUR)),
            "-€1.234,56"
        );
    }

    #[test]
    fn gbp_format_golden() {
        // British Pound: US-style formatting
        assert_eq!(format!("{}", Money::from_minor(0, iso::GBP)), "£0.00");
        assert_eq!(
            format!("{}", Money::from_minor(123456, iso::GBP)),
            "£1,234.56"
        );
    }

    #[test]
    fn jpy_format_golden() {
        // Japanese Yen: no decimal places (exponent 0)
        assert_eq!(format!("{}", Money::from_minor(0, iso::JPY)), "¥0");
        assert_eq!(format!("{}", Money::from_minor(1, iso::JPY)), "¥1");
        assert_eq!(format!("{}", Money::from_minor(1234, iso::JPY)), "¥1,234");
        assert_eq!(
            format!("{}", Money::from_minor(1234567, iso::JPY)),
            "¥1,234,567"
        );
    }

    #[test]
    fn inr_format_golden() {
        // Indian Rupee: Indian numbering (lakhs, crores) - 2,2,3 pattern
        assert_eq!(format!("{}", Money::from_minor(0, iso::INR)), "₹0.00");
        assert_eq!(format!("{}", Money::from_minor(100, iso::INR)), "₹1.00");
        // 1,00,000 (1 lakh)
        assert_eq!(
            format!("{}", Money::from_minor(10000000, iso::INR)),
            "₹1,00,000.00"
        );
        // 1,00,00,000 (1 crore)
        assert_eq!(
            format!("{}", Money::from_minor(1000000000, iso::INR)),
            "₹1,00,00,000.00"
        );
    }

    #[test]
    fn bhd_format_golden() {
        // Bahraini Dinar: 3 decimal places (exponent 3), Arabic symbol
        assert_eq!(format!("{}", Money::from_minor(0, iso::BHD)), "د.ب0.000");
        assert_eq!(format!("{}", Money::from_minor(1, iso::BHD)), "د.ب0.001");
        assert_eq!(format!("{}", Money::from_minor(1000, iso::BHD)), "د.ب1.000");
        assert_eq!(
            format!("{}", Money::from_minor(1234567, iso::BHD)),
            "د.ب1,234.567"
        );
    }

    #[test]
    fn byn_format_golden() {
        // Belarusian Ruble: symbol after amount, space digit separator, comma decimal (EnBy locale)
        assert_eq!(format!("{}", Money::from_minor(0, iso::BYN)), "0,00Br");
        assert_eq!(
            format!("{}", Money::from_minor(123456, iso::BYN)),
            "1 234,56Br"
        );
        assert_eq!(
            format!("{}", Money::from_minor(123456789, iso::BYN)),
            "1 234 567,89Br"
        );
    }
}
