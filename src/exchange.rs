use crate::currency::FormattableCurrency;
use crate::{Money, MoneyError};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Stores `ExchangeRate`s for easier access.
#[derive(Debug, Default)]
pub struct Exchange<'a, T: FormattableCurrency> {
    map: HashMap<String, ExchangeRate<'a, T>>,
}

impl<'a, T: FormattableCurrency> Exchange<'a, T> {
    pub fn new() -> Exchange<'a, T> {
        Exchange {
            map: HashMap::new(),
        }
    }

    /// Update an ExchangeRate or add it if does not exist.
    pub fn set_rate(&mut self, rate: &'a ExchangeRate<T>) {
        let key = Exchange::generate_key(rate.from, rate.to);
        self.map.insert(key, *rate);
    }

    /// Return the ExchangeRate given the currency pair.
    pub fn get_rate(&self, from: &T, to: &T) -> Option<ExchangeRate<T>> {
        let key = Exchange::generate_key(from, to);
        match self.map.get(&key) {
            Some(v) => Some(*v),
            None => None,
        }
    }

    fn generate_key(from: &T, to: &T) -> String {
        from.to_string() + "-" + &to.to_string()
    }
}

/// Stores rates of conversion between two currencies.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ExchangeRate<'a, T: FormattableCurrency> {
    pub from: &'a T,
    pub to: &'a T,
    rate: Decimal,
}

impl<'a, T: FormattableCurrency> ExchangeRate<'a, T> {
    pub fn new(from: &'a T, to: &'a T, rate: Decimal) -> Result<ExchangeRate<'a, T>, MoneyError> {
        if from == to {
            return Err(MoneyError::InvalidCurrency);
        }
        Ok(ExchangeRate { from, to, rate })
    }

    /// Converts a Money from one Currency to another using the exchange rate.
    pub fn convert(&self, amount: Money<'a, T>) -> Result<Money<'a, T>, MoneyError> {
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
    use rust_decimal_macros::*;
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
            GBP : {
                code: "GBP",
                exponent: 2,
                locale: EnUs,
                minor_units: 1,
                name: "British Pound",
                symbol: "£",
                symbol_first: true,
            },
            EUR : {
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
    fn exchange_stores_rates() {
        let usd = test::find("USD").unwrap();
        let eur = test::find("EUR").unwrap();
        let gbp = test::find("GBP").unwrap();

        let eur_usd_rate = ExchangeRate::new(usd, eur, dec!(1.5)).unwrap();
        let eur_gbp_rate = ExchangeRate::new(usd, gbp, dec!(1.6)).unwrap();

        let mut exchange = Exchange::new();
        exchange.set_rate(&eur_usd_rate);
        exchange.set_rate(&eur_gbp_rate);

        let fetched_rate = exchange.get_rate(usd, eur).unwrap();
        assert_eq!(fetched_rate.rate, dec!(1.5));

        let fetched_rate = exchange.get_rate(usd, gbp).unwrap();
        assert_eq!(fetched_rate.rate, dec!(1.6));
    }

    #[test]
    fn rate_convert() {
        let rate = ExchangeRate::new(test::USD, test::EUR, dec!(1.5)).unwrap();
        let amount = Money::from_minor(1_000, test::USD);
        let expected_amount = Money::from_minor(1_500, test::EUR);
        let converted_rate = rate.convert(amount).unwrap();
        assert_eq!(converted_rate, expected_amount);
    }

    #[test]
    fn rate_convert_errors_if_currencies_dont_match() {
        let rate = ExchangeRate::new(test::GBP, test::EUR, dec!(1.5)).unwrap();
        let amount = Money::from_minor(1_000, test::USD);

        assert_eq!(
            rate.convert(amount).unwrap_err(),
            MoneyError::InvalidCurrency,
        );
    }

    #[test]
    fn rate_new_errors_if_currencies_are_equal() {
        let rate = ExchangeRate::new(test::GBP, test::GBP, dec!(1.5));
        assert_eq!(rate.unwrap_err(), MoneyError::InvalidCurrency,);
    }
}
