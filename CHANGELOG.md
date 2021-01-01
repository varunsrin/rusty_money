# Change Log

## [0.3.6] - 2020-12-26

* Upgraded dependencies.

## [0.3.5] - 2020-07-26

* Bugfix: Incorrect metadata for CZK, HUF, ZMK [@zacharra]
* Feature: Money objects can be multiplied by Decimals [@sjoerdsimons]

## [0.3.4] - 2020-04-19

* Bugfix: from_string rejects incorrect digit separators like 1.00,00 EUR [@sjoerdsimons]

## [0.3.3] - 2020-03-13

* Bugfix: Fixing incorrect name and code for BRL (Brazilian real) [@diegooliveira]

## [0.3.2] - 2020-01-26

* Bugfix: Adding or subtracting different currencies now panics.

## [0.3.1] - 2020-01-11

* Feature: Support all ISO currencies.

## [0.3.0] - 2020-01-06

* Refactor: Most interfaces now return Result<T, MoneyError>  (breaking change)
* Refactor: Money::new takes i64 minor units instead of a Decimal (breaking change)
* Refactor: Currency::find takes strs instead of strings (breaking change)
* Feature: Money objects do not round amounts unless .round() or .format!() are called. (breaking change)
* Feature: Money objects can be multiplied and divided.
* Feature: Money can be converted into different Currencies using Exchange and ExchangeRate.  
* Feature: Money objects can be converted into human readable strings with more flexible structuring.
* Feature: Money objects can be rounded half up, half down or half even.
* Feature: Added 109 new ISO currencies, which can be looked up by ISO code.

## [0.2.0] - 2019-12-21

* Currency::new was renamed to Currency::find (breaking change)
* Crate include and use structure was tweaked (breaking change)
* AED, BHD, EUR and INR Currencies are now supported.
* Supported currencies are printed with separators, symbols and signs. (e.g. -$2,000 USD)
* Currency numeric ISO code can be looked up with currency alpha numeric code.

## [0.1.0] - 2019-12-08

* Basic Money and Currency implementation

## [Planned]

### v0.4.0

The big new feature is support for non-ISO currencies like crypto currencies. You can even define your own custom
currencies. The library no longer depends on lazy_static!, is a bit faster and gives you feature flags to get more 
control over the currency sets you need. Many breaking changes were needed to support this, so upgrade with care!

* Feature: define_currency_set! allows you to create your own custom currency modules.
* Feature: CryptoCurrencies (Bitcoin, Ethereum) are now available in a new crypto_currency module.
* Refactor: The Currency module is now called iso::Currency. (breaking change)
* Refactor: Currency::get() was deprecated in favor of direct module references like iso::USD.  (breaking change)
* Refactor: The money! macro was deprecated since it is no longer syntactically useful in the new module structure. Please use ::from_major, ::from_minor or ::from_str instead  (breaking change)
* Refactor: Money::from_str() takes a reference (iso::USD) instead of string ("USD") for the currency (breaking change).
* Refactor: Currency::from_string is now <currency_module>::from and accepts &str instead of String [@ObsceneGiraffe]. (breaking change)
* Refactor: Removed dependency on lazy_static! and made library thread safe.

### v0.4.1

* Serialization to store values in a database or send over web requests.
* Feature Flags
* Add crypto currencies
