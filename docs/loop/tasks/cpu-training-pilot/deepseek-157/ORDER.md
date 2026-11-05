# ORDER: cpu-training-pilot, deepseek-157 (generated copy stop gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `ed9b5b9aa`.

## Goal

Train a geometric stop gate on the actual longest-first copy-generated train
trajectories and test it unchanged on the held-out dev trajectories. This is
the first deployable-surrogate follow-up to deepseek-155's 11/13 oracle-stop
ceiling.

This is D84 CPU development research. It changes no serving, CUDA,
float-training or Claude-owned source.

## Fixed construction

For each sealed deepseek-153 longest-first trace, replay the prompt and
selected tokens through the exact `CpuFbsForward` writer path. Capture every
256-coordinate Q12 writer query from generation step zero through the
normalized reference length, inclusive.

Label the decision at step `reference_len` terminal and every earlier decision
nonterminal. Fit the existing discrete stop-signature mechanism with:

- `256` sign bits;
- `8` terminal and `8` nonterminal Hamming prototypes;
- nearest-prototype score
  `nearest_nonterminal - nearest_terminal`;
- a threshold selected only on train rows to maximize terminal stops subject to
  at most `floor(nonterminal_rows / 100)` training false stops.

Fit separately for each artifact seed on train panel 2711. Evaluate unchanged
on dev panel 2712. During evaluation, the first gate firing emits BOS;
otherwise the sealed longest-first token is fed forward.

Controls are:

- the fitted true gate;
- a class-preserving shuffled-label gate fitted on the same train rows;
- no gate.

The oracle ceiling from deepseek-155 is 11 exact train replies and 13 exact dev
replies per artifact seed.

## Bar

The candidate passes only if, for both artifact seeds:

- the true gate stops at least half the oracle ceiling: **6 train** and
  **7 dev** exact replies;
- true exact replies beat both shuffled-gate and no-gate exact replies by the
  paired two-sigma rule;
- premature BOS stops are at most **1%** of reachable replies: 4 train and 4
  dev;
- every fired decision has a recorded terminal/nonterminal row and unchanged
  ungated token sequence.

## Stop rules

Stop on source report, token, mask, artifact or hash drift; a trace that cannot
be replayed; non-monotonic decisions; class absence; failure to fit a nonzero
training gate under the 1% limit; nondeterminism; or a vacuous shuffled gate.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-157/**`
- `docs/labs/cpu-training-pilot-copy-generated-stop-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. Generated-trace replay, terminal labeling, fit/evaluate and shuffled/no-gate
   controls have finite tests.
2. Both artifact seeds fit on seed-2711 train and run unchanged on seed-2712
   dev.
3. Evidence and REPORT state whether the true gate recovers the oracle ceiling
   without excessive premature stops.
4. Exact-head review, protected merge and fresh-main verification complete.
