use std::str::FromStr;

/// Locales represent regions which have different formatting standards for currencies.  
///
/// Each Locale maps 1:1 to a LocalFormat, which contains the characteristics for formatting.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Locale {
    EnUs,
    EnIn,
    EnEu,
    EnBy,
}

/// `LocalFormat` contains a set of characteristics which can be used to format currencies.
///
/// Each LocalFormat maps 1:1 to a locale, which defines the region that maps to the characteristics.
#[derive(Debug, PartialEq, Eq)]
pub struct LocalFormat {
    pub name: &'static str,
    pub digit_separator: char,
    pub digit_separator_pattern: &'static str,
    pub exponent_separator: char,
}

impl LocalFormat {
    /// Returns a vector indicating where digit separators should be applied on a Money amount.
    ///
    /// For example, [3,3,3] indicates that the digit separator should be applied after the 3rd, 6th and 9th digits.  
    pub fn digit_separator_pattern(&self) -> Vec<usize> {
        let v: Vec<&str> = self.digit_separator_pattern.split(", ").collect();
        v.iter().map(|x| usize::from_str(x).unwrap()).collect()
    }

    /// Returns the associated LocalFormat given a Locale.
    pub fn from_locale(locale: Locale) -> LocalFormat {
        use Locale::*;

        match locale {
            EnUs => LocalFormat {
                name: "en-us",
                digit_separator: ',',
                digit_separator_pattern: "3, 3, 3",
                exponent_separator: '.',
            },
            EnIn => LocalFormat {
                name: "en-in",
                digit_separator: ',',
                digit_separator_pattern: "3, 2, 2",
                exponent_separator: '.',
            },
            EnEu => LocalFormat {
                name: "en-eu",
                digit_separator: '.',
                digit_separator_pattern: "3, 3, 3",
                exponent_separator: ',',
            },
            EnBy => LocalFormat {
                name: "en-by",
                digit_separator: ' ',
                digit_separator_pattern: "3, 3, 3",
                exponent_separator: ',',
            },
        }
    }
}
