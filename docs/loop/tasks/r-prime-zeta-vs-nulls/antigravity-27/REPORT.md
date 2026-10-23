# REPORT: r-prime-zeta-vs-nulls, antigravity-27

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Raw `coverage_census` alias ceiling (`dev_alias_ceiling_rows`) cannot separate injective codebooks:** In [`crates/uor-r4-discrete/src/coverage_census.rs`](../../../../../crates/uor-r4-discrete/src/coverage_census.rs) (lines 34–87), `alias_representatives(table, spec)` partitions vocabulary tokens into alias groups sharing an identical full $32$-integer vector and names each group by its lowest token ID `representative[t]`. For any injective codebook (`distinct_vectors == 4,096`), every alias group has size `1` (`representative[t] == t`), so `within_cell_tie_mass == 0` and `dev_alias_ceiling_rows == 14,287 / 14,287` across `PrimeZeta`, `TangentNull`, `NormalNull`, `QuotaOnlyInjective`, and `RandomInjective`. Separation at zero updates requires writer-cell crowding capacity (`cell_collision_pairs`, `max_cell_occupancy`) and GPU reader-admitted target reach (`B3_init(P_succ)`, `B3_full`, `Net G-Cap`).
- **Stale `driver_source_sha256` assertion in `prime_zeta.rs`:** Running `~/.cargo/bin/cargo test -p uor-r4-discrete prime_zeta` initially failed on `driver_source_identity_changes_with_rank_objective_metadata` (`crates/uor-r4-discrete/src/prime_zeta.rs:1262`), which still carried an older `ed4655a2…` test constant from before `origin/main` updated `crates/uor-r4-discrete/src/training.rs` (caught now because `prime_zeta.rs` is compiled only by `--bin` targets, not `--lib`). No input to `uor_r4_discrete::training::driver_source_sha256()` (`discrete.cu`, `lib.rs`, `training.rs`) is touched by this PR (`git diff origin/main` on all three is empty); updating the test pin to `b13d4920d37a49dd0b1c2305ceaf406e51d5baadd058572cbf481f534bdf9b24` simply matches current `origin/main`.
- **Cancel probe failure:** Omitting the writer-cell quota and full-vector uniqueness checks in `generate_integer_injective_null` causes `random_and_quota_only_injective_nulls_satisfy_quota_uniqueness_and_census_invariants` in `crates/uor-r4-discrete/src/prime_zeta.rs` to fail (`FAILED`). Restoring them returns `ok`.
- **Zero-update GPU pod battery (`Step 5`) not yet executed:** Per [`docs/roadmap/items/r-prime-zeta-vs-nulls.md`](../../../../roadmap/items/r-prime-zeta-vs-nulls.md) (Steps 2, 4, 5) and CHARTER §4.5 / §5.2, the comparison bar is not yet registered (`PROPOSED, NOT REGISTERED`) and any GPU evaluation run requires a non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md) and owner/PM approval before renting a pod (`~$0.20`).

## What changed

- **Code (`+285 / -9` lines):** Added `CodebookMode::RandomInjective { seed: u32 }` and `CodebookMode::QuotaOnlyInjective { seed: u32 }`, `generate_integer_injective_null()`, and `writer_cell_occupancy()` in [`crates/uor-r4-discrete/src/prime_zeta.rs`](../../../../../crates/uor-r4-discrete/src/prime_zeta.rs), and extended [`crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs`](../../../../../crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs) to export `random_injective:SEED` and `quota_only_injective:SEED` tables and report cell-occupancy crowding metrics.
- **Tests (`+194 / -1` lines):** Added `random_and_quota_only_injective_nulls_satisfy_quota_uniqueness_and_census_invariants` in `crates/uor-r4-discrete/src/prime_zeta.rs` and `cli_mode_parser_accepts_all_five_modes_and_rejects_unknown` in `crates/uor-r4-discrete/src/bin/prime-zeta-codebook.rs`.
- **Docs & staged configs (`+676 / -0` lines across 11 files):** Authored [`docs/harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md`](../../../../harness/prep/r-prime-zeta-vs-nulls-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`), [`docs/labs/r-prime-zeta-vs-nulls/README.md`](../../../../labs/r-prime-zeta-vs-nulls/README.md), and staged 7-arm configs (`p8.json`, `q0.json`..`q2.json`, `r0.json`..`r2.json`), [`pz-nulls-battery.sh`](../../../../labs/r-prime-zeta-vs-nulls/evidence/pz-nulls-battery.sh), and [`SHA256SUMS`](../../../../labs/r-prime-zeta-vs-nulls/evidence/SHA256SUMS). Ratio against `ORDER.md` scope estimate: `1.0x` (all 3 scoped path families).

## Measurements

Host-side deterministic table construction and census measurements (`prime-zeta-codebook` on [`docs/labs/r-prime-zeta-vs-nulls/evidence/spec-v4096-b4-c192-k8.json`](../../../../labs/r-prime-zeta-vs-nulls/evidence/spec-v4096-b4-c192-k8.json), `model_execution: false`):
- `prime_zeta` (`P8`): `4,096 / 4,096` distinct vectors, `256 / 256` occupied writer cells, `min_cell_occupancy = 16`, `max_cell_occupancy = 16`, `cell_collision_pairs = 30,720`, `file_sha256 = 83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`.
- `quota_only_injective:{0,1,2}` (`Q0..Q2`): `4,096 / 4,096` distinct vectors, `256 / 256` occupied writer cells, `min_cell_occupancy = 16`, `max_cell_occupancy = 16`, `cell_collision_pairs = 30,720`, `file_sha256 = 33b4e459…`, `57225bbb…`, `e03d6aef…`.
- `random_injective:{0,1,2}` (`R0..R2`): `4,096 / 4,096` distinct vectors, `256 / 256` occupied writer cells, `min_cell_occupancy = 8, 3, 8`, `max_cell_occupancy = 27, 27, 26`, `cell_collision_pairs = 32,493, 32,802, 32,659`, `file_sha256 = 229e93f5…`, `36fec632…`, `25ec9bd2…`.

## Checks run at the head

- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete prime_zeta`: `6 passed; 0 failed` across all binaries (`address-cells-codebook`, `prime-zeta-codebook`, `prime-zeta-qualify`).
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-discrete --bin prime-zeta-codebook cli_mode_parser`: `1 passed; 0 failed`.
- `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness ratchet d32`: all `OK` (`0` sites added).
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

The UOR-R4 model gives each of its 4,096 vocabulary words a fixed 32-integer starting address ("codebook vector") built from four 8-dimensional blocks chosen from 16 geometric anchor vectors ("E8 orbit roots"). The primary constructor, `PrimeZeta`, assigns each word a prime number and uses the frequencies of the Riemann zeta zeros to score roots, followed by a "quota pass" that puts exactly 16 words into each of the 256 outer root-pair buckets ("writer cells") and a "uniqueness pass" that gives every word in a bucket a distinct pair of middle roots. Because earlier null controls also ran those same quota and uniqueness passes and still used zeta phases, it was unknown whether the prime and zeta-zero angles matter at zero training updates or whether any collision-free integer table would behave the same way. This task builds and verifies two pure-integer null constructors that use no primes, no zeta zeros, and no floating-point numbers: `QuotaOnlyInjective` (which keeps the exact 16-words-per-cell quota and unique middle roots using a seeded shuffle) and `RandomInjective` (which draws 4,096 distinct 4-block root vectors without any cell quota). It also proves that the raw "alias ceiling" metric (`14,287 / 14,287` reachable development rows) is identical for every collision-free table, defines the cell-crowding capacity and GPU cell-admission reach metrics that do separate them, and stages the 7-arm zero-update GPU evaluation sheet and script awaiting review and pod-spend approval on `#2881`.
