use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Locale {
    EnUs,
    EnIn,
    EnEu,
}

/// The `LocalFormat` type
///
/// Stores formatting data relevant to the region.
#[derive(Debug, PartialEq, Eq)]
pub struct LocalFormat {
    pub name: &'static str,
    pub digit_separator: char,
    pub digit_separator_pattern: &'static str,
    pub exponent_separator: char,
}

/// Returns LocalFormat given the Locale.
impl LocalFormat {
    /// Returns a vector indicating where digit separators should be applied for a given currency.  
    pub fn digit_separator_pattern(&self) -> Vec<usize> {
        let v: Vec<&str> = self.digit_separator_pattern.split(", ").collect();
        v.iter().map(|x| usize::from_str(x).unwrap()).collect()
    }

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
        }
    }
}
