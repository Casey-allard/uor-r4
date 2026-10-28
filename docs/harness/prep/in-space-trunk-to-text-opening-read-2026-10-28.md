# Prep sheet: the opening read, carrying the prompt's first words to the reply's first token (in-space-trunk-to-text, claude-154, 28 Oct 2026)

Lab: claude. Item: [in-space-trunk-to-text](../../roadmap/items/in-space-trunk-to-text.md). Order: [claude-154](../../loop/tasks/in-space-trunk-to-text/claude-154/ORDER.md), with Casey's words verbatim. Code: branch `claude/opening-read` at `060ba8127855db6ab1b198a7c59dd932e0b85b80`: the read is `5a2a0953c` and its wiring `060ba8127`. Each pod builds and tests its own binary from that commit.

Host checks at that commit (laptop):
- every host test target passes, 99 tests, 0 failed;
- three negative gates bite (exit 101), and each passes when restored:
  - the turn anchor forced to the window start fails the slot test;
  - the config's skip rules removed fails the no-flags config test;
  - the checkpoint load removed fails the load test.

TEST FITNESS: FIT, round 1, by a fresh non-author reviewer (light gates, per the owner's 27 Oct direction). It made four cosmetic notes, all applied:
- the test count is fixed in advance, not read from the first pod;
- cell C's reading is narrowed;
- the per-seed gain over (a) is reported;
- I-5 is added, a check that the evaluation uses the read.

PRE-RUN: PASS, by a fresh non-author reviewer. Its two notes:
- each pod's tar is named by pod and arm, since (o) and (oL) at one seed write same-named folders on different pods;
- the in-training evaluations that `Ō` is read from run on the thread where training set the read, which the REPORT states.

## The question

**Does giving every position the first words of its user turn lift reply openings above claude-152's (a)?** And is the gain from those first words, rather than from any extra read path?

**Why this is the next test.** No arm in claude-152 or claude-153 cleared its openings bar. claude-145's probe looks up TRAIN's commonest opening by prompt tokens, and names:
- `607` of DEV's 1,226 openings from the prompt's first 8 tokens;
- `161` from the last 1–4 tokens before the reply.

The best trunk reaches `R16 = 0.034`, and the Hamming read matches by content, which at an opening is always the same last prompt token. The opening read hands the trunk those first 8 tokens directly.

## The mechanism

`--opening-read 8`:
- **Slots.** At each position `t`, the turn anchor is the position after the most recent `<|bos|>` or `<|eos|>` at or before `t` (or the window's first position). Slot `k` is the anchor plus `k`, kept if it is at or before `t`. With `--opening-anchor last`, the slots are the last 8 positions up to `t`.
- **The summary.** Each kept slot's token's point in the shared table (detached) is turned by that slot's learned quaternion, lane by lane. The kept slots are averaged.
- **Where it enters.** The summary enters the stream after layer 2 as `x + w ⊙ summary`, with `w` learned and starting at zero.

There is no exponential and no matrix product.

## The prompt-use test

claude-152's test, unchanged, on the pinned DEV at every checkpoint:
- `O` is `opening_hits` of 1,226;
- `C` is `copied_hits` of 4,130 copied rows;
- `T` is top-1 of 14,287 rows;
- rest is `rest_hits` of 9,094.

**One change to how `O` is read:** `Ō` is the mean of `O` over the last three checkpoints (steps 12,288, 14,336 and 16,384). Within one run, `O` moved by up to `124` between adjacent checkpoints in claude-153, so a bar read at one checkpoint also carries that noise. The reference is read the same way.

## Steps: the arms

Every arm runs `train-in-space --objective cell --hamming-read 4 --retain-rates 1 --decay-mult 1 --batch 16 --steps 16384 --eval-every 2048 --lr 0.001 --lr-end 0 --seed S` on the pinned TRAIN, at width 256 and seeds 1 and 2. That is claude-152's (a) exactly, plus:

| Arm | Extra flags | What it tests |
|---|---|---|
| (o) | `--opening-read 8` | the turn's first 8 tokens, at every position |
| (oL) control | `--opening-read 8 --opening-anchor last` | the same read with the same parameters, on the last 8 tokens: any extra read path |

**The reference** is claude-152's (a), with `Ō` read from its sealed `progress.jsonl` (`pod1-s1.tar`, `pod4-s2.tar` in `results/claude/prompt-use-2026-10-28/`):
- seed 1: `301`, `299` and `306`, so `Ō = 302.0`;
- seed 2: `222`, `217` and `245`, so `Ō = 228.0`.

## Bar (pre-registered), (o) at step 16,384 and the two checkpoints before it

The rule is claude-152's. Both of (o)'s seeds must clear the better reference seed plus `m(N) = 3·√(N·p(1−p))`, with `p` set at the better seed's rate.

- **Bar 1, against (a):** at both seeds, `Ō ≥ max(197, 302.0 + m(1,226))` = `302.0 + 45.3`, so `Ō ≥ 347.3`.
- **Bar 2, against (oL):** at both seeds, `Ō ≥ best(Ō_oL) + m(1,226)` at `p = best(Ō_oL)/1,226`.

| | Bar 2 passes: beats (oL) | Bar 2 fails |
|---|---|---|
| **Bar 1 passes: beats (a)** | **A.** The prompt's first words reach the opening. | **B.** An extra read path lifts openings; the turn anchor is not shown to be why. |
| **Bar 1 fails** | **C.** The anchor beats its control; any gain over (a) is under the margin. | **D.** No gain in openings from the opening read. |

- **Flag F:** at either seed, `T` at step 16,384 below (a)'s worse seed minus 500: `T < 9,328 − 500 = 8,828`. Fit is lost.
- A pass at one seed only counts as no pass. No third seed runs in this unit.
- A missing number gives no cell: that step is rerun into a new folder.

**What each outcome decides:**
- **A:** the opening read (turn anchor, `K = 8`) joins the recipe for later runs. The next unit reads the free replies' openings and the item's done-when.
- **B:** the read stays optional. The next unit asks why the control helps: placement, or capacity.
- **C:** the read stays optional. The next unit moves its placement (the readout, or after a later layer) before any recipe change.
- **D:** this configuration closes (CHARTER §4.1). The next unit tries exact memory for the opening: a read of TRAIN's counts keyed on the prompt's first tokens, against a no-memory control (L-010).
- **Flag F set:** no recipe change until fit is restored, whatever the cell.
- **The item's done-when,** read on each final checkpoint of (o) and (oL): done-when 1 is `O ≥ 197` with `T ≥ 7,002`; done-when 2 adds `R16 ≥ 0.05`.
- **Reported, not barred:** each arm's `Ō` gain over (a) at the same seed. Bar 1 asks seed 2 to gain about `119` and seed 1 about `45`, because (a)'s seeds differ, so an uneven real gain can land in C or D.

## Instrument checks

- **I-A, the anchor:** `--opening-bound` prints at how many DEV and TRAIN openings the read's first 8 slots equal the probe's first-8 key. **Run at `060ba812` on the laptop:**
  - DEV `1,226` of `1,226` agree, and TRAIN `212,968` of `212,968`;
  - the same run reproduces the probe: first-8 names `607`, first-4 `227`, last-4 `161`.
- **I-1:** the crate's CUDA tests pass on each pod's build and read `131 passed; 0 failed; 5 ignored`. One CUDA run on the first pod checks this before any arm; a different count is classified before any run. 131 is claude-152's 123 plus the 8 new library tests, two of them the CUDA tests of the read's `apply`:
  - a zero weight is no read;
  - the read adds the mean of its turned slots.
- **I-2:**
  - Each arm's step-0 `dev_nearest` hits, openings and distinct opening picks equal claude-152's (a) at the same seed: `2/0/730` at seed 1 and `5/3/681` at seed 2. The read's weight starts at zero, so step 0 is the model without it.
  - It is checked on each 256-step probe and again on each arm.
  - Each arm's `config.json` records width 256, `"hamming_read": 4`, `"opening_read": 8`, its anchor and `"train_split": null`.
  - A failed I-2 stops every reading of that pod.
- **I-3:** at step 16,384, the opening read's weight (`opening_weight_mean_abs`) and the Hamming read's weight are above 0.
- **I-4:** every loss is finite.
- **I-5, the evaluation uses the read:** the final checkpoint is copied without `opening_read.safetensors` and re-scored with `--inspect`. Its `nearest` reading must differ from the same checkpoint's re-score with the read. If they are identical, the read never reached the evaluation, and no cell is read.

## Data split

- **DEV:** the pinned DEV (14,287 rows, 1,226 openings, 4,130 copied rows), checked by sha256 at load.
- **TRAIN:** the pinned `mw-balp`, checked by sha256 at load.
- **Overlap:** TRAIN and DEV are separate pinned splits.

## Base artifact

None. Each arm starts from `build_fbs_model_at(OneAtomActive, seed, 256)`, as (a) did.

## Minimum test power

- **Panel noise:** the margin is three binomial standard deviations at (a)'s better `Ō`. Averaging three correlated checkpoints lowers the within-run noise, so this margin is conservative.
- **Seed noise:** (a)'s `Ō` seeds differ by `74`, a seed SD of about `52` from two values. The margin is about `0.87` of it.
- **The false-pass rate:** if (o) and (a) come from one distribution, both (o) seeds are the top two of four draws `1` time in `6` before any margin, and less with it. Bar 2 against (oL) is a second, independent hurdle for A.
- A real gain smaller than the seed spread can be missed; a miss is "keep and iterate" (CHARTER §4.1).

## Cost

Four 1-GPU RTX 5090 pods on Vast, one arm per pod: (o) and (oL) at seeds 1 and 2.
- An arm is about 26 minutes of training: (a) ran at about `32,000` context tokens/s, and the read adds little.
- Each pod also takes about 15 minutes to build and test, and 3 minutes for reach, inspect and decode.

About 3 pod-hours, projected under `$4` at `$0.56–1.10/h`. With fewer pods, a pod runs (o) and then (oL) at its seed.

## Stop rule

- **Stop on:**
  - a non-finite loss;
  - a crash;
  - a failed test, I-A or I-2;
  - a 256-step probe that projects an arm past 2 hours (below `6,991` context tokens/s), in which case profile first.

  A healthy run is never cut short.
- **Results:** they go to `caseyallard/uor-r4-store:results/claude/opening-read-2026-10-28/`, one tar per pod with a SHA256SUMS written on the pod, read back and checked before that pod comes down.
