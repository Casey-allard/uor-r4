# Prep sheet: does the reply use the user's input? A cell margin, more text, a wider trunk (in-space-trunk-to-text, claude-152, 28 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Order: [claude-152](../../loop/tasks/in-space-trunk-to-text/claude-152/ORDER.md), with Casey's words verbatim. Code: branch `claude/prompt-use`. Each pod builds and tests its own binary from the pinned commit.

TEST FITNESS: FIT, by claude's decision review after two fresh non-author rounds (light gates, per the owner's 27 Oct direction).
- **Round 1** (NOT FIT): the binomial bar ignored the seeds' spread; copied rows can rise from pair statistics; the ORDER's copy rule differed from the code's; (c) changes two things.
- **Round 2** (NOT FIT, pre-run FAIL): round 1's last three were closed. Still open: (b)'s copies had no no-read control on its own text, the false-pass rate of (b) and (c) was unstated, and three clauses were unclear.
- **After round 2,** at the two-round limit: the (b0) no-read control on `balp` is added and barred against, the false-pass rate is stated, and the three clauses are fixed. The decision review is logged on #2028.

## The question

**With the Hamming read, which of three changes makes replies use the prompt: (a) the cell margin objective, (b) 13 times more training text, or (c) a trunk twice as wide?** The new prompt-use test measures it.

**What matters, from the probe already run.** claude-145's opening count looks up TRAIN's commonest reply opening by the prompt words a window shows. It names:
- `161` of DEV's 1,226 openings from the last 1–4 tokens before the reply;
- `607` from the prompt's first 8 tokens.

The words that decide a reply sit at the prompt's start, beyond the trunk's reach of about 6 tokens. Every model so far is at `200–302`.

## The prompt-use test

It is computed in `dev_nearest`, at every checkpoint, on the pinned DEV.

- **`O`, openings:** `opening_hits` of 1,226. Read against `161` (always the commonest) and `607` (the first-8-token lookup).
- **`C`, copied rows:** `copied_hits` of `copied_rows`. These are DEV reply rows whose previous token and target stand side by side in the window's prompt, as a code word or a number the user gave does. Naming them takes reading the prompt.
- **`T`, top-1 on all 14,287 rows,** and `rest_hits` on reply rows that neither open nor copy.

## Steps: the arms

Every arm runs `train-in-space --objective cell --hamming-read H --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S` at seeds 1 and 2: claude-146's settings with the objective changed. Each arm changes one thing from its reference.

| Arm | Its change | Extra flags | Reference |
|---|---|---|---|
| (a) cell | the objective: the nearest wrong token's hinge alone, against `nearest`'s mean over 8 | `H = 4` | claude-146's `df1d7892` Hamming runs (`nearest`) at the same seed |
| (b) text | TRAIN becomes `balp`, 99,322,259 tokens | `--train-split balp --train-sha256 13261ba8… --train-mask-sha256 19c1a93f…`, `H = 4` | (a) at the same seed |
| (c) wide | width 512, and 8 read heads, so each head keeps its 64-bit code | (b)'s, plus `--width 512`, `H = 8` | (b) at the same seed |
| (b0) control | (b) with no read: the no-read floor on `balp` | (b)'s, with `H = 0` | none; it is (b)'s and (c)'s floor |

(c) changes two things together, because a head's code is one 64-bit word (`HammingRead::new` refuses wider heads), so 512 needs 8 heads. A D at (c) does not separate the width from the head count.

**The probe, re-scored.** `--inspect` on stored step-16,384 checkpoints at seeds 1 and 2, with the new test:
- claude-145's no-read runs: (a)'s no-read floor for `C`, barred below;
- claude-146's Hamming runs: (a)'s reference, and I-5;
- claude-151's spin runs: reported.

It shows how much the read already copies.

## Bar (pre-registered), each arm against its reference at step 16,384

Training is nondeterministic, and seeds spread far more than one panel's binomial noise: claude-146's twins read openings `298` and `200`, 98 apart. So each arm is read against the **better of its reference's two seeds**, and **both** of the arm's seeds must clear it. Write `best(X_ref)` for the larger of the reference's seed-1 and seed-2 values, and `m(X, N) = 3·√(N·p(1−p))` with `p = best(X_ref)/N`.

- **Openings pass:** at both seeds, `O ≥ max(197, best(O_ref) + m(O, 1,226))`. For (a), the twins give `best = 298` and `m = 45`, so the bar is `343`.
- **Copies pass:** three conditions, all at both seeds.
  1. **Against the reference:** `C ≥ best(C_ref) + m(C, N)`, where `N` is the panel's copied rows. `N` does not depend on the model, so it is the same for every arm.
  2. **Against the no-read floor:** `C ≥ best(C_floor) + m(C, N)`, where `best(C_floor)` is computed like `best(C_ref)`. The floor is what pair statistics name with no read, on the same TRAIN: claude-145's re-scored no-read runs for (a), and (b0) for (b) and (c). (c) has no width-512 no-read control; that is a stated limit.
  3. **Against the rest rows:** the copied rows' hit rate must gain more points over `best(C_ref)/N` than the rest rows' rate, `rest_hits / rest_rows`, gains over the same reference's `best`. A gain that only matches the rest is general fit, not prompt use. This guard has no margin, and the two groups start from different rates; it is a direction check beside 1 and 2.

| | Copies pass | Copies fail |
|---|---|---|
| **Openings pass** | **A.** The reply reads the prompt for what to say and what to copy. | **B.** The prompt picks the reply; no more copying. |
| **Openings fail** | **C.** More copying; the same templates. | **D.** No gain in prompt use. |

- **Flag F:** at either seed, `T < T_ref − 500` against the reference at the same seed, meaning fit is lost.
- Each pass already needs both seeds. A pass at one seed only is reported as such, as no pass, and no third seed runs in this unit.
- A missing number gives no cell: that step is rerun.

**What each cell decides:**
- **(a):** A, B or C makes `cell` the objective for later runs; D keeps `nearest`.
- **(b):** A, B or C makes `balp` TRAIN for later runs; D keeps the pinned TRAIN.
- **(c):** A, B or C makes 512 the width for later runs and sends the next unit to a larger width or more steps; D keeps 256.
- With flag F set, the change waits until fit is restored.
- A meets the item's done-when 1 if `O ≥ 197` and `T ≥ 7,002`. Done-when 2 also needs `R16 ≥ 0.05` from `--reach` on that checkpoint, which each arm's last checkpoint runs.

## Instrument checks

- **I-1:** the crate's CUDA tests pass on each pod's build. One negative gate: with the cell hinge's max replaced by a mean, `the_cell_objective_pushes_only_the_nearest_rival` must fail with exit 101 and its own `FAILED` line. The host gate (the copied-row test failing with the prompt-only check removed) ran on the laptop.
- **I-2:** (a)'s and (b)'s step-0 `dev_nearest` hits, openings and picks equal claude-146's twin's: `2/0/730` at seed 1, `5/3/681` at seed 2. The objective and TRAIN do not touch step 0. (c) is a different model.
- **I-3:** in every arm with a read, its weight is above 0 at step 16,384 (`--inspect`'s `hamming_weight_mean_abs`). (b0) has no read.
- **I-4:** every loss is finite.
- **I-5:** the re-scored claude-146 checkpoints reproduce their stored step-16,384 `hits` and `opening_hits` exactly (`8,687/298` and `8,709/200`). Otherwise the new test changed what it should not. **A failed I-5 stops every reading.**

## Data split

- **DEV:** the pinned DEV (14,287 rows, 1,226 openings), checked by sha256 at load (`PINNED_DEV_*`).
- **TRAIN:**
  - for (a), the pinned `mw-balp` (`PINNED_TRAIN_*`);
  - for (b) and (c), `balp`, from `caseyallard/uor-r4-store:data/ft-balp.tar` (sha256 `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`, the archive TRAIN's own pin names). Its members are `balp/tokens.u16` (sha256 `13261ba81180d8c59086787d80e34cd8ff2a144d6a818018725777d5a8d6bf62`) and `balp/response_mask.u8` (sha256 `19c1a93f504e4de83af73fe8380c8dde1dec3cb4bef52ba0c7c6148fc73aaf67`). The loader refuses any other bytes.
- **What `balp` is:** `mix-chat-corpus` of `mw-balp` twice, `mw-orcp` and `chat-v0-p2`, with the same tokenizer (`d36d3e87…`) and dialogue protocol 2.
- **Overlap:** `balp` holds TRAIN, not DEV. DEV is `ft/dev`, a separate pinned split.

## Base artifact

None. Each arm starts from `build_fbs_model_at(OneAtomActive, seed, width)`, which at 256 is today's `build_fbs_model`.

## Minimum test power

- **Panel noise:** each margin is three binomial standard deviations at the better reference seed's rate, on the panel's own row counts (1,226 openings; `N` copied rows, counted on the panel and reported).
- **Seed noise:** this is larger. claude-140 measured this training as nondeterministic, and claude-146's twins spread 98 openings. The bar handles it by requiring both arm seeds to beat the better reference seed, not a seed-paired difference. Two seeds against two seeds is still weak: a no-effect arm drawn from the twins' spread (mean about 249, standard deviation about 69) clears `343` at both seeds about 1% of the time if the seeds are independent. A real gain smaller than the spread can be missed. A miss here is "keep and iterate", not a close.
- **The false-pass rate for (b) and (c),** whose references are this unit's own runs. If an arm and its reference come from one distribution, both arm seeds beat the better reference seed plus `m` (about 0.65 of the twins' seed SD) about 6% of the time per arm, on openings alone. That SD is estimated from two values. Each of (b) and (c) can therefore pass by chance about once in 17. The decisions they drive (TRAIN and width for later runs) are re-tested by every later run, so the rate is stated and accepted, not reduced by more seeds in this unit.

## Cost

Four RTX 5090 pods at about `$0.563/h`:
- pod 1 runs the seed-1 probe, then (a), (b) and (b0) at seed 1;
- pod 2 does the same at seed 2;
- pods 3 and 4 run (c) at seeds 1 and 2.

(a), (b) and (b0) each take about 32 minutes alone (claude-151's arms ran at 27,000–28,000 context tokens/s), so pods 1 and 2 take about 1 hour 45 minutes. (c) is measured by its probe. Projected under `$5`.

## Stop rule

- **Stop on:** a non-finite loss, a crash, a failed gate, a failed I-5, or a 256-step probe that projects an arm past 2 hours, in which case profile first. A healthy run is never cut short.
- **Results:** they go to `caseyallard/uor-r4-store:results/claude/prompt-use-2026-10-28/`, sealed with a SHA256SUMS and read back before any pod comes down.
