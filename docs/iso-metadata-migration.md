# Migrating corrected ISO currency metadata

These changes alter the interpretation or display of existing currencies and
should ship in a release that permits breaking changes. No public fields or
lookup functions are removed. Historical BGN remains available through its
constant and both lookup functions; its definition moves to the historical section.

## Integer amounts and precision

[SIX List One](https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml),
published September 17, 2026, specifies exponent 2 for HUF, MGA, and MRU.

| Currency | Previous exponent | Corrected exponent | Previous `from_minor(100)` amount | Corrected amount |
| --- | ---: | ---: | ---: | ---: |
| HUF | 0 | 2 | 100 | 1.00 |
| MGA | 1 | 2 | 10.0 | 1.00 |
| MRU | 1 | 2 | 10.0 | 1.00 |

To preserve the major-unit value of persisted integers interpreted with the old
exponents, multiply HUF integers by 100 and MGA/MRU integers by 10, checking for
overflow. Apply this only to data encoded under the old library convention;
integers already encoded according to ISO need no conversion. Version the stored
representation so mixed data cannot be silently rescaled.

This affects `Money::from_minor`, `to_minor_units`, `try_to_minor_units`, splitting
and allocation precision, and `FastMoney` construction and conversion. `FastMoney`
stores an integer, so the same raw integer now denotes a different major amount.
`Money::from_major` and `Money::from_decimal` retain their major-unit value; default
display now uses two decimal places. Money's serde representation stores a decimal
major-unit amount and a currency code, so those amounts must not be multiplied.

ISO precision does not prescribe the smallest circulating coin or a cash-rounding
increment. These changes do not introduce cash rounding or modify the legacy
`Currency::minor_units` denomination field. That field does not drive Money's
scaling, allocation, or default formatting.

## Parsing and display

| Currency | Previous behavior | Corrected behavior |
| --- | --- | --- |
| BGN | Indian grouping and decimal point: `1,23,456.78лв.` | Space grouping and decimal comma: `123 456,78лв.` |
| IDR | `Rp1,234.56` | `Rp1.234,56` |
| XCG | `1,234.56ƒ` | `Cg1,234.56` |
| XTS | `12oz t` | `12XTS` |

Update snapshots and consumers of formatted strings. BGN and IDR changes also
affect `Money::from_str`, which accepts amount strings without currency symbols.
For example, IDR input `1.234` now means 1234 rather than 1.234. Do not pass old
localized strings through the new locale and assume their values are unchanged.
For machine-readable decimal major-unit input, parse it as a `Decimal` and use
`Money::from_decimal`, or retain an explicitly configured legacy currency.

BGN uses the existing `EnBy` locale, the closest supported format to Bulgarian:
it uses an ordinary space, whereas CLDR uses a nonbreaking space. This change
does not add full CLDR formatting or accept nonbreaking spaces in the parser.

The full metadata participates in `Currency` equality and hashing. Custom copies
of old descriptors will differ from the corrected constants and may produce
`CurrencyMismatch` when combined. Migrate such descriptors along with amounts.
Alphabetic and numeric identifiers stay unchanged. Numeric code 532 continues to
resolve to XCG; ANG remains accessible by its alphabetic code.

## Sources

- [ISO Amendment 180](https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/amendments/dl-currency-iso-amendment-180.pdf): BGN becomes historical on January 1, 2026, at 1 EUR = 1.95583 BGN.
- [CBCS Caribbean guilder FAQ](https://www.centralbank.cw/functions/banknotes-coins/caribbean-guilder/frequently-asked-questions): the official XCG symbol is Cg.
- [CBCS banknotes](https://www.centralbank.cw/functions/banknotes-coins/caribbean-guilder-banknotes): prefix usage such as Cg50.
- [Unicode CLDR Bulgarian](https://unicode.org/cldr/charts/49/summary/bg.html) and [Indonesian](https://unicode.org/cldr/charts/49/summary/id.html): locale separators.
- [SIX List One](https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml): XTS is a testing code; using XTS as its display symbol is a library choice, not an ISO-prescribed symbol.
