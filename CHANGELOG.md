# Change Log

## [0.4.1] - 2021-01-16

* ISO: Fixed symbols for BHD, ISK, NPR, UYU and alpha code for UYW.
* ISO: Added ALL, STN, UYW currencies [@ObsceneGiraffe](https://github.com/ObsceneGiraffe).
* Crypto: Added COMP, DAI, MKR, UNI, USDC, USDT, XTZ, ZEC
* Fix: `Exchange::get_rate` returns a lifetime `Option<ExchangeRate<'a, T>>` to prevent lifetime conflicts [@RestitutorOrbis](https://github.com/RestitutorOrbis).

## [0.4.0] - 2021-01-01

rusty_money now supports defining your own currencies and has a cryptocurrency module. It no longer depends on lazy_static!,
is a bit faster and and simpler to use. Many breaking changes were needed to support this, so upgrade with care!

* define_currency_set! allows you to create your own custom currency modules. [@jimpo](https://github.com/jimpo)
* CryptoCurrencies (Bitcoin, Ethereum) are now available in the `crypto` module.
* Removed dependency on lazy_static! and made library thread safe.

Breaking Changes:

* `Money::new` was deprecated, please use `Money::from_minor` instead.
* `Money::from_str()` accepts a currency reference `iso::USD` instead of string `"USD"`.
* `Money` requires lifetime and currency type annotiations.
* `money!` macro was deprecated, please use `Money::from_str` instead.
* `Currency::get()` was deprecated, please reference currencies directly by their module like `iso::USD`.
* The old Currency module is now called `iso::Currency`.
* `Currency::from_string` is now `iso::find` and accepts `&str` instead of `String` [@ObsceneGiraffe](https://github.com/ObsceneGiraffe).
* `ExchangeRate::add_or_update_rate` was deprecated, please use `ExchangeRate::set_rate` instead. [@jimpo](https://github.com/jimpo).

## [0.3.6] - 2020-12-26

* Upgraded dependencies.

## [0.3.5] - 2020-07-26

* Bugfix: Incorrect metadata for CZK, HUF, ZMK [@zacharra](https://github.com/zacharra)
* Feature: Money objects can be multiplied by Decimals [@sjoerdsimons](https://github.com/sjoerdsimons)

## [0.3.4] - 2020-04-19

* Bugfix: from_string rejects incorrect digit separators like 1.00,00 EUR [@sjoerdsimons](https://github.com/sjoerdsimons)

## [0.3.3] - 2020-03-13

* Bugfix: Fixing incorrect name and code for BRL (Brazilian real) [@diegooliveira](https://github.com/diegooliveira)

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

* Serialization and Deserialization of Money.
* Benchmarking performance.
* no-std by default.
