use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rust_decimal_macros::dec;

#[cfg(feature = "iso")]
use rusty_money::iso;
use rusty_money::{Exchange, ExchangeRate, Money};

#[cfg(feature = "fast")]
use rusty_money::FastMoney;

fn bench_money_arithmetic(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let a = Money::from_minor(100_000, iso::USD);
        let b = Money::from_minor(50_000, iso::USD);

        c.bench_function("money_add", |bencher| {
            bencher.iter(|| black_box(a).add(black_box(b)))
        });

        c.bench_function("money_sub", |bencher| {
            bencher.iter(|| black_box(a).sub(black_box(b)))
        });

        c.bench_function("money_mul", |bencher| {
            bencher.iter(|| black_box(a).mul(black_box(100i64)))
        });

        c.bench_function("money_div", |bencher| {
            bencher.iter(|| black_box(a).div(black_box(3i64)))
        });
    }
}

#[cfg(feature = "fast")]
fn bench_fastmoney_arithmetic(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let a = FastMoney::from_minor(100_000, iso::USD);
        let b = FastMoney::from_minor(50_000, iso::USD);

        c.bench_function("fastmoney_add", |bencher| {
            bencher.iter(|| black_box(a).add(black_box(b)))
        });

        c.bench_function("fastmoney_sub", |bencher| {
            bencher.iter(|| black_box(a).sub(black_box(b)))
        });

        c.bench_function("fastmoney_mul", |bencher| {
            bencher.iter(|| black_box(a).mul(black_box(100i64)))
        });

        c.bench_function("fastmoney_div", |bencher| {
            bencher.iter(|| black_box(a).div(black_box(3i64)))
        });
    }
}

#[cfg(feature = "fast")]
fn bench_fastmoney_conversion(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let fast = FastMoney::from_minor(123_456, iso::USD);
        let money = Money::from_minor(123_456, iso::USD);

        c.bench_function("fastmoney_to_money", |bencher| {
            bencher.iter(|| black_box(fast).to_money())
        });

        c.bench_function("fastmoney_from_money", |bencher| {
            bencher.iter(|| FastMoney::from_money(black_box(money)))
        });

        c.bench_function("fastmoney_from_money_lossy", |bencher| {
            bencher.iter(|| FastMoney::from_money_lossy(black_box(money)))
        });
    }
}

fn bench_exchange_lookup(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let mut exchange = Exchange::new();

        // Pre-populate with some rates
        let pairs = [
            (iso::USD, iso::EUR, dec!(0.85)),
            (iso::EUR, iso::GBP, dec!(0.86)),
            (iso::GBP, iso::JPY, dec!(155.0)),
            (iso::USD, iso::GBP, dec!(0.73)),
            (iso::EUR, iso::JPY, dec!(130.0)),
            (iso::USD, iso::JPY, dec!(110.0)),
        ];

        for (from, to, rate) in pairs {
            exchange.set_rate(&ExchangeRate::new(from, to, rate).unwrap());
        }

        c.bench_function("exchange_get_rate", |bencher| {
            bencher.iter(|| exchange.get_rate(black_box(iso::USD), black_box(iso::EUR)))
        });

        c.bench_function("exchange_set_rate", |bencher| {
            let rate = ExchangeRate::new(iso::USD, iso::EUR, dec!(0.85)).unwrap();
            bencher.iter(|| {
                let mut ex = Exchange::new();
                ex.set_rate(black_box(&rate))
            })
        });
    }
}

fn bench_exchange_convert(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let rate = ExchangeRate::new(iso::USD, iso::EUR, dec!(0.85)).unwrap();
        let amount = Money::from_minor(100_000, iso::USD);

        c.bench_function("exchange_convert", |bencher| {
            bencher.iter(|| black_box(rate).convert(black_box(&amount)))
        });
    }
}

fn bench_formatting(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        // Small amount
        let small = Money::from_minor(1_234, iso::USD);
        c.bench_function("money_display_small", |bencher| {
            bencher.iter(|| format!("{}", black_box(&small)))
        });

        // Large amount with many digit separators
        let large = Money::from_minor(123_456_789_012, iso::USD);
        c.bench_function("money_display_large", |bencher| {
            bencher.iter(|| format!("{}", black_box(&large)))
        });
    }
}

fn bench_parsing(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        c.bench_function("money_from_str_simple", |bencher| {
            bencher.iter(|| Money::from_str(black_box("12.34"), iso::USD))
        });

        c.bench_function("money_from_str_large", |bencher| {
            bencher.iter(|| Money::from_str(black_box("1,234,567.89"), iso::USD))
        });
    }
}

fn bench_parse_format_workloads(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        use std::fmt::Write;

        let mut parsing = c.benchmark_group("parse_workloads");
        for (name, amount, currency) in [
            ("ungrouped", "1234.56", iso::USD),
            ("integer", "1234", iso::USD),
            ("us_grouped", "1,234,567.89", iso::USD),
            ("eu_grouped", "1.234.567,89", iso::EUR),
            ("indian_grouped", "1,23,456.78", iso::INR),
            ("space_grouped", "1 234 567,89", iso::BYN),
            ("invalid", "12,34.56", iso::USD),
        ] {
            parsing.bench_function(name, |b| {
                b.iter(|| Money::from_str(black_box(amount), black_box(currency)))
            });
        }
        #[cfg(feature = "crypto")]
        parsing.bench_function("crypto_fraction", |b| {
            b.iter(|| Money::from_str(black_box("1.123456789012345678"), rusty_money::crypto::ETH))
        });
        parsing.finish();

        let mut formatting = c.benchmark_group("format_workloads");
        for currency in [iso::USD, iso::EUR, iso::INR, iso::JPY, iso::BYN] {
            let money = Money::from_minor(-123_456_789, currency);
            formatting.bench_with_input(
                BenchmarkId::new("string", currency.iso_alpha_code),
                &money,
                |b, m| b.iter(|| black_box(m).to_string()),
            );
        }

        let amounts: Vec<_> = (0..1000)
            .map(|i| Money::from_minor(i * 123_457 - 50_000_000, iso::USD))
            .collect();
        let mut output = String::with_capacity(32_000);
        formatting.bench_function("batch_1000_reused_buffer", |b| {
            b.iter(|| {
                output.clear();
                for money in black_box(&amounts) {
                    writeln!(&mut output, "{money}").unwrap();
                }
                black_box(&output);
            })
        });
        formatting.finish();
    }
}

fn bench_comparison(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let a = Money::from_minor(100_000, iso::USD);
        let b = Money::from_minor(50_000, iso::USD);

        c.bench_function("money_compare", |bencher| {
            bencher.iter(|| black_box(a).compare(black_box(&b)))
        });

        c.bench_function("money_gt", |bencher| {
            bencher.iter(|| black_box(a).gt(black_box(&b)))
        });

        c.bench_function("money_is_zero", |bencher| {
            bencher.iter(|| black_box(a).is_zero())
        });

        c.bench_function("money_is_positive", |bencher| {
            bencher.iter(|| black_box(a).is_positive())
        });
    }

    #[cfg(all(feature = "iso", feature = "fast"))]
    {
        let a = FastMoney::from_minor(100_000, iso::USD);
        let b = FastMoney::from_minor(50_000, iso::USD);

        c.bench_function("fastmoney_compare", |bencher| {
            bencher.iter(|| black_box(a).compare(black_box(&b)))
        });

        c.bench_function("fastmoney_is_zero", |bencher| {
            bencher.iter(|| black_box(a).is_zero())
        });
    }
}

fn bench_to_minor_units(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let money = Money::from_minor(123_456_789, iso::USD);

        c.bench_function("money_to_minor_units", |bencher| {
            bencher.iter(|| black_box(money).to_minor_units())
        });
    }

    #[cfg(all(feature = "iso", feature = "fast"))]
    {
        let fast = FastMoney::from_minor(123_456_789, iso::USD);

        c.bench_function("fastmoney_minor_units", |bencher| {
            bencher.iter(|| black_box(fast).minor_units())
        });
    }
}

fn bench_allocate(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let money = Money::from_minor(100_000, iso::USD);

        c.bench_function("money_allocate_3", |bencher| {
            bencher.iter(|| black_box(money).allocate(vec![1, 1, 1]))
        });

        c.bench_function("money_allocate_10", |bencher| {
            bencher.iter(|| black_box(money).allocate(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]))
        });

        c.bench_function("money_allocate_100", |bencher| {
            bencher.iter(|| black_box(money).allocate(vec![1; 100]))
        });

        c.bench_function("money_split_3", |bencher| {
            bencher.iter(|| black_box(money).split(3))
        });

        c.bench_function("money_split_10", |bencher| {
            bencher.iter(|| black_box(money).split(10))
        });
    }
}

fn bench_currency_lookup(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        c.bench_function("currency_lookup_alpha", |bencher| {
            bencher.iter(|| iso::find(black_box("USD")))
        });
        c.bench_function("currency_lookup_numeric", |bencher| {
            bencher.iter(|| iso::find_by_num_code(black_box("840")))
        });
        c.bench_function("currency_lookup_mixed_7", |bencher| {
            bencher.iter(|| {
                for code in black_box(["USD", "EUR", "GBP", "JPY", "INR", "BHD", "ZZZ"]) {
                    black_box(iso::find(black_box(code)));
                }
            })
        });
    }
}

fn bench_accessors(c: &mut Criterion) {
    #[cfg(feature = "iso")]
    {
        let money = Money::from_minor(100_000, iso::USD);

        c.bench_function("money_amount", |bencher| {
            bencher.iter(|| black_box(money.amount()))
        });

        c.bench_function("money_currency", |bencher| {
            bencher.iter(|| black_box(money.currency()))
        });
    }

    #[cfg(all(feature = "iso", feature = "fast"))]
    {
        let fast = FastMoney::from_minor(100_000, iso::USD);

        c.bench_function("fastmoney_minor_units_accessor", |bencher| {
            bencher.iter(|| black_box(fast.minor_units()))
        });

        c.bench_function("fastmoney_currency", |bencher| {
            bencher.iter(|| black_box(fast.currency()))
        });
    }
}

#[cfg(all(feature = "iso", feature = "serde"))]
fn bench_serde(c: &mut Criterion) {
    let money = Money::from_minor(123_456_789, iso::USD);
    let json = serde_json::to_string(&money).unwrap();

    c.bench_function("money_serde_serialize", |bencher| {
        bencher.iter(|| serde_json::to_string(black_box(&money)))
    });

    c.bench_function("money_serde_deserialize", |bencher| {
        bencher.iter(|| serde_json::from_str::<Money<iso::Currency>>(black_box(&json)))
    });
}

criterion_group!(
    benches,
    bench_money_arithmetic,
    bench_exchange_lookup,
    bench_exchange_convert,
    bench_formatting,
    bench_parsing,
    bench_parse_format_workloads,
    bench_comparison,
    bench_to_minor_units,
    bench_allocate,
    bench_accessors,
    bench_currency_lookup,
);

#[cfg(feature = "fast")]
criterion_group!(
    fast_benches,
    bench_fastmoney_arithmetic,
    bench_fastmoney_conversion,
);

#[cfg(all(feature = "iso", feature = "serde"))]
criterion_group!(serde_benches, bench_serde);

#[cfg(all(feature = "fast", feature = "serde"))]
criterion_main!(benches, fast_benches, serde_benches);

#[cfg(all(feature = "fast", not(feature = "serde")))]
criterion_main!(benches, fast_benches);

#[cfg(all(not(feature = "fast"), feature = "serde"))]
criterion_main!(benches, serde_benches);

#[cfg(all(not(feature = "fast"), not(feature = "serde")))]
criterion_main!(benches);
