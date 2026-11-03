# ORDER: cpu-training-pilot, deepseek-81

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `de9cfc84a`.

## Goal

Run the required pivot after three bounded CPU-training configurations left
held-out ranking unchanged. Add an opt-in BVG-K block update to the exact tied
token-table trainer, then test whether moving four coordinates per accepted
sample produces a replicated panel gain where one-coordinate rank-margin and
parameter-only updates did not.

The pivot decision is explicit: if the pre-registered block comparison fails on
both seeds, stop the tied-table/parameter-update branch and preserve the
negative. If it passes, keep full-corpus training and panel controls as the
next unit.

## Cancel probe

1. **Is another table-block mechanism already built?** No. The published
   trainer mutates one table coordinate; the parameter counterfactual mutates
   non-token groups. The public 1/16/64-step runs changed one coordinate per
   attempt and did not move held-out ranking.
2. **Is this only a repeat with a larger step?** No. The new arm selects and
   commits up to four distinct interior coordinates as one artifact patch after
   exact joint-margin and admission checks; its matched one-coordinate arm uses
   the same samples, updates, seed and panel.
3. **Is the source data present?** Yes. The pinned public FBS seeds and public
   masked corpus used by deepseek-2, -3, -9 and -11 are present locally with
   recorded hashes.

Not cancelled.

## Files in scope

- `crates/uor-r4-cpu-train/src/train.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-corpus-driver.rs`
- `docs/harness/prep/cpu-training-pilot-table-bvgk-pivot-2026-11-03.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-81/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/cpu-training-program.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Required design

- Add `coordinates_per_step` to `RankMarginConfig`, default 1, range 1..=64.
- A step ranks distinct interior coordinates by the target and hardest-rival
  L1 distance sum, proposes a move for each of the first K, patches only the
  tied writer table, reparses the complete artifact, and accepts the block only
  when the exact changed-forward margin falls and target admission is retained.
- Bind K into checkpoint identity. Resume old version-1 checkpoints with K=1;
  refuse version-2 checkpoints with a different K.
- Expose K in `cpu-corpus-driver` without changing existing defaults.
- Prove K=1 is byte-identical to the current path on a fixture, K=4 accepts a
  strict joint improvement, malformed/out-of-range K is refused, and checkpoint
  resume preserves K and artifact bytes.

## Done when

1. The focused unit tests, full CPU release library and driver tests pass.
2. A FIT sheet is written before the compute and fixes two seeds, matched
   zero-update/K=1/K=4 arms, the public 1,024-row document-disjoint panel,
   admission and exact-export gates, a ranking gain no narrower than the
   measured two-seed spread, and stop rules.
3. Six runs report baseline, K=1 and K=4 for both public seeds, with exact
   CPU/serving parity, target admission, top-1/top-8, wall time and RSS.
4. The notebook and roadmap record a replicated gain or a pivot negative; a
   negative stops further one-coordinate/table-only tuning on this line.
5. A fresh exact-head review approves the PR.

## Stop and ask if

- K=4 cannot preserve exact artifact parsing or target admission.
- The comparison exceeds 30 minutes or $0 external cost.
- An unexpected source/main collision touches Claude's active branch.
