# Rusty-Money &emsp; [![Build Status]][travis] [![Latest Version]][crates.io]

[Build Status]: https://travis-ci.com/varunsrin/rusty_money.svg?branch=master
[travis]: https://travis-ci.com/varunsrin/rusty_money
[Latest Version]: https://img.shields.io/crates/v/rusty-money.svg
[crates.io]: https://crates.io/crates/rusty-money

rusty_money takes care of calculating, rounding, displaying, and parsing units of money according to ISO 4217 standards.

## Example

The easiest way to create Money is by using the flexible money! macro:

 ```rust
use rusty_money::money;
use rusty_money::Money;

let hundred_dollars = money!(100, "USD");
let thousand_dollars = money!("1000", "USD");
let hundred_pounds = money!(100, "GBP");
```

Money handles rounding for you based on the properties of the currency:    

 ```rust
let usd = money!("-2000.009", "USD");
println!("{}", usd); // -$2,000.01

let eur = money!("-2000.009", "EUR");
println!("{}", eur); // -€2.000,01
```

You can perform basic operations on money like: 
 
```rust
 let hundred = money!(100, "USD");
 let thousand = money!(1000, "USD");
 println!("{}", thousand > hundred);     // true
 println!("{}", thousand.is_positive()); // true
 println!("{}", hundred + thousand);     // $1,000.00
```