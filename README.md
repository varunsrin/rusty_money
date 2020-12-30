# rusty-money &emsp; [![Build Status]][travis] [![Latest Version]][crates.io] [![Docs]][docs.rs]

[Build Status]: https://travis-ci.com/varunsrin/rusty_money.svg?branch=master
[travis]: https://travis-ci.com/varunsrin/rusty_money
[Latest Version]: https://img.shields.io/crates/v/rusty-money.svg
[crates.io]: https://crates.io/crates/rusty-money
[Docs]: https://docs.rs/rusty-money/badge.svg
[docs.rs]: https://docs.rs/rusty-money

A library that handles calculating, rounding, displaying, and parsing units of money according
to ISO 4217 standards. The main items exported by the library are `Money` and `Currency`.

## Usage

`Money` consists of an amount and a currency. The amount is a Decimal type. The currency can be anything that 
implements the `FormattableCurrency` trait. `IsoCurrency` is provided as an implementation of this trait which represents
ISO-417 Currencies, along with all their metadata and formatting styles. `Currency` is also provided implementing this 
trait which can be used to represent non-ISO currencies.

```rust
use rusty_money::{money, Money, Currency};


let v_bucks = Currency::new("VBX", 2)                 // Create a US Dollar with an exponent of 2.
Money::from_major(1, v_bucks)                         // One V Buck.
Money::from_minor(100, v_bucks)                       // One V Buck
Money::from_stringable("2,000.00", v_bucks).unwrap(); // One V Buck

// There's also an ISO Money library for working with common currencies.
use rusty_money::{IsoCurrency};
use rusty_money::Iso::*;

iso_money!(2000, "USD");                            // 2000 USD
iso_money!("2000.00", "USD");                       // 2000 USD
Money::from_major(2000, IsoCurrency::get(USD));     // 2000 USD
Money::from_minor(200000, IsoCurrency::get(USD));   // 2000 USD
IsoMoney::from_str("2,000.00", "USD").unwrap();     // 2000 USD


// Money objects with the same Currency can be compared:
let hundred_usd = iso_money!(100, "USD");
let thousand_usd = iso_money!(1000, "USD");
println!("{}", thousand > hundred);     // false
println!("{}", thousand.is_positive()); // true
```

## Precision and Rounding

Money objects are immutable, and operations that change the amount or currency of Money simply create
a new instance. Money uses a 128 bit fixed-precision [Decimal](https://github.com/paupino/rust-decimal)
to represent amounts, and it represents values as large as 2<sup>96</sup> / 10<sup>28</sup>. By default,
operations on Money always retain maximum possible precision. When you do need to round money, you can call
 the `round` function, which  supports three modes:

* [Half Up](https://en.wikipedia.org/wiki/Rounding#Round_half_up)
* [Half Down](https://en.wikipedia.org/wiki/Rounding#Round_half_down)
* [Half Even](https://en.wikipedia.org/wiki/Rounding#Round_half_even) (default)

```rust
use rusty_money::{money, Money, IsoCurrency, Round};

// Money can be added, subtracted, multiplied and divided:
iso_money!(100, "USD") + iso_money!(100, "USD");        // 200 USD
iso_money!(100, "USD") - iso_money!(100, "USD");        // 0 USD
iso_money!(1, "USD") * 3;                           // 3 USD
iso_money!(3, "USD") / 3;                           // 0.333333333... USD

// Money can be rounded by calling the round function:
let usd = iso_money!("-2000.005", "USD");           // 2000.005 USD
usd.round(2, Round::HalfEven);                  // 2000.00 USD
usd.round(2, Round::HalfUp);                    // 2000.01 USD
usd.round(0, Round::HalfUp);                    // 2000 USD
```

## Formatting

Currencies supporting the FormattableCurrency trait can be localized and formatted for display.

Calling `format!` or `println!` on Money returns a string with a rounded amount, using separators and symbols
according to the locale of the currency. If you need to customize this output, the `Formatter` module
accepts a more detailed set of parameters.

```rust
use rusty_money::{money, Money, IsoCurrency};

// ISO Money objects can be pretty printed, with appropriate rounding and formatting:
let usd = iso_money!("-2000.009", "USD");
let eur = iso_money!("-2000.009", "EUR");
println!("{}", usd); // -$2,000.01
println!("{}", eur); // -€2.000,01;

// TODO: How do you call the formatter directly?
```

## Exchange

The library also provides two additional types - `Exchange` and `ExchangeRates` to convert Money from one currency
to another.

```rust
use rusty_money::{money, Money, IsoCurrency, Exchange, ExchangeRate};
use rusty_money::Iso::*;
use rust_decimal_macros::*;

// Convert 1000 USD to EUR at a 2:1 exchange rate.
let rate = ExchangeRate::new(IsoCurrency::get(USD), IsoCurrency::get(EUR), dec!(0.5)).unwrap();
rate.convert(iso_money!(1000, "USD")); // 500 EUR

// An Exchange can be used to store ExchangeRates for later use
let mut exchange = Exchange::new();
exchange.set_rate(&rate);
exchange.get_rate(IsoCurrency::get(USD), IsoCurrency::get(EUR));
```
