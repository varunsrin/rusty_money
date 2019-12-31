use crate::currency::Currency;
use crate::locale::Locale;
use std::fmt;

// Allows iterating over the Iso Enum
macro_rules! define_enum {
    ($Name:ident { $($Variant:ident),* $(,)* }) =>
    {
        #[derive(Debug)]
        pub enum $Name {
            $($Variant),*,
        }
        pub const ISO_CURRENCIES: &'static [$Name] = &[$($Name::$Variant),*];
    }
}

// Enum that represents every ISO Currency
define_enum!(Iso {
    AED,
    AFN,
    AMD,
    ANG,
    AOA,
    ARS,
    AUD,
    AWG,
    AZN,
    BHD,
    EUR,
    GBP,
    INR,
    USD,
});

impl fmt::Display for Iso {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Returns Currency given an Iso Enum.
pub fn from_enum(code: &Iso) -> Currency {
    use Iso::*;
    use Locale::*;

    match code {
        AED => Currency {
            exponent: 2,
            iso_alpha_code: "AED",
            iso_numeric_code: "784",
            locale: EnUs,
            minor_denomination: 25,
            name: "United Arab Emirates Dirham",
            symbol: "د.إ",
            symbol_first: false,
        },
        AFN => Currency {
            exponent: 2,
            iso_alpha_code: "AFN",
            iso_numeric_code: "971",
            locale: EnUs,
            minor_denomination: 100,
            name: "Afghan Afghani",
            symbol: "؋",
            symbol_first: false,
        },
        AMD => Currency {
            exponent: 2,
            iso_alpha_code: "AMD",
            iso_numeric_code: "051",
            locale: EnUs,
            minor_denomination: 10,
            name: "Armenian Dram",
            symbol: "դր.",
            symbol_first: false,
        },
        ANG => Currency {
            exponent: 2,
            iso_alpha_code: "ANG",
            iso_numeric_code: "532",
            locale: EnUs,
            minor_denomination: 1,
            name: "Netherlands Antillean Gulden",
            symbol: "ƒ",
            symbol_first: false,
        },
        AOA => Currency {
            exponent: 2,
            iso_alpha_code: "AOA",
            iso_numeric_code: "973",
            locale: EnUs,
            minor_denomination: 10,
            name: "Angolan Kwanza",
            symbol: "Kz",
            symbol_first: false,
        },
        ARS => Currency {
            exponent: 2,
            iso_alpha_code: "ARS",
            iso_numeric_code: "032",
            locale: EnEu,
            minor_denomination: 1,
            name: "Argentine Peso",
            symbol: "$",
            symbol_first: true,
        },
        AUD => Currency {
            exponent: 2,
            iso_alpha_code: "AUD",
            iso_numeric_code: "036",
            locale: EnUs,
            minor_denomination: 5,
            name: "Australian Dollar",
            symbol: "$",
            symbol_first: true,
        },
        AWG => Currency {
            exponent: 2,
            iso_alpha_code: "AWG",
            iso_numeric_code: "533",
            locale: EnUs,
            minor_denomination: 5,
            name: "Aruban Florin",
            symbol: "ƒ",
            symbol_first: false,
        },
        AZN => Currency {
            exponent: 2,
            iso_alpha_code: "AZN",
            iso_numeric_code: "944",
            locale: EnUs,
            minor_denomination: 1,
            name: "Azerbaijani Manat",
            symbol: "₼",
            symbol_first: true,
        },

        // Start on B's
        
        

        
        BHD => Currency {
            exponent: 3,
            iso_alpha_code: "BHD",
            iso_numeric_code: "048",
            locale: EnUs,
            minor_denomination: 5,
            name: "Bahraini Dinar",
            symbol: "ب.د",
            symbol_first: true,
        },
        EUR => Currency {
            exponent: 2,
            iso_alpha_code: "EUR",
            iso_numeric_code: "978",
            locale: EnEu,
            minor_denomination: 1,
            name: "Euro",
            symbol: "€",
            symbol_first: true,
        },
        GBP => Currency {
            exponent: 2,
            iso_alpha_code: "GBP",
            iso_numeric_code: "826",
            locale: EnUs,
            minor_denomination: 1,
            name: "British Pound",
            symbol: "£",
            symbol_first: true,
        },
        INR => Currency {
            exponent: 2,
            iso_alpha_code: "INR",
            iso_numeric_code: "356",
            locale: EnIn,
            minor_denomination: 50,
            name: "Indian Rupee",
            symbol: "₹",
            symbol_first: true,
        },
        USD => Currency {
            exponent: 2,
            iso_alpha_code: "USD",
            iso_numeric_code: "840",
            locale: EnUs,
            minor_denomination: 1,
            name: "United States Dollar",
            symbol: "$",
            symbol_first: true,
        },
    }
}
