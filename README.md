# rusty-money &emsp; [![Build Status]][travis] [![Latest Version]][crates.io] [![Docs]][docs.rs]

[Build Status]: https://travis-ci.com/varunsrin/rusty_money.svg?branch=master
[travis]: https://travis-ci.com/varunsrin/rusty_money
[Latest Version]: https://img.shields.io/crates/v/rusty-money.svg
[crates.io]: https://crates.io/crates/rusty-money
[Docs]: https://docs.rs/rusty-money/badge.svg
[docs.rs]: https://docs.rs/rusty-money

rusty-money handles the messy parts of dealing with money like rounding, precision, parsing and internationalization.
It supports [ISO-4217](https://en.wikipedia.org/wiki/ISO_4217) currencies, common crypto currencies and lets you
define your own. The main items exported by the library are `Money` and the `iso` and `crypto` currency sets.

## Usage

A `Money` object is created by supplying an amount and a currency. Amounts can be specified in numeric or string types
but will be stored as precise decimals internally. You can select a bundled currency or make your own. Here's a
quick example of how you would make your own `Currency` and then create some `Money` with it:

```rust
use rusty-money::{Money, define_currency_set};

define_currency_set!(
  video_game {
    GIL: {
      code: "GIL",
      exponent: 2,
      locale: Locale::EnUs,
      minor_units: 100,
      name: "GIL",
      symbol: "G",
      symbol_first: true,
    }
  }
);

Money::from_major(2_000, video_game::GIL);              // 2000 GIL
Money::from_minor(200_000, video_game::GIL);            // 2000 GIL
Money::from_str("2,000.00", video_game::GIL).unwrap();  // 2000 GIL
 
// Currencies can be looked up by code. 
let gil = video_game::find("GIL").unwrap();                        
Money::from_major(2_000, gil);                          // 2000 GIL
```

## Features: Currency Sets

rusty_money provides two currency sets for convenience : `iso`, which implements ISO-4217 currencies and `crypto` which
implements popular cryptocurencies. `iso` is enabled by default, and you can add `crypto` by enabling the feature:

```toml
[dependencies]
rusty-money = { version = "0.4.0", features = ["iso", "crypto"] }
```

The currency sets can then be used like this:

```rust
use rusty-money::{Money, iso, crypto};

Money::from_major(2_000, iso::USD);        // 2000 U.S Dollars
Money::from_major(2_000, iso::GBP);        // 2000 British Pounds
Money::from_major(2, crypto::BTC);         // 2 Bitcoin
```

Money objects of the same currency can be compared:

 ```rust
use rusty-money::{Money, iso};
let hundred = Money::from_minor(10_000, iso::USD);
let thousand = Money::from_minor(100_000, iso::USD);

println!("{}", thousand > hundred);     // false
println!("{}", thousand.is_positive()); // true
```

## Precision, Rounding and Math

Money objects are immutable, and operations that change amounts create a new instance of Money. Amounts are stored
as 128 bit fixed-precision [Decimals](https://github.com/paupino/rust-decimal), and handle values as large as
2<sup>96</sup> / 10<sup>28</sup>. Operations on Money retain the maximum possible precision. When you want less
precision, you call the `round` function, which  supports three modes:

* [Half Up](https://en.wikipedia.org/wiki/Rounding#Round_half_up)
* [Half Down](https://en.wikipedia.org/wiki/Rounding#Round_half_down)
* [Half Even](https://en.wikipedia.org/wiki/Rounding#Round_half_even) (default)

Money can be added, subtracted, multiplied and divided like this:

```rust
use rusty-money::{Money, Round, iso};

Money::from_minor(100, iso::USD) + Money::from_minor(100, iso::USD);  // 2 USD
Money::from_minor(100, iso::USD) - Money::from_minor(100, iso::USD);  // 0 USD
Money::from_minor(100, iso::USD) * 3;                                 // 3 USD
Money::from_minor(100, iso::USD) / 3;                                 // 0.333... USD

let usd = Money::from_str("-2000.005", iso::USD).unwrap();            // 2000.005 USD
usd.round(2, Round::HalfEven);                                        // 2000.00 USD
usd.round(2, Round::HalfUp);                                          // 2000.01 USD
usd.round(0, Round::HalfUp);                                          // 2000 USD
```

## Formatting

Calling `format!` or `println!` on Money returns a string with a rounded amount, using separators and symbols
according to the locale of the currency. If you need to customize this output, the `Formatter` module
accepts a more detailed set of parameters.

```rust
use rusty-money::{Money, iso};
let usd = Money::from_str("-2000.009", iso::USD).unwrap();
let eur = Money::from_str("-2000.009", iso::EUR).unwrap();

println!("{}", usd);                                        // -$2,000.01
println!("{}", eur);                                        // -€2.000,01;
```

## Exchange

The library also provides two additional types - `Exchange` and `ExchangeRates` to convert Money from one currency
to another.

```rust
use rusty-money::{Money, Exchange, ExchangeRate, iso};
use rust_decimal_macros::*;

// Convert 1000 USD to EUR at a 2:1 exchange rate.
let rate = ExchangeRate::new(iso::USD, iso::EUR, dec!(0.5)).unwrap();
rate.convert(Money::from_minor(100_000, iso::USD));                    // 500 EUR

// An Exchange can be used to store ExchangeRates for later use
let mut exchange = Exchange::new();
exchange.set_rate(&rate);
exchange.get_rate(iso::USD, iso::EUR);
```
