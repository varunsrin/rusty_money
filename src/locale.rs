/// Enumerates regions which have unique formatting standards for Currencies.
///
/// Each Locale maps 1:1 to a LocalFormat, which contains the characteristics for formatting.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Locale {
    EnUs,
    EnIn,
    EnEu,
    EnBy,
}

/// Stores currency formatting metadata for a specific region (e.g. EN-US).
#[derive(Debug, PartialEq, Eq)]
pub struct LocalFormat {
    pub name: &'static str,
    pub digit_separator: char,
    pub digit_separator_pattern: &'static [usize],
    pub exponent_separator: char,
}

// Pre-defined patterns to avoid allocation
const PATTERN_3_3_3: &[usize] = &[3, 3, 3];
const PATTERN_3_2_2: &[usize] = &[3, 2, 2];

impl LocalFormat {
    /// Returns the associated LocalFormat given a Locale.
    pub fn from_locale(locale: Locale) -> LocalFormat {
        use Locale::*;

        match locale {
            EnUs => LocalFormat {
                name: "en-us",
                digit_separator: ',',
                digit_separator_pattern: PATTERN_3_3_3,
                exponent_separator: '.',
            },
            EnIn => LocalFormat {
                name: "en-in",
                digit_separator: ',',
                digit_separator_pattern: PATTERN_3_2_2,
                exponent_separator: '.',
            },
            EnEu => LocalFormat {
                name: "en-eu",
                digit_separator: '.',
                digit_separator_pattern: PATTERN_3_3_3,
                exponent_separator: ',',
            },
            EnBy => LocalFormat {
                name: "en-by",
                digit_separator: ' ',
                digit_separator_pattern: PATTERN_3_3_3,
                exponent_separator: ',',
            },
        }
    }
}
