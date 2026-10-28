# Prep sheet: width 512 with the cell objective on the pinned TRAIN (in-space-trunk-to-text, claude-153, 28 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Order: [claude-153](../../loop/tasks/in-space-trunk-to-text/claude-153/ORDER.md), with Casey's words verbatim. Code: main at `e2f15d3fcc91a6042b77fbcf717a79c39eaf215d`, unchanged. Each pod builds and tests its own binary from that commit.

TEST FITNESS: FIT, round 2 (light gates, per the owner's 27 Oct direction). PRE-RUN: PASS. Both by one fresh non-author reviewer, logged on #2028.
- **Round 1** (NOT FIT, fresh non-author): flag F at seed 2 (`T < 10,073`) sat inside (a)'s own seed spread, so it would fire about half the time with no effect. Three smaller points: the copies false-pass rate was unstated, nothing checked the head count, and one sentence misstated (c)'s fit. All four are fixed below.
- **Round 2** (FIT) and **pre-run review** (PASS), fresh non-author: all four closed. No finding blocks. Its notes, applied:
  - flag F's own false-fire rate is now stated;
  - a rerun goes to a new folder, never the old one (L-002);
  - I-1 checks the test count, not only the exit code;
  - every `uor-pod` command exports the Vast adapter, so the cap check sees Vast pods.

## The question

**On the TRAIN that works, does a trunk twice as wide make replies use the prompt more than claude-152's (a)?** (a) is the cell objective at width 256 with 4 Hamming read heads.

What claude-152 left open:
- **(a)** was cell C: copies rose (`1,517 / 2,933` of 4,130 copied rows) and openings did not (`306 / 245` of 1,226).
- **Width 512** was run only on `balp`, where fit was lost. There it copied most (`3,342 / 3,330`), but its fit (`T = 5,434 / 4,957`) stayed below (a)'s and below the M1 count table's `7,002`. So the width's effect on the pinned TRAIN is unmeasured.

## The prompt-use test

claude-152's test, unchanged. It is computed in `dev_nearest` at every checkpoint, on the pinned DEV:
- **`O`, openings:** `opening_hits` of 1,226.
- **`C`, copied rows:** `copied_hits` of the panel's `4,130` copied rows. These are reply rows whose previous token and target stand side by side in the window's prompt.
- **`T`:** top-1 on all 14,287 rows.
- **Rest:** `rest_hits` of the `9,094` reply rows that neither open nor copy.

## Steps: the arms

Every arm runs `train-in-space --objective cell --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S` on the pinned TRAIN, at seeds 1 and 2. These are claude-152's settings for (a), exactly.

| Arm | Extra flags | Its change | Reference |
|---|---|---|---|
| (w) wide | `--width 512 --hamming-read 8` | width 512, and 8 read heads, so each head keeps its 64-bit code | claude-152's (a) (`--hamming-read 4`, width 256) at the same seed |
| (w0) control | `--width 512 --hamming-read 0` | (w) with no read | none; it is (w)'s no-read floor for copies |

(w) changes two things together, the width and the head count, because a head's code is one 64-bit word (`HammingRead::new` refuses wider heads). A result at (w) does not separate them. That limit is stated, not tested here.

## Bar (pre-registered), (w) against (a) at step 16,384

The rule is claude-152's. Each arm is read against the **better of its reference's two seeds**, and **both** of the arm's seeds must clear it. Write `best(X)` for the larger of a run's seed-1 and seed-2 values, and `m(X, N) = 3·√(N·p(1−p))` with `p = best(X)/N`.

(a)'s step-16,384 values, seed 1 / seed 2 (claude-152's REPORT): `T` `9,328 / 10,573`; `O` `306 / 245`; `C` `1,517 / 2,933`; rest `7,600 / 7,490`.

- **Openings pass:** at both seeds, `O ≥ max(197, 306 + m(O, 1,226))` = `306 + 45.5`, so `O ≥ 352`.
- **Copies pass:** all three, at both seeds.
  1. **Against (a):** `C ≥ 2,933 + m(C, 4,130)` = `2,933 + 87.5`, so `C ≥ 3,021`.
  2. **Against the no-read floor:** `C ≥ best(C_w0) + m(C_w0, 4,130)`, with (w0)'s two step-16,384 values.
  3. **Against the rest rows:** `C/4,130 − 2,933/4,130` (in points) must exceed `rest/9,094 − 7,600/9,094`. A gain that only matches the rest is general fit, not prompt use. This guard has no margin; it is a direction check beside 1 and 2.

| | Copies pass | Copies fail |
|---|---|---|
| **Openings pass** | **A.** The reply reads the prompt for what to say and what to copy. | **B.** The prompt picks the reply; no more copying. |
| **Openings fail** | **C.** More copying; the same templates. | **D.** No gain in prompt use over (a). |

- **Flag F:** at either seed, `T` below (a)'s worse seed minus 500: `T < 9,328 − 500 = 8,828`. Fit is lost.
  - This changes claude-152's same-seed rule. Training is nondeterministic, so a seed number pairs nothing: (a)'s own seeds read `9,328` and `10,573`. A seed-2 flag at `10,073` would fire about half the time with no effect.
  - Its own false-fire rate: if (w) fits like (a) (mean about `9,950`, SD about `880`, from two values), each seed reads below `8,828` about 10% of the time, so either seed does about 19%. A false flag keeps width 256 for one unit.
- A pass at one seed only is reported as such, and counts as no pass. No third seed runs in this unit.
- A missing number gives no cell: that step is rerun.

**What each outcome decides:**
- **A, B or C:** width 512 with 8 heads becomes the trunk for later runs on the pinned TRAIN. The next unit takes a larger width or more steps.
- **D:** width 256 stays. The next unit goes after the openings directly: the words that decide a reply sit at the prompt's start, beyond the trunk's reach.
- **Flag F set:** width 256 stays until fit is restored, whatever the cell.
- **The item's done-when, read on each (w) and (w0) final checkpoint:** done-when 1 is `O ≥ 197` with `T ≥ 7,002`. Done-when 2 adds `R16 ≥ 0.05` from `--reach` on that checkpoint. A checkpoint meeting both meets the item's bar, whatever the cell.

## Instrument checks

- **I-1:** the crate's CUDA tests pass on each pod's build (`--features cuda --lib`), and read `123 passed; 0 failed; 5 ignored` as on claude-152's pods: the code is the same.
  - The cell objective's negative gate is not rerun. The code is byte for byte the code it failed against on all four of claude-152's pods: `git diff d102c594 e2f15d3fc -- crates/ Cargo.lock` is empty.
- **I-2:** (w)'s and (w0)'s step-0 `dev_nearest` hits, openings and distinct opening picks equal claude-152's width-512 (c) at the same seed: `1/0/756` at seed 1 and `3/1/659` at seed 2 (step 0 of `c-s1` and `c-s2`, sealed in the store).
  - Step 0 depends on neither TRAIN nor the read: claude-152's (a), (b) and (b0), on two TRAINs and with 4 or 0 heads, all equal claude-146's twins at step 0.
  - It is checked first on the 256-step probe, so a wrong build stops in minutes, and again on the arm.
  - Each arm's `config.json` must also record `"width": 512`, its `"hamming_read"` (`8` or `0`) and `"train_split": null` (the pinned TRAIN). Step 0 cannot see the head count, so this checks it.
  - A failed I-2 stops every reading of that pod.
- **I-3:** (w)'s read weight is above 0 at step 16,384 (`--inspect`'s `hamming_weight_mean_abs`). (w0) has no read.
- **I-4:** every loss is finite.

## Data split

- **DEV:** the pinned DEV (14,287 rows, 1,226 openings, 4,130 copied rows), checked by sha256 at load (`PINNED_DEV_*`).
- **TRAIN:** the pinned `mw-balp` (`PINNED_TRAIN_*`), checked by sha256 at load. No `--train-split`.
- **Overlap:** TRAIN and DEV are separate pinned splits (`mw-balp/train`, `ft/dev`).

## Base artifact

None. Each arm starts from `build_fbs_model_at(OneAtomActive, seed, 512)`, the same initial model claude-152's (c) started from at the same seed (I-2 checks it).

## Minimum test power

- **Panel noise:** each margin is three binomial standard deviations at (a)'s better seed, on the panel's own row counts.
- **Seed noise is larger.** (a)'s two seeds read copies `1,517` and `2,933`, and `T` `9,328` and `10,573`. The bar asks both (w) seeds to beat (a)'s better seed, so a real gain smaller than that spread can be missed. A miss is "keep and iterate", not a close (CHARTER §4.1).
- **The false-pass rate:** if (w) and (a) come from one distribution, both (w) seeds are the top two of four draws `1` time in `6`, before any margin.
  - **On openings,** the margin is about `1.05` of (a)'s seed SD (`43`, from two values), which cuts that well below `1` in `6`.
  - **On copies,** it is only about `0.09` of (a)'s seed SD (`1,001`, from two values), so copies bar 1 alone passes by chance about `1` time in `7`. Bars 2 and 3 cut that further, by an amount this sheet cannot state. Copies decide cell C, the likeliest outcome, so C is the weakest reading.
  - The decision it drives (the width for later runs) is re-tested by every later run, so the rate is stated and accepted.

## Cost

Four Vast RTX 5090 pods, one GPU each, at about `$0.563/h`, one arm per pod: (w) at seeds 1 and 2, and (w0) at seeds 1 and 2.
- **(w)** on the pinned TRAIN is projected at about `16,800` context tokens/s: claude-152's (c) ran at `12,512` on `balp`, and `balp` cost (a) about 25%. That is about 50 minutes for 16,384 steps of `16 × 192` tokens.
- **(w0)** has no read, so it should be faster; its 256-step probe measures it.
- Each pod also builds and tests (about 15 minutes) and runs reach, inspect and decode (about 3 minutes).

About 4.2 pod-hours in all, projected under `$4`. If fewer than four pods can be had, pods 1 and 2 run (w) and then (w0) at their seed.

## Stop rule

- **Stop on:** a non-finite loss, a crash, a failed test or I-2, or a 256-step probe that projects an arm past 2 hours (below `6,991` context tokens/s). In that last case, profile first. A healthy run is never cut short.
- **Results:** they go to `caseyallard/uor-r4-store:results/claude/wide-cell-2026-10-28/`, one tar per pod with a SHA256SUMS written on the pod, read back to the laptop and checked before that pod comes down.
