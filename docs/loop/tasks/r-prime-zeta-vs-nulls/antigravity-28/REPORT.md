# REPORT: r-prime-zeta-vs-nulls, antigravity-28

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Broken self-link in `docs/labs/r-prime-zeta-vs-nulls/README.md:73`:** The staged notebook row at line 73 linked `../labs/r-prime-zeta-vs-nulls/README.md` (which is valid from `docs/research/notebook/m1-prime-zeta.md` after backfill, but resolves to `docs/labs/labs/r-prime-zeta-vs-nulls/README.md` from inside `docs/labs/r-prime-zeta-vs-nulls/README.md`). Running `uor-harness check --only links` on `409ae0ba8` failed with `1 error` (`docs.links`); changing the link target to `README.md` resolves it cleanly.
- **Unexplained `driver_source_sha256()` test pin update (`ed4655…` -> `b13d4920…` at `prime_zeta.rs:1262`):** Because `crates/uor-r4-discrete/src/prime_zeta.rs` is compiled via `#[path = "../prime_zeta.rs"]` inside the binary targets rather than `--lib`, its unit test `driver_source_identity_changes_with_rank_objective_metadata` had not been re-pinned when `origin/main` previously updated `training.rs`. Clarified in both [`antigravity-27/REPORT.md`](../antigravity-27/REPORT.md) and this report that `training::driver_source_sha256()` and all of its hashed source inputs (`discrete.cu`, `lib.rs`, `training.rs`) are 100% unchanged against `origin/main`.
- **Out-of-spec regime and `blocks >= 5` extra-block assignment in `generate_integer_injective_null`:** Without an entry guard, calling `QuotaOnlyInjective` with a vocabulary not divisible by `256` (or exceeding `256 * interior_cap`) or `RandomInjective` with `blocks < 3` or `vocabulary > 16^blocks / 2` would silently fall back to non-uniform cell quotas or lexicographic linear probing, and `blocks >= 5` assigned extra interior blocks via deterministic `(rank_in_cell + extra) & 0x0f`. Added strict entry guards and `splitmix64` Fisher–Yates per-cell permutations for `blocks >= 5` while keeping `B = 4` execution bit-identical.

## What changed

- **Docs (`+3 / -3` lines across 2 files):** Fixed the self-link at [`docs/labs/r-prime-zeta-vs-nulls/README.md`](../../../../labs/r-prime-zeta-vs-nulls/README.md) (`:73`) to `README.md` and clarified the `driver_source_sha256()` test-pin note in [`antigravity-27/REPORT.md`](../antigravity-27/REPORT.md).
- **Code & tests (`+118 / -41` lines in 1 file):** Updated `generate_integer_injective_null` and `random_and_quota_only_injective_nulls_satisfy_quota_uniqueness_and_census_invariants` in [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../../../../crates/uor-r4-discrete/src/prime_zeta.rs) to enforce `blocks in 3..=16`, uniform cell-quota capacity (`vocab % 256 == 0 && vocab / 256 <= interior_cap`), and `vocab <= 16^blocks / 2`, draw `blocks >= 5` extra interior blocks via `splitmix64` Fisher–Yates shuffles, remove the non-uniform fallback probe, and document that `phases_q29` and `spinors_s3_q30` carry deterministic integer XOR schema tags while the training battery reads only `root_indices` / `Snapshot.table`. Ratio against `ORDER.md` scope estimate: `1.0x`.

## Measurements

Host-side deterministic verification (`model_execution: false`):
- All `9 / 9` staged evidence files in [`docs/labs/r-prime-zeta-vs-nulls/evidence/SHA256SUMS`](../../../../labs/r-prime-zeta-vs-nulls/evidence/SHA256SUMS) (`p8.json`, `q0.json`..`q2.json`, `r0.json`..`r2.json`, `spec-v4096-b4-c192-k8.json`, `pz-nulls-battery.sh`) pass `shasum -c` unchanged (`q0.table_sha256 = 0098d4c3…`, `r0.table_sha256 = 35c8da62…`).
- `QuotaOnlyInjective` at `B = 5, V = 4,096` produces `4,096 / 4,096` unique 5-block root vectors with `[16; 256]` writer-cell occupancy.

## Checks run at the head

- `uor-harness check --only links`: `0 error`.
- `(cd docs/labs/r-prime-zeta-vs-nulls/evidence && shasum -c SHA256SUMS)`: `9 / 9 OK`.
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete prime_zeta`: `6 passed; 0 failed` across all binaries.
- `uor-harness loop report --check`: `PASS`.
- `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness loop gates --full`: `10 / 10 GREEN`.

## Skills used

- [`uor-research-architect`](../../../../../.agents/skills/uor-research-architect/SKILL.md)
- [`scope-guard`](file:///Users/casey.allard/.gemini/config/skills/scope-guard/SKILL.md)
- [`solution-integrity`](file:///Users/casey.allard/.gemini/config/skills/solution-integrity/SKILL.md)

## Still open

- Non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md), owner/PM confirmation of the comparison bar and `~$0.20` pod spend on [`#2881`](https://github.com/UOR-Foundation/uor-r4/issues/2881) (`6353762758`), execution of [`docs/labs/r-prime-zeta-vs-nulls/evidence/pz-nulls-battery.sh`](../../../../labs/r-prime-zeta-vs-nulls/evidence/pz-nulls-battery.sh), and recording the completed row in `docs/research/notebook/m1-prime-zeta.md`. References #2881.

## Plain summary

This repair resolves the broken relative link in the lab report (`docs/labs/r-prime-zeta-vs-nulls/README.md:73`) that caused the documentation link check to fail, documents why the existing `driver_source_sha256` unit-test constant in `prime_zeta.rs` was updated to match `origin/main` without changing any training-driver source files, and hardens the pure-integer null codebook constructors (`QuotaOnlyInjective` and `RandomInjective`) so they reject out-of-range table dimensions instead of silently relaxing cell-quota uniformity, while keeping all six exported `V = 4,096, B = 4` null tables bit-identical.
