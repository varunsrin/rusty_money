# Formal contracts

The **Kani contracts** check runs on every pull request and push to `master`.
It verifies five contracts against the actual Rust implementation using Kani
0.68.0 / CBMC 6.11.0. Any failed proof, insufficient loop bound, timeout, or tool
error fails the check. GitHub branch protection can require this check separately.

| Contract | Input domain | Claim |
| --- | --- | --- |
| Addition | All pairs of i64 amounts, USD | Exact i128 reference sum when representable; Overflow otherwise; currency preserved |
| Subtraction | All pairs of i64 amounts, USD | Exact i128 reference difference when representable; Overflow otherwise; currency preserved |
| Negation | All i64 amounts, USD | Exact i128 reference negation when representable; Overflow otherwise; currency preserved |
| Currency mismatch | All pairs of i64 amounts, USD/EUR | CurrencyMismatch takes precedence over arithmetic overflow |
| Large-scale construction | All i64 amounts, all u32 exponents >= 19 | Zero succeeds as zero; every nonzero amount returns Overflow |

The loop-unwinding bound of 40 accommodates exponentiation and currency
byte comparisons. Unwinding assertions remain enabled, so insufficient bounds
cannot produce a passing proof. The bound does not limit the integer inputs.

## Run locally

Install Rust through rustup, Python 3, and the pinned verifier:

```sh
cargo install --locked kani-verifier --version 0.68.0
cargo kani setup
cargo fetch --manifest-path verification/Cargo.toml --locked
python3 verification/run.py --revision HEAD \
  --harness fast_add_contract --harness fast_sub_contract \
  --harness fast_neg_contract --harness fast_mismatch_contract \
  --harness major_large_scale_contract \
  --timeout 120 --output /tmp/rusty-money-proofs
```

Use a new output directory for each run. The runner archives the selected Git
commit into a temporary directory and copies these harnesses alongside it.
It checks committed production code; uncommitted production edits are excluded.
The default PR checkout is GitHub's proposed merge commit, so CI verifies the
code that would be merged. Manual workflow dispatch also accepts a historical
revision. Neither mode changes the working checkout.

The separate Cargo package pins rust_decimal 1.43.0, including its transitive
lockfile, for reproducible comparisons. Dependency updates require updating this
verification package as well. No production functions or dependency bodies are
stubbed. Ordinary `cargo test` does not run these proofs; Kani activates `cfg(kani)`.

Logs, exact revision, harness source and SHA-256, verifier/platform versions,
commands, timings, and the dependency lock are saved in the output directory.
CI uploads them as `verification-evidence`, including when a check fails.
Temporary source archives are retained locally for inspection.

## Evidence and limits

Running identical contracts against the pre-fix source
`2e91f477e4b7b751a8e43cc74f12256c62f2d4e0` and the constructor fix
`9761d53150ecf38c5e02de51b17aec501c753eb8` found one historical defect category:
the large-scale constructor contract fails before the fix and passes afterward.
The other four contracts pass on both revisions.

Kani generated exponent `2147483648` and amount `i64::MIN` without these values
being supplied to the harness. Native replay confirmed that the old constructor
panics in debug and returns zero in release; the fixed constructor returns
Overflow. A weaker panic-only Kani check missed this defect, which is why the
check asserts the mathematical result/error contract.

This is a retrospective experiment with known bug categories, not blind discovery
or verification of the entire library. General Decimal arithmetic and allocation
proof attempts timed out and are excluded from this gate. These contracts do not
cover other custom currency implementations, formatting, parsing, serde, general
multiplication/division, small-exponent construction, or resource exhaustion.
Proofs remain conditional on Kani's Rust model, compiler, solver, and target.
