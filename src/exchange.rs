use crate::currency::Currency;
use crate::money::Money;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// An Exchange Type which stores a collection of exchange rates pairs between currencies.
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
    pub fn get_rate(self, from: Currency, to: Currency) -> ExchangeRate {
        let key = Exchange::generate_key(from, to);
        match self.map.get(&key) {
            Some(v) => *v,
            // TODO - Add Error Type
            None => panic!(0),
        }
    }

    fn generate_key(from: Currency, to: Currency) -> String {
        from.to_string() + "-" + &to.to_string()
    }
}

/// An ExchangeRate Type which stores a rate of conversion between two currencies.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ExchangeRate {
    pub from: Currency,
    pub to: Currency,
    rate: Decimal,
}

impl ExchangeRate {
    pub fn new(from: Currency, to: Currency, rate: Decimal) -> ExchangeRate {
        ExchangeRate { from, to, rate }
    }

    /// Converts a Money from one Currency to another using the exchange rate.
    pub fn convert(&self, amount: Money) -> Money {
        if amount.currency() != self.from {
            panic!();
        }
        let converted_amount = amount.amount() * self.rate;
        Money::from_decimal(converted_amount, self.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::money;
    use rust_decimal_macros::*;

    #[test]
    fn exchange_stores_rates() {
        let usd = Currency::find("USD");
        let eur = Currency::find("EUR");
        let rate = ExchangeRate::new(usd, eur, dec!(1.5));

        let mut exchange = Exchange::new();
        exchange.add_or_update_rate(&rate);
        let fetched_rate = exchange.get_rate(usd, eur);
        assert_eq!(fetched_rate.rate, dec!(1.5));
    }

    #[test]
    fn rate_converts_money() {
        let rate = ExchangeRate::new(Currency::find("USD"), Currency::find("EUR"), dec!(1.5));
        let amount = money!(10, "USD");
        let expected_amount = money!("15", "EUR");
        assert_eq!(rate.convert(amount), expected_amount);
    }

    #[test]
    #[should_panic]
    fn rate_fails_if_currencies_dont_match() {
        let rate = ExchangeRate::new(Currency::find("GBP"), Currency::find("EUR"), dec!(1.5));
        let amount = money!(10, "USD");
        rate.convert(amount);
    }
}
