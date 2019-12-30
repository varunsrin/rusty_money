# Change Log

### Unreleased
* Feature: Money objects do not round amounts unless .round() or .format!() are called. (breaking change)
* Feature: Money objects can be multiplied and divided.
* Feature: Money can be converted using Exchange and ExchangeRate.  
* Feature: Currencies can be looked up by ISO code, country code. 
* Feature: Money objects can stringified with flexible formats, instead of just the currency's default format. 
* Refactor: Most interfaces now return Result<T, MoneyError> instead of panicking or returning <T> (breaking change)
* Refactor: Money::new now accepts i64 minor units instead of Decimal (breaking change)
* Refactor: Currency::find takes strs instead of strings (breaking change)


## [0.2.0] - 2019-12-21
* Currency::new was renamed to Currency::find (breaking change)
* Crate include and use structure was tweaked (breaking change)
* AED, BHD, EUR and INR Currencies are now supported.
* Supported currencies are printed with separators, symbols and signs. (e.g. -$2,000 USD)
* Currency numeric ISO code can be looked up with currency alpha numeric code. 

## [0.1.0] - 2019-12-08
* Basic Money and Currency implementation

## [Planned]

### v0.2.1
* Add rounding modes. 
* Add all ISO standard currencies. 
* Add support for crypto currencies.  
* More thorough examples that use all features. 


