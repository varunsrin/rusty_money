use crate::MoneyError;
use crate::currency::FormattableCurrency;
use crate::format::{Formatter, Params, Position};
use crate::locale::LocalFormat;

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::str::FromStr;

use rust_decimal::Decimal;

/// Represents an amount of a given currency.
///
/// Money represents financial amounts through a Decimal (owned) and a Currency (reference).
/// Operations on Money objects always create new instances of Money, with the exception
/// of `round()`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Money<'a, T: FormattableCurrency> {
    amount: Decimal,
    currency: &'a T,
}

impl<'a, T: FormattableCurrency> Add for Money<'a, T> {
    type Output = Money<'a, T>;
    fn add(self, other: Money<'a, T>) -> Money<'a, T> {
        if self.currency != other.currency {
            panic!(
                "Cannot add Money with currency {} to Money with currency {}",
                self.currency.code(),
                other.currency.code()
            );
        }
        Money::from_decimal(self.amount + other.amount, self.currency)
    }
}

impl<'a, T: FormattableCurrency> AddAssign for Money<'a, T> {
    fn add_assign(&mut self, other: Self) {
        if self.currency != other.currency {
            panic!(
                "Cannot add Money with currency {} to Money with currency {}",
                self.currency.code(),
                other.currency.code()
            );
        }
        *self = Self {
            amount: self.amount + other.amount,
            currency: self.currency,
        };
    }
}

impl<'a, T: FormattableCurrency> Sub for Money<'a, T> {
    type Output = Money<'a, T>;
    fn sub(self, other: Money<'a, T>) -> Money<'a, T> {
        if self.currency != other.currency {
            panic!(
                "Cannot subtract Money with currency {} from Money with currency {}",
                other.currency.code(),
                self.currency.code()
            );
        }
        Money::from_decimal(self.amount - other.amount, self.currency)
    }
}

impl<'a, T: FormattableCurrency> SubAssign for Money<'a, T> {
    fn sub_assign(&mut self, other: Self) {
        if self.currency != other.currency {
            panic!(
                "Cannot subtract Money with currency {} from Money with currency {}",
                other.currency.code(),
                self.currency.code()
            );
        }
        *self = Self {
            amount: self.amount - other.amount,
            currency: self.currency,
        };
    }
}

impl<'a, T: FormattableCurrency> Neg for Money<'a, T> {
    type Output = Money<'a, T>;

    fn neg(self) -> Self::Output {
        Money {
            amount: -self.amount,
            currency: self.currency,
        }
    }
}

macro_rules! impl_mul_div {
    ($type:ty) => {
        impl<'a, T: FormattableCurrency> Mul<$type> for Money<'a, T> {
            type Output = Money<'a, T>;

            fn mul(self, rhs: $type) -> Money<'a, T> {
                Money::from_decimal(self.amount * Decimal::from(rhs), self.currency)
            }
        }

        impl<'a, T: FormattableCurrency> Mul<Money<'a, T>> for $type {
            type Output = Money<'a, T>;

            fn mul(self, rhs: Money<'a, T>) -> Money<'a, T> {
                Money::from_decimal(rhs.amount * Decimal::from(self), rhs.currency)
            }
        }

        impl<'a, T: FormattableCurrency> MulAssign<$type> for Money<'a, T> {
            fn mul_assign(&mut self, rhs: $type) {
                *self = Self {
                    amount: self.amount * Decimal::from(rhs),
                    currency: self.currency,
                };
            }
        }

        impl<'a, T: FormattableCurrency> Div<$type> for Money<'a, T> {
            type Output = Money<'a, T>;

            fn div(self, rhs: $type) -> Money<'a, T> {
                Money::from_decimal(self.amount / Decimal::from(rhs), self.currency)
            }
        }

        impl<'a, T: FormattableCurrency> Div<Money<'a, T>> for $type {
            type Output = Money<'a, T>;

            fn div(self, rhs: Money<'a, T>) -> Money<'a, T> {
                Money::from_decimal(Decimal::from(self) / rhs.amount, rhs.currency)
            }
        }

        impl<'a, T: FormattableCurrency> DivAssign<$type> for Money<'a, T> {
            fn div_assign(&mut self, rhs: $type) {
                *self = Self {
                    amount: self.amount / Decimal::from(rhs),
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
impl_mul_div!(Decimal);

impl<'a, T: FormattableCurrency> PartialOrd for Money<'a, T> {
    fn partial_cmp(&self, other: &Money<'a, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, T: FormattableCurrency> Ord for Money<'a, T> {
    fn cmp(&self, other: &Money<'a, T>) -> Ordering {
        if self.currency != other.currency {
            panic!(
                "Cannot compare Money with currency {} to Money with currency {}",
                self.currency.code(),
                other.currency.code()
            );
        }
        self.amount.cmp(&other.amount)
    }
}

impl<'a, T: FormattableCurrency> Money<'a, T> {
    /// Creates a Money object given an amount string and a currency str.
    ///
    /// Supports fuzzy amount strings like "100", "100.00" and "-100.00"
    pub fn from_str(amount: &str, currency: &'a T) -> Result<Money<'a, T>, MoneyError> {
        let format = LocalFormat::from_locale(currency.locale());
        let amount_parts: Vec<&str> = amount.split(format.exponent_separator).collect();

        let mut split_decimal: Vec<&str> = amount_parts[0].split(format.digit_separator).collect();
        let mut parsed_decimal = split_decimal.concat();

        // Sanity check the decimal seperation
        for &num in format.digit_separator_pattern().iter() {
            if split_decimal.len() <= 1 {
                break;
            }
            let current = split_decimal.pop().unwrap();
            if current.len() != num {
                return Err(MoneyError::InvalidAmount);
            }
        }

        if amount_parts.len() == 1 {
            parsed_decimal += ".";
            for _ in 0..currency.exponent() {
                parsed_decimal += "0";
            }
        } else if amount_parts.len() == 2 {
            i32::from_str(amount_parts[1])?;
            parsed_decimal = parsed_decimal + "." + amount_parts[1];
        } else {
            return Err(MoneyError::InvalidAmount);
        }

        let decimal = Decimal::from_str(&parsed_decimal).map_err(|_| MoneyError::InvalidAmount)?;
        Ok(Money::from_decimal(decimal, currency))
    }

    /// Creates a Money object given an integer and a currency reference.
    ///
    /// The integer represents minor units of the currency (e.g. 1000 -> 10.00 in USD )
    pub fn from_minor(amount: i64, currency: &'a T) -> Money<'a, T> {
        let amount = Decimal::new(amount, currency.exponent());
        Money { amount, currency }
    }

    /// Creates a Money object given an integer and a currency reference.
    ///
    /// The integer represents major units of the currency (e.g. 1000 -> 1,000 in USD )
    pub fn from_major(amount: i64, currency: &'a T) -> Money<'a, T> {
        let amount = Decimal::new(amount, 0);
        Money { amount, currency }
    }

    /// Creates a Money object given a decimal amount and a currency reference.
    pub fn from_decimal(amount: Decimal, currency: &'a T) -> Money<'a, T> {
        Money { amount, currency }
    }

    /// Returns a reference to the Decimal amount.
    pub fn amount(&self) -> &Decimal {
        &self.amount
    }

    /// Returns the Currency type.
    pub fn currency(&self) -> &'a T {
        self.currency
    }

    /// Returns true if amount == 0.
    pub fn is_zero(&self) -> bool {
        self.amount == Decimal::ZERO
    }

    /// Returns true if amount > 0.
    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive() && self.amount != Decimal::ZERO
    }

    /// Returns true if amount < 0.
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative() && self.amount != Decimal::ZERO
    }

    /// Divides money equally into n shares.
    ///
    /// If the division cannot be applied perfectly, it allocates the remainder
    /// to some of the shares.
    pub fn allocate_to(&self, number: u32) -> Result<Vec<Money<'a, T>>, MoneyError> {
        let shares: Vec<u32> = (0..number).map(|_| 1).collect();
        self.allocate(shares)
    }

    /// Divides money into n shares according to the given weights.
    ///
    /// If the division cannot be applied perfectly, it allocates the remainder
    /// to some of the shares.
    pub fn allocate(&self, shares: Vec<u32>) -> Result<Vec<Money<'a, T>>, MoneyError> {
        if shares.is_empty() {
            return Err(MoneyError::InvalidRatio);
        }

        let share_total: u64 = shares.iter().map(|&x| x as u64).sum();

        if share_total == 0 {
            return Err(MoneyError::InvalidRatio);
        }

        // Convert to minor units (e.g., $11.00 -> 1100 cents)
        let minor_per_major = Decimal::from(10u64.pow(self.currency.exponent()));
        let total_minor = (self.amount * minor_per_major).floor();

        // Allocate in minor units
        let share_total_decimal = Decimal::from(share_total);
        let mut allocations_minor: Vec<Decimal> = Vec::with_capacity(shares.len());
        let mut allocated = Decimal::ZERO;

        for &share in &shares {
            let share_value = (total_minor * Decimal::from(share) / share_total_decimal).floor();
            allocations_minor.push(share_value);
            allocated += share_value;
        }

        // Distribute remainder one minor unit at a time
        let mut remainder = total_minor - allocated;
        let mut i: usize = 0;
        while remainder > Decimal::ZERO {
            allocations_minor[i] += Decimal::ONE;
            remainder -= Decimal::ONE;
            i += 1;
        }

        // Convert back to major units
        let major_per_minor = Decimal::new(1, self.currency.exponent());
        Ok(allocations_minor
            .into_iter()
            .map(|minor| Money::from_decimal(minor * major_per_minor, self.currency))
            .collect())
    }

    /// Returns a `Money` rounded to the specified number of minor units using the rounding strategy.
    pub fn round(&self, digits: u32, strategy: Round) -> Money<'a, T> {
        let mut money = *self;

        money.amount = match strategy {
            Round::HalfDown => money
                .amount
                .round_dp_with_strategy(digits, rust_decimal::RoundingStrategy::MidpointTowardZero),
            Round::HalfUp => money.amount.round_dp_with_strategy(
                digits,
                rust_decimal::RoundingStrategy::MidpointAwayFromZero,
            ),
            Round::HalfEven => money.amount.round_dp_with_strategy(
                digits,
                rust_decimal::RoundingStrategy::MidpointNearestEven,
            ),
        };

        money
    }
}

/// Strategies that can be used to round Money.
///
/// For more details, see [rust_decimal::RoundingStrategy]
pub enum Round {
    HalfUp,
    HalfDown,
    HalfEven,
}

impl<'a, T: FormattableCurrency> fmt::Display for Money<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let currency = self.currency;
        let format = LocalFormat::from_locale(currency.locale());

        let mut format_params = Params {
            digit_separator: format.digit_separator,
            exponent_separator: format.exponent_separator,
            separator_pattern: format.digit_separator_pattern(),
            rounding: Some(currency.exponent()),
            symbol: Some(currency.symbol()),
            code: Some(currency.code()),
            ..Default::default()
        };

        if currency.symbol_first() {
            format_params.positions = vec![Position::Sign, Position::Symbol, Position::Amount];
            write!(f, "{}", Formatter::money(self, format_params))
        } else {
            format_params.positions = vec![Position::Sign, Position::Amount, Position::Symbol];
            write!(f, "{}", Formatter::money(self, format_params))
        }
    }
}

// Serde support
#[cfg(feature = "serde")]
mod serde_support {
    use super::*;
    use crate::currency::Findable;
    use rust_decimal::Decimal;
    use serde::de::{self, Deserializer, MapAccess, Visitor};
    use serde::ser::{SerializeStruct, Serializer};
    use serde::{Deserialize, Serialize};
    use std::fmt;
    use std::marker::PhantomData;

    impl<T: FormattableCurrency> Serialize for Money<'_, T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut state = serializer.serialize_struct("Money", 2)?;
            state.serialize_field("amount", &self.amount)?;
            state.serialize_field("currency", self.currency.code())?;
            state.end()
        }
    }

    impl<'de, T: Findable + 'static> Deserialize<'de> for Money<'static, T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            #[derive(Deserialize)]
            #[serde(field_identifier, rename_all = "lowercase")]
            enum Field {
                Amount,
                Currency,
            }

            struct MoneyVisitor<T>(PhantomData<T>);

            impl<'de, T: Findable + 'static> Visitor<'de> for MoneyVisitor<T> {
                type Value = Money<'static, T>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct Money with amount and currency fields")
                }

                fn visit_map<V>(self, mut map: V) -> Result<Money<'static, T>, V::Error>
                where
                    V: MapAccess<'de>,
                {
                    let mut amount: Option<Decimal> = None;
                    let mut currency_code: Option<String> = None;

                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Amount => {
                                if amount.is_some() {
                                    return Err(de::Error::duplicate_field("amount"));
                                }
                                amount = Some(map.next_value()?);
                            }
                            Field::Currency => {
                                if currency_code.is_some() {
                                    return Err(de::Error::duplicate_field("currency"));
                                }
                                currency_code = Some(map.next_value()?);
                            }
                        }
                    }

                    let amount = amount.ok_or_else(|| de::Error::missing_field("amount"))?;
                    let currency_code =
                        currency_code.ok_or_else(|| de::Error::missing_field("currency"))?;

                    let currency = T::find(&currency_code).ok_or_else(|| {
                        de::Error::custom(format!("unknown currency code: {}", currency_code))
                    })?;

                    Ok(Money::from_decimal(amount, currency))
                }
            }

            const FIELDS: &[&str] = &["amount", "currency"];
            deserializer.deserialize_struct("Money", FIELDS, MoneyVisitor(PhantomData))
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
            },
            GBP: {
                code: "GBP",
                exponent: 2,
                locale: EnUs,
                minor_units: 1,
                name: "British Pound",
                symbol: "£",
                symbol_first: true,
            },
            EUR: {
                code: "EUR",
                exponent: 2,
                locale: EnEu,
                minor_units: 1,
                name: "Euro",
                symbol: "€",
                symbol_first: true,
            },
            INR: {
                code: "INR",
                exponent: 2,
                locale: EnIn,
                minor_units: 50,
                name: "Indian Rupee",
                symbol: "₹",
                symbol_first: true,
            },
            BHD: {
                code: "BHD",
                exponent: 3,
                locale: EnUs,
                minor_units: 5,
                name: "Bahraini Dinar",
                symbol: "ب.د",
                symbol_first: true,
            },
            AED: {
                code: "AED",
                exponent: 2,
                locale: EnUs,
                minor_units: 25,
                name: "United Arab Emirates Dirham",
                symbol: "د.إ",
                symbol_first: false,
            },
            JPY: {
                code: "JPY",
                exponent: 0,
                locale: EnUs,
                minor_units: 1,
                name: "Japanese Yen",
                symbol: "¥",
                symbol_first: true,
            }
        }
    );

    mod construction {
        use super::*;

        #[test]
        fn major_minor_equivalence() {
            let _usd = test::find("USD"); // Prevents unused code warnings
            let major_usd = Money::from_major(10, test::USD);
            let minor_usd = Money::from_minor(1000, test::USD);
            assert_eq!(major_usd, minor_usd);
        }

        #[test]
        fn from_minor_vs_from_major_usd() {
            let from_minor = Money::from_minor(10000, test::USD);
            let from_major = Money::from_major(100, test::USD);
            assert_eq!(
                format!("{}", from_minor),
                format!("{}", from_major),
                "from_minor and from_major should format identically"
            );
            assert_eq!("$100.00", format!("{}", from_major));
        }

        #[test]
        fn from_minor_vs_from_major_eur() {
            let from_minor = Money::from_minor(10000, test::EUR);
            let from_major = Money::from_major(100, test::EUR);
            assert_eq!(
                format!("{}", from_minor),
                format!("{}", from_major),
                "from_minor and from_major should format identically"
            );
            assert_eq!("€100,00", format!("{}", from_major));
        }

        #[test]
        fn from_minor_with_zero_exponent() {
            // For JPY (exponent 0), from_minor and from_major should be identical
            let from_minor = Money::from_minor(100, test::JPY);
            let from_major = Money::from_major(100, test::JPY);
            assert_eq!(from_minor, from_major);
            assert_eq!(format!("{}", from_minor), "¥100");
        }

        #[test]
        fn from_str_parses_correctly() {
            let expected_money = Money::from_minor(2999, test::GBP);
            let money = Money::from_str("29.99", test::GBP).unwrap();
            assert_eq!(money, expected_money);
        }

        #[test]
        fn from_str_parses_64_bit_numbers() {
            let expected_money = Money::from_major(i64::MAX, test::GBP);
            let money = Money::from_str(&i64::MAX.to_string(), test::GBP).unwrap();
            assert_eq!(money, expected_money);
        }

        #[test]
        fn from_str_parses_signs() {
            let expected_money = Money::from_minor(-300, test::GBP);
            let money = Money::from_str("-3", test::GBP).unwrap();
            assert_eq!(money, expected_money);

            let expected_money = Money::from_minor(300, test::GBP);
            let money = Money::from_str("+3", test::GBP).unwrap();
            assert_eq!(money, expected_money);
        }

        #[test]
        fn from_str_ignores_separators() {
            let expected_money = Money::from_minor(100000000, test::GBP);
            let money = Money::from_str("1,000,000", test::GBP).unwrap();
            assert_eq!(money, expected_money);
        }

        #[test]
        fn from_str_parse_errors() {
            // Delimiter precedes separators
            assert_eq!(
                Money::from_str("1.0000,000", test::GBP).unwrap_err(),
                MoneyError::InvalidAmount
            );

            // Multiple delimiters
            assert_eq!(
                Money::from_str("1.0000.000", test::GBP).unwrap_err(),
                MoneyError::InvalidAmount
            );

            // Unrecognized character
            assert_eq!(
                Money::from_str("1.0000!000", test::GBP).unwrap_err(),
                MoneyError::InvalidAmount
            );

            // Only separators, no digits
            assert_eq!(
                Money::from_str(",", test::GBP).unwrap_err(),
                MoneyError::InvalidAmount
            );
            assert_eq!(
                Money::from_str(".", test::GBP).unwrap_err(),
                MoneyError::InvalidAmount
            );
            assert_eq!(
                Money::from_str(",,.", test::GBP).unwrap_err(),
                MoneyError::InvalidAmount
            );

            // Whitespace-only string
            assert_eq!(
                Money::from_str("   ", test::USD).unwrap_err(),
                MoneyError::InvalidAmount
            );

            // Leading/trailing spaces
            assert_eq!(
                Money::from_str(" 100 ", test::USD).unwrap_err(),
                MoneyError::InvalidAmount
            );

            // Empty string parses to zero (arguably a bug, but documenting current behavior)
            let empty = Money::from_str("", test::USD).unwrap();
            assert_eq!(empty, Money::from_minor(0, test::USD));

            // Invalid decimal/separator combinations per locale
            assert_eq!(
                Money::from_str("1,00.00", test::GBP).unwrap_err(),
                MoneyError::InvalidAmount
            );
            assert_eq!(
                Money::from_str("1.00,00", test::EUR).unwrap_err(),
                MoneyError::InvalidAmount
            );
            assert_eq!(
                Money::from_str("1.00.000,00", test::EUR).unwrap_err(),
                MoneyError::InvalidAmount
            );
            assert_eq!(
                Money::from_str("1.00.000.000,00", test::EUR).unwrap_err(),
                MoneyError::InvalidAmount
            );
            assert_eq!(
                Money::from_str("1,00.00", test::INR).unwrap_err(),
                MoneyError::InvalidAmount
            );
            assert_eq!(
                Money::from_str("1.000.000.00", test::INR).unwrap_err(),
                MoneyError::InvalidAmount
            );
        }
    }

    mod arithmetic {
        use super::*;

        #[test]
        fn addition_and_subtraction() {
            assert_eq!(
                Money::from_major(2, test::USD),
                Money::from_major(1, test::USD) + Money::from_major(1, test::USD)
            );
            assert_eq!(
                Money::from_major(0, test::USD),
                Money::from_major(1, test::USD) - Money::from_major(1, test::USD)
            );
        }

        #[test]
        #[should_panic]
        fn addition_panics_on_different_currencies() {
            let _no_op = Money::from_minor(100, test::USD) + Money::from_minor(100, test::GBP);
        }

        #[test]
        #[should_panic]
        fn subtraction_panics_on_different_currencies() {
            let _no_op = Money::from_minor(100, test::USD) - Money::from_minor(100, test::GBP);
        }

        #[test]
        #[should_panic]
        fn add_assign_panics_on_different_currencies() {
            let mut money = Money::from_minor(100, test::USD);
            money += Money::from_minor(100, test::GBP);
        }

        #[test]
        #[should_panic]
        fn sub_assign_panics_on_different_currencies() {
            let mut money = Money::from_minor(100, test::USD);
            money -= Money::from_minor(100, test::GBP);
        }

        #[test]
        fn multiplication_and_division() {
            // Multiplication integer
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(100, test::USD) * 2
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-100, test::USD) * -2
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                -2 * Money::from_minor(-100, test::USD)
            );

            // Multiplication decimal
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(100, test::USD) * Decimal::new(2, 0)
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-100, test::USD) * Decimal::new(-2, 0)
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Decimal::new(-2, 0) * Money::from_minor(-100, test::USD)
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(400, test::USD) * Decimal::new(5, 1)
            );

            // Division integer
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(400, test::USD) / 2
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-400, test::USD) / -2
            );
            assert_eq!(
                Money::from_minor(50, test::USD),
                -1 / Money::from_minor(-200, test::USD)
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-200, test::USD) / -1
            );

            // Division decimal
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(400, test::USD) / Decimal::new(2, 0)
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-400, test::USD) / Decimal::new(-2, 0)
            );
            assert_eq!(
                Money::from_minor(50, test::USD),
                Decimal::new(-1, 0) / Money::from_minor(-200, test::USD)
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-200, test::USD) / Decimal::new(-1, 0)
            );
            assert_eq!(
                Money::from_minor(400, test::USD),
                Money::from_minor(-200, test::USD) / Decimal::new(-5, 1)
            );

            // MulAssign integer
            let mut money = Money::from_minor(100, test::USD);
            money *= 2;
            assert_eq!(Money::from_minor(200, test::USD), money);

            // MulAssign decimal
            let mut money = Money::from_minor(100, test::USD);
            money *= Decimal::new(2, 0);
            assert_eq!(Money::from_minor(200, test::USD), money);

            // DivAssign integer
            let mut money = Money::from_minor(100, test::USD);
            money /= -2;
            assert_eq!(Money::from_minor(-50, test::USD), money);

            // DivAssign decimal
            let mut money = Money::from_minor(100, test::USD);
            money /= Decimal::new(-2, 0);
            assert_eq!(Money::from_minor(-50, test::USD), money);
        }

        #[test]
        fn negation() {
            let money = Money::from_minor(100, test::USD);
            assert_eq!(-money, Money::from_minor(-100, test::USD));
        }

        #[test]
        #[should_panic]
        fn division_by_zero_panics() {
            let money = Money::from_minor(100, test::USD);
            let _result = money / 0;
        }

        #[test]
        fn copy_semantics() {
            let money = Money::from_major(1, test::USD);
            let _1st_derived_money = money * 3;
            // if Money didn't impl Copy, this would fail to compile
            let _2nd_derived_money = money * 3;
        }
    }

    mod comparison {
        use super::*;

        #[test]
        fn ordering_and_equality() {
            // Greater Than
            assert!(Money::from_minor(200, test::USD) > Money::from_minor(100, test::USD));
            // Less Than
            assert!(Money::from_minor(100, test::USD) < Money::from_minor(200, test::USD));
            // Equals
            assert!(Money::from_minor(100, test::USD) == Money::from_minor(100, test::USD));
            assert!(Money::from_minor(100, test::USD) != Money::from_minor(100, test::GBP));
        }

        #[test]
        fn sign_predicates() {
            // is positive
            assert!(Money::from_minor(100, test::USD).is_positive());
            assert!(!Money::from_minor(0, test::USD).is_positive());
            assert!(!Money::from_minor(-100, test::USD).is_positive());
            // is zero
            assert!(Money::from_minor(0, test::USD).is_zero());
            assert!(!Money::from_minor(100, test::USD).is_zero());
            assert!(!Money::from_minor(-100, test::USD).is_zero());
            // is negative
            assert!(Money::from_minor(-100, test::USD).is_negative());
            assert!(!Money::from_minor(100, test::USD).is_negative());
            assert!(!Money::from_minor(0, test::USD).is_negative());
        }

        #[test]
        #[should_panic]
        fn greater_than_panics_on_different_currencies() {
            assert!(Money::from_minor(100, test::USD) > Money::from_minor(100, test::GBP));
        }

        #[test]
        #[should_panic]
        fn less_than_panics_on_different_currencies() {
            assert!(Money::from_minor(100, test::USD) < Money::from_minor(100, test::GBP));
        }

        #[test]
        fn is_zero_with_negative_zero() {
            // Decimal can represent -0, ensure is_zero handles it
            let neg_zero = Money::from_decimal(-Decimal::ZERO, test::USD);
            assert!(neg_zero.is_zero());
            assert!(!neg_zero.is_negative());
            assert!(!neg_zero.is_positive());
        }
    }

    mod allocation {
        use super::*;

        #[test]
        fn allocate_shares() {
            // $100 split 70/20/10
            let money = Money::from_minor(10000, test::USD);
            let allocated = money.allocate(vec![70, 20, 10]).unwrap();

            assert_eq!(allocated[0], Money::from_minor(7000, test::USD));
            assert_eq!(allocated[1], Money::from_minor(2000, test::USD));
            assert_eq!(allocated[2], Money::from_minor(1000, test::USD));
        }

        #[test]
        fn allocate_indivisible_remainder() {
            // 11.00 USD split into thirds: 3.67 + 3.67 + 3.66 = 11.00
            let money = Money::from_minor(1_100, test::USD);
            let expected = vec![
                Money::from_minor(367, test::USD),
                Money::from_minor(367, test::USD),
                Money::from_minor(366, test::USD),
            ];

            // Using allocate with equal shares
            assert_eq!(expected, money.allocate(vec![1, 1, 1]).unwrap());

            // Using allocate_to (convenience wrapper)
            assert_eq!(expected, money.allocate_to(3).unwrap());
        }

        #[test]
        fn allocate_with_zero_shares() {
            // Zero shares are allowed if at least one is non-zero
            let money = Money::from_minor(1_000, test::USD);
            let allocated = money.allocate(vec![1, 0, 0]).unwrap();
            assert_eq!(allocated[0], Money::from_minor(1_000, test::USD));
            assert_eq!(allocated[1], Money::from_minor(0, test::USD));
            assert_eq!(allocated[2], Money::from_minor(0, test::USD));
        }

        #[test]
        fn allocate_negative_amount() {
            let money = Money::from_minor(-1100, test::USD);
            let allocated = money.allocate(vec![1, 1, 1]).unwrap();

            // Sum should equal original
            let sum: Decimal = allocated.iter().map(|m| *m.amount()).sum();
            assert_eq!(sum, *money.amount());

            // All allocations should be negative or zero
            for m in &allocated {
                assert!(m.amount() <= &Decimal::ZERO);
            }
        }

        #[test]
        fn allocate_zero_amount() {
            let money = Money::from_minor(0, test::USD);
            let allocated = money.allocate(vec![1, 1, 1]).unwrap();

            for m in &allocated {
                assert!(m.is_zero());
            }
        }

        #[test]
        fn allocate_single_share() {
            let money = Money::from_minor(1000, test::USD);
            let allocated = money.allocate(vec![1]).unwrap();

            assert_eq!(allocated.len(), 1);
            assert_eq!(allocated[0], money);
        }

        #[test]
        fn allocate_with_zero_exponent_currency() {
            // JPY has exponent 0, so minor == major
            let money = Money::from_major(1000, test::JPY);
            let allocated = money.allocate(vec![1, 1, 1]).unwrap();

            let sum: Decimal = allocated.iter().map(|m| *m.amount()).sum();
            assert_eq!(sum, *money.amount());

            // 1000 / 3 = 334, 333, 333
            assert_eq!(allocated[0], Money::from_major(334, test::JPY));
            assert_eq!(allocated[1], Money::from_major(333, test::JPY));
            assert_eq!(allocated[2], Money::from_major(333, test::JPY));
        }

        #[test]
        fn allocate_more_shares_than_minor_units() {
            let money = Money::from_minor(100, test::USD); // $1.00
            let shares: Vec<u32> = vec![1; 101]; // 101 equal shares
            let allocated = money.allocate(shares).unwrap();

            assert_eq!(allocated.len(), 101);

            // First 100 shares get $0.01 each (remainder distributed first-to-last)
            for m in &allocated[..100] {
                assert_eq!(*m.amount(), Decimal::new(1, 2));
            }

            // Last share gets $0.00 (not enough cents to go around)
            assert!(allocated[100].is_zero());
        }

        #[test]
        fn allocate_errors() {
            // Error if the shares vector is empty
            let monies = Money::from_minor(100, test::USD).allocate(Vec::new());
            assert_eq!(monies.unwrap_err(), MoneyError::InvalidRatio);

            // Error if all shares are zero (would cause division by zero)
            let monies = Money::from_minor(100, test::USD).allocate(vec![0, 0, 0]);
            assert_eq!(monies.unwrap_err(), MoneyError::InvalidRatio);

            // Error if allocate_to is called with zero
            let monies = Money::from_minor(100, test::USD).allocate_to(0);
            assert_eq!(monies.unwrap_err(), MoneyError::InvalidRatio);
        }
    }

    mod rounding {
        use super::*;

        #[test]
        fn precision_and_rounding() {
            // Dividing 20 by 3 rounds to 6.67 in USD and 6.667 in BHD
            let expected_money = Money::from_minor(667, test::USD);
            let mut money = Money::from_minor(2_000, test::USD);
            money /= 3;
            assert_eq!(money.round(2, Round::HalfEven), expected_money);

            let expected_money = Money::from_minor(6_667, test::BHD);
            let mut money = Money::from_minor(20_000, test::BHD);
            money /= 3;
            assert_eq!(money.round(3, Round::HalfEven), expected_money);
        }

        #[test]
        fn half_up_at_boundary() {
            // 2.5 should round to 3 with HalfUp
            let money = Money::from_str("2.50", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfUp);
            assert_eq!(*rounded.amount(), Decimal::from(3));

            // 2.4 should round to 2
            let money = Money::from_str("2.40", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfUp);
            assert_eq!(*rounded.amount(), Decimal::from(2));
        }

        #[test]
        fn half_down_at_boundary() {
            // 2.5 should round to 2 with HalfDown
            let money = Money::from_str("2.50", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfDown);
            assert_eq!(*rounded.amount(), Decimal::from(2));

            // 2.6 should round to 3
            let money = Money::from_str("2.60", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfDown);
            assert_eq!(*rounded.amount(), Decimal::from(3));
        }

        #[test]
        fn half_even_rounds_to_even() {
            // 2.5 should round to 2 (nearest even)
            let money = Money::from_str("2.50", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfEven);
            assert_eq!(*rounded.amount(), Decimal::from(2));

            // 3.5 should round to 4 (nearest even)
            let money = Money::from_str("3.50", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfEven);
            assert_eq!(*rounded.amount(), Decimal::from(4));
        }

        #[test]
        fn negative_amounts() {
            // -2.5 with HalfUp should round away from zero to -3
            let money = Money::from_str("-2.50", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfUp);
            assert_eq!(*rounded.amount(), Decimal::from(-3));

            // -2.5 with HalfDown should round toward zero to -2
            let money = Money::from_str("-2.50", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfDown);
            assert_eq!(*rounded.amount(), Decimal::from(-2));

            // -2.5 with HalfEven should round to -2 (nearest even)
            let money = Money::from_str("-2.50", test::USD).unwrap();
            let rounded = money.round(0, Round::HalfEven);
            assert_eq!(*rounded.amount(), Decimal::from(-2));
        }
    }

    mod formatting {
        use super::*;

        #[test]
        fn separates_digits() {
            let usd = Money::from_minor(0, test::USD);
            assert_eq!(format!("{}", usd), "$0.00");

            let usd = Money::from_minor(10_000_000, test::USD);
            assert_eq!(format!("{}", usd), "$100,000.00");

            let usd = Money::from_minor(-10_000_000, test::USD);
            assert_eq!(format!("{}", usd), "-$100,000.00");

            let usd = Money::from_minor(100_000_000_000, test::USD);
            assert_eq!(format!("{}", usd), "$1,000,000,000.00");

            let inr = Money::from_minor(10_000_000, test::INR);
            assert_eq!(format!("{}", inr), "₹1,00,000.00");

            let inr = Money::from_minor(-1_000_000_000, test::INR);
            assert_eq!(format!("{}", inr), "-₹1,00,00,000.00");
        }

        #[test]
        fn places_symbols_correctly() {
            let money = Money::from_minor(0, test::USD);
            assert_eq!(format!("{}", money), "$0.00");

            let money = Money::from_minor(0, test::AED);
            assert_eq!(format!("{}", money), "0.00د.إ");
        }

        #[test]
        fn uses_correct_separators() {
            let money = Money::from_minor(100_000, test::EUR);
            assert_eq!(format!("{}", money), "€1.000,00");
        }

        #[test]
        fn rounds_exponent() {
            // 19.999 rounds to 20 for USD
            let money = Money::from_str("19.9999", test::USD).unwrap();
            assert_eq!("$20.00", format!("{}", money));

            // 29.111 rounds to 29.11 for USD
            let money = Money::from_str("29.111", test::USD).unwrap();
            assert_eq!("$29.11", format!("{}", money));

            // 39.1155 rounds to 39.116 for BHD
            let money = Money::from_str("39.1155", test::BHD).unwrap();
            assert_eq!("ب.د39.116", format!("{}", money));
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use crate::Money;
    use crate::define_currency_set;

    define_currency_set!(
        test {
            USD: {
                code: "USD",
                exponent: 2,
                locale: EnUs,
                minor_units: 100,
                name: "US Dollar",
                symbol: "$",
                symbol_first: true,
            },
            EUR: {
                code: "EUR",
                exponent: 2,
                locale: EnEu,
                minor_units: 1,
                name: "Euro",
                symbol: "€",
                symbol_first: true,
            }
        }
    );

    #[test]
    fn serialize_money_to_json() {
        let money = Money::from_minor(12345, test::USD);
        let json = serde_json::to_string(&money).unwrap();
        assert_eq!(json, r#"{"amount":"123.45","currency":"USD"}"#);
    }

    #[test]
    fn serialize_negative_money() {
        let money = Money::from_minor(-9999, test::EUR);
        let json = serde_json::to_string(&money).unwrap();
        assert_eq!(json, r#"{"amount":"-99.99","currency":"EUR"}"#);
    }

    #[test]
    fn deserialize_money_from_json() {
        let json = r#"{"amount":"123.45","currency":"USD"}"#;
        let money: Money<test::Currency> = serde_json::from_str(json).unwrap();
        assert_eq!(money, Money::from_minor(12345, test::USD));
    }

    #[test]
    fn deserialize_money_reversed_field_order() {
        let json = r#"{"currency":"EUR","amount":"50.00"}"#;
        let money: Money<test::Currency> = serde_json::from_str(json).unwrap();
        assert_eq!(money, Money::from_minor(5000, test::EUR));
    }

    #[test]
    fn deserialize_negative_money() {
        let json = r#"{"amount":"-99.99","currency":"EUR"}"#;
        let money: Money<test::Currency> = serde_json::from_str(json).unwrap();
        assert_eq!(money, Money::from_minor(-9999, test::EUR));
    }

    #[test]
    fn roundtrip_serialization() {
        let original = Money::from_minor(123456789, test::USD);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Money<test::Currency> = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn deserialize_unknown_currency_fails() {
        let json = r#"{"amount":"100.00","currency":"XYZ"}"#;
        let result: Result<Money<test::Currency>, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown currency"));
    }

    #[test]
    fn deserialize_missing_amount_fails() {
        let json = r#"{"currency":"USD"}"#;
        let result: Result<Money<test::Currency>, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("amount"));
    }

    #[test]
    fn deserialize_missing_currency_fails() {
        let json = r#"{"amount":"100.00"}"#;
        let result: Result<Money<test::Currency>, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    #[test]
    fn serialize_preserves_precision() {
        // Test with many decimal places
        let money = Money::from_decimal(
            rust_decimal::Decimal::new(123456789012345678, 18),
            test::USD,
        );
        let json = serde_json::to_string(&money).unwrap();
        let deserialized: Money<test::Currency> = serde_json::from_str(&json).unwrap();
        assert_eq!(money.amount(), deserialized.amount());
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use crate::define_currency_set;
    use proptest::prelude::*;
    use rust_decimal::Decimal;

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
            },
            JPY: {
                code: "JPY",
                exponent: 0,
                locale: EnUs,
                minor_units: 1,
                name: "Japanese Yen",
                symbol: "¥",
                symbol_first: true,
            },
            BHD: {
                code: "BHD",
                exponent: 3,
                locale: EnUs,
                minor_units: 5,
                name: "Bahraini Dinar",
                symbol: "ب.د",
                symbol_first: true,
            }
        }
    );

    // Strategy for generating Money amounts in minor units
    // Use i32 range to avoid overflow in arithmetic tests
    fn minor_amount() -> impl Strategy<Value = i64> {
        -1_000_000_000i64..1_000_000_000i64
    }

    // Strategy for non-zero multipliers
    fn non_zero_multiplier() -> impl Strategy<Value = i64> {
        prop_oneof![-1_000_000i64..-1i64, 1i64..1_000_000i64,]
    }

    // Strategy for allocation shares (non-empty, at least one non-zero)
    fn valid_shares() -> impl Strategy<Value = Vec<u32>> {
        prop::collection::vec(0u32..100u32, 1..10)
            .prop_filter("at least one non-zero share", |shares| {
                shares.iter().any(|&s| s > 0)
            })
    }

    mod arithmetic_properties {
        use super::*;

        proptest! {
            #[test]
            fn addition_is_commutative(a in minor_amount(), b in minor_amount()) {
                let money_a = Money::from_minor(a, test::USD);
                let money_b = Money::from_minor(b, test::USD);
                prop_assert_eq!(money_a + money_b, money_b + money_a);
            }

            #[test]
            fn addition_is_associative(a in minor_amount(), b in minor_amount(), c in minor_amount()) {
                let money_a = Money::from_minor(a, test::USD);
                let money_b = Money::from_minor(b, test::USD);
                let money_c = Money::from_minor(c, test::USD);
                prop_assert_eq!((money_a + money_b) + money_c, money_a + (money_b + money_c));
            }

            #[test]
            fn zero_is_additive_identity(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                let zero = Money::from_minor(0, test::USD);
                prop_assert_eq!(money + zero, money);
                prop_assert_eq!(zero + money, money);
            }

            #[test]
            fn subtraction_is_inverse_of_addition(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                let zero = Money::from_minor(0, test::USD);
                prop_assert_eq!(money - money, zero);
            }

            #[test]
            fn negation_is_self_inverse(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                prop_assert_eq!(-(-money), money);
            }

            #[test]
            fn multiplication_by_one_is_identity(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                prop_assert_eq!(money * 1i64, money);
                prop_assert_eq!(1i64 * money, money);
            }

            #[test]
            #[allow(clippy::erasing_op)]
            fn multiplication_by_zero_gives_zero(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                let zero = Money::from_minor(0, test::USD);
                prop_assert_eq!(money * 0i64, zero);
            }

            #[test]
            fn division_by_one_is_identity(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                prop_assert_eq!(money / 1i64, money);
            }

            #[test]
            fn multiplication_then_division_roundtrip(a in minor_amount(), n in non_zero_multiplier()) {
                let money = Money::from_minor(a, test::USD);
                let multiplied = money * n;
                let divided = multiplied / n;
                // May not be exactly equal due to decimal precision, but should be very close
                let diff = (*money.amount() - *divided.amount()).abs();
                prop_assert!(diff < Decimal::new(1, 10), "difference too large: {}", diff);
            }

            #[test]
            fn distributive_property(a in minor_amount(), n in -1000i64..1000, m in -1000i64..1000) {
                let money = Money::from_minor(a, test::USD);
                let left = money * (n + m);
                let right = money * n + money * m;
                prop_assert_eq!(left, right);
            }
        }
    }

    mod allocation_properties {
        use super::*;

        proptest! {
            #[test]
            fn allocation_sum_equals_original(amount in minor_amount(), shares in valid_shares()) {
                let money = Money::from_minor(amount, test::USD);
                let allocated = money.allocate(shares).unwrap();

                let sum: Decimal = allocated.iter().map(|m| *m.amount()).sum();
                prop_assert_eq!(sum, *money.amount(), "sum of allocations must equal original");
            }

            #[test]
            fn allocation_count_equals_shares_count(amount in minor_amount(), shares in valid_shares()) {
                let money = Money::from_minor(amount, test::USD);
                let allocated = money.allocate(shares.clone()).unwrap();

                prop_assert_eq!(allocated.len(), shares.len());
            }

            #[test]
            fn allocation_preserves_sign_for_positive(amount in 1i64..1_000_000_000, shares in valid_shares()) {
                // Positive amounts should never allocate to negative
                let money = Money::from_minor(amount, test::USD);
                let allocated = money.allocate(shares).unwrap();

                for m in allocated {
                    prop_assert!(!m.is_negative(), "positive money should not allocate to negative");
                }
            }

            #[test]
            fn zero_allocates_to_zeros(shares in valid_shares()) {
                let money = Money::from_minor(0, test::USD);
                let allocated = money.allocate(shares).unwrap();

                for m in allocated {
                    prop_assert!(m.is_zero(), "zero money should allocate to zeros");
                }
            }

            #[test]
            fn allocate_to_is_consistent_with_allocate(amount in minor_amount(), n in 1u32..20) {
                let money = Money::from_minor(amount, test::USD);
                let shares: Vec<u32> = vec![1; n as usize];

                let via_allocate = money.allocate(shares).unwrap();
                let via_allocate_to = money.allocate_to(n).unwrap();

                prop_assert_eq!(via_allocate, via_allocate_to);
            }

            #[test]
            fn allocation_is_deterministic(amount in minor_amount(), shares in valid_shares()) {
                let money = Money::from_minor(amount, test::USD);
                let first = money.allocate(shares.clone()).unwrap();
                let second = money.allocate(shares).unwrap();

                prop_assert_eq!(first, second);
            }

            #[test]
            fn allocation_works_for_zero_exponent_currency(amount in -1_000_000i64..1_000_000, shares in valid_shares()) {
                let money = Money::from_major(amount, test::JPY);
                let allocated = money.allocate(shares).unwrap();

                let sum: Decimal = allocated.iter().map(|m| *m.amount()).sum();
                prop_assert_eq!(sum, *money.amount());
            }

            #[test]
            fn allocation_works_for_high_exponent_currency(amount in minor_amount(), shares in valid_shares()) {
                let money = Money::from_minor(amount, test::BHD);
                let allocated = money.allocate(shares).unwrap();

                let sum: Decimal = allocated.iter().map(|m| *m.amount()).sum();
                prop_assert_eq!(sum, *money.amount());
            }
        }
    }

    mod rounding_properties {
        use super::*;

        proptest! {
            #[test]
            fn rounding_is_idempotent(amount in minor_amount(), digits in 0u32..5) {
                let money = Money::from_minor(amount, test::USD);
                let once = money.round(digits, Round::HalfEven);
                let twice = once.round(digits, Round::HalfEven);
                prop_assert_eq!(once, twice);
            }

            #[test]
            fn rounding_preserves_sign_except_zero(amount in minor_amount(), digits in 0u32..5) {
                let money = Money::from_minor(amount, test::USD);
                let rounded = money.round(digits, Round::HalfEven);

                if money.is_positive() && !rounded.is_zero() {
                    prop_assert!(rounded.is_positive() || rounded.is_zero());
                } else if money.is_negative() && !rounded.is_zero() {
                    prop_assert!(rounded.is_negative() || rounded.is_zero());
                }
            }

            #[test]
            fn half_up_rounds_away_from_zero_at_midpoint(amount in 1i64..1000) {
                // Test positive midpoint: X.5 -> X+1
                let money = Money::from_decimal(
                    Decimal::new(amount * 10 + 5, 1), // e.g., 1.5, 2.5, 3.5
                    test::USD
                );
                let rounded = money.round(0, Round::HalfUp);
                prop_assert_eq!(*rounded.amount(), Decimal::from(amount + 1));
            }

            #[test]
            fn half_down_rounds_toward_zero_at_midpoint(amount in 1i64..1000) {
                // Test positive midpoint: X.5 -> X
                let money = Money::from_decimal(
                    Decimal::new(amount * 10 + 5, 1), // e.g., 1.5, 2.5, 3.5
                    test::USD
                );
                let rounded = money.round(0, Round::HalfDown);
                prop_assert_eq!(*rounded.amount(), Decimal::from(amount));
            }
        }
    }

    mod from_minor_major_properties {
        use super::*;

        proptest! {
            #[test]
            fn from_minor_to_major_roundtrip(amount in -1_000_000i64..1_000_000) {
                // For USD (exponent 2): from_minor(100) == from_major(1)
                let from_minor = Money::from_minor(amount * 100, test::USD);
                let from_major = Money::from_major(amount, test::USD);
                prop_assert_eq!(from_minor, from_major);
            }

            #[test]
            fn from_minor_from_major_equivalence_jpy(amount in -1_000_000i64..1_000_000) {
                // For JPY (exponent 0): from_minor == from_major
                let from_minor = Money::from_minor(amount, test::JPY);
                let from_major = Money::from_major(amount, test::JPY);
                prop_assert_eq!(from_minor, from_major);
            }

            #[test]
            fn currency_is_preserved(amount in minor_amount()) {
                let money = Money::from_minor(amount, test::USD);
                prop_assert_eq!(money.currency(), test::USD);
            }
        }
    }
}
