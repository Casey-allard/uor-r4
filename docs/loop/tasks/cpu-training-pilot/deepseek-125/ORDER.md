# ORDER: cpu-training-pilot, deepseek-125

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `09c4f4abb`.

## Goal

Run the larger bounded 19-table count-training comparison left open by
deepseek-124. On the same fixed document-disjoint split, increase the additive
training dose from 128 to the driver's pre-existing 4,096-row ceiling and test
whether exact targets improve held-out top-1 over no update and a deterministic
shuffled-target control.

This is D84 CPU development research. It is not GPU qualification or a language
capability claim.

## Design

- Reuse the merged `cpu-trunk-count-driver` without changing model math.
- Keep the public seed-1 artifact, document split, evaluation stride 2 and 512
  held-out rows fixed.
- Select the first 4,096 training rows by the established deterministic order;
  do not tune the cutoff after observing results.
- Compare no update, exact additive counts and shuffled-target counts.
- Require independent full count-table reconstruction, unchanged semantic
  sections, exact CPU/integer parity and deterministic byte-identical repeats.

## Pre-registration

Write `docs/harness/prep/cpu-training-pilot-trunk-count-dose-2026-11-04.md`
before compute.

Primary bar on the 512 held-out rows:

- exact top-1 exceeds no update;
- exact top-1 exceeds shuffled targets;
- each paired gain is at least `max(2, ceil(2 * sqrt(b + c)))`;
- exact admitted and top-8 counts are not lower than either control.

All instrument checks must pass. A negative closes this fixed 4,096-row additive
count configuration, not other count schedules, table families or geometric
training.

## Files in scope

- `docs/harness/prep/cpu-training-pilot-trunk-count-dose-2026-11-04.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-125/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

No source change is expected. Stop if the merged driver needs model-math edits.

## Done when

1. FIT and pre-run checks bind the existing source, driver binary and fresh
   build/output roots.
2. Exact and shuffled runs update all count tables by exactly 4,096 entries,
   preserve semantic sections and pass independent full-table reconstruction.
3. All arms have zero query/cell/top-8/token mismatches over 512 held-out rows.
4. The paired bar is reported on both top-1 controls with admitted/top-8
   non-loss and deterministic repeat evidence.
5. Report, evidence, notebook/roadmap, all CPU/documentation/loop checks, fresh
   exact-head review, protected merge and fresh-main verification complete.

## Stop

Stop on source drift, a required code change, cap/RSS failure, a non-vacuous
control failure, count mismatch, CPU/serving divergence or evaluation-specific
selection. Do not run a larger dose after seeing this result in the same unit.
