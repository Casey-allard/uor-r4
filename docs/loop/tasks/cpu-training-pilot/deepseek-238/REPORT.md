# REPORT: cpu-training-pilot, deepseek-238 (D84 centroid cache evaluator)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `2b08d79c4`.

## What did not work, first

- The first centroid evaluator ran in `uor-r4-float-train`, outside the named D84
  CPU module. This unit moves evaluation into `uor-r4-cpu-train`.
- Its first matched control rotated across all eligible observations, so a rare or
  singleton target could receive another target's query distribution. The replacement
  groups eligible target IDs by exact observation count, cycles each non-singleton group
  once, keeps singleton targets on their own centroid in both arms, and faults if the
  resulting shuffle or table is vacuous.
- Its power, pooled identity and pooled comparison existed only in prose. The new
  evaluator enforces fitted-row/observation floors, computes split and pooled
  `(position,target)` identities, and emits the pooled paired bar.
- The first evaluator loaded a 27.4M-row TRAIN query file into memory. The new fit streams
  the file once, updating the SHA-256 and both candidate/control sums in the same pass.
- A fresh adversarial review found that the diagnostic API could omit or lower registered
  gates, skipped Q12 validation on ineligible TRAIN rows, and did not inspect every
  manifest/checkpoint/run-config hash. The decision binary now uses a separately pinned
  registered profile; the aggregate rejects diagnostic reports and recomputes the power
  and bar fields from their counts.
- A second exact-head review found that the aggregate still trusted derived `top8_holds`
  and paired `passes` booleans, and that its worker test never crossed the 64-row Rayon
  block boundary. The aggregate now reconciles panel and pooled counts, recomputes both
  paired tests from their raw fields, verifies actual TRAIN/panel/cache pins, and requires
  one explicit seed-1 and one seed-2 report. Worker and top-8 tests now span multiple
  blocks.
- A third exact-head review found that the aggregate still checked only the shape of each
  panel row identity and trusted the derived exact-identity flag. It now rolls the four
  panel identities into the registered pooled identity, checks `exact` against
  `changed_from_frozen`, sums per-panel paired evidence, and enforces the paired-count
  algebra.
- A fourth exact-head review found two fail-open aggregate cases: absent panel decision
  hashes compared equal as JSON nulls, and paired discordant counts were not bounded by
  the panel row count. Every identity and arm decision hash must now be valid SHA-256,
  and paired disagreements must fit within the scored rows. Negative tests reproduce both
  malformed reports.
- The current main CPU crate failed to compile its arbitration binary because two enum
  arms were absent from one match. That baseline repair is isolated in its own commit.

## What changed

- Added `query_cache_eval` to `uor-r4-cpu-train`.
- Added `cpu-query-cache-evaluate`.
- Added `cpu-query-cache-aggregate` for the two-seed control/power/bar precedence.
- Verifies the cache seal, table, every declared split, every checkpoint file and the run
  config. The registered binary pins the n3S panel names, input hashes, context, width,
  row identity, row count and power floors; every TRAIN query value, including an
  ineligible row, must be on the exact Q12 `1/4096` grid.
- Converts queries and tables to exact i16 Q12 and evaluates whole-table L1 in ordered
  64-query Rayon blocks.
- Streams TRAIN into target and exact-observation-count-stratified control sums, then
  evaluates original, centroid and control tables on held-out rows. The report names the
  control strategy and records its strata and singleton-target counts.
- Reports per-panel and pooled counts, pooled identities, power, candidate/control paired
  tests, top-8 preservation and deterministic decision hashes. It also checks the
  registered TRAIN token and mask hashes and reports split contexts.
- The two-seed aggregate accepts only `n3s-registered-v1` reports, pins their shared
  evaluation definition, requires one seed-1 and one seed-2 report, and recomputes power,
  panel/pooled totals, pooled identity, top-8, both paired tests, bar and status from the
  raw fields instead of trusting derived booleans.
- Added focused tests for exact Q12 conversion/rounding and top-8 ties, an exact-count
  non-vacuous shuffle with a self-mapped singleton stratum, ineligible-row Q12 rejection,
  registered gates, every clean and tampered manifest/checkpoint/run-config hash, split
  and pooled identity, power, multi-block top-8 membership, one-versus-two worker
  determinism across multiple blocks, malformed decision hashes, impossible paired counts,
  and false pooled-identity, exact-flag, top-8 and paired-count aggregate reports.

No GPU scorer, trainer, serving path or model artifact changed.

## Measurements

Nothing model-level was measured. The worker-count test compares one and two Rayon
threads over three query blocks after removing wall time and worker count; every semantic
field is equal.

The new binary is the D84 evaluator for the separate n3S query-cache experiment. Its
hardware and full run resource ledger belong to that experiment sheet; no real cache was
available at this head. Local evaluation must pass
`--checkpoint-root <extracted-step-directory>` when the capture-time checkpoint path is
not available.

## Checks run at the head

- `cargo test -p uor-r4-cpu-train --locked --offline --no-fail-fast`:
  **437 library tests passed, 4 ignored; all binary tests passed**.
- Focused `query_cache_eval` tests: **12 passed**.
- `cargo check -p uor-r4-cpu-train --bin cpu-query-cache-evaluate --locked --offline`:
  passed.
- Scoped strict Clippy, with only the four pre-existing crate lint categories allowed
  (`type_complexity`, `needless_borrow`, `cloned_ref_to_slice_refs`,
  `unnecessary_min_or_max`): passed.
- D23/D25/D32 ratchets: passed.
- Rustfmt on the evaluator and both query-cache binaries, plus `git diff --check`: passed.
- Full harness checks are run before delivery.

## Skills used

No system-provided Codex skill applied. D84 and repository task rules were used directly.

## Still open

The n3S GPU capture must use the merged D84 binary pin. The one-model sheet must be
updated from the float-crate evaluator to `cpu-query-cache-evaluate`, including the
extracted checkpoint root, then receive a fresh non-author FIT and D67 review. No real
centroid panel has been opened.

## Plain summary

TRAIN has 27.4 million rows, so loading all captured queries into memory was too large and
the old matcher treated some labels differently from their control. This CPU module now
streams and validates TRAIN, moves every eligible label in an exact-count-preserving
control while leaving singleton strata on their own centroid, enforces the registered
power and row-identity gates, and recomputes both raw-count pair tests before combining
one explicit seed-1 and seed-2 report in Rust. The GPU capture still has to run before any
model result exists.
