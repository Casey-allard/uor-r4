# ORDER: cpu-training-pilot, deepseek-179 (expanded-cell anchored candidate ranker)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `d416181c1`.

## Goal

Test whether CPU-only access to more geometric writer cells can supply enough
supervision for a longest-32-anchored correction ranker to improve copy/body
selection without replacing the stronger baseline on its correct decisions.

The served writer's eight-cell admission is unchanged. The expanded cell order
is a CPU development candidate source only.

## Attempt history and correction

The first pre-registration selected 12 cells, but seed 2 had only 2,043
target-present groups, five below the 2,048 floor. Its driver stopped before
evaluation. The preserved classification is `INSTRUMENT_FAULT /
K12_SELECTION_ERROR`; its raw automation outcome remains
`INSUFFICIENT_TRAINING_DATA`. Seed 2769 is closed and must not be reused.

The same train-only sweep already showed that 16 cells clear every floor on
both artifacts. The corrected registration fixes K=16 on a new, unused seed
2770. This is a corrected selection of an already registered sweep value, not
a change after reading a fresh result.

## Why this is the next unit

Deepseek-177 reached the served eight-cell ceiling: 4,096 tokens over 256 cells
at 16 tokens per cell is 128 writer candidates. Artifact seed 1 then reached
2,442 target-present groups but the unconstrained ranker selected depth 32 on
14,505 of 28,480 decisions and scored only 265 body positions against 601 for
longest-32. Artifact seed 2 stopped at 1,847 groups.

This unit changes both defects: it broadens the geometric cell search only in
the CPU candidate source, and it trains only corrections where longest-32 is
wrong.

## Cancel probe

- Confirm no other open order or merged result already tests an expanded-cell
  CPU candidate order with an anchored correction objective.
- Preserve the K12/seed-2769 fault and confirm seed 2770 and its `dev,dev`
  panel are absent.
- Confirm no Claude-owned or serving source is needed.
- Confirm the public artifacts expose at least 32 complete geometric cells.

## Fixed construction

1. Add a CPU-only writer candidate order over a configurable number of
   geometric cells. It uses the same 16 SpiralCore roots, two eight-coordinate
   ports, integer L1 cell score, and deterministic cell/token tie order as the
   served writer. The served writer still keeps eight cells.
2. Sweep `cells = 8, 12, 16, 24, 32` on closed train panel 2711 for both public
   artifacts before creating any fresh panel.
3. For each setting form one unique-token candidate union from:
   - the highest-scoring non-BOS writer tokens admitted by that many cells;
   - exact-copy depths one through 32;
   - the current writer/point fallback;
   - the longest-32 token explicitly, even if another source would omit it.
4. Keep the deepest exact-copy source, writer score/rank and longest-32
   identity for each token.
5. Use twelve bounded integer candidate features:
   - bias;
   - exact-copy depth, or zero;
   - query/token sign agreement;
   - best sign agreement over eight cyclic row rotations;
   - source recency;
   - within-window occurrence frequency;
   - writer-token agreement;
   - writer-rank bucket;
   - is-longest-32;
   - signed exact-depth gap from longest-32;
   - sign-agreement gain over longest-32;
   - rotated-agreement gain over longest-32.
6. Choose the smallest cell count that, on both artifacts, yields:
   - at least 2,048 target-present groups;
   - at least 64 distinct target tokens;
   - at least 256 groups where longest-32 is wrong and the reference target is
     present.
7. If no setting clears all three floors, stop as
   `INSUFFICIENT_TRAINING_DATA` and open no fresh seed.
8. Once a setting is selected, pre-register it with a fresh FIT and GO on
   `cell=dev,dev`, seed 2770, 128 conversations and the two public artifacts.
9. Train an anchored correction perceptron for eight passes at learning rate
   four with weights clipped to `[-4096, 4096]`. Baseline-correct groups make
   no update. Baseline-wrong groups compare the reference target with the
   longest-32 candidate, not with whichever candidate the current weights
   happen to choose.
10. At inference longest-32 is the default. Another candidate replaces it only
    when its learned score is strictly greater than the longest-32 score. Ties
    always keep longest-32.
11. Evaluate these arms:
    - anchored trained ranker;
    - anchored shuffled-target ranker;
    - anchored fixed geometry-only scorer;
    - unanchored trained ranker over the same union;
    - longest-32;
    - writer.
12. No reference token is read during generated inference.

## Bar

On seed 2770 and both artifact seeds, the anchored trained ranker must:

- at least 200 reachable replies per artifact;
- beat longest-32, writer, anchored geometry-only, anchored shuffled-target and
  the unanchored trained control on correct body positions by the existing
  paired two-sigma rule;
- change at least one token against each control;
- emit zero premature BOS tokens;
- repeat byte-identically after timing fields are removed, with reply and
  decision hashes for every arm.

The served writer and all serving, CUDA, float-inference, termination and
Claude-owned source remain unchanged.

## Stop rule

Stop on source, candidate, feature, label, weight or report drift; failure of
any train-only calibration floor; fewer than 200 reachable replies; a vacuous
control; nondeterminism; or any need to edit a serving or Claude-owned path.
Once seed 2770 is read, do not change the selected cell count, features,
anchor, threshold or bar.

## Done when

1. Expanded-cell candidate access, anchored fitting, controls and tests are
   implemented in the CPU-training lane.
2. Both artifacts are calibrated on train panel 2711; a selected cell count is
   written into a FIT/GO-reviewed pre-registration, or the run stops without a
   fresh panel.
3. If FIT and GO pass, seed 2770 runs exactly as registered with deterministic
   repeats and compact evidence.
4. REPORT classifies KEEP or REJECT against the bar, updates the notebook,
   roadmap and current state, and makes no complete-reply, language or serving
   claim.
5. Reviewed merge, fresh-main verification and claim release complete.
