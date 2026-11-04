# ORDER: cpu-training-pilot, deepseek-124

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `2326bece3`.

## Goal

Close the next CPU writeback gap on the public 19-table M1 trunk. The CPU path
can read and audit this artifact but cannot train any of its 15 post-rotation
tables. Add exact additive sufficient-statistic updates for the trunk's unigram
and 12 sparse count tables, export a new 19-table artifact, and prove document-
disjoint evaluation parity through the independent CPU reader and integer
serving.

This is D84 host/training infrastructure. The bounded public run is a mechanism
test, not a quality or GPU-qualification claim.

## Design

- Extend `CpuDiscreteTrunkReader` to derive one exact `TrunkTrainRow` from a
  teacher-forced prefix and target using the same read, recurrence and query
  codes as normal CPU execution.
- Add a CPU-side exact accumulator over the public `TrunkTables` fields.
  Increment `unigram_counts` and all 12 sparse count tables with checked counts;
  recompute `unigram_best_global` by the existing count-descending/token-
  ascending rule.
- Export through `uor_r4_discrete::trunk::export_trunk_d11`, re-import, and
  require exact equality of the snapshot, recurrence table, normalization
  frames and all 19 decoded tables.
- Preserve codebook, routes, gates, source identity and artifact shape. Only the
  count-table payloads may change.
- Add `cpu-trunk-count-driver` for a bounded public run:
  - deterministic document-disjoint train/evaluation split;
  - no-update, exact additive-update and matched shuffled-target-update arms;
  - the same held-out rows for every arm;
  - report served decision changes, admitted/top-1/top-8 readings, exact
    CPU/integer parity, artifact/table hashes, timing and resource use.

## Pre-registration

Write `docs/harness/prep/cpu-training-pilot-trunk-count-2026-11-04.md` before
public compute. The primary result is writeback correctness rather than a
capability gain:

- both updated artifacts re-import and retain 19 tables;
- unchanged semantic sections are byte-identical or decode-equal as specified;
- every count increment equals the independently tallied training rows;
- all held-out rows preserve zero query/token/admitted-cell/top-8 parity;
- exact update reconstructs deterministically, the shuffled control changes
  target assignments and differs from exact, and a seed-1 repeat matches every
  non-timing field.

Capability direction is descriptive only. A gain or loss closes no mechanism;
this unit establishes that the public full trunk can be trained and written
back exactly on CPU.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-trunk-count-2026-11-04.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-124/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. Finite tests cover exact row derivation, count accumulation, global unigram
   tie behavior, target/query validation, deterministic shuffled targets,
   artifact round-trip and malformed input refusal.
2. FIT is recorded before public compute.
3. The public 19-table artifact is updated, re-imported and evaluated on
   document-disjoint rows with zero CPU/serving mismatches for every arm.
4. The report proves which bytes/tables changed and records deterministic
   replay, hashes, timing, RSS and bounded storage.
5. Full CPU release tests, strict no-deps Clippy, formatting, report, roadmap,
   catalog, prep and loop gates pass, followed by fresh exact-head review,
   protected merge and fresh-main verification.

## Stop

Stop on source/artifact drift, a semantic section change, count overflow or
mismatch, nondeterministic serialization, a loss of CPU/serving parity, or any
need to edit Claude-owned float-training or integer serving files.
