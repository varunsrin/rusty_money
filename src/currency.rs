use crate::Locale;

#[cfg(feature = "crypto")]
mod crypto_currencies;
#[cfg(feature = "crypto")]
pub use crypto_currencies::crypto;

#[cfg(feature = "iso")]
mod iso_currencies;
#[cfg(feature = "iso")]
pub use iso_currencies::iso;

/// Pre-requisite for a Currency to be accepted by a Money.
pub trait FormattableCurrency: PartialEq + Eq + Copy {
    fn to_string(&self) -> String;

    fn exponent(&self) -> u32;

    fn code(&self) -> &'static str;

    fn locale(&self) -> Locale;

    fn symbol(&self) -> &'static str;

    fn symbol_first(&self) -> bool;
}

#[macro_export]
/// Create custom currencies for use with Money types
macro_rules! define_currency_set {
    (
        $(
            $(#[$attr:meta])*
            $module:ident {
                $(
                    $currency:ident: {
                    code: $code:expr,
                    exponent: $exp:expr,
                    locale: $loc:expr,
                    minor_units: $min_dem:expr,
                    name: $name:expr,
                    symbol: $sym:expr,
                    symbol_first: $sym_first:expr,
                    }
                ),+
            }
        ),+
    ) => {
            $(
                $(#[$attr])*
                pub mod $module {
                    use $crate::{Locale, FormattableCurrency, Locale::*};
                    use std::fmt;

                    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
                    pub struct Currency {
                        pub code: &'static str,
                        pub exponent: u32,
                        pub locale: Locale,
                        pub minor_units: u64,
                        pub name: &'static str,
                        pub symbol: &'static str,
                        pub symbol_first: bool,
                    }

                    impl FormattableCurrency for Currency {
                        fn to_string(&self) -> String {
                            self.code().to_string()
                        }

                        fn exponent(&self) -> u32 {
                            self.exponent
                        }

                        fn code(&self) -> &'static str {
                            self.code
                        }

                        fn locale(&self) -> Locale {
                            self.locale
                        }

                        fn symbol(&self) -> &'static str {
                            self.symbol
                        }

                        fn symbol_first(&self) -> bool {
                            self.symbol_first
                        }
                    }

                    $(
                        pub const $currency: self::Currency = self::Currency {
                        code: $code,
                        exponent: $exp,
                        locale: $loc,
                        minor_units: $min_dem,
                        name: $name,
                        symbol: $sym,
                        symbol_first: $sym_first,
                        };
                    )+

                    pub fn find(code: &str) -> Option<self::Currency> {
                        match code {
                            $($code => (Some($currency)),)+
                            _ => None,
                        }
                    }

                    impl fmt::Display for Currency {
                        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                            write!(f, "{}", self.code)
                        }
                    }
                }
            )+
    };
}

#[cfg(test)]
mod tests {
    define_currency_set!(
      real {
        USD: {
          code: "USD",
          exponent: 2,
          locale: EnUs,
          minor_units: 100,
          name: "USD",
          symbol: "$",
          symbol_first: true,
        }
      },
      magic {
        FOO: {
            code: "FOO",
            exponent: 3,
            locale: EnUs,
            minor_units: 100,
            name: "FOO",
            symbol: "F",
            symbol_first: true,
          }
      }
    );

    #[test]
    fn currencies_in_different_modules_are_not_equal() {
        assert_eq!(real::USD.code, "USD");
        assert_eq!(magic::FOO.code, "FOO");
    }

    #[test]
    fn find_works_in_modules() {
        assert_eq!(real::find("USD").unwrap().code, "USD");
        assert_eq!(magic::find("FOO").unwrap().code, "FOO");
    }
}
