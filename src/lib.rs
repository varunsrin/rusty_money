//! A library that handles calculating, rounding, displaying, and parsing units of money according
//! to ISO 4217 standards. The main items exported by the library are `Money` and `Currency`.
//!
//! # Usage
//!
//! `Money` consists of an amount and a currency. An amount is a Decimal type that is owned by Money, while a currency
//! is a reference to anything that implements `FormattableCurrency`. rusty_money provides two currency sets:
//! `IsoCurrency`, which implements ISO-4217 currencies and `CryptoCurrency` which implements popular cryptocurencies.  
//!
//! Money objects can be created from `IsoCurrencies` in a few different ways:
//!
//! ```edition2018
//! use rusty_money::{Money, IsoCurrency, Iso};
//!   
//! let usd_currency = IsoCurrency::get(Iso::USD);
//! Money::from_major(2_000, usd_currency);                             // 2000 USD
//! Money::from_minor(200_000, usd_currency);                           // 2000 USD
//! Money::from_string("2,000.00".to_string(), usd_currency).unwrap();  // 2000 USD
//! ```
//!
//! Money objects can be created from `CryptoCurrencies` in similar ways:
//!
//! ```edition2018
//! use rusty_money::{Money, CryptoCurrency, crypto};
//!
//! Money::from_major(2, crypto::BTC);             // 2 BTC
//! Money::from_minor(200_000_000, crypto::BTC);   // 2 BTC
//! ```
//!
//! Money objects with the same Currency can be compared:
//!  ```edition2018
//! use rusty_money::{Money, Iso, IsoCurrency, iso_money};
//!
//! let hundred = iso_money!(10_000, Iso::USD);
//! let thousand = iso_money!(100_000, Iso::USD);
//! println!("{}", thousand > hundred);     // false
//! println!("{}", thousand.is_positive()); // true
//! ```
//!
//! ## Precision, Rounding and Math
//!
//! Money objects are immutable, and operations that change the amount or currency of Money simply create
//! a new instance. Money uses a 128 bit fixed-precision [Decimal](https://github.com/paupino/rust-decimal)
//! to represents amounts, and it represents values as large as 2<sup>96</sup> / 10<sup>28</sup>. By default
//! operations on Money always retain maximum possible precision. When you do need to round money, you can call
//!  the `round` function, which  supports three modes:
//! * [Half Up](https://en.wikipedia.org/wiki/Rounding#Round_half_up)
//! * [Half Down](https://en.wikipedia.org/wiki/Rounding#Round_half_down)
//! * [Half Even](https://en.wikipedia.org/wiki/Rounding#Round_half_even) (default)
//!
//! ```edition2018
//! use rusty_money::{Money, IsoCurrency, Round, Iso, iso_money};
//!
//! // Money can be added, subtracted, multiplied and divided:
//! iso_money!(10_000, Iso::USD) + iso_money!(10_000, Iso::USD); // 200 USD
//! iso_money!(10_000, Iso::USD) - iso_money!(10_000, Iso::USD); // 0 USD
//! iso_money!(100, Iso::USD) * 3;                                             // 3 USD
//! iso_money!(100, Iso::USD) / 3;                                             // 0.333333333... USD
//!
//! // Money can be rounded by calling the round function:
//! let usd = Money::from_string("-2000.005".to_string(), IsoCurrency::get(Iso::USD)).unwrap();  // 2000.005 USD
//! usd.round(2, Round::HalfEven);                                 // 2000.00 USD
//! usd.round(2, Round::HalfUp);                                   // 2000.01 USD
//! usd.round(0, Round::HalfUp);                                   // 2000 USD
//!```
//!
//! ## Formatting
//!
//! Calling `format!` or `println!` on Money returns a string with a rounded amount, using separators and symbols
//! according to the locale of the currency. If you need to customize this output, the `Formatter` module
//! accepts a more detailed set of parameters.
//!
//! ```edition2018
//! use rusty_money::{Money, IsoCurrency, Iso};
//!
//! // Money objects can be pretty printed, with appropriate rounding and formatting:
//! let usd = Money::from_string("-2000.009".to_string(), IsoCurrency::get(Iso::USD)).unwrap();
//! let eur = Money::from_string("-2000.009".to_string(), IsoCurrency::get(Iso::EUR)).unwrap();
//! println!("{}", usd); // -$2,000.01
//! println!("{}", eur); // -€2.000,01;
//! ```
//!
//! ## Exchange
//!
//! The library also provides two additional types - `Exchange` and `ExchangeRates` to convert Money from one currency
//! to another.
//!
//! ```edition2018
//! use rusty_money::{Money, IsoCurrency, Exchange, ExchangeRate, iso_money, Iso};
//! use rusty_money::Iso::*;
//! use rust_decimal_macros::*;
//!
//! // Convert 1000 USD to EUR at a 2:1 exchange rate.
//! let rate = ExchangeRate::new(IsoCurrency::get(USD), IsoCurrency::get(EUR), dec!(0.5)).unwrap();
//! rate.convert(iso_money!(100_000, Iso::USD));                                     // 500 EUR
//!
//! // An Exchange can be used to store ExchangeRates for later use
//! let mut exchange = Exchange::new();
//! exchange.set_rate(&rate);
//! exchange.get_rate(IsoCurrency::get(USD), IsoCurrency::get(EUR));
//! ```
//!

mod currency;
mod error;
mod exchange;
mod format;
mod iso_currency;
mod locale;
mod money;

pub use currency::*;
pub use error::MoneyError;
pub use exchange::*;
pub use format::*;
pub use iso_currency::*;
pub use locale::*;
pub use money::*;

#[macro_use]
extern crate lazy_static;
