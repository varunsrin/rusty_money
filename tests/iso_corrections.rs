#![cfg(feature = "iso")]

use rust_decimal_macros::dec;
use rusty_money::{Money, MoneyError, iso};

#[test]
fn corrected_exponents_scale_and_convert_minor_units() {
    // ISO uses two decimal places for all three, independently of cash usage.
    for currency in [iso::HUF, iso::MGA, iso::MRU] {
        let positive = Money::from_minor(123, currency);
        let negative = Money::from_minor(-123, currency);
        assert_eq!(*positive.amount(), dec!(1.23));
        assert_eq!(*negative.amount(), dec!(-1.23));
        assert_eq!(positive.try_to_minor_units(), Ok(123));
        assert_eq!(negative.try_to_minor_units(), Ok(-123));
        assert_eq!(Money::from_major(1, currency).try_to_minor_units(), Ok(100));
        assert_eq!(
            Money::from_decimal(dec!(0.001), currency).try_to_minor_units(),
            Err(MoneyError::PrecisionLoss)
        );
    }
    assert_eq!(Money::from_minor(123, iso::HUF).to_string(), "1,23Ft");
    assert_eq!(Money::from_minor(123, iso::MGA).to_string(), "Ar1.23");
    assert_eq!(Money::from_minor(123, iso::MRU).to_string(), "1.23UM");
}

#[cfg(feature = "fast")]
#[test]
fn fast_money_uses_corrected_exponents_in_both_directions() {
    use rusty_money::FastMoney;
    for currency in [iso::HUF, iso::MGA, iso::MRU] {
        let fast = FastMoney::from_major(1, currency).unwrap();
        assert_eq!(fast.minor_units(), 100);
        assert_eq!(*fast.to_money().amount(), dec!(1));
        assert_eq!(
            *FastMoney::from_minor(-123, currency).to_money().amount(),
            dec!(-1.23)
        );
        let money = Money::from_decimal(dec!(1.23), currency);
        assert_eq!(FastMoney::from_money(money).unwrap().minor_units(), 123);
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
    assert_eq!(iso::find("BGN"), Some(iso::BGN));
    assert_eq!(iso::find_by_num_code("975"), Some(iso::BGN));
}

#[test]
fn corrected_symbols_format_without_changing_currency_lookup() {
    assert_eq!(
        Money::from_minor(123456, iso::XCG).to_string(),
        "Cg1,234.56"
    );
    assert_eq!(Money::from_minor(-123, iso::XCG).to_string(), "-Cg1.23");
    assert_eq!(Money::from_major(12, iso::XTS).to_string(), "12XTS");
    assert_eq!(iso::find_by_num_code("532"), Some(iso::XCG));
    assert_eq!(iso::find("ANG"), Some(iso::ANG));
}
