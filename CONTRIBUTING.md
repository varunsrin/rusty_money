# Contributing

## Testing

For everyday development, run `cargo test --all-features`. Proptest uses its
normal 256-case default; increase it with `PROPTEST_CASES=10000` when investigating
arithmetic or representation changes. Saved failures in `proptest-regressions/`
are replayed automatically and should stay checked in.

The pre-commit hook checks formatting. Set `RUSTY_MONEY_FULL_CHECKS=1` when committing
to also run strict Clippy and the all-feature suite with 1,000 property cases.
These full checks always run in CI, independently of the hook.

PR CI checks formatting, strict Clippy, and the full debug and release library
suites with 10,000 property cases. Release testing matters because integer
overflow can behave differently between build modes. It also tests default
features (including README examples), no-default-feature library code, and
`crypto,fast,serde` without ISO, using 256 cases for these feature checks.

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
PROPTEST_CASES=10000 cargo test --all-features
PROPTEST_CASES=10000 cargo test --release --all-features --lib
cargo test
cargo test --no-default-features --lib
cargo test --no-default-features --features crypto,fast,serde --lib
```

Beta and nightly compatibility runs happen weekly and on manual dispatch of the
Rust workflow. Nightly failures remain advisory. Scheduled runs start after the
workflow reaches the default branch. Doctests for optional APIs must be feature
gated and exercised in the all-feature run, rather than ignored.

Prefer fixed boundary tables for simple predicates and zero/one identities.
Use properties with independent oracles for allocation, precision, formatting,
and identity compatibility. Preserve regressions for known bugs, error
precedence, custom exponents, and numeric limits; fewer test functions should
not mean weaker contracts.

Run Criterion benchmarks for performance changes with
`cargo bench --all-features --bench benchmark`. Benchmarks are not a timing gate
on shared CI runners. Keep exploratory dependency audits separate from the
normal library suite.
