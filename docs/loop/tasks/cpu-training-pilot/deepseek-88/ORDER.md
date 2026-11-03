# ORDER: cpu-training-pilot, deepseek-88 (discrete K1/K4 corpus comparison)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `498e36574`.

## Goal

Drive the exact discrete proposal/apply learner at K1 and K4 over the public
four-table corpus, then run a FIT-reviewed two-seed comparison against a fresh
baseline. This is a different learning mechanism from the exhausted
rank-margin and fixed parameter-group comparisons.

## Cancel probe

The public Stage-T four-table artifacts are anonymously available and hash to
`fc31bcc7…39ca` and `a75ea7f5…12a`. The checkpoint driver already supports a
coordinate block and exact joint BVG-K apply, but `cpu-discrete-corpus-driver`
hardcodes K1, rejects K>1 on resume, scores/applies only `indices[0]`, and does
not hold out or report an evaluation panel. The unit stands.

## Files in scope

- `crates/uor-r4-cpu-train/src/discrete_route.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-discrete-corpus-driver.rs`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `docs/harness/prep/cpu-training-pilot-discrete-k1-k4-2026-11-03.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-88/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. The driver accepts a bound coordinate-block setting, defaults to K1 and
   remains backward-compatible.
2. The checkpoint schedule/binding records K and refuses resume under a
   different K.
3. A K-block scores every member, computes exact joint loss after all proposed
   moves, uses the existing BVG-K decision/apply path, and reports accepted
   block sizes and committed moves.
4. Focused tests cover K1 byte compatibility, K4 joint apply, resume and
   evaluation/export parity.
5. A FIT-reviewed design fixes two artifact seeds, baseline/K1/K4 arms,
   exact admission/export bars and a ranking bar before compute.
6. Six bounded runs complete or stop under the registered rule; every negative
   and instrument fault is retained.
7. REPORT, notebook, program, roadmap and evidence checksums are complete at
   the exact reviewed head.

## Stop and ask if

The public artifacts do not parse through the four-table CPU model, the joint
block cannot be evaluated exactly, or a run would exceed 45 minutes without a
new ORDER.
