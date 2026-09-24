use crate::MoneyError;
use crate::currency::FormattableCurrency;
use crate::exchange::Exchange;
use crate::format::{Formatter, Params, Position};
use crate::locale::LocalFormat;

use std::cmp::Ordering;
use std::fmt;
use std::ops::Neg;
use std::str::FromStr;

use rust_decimal::Decimal;

/// Represents an amount of a given currency.
///
/// Money stores an owned [`Decimal`] and a currency reference. Stored precision is
/// independent of the currency's exponent, within Decimal's finite range and scale.
/// Arithmetic and [`round`](Self::round) return new values without changing the original.
/// Display rounds to the currency's exponent using [`Round::HalfEven`] without
/// changing the stored amount.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Money<'a, T: FormattableCurrency> {
    amount: Decimal,
    currency: &'a T,
}

impl<'a, T: FormattableCurrency> Neg for Money<'a, T> {
    type Output = Money<'a, T>;

    #[inline]
    fn neg(self) -> Self::Output {
        Money {
            amount: -self.amount,
            currency: self.currency,
        }
    }
}

impl<'a, T: FormattableCurrency> Money<'a, T> {
    /// Creates a Money object given an amount string and a currency str.
    ///
    /// Supports fuzzy amount strings like "100", "100.00" and "-100.00"
    pub fn from_str(amount: &str, currency: &'a T) -> Result<Money<'a, T>, MoneyError> {
        if amount.is_empty() {
            return Err(MoneyError::InvalidAmount);
        }

        let format = LocalFormat::from_locale(currency.locale());
        let mut amount_parts = amount.split(format.exponent_separator);
        let integer = amount_parts.next().unwrap();
        let fraction = amount_parts.next();
        if amount_parts.next().is_some() {
            return Err(MoneyError::InvalidAmount);
        }

        if let Some(fraction) = fraction {
            // Validate fractional digits without imposing an integer range limit.
            if fraction.is_empty() || !fraction.bytes().all(|digit| digit.is_ascii_digit()) {
                return Err(MoneyError::InvalidAmount);
            }
        }

        let grouped = integer.contains(format.digit_separator);
        if grouped {
            let mut groups = integer.rsplit(format.digit_separator).peekable();
            for &size in format.digit_separator_pattern {
                let group = groups.next().unwrap();
                if groups.peek().is_none() {
                    break;
                }
                if group.len() != size {
                    return Err(MoneyError::InvalidAmount);
                }
            }
        }

        // A decimal point and ungrouped digits already match Decimal's input format.
        let normalized;
        let decimal_input = if fraction.is_some() && !grouped && format.exponent_separator == '.' {
            amount
        } else {
            let mut buffer = String::with_capacity(amount.len());
            if grouped {
                for group in integer.split(format.digit_separator) {
                    buffer.push_str(group);
                }
            } else {
                buffer.push_str(integer);
            }
            buffer.push('.');
            if let Some(fraction) = fraction {
                buffer.push_str(fraction);
            } else {
                // Preserve the scale of integer input, including trailing zeroes.
                for _ in 0..currency.exponent() {
                    buffer.push('0');
                }
            }
            normalized = buffer;
            &normalized
        };

        let decimal = Decimal::from_str(decimal_input).map_err(|_| MoneyError::InvalidAmount)?;
        Ok(Money::from_decimal(decimal, currency))
    }

    /// Creates a Money object given an integer and a currency reference.
    ///
    /// The integer represents minor units of the currency (e.g. 1000 -> 10.00 in USD )
    ///
    /// # Panics
    /// Panics if the currency's exponent exceeds Decimal's maximum scale of 28.
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
    /// The amount is stored unchanged, including any fractional minor units.
    pub fn from_decimal(amount: Decimal, currency: &'a T) -> Money<'a, T> {
        Money { amount, currency }
    }

    /// Returns a reference to the Decimal amount.
    #[inline]
    pub fn amount(&self) -> &Decimal {
        &self.amount
    }

    /// Returns the Currency type.
    #[inline]
    pub fn currency(&self) -> &'a T {
        self.currency
    }

    /// Returns true if amount == 0.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.amount == Decimal::ZERO
    }

    /// Returns true if amount > 0.
    #[inline]
    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive() && self.amount != Decimal::ZERO
    }

    /// Returns true if amount < 0.
    #[inline]
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative() && self.amount != Decimal::ZERO
    }

    /// Returns a new Money with the absolute value of the amount.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// let negative = Money::from_minor(-500, iso::USD);
    /// let positive = negative.abs();
    /// assert_eq!(positive, Money::from_minor(500, iso::USD));
    /// ```
    #[inline]
    pub fn abs(&self) -> Money<'a, T> {
        Money::from_decimal(self.amount.abs(), self.currency)
    }

    /// Returns the amount in minor units (e.g., cents for USD, pence for GBP).
    ///
    /// Prefer [`try_to_minor_units`](Self::try_to_minor_units) for an exact,
    /// checked conversion.
    ///
    /// The conversion multiplies by 10^exponent where exponent is the currency's
    /// decimal places (2 for USD, 0 for JPY, 3 for BHD).
    ///
    /// Fractional minor units are truncated toward zero. If the scaled value does
    /// not fit in `i64`, returns 0. For example, USD `-1.005` becomes `-100` cents.
    ///
    /// # Panics
    /// Intermediate scaling can overflow before the `i64` conversion, including
    /// for large Decimal amounts or custom currency exponents.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// let money = Money::from_major(123, iso::USD); // $123.00
    /// assert_eq!(money.to_minor_units(), 12300);    // 12300 cents
    ///
    /// let jpy = Money::from_major(500, iso::JPY);   // ¥500 (exponent 0)
    /// assert_eq!(jpy.to_minor_units(), 500);
    /// ```
    #[inline]
    pub fn to_minor_units(&self) -> i64 {
        use rust_decimal::prelude::ToPrimitive;
        let scale = Decimal::from(10u64.pow(self.currency.exponent()));
        (self.amount * scale).trunc().to_i64().unwrap_or(0)
    }

    /// Returns the exact amount in minor units, without rounding or truncation.
    ///
    /// Trailing fractional zeros do not count as excess precision. To choose a
    /// rounding policy, call [`round`](Self::round) with the currency's exponent
    /// before converting. This method does not change the stored amount.
    ///
    /// # Errors
    /// Returns [`MoneyError::PrecisionLoss`] if the amount includes a fractional
    /// minor unit, or [`MoneyError::Overflow`] if the integral minor-unit amount
    /// is outside the `i64` range. Fractional minor units are checked first.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, MoneyError, Round, iso};
    /// use rust_decimal_macros::dec;
    ///
    /// let exact = Money::from_decimal(dec!(12.3400), iso::USD);
    /// assert_eq!(exact.try_to_minor_units(), Ok(1234));
    ///
    /// let fractional = Money::from_decimal(dec!(-1.005), iso::USD);
    /// assert_eq!(fractional.try_to_minor_units(), Err(MoneyError::PrecisionLoss));
    /// assert_eq!(fractional.round(2, Round::HalfUp).try_to_minor_units(), Ok(-101));
    ///
    /// let large = Money::from_major(100_000_000_000_000_000, iso::USD);
    /// assert_eq!(large.try_to_minor_units(), Err(MoneyError::Overflow));
    /// ```
    pub fn try_to_minor_units(&self) -> Result<i64, MoneyError> {
        let coefficient = self.amount.mantissa();
        if coefficient == 0 {
            return Ok(0);
        }

        let scale = self.amount.scale();
        let exponent = self.currency.exponent();
        // Rescale the integer coefficient directly to avoid Decimal rounding
        // and overflowing intermediate powers for high-exponent currencies.
        let minor_units = if scale > exponent {
            // Decimal's scale is at most 28, so this power fits in i128.
            let divisor = 10i128.pow(scale - exponent);
            if coefficient % divisor != 0 {
                return Err(MoneyError::PrecisionLoss);
            }
            coefficient / divisor
        } else {
            let multiplier = 10i128
                .checked_pow(exponent - scale)
                .ok_or(MoneyError::Overflow)?;
            coefficient
                .checked_mul(multiplier)
                .ok_or(MoneyError::Overflow)?
        };

        i64::try_from(minor_units).map_err(|_| MoneyError::Overflow)
    }

    /// Returns the amount as a 64-bit float.
    ///
    /// # Precision Loss
    /// IEEE 754 `f64` has 53 bits of mantissa, providing ~15-17 significant decimal digits.
    /// Precision loss occurs when:
    ///
    /// - **Large amounts**: Values above ~9,007,199,254,740,992 (2^53) lose integer precision.
    ///   For USD, this is ~$90 trillion - unlikely in practice.
    /// - **Many decimal places**: Values like `0.1` cannot be exactly represented in binary
    ///   floating-point. The error is typically < 1e-15 relative to the value.
    /// - **Repeated arithmetic**: Errors accumulate with successive float operations.
    ///
    /// For precise calculations, use [`amount()`](Self::amount) which returns a `Decimal`.
    ///
    /// Returns `f64::NAN` if conversion fails (e.g., value exceeds f64 range).
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// let money = Money::from_minor(12345, iso::USD);
    /// assert_eq!(money.to_f64_lossy(), 123.45);
    /// ```
    pub fn to_f64_lossy(&self) -> f64 {
        use rust_decimal::prelude::ToPrimitive;
        self.amount.to_f64().unwrap_or(f64::NAN)
    }

    /// Adds two Money values, returning an error on currency mismatch or overflow.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// let a = Money::from_minor(100, iso::USD);
    /// let b = Money::from_minor(200, iso::USD);
    /// let sum = a.add(b).unwrap();
    /// assert_eq!(sum, Money::from_minor(300, iso::USD));
    /// ```
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if the two Money values have different currencies.
    /// Returns `MoneyError::Overflow` if the addition overflows.
    #[inline]
    pub fn add(&self, other: Money<'a, T>) -> Result<Money<'a, T>, MoneyError> {
        if self.currency.is_currency_mismatch(other.currency) {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.code(),
                actual: other.currency.code(),
            });
        }
        self.amount
            .checked_add(other.amount)
            .map(|result| Money::from_decimal(result, self.currency))
            .ok_or(MoneyError::Overflow)
    }

    /// Subtracts two Money values, returning an error on currency mismatch or overflow.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// let a = Money::from_minor(300, iso::USD);
    /// let b = Money::from_minor(100, iso::USD);
    /// let diff = a.sub(b).unwrap();
    /// assert_eq!(diff, Money::from_minor(200, iso::USD));
    /// ```
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if the two Money values have different currencies.
    /// Returns `MoneyError::Overflow` if the subtraction overflows.
    #[inline]
    pub fn sub(&self, other: Money<'a, T>) -> Result<Money<'a, T>, MoneyError> {
        if self.currency.is_currency_mismatch(other.currency) {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.code(),
                actual: other.currency.code(),
            });
        }
        self.amount
            .checked_sub(other.amount)
            .map(|result| Money::from_decimal(result, self.currency))
            .ok_or(MoneyError::Overflow)
    }

    /// Multiplies a Money value by a scalar, returning an error on overflow.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// let money = Money::from_minor(100, iso::USD);
    /// let tripled = money.mul(3).unwrap();
    /// assert_eq!(tripled, Money::from_minor(300, iso::USD));
    /// ```
    ///
    /// # Errors
    /// Returns `MoneyError::Overflow` if the multiplication overflows.
    #[inline]
    pub fn mul<N: Into<Decimal>>(&self, n: N) -> Result<Money<'a, T>, MoneyError> {
        self.amount
            .checked_mul(n.into())
            .map(|result| Money::from_decimal(result, self.currency))
            .ok_or(MoneyError::Overflow)
    }

    /// Divides a Money value by a scalar, returning an error on division by zero or overflow.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// let money = Money::from_minor(100, iso::USD);
    /// let half = money.div(2i64).unwrap();
    /// assert_eq!(half, Money::from_minor(50, iso::USD));
    /// ```
    ///
    /// # Errors
    /// Returns `MoneyError::DivisionByZero` if `n` is zero.
    /// Returns `MoneyError::Overflow` if the division overflows.
    #[inline]
    pub fn div<N: Into<Decimal> + Copy + PartialEq + Default>(
        &self,
        n: N,
    ) -> Result<Money<'a, T>, MoneyError> {
        let divisor = n.into();
        if divisor.is_zero() {
            return Err(MoneyError::DivisionByZero);
        }
        self.amount
            .checked_div(divisor)
            .map(|result| Money::from_decimal(result, self.currency))
            .ok_or(MoneyError::Overflow)
    }

    /// Converts this Money to another currency using the provided exchange rates.
    ///
    /// This is a convenience method that looks up the exchange rate and performs
    /// the conversion in one step.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, Exchange, ExchangeRate, iso};
    /// use rust_decimal_macros::dec;
    ///
    /// let mut exchange = Exchange::new();
    /// let rate = ExchangeRate::new(iso::USD, iso::EUR, dec!(0.85)).unwrap();
    /// exchange.set_rate(&rate);
    ///
    /// let usd = Money::from_minor(1000, iso::USD); // $10.00
    /// let eur = usd.exchange_to(iso::EUR, &exchange).unwrap();
    /// assert_eq!(eur, Money::from_minor(850, iso::EUR)); // €8.50
    /// ```
    ///
    /// # Errors
    /// Returns `MoneyError::InvalidCurrency` if no exchange rate exists for the currency pair.
    /// Returns `MoneyError::Overflow` if the conversion overflows.
    pub fn exchange_to(
        &self,
        target: &'a T,
        exchange: &Exchange<'a, T>,
    ) -> Result<Money<'a, T>, MoneyError> {
        let rate = exchange
            .get_rate(self.currency, target)
            .ok_or(MoneyError::InvalidCurrency)?;
        rate.convert(self)
    }

    /// Compares two Money values, returning the ordering or an error if currencies don't match.
    ///
    /// For simple boolean comparisons, prefer the helper methods: `gt`, `gte`, `lt`, `lte`, `eq`.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// use std::cmp::Ordering;
    /// let a = Money::from_minor(100, iso::USD);
    /// let b = Money::from_minor(200, iso::USD);
    /// assert_eq!(a.compare(&b).unwrap(), Ordering::Less);
    /// ```
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if the two Money values have different currencies.
    #[inline]
    pub fn compare(&self, other: &Money<'a, T>) -> Result<Ordering, MoneyError> {
        if self.currency.is_currency_mismatch(other.currency) {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.code(),
                actual: other.currency.code(),
            });
        }
        Ok(self.amount.cmp(&other.amount))
    }

    /// Returns true if self > other.
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if currencies don't match.
    #[inline]
    pub fn gt(&self, other: &Money<'a, T>) -> Result<bool, MoneyError> {
        Ok(self.compare(other)?.is_gt())
    }

    /// Returns true if self >= other.
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if currencies don't match.
    #[inline]
    pub fn gte(&self, other: &Money<'a, T>) -> Result<bool, MoneyError> {
        Ok(self.compare(other)?.is_ge())
    }

    /// Returns true if self < other.
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if currencies don't match.
    #[inline]
    pub fn lt(&self, other: &Money<'a, T>) -> Result<bool, MoneyError> {
        Ok(self.compare(other)?.is_lt())
    }

    /// Returns true if self <= other.
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if currencies don't match.
    #[inline]
    pub fn lte(&self, other: &Money<'a, T>) -> Result<bool, MoneyError> {
        Ok(self.compare(other)?.is_le())
    }

    /// Returns true if self == other (same currency and amount).
    ///
    /// # Errors
    /// Returns `MoneyError::CurrencyMismatch` if currencies don't match.
    #[inline]
    pub fn eq(&self, other: &Money<'a, T>) -> Result<bool, MoneyError> {
        Ok(self.compare(other)?.is_eq())
    }

    /// Divides money equally into n shares.
    ///
    /// Floors the amount to integral minor units, then floors each equal share.
    /// Remaining minor units are added to the first shares in order. Flooring
    /// also applies to negative values: USD `-1.005` is split as `-1.01`, not `-1.00`.
    /// Round explicitly first if a different policy is needed.
    ///
    /// # Errors
    /// Returns [`MoneyError::InvalidRatio`] if `n` is zero.
    /// Returns [`MoneyError::Overflow`] if the currency exponent exceeds 28,
    /// minor units do not fit in `i128`, or a share cannot be represented exactly
    /// as a Decimal. Successful results preserve the floored total.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, iso};
    /// let money = Money::from_minor(1000, iso::USD); // $10.00
    /// let parts = money.split(3).unwrap();
    /// assert_eq!(parts[0], Money::from_minor(334, iso::USD)); // $3.34
    /// assert_eq!(parts[1], Money::from_minor(333, iso::USD)); // $3.33
    /// assert_eq!(parts[2], Money::from_minor(333, iso::USD)); // $3.33
    /// ```
    pub fn split(&self, n: u32) -> Result<Vec<Money<'a, T>>, MoneyError> {
        if n == 0 {
            return Err(MoneyError::InvalidRatio);
        }
        let total_minor = self.allocation_minor_units()?;
        let base_share = total_minor.div_euclid(i128::from(n));
        let remainder_count = total_minor.rem_euclid(i128::from(n)) as usize;
        let low_share = self.allocation_share(base_share)?;
        // Do not compute an unused higher share at the maximum representable value.
        let high_share = if remainder_count == 0 {
            low_share
        } else {
            self.allocation_share(base_share.checked_add(1).ok_or(MoneyError::Overflow)?)?
        };
        let mut result = Vec::with_capacity(n as usize);
        for i in 0..n as usize {
            if i < remainder_count {
                result.push(high_share);
            } else {
                result.push(low_share);
            }
        }
        Ok(result)
    }

    /// Divides money into n shares according to the given weights.
    ///
    /// Floors the amount to integral minor units, then floors each weighted share.
    /// This rounds toward negative infinity, not toward zero. Fractions smaller
    /// than a minor unit are discarded; round explicitly first to choose another policy.
    ///
    /// If the division cannot be applied perfectly, it allocates the remainder
    /// to shares with non-zero weights in input order. Zero-weight shares receive zero.
    ///
    /// # Errors
    /// Returns [`MoneyError::InvalidRatio`] for empty or all-zero weights.
    /// Returns [`MoneyError::Overflow`] if the currency exponent exceeds 28,
    /// minor-unit arithmetic exceeds `i128`, the weight sum exceeds `u64`, or a
    /// share cannot be represented exactly as a Decimal, or result storage
    /// cannot be reserved. Successful results preserve the floored total.
    pub fn allocate(&self, shares: Vec<u32>) -> Result<Vec<Money<'a, T>>, MoneyError> {
        if shares.is_empty() {
            return Err(MoneyError::InvalidRatio);
        }
        let share_total = shares.iter().try_fold(0u64, |total, &share| {
            total
                .checked_add(u64::from(share))
                .ok_or(MoneyError::Overflow)
        })?;
        if share_total == 0 {
            return Err(MoneyError::InvalidRatio);
        }
        let total_minor = self.allocation_minor_units()?;
        let denominator = i128::from(share_total);
        let quotient = total_minor.div_euclid(denominator);
        let residual = total_minor.rem_euclid(denominator);
        let mut allocations_minor = Vec::with_capacity(shares.len());
        let mut allocated = 0i128;
        for &share in &shares {
            // floor(total * weight / sum), without multiplying the full total
            // by the weight. residual < u64::MAX and weight <= u32::MAX.
            let weight = i128::from(share);
            let share_value = quotient
                .checked_mul(weight)
                .and_then(|base| base.checked_add(residual * weight / denominator))
                .ok_or(MoneyError::Overflow)?;
            allocations_minor.push(share_value);
            allocated = allocated
                .checked_add(share_value)
                .ok_or(MoneyError::Overflow)?;
        }
        let mut remainder = total_minor
            .checked_sub(allocated)
            .ok_or(MoneyError::Overflow)?;
        for (allocation, &weight) in allocations_minor.iter_mut().zip(&shares) {
            if remainder == 0 {
                break;
            }
            if weight > 0 {
                *allocation = allocation.checked_add(1).ok_or(MoneyError::Overflow)?;
                remainder -= 1;
            }
        }
        debug_assert_eq!(remainder, 0);
        // Result collection does not retain this iterator's exact length hint.
        // Reserve once to avoid repeatedly reallocating the output vector.
        let mut result = Vec::new();
        result
            .try_reserve_exact(allocations_minor.len())
            .map_err(|_| MoneyError::Overflow)?;
        for minor in allocations_minor {
            result.push(self.allocation_share(minor)?);
        }
        Ok(result)
    }

    // Floor to minor units without Decimal division or an overflowing u64 power.
    fn allocation_minor_units(&self) -> Result<i128, MoneyError> {
        let exponent = self.currency.exponent();
        if exponent > Decimal::MAX_SCALE {
            return Err(MoneyError::Overflow);
        }
        let coefficient = self.amount.mantissa();
        let scale = self.amount.scale();
        if scale > exponent {
            Ok(coefficient.div_euclid(10i128.pow(scale - exponent)))
        } else {
            coefficient
                .checked_mul(10i128.pow(exponent - scale))
                .ok_or(MoneyError::Overflow)
        }
    }

    // Remove only redundant zeros if needed to fit Decimal's 96-bit coefficient.
    // Never round a share: that would invalidate conservation of the total.
    fn allocation_share(&self, mut minor: i128) -> Result<Money<'a, T>, MoneyError> {
        let mut scale = self.currency.exponent();
        while minor.unsigned_abs() > Decimal::MAX.mantissa() as u128 && scale > 0 && minor % 10 == 0
        {
            minor /= 10;
            scale -= 1;
        }
        let amount =
            Decimal::try_from_i128_with_scale(minor, scale).map_err(|_| MoneyError::Overflow)?;
        Ok(Money::from_decimal(amount, self.currency))
    }

    /// Returns a new `Money` rounded to `digits` decimal places using the strategy.
    ///
    /// The original amount is unchanged. Use the currency's exponent for rounding
    /// to minor units. This cannot increase Decimal's available precision.
    ///
    /// # Example
    /// ```
    /// use rusty_money::{Money, Round, iso};
    /// let amount = Money::from_str("-1.005", iso::USD).unwrap();
    /// assert_eq!(amount.round(2, Round::HalfUp).amount().to_string(), "-1.01");
    /// assert_eq!(amount.round(2, Round::HalfDown).amount().to_string(), "-1.00");
    /// assert_eq!(amount.amount().to_string(), "-1.005");
    /// ```
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
    /// Round to nearest, with exact midpoints rounded away from zero.
    HalfUp,
    /// Round to nearest, with exact midpoints rounded toward zero.
    HalfDown,
    /// Round to nearest, with exact midpoints rounded to an even retained digit.
    HalfEven,
}

// Static position arrays for zero-allocation Display formatting
const POSITIONS_SYMBOL_FIRST: &[Position] = &[Position::Sign, Position::Symbol, Position::Amount];
const POSITIONS_SYMBOL_LAST: &[Position] = &[Position::Sign, Position::Amount, Position::Symbol];

impl<'a, T: FormattableCurrency> fmt::Display for Money<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let currency = self.currency;
        let format = LocalFormat::from_locale(currency.locale());

        let positions = if currency.symbol_first() {
            POSITIONS_SYMBOL_FIRST
        } else {
            POSITIONS_SYMBOL_LAST
        };

        let format_params = Params {
            digit_separator: format.digit_separator,
            exponent_separator: format.exponent_separator,
            separator_pattern: format.digit_separator_pattern,
            positions,
            rounding: Some(currency.exponent()),
            symbol: Some(currency.symbol()),
            code: Some(currency.code()),
        };

        Formatter::write_money(self, format_params, f)
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
    use std::borrow::Cow;
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
                    let mut currency_code: Option<Cow<'de, str>> = None;

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
        fn from_str_preserves_long_fractional_amounts() {
            for amount in [
                "1.2147483648",
                "1.11111111111",
                "0.123456789012345678",
                "-1.123456789012345678",
                "0.1234567890123456789012345678",
            ] {
                let expected = Decimal::from_str(amount).unwrap();
                for currency in [test::USD, test::EUR, test::INR] {
                    let separator = LocalFormat::from_locale(currency.locale()).exponent_separator;
                    let localized = amount.replace('.', &separator.to_string());
                    let money = Money::from_str(&localized, currency).unwrap();
                    assert_eq!(*money.amount(), expected, "{localized} {}", currency.code());
                }
            }
        }

        #[cfg(feature = "crypto")]
        #[test]
        fn from_str_preserves_ethereum_fractional_amounts() {
            for amount in ["1.11111111111", "0.123456789012345678"] {
                let expected =
                    Money::from_decimal(Decimal::from_str(amount).unwrap(), crate::crypto::ETH);
                assert_eq!(
                    Money::from_str(amount, crate::crypto::ETH).unwrap(),
                    expected
                );
            }
        }

        #[test]
        fn from_str_rejects_invalid_fractional_parts() {
            for fraction in ["", "+1", "-1", "1_0", "1a", "1 0", "1,000", "１２", "1.2"] {
                let amount = format!("1.{fraction}");
                assert_eq!(
                    Money::from_str(&amount, test::USD).unwrap_err(),
                    MoneyError::InvalidAmount,
                    "{amount}"
                );
            }
        }

        #[test]
        fn from_str_preserves_normalized_values_and_scales() {
            let space_currency = test::Currency {
                locale: crate::Locale::EnBy,
                ..*test::USD
            };
            for (input, currency, expected) in [
                ("1234", test::USD, "1234.00"),
                ("1234", test::JPY, "1234"),
                ("1.50", test::JPY, "1.50"),
                ("1,234.500", test::USD, "1234.500"),
                ("1.234,500", test::EUR, "1234.500"),
                ("1,23,456.7800", test::INR, "123456.7800"),
                ("1 234,50", &space_currency, "1234.50"),
                ("-0", test::USD, "0.00"),
                ("0.0000", test::USD, "0.0000"),
                (".50", test::USD, "0.50"),
                ("+1.50", test::USD, "1.50"),
                ("1_234.50", test::USD, "1234.50"),
                (",123.45", test::USD, "123.45"),
                ("12,34,567,890,123.00", test::USD, "1234567890123.00"),
                (
                    "79228162514264337593543950335",
                    test::USD,
                    "79228162514264337593543950335",
                ),
            ] {
                let money = Money::from_str(input, currency).unwrap();
                let expected = Decimal::from_str(expected).unwrap();
                // Decimal equality ignores scale, but callers can observe it through amount().
                assert_eq!(money.amount().serialize(), expected.serialize(), "{input}");
            }
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

            // Empty string returns an error
            assert_eq!(
                Money::from_str("", test::USD).unwrap_err(),
                MoneyError::InvalidAmount
            );

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
            // Addition
            let sum = Money::from_major(1, test::USD)
                .add(Money::from_major(1, test::USD))
                .unwrap();
            assert_eq!(Money::from_major(2, test::USD), sum);

            // Subtraction
            let diff = Money::from_major(1, test::USD)
                .sub(Money::from_major(1, test::USD))
                .unwrap();
            assert_eq!(Money::from_major(0, test::USD), diff);
        }

        #[test]
        fn addition_returns_error_on_different_currencies() {
            let result = Money::from_minor(100, test::USD).add(Money::from_minor(100, test::GBP));
            assert!(result.is_err());
            match result.unwrap_err() {
                MoneyError::CurrencyMismatch { expected, actual } => {
                    assert_eq!(expected, "USD");
                    assert_eq!(actual, "GBP");
                }
                _ => panic!("Expected CurrencyMismatch error"),
            }
        }

        #[test]
        fn subtraction_returns_error_on_different_currencies() {
            let result = Money::from_minor(100, test::USD).sub(Money::from_minor(100, test::GBP));
            assert!(result.is_err());
            match result.unwrap_err() {
                MoneyError::CurrencyMismatch { expected, actual } => {
                    assert_eq!(expected, "USD");
                    assert_eq!(actual, "GBP");
                }
                _ => panic!("Expected CurrencyMismatch error"),
            }
        }

        #[test]
        fn multiplication() {
            // Multiplication with integer
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(100, test::USD).mul(2).unwrap()
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-100, test::USD).mul(-2).unwrap()
            );

            // Multiplication with decimal
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(100, test::USD)
                    .mul(Decimal::new(2, 0))
                    .unwrap()
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-100, test::USD)
                    .mul(Decimal::new(-2, 0))
                    .unwrap()
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(400, test::USD)
                    .mul(Decimal::new(5, 1))
                    .unwrap()
            );
        }

        #[test]
        fn division() {
            // Division with integer
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(400, test::USD).div(2).unwrap()
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-400, test::USD).div(-2).unwrap()
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-200, test::USD).div(-1).unwrap()
            );

            // Division with decimal
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(400, test::USD)
                    .div(Decimal::new(2, 0))
                    .unwrap()
            );
            assert_eq!(
                Money::from_minor(200, test::USD),
                Money::from_minor(-400, test::USD)
                    .div(Decimal::new(-2, 0))
                    .unwrap()
            );
            assert_eq!(
                Money::from_minor(400, test::USD),
                Money::from_minor(-200, test::USD)
                    .div(Decimal::new(-5, 1))
                    .unwrap()
            );
        }

        #[test]
        fn division_by_zero_returns_error() {
            let money = Money::from_minor(100, test::USD);
            let result = money.div(0);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MoneyError::DivisionByZero);
        }

        #[test]
        fn negation() {
            let money = Money::from_minor(100, test::USD);
            assert_eq!(-money, Money::from_minor(-100, test::USD));
        }

        #[test]
        fn copy_semantics() {
            let money = Money::from_major(1, test::USD);
            let _1st_derived_money = money.mul(3).unwrap();
            // if Money didn't impl Copy, this would fail to compile
            let _2nd_derived_money = money.mul(3).unwrap();
        }

        #[test]
        fn multiplication_overflow_returns_error() {
            let money = Money::from_decimal(Decimal::MAX, test::USD);
            let result = money.mul(2);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MoneyError::Overflow);
        }

        #[test]
        fn addition_overflow_returns_error() {
            for (amount, increment) in [(Decimal::MAX, 1), (Decimal::MIN, -1)] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(
                    money.add(Money::from_major(increment, test::USD)),
                    Err(MoneyError::Overflow)
                );
            }
        }

        #[test]
        fn subtraction_overflow_returns_error() {
            for (amount, decrement) in [(Decimal::MIN, 1), (Decimal::MAX, -1)] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(
                    money.sub(Money::from_major(decrement, test::USD)),
                    Err(MoneyError::Overflow)
                );
            }
        }

        #[test]
        fn division_overflow_returns_error() {
            for amount in [Decimal::MAX, Decimal::MIN] {
                let money = Money::from_decimal(amount, test::USD);
                for divisor in [Decimal::new(1, 1), Decimal::new(-1, 1)] {
                    assert_eq!(money.div(divisor), Err(MoneyError::Overflow));
                }
            }
        }

        #[test]
        fn arithmetic_at_decimal_limits_succeeds() {
            let zero = Money::from_major(0, test::USD);
            for amount in [Decimal::MAX, Decimal::MIN] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(money.add(zero), Ok(money));
                assert_eq!(money.sub(zero), Ok(money));
                assert_eq!(money.div(1), Ok(money));
                assert_eq!(money.div(-1), Ok(Money::from_decimal(-amount, test::USD)));
            }

            let one = Money::from_major(1, test::USD);
            let near_max = Money::from_decimal(Decimal::MAX - Decimal::ONE, test::USD);
            let near_min = Money::from_decimal(Decimal::MIN + Decimal::ONE, test::USD);
            assert_eq!(near_max.add(one).unwrap().amount(), &Decimal::MAX);
            assert_eq!(near_min.sub(one).unwrap().amount(), &Decimal::MIN);
        }

        #[test]
        fn currency_mismatch_takes_precedence_over_overflow() {
            let money = Money::from_decimal(Decimal::MAX, test::USD);
            let expected = Err(MoneyError::CurrencyMismatch {
                expected: "USD",
                actual: "GBP",
            });
            assert_eq!(money.add(Money::from_major(1, test::GBP)), expected);
            assert_eq!(money.sub(Money::from_major(-1, test::GBP)), expected);
        }

        #[test]
        fn division_by_zero_at_decimal_limits_returns_division_by_zero() {
            for amount in [Decimal::MAX, Decimal::MIN, Decimal::ZERO] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(money.div(0), Err(MoneyError::DivisionByZero));
                assert_eq!(
                    money.div(Decimal::new(0, 2)),
                    Err(MoneyError::DivisionByZero)
                );
            }
        }

        #[test]
        fn division_checks_the_converted_divisor_for_zero() {
            #[derive(Clone, Copy, PartialEq)]
            struct Divisor(Decimal);

            impl Default for Divisor {
                fn default() -> Self {
                    Self(Decimal::ONE)
                }
            }

            impl From<Divisor> for Decimal {
                fn from(value: Divisor) -> Self {
                    value.0
                }
            }

            let money = Money::from_major(1, test::USD);
            assert_eq!(
                money.div(Divisor(Decimal::ZERO)),
                Err(MoneyError::DivisionByZero)
            );
            assert_eq!(money.div(Divisor::default()), Ok(money));
        }
    }

    mod exchange {
        use super::*;
        use crate::ExchangeRate;
        use rust_decimal_macros::dec;

        #[test]
        fn exchange_to_overflow_returns_error() {
            let mut exchange = Exchange::new();
            let rate = ExchangeRate::new(test::USD, test::EUR, dec!(2)).unwrap();
            exchange.set_rate(&rate);

            for amount in [Decimal::MAX, Decimal::MIN] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(
                    money.exchange_to(test::EUR, &exchange),
                    Err(MoneyError::Overflow)
                );
            }
        }

        #[test]
        fn exchange_to_converts_currency() {
            let mut exchange = Exchange::new();
            let rate = ExchangeRate::new(test::USD, test::EUR, dec!(0.85)).unwrap();
            exchange.set_rate(&rate);

            let usd = Money::from_minor(1000, test::USD); // $10.00
            let eur = usd.exchange_to(test::EUR, &exchange).unwrap();

            assert_eq!(eur, Money::from_minor(850, test::EUR)); // €8.50
            assert_eq!(eur.currency(), test::EUR);
        }

        #[test]
        fn exchange_to_returns_error_when_rate_missing() {
            let exchange = Exchange::<test::Currency>::new();
            let usd = Money::from_minor(1000, test::USD);

            let result = usd.exchange_to(test::EUR, &exchange);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), MoneyError::InvalidCurrency);
        }

        #[test]
        fn exchange_to_with_zero_amount() {
            let mut exchange = Exchange::new();
            let rate = ExchangeRate::new(test::USD, test::EUR, dec!(0.85)).unwrap();
            exchange.set_rate(&rate);

            let zero_usd = Money::from_minor(0, test::USD);
            let zero_eur = zero_usd.exchange_to(test::EUR, &exchange).unwrap();

            assert!(zero_eur.is_zero());
            assert_eq!(zero_eur.currency(), test::EUR);
        }

        #[test]
        fn exchange_to_preserves_sign() {
            let mut exchange = Exchange::new();
            let rate = ExchangeRate::new(test::USD, test::EUR, dec!(0.85)).unwrap();
            exchange.set_rate(&rate);

            let negative_usd = Money::from_minor(-1000, test::USD);
            let negative_eur = negative_usd.exchange_to(test::EUR, &exchange).unwrap();

            assert!(negative_eur.is_negative());
            assert_eq!(negative_eur, Money::from_minor(-850, test::EUR));
        }
    }

    mod checked_minor_units {
        use super::*;
        use proptest::prelude::*;
        use rust_decimal_macros::dec;

        #[test]
        fn converts_major_units_and_trailing_zeros() {
            for (amount, currency, expected) in [
                (dec!(123.4500), test::USD, 12345),
                (dec!(-123.4500), test::USD, -12345),
                (dec!(500.00), test::JPY, 500),
                (dec!(1.2340), test::BHD, 1234),
            ] {
                let money = Money::from_decimal(amount, currency);
                assert_eq!(money.try_to_minor_units(), Ok(expected));
            }
        }

        #[test]
        fn roundtrips_i64_limits_across_currency_exponents() {
            for exponent in [0, 2, 28] {
                let currency = test::Currency {
                    exponent,
                    ..*test::USD
                };
                for minor in [i64::MIN, i64::MAX] {
                    let money = Money::from_minor(minor, &currency);
                    assert_eq!(money.try_to_minor_units(), Ok(minor));
                }
            }
        }

        #[test]
        fn rejects_fractional_minor_units() {
            for amount in [dec!(1.001), dec!(-1.001), dec!(0.001), dec!(-0.001)] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(money.try_to_minor_units(), Err(MoneyError::PrecisionLoss));
            }
            let yen = Money::from_decimal(dec!(1.1), test::JPY);
            assert_eq!(yen.try_to_minor_units(), Err(MoneyError::PrecisionLoss));
        }

        #[test]
        fn rejects_out_of_range_integral_minor_units() {
            for exponent in [0, 2, 28] {
                let currency = test::Currency {
                    exponent,
                    ..*test::USD
                };
                for coefficient in [i64::MAX as i128 + 1, i64::MIN as i128 - 1] {
                    let amount = Decimal::from_i128_with_scale(coefficient, exponent);
                    let money = Money::from_decimal(amount, &currency);
                    assert_eq!(money.try_to_minor_units(), Err(MoneyError::Overflow));
                }
            }
            for amount in [Decimal::MAX, Decimal::MIN, dec!(100000000000000000)] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(money.try_to_minor_units(), Err(MoneyError::Overflow));
            }
        }

        #[test]
        fn precision_loss_takes_precedence_over_range_errors() {
            let amount = dec!(92233720368547758.081);
            for amount in [amount, -amount] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(money.try_to_minor_units(), Err(MoneyError::PrecisionLoss));
            }
        }

        #[test]
        fn handles_extreme_custom_exponents_without_panicking() {
            for exponent in [29, 38, 39, u32::MAX] {
                let currency = test::Currency {
                    exponent,
                    ..*test::USD
                };
                for amount in [Decimal::ONE, -Decimal::ONE, Decimal::MAX, Decimal::MIN] {
                    let money = Money::from_decimal(amount, &currency);
                    assert_eq!(money.try_to_minor_units(), Err(MoneyError::Overflow));
                }
                for amount in [Decimal::ZERO, dec!(0.0000)] {
                    let money = Money::from_decimal(amount, &currency);
                    assert_eq!(money.try_to_minor_units(), Ok(0));
                }
            }
            let currency = test::Currency {
                exponent: 29,
                ..*test::USD
            };
            let tiny = Money::from_decimal(Decimal::new(1, 28), &currency);
            assert_eq!(tiny.try_to_minor_units(), Ok(10));
        }

        proptest! {
            #[test]
            fn equivalent_decimal_representations_preserve_minor_units(
                minor in any::<i64>(),
                exponent in 0u32..=28,
                padding in 0u32..=9,
            ) {
                let currency = test::Currency { exponent, ..*test::USD };
                // Nine extra zeros fit even for i64 bounds; keep the scale within 28.
                let padding = padding.min(28 - exponent);
                let padded = Decimal::from_i128_with_scale(
                    i128::from(minor) * 10i128.pow(padding), exponent + padding,
                );
                // These encode the same known minor-unit count with different scales.
                // Normalization also exercises scaling up when minor ends in zeros.
                for amount in [padded, padded.normalize()] {
                    let money = Money::from_decimal(amount, &currency);
                    prop_assert_eq!(money.try_to_minor_units(), Ok(minor));
                }
            }
        }
    }

    mod comparison {
        use super::*;

        #[test]
        fn ordering_via_compare() {
            let small = Money::from_minor(100, test::USD);
            let large = Money::from_minor(200, test::USD);
            let equal = Money::from_minor(100, test::USD);

            // Greater Than
            assert!(large.gt(&small).unwrap());
            // Less Than
            assert!(small.lt(&large).unwrap());
            // Equals
            assert!(small.eq(&equal).unwrap());
        }

        #[test]
        fn equality() {
            // Same currency and amount
            assert!(Money::from_minor(100, test::USD) == Money::from_minor(100, test::USD));
            // Different amount
            assert!(Money::from_minor(100, test::USD) != Money::from_minor(200, test::USD));
            // Different currency (PartialEq returns false, doesn't panic)
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
        fn is_zero_with_negative_zero() {
            // Decimal can represent -0, ensure is_zero handles it
            let neg_zero = Money::from_decimal(-Decimal::ZERO, test::USD);
            assert!(neg_zero.is_zero());
            assert!(!neg_zero.is_negative());
            assert!(!neg_zero.is_positive());
        }

        #[test]
        fn abs_returns_absolute_value() {
            // Negative becomes positive
            let negative = Money::from_minor(-500, test::USD);
            assert_eq!(negative.abs(), Money::from_minor(500, test::USD));

            // Positive stays positive
            let positive = Money::from_minor(500, test::USD);
            assert_eq!(positive.abs(), Money::from_minor(500, test::USD));

            // Zero stays zero
            let zero = Money::from_minor(0, test::USD);
            assert_eq!(zero.abs(), Money::from_minor(0, test::USD));
        }

        #[test]
        fn to_minor_units_converts_correctly() {
            // USD (exponent 2): $123.45 = 12345 cents
            let usd = Money::from_minor(12345, test::USD);
            assert_eq!(usd.to_minor_units(), 12345);

            // From major units
            let usd_major = Money::from_major(100, test::USD);
            assert_eq!(usd_major.to_minor_units(), 10000);

            // JPY (exponent 0): ¥500 = 500 (no minor units)
            let jpy = Money::from_major(500, test::JPY);
            assert_eq!(jpy.to_minor_units(), 500);

            // BHD (exponent 3): 1.234 BHD = 1234 fils
            let bhd = Money::from_minor(1234, test::BHD);
            assert_eq!(bhd.to_minor_units(), 1234);

            // Negative amounts
            let negative = Money::from_minor(-500, test::USD);
            assert_eq!(negative.to_minor_units(), -500);

            // Zero
            let zero = Money::from_minor(0, test::USD);
            assert_eq!(zero.to_minor_units(), 0);
        }

        #[test]
        fn to_f64_lossy_converts_correctly() {
            // Basic conversion
            let money = Money::from_minor(12345, test::USD);
            assert!((money.to_f64_lossy() - 123.45).abs() < 0.0001);

            // Whole numbers
            let whole = Money::from_major(100, test::USD);
            assert!((whole.to_f64_lossy() - 100.0).abs() < 0.0001);

            // Negative amounts
            let negative = Money::from_minor(-5000, test::USD);
            assert!((negative.to_f64_lossy() - (-50.0)).abs() < 0.0001);

            // Zero
            let zero = Money::from_minor(0, test::USD);
            assert_eq!(zero.to_f64_lossy(), 0.0);

            // JPY (no decimals)
            let jpy = Money::from_major(1000, test::JPY);
            assert!((jpy.to_f64_lossy() - 1000.0).abs() < 0.0001);
        }
    }

    mod compare_tests {
        use super::*;
        use std::cmp::Ordering;

        #[test]
        fn compare_same_currency() {
            let a = Money::from_minor(100, test::USD);
            let b = Money::from_minor(200, test::USD);
            let c = Money::from_minor(100, test::USD);

            assert_eq!(a.compare(&b).unwrap(), Ordering::Less);
            assert_eq!(b.compare(&a).unwrap(), Ordering::Greater);
            assert_eq!(a.compare(&c).unwrap(), Ordering::Equal);
        }

        #[test]
        fn compare_different_currencies() {
            let a = Money::from_minor(100, test::USD);
            let b = Money::from_minor(100, test::GBP);
            let result = a.compare(&b);
            assert!(result.is_err());
            match result.unwrap_err() {
                MoneyError::CurrencyMismatch { expected, actual } => {
                    assert_eq!(expected, "USD");
                    assert_eq!(actual, "GBP");
                }
                _ => panic!("Expected CurrencyMismatch error"),
            }
        }

        #[test]
        fn comparison_helpers() {
            let small = Money::from_minor(100, test::USD);
            let large = Money::from_minor(200, test::USD);
            let equal = Money::from_minor(100, test::USD);

            // gt: greater than (strict)
            assert!(large.gt(&small).unwrap());
            assert!(!small.gt(&large).unwrap());
            assert!(!small.gt(&equal).unwrap()); // equal is not greater

            // gte: greater than or equal
            assert!(large.gte(&small).unwrap());
            assert!(!small.gte(&large).unwrap());
            assert!(small.gte(&equal).unwrap()); // equal satisfies >=

            // lt: less than (strict)
            assert!(small.lt(&large).unwrap());
            assert!(!large.lt(&small).unwrap());
            assert!(!small.lt(&equal).unwrap()); // equal is not less

            // lte: less than or equal
            assert!(small.lte(&large).unwrap());
            assert!(!large.lte(&small).unwrap());
            assert!(small.lte(&equal).unwrap()); // equal satisfies <=

            // eq: equality
            assert!(small.eq(&equal).unwrap());
            assert!(!small.eq(&large).unwrap());
        }

        #[test]
        fn helpers_return_error_on_currency_mismatch() {
            let usd = Money::from_minor(100, test::USD);
            let gbp = Money::from_minor(100, test::GBP);

            assert!(usd.gt(&gbp).is_err());
            assert!(usd.gte(&gbp).is_err());
            assert!(usd.lt(&gbp).is_err());
            assert!(usd.lte(&gbp).is_err());
            assert!(usd.eq(&gbp).is_err());
        }
    }

    mod allocation {
        use super::*;

        fn integer_total(parts: &[Money<'_, test::Currency>]) -> i128 {
            parts
                .iter()
                .map(|part| {
                    assert_eq!(part.amount().scale(), 0);
                    part.amount().mantissa()
                })
                .sum()
        }

        #[test]
        fn preserves_totals_near_decimal_limits() {
            for offset in 0..=10 {
                let magnitude = Decimal::MAX.mantissa() - offset;
                for total in [magnitude, -magnitude] {
                    let money =
                        Money::from_decimal(Decimal::from_i128_with_scale(total, 0), test::JPY);
                    for count in 1..=9 {
                        let parts = money.split(count).unwrap();
                        assert_eq!(integer_total(&parts), total);
                        for (i, part) in parts.iter().enumerate() {
                            let expected = total.div_euclid(i128::from(count))
                                + i128::from((i as i128) < total.rem_euclid(i128::from(count)));
                            assert_eq!(part.amount().mantissa(), expected);
                        }
                        let allocated = money.allocate(vec![1; count as usize]).unwrap();
                        assert_eq!(allocated, parts);
                    }
                }
            }
        }

        #[test]
        fn single_recipient_handles_large_scaled_totals() {
            for amount in [Decimal::MAX, Decimal::MIN] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(money.split(1), Ok(vec![money]));
                assert_eq!(money.allocate(vec![u32::MAX]), Ok(vec![money]));
                let parts = money.allocate(vec![0, u32::MAX, 0]).unwrap();
                assert!(parts[0].is_zero());
                assert_eq!(parts[1], money);
                assert!(parts[2].is_zero());
            }
        }

        #[test]
        fn unrepresentable_shares_return_overflow_instead_of_rounding() {
            for amount in [Decimal::MAX, Decimal::MIN] {
                let money = Money::from_decimal(amount, test::USD);
                assert_eq!(money.split(2), Err(MoneyError::Overflow));
                assert_eq!(money.allocate(vec![1, 1]), Err(MoneyError::Overflow));
            }
        }

        #[test]
        fn excessive_scaling_returns_overflow() {
            for exponent in [28, 29, u32::MAX] {
                let currency = test::Currency {
                    exponent,
                    ..*test::USD
                };
                let money = Money::from_decimal(Decimal::MAX, &currency);
                assert_eq!(money.split(1), Err(MoneyError::Overflow));
                assert_eq!(money.allocate(vec![1]), Err(MoneyError::Overflow));
                assert_eq!(money.split(0), Err(MoneyError::InvalidRatio));
                assert_eq!(money.allocate(vec![0]), Err(MoneyError::InvalidRatio));
            }
        }

        #[test]
        fn flooring_and_zero_weight_policy_is_unchanged() {
            use rust_decimal_macros::dec;
            for (amount, expected) in [(dec!(1.005), [50, 50]), (dec!(-1.005), [-50, -51])] {
                let money = Money::from_decimal(amount, test::USD);
                let expected: Vec<_> = expected
                    .into_iter()
                    .map(|minor| Money::from_minor(minor, test::USD))
                    .collect();
                assert_eq!(money.split(2).unwrap(), expected);
                let allocated = money.allocate(vec![0, 1, 1]).unwrap();
                assert!(allocated[0].is_zero());
                assert_eq!(&allocated[1..], expected.as_slice());
            }
        }

        #[test]
        fn tiny_high_exponent_amounts_allocate_exactly() {
            for exponent in [19, 20, 28] {
                let currency = test::Currency {
                    exponent,
                    ..*test::USD
                };
                for minor in [-5, 5] {
                    let money = Money::from_minor(minor, &currency);
                    let split = money.split(2).unwrap();
                    assert_eq!(
                        split
                            .iter()
                            .map(|m| m.try_to_minor_units().unwrap())
                            .sum::<i64>(),
                        minor
                    );
                    assert_eq!(money.allocate(vec![1, 1]).unwrap(), split);
                }
            }
        }

        proptest::proptest! {
            #[test]
            fn whole_unit_allocations_match_integer_oracle(
                lo in proptest::prelude::any::<u32>(),
                mid in proptest::prelude::any::<u32>(),
                hi in proptest::prelude::any::<u32>(),
                negative in proptest::prelude::any::<bool>(),
                weights in proptest::collection::vec(
                    proptest::prop_oneof![
                        proptest::strategy::Just(0u32),
                        proptest::strategy::Just(1u32),
                        proptest::strategy::Just(u32::MAX),
                        proptest::prelude::any::<u32>(),
                    ],
                    1..9,
                ),
            ) {
                let amount = Decimal::from_parts(lo, mid, hi, negative, 0);
                // This property covers whole units. Fixed cases above cover currency
                // scaling, fractional inputs, and unrepresentable output shares.
                let total = amount.mantissa();
                let sum: u128 = weights.iter().map(|&w| u128::from(w)).sum();
                proptest::prop_assume!(sum > 0);
                // Independent oracle: the 96-bit magnitude times a u32 weight fits u128.
                let mut expected: Vec<i128> = weights.iter().map(|&w| {
                    let numerator = total.unsigned_abs() * u128::from(w);
                    let quotient = numerator / sum;
                    if total < 0 { -(numerator.div_ceil(sum) as i128) }
                    else { quotient as i128 }
                }).collect();
                let mut remainder = total - expected.iter().sum::<i128>();
                for (value, weight) in expected.iter_mut().zip(&weights) {
                    if remainder > 0 && *weight > 0 { *value += 1; remainder -= 1; }
                }
                let money = Money::from_decimal(amount, test::JPY);
                let actual = money.allocate(weights).unwrap();
                proptest::prop_assert_eq!(integer_total(&actual), total);
                let actual: Vec<_> = actual.iter().map(|m| m.amount().mantissa()).collect();
                proptest::prop_assert_eq!(actual, expected);
            }
        }

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

            // Using split (convenience wrapper)
            assert_eq!(expected, money.split(3).unwrap());
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
        fn allocate_remainder_skips_zero_weight_recipients() {
            let cases = [
                (1, vec![0, 1, 1], vec![0, 1, 0]),
                (2, vec![1, 0, 1, 0, 1, 0], vec![1, 0, 1, 0, 0, 0]),
                (-1, vec![0, 1, 1], vec![0, 0, -1]),
                (-2, vec![0, 1, 0, 1, 0, 1, 0], vec![0, 0, 0, -1, 0, -1, 0]),
            ];

            for (amount, shares, expected) in cases {
                let money = Money::from_minor(amount, test::USD);
                let allocated = money.allocate(shares).unwrap();
                let expected: Vec<_> = expected
                    .into_iter()
                    .map(|minor| Money::from_minor(minor, test::USD))
                    .collect();
                assert_eq!(allocated, expected);
                assert_eq!(
                    allocated.iter().map(|m| *m.amount()).sum::<Decimal>(),
                    *money.amount()
                );
            }
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

            // Error if split is called with zero
            let monies = Money::from_minor(100, test::USD).split(0);
            assert_eq!(monies.unwrap_err(), MoneyError::InvalidRatio);
        }

        // ============ Sum Invariant Tests ============
        // These tests verify the critical invariant: money is never created or destroyed

        #[test]
        fn split_sum_equals_original() {
            // Test various amounts and split counts to ensure sum always equals original
            let amounts = [1, 2, 3, 7, 10, 11, 100, 101, 999, 1000, 10001, 123456];
            let splits = [2, 3, 5, 7, 11, 13];

            for &amount in &amounts {
                for &n in &splits {
                    let money = Money::from_minor(amount, test::USD);
                    let parts = money.split(n).unwrap();

                    // Sum all parts
                    let sum = parts
                        .iter()
                        .fold(Money::from_minor(0, test::USD), |acc, &m| {
                            acc.add(m).unwrap()
                        });

                    assert_eq!(
                        sum.to_minor_units(),
                        money.to_minor_units(),
                        "split({}) of {} minor units: sum {} != original {}",
                        n,
                        amount,
                        sum.to_minor_units(),
                        money.to_minor_units()
                    );
                }
            }
        }

        #[test]
        fn allocate_sum_equals_original() {
            // Test weighted allocation preserves total
            let test_cases: Vec<(i64, Vec<u32>)> = vec![
                (10000, vec![70, 20, 10]),      // Standard percentage split
                (1100, vec![1, 1, 1]),          // Equal thirds with remainder
                (100, vec![1, 2, 3, 4]),        // Unequal weights
                (7, vec![1, 1, 1, 1, 1, 1, 1]), // More shares than cents
                (1000000, vec![33, 33, 34]),    // Large amount
                (1, vec![1, 1, 1]),             // Single cent split 3 ways
            ];

            for (amount, shares) in test_cases {
                let money = Money::from_minor(amount, test::USD);
                let parts = money.allocate(shares.clone()).unwrap();

                let sum = parts
                    .iter()
                    .fold(Money::from_minor(0, test::USD), |acc, m| {
                        acc.add(*m).unwrap()
                    });

                assert_eq!(
                    sum.to_minor_units(),
                    money.to_minor_units(),
                    "allocate({:?}) of {} minor units: sum {} != original {}",
                    shares,
                    amount,
                    sum.to_minor_units(),
                    money.to_minor_units()
                );
            }
        }

        #[test]
        fn split_sum_equals_original_negative() {
            // Negative amounts must also preserve sum
            let money = Money::from_minor(-1100, test::USD);
            let parts = money.split(3).unwrap();

            let sum = parts
                .iter()
                .fold(Money::from_minor(0, test::USD), |acc, &m| {
                    acc.add(m).unwrap()
                });

            assert_eq!(sum.to_minor_units(), money.to_minor_units());
        }

        #[test]
        fn split_sum_equals_original_different_exponents() {
            // Test with currencies of different exponents
            let currencies_and_amounts: Vec<(&test::Currency, i64)> = vec![
                (test::USD, 12345), // exponent 2
                (test::JPY, 12345), // exponent 0
                (test::BHD, 12345), // exponent 3
            ];

            for (currency, amount) in currencies_and_amounts {
                let money = Money::from_minor(amount, currency);
                let parts = money.split(7).unwrap();

                let sum = parts
                    .iter()
                    .fold(Money::from_minor(0, currency), |acc, &m| {
                        acc.add(m).unwrap()
                    });

                assert_eq!(
                    sum.to_minor_units(),
                    money.to_minor_units(),
                    "Currency {} with exponent {}: sum mismatch",
                    currency.code(),
                    currency.exponent()
                );
            }
        }
    }

    mod rounding {
        use super::*;

        #[test]
        fn precision_and_rounding() {
            // Dividing 20 by 3 rounds to 6.67 in USD and 6.667 in BHD
            let expected_money = Money::from_minor(667, test::USD);
            let money = Money::from_minor(2_000, test::USD).div(3).unwrap();
            assert_eq!(money.round(2, Round::HalfEven), expected_money);

            let expected_money = Money::from_minor(6_667, test::BHD);
            let money = Money::from_minor(20_000, test::BHD).div(3).unwrap();
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
                prop_assert_eq!(money_a.add(money_b).unwrap(), money_b.add(money_a).unwrap());
            }

            #[test]
            fn addition_is_associative(a in minor_amount(), b in minor_amount(), c in minor_amount()) {
                let money_a = Money::from_minor(a, test::USD);
                let money_b = Money::from_minor(b, test::USD);
                let money_c = Money::from_minor(c, test::USD);
                let left = money_a.add(money_b).unwrap().add(money_c).unwrap();
                let right = money_a.add(money_b.add(money_c).unwrap()).unwrap();
                prop_assert_eq!(left, right);
            }

            #[test]
            fn zero_is_additive_identity(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                let zero = Money::from_minor(0, test::USD);
                prop_assert_eq!(money.add(zero).unwrap(), money);
                prop_assert_eq!(zero.add(money).unwrap(), money);
            }

            #[test]
            fn subtraction_is_inverse_of_addition(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                let zero = Money::from_minor(0, test::USD);
                prop_assert_eq!(money.sub(money).unwrap(), zero);
            }

            #[test]
            fn negation_is_self_inverse(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                prop_assert_eq!(-(-money), money);
            }

            #[test]
            fn multiplication_by_one_is_identity(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                prop_assert_eq!(money.mul(1i64).unwrap(), money);
            }

            #[test]
            #[allow(clippy::erasing_op)]
            fn multiplication_by_zero_gives_zero(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                let zero = Money::from_minor(0, test::USD);
                prop_assert_eq!(money.mul(0i64).unwrap(), zero);
            }

            #[test]
            fn division_by_one_is_identity(a in minor_amount()) {
                let money = Money::from_minor(a, test::USD);
                prop_assert_eq!(money.div(1i64).unwrap(), money);
            }

            #[test]
            fn multiplication_then_division_roundtrip(a in minor_amount(), n in non_zero_multiplier()) {
                let money = Money::from_minor(a, test::USD);
                let multiplied = money.mul(n).unwrap();
                let divided = multiplied.div(n).unwrap();
                // May not be exactly equal due to decimal precision, but should be very close
                let diff = (*money.amount() - *divided.amount()).abs();
                prop_assert!(diff < Decimal::new(1, 10), "difference too large: {}", diff);
            }

            #[test]
            fn distributive_property(a in minor_amount(), n in -1000i64..1000, m in -1000i64..1000) {
                let money = Money::from_minor(a, test::USD);
                let left = money.mul(n + m).unwrap();
                let right = money.mul(n).unwrap().add(money.mul(m).unwrap()).unwrap();
                prop_assert_eq!(left, right);
            }
        }
    }

    mod allocation_properties {
        use super::*;

        proptest! {
            #[test]
            fn inserting_zero_weight_preserves_allocations(
                amount in minor_amount(),
                shares in valid_shares(),
                index in 0usize..10,
            ) {
                let money = Money::from_minor(amount, test::USD);
                let index = index % (shares.len() + 1);
                let mut expected = money.allocate(shares.clone()).unwrap();
                expected.insert(index, Money::from_minor(0, test::USD));

                let mut with_zero = shares;
                with_zero.insert(index, 0);
                prop_assert_eq!(money.allocate(with_zero).unwrap(), expected);
            }

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
            fn split_is_consistent_with_allocate(amount in minor_amount(), n in 1u32..20) {
                let money = Money::from_minor(amount, test::USD);
                let shares: Vec<u32> = vec![1; n as usize];

                let via_allocate = money.allocate(shares).unwrap();
                let via_split = money.split(n).unwrap();

                prop_assert_eq!(via_allocate, via_split);
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
