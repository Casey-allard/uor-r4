# ORDER: cpu-training-pilot, deepseek-145 (trajectory fallback-accounting repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Parent: [deepseek-144](../deepseek-144/ORDER.md). Base: `d5d173073`.

## Goal

Repair the trajectory readout's full-generation accounting without changing the
registered first-order trajectory construction, thresholds, split or stop rule.

The fresh review at `28fda2f02` found that `readout_fallback_accounting`
compared `ContentReadoutReading.decisions`, which counts only decisions aligned
with reference positions, against readout plus fallback counters that cover
every generated decision. Prior sealed generation evidence demonstrates the
populations differ: 973 reference-aligned decisions against 1,648 total
generated decisions. A normal cap-stopped trajectory would therefore fail for
an instrument reason.

## Repair

- Count total generated decisions explicitly in `ContentReadoutReading`.
- Require generated decisions to equal readout plus exact-writer fallback.
- Add a regression that evaluates a capped generated trajectory whose reference
  horizon is shorter than the generation.
- Preserve sealed attempt 1 unchanged.
- Rebuild and rerun the registered seed-1 `--select-only --trajectory-readout`
  preflight from the corrected source in a fresh attempt-2 evidence root.
- Update the REPORT and resource ledger to bind the corrected source and fresh
  evidence. The expected mechanism result is unchanged: code top-1 300/894,
  trajectory 297/894, so the preflight must still stop before seed 2 and
  generation.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `.gitattributes`
- `docs/harness/prep/cpu-training-pilot-trajectory-readout-2026-11-04.md`
- `docs/labs/cpu-training-pilot-trajectory-readout-2026-11-04/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-145/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

No integer-session, serving, CUDA, float-training or Claude-owned source
changes.

## Done when

1. The accounting predicate uses a total-generated-decision population, and a
   capped-generation regression fails with the old comparison and passes with
   the repair.
2. Attempt 1 remains byte-identical to its original seal.
3. Corrected-source attempt 2 reproduces the registered seed-1 preflight values
   and preserves `NOT_RUN` for seed 2, generation and the worker repeat.
4. REPORT, notebook and roadmap classify the unchanged bounded negative, and
   the evidence manifest, resource ledger and source pin agree.
5. Focused tests, build controls, wording/formats, roadmap, D23/D25/D32,
   `uor-harness check`, report/local PR checks and a fresh exact-head review
   pass; protected merge and fresh-main verification complete.

## Stop rules

Stop on source/input/split drift, any change to the registered mechanism or
bar, a preflight value that differs, evidence-root reuse, a resource breach, or
any need to change integer serving, CUDA, float training or Claude-owned source.
