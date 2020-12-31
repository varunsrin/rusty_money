//! A library that handles calculating, rounding, displaying, and parsing units of money according
//! to ISO 4217 standards. The main items exported by the library are `Money` and `Currency`.
//!
//! # Usage
//!
//! `Money` consists of an amount and a currency. An amount is a Decimal type that is owned by Money, while a currency
//! is a reference to a struct that implements `FormattableCurrency`. 
//! 
//! You can create `Currencies` with the `define_currency_set!` macro and then create `Money` objects with them:  
//!
//! ```edition2018
//! use rusty_money::{Money, define_currency_set};
//!
//! define_currency_set!(
//!   video_game {
//!     GIL: {
//!       code: "GIL",
//!       exponent: 2,
//!       locale: Locale::EnUs,
//!       minor_units: 100,
//!       name: "GIL",
//!       symbol: "G",
//!       symbol_first: true,
//!     }
//!   }
//! );
//! 
//! Money::from_major(2_000, video_game::GIL);   // 2000 GIL
//!  
//! let gil = video_game::find("GIL").unwrap();                        
//! Money::from_major(2_000, gil);               // 2000 GIL
//! ```
//! 
//! ## Features: Currency Sets
//! rusty_money provides two currency sets for convenience : `IsoCurrency`, which implements ISO-4217 currencies  
//! and `CryptoCurrency` which implements popular cryptocurencies. This can be enabled in Cargo.toml:
//! 
//! ```toml
//! [dependencies]
//! rusty_money = { version = "0.4.0", features = ["iso", "crypto"] }
//! ```
//! And then you can use the currencies like this: 
//! 
//! ```edition2018
//! use rusty_money::{Money, iso, crypto};
//!   
//! Money::from_major(2_000, iso::USD);              // 2000 USD
//! Money::from_minor(200_000, iso::USD);            // 2000 USD
//! Money::from_str("2,000.00", iso::USD).unwrap();  // 2000 USD
//! 
//! Money::from_major(2, crypto::BTC);            // 2 BTC
//! Money::from_minor(200_000_000, crypto::BTC);  // 2 BTC
//! ```
//!
//! Money objects with the same Currency can be compared:
//!
//!  ```edition2018
//! use rusty_money::{Money, iso};
//!
//! let hundred = Money::from_minor(10_000, iso::USD);
//! let thousand = Money::from_minor(100_000, iso::USD);
//! println!("{}", thousand > hundred);     // false
//! println!("{}", thousand.is_positive()); // true
//! ```
//!
//! ## Precision, Rounding and Math
//!
//! Money objects are immutable, and operations that change the amount or currency of Money create a 
//! a new instance. Money uses a 128 bit fixed-precision [Decimal](https://github.com/paupino/rust-decimal)
//! to represents amounts, and it represents values as large as 2<sup>96</sup> / 10<sup>28</sup>. By default
//! operations on Money always retain maximum possible precision. When you do need to round money, you can call
//!  the `round` function, which  supports three modes:
//!
//! * [Half Up](https://en.wikipedia.org/wiki/Rounding#Round_half_up)
//! * [Half Down](https://en.wikipedia.org/wiki/Rounding#Round_half_down)
//! * [Half Even](https://en.wikipedia.org/wiki/Rounding#Round_half_even) (default)
//!
//! ```edition2018
//! use rusty_money::{Money, Round, iso};
//!
//! // Money can be added, subtracted, multiplied and divided:
//! Money::from_minor(100, iso::USD) + Money::from_minor(100, iso::USD);  // 2 USD
//! Money::from_minor(100, iso::USD) - Money::from_minor(100, iso::USD);  // 0 USD
//! Money::from_minor(100, iso::USD) * 3;                                 // 3 USD
//! Money::from_minor(100, iso::USD) / 3;                                 // 0.333... USD
//!
//! // Money can be rounded by calling the round function:
//! let usd = Money::from_str("-2000.005", iso::USD).unwrap();  // 2000.005 USD
//! usd.round(2, Round::HalfEven);                              // 2000.00 USD
//! usd.round(2, Round::HalfUp);                                // 2000.01 USD
//! usd.round(0, Round::HalfUp);                                // 2000 USD
//!```
//!
//! ## Formatting
//!
//! Calling `format!` or `println!` on Money returns a string with a rounded amount, using separators and symbols
//! according to the locale of the currency. If you need to customize this output, the `Formatter` module
//! accepts a more detailed set of parameters.
//!
//! ```edition2018
//! use rusty_money::{Money, iso};
//!
//! // Money objects can be pretty printed, with appropriate rounding and formatting:
//! let usd = Money::from_str("-2000.009", iso::USD).unwrap();
//! let eur = Money::from_str("-2000.009", iso::EUR).unwrap();
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
//! use rusty_money::{Money, Exchange, ExchangeRate, iso};
//! use rust_decimal_macros::*;
//!
//! // Convert 1000 USD to EUR at a 2:1 exchange rate.
//! let rate = ExchangeRate::new(iso::USD, iso::EUR, dec!(0.5)).unwrap();
//! rate.convert(Money::from_minor(100_000, iso::USD));                                     // 500 EUR
//!
//! // An Exchange can be used to store ExchangeRates for later use
//! let mut exchange = Exchange::new();
//! exchange.set_rate(&rate);
//! exchange.get_rate(iso::USD, iso::EUR);
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
