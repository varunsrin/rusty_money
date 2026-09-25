#![cfg(feature = "iso")]

use rust_decimal::Decimal;
use rusty_money::{Money, iso};

#[test]
fn added_codes_resolve_by_both_identifiers_and_scale_amounts() {
    // SIX List One, published 2026-09-17. N.A. entries use the documented
    // library fallback of 0, not an ISO-prescribed decimal exponent.
    for (alpha, numeric, exponent) in [
        ("BOV", "984", 2),
        ("CHE", "947", 2),
        ("CHW", "948", 2),
        ("COU", "970", 2),
        ("MXV", "979", 2),
        ("USN", "997", 2),
        ("UYI", "940", 0),
        ("XAD", "396", 2),
        ("XSU", "994", 0),
        ("XUA", "965", 0),
        ("XXX", "999", 0),
    ] {
        let currency = iso::find(alpha).unwrap();
        assert_eq!(iso::find_by_num_code(numeric), Some(currency));
        let money = Money::from_minor(123, currency);
        assert_eq!(*money.amount(), Decimal::new(123, exponent), "{alpha}");
        assert_eq!(money.try_to_minor_units(), Ok(123), "{alpha}");
    }
}

#[test]
fn added_funds_use_documented_code_symbols_and_na_display_fallback() {
    assert_eq!(Money::from_minor(123, iso::XAD).to_string(), "1.23XAD");
    assert_eq!(Money::from_minor(123, iso::UYI).to_string(), "123UYI");
    for currency in [iso::XSU, iso::XUA, iso::XXX] {
        let money = Money::from_decimal(Decimal::new(125, 2), currency);
        assert_eq!(*money.amount(), Decimal::new(125, 2));
        assert_eq!(money.to_string(), format!("1{}", currency.iso_alpha_code));
    }
}

#[cfg(feature = "serde")]
#[test]
fn added_fund_round_trips_through_serde_lookup() {
    let money = Money::from_minor(123, iso::XAD);
    let json = serde_json::to_string(&money).unwrap();
    let restored: Money<iso::Currency> = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, money);
    assert_eq!(restored.currency(), iso::XAD);
}
