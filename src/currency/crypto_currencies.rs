use crate::define_currency_set;

define_currency_set!(
  crypto {
      BTC: {
          code: "BTC",
          exponent: 8,
          locale: EnUs,
          minor_units: 100_000_000,
          name: "Bitcoin",
          symbol: "₿",
          symbol_first: true,
      },
      ETH: {
          code: "ETH",
          exponent: 18,
          locale: EnUs,
          minor_units: 1_000_000_000_000_000_000,
          name: "Ethereum",
          symbol: "ETH",
          symbol_first: false,
      }
  }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_returns_known_currencies() {
        let currency_by_code = crypto::find("BTC").unwrap();
        assert_eq!(currency_by_code.code, "BTC");
        assert_eq!(currency_by_code.exponent, 8);
        assert_eq!(currency_by_code.symbol, "₿");
    }

    #[test]
    fn find_returns_none_on_unknown_currency() {
        assert_eq!(crypto::find("fake"), None,);
    }

    #[test]
    fn currency_can_be_accessed_by_reference() {
        assert_eq!(crypto::ETH.code, "ETH");
        assert_eq!(crypto::ETH.exponent, 18);
        assert_eq!(crypto::ETH.symbol, "ETH");
    }

    #[test]
    fn find_and_reference_point_to_same() {
        assert_eq!(crypto::BTC, crypto::find("BTC").unwrap());
    }
}
