//! A library that handles calculating, rounding, displaying, and parsing units of money according
//! to ISO 4217 standards. The main items exported by the library iare `Money` and `Currency`.
//!
//! # Usage
//!
//! `Money` consists of an amount, which is represented by a Decimal type that it owns and a
//! `Currency`, which is holds a reference to. `Currency` represents an ISO-4217 currency, and
//!  stores metadata like its numeric code, full name and symbol.
//!
//! ```edition2018
//! // Money can be initialized in a few ways:
//! use rusty_money::{money, Money, Currency};
//! use rusty_money::Iso::*;
//!
//! money!(2000, "USD")                             // 2000 USD
//! money!("2000.00", "USD")                        // 2000 USD
//! Money::new(200000, Currency::get(USD));         // 2000 USD
//! Money::from_major(2000, Currency::get(USD));    // 2000 USD
//! Money::from_minor(200000, Currency::get(USD));  // 2000 USD
//! Money::from_str("2,000.00", "USD").unwrap();    // 2000 USD
//!
//!
//! // Money objects with the same Currency can be compared:
//! let hundred = money!(100, "USD");
//! let thousand = money!(1000, "USD");
//! println!("{}", thousand > hundred);     // false
//! println!("{}", thousand.is_positive()); // true
//! ```
//!
//! ## Precision and Rounding
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
//! use rusty_money::{money, Money, Currency};
//!
//! // Money can be added, subtracted, multiplied and divided:
//! money!(100, "USD") + money!(100, "USD");        // 200 USD
//! money!(100, "USD") - money!(100, "USD");        // 0 USD
//! money!(1, "USD") * 3;                           // 3 USD
//! money!(3, "USD") / 3;                           // 0.333333333... USD
//!```
//!
//! ## Formatting
//!
//! Calling `format!` or `println!` on Money returns a string with a rounded amount, using separators and symbols
//! according to the locale of the currency. If you need to customize this output, the `Formatter` module
//! accepts a more detailed set of parameters.
//!
//! ```edition2018
//! // Money objects can be pretty printed, with appropriate rounding and formatting:
//! let usd = money!("-2000.009", "USD");
//! let eur = money!("-2000.009", "EUR");
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
//! use rusty_money::{money, Exchange, ExchangeRate};
//! use rust_decimal_macros::*;
//!
//! // Convert 1000 USD to EUR at a 2:1 exchange rate.
//! let rate = ExchangeRate::new(Currency::get(USD), Currency::get(EUR), dec!(0.5)).unwrap();
//! rate.convert(money!(1000, "USD")); // 500 EUR
//!
//! // An Exchange can be used to store ExchangeRates for later use
//! let mut exchange = Exchange::new();
//! exchange.add_or_update_rate(&rate);
//! exchange.get_rate(Currency::get(USD), Currency::get(EUR));
//! ```
//!

mod currency;
mod error;
mod exchange;
mod format;
mod locale;
mod money;

pub use currency::*;
pub use error::MoneyError;
pub use exchange::*;
pub use format::*;
pub use locale::*;
pub use money::*;

#[macro_use]
extern crate lazy_static;
