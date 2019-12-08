# Debtsolver

Money handles currencies in Rust and helps with rounding, currency tracking and parsing. 

## Example


The easiest way to create Money is by using the flexible money! macro:

 ```rust
use money::Money;
use money::Currency;

fn main() {
    let hundred_dollars = money!(100, "USD");
    let thousand_dollars = money!("1000", "USD");
    let hundred_pounts = money!(100, "GBP");

```

Money handles rounding for you based on the properties of the currency:    

 ```rust
money = money!("-200.009", "USD");
println!("{:?}", money) // -200.01 USD
```

You can perform basic operations on money like: 
 
```rust
 hundred = money!("100", "USD");
 thousand = money!("1000", "USD")
 println!("{:?}", hundred + thousand)     // 1000 USD
 println!("{:?}", thousand > hundred)     // false
 println!("{:?}", thousand.is_positive()) // true
```
 
Currency is still a work in progress, but has hardcoded values for USD and GBP.