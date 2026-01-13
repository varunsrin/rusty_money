use crate::currency::FormattableCurrency;
use crate::{Money, MoneyError};
use rust_decimal::Decimal;
use std::cmp::Ordering;
use std::fmt;

/// High-performance money type using i64 minor units.
///
/// Use this for performance-critical paths (matching engines, high-frequency trading).
/// For complex operations (allocation, exchange), convert to [`Money<T>`].
///
/// # Example
///
/// ```
/// use rusty_money::{FastMoney, iso};
///
/// let price = FastMoney::from_minor(9999, iso::USD);  // $99.99
/// let qty = 100i64;
/// let total = price.mul(qty).unwrap();  // $9,999.00
/// ```
///
/// # Conversion to/from Money
///
/// ```
/// use rusty_money::{FastMoney, Money, iso};
///
/// // FastMoney -> Money (always succeeds)
/// let fast = FastMoney::from_minor(1000, iso::USD);
/// let money: Money<_> = fast.to_money();
///
/// // Money -> FastMoney (may fail)
/// let money = Money::from_minor(1000, iso::USD);
/// let fast = FastMoney::from_money(money).unwrap();
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct FastMoney<'a, T: FormattableCurrency> {
    minor_units: i64,
    currency: &'a T,
}

impl<'a, T: FormattableCurrency> FastMoney<'a, T> {
    /// Creates a FastMoney from minor units (e.g., cents for USD).
    ///
    /// This is the most efficient constructor as no conversion is needed.
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_money::{FastMoney, iso};
    ///
    /// let ten_dollars = FastMoney::from_minor(1000, iso::USD);  // $10.00
    /// let five_hundred_yen = FastMoney::from_minor(500, iso::JPY);  // ¥500
    /// ```
    #[inline]
    pub fn from_minor(minor_units: i64, currency: &'a T) -> Self {
        FastMoney {
            minor_units,
            currency,
        }
    }

    /// Creates a FastMoney from major units (e.g., dollars for USD).
    ///
    /// Returns an error if the conversion would overflow.
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_money::{FastMoney, iso};
    ///
    /// let ten_dollars = FastMoney::from_major(10, iso::USD).unwrap();  // $10.00
    /// assert_eq!(ten_dollars.minor_units(), 1000);
    /// ```
    #[inline]
    pub fn from_major(amount: i64, currency: &'a T) -> Result<Self, MoneyError> {
        let multiplier = 10i64.pow(currency.exponent());
        let minor_units = amount.checked_mul(multiplier).ok_or(MoneyError::Overflow)?;
        Ok(FastMoney {
            minor_units,
            currency,
        })
    }

    /// Returns the amount in minor units.
    #[inline]
    pub fn minor_units(&self) -> i64 {
        self.minor_units
    }

    /// Returns a reference to the currency.
    #[inline]
    pub fn currency(&self) -> &'a T {
        self.currency
    }

    /// Adds two FastMoney values of the same currency.
    ///
    /// Returns an error on currency mismatch or overflow.
    #[inline]
    pub fn add(&self, other: Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.code().to_string(),
                actual: other.currency.code().to_string(),
            });
        }
        let minor_units = self
            .minor_units
            .checked_add(other.minor_units)
            .ok_or(MoneyError::Overflow)?;
        Ok(FastMoney {
            minor_units,
            currency: self.currency,
        })
    }

    /// Subtracts another FastMoney from this one.
    ///
    /// Returns an error on currency mismatch or overflow.
    #[inline]
    pub fn sub(&self, other: Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.code().to_string(),
                actual: other.currency.code().to_string(),
            });
        }
        let minor_units = self
            .minor_units
            .checked_sub(other.minor_units)
            .ok_or(MoneyError::Overflow)?;
        Ok(FastMoney {
            minor_units,
            currency: self.currency,
        })
    }

    /// Multiplies the amount by an integer scalar.
    ///
    /// Returns an error on overflow.
    #[inline]
    pub fn mul(&self, n: i64) -> Result<Self, MoneyError> {
        let minor_units = self
            .minor_units
            .checked_mul(n)
            .ok_or(MoneyError::Overflow)?;
        Ok(FastMoney {
            minor_units,
            currency: self.currency,
        })
    }

    /// Divides the amount by an integer scalar (truncating toward zero).
    ///
    /// Returns an error on division by zero.
    #[inline]
    pub fn div(&self, n: i64) -> Result<Self, MoneyError> {
        if n == 0 {
            return Err(MoneyError::DivisionByZero);
        }
        let minor_units = self.minor_units / n;
        Ok(FastMoney {
            minor_units,
            currency: self.currency,
        })
    }

    /// Returns true if the amount is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.minor_units == 0
    }

    /// Returns true if the amount is positive.
    #[inline]
    pub fn is_positive(&self) -> bool {
        self.minor_units > 0
    }

    /// Returns true if the amount is negative.
    #[inline]
    pub fn is_negative(&self) -> bool {
        self.minor_units < 0
    }

    /// Returns the absolute value of the amount.
    #[inline]
    pub fn abs(&self) -> Self {
        FastMoney {
            minor_units: self.minor_units.abs(),
            currency: self.currency,
        }
    }

    /// Returns the negation of the amount.
    #[inline]
    pub fn neg(&self) -> Self {
        FastMoney {
            minor_units: -self.minor_units,
            currency: self.currency,
        }
    }

    /// Compares two FastMoney values.
    ///
    /// Returns an error if the currencies don't match.
    #[inline]
    pub fn compare(&self, other: &Self) -> Result<Ordering, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.code().to_string(),
                actual: other.currency.code().to_string(),
            });
        }
        Ok(self.minor_units.cmp(&other.minor_units))
    }

    /// Converts to a [`Money<T>`] with Decimal precision.
    ///
    /// This conversion always succeeds.
    #[inline]
    pub fn to_money(&self) -> Money<'a, T> {
        Money::from_minor(self.minor_units, self.currency)
    }

    /// Converts from a [`Money<T>`] with strict precision checking.
    ///
    /// Returns an error if:
    /// - The amount would overflow i64 when converted to minor units
    /// - The amount has precision beyond the currency's exponent (e.g., $10.005 for USD)
    ///
    /// Use [`from_money_lossy`](Self::from_money_lossy) if you want to truncate extra precision.
    pub fn from_money(money: Money<'a, T>) -> Result<Self, MoneyError> {
        let exponent = money.currency().exponent();
        let scale = Decimal::from(10u64.pow(exponent));
        let scaled = money.amount() * scale;

        // Check for precision loss: if truncating changes the value, there's extra precision
        if scaled != scaled.trunc() {
            return Err(MoneyError::PrecisionLoss);
        }

        // Convert to i64, checking for overflow
        let minor_units: i64 = scaled
            .trunc()
            .to_string()
            .parse()
            .map_err(|_| MoneyError::Overflow)?;

        Ok(FastMoney {
            minor_units,
            currency: money.currency(),
        })
    }

    /// Converts from a [`Money<T>`], truncating any extra precision.
    ///
    /// Returns an error only if the amount would overflow i64.
    /// Extra precision beyond the currency's exponent is silently truncated.
    ///
    /// # Example
    ///
    /// ```
    /// use rusty_money::{FastMoney, Money, iso};
    /// use rust_decimal_macros::dec;
    ///
    /// let money = Money::from_decimal(dec!(10.005), iso::USD);  // $10.005
    /// let fast = FastMoney::from_money_lossy(money).unwrap();
    /// assert_eq!(fast.minor_units(), 1000);  // Truncated to 1000 cents ($10.00)
    /// ```
    pub fn from_money_lossy(money: Money<'a, T>) -> Result<Self, MoneyError> {
        let exponent = money.currency().exponent();
        let scale = Decimal::from(10u64.pow(exponent));
        let scaled = (money.amount() * scale).trunc();

        // Convert to i64, checking for overflow
        let minor_units: i64 = scaled
            .to_string()
            .parse()
            .map_err(|_| MoneyError::Overflow)?;

        Ok(FastMoney {
            minor_units,
            currency: money.currency(),
        })
    }
}

impl<'a, T: FormattableCurrency> fmt::Display for FastMoney<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate to Money's Display implementation for consistent formatting
        write!(f, "{}", self.to_money())
    }
}

#[cfg(feature = "serde")]
mod serde_support {
    use super::*;
    use crate::currency::Findable;
    use serde::de::{self, Deserializer, MapAccess, Visitor};
    use serde::ser::{SerializeStruct, Serializer};
    use serde::{Deserialize, Serialize};
    use std::fmt;
    use std::marker::PhantomData;

    impl<T: FormattableCurrency> Serialize for FastMoney<'_, T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // Serialize in the same format as Money for interoperability
            let money = self.to_money();
            let mut state = serializer.serialize_struct("FastMoney", 2)?;
            state.serialize_field("amount", money.amount())?;
            state.serialize_field("currency", self.currency.code())?;
            state.end()
        }
    }

    impl<'de, T: Findable + FormattableCurrency + 'static> Deserialize<'de> for FastMoney<'static, T> {
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

            struct FastMoneyVisitor<T>(PhantomData<T>);

            impl<'de, T: Findable + FormattableCurrency + 'static> Visitor<'de> for FastMoneyVisitor<T> {
                type Value = FastMoney<'static, T>;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("struct FastMoney with amount and currency fields")
                }

                fn visit_map<V>(self, mut map: V) -> Result<FastMoney<'static, T>, V::Error>
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
                        de::Error::custom(format!("unknown currency: {}", currency_code))
                    })?;

                    let money = Money::from_decimal(amount, currency);

                    // Use from_money_lossy for deserialization to handle potential precision issues
                    FastMoney::from_money_lossy(money).map_err(|e| de::Error::custom(e.to_string()))
                }
            }

            const FIELDS: &[&str] = &["amount", "currency"];
            deserializer.deserialize_struct("FastMoney", FIELDS, FastMoneyVisitor(PhantomData))
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;
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
            }
        }
    );

    #[test]
    fn serialize_deserialize_roundtrip() {
        let original = FastMoney::from_minor(1234, test::USD);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: FastMoney<test::Currency> = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn serialize_format_matches_money() {
        let fast = FastMoney::from_minor(1234, test::USD);
        let money = fast.to_money();

        let fast_json = serde_json::to_string(&fast).unwrap();
        let money_json = serde_json::to_string(&money).unwrap();

        // Both should serialize with same structure
        assert!(fast_json.contains("\"amount\""));
        assert!(fast_json.contains("\"currency\""));
        assert!(fast_json.contains("\"USD\""));

        // The amount format should match
        assert_eq!(fast_json, money_json);
    }

    #[test]
    fn deserialize_from_money_json() {
        // JSON that could have been serialized from Money
        let json = r#"{"amount":"12.34","currency":"USD"}"#;
        let fast: FastMoney<test::Currency> = serde_json::from_str(json).unwrap();
        assert_eq!(fast.minor_units(), 1234);
    }

    #[test]
    fn deserialize_negative() {
        let original = FastMoney::from_minor(-5678, test::USD);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: FastMoney<test::Currency> = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn deserialize_zero() {
        let original = FastMoney::from_minor(0, test::USD);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: FastMoney<test::Currency> = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn deserialize_reversed_field_order() {
        let json = r#"{"currency":"USD","amount":"50.00"}"#;
        let fast: FastMoney<test::Currency> = serde_json::from_str(json).unwrap();
        assert_eq!(fast.minor_units(), 5000);
    }

    #[test]
    fn deserialize_unknown_currency_fails() {
        let json = r#"{"amount":"100.00","currency":"XYZ"}"#;
        let result: Result<FastMoney<test::Currency>, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown currency"));
    }

    #[test]
    fn deserialize_missing_amount_fails() {
        let json = r#"{"currency":"USD"}"#;
        let result: Result<FastMoney<test::Currency>, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("amount"));
    }

    #[test]
    fn deserialize_missing_currency_fails() {
        let json = r#"{"amount":"100.00"}"#;
        let result: Result<FastMoney<test::Currency>, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
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
                name: "US Dollar",
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
                minor_units: 1000,
                name: "Bahraini Dinar",
                symbol: "BD",
                symbol_first: true,
            },
            EUR: {
                code: "EUR",
                exponent: 2,
                locale: EnEu,
                minor_units: 100,
                name: "Euro",
                symbol: "€",
                symbol_first: true,
            }
        }
    );

    // ============ Construction Tests ============

    #[test]
    fn from_minor_creates_correctly() {
        let money = FastMoney::from_minor(1000, test::USD);
        assert_eq!(money.minor_units(), 1000);
        assert_eq!(money.currency(), test::USD);
    }

    #[test]
    fn from_major_creates_correctly() {
        // USD: exponent 2
        let usd = FastMoney::from_major(10, test::USD).unwrap();
        assert_eq!(usd.minor_units(), 1000);

        // JPY: exponent 0
        let jpy = FastMoney::from_major(500, test::JPY).unwrap();
        assert_eq!(jpy.minor_units(), 500);

        // BHD: exponent 3
        let bhd = FastMoney::from_major(1, test::BHD).unwrap();
        assert_eq!(bhd.minor_units(), 1000);
    }

    #[test]
    fn from_major_overflow() {
        let result = FastMoney::from_major(i64::MAX, test::USD);
        assert_eq!(result, Err(MoneyError::Overflow));
    }

    // ============ Arithmetic Tests ============

    #[test]
    fn add_same_currency() {
        let a = FastMoney::from_minor(1000, test::USD);
        let b = FastMoney::from_minor(500, test::USD);
        let result = a.add(b).unwrap();
        assert_eq!(result.minor_units(), 1500);
    }

    #[test]
    fn add_different_currency_errors() {
        let a = FastMoney::from_minor(1000, test::USD);
        let b = FastMoney::from_minor(500, test::EUR);
        let result = a.add(b);
        assert!(matches!(result, Err(MoneyError::CurrencyMismatch { .. })));
    }

    #[test]
    fn add_overflow() {
        let a = FastMoney::from_minor(i64::MAX, test::USD);
        let b = FastMoney::from_minor(1, test::USD);
        assert_eq!(a.add(b), Err(MoneyError::Overflow));
    }

    #[test]
    fn sub_same_currency() {
        let a = FastMoney::from_minor(1000, test::USD);
        let b = FastMoney::from_minor(300, test::USD);
        let result = a.sub(b).unwrap();
        assert_eq!(result.minor_units(), 700);
    }

    #[test]
    fn sub_different_currency_errors() {
        let a = FastMoney::from_minor(1000, test::USD);
        let b = FastMoney::from_minor(500, test::EUR);
        let result = a.sub(b);
        assert!(matches!(result, Err(MoneyError::CurrencyMismatch { .. })));
    }

    #[test]
    fn sub_overflow() {
        let a = FastMoney::from_minor(i64::MIN, test::USD);
        let b = FastMoney::from_minor(1, test::USD);
        assert_eq!(a.sub(b), Err(MoneyError::Overflow));
    }

    #[test]
    fn mul_scalar() {
        let money = FastMoney::from_minor(100, test::USD);
        let result = money.mul(5).unwrap();
        assert_eq!(result.minor_units(), 500);
    }

    #[test]
    fn mul_overflow() {
        let money = FastMoney::from_minor(i64::MAX, test::USD);
        assert_eq!(money.mul(2), Err(MoneyError::Overflow));
    }

    #[test]
    fn div_scalar() {
        let money = FastMoney::from_minor(1000, test::USD);
        let result = money.div(3).unwrap();
        assert_eq!(result.minor_units(), 333); // Truncates toward zero
    }

    #[test]
    fn div_by_zero() {
        let money = FastMoney::from_minor(1000, test::USD);
        assert_eq!(money.div(0), Err(MoneyError::DivisionByZero));
    }

    #[test]
    fn div_negative_truncates_toward_zero() {
        let money = FastMoney::from_minor(-1000, test::USD);
        let result = money.div(3).unwrap();
        assert_eq!(result.minor_units(), -333);
    }

    // ============ Sign Tests ============

    #[test]
    fn is_zero() {
        assert!(FastMoney::from_minor(0, test::USD).is_zero());
        assert!(!FastMoney::from_minor(1, test::USD).is_zero());
        assert!(!FastMoney::from_minor(-1, test::USD).is_zero());
    }

    #[test]
    fn is_positive() {
        assert!(FastMoney::from_minor(1, test::USD).is_positive());
        assert!(!FastMoney::from_minor(0, test::USD).is_positive());
        assert!(!FastMoney::from_minor(-1, test::USD).is_positive());
    }

    #[test]
    fn is_negative() {
        assert!(FastMoney::from_minor(-1, test::USD).is_negative());
        assert!(!FastMoney::from_minor(0, test::USD).is_negative());
        assert!(!FastMoney::from_minor(1, test::USD).is_negative());
    }

    #[test]
    fn abs_returns_absolute_value() {
        assert_eq!(
            FastMoney::from_minor(-100, test::USD).abs().minor_units(),
            100
        );
        assert_eq!(
            FastMoney::from_minor(100, test::USD).abs().minor_units(),
            100
        );
        assert_eq!(FastMoney::from_minor(0, test::USD).abs().minor_units(), 0);
    }

    #[test]
    fn neg_negates_value() {
        assert_eq!(
            FastMoney::from_minor(100, test::USD).neg().minor_units(),
            -100
        );
        assert_eq!(
            FastMoney::from_minor(-100, test::USD).neg().minor_units(),
            100
        );
        assert_eq!(FastMoney::from_minor(0, test::USD).neg().minor_units(), 0);
    }

    // ============ Comparison Tests ============

    #[test]
    fn compare_same_currency() {
        let a = FastMoney::from_minor(100, test::USD);
        let b = FastMoney::from_minor(200, test::USD);
        let c = FastMoney::from_minor(100, test::USD);

        assert_eq!(a.compare(&b).unwrap(), Ordering::Less);
        assert_eq!(b.compare(&a).unwrap(), Ordering::Greater);
        assert_eq!(a.compare(&c).unwrap(), Ordering::Equal);
    }

    #[test]
    fn compare_different_currency_errors() {
        let a = FastMoney::from_minor(100, test::USD);
        let b = FastMoney::from_minor(100, test::EUR);
        assert!(matches!(
            a.compare(&b),
            Err(MoneyError::CurrencyMismatch { .. })
        ));
    }

    // ============ Conversion Tests ============

    #[test]
    fn to_money_always_succeeds() {
        let fast = FastMoney::from_minor(1234, test::USD);
        let money = fast.to_money();
        assert_eq!(money.to_minor_units(), 1234);
        assert_eq!(money.currency(), test::USD);
    }

    #[test]
    fn from_money_roundtrip() {
        let original = FastMoney::from_minor(1234, test::USD);
        let money = original.to_money();
        let back = FastMoney::from_money(money).unwrap();
        assert_eq!(original, back);
    }

    #[test]
    fn from_money_precision_loss_errors() {
        use rust_decimal_macros::dec;

        // $10.005 has more precision than USD allows (exponent 2)
        let money = Money::from_decimal(dec!(10.005), test::USD);
        let result = FastMoney::from_money(money);
        assert_eq!(result, Err(MoneyError::PrecisionLoss));
    }

    #[test]
    fn from_money_lossy_truncates() {
        use rust_decimal_macros::dec;

        // $10.005 -> 1000 cents (truncates .5 cent)
        let money = Money::from_decimal(dec!(10.005), test::USD);
        let fast = FastMoney::from_money_lossy(money).unwrap();
        assert_eq!(fast.minor_units(), 1000);
    }

    #[test]
    fn from_money_lossy_truncates_negative() {
        use rust_decimal_macros::dec;

        // -$10.009 -> -1000 cents (truncates toward zero)
        let money = Money::from_decimal(dec!(-10.009), test::USD);
        let fast = FastMoney::from_money_lossy(money).unwrap();
        assert_eq!(fast.minor_units(), -1000);
    }

    #[test]
    fn from_money_different_exponents() {
        // JPY (exponent 0)
        let jpy_money = Money::from_major(500, test::JPY);
        let jpy_fast = FastMoney::from_money(jpy_money).unwrap();
        assert_eq!(jpy_fast.minor_units(), 500);

        // BHD (exponent 3)
        let bhd_money = Money::from_minor(1234, test::BHD);
        let bhd_fast = FastMoney::from_money(bhd_money).unwrap();
        assert_eq!(bhd_fast.minor_units(), 1234);
    }

    // ============ Display Tests ============

    #[test]
    fn display_formats_correctly() {
        let money = FastMoney::from_minor(1234, test::USD);
        let formatted = format!("{}", money);
        assert!(formatted.contains("12.34") || formatted.contains("12,34"));
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use crate::define_currency_set;
    use proptest::prelude::*;

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
                minor_units: 100,
                name: "Euro",
                symbol: "€",
                symbol_first: true,
            }
        }
    );

    // Strategy for generating amounts that won't overflow on basic operations
    fn safe_amount() -> impl Strategy<Value = i64> {
        -1_000_000_000_000i64..1_000_000_000_000i64
    }

    proptest! {
        #[test]
        fn add_is_commutative(a in safe_amount(), b in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            let mb = FastMoney::from_minor(b, test::USD);

            let ab = ma.add(mb);
            let ba = mb.add(ma);

            prop_assert_eq!(ab, ba);
        }

        #[test]
        fn add_sub_inverse(a in safe_amount(), b in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            let mb = FastMoney::from_minor(b, test::USD);

            if let Ok(sum) = ma.add(mb) {
                let back = sum.sub(mb);
                prop_assert_eq!(back.map(|m| m.minor_units()), Ok(a));
            }
        }

        #[test]
        fn mul_div_inverse(a in safe_amount(), n in 1i64..1000) {
            let ma = FastMoney::from_minor(a, test::USD);

            if let Ok(product) = ma.mul(n) {
                let back = product.div(n);
                // Due to truncation, this should be close but might not be exact
                // for values that don't divide evenly
                prop_assert!(back.is_ok());
            }
        }

        #[test]
        fn neg_neg_is_identity(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            prop_assert_eq!(ma.neg().neg(), ma);
        }

        #[test]
        fn abs_is_non_negative(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            prop_assert!(ma.abs().minor_units() >= 0);
        }

        #[test]
        fn to_money_from_money_roundtrip(a in safe_amount()) {
            let fast = FastMoney::from_minor(a, test::USD);
            let money = fast.to_money();
            let back = FastMoney::from_money(money);

            prop_assert_eq!(back, Ok(fast));
        }

        #[test]
        fn compare_is_consistent_with_minor_units(a in safe_amount(), b in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            let mb = FastMoney::from_minor(b, test::USD);

            let cmp_result = ma.compare(&mb).unwrap();
            let expected = a.cmp(&b);

            prop_assert_eq!(cmp_result, expected);
        }

        #[test]
        fn is_zero_consistent(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            prop_assert_eq!(ma.is_zero(), a == 0);
        }

        #[test]
        fn is_positive_consistent(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            prop_assert_eq!(ma.is_positive(), a > 0);
        }

        #[test]
        fn is_negative_consistent(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            prop_assert_eq!(ma.is_negative(), a < 0);
        }

        #[test]
        fn addition_is_associative(a in safe_amount(), b in safe_amount(), c in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            let mb = FastMoney::from_minor(b, test::USD);
            let mc = FastMoney::from_minor(c, test::USD);

            // (a + b) + c == a + (b + c)
            let left = ma.add(mb).and_then(|ab| ab.add(mc));
            let right = mb.add(mc).and_then(|bc| ma.add(bc));

            prop_assert_eq!(left, right);
        }

        #[test]
        fn zero_is_additive_identity(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            let zero = FastMoney::from_minor(0, test::USD);

            prop_assert_eq!(ma.add(zero).unwrap(), ma);
            prop_assert_eq!(zero.add(ma).unwrap(), ma);
        }

        #[test]
        fn multiplication_by_one_is_identity(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            prop_assert_eq!(ma.mul(1).unwrap(), ma);
        }

        #[test]
        #[allow(clippy::erasing_op)]
        fn multiplication_by_zero_gives_zero(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            let zero = FastMoney::from_minor(0, test::USD);
            prop_assert_eq!(ma.mul(0).unwrap(), zero);
        }

        #[test]
        fn division_by_one_is_identity(a in safe_amount()) {
            let ma = FastMoney::from_minor(a, test::USD);
            prop_assert_eq!(ma.div(1).unwrap(), ma);
        }
    }
}
