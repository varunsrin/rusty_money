use crate::currency::*;
use crate::{Money, MoneyError};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// A struct to store `ExchangeRate`s.
#[derive(Debug, Default)]
pub struct Exchange<T: CurrencyType>
where
    T: 'static,
{
    map: HashMap<String, ExchangeRate<T>>,
}

impl<T: CurrencyType> Exchange<T> {
    pub fn new() -> Exchange<T> {
        Exchange {
            map: HashMap::new(),
        }
    }

    /// Update an ExchangeRate or add it if does not exist.
    pub fn add_or_update_rate(&mut self, rate: &ExchangeRate<T>) {
        let key = Exchange::generate_key(rate.from, rate.to);
        self.map.insert(key, *rate);
    }

    /// Return the ExchangeRate given the currency pair.
    pub fn get_rate(&self, from: &'static T, to: &'static T) -> Option<ExchangeRate<T>> {
        let key = Exchange::generate_key(from, to);
        match self.map.get(&key) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    fn generate_key(from: &'static T, to: &'static T) -> String {
        from.to_string() + "-" + &to.to_string()
    }
}

/// A struct to store rates of conversion between two currencies.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ExchangeRate<T: CurrencyType>
where
    T: 'static,
{
    pub from: &'static T,
    pub to: &'static T,
    rate: Decimal,
}

impl<T: CurrencyType> ExchangeRate<T> {
    pub fn new(
        from: &'static T,
        to: &'static T,
        rate: Decimal,
    ) -> Result<ExchangeRate<T>, MoneyError> {
        if from == to {
            return Err(MoneyError::InvalidCurrency);
        }
        Ok(ExchangeRate { from, to, rate })
    }

    /// Converts a Money from one Currency to another using the exchange rate.
    pub fn convert(&self, amount: Money<T>) -> Result<Money<T>, MoneyError> {
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
        let usd = IsoCurrency::get(USD);
        let eur = IsoCurrency::get(EUR);
        let gbp = IsoCurrency::get(GBP);

        let eur_usd_rate = ExchangeRate::new(usd, eur, dec!(1.5)).unwrap();
        let eur_gbp_rate = ExchangeRate::new(usd, gbp, dec!(1.6)).unwrap();

        let mut exchange = Exchange::new();
        exchange.add_or_update_rate(&eur_usd_rate);
        exchange.add_or_update_rate(&eur_gbp_rate);

        let fetched_rate = exchange.get_rate(usd, eur).unwrap();
        assert_eq!(fetched_rate.rate, dec!(1.5));

        let fetched_rate = exchange.get_rate(usd, gbp).unwrap();
        assert_eq!(fetched_rate.rate, dec!(1.6));
    }

    #[test]
    fn rate_convert() {
        let rate =
            ExchangeRate::new(IsoCurrency::get(USD), IsoCurrency::get(EUR), dec!(1.5)).unwrap();
        let amount = money!(10, "USD");
        let expected_amount = money!("15", "EUR");
        let converted_rate = rate.convert(amount).unwrap();
        assert_eq!(converted_rate, expected_amount);
    }

    #[test]
    fn rate_convert_errors_if_currencies_dont_match() {
        let rate =
            ExchangeRate::new(IsoCurrency::get(GBP), IsoCurrency::get(EUR), dec!(1.5)).unwrap();
        let amount = money!(10, "USD");

        assert_eq!(
            rate.convert(amount).unwrap_err(),
            MoneyError::InvalidCurrency,
        );
    }

    #[test]
    fn rate_new_errors_if_currencies_are_equal() {
        let rate = ExchangeRate::new(
            IsoCurrency::get(GBP),
            IsoCurrency::get(GBP),
            dec!(1.5),
        );
        assert_eq!(rate.unwrap_err(), MoneyError::InvalidCurrency,);
    }
}
