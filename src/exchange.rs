use crate::currency::*;
use crate::{Money, MoneyError};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// A struct to store `ExchangeRate`s.
#[derive(Debug, Default)]
pub struct Exchange {
    map: HashMap<String, ExchangeRate>,
}

impl Exchange {
    pub fn new() -> Exchange {
        Exchange {
            map: HashMap::new(),
        }
    }

    /// Update an ExchangeRate or add it if does not exist.
    pub fn add_or_update_rate(&mut self, rate: &ExchangeRate) {
        let key = Exchange::generate_key(rate.from, rate.to);
        self.map.insert(key, *rate);
    }

    /// Return the ExchangeRate given the currency pair.
    pub fn get_rate(self, from: &'static Currency, to: &'static Currency) -> Option<ExchangeRate> {
        let key = Exchange::generate_key(from, to);
        match self.map.get(&key) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    fn generate_key(from: &'static Currency, to: &'static Currency) -> String {
        from.to_string() + "-" + &to.to_string()
    }
}

/// A struct to store rates of conversion between two currencies.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ExchangeRate {
    pub from: &'static Currency,
    pub to: &'static Currency,
    rate: Decimal,
}

impl ExchangeRate {
    pub fn new(
        from: &'static Currency,
        to: &'static Currency,
        rate: Decimal,
    ) -> Result<ExchangeRate, MoneyError> {
        if from == to {
            return Err(MoneyError::InvalidCurrency);
        }
        Ok(ExchangeRate { from, to, rate })
    }

    /// Converts a Money from one Currency to another using the exchange rate.
    pub fn convert(&self, amount: Money) -> Result<Money, MoneyError> {
        if amount.currency() != self.from {
            return Err(MoneyError::InvalidCurrency);
        }
        let converted_amount = amount.amount() * self.rate;
        Ok(Money::from_decimal(converted_amount, self.to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money;
    use crate::Iso::*;
    use rust_decimal_macros::*;

    #[test]
    fn exchange_stores_rates() {
        let usd = Currency::get(USD);
        let eur = Currency::get(EUR);
        let rate = ExchangeRate::new(usd, eur, dec!(1.5)).unwrap();

        let mut exchange = Exchange::new();
        exchange.add_or_update_rate(&rate);
        let fetched_rate = exchange.get_rate(usd, eur).unwrap();
        assert_eq!(fetched_rate.rate, dec!(1.5));
    }

    #[test]
    fn rate_convert() {
        let rate = ExchangeRate::new(Currency::get(USD), Currency::get(EUR), dec!(1.5)).unwrap();
        let amount = money!(10, "USD");
        let expected_amount = money!("15", "EUR");
        let converted_rate = rate.convert(amount).unwrap();
        assert_eq!(converted_rate, expected_amount);
    }

    #[test]
    fn rate_convert_errors_if_currencies_dont_match() {
        let rate =
            ExchangeRate::new(Currency::get(Iso::GBP), Currency::get(Iso::EUR), dec!(1.5)).unwrap();
        let amount = money!(10, "USD");

        assert_eq!(
            rate.convert(amount).unwrap_err(),
            MoneyError::InvalidCurrency,
        );
    }

    #[test]
    fn rate_new_errors_if_currencies_are_equal() {
        let rate = ExchangeRate::new(Currency::get(Iso::GBP), Currency::get(Iso::GBP), dec!(1.5));
        assert_eq!(rate.unwrap_err(), MoneyError::InvalidCurrency,);
    }
}
