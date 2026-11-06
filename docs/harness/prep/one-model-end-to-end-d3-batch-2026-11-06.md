# Prep sheet: does a larger batch on D3 make the trunk's seeds agree, and beat T2? (one-model-end-to-end, claude-298, 6 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-298. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit goes to making the trunk's no-memory top-1 beat its TRAIN's unigram and T2 on the human held-out panel, beyond the paired band, at every seed.

TEST FITNESS: FIT WITH CHANGES (round 1, a fresh non-author review, 6 Nov ~15:20Z: no blockers; its four majors and six minors are applied below; see the end).

**Why.** claude-297 (#3774, merged as `1f2bd76ce58503f25ef24e415dd193bf57086ddb`) ran the (near) recipe as a 2×2 of steps and data at batch 16. Its readings:
- **The one-track bar is not met.**
  - (n3L), TRAIN D3 at 32,768 steps, got 272,867 / 234,892 of the panel's 1,460,697 rows against D3's T2 at 256,660. That reads `inside` (d +16,207 / −21,768, band 37,975).
  - The other three cells are `behind` at both seeds.
- **Data is the one lever decided at both seeds.** At 8,192 steps D3 beats D1 by 20,217 / 17,386 rows (band 2,831), unseen rows included. That is claude-297's sheet's branch "None does, and data moves it": the next unit "trains longer at D3, or on more human text".
- **The band is the seed spread, and added seeds cannot close it.**
  - `scale-probe pair` gives every seed a band of at least max d − min d. `ahead` needs every seed beyond its band, and the tool takes at most three seeds (`docs/labs/scale-probe/instrument/src/lib.rs` 919–953, 1129–1149).
  - With seed 2's d negative, no seed set that includes seed 2 can read (n3L) `ahead`. claude-297's decision review set that sheet's "add seeds" line aside for this reason.
- **The spread comes from the optimizer, not the measurement.**
  - Seed 2's lost first run and its rerun on another host read the same loss at step 31,744, to every printed digit (0.9592743465327658).
  - (n3L)'s two seeds had separated by step 8,192: loss 0.9601 against 0.9791.
- **Batch 16 is a noisy gradient.** A batch is 16 × 192 = 3,072 positions.
  - The loss is a per-position mean (`mean_all`, `in_space_long.rs` 1144–1146) under AdamW.
  - So a larger batch at the same `--lr` takes steps of the same size with less noise. The learning rate falls by step (`in_space_long.rs` 12037–12043).
  - A larger batch is the one change that attacks both the spread and "train longer at D3".
  - **Equal tokens is not equal updates.** (b64S) makes a quarter of (n3L)'s updates, with a quarter of its total weight decay and a schedule a quarter as long in updates. The arm that isolates gradient noise is (b64L): the same 32,768 steps, schedule and decay as (n3L), at a quarter of the noise.

## Question

At TRAIN D3 and (near)'s recipe, with only the batch changed from 16 to 64:
- **Is the seed spread gradient noise?** Read on (b64L), which runs 32,768 steps at batch 64, against (n3L)'s 32,768 at batch 16.
- **Does the trunk pass D3's T2 at both seeds?** (b64L) trains on four times (n3L)'s tokens. (b64S) runs 8,192 steps at batch 64, the same positions as (n3L) in a quarter of the updates.

## Base artifact

- **The pin:** `1f2bd76ce58503f25ef24e415dd193bf57086ddb` (`main` after #3774). No code changes in this unit.
  - Between claude-297's pin (`92ea7ccdf`) and this one, `git diff --stat` is empty for `crates/uor-r4-float-train`, `crates/uor-r4-core`, `crates/uor-r4-tokenizer`, `crates/uor-r4-integer`, `docs/labs/scale-probe`, `Cargo.toml` and `Cargo.lock`.
  - So claude-297's sealed (n3L) grades at seeds 1 and 2 are this unit's batch-16 control, read without a rerun (I-r).
- **The recipe:** claude-297's (n3L), with `--batch 64` and `--steps` per arm. The flags are `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --eval-every 1024`, plus `--train-split` D3.
- **The binaries:** `train-in-space` is built on each pod at the pin, with its sha256 recorded. The scale-probe instrument grades on the laptop at the pin.

## Data split

claude-297's, unchanged:
- **TRAIN D3:** `results/claude/scale-probe-2026-11-05/inputs/d3`, `tokens.u16` `ed8f2537…`, `response_mask.u8` `ec6e520c…`, 27,416,554 scored.
- **The panel:** claude-295's four held-out panels, 1,460,697 no-memory rows, identity `5f88a5b3…`.
- **D3's tables:** T2 gets 256,660 panel rows and the unigram 37,421.
- **D3's seen set:** claude-295's `scale-probe seen`, NGRAM 8. It marks 227,312 rows seen and 1,228,450 unseen.
- **DEV:** claude-261's templated DEV, read in-run only and for no bar.

## Arms

All at TRAIN D3, width 256, `--objective nearest`, seeds 1 and 2. Positions trained = steps × batch × 192.

| arm | batch | steps | positions | passes over D3's scored positions |
|---|---|---|---|---|
| (n3L), claude-297, sealed | 16 | 32,768 | 100.7M | 3.67 |
| (b64S) | 64 | 8,192 | 100.7M | 3.67 |
| (b64L) | 64 | 32,768 | 402.7M | 14.69 |

## Readings

For each arm, at seeds 1 and 2:
- **The panel:** whole-table nearest no-memory top-1 (`scale-probe grade`).
- **Against D3's T2 and unigram** (`pair table=t2|unigram`, `clean=` D3's seen set), on all rows and on unseen rows, by sub-panel too.
- **The levers,** cell against cell (`pair`, no table, `clean=` D3's seen set):
  - batch at equal steps: (b64L) against claude-297's (n3L);
  - batch at equal tokens: (b64S) against (n3L);
  - tokens at batch 64: (b64L) against (b64S).

  A lever against (n3L) inherits (n3L)'s own seed spread, so it can read `ahead` or `behind` only on a move of more than about 38,000 rows at both seeds. An `inside` there is not "batch does nothing".
- **The spread** of an arm is the `spread` field of its `pair table=t2` against D3's T2: max − min of d, whole panel, all rows, seeds 1 and 2. It is read beside (n3L)'s 37,975.
- **T2's share:** `model_hits − b` over T2's hits, from the T2 pair.
- **The loss,** read only within an arm, with the parting step: the first progress row (every 1,024 steps) at which the two seeds' `train_loss_mean` differ by more than 0.005. Arms are compared on that step, never on loss values. (n3L)'s parting step is read from claude-297's sealed `progress.jsonl` the same way.
- **Peak GPU memory** of each arm, from `gpu.csv`.

## Bar

Verdict words are `scale-probe pair`'s, at both seeds. Mixed reads as "not distinguishable at these seeds", never as a negative (L-007).

| an arm against D3's T2 (whole panel, all rows) | against D3's unigram | reading |
|---|---|---|
| ahead | ahead | **The one-track bar is met** by that arm. The unseen-row verdict is written beside it in the same sentence: "met; unseen ahead" or "met; unseen behind (the lead is on contexts TRAIN holds)". |
| ahead | not ahead | **Not met.** The grade is checked. |
| mixed or inside | any | **Not distinguishable from T2 at these seeds.** The next unit is chosen by the levers and the spread (below), never by adding seeds to a pair whose band is already its spread. |
| behind | any | **Not met** for that configuration only (§4.1). |

Two arms each get a chance at the bar. The band at both seeds is the guard.

**The levers are not a bar.** Each reads `ahead`, `behind`, `inside` or `mixed`, and says only which way the trunk moves.

## What each outcome unlocks

With two seeds, each spread is one draw, so "narrows" needs two signs at once. **Batch narrows the spread** when both batch-64 arms' spreads are under 18,988 (half of 37,975), **and** (b64L)'s parting step is later than (n3L)'s. **It does not narrow** when (b64L)'s spread is at least 37,975 and its parting step is no later than (n3L)'s. Anything between reads "the spread is not decided at two seeds".

- **An arm meets the bar:** its recipe is the trunk's human-text recipe, carried with its unseen-row qualifier. The next unit is width 1,024 on it, with unseen rows read again there, plus the served-form check (§3.7).
- **An arm reads ahead of T2 but not of the unigram:** the grade is checked before anything else.
- **None meets it; batch narrows the spread; tokens move it** ((b64L) against (b64S) `ahead`): the next unit trains longer still at batch 64, or on more human text.
- **None meets it; batch narrows the spread; tokens do not move it:** the gradient noise is fixed and D3 is spent. The next unit is more human text, or the objective's next change: claude-297's roadmap step 2 (a centroid refit) or step 4 (an address margin), each under its own sheet.
- **None meets it; batch does not narrow the spread:** the spread is not gradient noise. The next unit reads where the seeds part and takes roadmap step 2 or step 4.
- **Anything else,** the spread undecided included: the levers and the parting steps pick the next unit. That choice is recorded as a decision review in the REPORT.

## Instrument checks

- **I-0:** `cargo test -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin.
- **I-1:** each arm's 64-step smoke, at `--eval-every 32`, exits 0 with a finite loss.
- **I-t, the throughput gate (registered here, before training).** It bounds the whole one-card pod, not one arm.
  - **The rate:** r = (train_seconds at step 64 − train_seconds at step 32) / 32, from (b64L)'s smoke. Taking the difference drops step 1's warm-up.
  - **The projection:** a pod's wall is W = 1.05 × r × (steps of (b64S) + steps of (b64L)) + 1,800 s.
    - The 1.05 covers the in-run DEV evaluations: claude-297's training walls ran within 2% of `train_seconds × steps`.
    - The 1,800 s covers setup, scoring and the seal. claude-297's rerun pod spent 1,711 s outside training on four arms.
  - **The rule:** with (b64S) at 8,192 and (b64L) at 32,768 steps, if W ≤ 21,000 s (5.83 h), nothing changes. Otherwise (b64L) runs 16,384 steps. If W still exceeds 21,000 s, (b64S) is NOT_RUN by the gate and (b64L) runs 16,384 steps alone.
  - The choice is recorded in `steps-b64L.txt` and in the REPORT. Nothing else changes.
- **I-3:** D3's and the panels' shas match this sheet. Each run's `config.json` names D3's shas, batch 64, its steps and the recipe.
- **I-4:** every graded checkpoint scores the same 1,460,697 rows (identity `5f88a5b3…`).
- **I-5:** each scored checkpoint's `.i5` reads `same`, and DEV's rows agree across the four per-panel processes.
- **I-s:** D3's seen set names D3's `tokens_sha256` and the panel identity; `pair` refuses otherwise.
- **I-r, the control's standing:**
  - claude-297's (n3L) grades are re-read from `local/claude/claude-297/grade/n3L-s{1,2}`. Their sources are sealed as `runs/s{1,2}` `1c7a0978…` / `dd3cc888…`.
  - Its `pair` against D3's T2 is re-run and must reproduce claude-297's d +16,207 / −21,768 before any lever against it is read.

## Steps

1. The ORDER at the pin; this sheet's FIT; the job script's D67 pre-run review.
2. `uor-pod status` first. Then two Vast pods with one RTX 5090 each, one per seed, at most $1.15/h each, leased, with a watcher armed on each. Each pod's job runs (b64S), then (b64L), on its one card. There is no two-card fallback: two such pods for 6 hours would pass the cap. If no one-card host answers, the launch retries later.
3. On each pod: setup, I-0, I-3, the smokes (I-1), then I-t.
4. Score each final checkpoint with claude-297's `--score-rows` command, one panel per process.
5. Seal each pod to `results/claude/one-model-end-to-end-d3-batch-2026-11-06/runs/s{1,2}/`, read it back, then take the pod down.
6. On the laptop: I-r, then `grade` against D3's tables, then `pair` as in Readings.

## Minimum test power

- On claude-297's (n3L) pair, the McNemar floor is about 1,100 rows (2·√(b + c) with b + c ≈ 300,000). Its band of 37,975 is all spread.
- A lead of 10,000 rows at both seeds, with a spread under 10,000, reads `ahead`.

## Cost

- **Training:** claude-297's batch-16 step took about 0.18–0.20 s.
  - If batch 64 takes up to 4× that (0.8 s), (b64S) is 1.8 h and (b64L) 7.3 h. I-t would then cut (b64L) to 16,384 steps, 3.6 h.
  - At a likelier 0.4 s, (b64S) is 0.9 h and (b64L) 3.6 h.
- **Wall time:** I-t holds each one-card pod's projected wall at 5.83 hours or less, inside the 6-hour one-card exception to the 4-hour cap that claude-297 recorded.
- **Spend:** at most 2 × 6 h × $1.15 = $13.80, under the ORDER's $14. About $9 at a likelier 0.4 s a step. The charter's extend rule holds: a healthy run is never cut for a self-set budget, and any overrun is reported.

## Stop rule

- **Before training:** a failure of I-0 or I-3 stops that pod.
- **An arm fails:** a failed smoke, a non-finite loss or an out-of-memory error makes that arm NOT_RUN at that seed, with the failing step. The other arm goes on.
  - An out-of-memory error at batch 64 is an instrument fault, not a reading. It leaves the Question open, and the next sheet takes batch 64 by gradient accumulation.
  - Width 256 at context 192 and batch 64 is about 3 MB of activations a layer, so this is not expected. The smoke's peak memory is read from `gpu.csv`.
- **A checkpoint fails I-4 or I-5:** its grades are not read, and it is NOT_RUN at that seed.
- **I-r fails:** no lever against (n3L) is read. The arms' bar readings still stand.
- No arm is retuned after it is seen. I-t is decided from the smoke, before training.
- **Export gap:** a recipe adopted from a bar pass owes the served-form check (§3.7) in the next unit, before any serving claim.

## Changes after FIT round 1 (FIT WITH CHANGES: 0 blockers, 4 majors, 6 minors)

1. **Major 1, which arm tests noise.** Equal tokens is a quarter of the updates, so the noise question is read on (b64L) against (n3L), at equal steps. (b64S) is a second reading (Why, Question, Readings).
2. **Major 2, one draw against one draw.** "Batch narrows the spread" now needs both batch-64 spreads under 18,988 and a later parting step for (b64L). The opposite needs (b64L)'s spread at 37,975 or more and no later parting. Anything between is "not decided at two seeds" (What each outcome unlocks).
3. **Major 3, the gate bounds the pod.** I-t projects the whole pod's wall, both arms plus DEV evaluations, setup, scoring and the seal, from claude-297's measured overheads. It cuts (b64L) to 16,384 steps, then drops (b64S), to stay at or under 5.83 h. The price ceiling falls to $1.15/h and the two-card fallback is dropped, so the worst case is $13.80.
4. **Major 4, warm-up.** The rate is the difference between steps 32 and 64 of an `--eval-every 32` smoke.
5. **Minors.**
   - The outcome map now covers every case: ahead of T2 but not of the unigram, narrows without tokens moving it, and a catch-all decision review.
   - The spread is defined.
   - The batch lever's floor of about 38,000 rows is stated.
   - The unseen qualifier is carried to width 1,024.
   - Out of memory is classed as an instrument fault, with a fallback to gradient accumulation.
   - Parting steps are compared, not loss values.
