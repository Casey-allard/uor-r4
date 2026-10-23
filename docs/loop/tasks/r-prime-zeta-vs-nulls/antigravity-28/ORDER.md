# ORDER: r-prime-zeta-vs-nulls, antigravity-28

Lab: antigravity. Item: [r-prime-zeta-vs-nulls](../../../../roadmap/items/r-prime-zeta-vs-nulls.md). Base: `409ae0ba8`.

## Goal

Repair all findings from [Claude's review (`#3014` comment `6353975097`)](https://github.com/UOR-Foundation/uor-r4/pull/3014#issuecomment-6353975097):
1. **Fix the broken self-link in [`docs/labs/r-prime-zeta-vs-nulls/README.md`](../../../../labs/r-prime-zeta-vs-nulls/README.md) (`:73`)** so `uor-harness check --only links` and `docs archive and links` CI pass with `0` errors.
2. **Document the re-pinned `driver_source_sha256()` test constant (`ed4655…` -> `b13d4920…` at `prime_zeta.rs:1262`)** in [`antigravity-27/REPORT.md`](../antigravity-27/REPORT.md) and [`antigravity-28/REPORT.md`](REPORT.md) so readers know the driver's sources are unchanged against `origin/main` and the edit only repaired a stale test pin on `main`.
3. **Tighten `generate_integer_injective_null` in [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../../../../crates/uor-r4-discrete/src/prime_zeta.rs):**
   - Refuse out-of-spec configurations where `QuotaOnlyInjective` cannot form uniform cell quotas (`vocab_size % 256 != 0` or `vocab_size > 256 * 16_usize.pow((blocks - 2) as u32)`) or where `RandomInjective` exceeds half the total tuple space (`vocab_size * 2 > total_tuples`), so the nulls are never silently run in an unfair or non-uniform regime;
   - Draw any extra interior blocks (`blocks >= 5`) via `splitmix64` Fisher–Yates permutation over the interior tuple index rather than deterministic `(rank + extra) & 0xf`, preserving all 6 exported `B = 4` tables (`q0`..`q2`, `r0`..`r2`) bit-identically;
   - Document on `generate_integer_injective_null` that `phases_q29` and `spinors_s3_q30` hold deterministic integer tags for JSON schema compatibility while the training battery reads only `root_indices` (`pinned_table_v1`).

## Cancel probe (run first)

`uor-harness check --only links` on `409ae0ba8` fails on `docs/labs/r-prime-zeta-vs-nulls/README.md:73` (`../labs/r-prime-zeta-vs-nulls/README.md`).

## Files in scope

- `crates/uor-r4-discrete/src/prime_zeta.rs`
- `docs/labs/r-prime-zeta-vs-nulls/README.md`
- `docs/loop/tasks/r-prime-zeta-vs-nulls/**`

## Done when

- `uor-harness check --only links` passes with `0` errors.
- `shasum -c docs/labs/r-prime-zeta-vs-nulls/evidence/SHA256SUMS` passes `9 / 9` unchanged.
- `prime_zeta` unit tests, `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside `scope` must be modified.
