# ORDER: exact final served-gain distance training, deepseek-97

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `c4c267c534d7160ac6d544c420c90249a1cddeba`.

## Goal

Add a new training-through-serving path for a full-width final gain. The
teacher-forced token trajectory fixes the recurrent state. Cache the exact
normalization immediately before `final_norm_gain`, then score candidate gain
codes by replaying only the final gain, fixed turn-memory movement in the
normalization-fine domain, Q12 conversion, writer decision and point fallback.

Run a materially new bounded CPU comparison on both public FBS seeds against
matched frozen and equal-move random controls. This is distance training, not
the failed rank-margin objective.

## Cancel probe (run first)

At base `c4c267c53`, `git grep` found the generic normalization group's
`final_norm_gain` candidate path in `counterfactual.rs`, but no
`CpuServedGainTrainer`, final-gain replay or gain-panel driver. The existing
normalization comparison was the failed rank-margin/selection configuration in
[deepseek-11](../deepseek-11/REPORT.md), so it does not test a distance
objective over the complete served writer decision. The task therefore stands
as a distinct mechanism/objective comparison and does not repeat that
configuration.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-gain-2026-11-03.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-97/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. The traced CPU FBS path exposes the exact pre-final-gain normalization
   without changing the public serving path.
2. Cached final-gain replay exactly matches full-forward evaluation for
   teacher-forced samples, including turn-memory movement and point fallback;
   it fails closed on artifacts whose Hamming readout depends on the changing
   query.
3. Candidate final-gain codes are evaluated by complete served target distance,
   with ordered parallel reduction and checked aggregate arithmetic.
4. A FIT-reviewed two-seed comparison with frozen and equal-move random
   controls completes under the registered cost and stop.
5. All accepted artifacts pass complete CPU/serving parity; failures and
   negative capability results are retained.
6. Focused and full tests, strict Clippy, formatting, evidence, roadmap,
   REPORT, loop checks and exact-head review pass.

## Stop if

- Cached and full-forward final-gain training differ.
- Any accepted artifact loses complete CPU/serving parity.
- The path requires float serving, dense matrix multiplication, softmax or
  squashing.
- The comparison repeats the rank-margin objective or lacks a matched control.
