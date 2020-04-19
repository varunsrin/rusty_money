# Change Log

## [0.3.4] - 2020-04-19

* Bugfix: from_string rejects incorrect digit separators like 1.00,00 EUR [@sjoerdsimons]

## [0.3.3] - 2020-03-13

* Bugfix: Fixing incorrect name and iso_alpha_code for BRL (Brazilian real) [@diegooliveira]

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

* Currency Declaration: Allow declaration of new currency types (e.g. a cryptocurrency).