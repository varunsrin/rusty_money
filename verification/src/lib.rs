//! Kani harnesses against the public production API. No replacement arithmetic.

#[cfg(kani)]
mod proofs {
    use rusty_money::{FastMoney, MoneyError, iso};

    fn check_integer_result(result: Result<FastMoney<'_, iso::Currency>, MoneyError>, exact: i128) {
        if exact < i64::MIN as i128 || exact > i64::MAX as i128 {
            assert!(matches!(result, Err(MoneyError::Overflow)));
        } else {
            let value = result.unwrap();
            assert_eq!(value.minor_units() as i128, exact);
            assert_eq!(value.currency(), iso::USD);
        }
    }

    /// All pairs of i64 inputs, including overflow, compared with wider arithmetic.
    #[kani::proof]
    #[kani::unwind(40)]
    fn fast_add_contract() {
        let a: i64 = kani::any();
        let b: i64 = kani::any();
        check_integer_result(
            FastMoney::from_minor(a, iso::USD).add(FastMoney::from_minor(b, iso::USD)),
            a as i128 + b as i128,
        );
    }

    #[kani::proof]
    #[kani::unwind(40)]
    fn fast_sub_contract() {
        let a: i64 = kani::any();
        let b: i64 = kani::any();
        check_integer_result(
            FastMoney::from_minor(a, iso::USD).sub(FastMoney::from_minor(b, iso::USD)),
            a as i128 - b as i128,
        );
    }

    #[kani::proof]
    #[kani::unwind(40)]
    fn fast_neg_contract() {
        let a: i64 = kani::any();
        check_integer_result(FastMoney::from_minor(a, iso::USD).neg(), -(a as i128));
    }

    /// Currency mismatch takes precedence even if the arithmetic would overflow.
    #[kani::proof]
    #[kani::unwind(40)]
    fn fast_mismatch_contract() {
        let a = FastMoney::from_minor(kani::any(), iso::USD);
        let b = FastMoney::from_minor(kani::any(), iso::EUR);
        assert!(matches!(a.add(b), Err(MoneyError::CurrencyMismatch { .. })));
        assert!(matches!(a.sub(b), Err(MoneyError::CurrencyMismatch { .. })));
    }

    /// 10^e exceeds i64::MAX for every e >= 19; only zero is representable.
    #[kani::proof]
    #[kani::unwind(40)]
    fn major_large_scale_contract() {
        let mut currency = *iso::USD;
        currency.exponent = kani::any();
        kani::assume(currency.exponent >= 19);
        let amount: i64 = kani::any();
        let result = FastMoney::from_major(amount, &currency);
        if amount == 0 {
            assert_eq!(result.unwrap().minor_units(), 0);
        } else {
            assert!(matches!(result, Err(MoneyError::Overflow)));
        }
    }
}
