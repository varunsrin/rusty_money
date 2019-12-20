# Rusty-Money

Money lets you handle currencies in Rust easily and takes care of rounding, currency tracking
and parsing monetary symbols according to ISO 4217 standards.

## Example

The easiest way to create Money is by using the flexible money! macro:

 ```rust
use rusty-money::money;
use rusty-money::Money;
use rusty-money::Currency;

fn main() {
    let hundred_dollars = money!(100, "USD");
    let thousand_dollars = money!("1000", "USD");
    let hundred_pounds = money!(100, "GBP");

```

Money handles rounding for you based on the properties of the currency:    

 ```rust
money = money!("-200.009", "USD");
println!("{}", money) // -$2,000.01

money = money!("-200.009", "USD");
println!("{}", money) // -€2,000.01

```

You can perform basic operations on money like: 
 
```rust
 hundred = money!("100", "USD");
 thousand = money!("1000", "USD")
 println!("{}", hundred + thousand)     // $1,000.00
 println!("{}", thousand > hundred)     // true
 println!("{}", thousand.is_positive()) // true
```