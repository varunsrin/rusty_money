//! Handle money and currency conversions.
//!
//! Money lets you handle currencies in Rust easily and takes care of rounding, currency tracking
//! and parsing monetary symbols according to ISO 4217 standards.
//!
//! # Use
//!
//! The easiest way to create Money is by using the flexible money! macro:
//!
//! ```edition2018
//! use rusty_money::money;
//! use rusty_money::Money;
//!
//! money = money!("-200.00", "USD");
//! money = money!(-200, "USD");
//! ```
//!
//! Money handles rounding and formatting for you based on the properties of the currency:    
//!
//! ```edition2018
//! //! use rusty_money::money;
//! use rusty_money::Money;
//! 
//! money = money!("-2000.009", "USD");
//! println!("{}", money); // -$2,000.01
//!
//! money = money!("-2000.009", "EUR");
//! println!("{}", money) // -€2,000.01;
//! ```
//!   
//! You can perform basic operations on money like:
//!
//! ```edition2018
//! //! use rusty_money::money;
//! use rusty_money::Money;
//! 
//! hundred = money!("100", "USD");
//! thousand = money!("1000", "USD");
//! println!("{}", hundred + thousand);     // $1,000.00 USD
//! println!("{}", thousand > hundred);     // false
//! println!("{}", thousand.is_positive()); // true
//! ```

pub mod currency;
pub mod money;
#[macro_use]
extern crate lazy_static;
