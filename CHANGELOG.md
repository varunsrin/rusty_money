# Change Log

### Unreleased
* Money::new now accepts i64 minor units instead of Decimal (breaking change)
* Money objects do not round amounts unless .round() or .format!() are called. (breaking change)
* Currency::find takes strs instead of strings (breaking change)
* Money objects can be multiplied and divided.
* Money can be converted using Exchange and ExchangeRate.  
* Currencies can be looked up by ISO code, country code. 


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


