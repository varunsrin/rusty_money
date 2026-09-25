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
    /// Returns whether two currencies have different identities.
    ///
    /// The default preserves `PartialEq::ne`. Implementations may optimize this
    /// check, but must preserve its result, including all metadata that participates
    /// in equality. Money arithmetic uses this to validate matching currencies.
    #[inline]
    fn is_currency_mismatch(&self, other: &Self) -> bool {
        self != other
    }

    fn to_string(&self) -> String;

    fn exponent(&self) -> u32;

    fn code(&self) -> &'static str;

    fn locale(&self) -> Locale;

    fn symbol(&self) -> &'static str;

    fn symbol_first(&self) -> bool;
}

/// Trait for currency types that can be looked up by code.
/// Required for deserializing Money.
pub trait Findable: FormattableCurrency + Sized {
    /// Look up a currency by its code (e.g., "USD").
    fn find(code: &str) -> Option<&'static Self>;
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
                    use $crate::{Locale, FormattableCurrency, Findable, Locale::*};
                    use std::fmt;

                    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
                    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

                    impl Findable for Currency {
                        fn find(code: &str) -> Option<&'static Self> {
                            find(code)
                        }
                    }

                    $(
                        pub const $currency: &'static self::Currency = &self::Currency {
                        code: $code,
                        exponent: $exp,
                        locale: $loc,
                        minor_units: $min_dem,
                        name: $name,
                        symbol: $sym,
                        symbol_first: $sym_first,
                        };
                    )+

                    pub fn find(code: &str) -> Option<&'static self::Currency> {
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
    fn default_identity_preserves_custom_inequality() {
        use crate::{FormattableCurrency, Locale, Money, MoneyError};
        use std::sync::atomic::{AtomicUsize, Ordering};

        static INEQUALITY_CALLS: AtomicUsize = AtomicUsize::new(0);
        #[derive(Clone, Copy)]
        struct Custom {
            identity: u8,
            symbol: &'static str,
        }
        impl PartialEq for Custom {
            fn eq(&self, other: &Self) -> bool {
                self.identity == other.identity
            }
            #[allow(clippy::partialeq_ne_impl)]
            fn ne(&self, other: &Self) -> bool {
                INEQUALITY_CALLS.fetch_add(1, Ordering::Relaxed);
                !self.eq(other)
            }
        }
        impl Eq for Custom {}
        impl FormattableCurrency for Custom {
            fn to_string(&self) -> String {
                self.code().into()
            }
            fn code(&self) -> &'static str {
                "USD"
            }
            fn exponent(&self) -> u32 {
                2
            }
            fn locale(&self) -> Locale {
                Locale::EnUs
            }
            fn symbol(&self) -> &'static str {
                self.symbol
            }
            fn symbol_first(&self) -> bool {
                true
            }
        }
        let a_currency = Custom {
            identity: 1,
            symbol: "$",
        };
        let b_currency = Custom {
            identity: 1,
            symbol: "US$",
        };
        let other_currency = Custom {
            identity: 2,
            symbol: "$",
        };
        assert!(!a_currency.is_currency_mismatch(&b_currency));
        assert_eq!(INEQUALITY_CALLS.load(Ordering::Relaxed), 1);
        let a = Money::from_minor(100, &a_currency);
        let b = Money::from_minor(50, &b_currency);
        assert_eq!(a.add(b).unwrap().try_to_minor_units(), Ok(150));
        assert_eq!(INEQUALITY_CALLS.load(Ordering::Relaxed), 2);
        let different = Money::from_minor(50, &other_currency);
        assert!(matches!(
            a.add(different),
            Err(MoneyError::CurrencyMismatch { .. })
        ));
        assert_eq!(INEQUALITY_CALLS.load(Ordering::Relaxed), 3);
        #[cfg(feature = "fast")]
        {
            let a = crate::FastMoney::from_minor(100, &a_currency);
            let b = crate::FastMoney::from_minor(50, &b_currency);
            assert_eq!(a.add(b).unwrap().minor_units(), 150);
            assert_eq!(INEQUALITY_CALLS.load(Ordering::Relaxed), 4);
        }
    }

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
