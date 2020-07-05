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

`Money` consists of an amount, which is represented by a Decimal type that it owns and a
`IsoCurrency`, which it holds a reference to. `IsoCurrency` represents an ISO-4217 currency, and
 stores metadata like its numeric code, full name and symbol.

```rust
// Money can be initialized in a few ways:
use rusty_money::{money, Money, IsoCurrency};
use rusty_money::Iso::*;

money!(2000, "USD");                            // 2000 USD
money!("2000.00", "USD");                       // 2000 USD
Money::new(200000, IsoCurrency::get(USD));         // 2000 USD
Money::from_major(2000, IsoCurrency::get(USD));    // 2000 USD
Money::from_minor(200000, IsoCurrency::get(USD));  // 2000 USD
Money::from_str("2,000.00", "USD").unwrap();    // 2000 USD


// Money objects with the same Currency can be compared:
let hundred = money!(100, "USD");
let thousand = money!(1000, "USD");
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
money!(100, "USD") + money!(100, "USD");        // 200 USD
money!(100, "USD") - money!(100, "USD");        // 0 USD
money!(1, "USD") * 3;                           // 3 USD
money!(3, "USD") / 3;                           // 0.333333333... USD

// Money can be rounded by calling the round function:
let usd = money!("-2000.005", "USD");           // 2000.005 USD
usd.round(2, Round::HalfEven);                  // 2000.00 USD
usd.round(2, Round::HalfUp);                    // 2000.01 USD
usd.round(0, Round::HalfUp);                    // 2000 USD
```

## Formatting

Calling `format!` or `println!` on Money returns a string with a rounded amount, using separators and symbols
according to the locale of the currency. If you need to customize this output, the `Formatter` module
accepts a more detailed set of parameters.

```rust
use rusty_money::{money, Money, IsoCurrency};

// Money objects can be pretty printed, with appropriate rounding and formatting:
let usd = money!("-2000.009", "USD");
let eur = money!("-2000.009", "EUR");
println!("{}", usd); // -$2,000.01
println!("{}", eur); // -€2.000,01;
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
rate.convert(money!(1000, "USD")); // 500 EUR

// An Exchange can be used to store ExchangeRates for later use
let mut exchange = Exchange::new();
exchange.add_or_update_rate(&rate);
exchange.get_rate(IsoCurrency::get(USD), IsoCurrency::get(EUR));
```
