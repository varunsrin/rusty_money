//! A library that handles calculating, rounding, displaying, and parsing units of money according
//! to ISO 4217 standards. The main item exported by the library is `Money`.
//!
//! # Use
//!
//! The easiest way to create Money is by using the flexible money! macro:
//!
//! ```edition2018
//! use rusty_money::money;
//! use rusty_money::Money;
//!
//! let usd = money!("-200.00", "USD");
//! let usd = money!(-200, "USD");
//! ```
//!
//! Money handles rounding and formatting for you based on the properties of the currency:    
//!
//! ```edition2018
//! use rusty_money::money;
//! use rusty_money::Money;
//!
//! let usd = money!("-2000.009", "USD");
//! println!("{}", usd); // -$2,000.01
//!
//! let eur = money!("-2000.009", "EUR");
//! println!("{}", eur) // -€2.000,01;
//! ```
//!   
//! You can perform basic operations on money like:
//!
//! ```edition2018
//! use rusty_money::money;
//! use rusty_money::Money;
//!
//! let hundred = money!(100, "USD");
//! let thousand = money!(1000, "USD");
//! println!("{}", thousand > hundred);     // false
//! println!("{}", thousand.is_positive()); // true
//! println!("{}", hundred + thousand);     // $1,000.00 USD
//! ```

mod currency;
mod money;
pub use currency::*;
pub use money::*;

#[macro_use]
extern crate lazy_static;
