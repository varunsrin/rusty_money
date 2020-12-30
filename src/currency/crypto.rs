use crate::currency::CryptoCurrency;
use crate::locale::Locale::*;

macro_rules! define_crypto {
  (
    $(
      $currency:ident: {
        code: $code:expr,
        exponent: $exp:expr,
        locale: $loc:expr,
        minor_denomination: $min_dem:expr,
        name: $name:expr,
        symbol: $sym:expr,
        symbol_first: $sym_first:expr,
      }
    ),+
  ) => {
    $(
      pub const $currency: &'static CryptoCurrency = &CryptoCurrency {
        code: $code,
        exponent: $exp,
        locale: $loc,
        minor_denomination: $min_dem,
        name: $name,
        symbol: $sym,
        symbol_first: $sym_first,
      };
    )+

    pub fn find_by_code(code: &str) -> Option<&'static CryptoCurrency> {
      match code {
        $($code => (Some($currency)),)+
        _ => None,
      }
    }
  };
}

define_crypto!(
    BTC: {
        code: "BTC",
        exponent: 8,
        locale: EnUs,
        minor_denomination: 100_000_000,
        name: "Bitcoin",
        symbol: "₿",
        symbol_first: true,
    },
    ETH: {
        code: "ETH",
        exponent: 18,
        locale: EnUs,
        minor_denomination: 1_000_000_000_000_000_000,
        name: "Ethereum",
        symbol: "ETH",
        symbol_first: false,
    }
);
