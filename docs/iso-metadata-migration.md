# Updating ISO currency metadata

This update fixes how several currencies are read and displayed. If you store
amounts as integers or formatted strings, check the changes below before upgrading.

## Stored integers

HUF, MGA, and MRU now use two decimal places, as specified by ISO.

| Currency | Old `from_minor(100)` amount | New amount | Convert integers stored using the old scale |
| --- | ---: | ---: | --- |
| HUF | 100 | 1.00 | Multiply by 100 |
| MGA | 10.0 | 1.00 | Multiply by 10 |
| MRU | 10.0 | 1.00 | Multiply by 10 |

Only rescale integers written using the old library settings, and check for
overflow. Integers already stored using ISO's scale need no change. Keep track of
which scale your stored data uses.

This affects minor-unit conversions, splitting, allocation, and `FastMoney`.
Amounts passed to `Money::from_major` or `Money::from_decimal` keep their value.
Money's serde format stores decimal amounts in major units; do not rescale them.

The `Currency::minor_units` field is unchanged. Amount conversion and formatting
use `exponent`. These corrections do not add cash rounding.

## Strings and display

| Currency | Old display | New display |
| --- | --- | --- |
| BGN | `1,23,456.78лв.` | `123 456,78лв.` |
| IDR | `Rp1,234.56` | `Rp1.234,56` |
| XCG | `1,234.56ƒ` | `Cg1,234.56` |
| XTS | `12oz t` | `12XTS` |

Update tests and consumers that expect the old display. BGN and IDR parsing also
changes: IDR input `1.234` now means 1234 rather than 1.234. Convert old localized
strings before using the new settings. For machine-readable decimal input, parse
it as a `Decimal` and use `Money::from_decimal`.

BGN uses ordinary spaces with the existing `EnBy` format. CLDR uses nonbreaking
spaces, which this parser does not accept. BGN is now listed as historical, but
its constant and both lookups remain available.

Currency equality includes metadata. Update custom copies of the old currencies
to avoid `CurrencyMismatch` when combining them with the corrected constants.

## Sources

- [ISO Amendment 180](https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/amendments/dl-currency-iso-amendment-180.pdf): BGN becomes historical on January 1, 2026, at 1 EUR = 1.95583 BGN.
- [CBCS Caribbean guilder FAQ](https://www.centralbank.cw/functions/banknotes-coins/caribbean-guilder/frequently-asked-questions): the official XCG symbol is Cg.
- [CBCS banknotes](https://www.centralbank.cw/functions/banknotes-coins/caribbean-guilder-banknotes): prefix usage such as Cg50.
- [Unicode CLDR Bulgarian](https://unicode.org/cldr/charts/49/summary/bg.html) and [Indonesian](https://unicode.org/cldr/charts/49/summary/id.html): locale separators.
- [SIX List One](https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml): XTS is a testing code; using XTS as its display symbol is a library choice, not an ISO-prescribed symbol.
