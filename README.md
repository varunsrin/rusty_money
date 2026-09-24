# rusty-money

[![Build Status]][Github Action] [![Latest Version]][crates.io] [![Docs]][docs.rs]

[Build Status]: https://github.com/varunsrin/rusty_money/actions/workflows/rust.yml/badge.svg
[Github Action]: https://github.com/varunsrin/rusty_money/actions/workflows/rust.yml
[Latest Version]: https://img.shields.io/crates/v/rusty-money.svg
[crates.io]: https://crates.io/crates/rusty-money
[Docs]: https://docs.rs/rusty-money/badge.svg
[docs.rs]: https://docs.rs/rusty-money

## Introduction

rusty-money is a library for handling monetary values in Rust. 

It handles the complex parts of dealing with money: rounding, precision, parsing, i18n, exchange rates, and serialization. It follows [ISO-4217](https://en.wikipedia.org/wiki/ISO_4217) currency definitions and is inspired by Fowler's [money pattern](https://martinfowler.com/eaaCatalog/money.html), Golang's [go-money](https://github.com/Rhymond/go-money) and Ruby's [money](https://github.com/RubyMoney/money). 

The design principles behind rusty-money are: 

- **Safety by default**. Methods return `Result` if fallible, and are labelled if they can cause precision loss. 
- **Blazing fast performance**. Operations are designed to be as fast as possible without compromising safety. 
- **Ergonomic interfaces**. Simple methods with runtime currency checks over generic constraints. 

The library supports many features necessary for building high performance financial apps: 

- **Currencies** – Supports 150+ ISO-4217 currencies and cryptocurrencies, lets you define new ones. 
- **Internationalization** – Locale-aware formatting and parsing for displaying currencies internationally. 
- **Flexible Representations** – Choose 128-bit decimals or 64-bit ints for precision or performance.  
- **Exchange** – Built-in support for currency conversion and exchange rate management.
- **Utilities** – Helpers for common operations like splitting money safely and fairly.
- **Serialization** – Fast serialization and deserialization

There are three main interfaces: 

- **`Money`** — Uses 128-bit decimals for high precision and supports allocating, formatting and exchanging.

- **`FastMoney`** — Stores integral minor units in an `i64`. Integer division and explicitly lossy conversion truncate toward zero.

- **`Exchange`** — Manages currency exchange rates and converts between different currences. 


## Quickstart

Add rusty-money to your `Cargo.toml`:

```toml
[dependencies]
rusty-money = "0.5"
```

Create some money, do math, and split the bill:

```rust
use rusty_money::{Money, iso};

// Create money from major or minor units
let total = Money::from_major(100, iso::USD);      // => $100.00
let tip = Money::from_minor(1875, iso::USD);        // => $18.75

// Arithmetic returns Result for safety
let total = total.add(tip).unwrap();               // => $118.75

// Split fairly among 3 people (remainder goes to first shares)
let shares = total.split(3).unwrap();
// => [$39.59, $39.58, $39.58]

println!("{}", total);                             // => $118.75
```

## Money

`Money` stores a `rust_decimal::Decimal`: a 96-bit integer coefficient, a sign, and a scale from 0 to 28, in a 128-bit representation. The currency's exponent does not restrict the stored amount: USD can hold `1.005` even though a cent is `0.01`.

Arithmetic retains the precision available in `Decimal` without automatically rounding to the currency's minor unit. This is finite decimal arithmetic, not arbitrary precision: repeating fractions and results requiring too many significant digits can be rounded by `Decimal`.

### Creating Money

```rust
use rusty_money::{Money, iso};

// From minor units (cents, pence, etc.)
Money::from_minor(1000, iso::USD);                 // => $10.00

// From major units (dollars, pounds, etc.)
Money::from_major(10, iso::USD);                   // => $10.00

// From a decimal
use rust_decimal_macros::dec;
Money::from_decimal(dec!(10.50), iso::USD);        // => $10.50

// Parse from string (locale-aware)
Money::from_str("1,000.99", iso::USD).unwrap();    // => $1,000.99
Money::from_str("1.000,99", iso::EUR).unwrap();    // => €1.000,99
Money::from_str("1,00,00,000.99", iso::INR).unwrap(); // => ₹1,00,00,000.99
```

### Arithmetic

The `add`, `sub`, `mul`, and `div` methods return `Result` to handle currency mismatches, overflow, or division by zero:

```rust
use rusty_money::{Money, iso};

let a = Money::from_major(100, iso::USD);
let b = Money::from_major(50, iso::USD);

a.add(b).unwrap();                                 // => $150.00
a.sub(b).unwrap();                                 // => $50.00
a.mul(3).unwrap();                                 // => $300.00
a.div(4).unwrap();                                 // => $25.00

// Currency mismatch returns an error
let eur = Money::from_major(50, iso::EUR);
assert!(a.add(eur).is_err());
```

### Comparison

```rust
use rusty_money::{Money, iso};

let hundred = Money::from_major(100, iso::USD);
let fifty = Money::from_major(50, iso::USD);

hundred.gt(&fifty).unwrap();                       // => true
fifty.lt(&hundred).unwrap();                       // => true
hundred.eq(&hundred).unwrap();                     // => true

// Predicates
hundred.is_positive();                             // => true
hundred.is_negative();                             // => false
hundred.is_zero();                                 // => false
```

### Rounding

`round(digits, strategy)` returns a new value rounded to `digits` decimal places; it does not change the original. Pass the currency's exponent to round to minor units. All three strategies round to the nearest value and differ only at exact midpoints:

| Strategy | Midpoint rule | `1.005` to two places | `-1.005` to two places |
| --- | --- | --- | --- |
| `HalfUp` | Away from zero | `1.01` | `-1.01` |
| `HalfDown` | Toward zero | `1.00` | `-1.00` |
| `HalfEven` | To the nearest even retained digit | `1.00` | `-1.00` |

```rust
use rusty_money::{Money, Round, iso};

let amount = Money::from_str("10.005", iso::USD).unwrap();

let rounded = amount.round(iso::USD.exponent, Round::HalfUp);
assert_eq!(rounded.try_to_minor_units(), Ok(1001));
assert_eq!(amount.amount().to_string(), "10.005"); // Original is unchanged
```

### Converting to minor units

Use `try_to_minor_units()` for exact `i64` output. It rejects fractional minor units with `MoneyError::PrecisionLoss` and integral amounts outside `i64` with `MoneyError::Overflow`. Trailing zeros are accepted. Round first if your application wants to accept excess precision:

```rust
use rusty_money::{Money, MoneyError, Round, iso};

let amount = Money::from_str("-1.005", iso::USD).unwrap();
assert_eq!(amount.try_to_minor_units(), Err(MoneyError::PrecisionLoss));
assert_eq!(amount.round(2, Round::HalfUp).try_to_minor_units(), Ok(-101));
assert_eq!(amount.round(2, Round::HalfDown).try_to_minor_units(), Ok(-100));

let padded = Money::from_str("1.2300", iso::USD).unwrap();
assert_eq!(padded.try_to_minor_units(), Ok(123));
```

The older `to_minor_units()` method truncates toward zero: USD `-1.005` becomes `-100` cents. It returns zero if conversion to `i64` fails and can panic if its intermediate scaling overflows. Use the checked method when zero must be distinguishable from failure.

### Allocation

`split` and `allocate` work in integral minor units. Both first **floor** the amount to the currency's minor-unit scale, including for negative amounts. This differs from truncation toward zero: USD `1.005` becomes `1.00`, while USD `-1.005` becomes `-1.01`.

Each share is also floored, then remaining minor units are added to recipients in input order. For `allocate`, zero-weight recipients always receive zero and are skipped when distributing the remainder. The intended total is the floored amount; fractions smaller than a minor unit are not preserved. Round explicitly before splitting if you need another policy.

```rust
use rusty_money::{Money, iso};

let total = Money::from_major(100, iso::USD);

// Equal split (remainder distributed to first shares)
let shares = total.split(3).unwrap();
// => [$33.34, $33.33, $33.33]

// Weighted allocation
let parts = total.allocate(vec![70, 20, 10]).unwrap();
// => [$70.00, $20.00, $10.00]

let negative = Money::from_str("-1.005", iso::USD).unwrap();
let parts = negative.allocate(vec![0, 1, 1]).unwrap();
assert_eq!(parts.iter().map(|m| m.try_to_minor_units().unwrap()).collect::<Vec<_>>(), vec![0, -50, -51]);
```

Zero split counts, empty weights, and all-zero weights return `MoneyError::InvalidRatio`. Shares are calculated with exact integer quotient/remainder arithmetic and successful results preserve the floored total. `MoneyError::Overflow` is returned if the currency exponent exceeds 28, minor-unit arithmetic exceeds `i128`, the weight sum exceeds `u64`, or an individual share cannot be represented exactly as a Decimal.

### Formatting

`Display` formats according to the currency's locale and rounds the displayed amount to its exponent using `HalfEven`. It does not change the stored amount, so displaying an amount is not a substitute for rounding before settlement. For example, USD `1.005` displays as `$1.00` while `amount()` still returns `1.005`.

`Formatter::money` with `Params` allows explicit presentation settings; `rounding: Some(n)` uses `HalfEven` at `n` decimal places, while `None` retains the stored scale. Appending trailing zeros is limited by Decimal's representable scale and coefficient.

```rust
use rusty_money::{Money, iso};

let usd = Money::from_major(-2000, iso::USD);
let eur = Money::from_major(-2000, iso::EUR);
let inr = Money::from_major(-100000, iso::INR);

println!("{}", usd);                               // => -$2,000.00
println!("{}", eur);                               // => -€2.000,00
println!("{}", inr);                               // => -₹1,00,000.00
```

### Custom Currencies

Define your own currencies using the `define_currency_set!` macro:

```rust
use rusty_money::{Money, define_currency_set};

define_currency_set!(
    game {
        GIL: {
            code: "GIL",
            exponent: 0,
            locale: Locale::EnUs,
            minor_units: 1,
            name: "Gil",
            symbol: "G",
            symbol_first: false,
        }
    }
);

let gold = Money::from_major(500, game::GIL);
println!("{}", gold);                              // => 500G
```

## Exchange Rates

Convert money between currencies using `ExchangeRate` and `Exchange`:

```rust
use rusty_money::{Money, Exchange, ExchangeRate, iso};
use rust_decimal_macros::dec;

// Create a rate: 1 USD = 0.85 EUR
let rate = ExchangeRate::new(iso::USD, iso::EUR, dec!(0.85)).unwrap();

// Convert directly
let usd = Money::from_major(100, iso::USD);
let eur = rate.convert(&usd).unwrap();             // => €85.00

// Or store rates in an Exchange for reuse
let mut exchange = Exchange::new();
exchange.set_rate(&rate);

// Look up and convert
if let Some(r) = exchange.get_rate(iso::USD, iso::EUR) {
    let result = r.convert(&usd).unwrap();
    println!("{}", result);                        // => €85.00
}

// Convenience method on Money
let euros = usd.exchange_to(iso::EUR, &exchange).unwrap();
```

## Fast Money

`FastMoney` (the `fast` feature) stores an `i64` count of minor units. It cannot represent fractional minor units, and its range in major units depends on the currency's exponent. For USD, its maximum is `92,233,720,368,547,758.07`; for an exponent-18 currency, it is `9.223372036854775807`.

`from_money` checks for excess precision; `from_money_lossy` explicitly truncates toward zero. Integer division also truncates toward zero, so `-100` minor units divided by `3` becomes `-33`. Addition, subtraction, and multiplication retain exact integer results when they fit.

Both conversion methods return `MoneyError::Overflow` when the resulting minor-unit amount does not fit in `i64`. With the `serde` feature, `FastMoney` deserialization uses the lossy conversion: it truncates fractional minor units and returns a deserialization error for out-of-range amounts.


Only choose `FastMoney` over `Money`: 

- You're doing arithmetic operations with high-frequency and performance is critical.
- Amounts fit within currency precision (no fractional cents).


### Usage

```rust
use rusty_money::{FastMoney, Money, iso};

// Create from minor units (no conversion needed)
let fast = FastMoney::from_minor(10000, iso::USD);  // => $100.00

// Create from major units
let fast = FastMoney::from_major(100, iso::USD).unwrap();

// Fast arithmetic
let a = FastMoney::from_minor(1000, iso::USD);
let b = FastMoney::from_minor(500, iso::USD);
let sum = a.add(b).unwrap();                       // => $15.00

// Convert to Money for advanced features
let money = sum.to_money();
let shares = money.split(3).unwrap();

// Convert back (strict mode errors on precision loss)
let fast_again = FastMoney::from_money(money).unwrap();

// Or use lossy conversion if you accept truncation
let fast_lossy = FastMoney::from_money_lossy(fast_again.to_money());
```

### Precision Differences

`Money` retains sub-minor-unit precision, whereas `FastMoney` integer division discards it. Neither representation makes every division reversible:

```rust
use rusty_money::{FastMoney, Money, iso};

// Money retains Decimal's available precision, not an exact rational 1/3.
let money = Money::from_major(1, iso::USD);
let divided = money.div(3).unwrap();
let restored = divided.mul(3).unwrap();
assert!(restored.amount() < money.amount());
assert_eq!(restored.to_string(), "$1.00");         // Display rounding hides the difference

let fast = FastMoney::from_minor(100, iso::USD);
let restored = fast.div(3).unwrap().mul(3).unwrap();
assert_eq!(restored.minor_units(), 99);
```

## Performance

Common operations take tens of nanoseconds (local Criterion medians on an Apple M4 Max, release build; existing amounts are created before timing, and results vary by hardware and input).

| Operation | Example call | Approximate time |
| --- | --- | --- |
| Find a currency | `iso::find("USD")` | 2 ns |
| Parse an amount | `Money::from_str("12.34", iso::USD)` | 13 ns |
| Add $1,000 and $500 | `amount.add(other)` | 11 ns |
| Format $12.34 | `format!("{amount}")` | 56 ns |
| Allocate $1,000 equally | `amount.allocate(vec![1, 1, 1])` | 75 ns |

## Feature Flags

```toml
[dependencies]
# Default: ISO-4217 currencies only
rusty-money = "0.5"

# Add cryptocurrency support
rusty-money = { version = "0.5", features = ["crypto"] }

# Add FastMoney
rusty-money = { version = "0.5", features = ["fast"] }

# Add serde serialization
rusty-money = { version = "0.5", features = ["serde"] }

# Everything
rusty-money = { version = "0.5", features = ["iso", "crypto", "fast", "serde"] }
```

## License

MIT
