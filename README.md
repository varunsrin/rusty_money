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

- **Safety by default**. Operations return `Result` types if they can fail, and are labelled if they can cause precision loss. 
- **Blazing fast performance**. Each operation is designed to be as fast as possible without violating safety requirements. 
- **Ergonomic interfaces**. Simple methods with runtime currency checks over generic constraints. 

The library supports many features necessary for building high performance financial apps: 

- **Currencies** – Supports 150+ ISO-4217 currencies and cryptocurrencies, lets you define new ones. 
- **Internationalization** – Locale-aware formatting and parsing for displaying currencies internationally. 
- **Flexible Representations** – Choose between 128-bit decimals or 64-bit ints for maximum precision or performance.  
- **Exchange** – Built-in support for currency conversion and exchange rate management.
- **Utilities** – Helpers for common operations like splitting money safely and fairly.
- **Serialization** – Fast serialization and deserialization

There are three main interfaces: 

- **`Money`** — Uses 128-bit decimals for high precision and supports allocating, formatting and exchanging. The default option.

- **`FastMoney`** — Uses 64-bit integers and truncates to minor units. Up to 5x faster for arithmetic but less precise and feature-rich. 

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

`Money` is the core type for monetary calculations. It stores amounts as 128-bit decimals, providing precision up to 28 decimal places.

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

All arithmetic operations return `Result` to handle currency mismatches and overflow:

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

Money preserves maximum precision until you explicitly round:

```rust
use rusty_money::{Money, Round, iso};

let amount = Money::from_str("10.005", iso::USD).unwrap();

amount.round(2, Round::HalfUp);                    // => $10.01
amount.round(2, Round::HalfDown);                  // => $10.00
amount.round(2, Round::HalfEven);                  // => $10.00 (banker's rounding)
```

### Allocation

Splitting money fairly is tricky—`$100.00` split 3 ways can't be done evenly. rusty-money handles remainder distribution automatically:

```rust
use rusty_money::{Money, iso};

let total = Money::from_major(100, iso::USD);

// Equal split (remainder distributed to first shares)
let shares = total.split(3).unwrap();
// => [$33.34, $33.33, $33.33]

// Weighted allocation
let parts = total.allocate(vec![70, 20, 10]).unwrap();
// => [$70.00, $20.00, $10.00]
```

### Formatting

`Money` formats according to its currency's locale:

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

`FastMoney` uses `i64` minor units (cents) instead of 128-bit decimals, providing significantly faster arithmetic for performance-critical code paths. It comes with a narrower feature set and has lower precision due to the use of integers.


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

FastMoney truncates intermediate results to minor units, which can accumulate into different final values:

```rust
use rusty_money::{FastMoney, Money, iso};

// With Money (high precision): $10.00 / 3 keeps full decimal precision
let money = Money::from_major(10, iso::USD);
let divided = money.div(3).unwrap();               // => $3.3333333...
let restored = divided.mul(3).unwrap();            // => $10.00 (no loss)

// With FastMoney (low precision): truncates to minor units
let fast = FastMoney::from_major(10, iso::USD).unwrap();
let divided = fast.div(3).unwrap();                // => $3.33 (truncated)
let restored = divided.mul(3).unwrap();            // => $9.99 (1 cent lost)
```

## Feature Flags

```toml
[dependencies]
# Default: ISO-4217 currencies only
rusty-money = "0.5"

# Add cryptocurrency support
rusty-money = { version = "0.4", features = ["crypto"] }

# Add FastMoney
rusty-money = { version = "0.4", features = ["fast"] }

# Add serde serialization
rusty-money = { version = "0.4", features = ["serde"] }

# Everything
rusty-money = { version = "0.4", features = ["iso", "crypto", "fast", "serde"] }
```

## License

MIT
