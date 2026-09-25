#![cfg(feature = "iso")]

use rust_decimal_macros::dec;
use rusty_money::{Money, iso};

#[test]
fn corrected_exponents_use_two_decimal_places() {
    for currency in [iso::HUF, iso::MGA, iso::MRU] {
        assert_eq!(*Money::from_minor(123, currency).amount(), dec!(1.23));
        assert_eq!(Money::from_major(1, currency).try_to_minor_units(), Ok(100));
    }
}

#[test]
fn corrected_locales_parse_and_display_local_amounts() {
    let idr = Money::from_str("1.234,56", iso::IDR).unwrap();
    assert_eq!(*idr.amount(), dec!(1234.56));
    assert_eq!(idr.to_string(), "Rp1.234,56");
    assert_eq!(
        *Money::from_str("1.234", iso::IDR).unwrap().amount(),
        dec!(1234)
    );
    assert!(Money::from_str("1,234.56", iso::IDR).is_err());

    let bgn = Money::from_str("123 456,78", iso::BGN).unwrap();
    assert_eq!(*bgn.amount(), dec!(123456.78));
    assert_eq!(bgn.to_string(), "123 456,78лв.");
    assert!(Money::from_str("1,23,456.78", iso::BGN).is_err());
}

#[test]
fn corrected_symbols_display_with_amounts() {
    assert_eq!(
        Money::from_minor(123456, iso::XCG).to_string(),
        "Cg1,234.56"
    );
    assert_eq!(Money::from_major(12, iso::XTS).to_string(), "12XTS");
}
