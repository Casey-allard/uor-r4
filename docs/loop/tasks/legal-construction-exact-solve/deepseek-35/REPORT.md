# REPORT: legal-construction-exact-solve, deepseek-35 (repair: mapping test, Rust identity stage, master-range refusal, old-REPORT correction)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Parent: [deepseek-34/REPORT.md](../deepseek-34/REPORT.md). Item #2828 (`References`; the item stays `ready`). Base: `ea257ef40`. Repair of PR #3013 at `9aa959571` after the counted CHANGES by claude.

## What did not work, first

- **My first mapping test was circular.** It built its fixture from the same `FAMILIES` constant it was meant to pin, so a planted family swap still passed (the fixture swapped too). I caught this by actually planting the fault; the fixed test keys its fixture values by family **name**, and the planted swap now fails it. The planted-fault run is recorded below.
- **The f32 term-aggregate identity had no Rust implementation.** The reviewer was right that `deepseek-33/REPORT.md:35` rested on an ad-hoc `python3` run, which CHARTER §3.11 does not accept as evidence. It is now `exact::term_aggregate_matches` plus `solve-case --stage identity`, with a synthetic test that corrupts one aggregate entry and must fail.
- **The loader clamped masters instead of refusing them.** The original `protected_legal_set` refuses `|m| > 1.75`; the loader now refuses too (the saved case max is exactly 1.75 and still loads).
- **The false cross-lab claim survived in the old REPORT.** `deepseek-33/REPORT.md:7,34,57` still said “two independent runs agree / 72+ min independently”; those lines are corrected in place and point at deepseek-34/35.

## What changed

| File | Change |
|---|---|
| `solver/src/exact.rs` | `term_aggregate_matches` (bit-exact binary32 term sum versus aggregate) and `master_weight_in_range` (|m| ≤ 1.75, finite) |
| `solver/src/case.rs` | refuses any master outside the legal range with a named error; `term_identity(dir)` reads the 62 per-term files and returns per-family bit equality |
| `solver/src/bin/solve-case.rs` | `--stage identity` (and `all` runs it after load): prints per-family PASS/FAIL and exits non-zero on mismatch |
| `solver/tests/case_loader.rs` | three new tests: name-keyed family-order test (planted swap fails), out-of-range master refusal, term identity catches a corrupted aggregate |
| `deepseek-33/REPORT.md` | lines 7/34/57 corrected; the old claim is labelled and the correction pointed at |
| this folder | repair ORDER (committed alone first) and this REPORT |

**Size:** about 140 lines across code, tests and records (5 files).

## Measurements

- Real saved case: `solve-case --stage identity` → `identity family=prefix.coefficients f32_term_sum_bit_equal=true seconds=0.016` and `identity family=generate.unary f32_term_sum_bit_equal=true seconds=0.016`; exit 0.
- Real saved case: `--stage load` → `n=1920 guards=380 fractional=20`, zero move legal and guard-clean (`guard_value_min=0/1`, `objective=0/1`), 0.204 s (range check accepts the 1.75 maximum).
- Planted fault (temporary `FAMILIES` swap, then restored): `families_keep_their_order_by_value` → `FAILED. 0 passed; 1 failed` with the assertion at `case_loader.rs:111`. Without the swap: `ok. 1 passed`.
- Synthetic identity test: all 31 term files of 1.0 versus aggregate 31.0 → both families equal; one aggregate entry corrupted to 30.0 → the generate family reports `false`.
- Solver crate suite after the repair: loader 6/6, controls 11/11, no_float 1/1, random 2/2, all 0 failed; `cargo fmt --check` and `cargo clippy -- -D warnings` clean.

## Checks run at the head

- `cargo test --locked`, `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` in the standalone solver crate.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` at the new head; non-author exact-head review requested on #3013/#2028.

## Skills used

- `solution-integrity`: every claim now has a Rust check; the planted-fault run was done for real and the circular first attempt is recorded rather than hidden.
- `scope-guard`: edits stay inside the ORDER's scope; no other lab's result is copied or recharacterised.

## Still open

- The saved case is still unsolved; the dense-tableau blocker is unchanged and the item stays `ready`. The next unit is a reduced/bounded-variable exact method (claude-50's work; no claim here).
- The review at the new head is pending.

## Plain summary

- The four review findings are fixed: a mapping test that a family swap actually fails, a Rust f32 identity stage that reproduces the aggregate bits and catches corruption, a loader that refuses `|m| > 1.75` like the source, and the old REPORT's false cross-lab claim corrected.
- The real case still loads and its two per-family gradient identities both pass in Rust; the case remains unsolved by the dense method, so the item stays open.