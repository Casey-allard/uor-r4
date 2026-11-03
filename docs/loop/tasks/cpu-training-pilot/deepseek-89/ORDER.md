# ORDER: cpu-training-pilot, deepseek-89 (discrete joint-loss eligibility repair)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `498e36574`.

## Goal

Repair and re-evidence deepseek-88 after exact-head review found that joint loss
skipped a baseline-eligible row when the proposed block removed that row's
target eligibility.

## Finding

The first K1/K4 comparison used a six-run binary whose joint-loss loop skipped
rows that lost target rivals after the proposed block. The correct BVG-K rule
is to reject the joint candidate. A focused regression now pins that rule.

Seed 1 already terminated the registered experiment at K4 admission, so the
conclusion cannot create a positive. The six runs must still be rerun from the
corrected exact head and their receipts replaced.

## Repair

- Preserve the superseded six-run evidence in Git history.
- Reject a joint block when any baseline-eligible row loses eligibility.
- Rerun all six arms from the exact corrected binary in a fresh output tree.
- Regenerate comparison totals, report, notebook and evidence checksums.

## Scope

- `crates/uor-r4-cpu-train/src/discrete_route.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-88/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-89/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The eligible-row loss rule is implemented and regression-tested.
2. All six receipts come from the corrected exact-head binary.
3. The binding, ranking and telemetry readings are regenerated.
4. A fresh exact-head review approves the corrected PR.

## Stop if

The corrected binary changes a non-instrument result without a clear cause, or
any evidence input differs from the FIT sheet.
