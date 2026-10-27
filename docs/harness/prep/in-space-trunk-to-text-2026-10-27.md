# Prep sheet: fold against clamp for the rate score, whole-table nearest objective (in-space-trunk-to-text, 27 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Code: branch `claude/train-in-space` at `f2f002d6a66d22f87d07b4db1cf101b0bddf097c`.

TEST FITNESS: FIT (round 2, a fresh non-author reviewer, 27 Oct 2026; round 1 was NOT FIT on two blockers, both fixed; posted on #2028)

## The question

The trained trunk reads about 6 tokens back, and 61–63% of lane positions sit at the shortest-memory rate entry, against 33% at step 0. The rate score is clamped to `[0, 1]`, and a clamp has zero gradient outside it, so a lane pushed past either end stops learning there. **Does folding the score into `[0, 1]` by reflection (slope +1 or −1 everywhere) let lanes keep longer memory, so that reply openings read their prompts?**

## Steps: the arms (identical except as stated)

Common: `train-in-space --objective nearest --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --seed 1 --lr-end 0`, the FBS sheet trunk (`build_fbs_model(OneAtomActive)`), 1 × RTX 5090, one shared vector table, the target scored against the whole table's 8 nearest wrong tokens with margin 1, decoded as the whole table's nearest token.

| Arm | Pod | Rate score | Peak rate |
|---|---|---|---|
| `n-nearest` (control, already running) | `vast-57919115`, `long3` | clamp | `5e-4` |
| `n-nearest-1e-3` (control, already running) | `vast-57919115`, `long3` | clamp | `1e-3` |
| `f-fold-5e-4` | `vast-57941850`, `long4` | fold | `5e-4` |
| `f-fold-1e-3` | `vast-57941850`, `long4` | fold | `1e-3` |

The control arms ran from commit `047af46288fdd93c6ad2346d41f71e00a20bd5eb`, whose clamp path is byte-identical in behaviour to `f2f002d6` with `--fold-scores 0` (the fold is a switch that is off by default).

## What is measured (DEV, all 14,287 targets, 400 documents, 428 windows)

- `nearest.hits`: whole-table nearest top-1.
- `nearest.opening_hits` over `opening_rows` (1,226 rows whose previous corpus token is not a response token), against `commonest_opening_rows` (161: always naming the commonest opening token).
- `--reach` at steps 0 and 16,384: the opening point's mean shift when the token `k` places back is replaced by BOS, `k ∈ {0,…,6,8,10,12,16,24,32}`; and each layer's rate-entry histogram.
- Greedy replies to six fixed DEV prompts at every checkpoint (`samples.md`).
- Training context tokens per second, reported only: the clamped twins share their GPU three ways and the fold arms two ways, so speeds are not compared between them.

**The clamped twins are re-read with the same binary as the fold arms.** Their own binary (047af46) predates the opening count and `--reach`. After long3 ends, a job on `vast-57919115` builds this sheet's commit and runs `--inspect` on `long3/n-nearest` and `long3/n-nearest-1e-3` (every checkpoint) and `--reach --fold-scores 0` on each twin's `models/step-000000` and `models/step-016384`. Every number in the table below for a twin comes from that re-read.

## Bar (pre-registered): the decision table (fold arm against its clamped twin, same rate, both at step 16,384)

Definitions, per arm: **openings** `O` = `nearest.opening_hits` at step 16,384. **Relative reach** `R16` = `mean_shift(k = 16) / mean_shift(k = 0)`, from the same `--reach`, so a larger hidden scale cannot pass it. The clamped trunk of 27 Oct read `R16 = 0.018 / 5.97 = 0.003`.

- **Openings pass** when `O_fold ≥ max(197, O_twin + 36)`. Here 197 = 161 + 36, and 36 is three binomial standard deviations at the commonest-token rate, `√(1,226 · 0.131 · 0.869) = 11.8`.
- **Reach pass** when `R16_fold ≥ max(0.05, 2 · R16_twin)`, so a token 16 places back must move the opening point by at least 5% of what its own last token does, and twice the twin's share.

| | Reach pass | Reach fails |
|---|---|---|
| **Openings pass** | **A.** Dead clamp lanes were the cause. Unlocks longer fold runs and the reply text; check item done-when (1)–(2). | **B.** Openings improved from context nearer than 16 tokens. Unlocks a unit that reads which prompt types improved, before any longer-memory work. |
| **Openings fail** | **C.** Memory survives but the readout ignores it. Unlocks a unit on what reaches the opening point, not on memory length. | **D.** The clamp was not the cause. Unlocks a unit on the gradient's preference for short memory. |

The four cells are exclusive and cover every outcome. **Flag F**, reported beside the cell and not instead of it: the fold's whole-panel nearest top-1 at step 16,384 is more than 500 below its twin's, so the fold costs fit and its boundary attractors need study before the fold is kept.

Both rates must land in the same cell for the reading to count; a split is inconclusive. A missing number (a nonzero exit from an inspect or reach step, or a missing checkpoint) gives no cell: the step is rerun, never read as a fail. `R16` averages over the openings with at least 16 tokens before them and `shift(0)` over all openings; the bias is the same for every arm.

## Instrument checks

- I-1: the fold's CUDA unit test (`the_fold_reflects_into_the_unit_interval_and_keeps_its_gradient`) passes on the run's build: values `[0.5, 0.3, 0, 0.4, 1, 0.8, 0.7, 1]` for inputs `[-2.5, -0.3, 0, 0.4, 1, 1.2, 2.7, 3]`, slopes ±1.
- I-2: the clamped arms reproduce the earlier runs' step-0 DEV reading (`nearest 2`, dev loss `2.81055`), so build and data match. The fold arms differ at step 0 only through scores already outside `[0, 1]`.
- I-3: on the fold arms' step-0 checkpoint, the rate histogram of `RateBins.layer = 1` (layers count from 0) has less than 33% of `all_positions` at entry 0. The clamp's step-0 share in that layer was `64,236 / 196,608`. Folded scores no longer pile up at the ends.
- I-4: no non-finite loss (the runner stops on one).

## Data split (pinned in `hamilton_sheet.rs`, checked at load)

Training reads only TRAIN, `mw-balp/train` (68,218 usable 192-token windows). Every reading is on DEV, `ft/dev` (14,287 scored targets, 400 documents, 428 windows), which no arm trains on.

`caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/`: `tokenizer.json` `d36d3e87…f89`, `mw-balp/train/tokens.u16` `3dee858f…a4c8`, `mw-balp/train/response_mask.u8` `b5e7352d…49e3`, `ft/dev/tokens.u16` `0919e90f…555b`, `ft/dev/response_mask.u8` `a838bb1a…b078`.

## Base artifact

No trained checkpoint: every arm starts from `build_fbs_model(OneAtomActive, seed 1)`, so all four arms share one initialisation. The fold arms run the code of `eec397de469bc78f529dd929381358f44a037cc5` (tree `e2276eb032a4900b0bb7780486334a0bb242ee10`, binary sha256 `a55e7b856535e2abeba9a88214fee279e9208e6693aa176116515cb62a379bb4`); the twins ran `047af46288fdd93c6ad2346d41f71e00a20bd5eb` and are re-read with this sheet's commit.

## Minimum test power

One seed (1) per arm. The two rates are the replication: a reading counts only when both rates land in the same cell. The openings threshold is three binomial standard deviations above the commonest-token count.

## Cost

Two arms side by side on one RTX 5090 at `$0.563/h`: about 0.23 s per step each, about 65 minutes, about `$0.70`. Pods together `$1.13/h` of the `$8/h` cap.

## Stop rule

Stop on a non-finite loss or a crash; a healthy run is never cut short. Results go to `caseyallard/uor-r4-store:results/claude/train-in-space-2026-10-27/long4` (and the twins' re-read to `long3`) by `uor-store put` from the pod, read back with `uor-store ls`, before either pod comes down.
