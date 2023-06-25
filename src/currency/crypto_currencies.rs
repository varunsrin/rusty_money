use crate::define_currency_set;

define_currency_set!(
    /// Crypto Currency Set
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
        COMP: {
            code: "COMP",
            exponent: 18,
            locale: EnUs,
            minor_units: 1_000_000_000_000_000_000,
            name: "Compound",
            symbol: "COMP",
            symbol_first: false,
        },
        DAI: {
            code: "DAI",
            exponent: 18,
            locale: EnUs,
            minor_units: 1_000_000_000_000_000_000,
            name: "Dai Stablecoin",
            symbol: "DAI",
            symbol_first: false,
        },
        ETH: {
            code: "ETH",
            exponent: 18,
            locale: EnUs,
            minor_units: 1_000_000_000_000_000_000,
            name: "Ethereum",
            symbol: "ETH",
            symbol_first: false,
        },
        MKR: {
            code: "MKR",
            exponent: 18,
            locale: EnUs,
            minor_units: 1_000_000_000_000_000_000,
            name: "Maker",
            symbol: "MKR",
            symbol_first: false,
        },
        UNI: {
            code: "UNI",
            exponent: 18,
            locale: EnUs,
            minor_units: 1_000_000_000_000_000_000,
            name: "Uniswap",
            symbol: "UNI",
            symbol_first: false,
        },
        USDC: {
            code: "USDC",
            exponent: 6,
            locale: EnUs,
            minor_units: 1_000_000,
            name: "USD Coin",
            symbol: "USDC",
            symbol_first: false,
        },
        USDT: {
            code: "USDT",
            exponent: 6,
            locale: EnUs,
            minor_units: 1_000_000,
            name: "Tether",
            symbol: "USDT",
            symbol_first: false,
        },
        XTZ: {
            code: "XTZ",
            exponent: 6,
            locale: EnUs,
            minor_units: 1_000_000,
            name: "Tezos",
            symbol: "XTZ",
            symbol_first: false,
        },
        ZEC: {
            code: "ZEC",
            exponent: 8,
            locale: EnUs,
            minor_units: 100_000_000,
            name: "ZCash",
            symbol: "ZEC",
            symbol_first: false,
        },
        // https://www.bitcoincash.org/
        BCH: {
            code: "BCH",
            exponent: 8,
            locale: EnUs,
            minor_units: 100_000_000,
            name: "Bitcoin Cash",
            symbol: "BCH",
            symbol_first: false,
        },
        // https://bitcoinsv.com/
        BSV: {
            code: "BSV",
            exponent: 8,
            locale: EnUs,
            minor_units: 100_000_000,
            name: "Bitcoin SV",
            symbol: "BSV",
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
        assert_eq!(crypto::find("fake"), None);
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
