# Prep sheet: does the averaged objective beat T2 on human text with more steps and more data? (one-model-end-to-end, claude-297, 6 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-297. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit goes to making the trunk learn human text, until the trunk's no-memory top-1 beats its TRAIN's unigram and T2 on the human held-out panel. This is that track's step 3: "whatever it finds, until the bar is met".

TEST FITNESS: FIT WITH CHANGES (round 1, a fresh non-author review, 6 Nov ~07:50Z: no blockers; its four majors and five minors are applied below; see the end).

**Why.** claude-296 (#3763, merged as `92ea7ccdf45d7924f6cd3cac1498d262ee6ce692`) classified claude-295's flat loss on human text.
- **Q1, mechanism, the objective.** Under T2's fixed targets, the same trunk, loader and inputs get 249,488 / 244,254 of the panel's 1,460,697 rows.
- **Q2, (near) moves.** `--objective nearest` on the observed targets gets 216,675 / 223,689: ahead of the control (18,402 / 23,139) and of TRAIN's unigram (37,421), behind T2 (253,018) by 36,343 / 29,329 rows (band 7,014).
- **Still falling.** (near)'s loss went 1.00259 → 0.99367 (s1) and 1.00253 → 0.99295 (s2), and was still falling at step 8,192.
- **A memorisation sign.** On oasst2, (near) is ahead of T2 on all rows and behind it on the unseen rows, whose 8-token context is not in TRAIN (d −1,981 / −1,576, band 405).
- **A literature lead, for the reading only.** For free scores, the multiclass hinge against the single best rival (Crammer–Singer) is reported not to be consistent for top-1 when no class has probability above one half; its conditional minimiser can then be all scores equal (Zhang 2004, JMLR 5; Tewari and Bartlett, COLT 2005 / JMLR 2007). At equal scores either hinge here is exactly 1, where claude-295's cell loss sat (1.0030–1.0039). Nothing here tests it. It is why (cell) is not rerun.

## Question

With (near) unchanged, do more steps (32,768, four times claude-296's) or more data (D3, four times D1) take the trunk past its own TRAIN's T2 on the human panel?
- The 2×2 separates the levers: steps at D1, data at 8,192 steps, and both.
- The one-track bar is met by an arm whose `pair` verdict against its own TRAIN's T2 is `ahead` at both seeds, with `ahead` against its unigram too.

## Base artifact

- **The pin:** `92ea7ccdf45d7924f6cd3cac1498d262ee6ce692` (`main` after #3763). No code changes in this unit.
- **All four cells are trained at the pin,** (n1S) included, so every pair compares one pin, built on each pod. The two pods' `binaries.sha256` may differ, as compiled CUDA builds do; that is expected, not an I-3 failure. Between claude-296's pin (`48c4bf384`) and this one:
  - `uor-r4-float-train`, `uor-r4-core`, `uor-r4-tokenizer` and the scale-probe instrument are unchanged;
  - `uor-r4-integer`, which the trainer links, gains a prompt-reply serving module (`stack/prompt_reply.rs`, `openai.rs`, `uor-chat.rs`).

  So the trainer binary is not claude-296's, which is why (n1S) is rerun.
- **The recipe:** claude-296's (near) exactly: `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 16 --eval-every 1024`, with `--train-split` and `--steps` per arm.
- **The binaries:** `train-in-space` built on each pod at the pin, its sha256 recorded. The scale-probe instrument grades on the laptop at the pin.

## Data split

- **D1:** `results/claude/scale-probe-2026-11-05/inputs/d1`, `tokens.u16` `21cb8d3b…`, `response_mask.u8` `8c2f4d11…`, 6,865,470 scored.
- **D3:** `results/claude/scale-probe-2026-11-05/inputs/d3`, `tokens.u16` `ed8f2537…`, `response_mask.u8` `ec6e520c…`, 27,416,554 scored. D1 ⊂ D3, root by root.
- **The panel:** claude-295's four held-out panels, 1,460,697 no-memory rows, identity `5f88a5b3…`.
- **Tables:** each arm is graded against its own TRAIN's unigram and T2. D1's T2 gets 253,018 panel rows and D3's 256,660; both unigrams get 37,421 (claude-295's grades).
- **Seen sets** (claude-295's `scale-probe seen`, NGRAM 8): D1's marks 129,173 rows seen and 1,326,589 unseen; D3's 227,312 seen and 1,228,450 unseen. D3's unseen rows are a strict subset of D1's.
- **DEV:** claude-261's templated DEV, in-run only, read for no bar.

## Arms

All at width 256, `--objective nearest`, seeds 1 and 2. "Passes" is scale-probe's measure, positions trained over TRAIN's scored positions, so (n3L) makes as many passes as (n1S) over four times the data.

| arm | TRAIN | steps | passes |
|---|---|---|---|
| (n1S) | D1 | 8,192 | 3.67 (claude-296's (near), rerun at the pin) |
| (n1L) | D1 | 32,768 | 14.66 |
| (n3S) | D3 | 8,192 | 0.92 |
| (n3L) | D3 | 32,768 | 3.67 |

## Readings

For every arm, at seeds 1 and 2:
- **The panel:** whole-table nearest no-memory top-1 (`scale-probe grade`).
- **Against its own TRAIN's T2 and unigram** (`pair table=t2|unigram`, `clean=` its own TRAIN's seen set), on all rows and on unseen rows, by sub-panel too.
- **The levers,** cell against cell (`pair`, no table). The D1 arm is always the A side, so `clean=` is D1's seen set for every one; the verdict is read with its sign flipped where the D3 arm is B:
  - steps at D1: (n1L) against (n1S);
  - data at 8,192 steps: (n3S) against (n1S);
  - data at 32,768 steps: (n3L) against (n1L);
  - both: (n3L) against (n1S).
- **T2's share:** `model_hits − b` over T2's hits, from the own-TRAIN T2 pair.
- **The prediction mix** on wiki-00000.
- **The loss,** read only within an arm. `--lr-end 0` decays the learning rate over each run's own `--steps`, so a 32,768-step run at step 8,192 is still at 75% of its rate and is not compared with an 8,192-step endpoint.
- **Record only:** (n1S)'s panel top-1 beside claude-296's sealed (near) at the same seeds, a reproduction across the two pins.

## Bar

Verdict words are `scale-probe pair`'s, beyond the band at both seeds. Mixed reads as "not distinguishable at these seeds", never as a negative (L-007).

| an arm against its own TRAIN's T2 (whole panel, all rows) | against its own unigram | reading |
|---|---|---|
| ahead | ahead | **The one-track bar is met** by that arm. The unseen-row verdict is written beside it in the same sentence: "met; unseen ahead" or "met; unseen behind (the lead is on contexts TRAIN holds)". |
| ahead | not ahead | **Not met** (an arm cannot rank above T2 everywhere and yet trail the unigram; this row is read for completeness and sends the grade to be checked). |
| mixed or inside | any | **Not distinguishable from T2 at these seeds.** The next unit adds seeds to that arm before any other change. |
| behind | any | **Not met** for that configuration only (§4.1). The levers say which way the next unit goes. |

Four arms each get a chance at the bar, a four-way selection; the band at both seeds is the guard, and an arm that meets it is reported with the others' readings beside it.

**The levers are not a bar.** Each reads `ahead`, `behind`, `inside` or `mixed` and only says which lever moves.

## What each outcome unlocks

- **An arm meets the bar:** its recipe is the trunk's human-text recipe. The next unit is width 1,024 on it (scale-probe's run unit 2, parked by the one track), and the served-form check (§3.7).
- **None does, and data moves it** ((n3L) against (n1L), or (n3S) against (n1S), `ahead`): the next unit trains longer at D3, or on more human text.
- **None does, and neither lever moves it:** the objective is next, with a rival-count flag and a hinge that is consistent for top-1, each under its own sheet.

## Instrument checks

- **I-0:** `cargo test -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin.
- **I-1:** each arm's 64-step smoke exits 0 with a finite loss.
- **I-3:** D1's, D3's and the panel's shas match this sheet; each run's `config.json` names its TRAIN's shas, its steps and the recipe.
- **I-4:** every graded checkpoint scores the same 1,460,697 rows (identity `5f88a5b3…`).
- **I-5:** each scored checkpoint's `.i5` reads `same`, and DEV's rows agree across the four per-panel processes.
- **I-s:** D1's and D3's seen sets (`local/claude/claude-295/seen/d{1,3}`) name their TRAIN's `tokens_sha256` and the panel identity; `pair` refuses otherwise.

## Steps

1. The ORDER at the pin; this sheet's FIT; the job script's D67 pre-run review.
2. `uor-pod status` first. Two Vast pods with two RTX 5090s each, one per seed, at most $2.00/h each, if the fleet has room inside the shared caps (4 pods, $8/h). If no two-card host answers for a seed, that seed gets a one-card pod: its job runs the four cells in turn, longest first, in about 4.5 hours, the one exception to the 4-hour cap (6 hours, recorded). Leased, with a watcher armed on each, whose spend stop is the pod's price times 4 hours.
3. On each pod: setup, I-0, I-3, the arms' smokes (I-1).
4. One pod per seed. On each, card 0 runs (n3L), then (n1S); card 1 runs (n1L), then (n3S). Every run trains alone on its card.
5. Score each final checkpoint with claude-296's `--score-rows` command, one panel per process.
6. Seal each pod to `results/claude/one-model-end-to-end-steps-data-2026-11-06/runs/s{1,2}/`, read back, then take the pod down.
7. On the laptop: `grade` against each arm's own TRAIN, then `pair` as in Readings.

## Minimum test power

- claude-296's (near) against T2: band 7,014 on the whole panel. The gap to close is 29,329–36,343 rows, four to five bands.
- Seeds: two a side, as `pair` reads.

## Cost

- **Training:** 32,768 steps at about 0.197 s a step is about 1.79 hours; 8,192 steps about 27 minutes. That rate is claude-296's, DEV evaluations every 1,024 steps included, the same spacing used here. Each card trains 2.24 hours.
- **Wall time:** about 2.8 hours with setup, scoring and the seal.
- **About $10** at two two-card pods near $1.8/h each. The cap is $16 (two pods at most $2.00/h for 4 hours) and 4 hours a pod, with the one-card fallback's 6 hours as the one exception. The charter's extend rule holds: a healthy run is never cut for a self-set budget.

## Stop rule

- **Before training:** a failure of I-0 or I-3 stops that pod.
- **An arm fails:** a failed smoke, a non-finite loss or an out-of-memory error makes that arm NOT_RUN at that seed, with the failing step; the others go on.
- **A checkpoint fails I-4 or I-5:** its grades are not read, and it is NOT_RUN at that seed.
- No arm is retuned after it is seen.
- **Export gap:** a recipe adopted from a bar pass owes the served-form check (§3.7) in the next unit, before any serving claim.

## Changes after FIT round 1 (FIT WITH CHANGES: 0 blockers, 4 majors, 5 minors)

- **M1:** "passes" now uses one measure for every arm, and says (n3L) makes as many passes as (n1S).
- **M2:** the lever readings are `pair` verdicts with one named seen set, D1's, with the D1 arm as the A side; "closes the gap" is gone.
- **M3:** the D1 8,192-step cell is rerun at the pin as (n1S), so every pair compares one binary; claude-296's sealed (near) is a record-only reproduction.
- **M4:** the bar is "`ahead` at both seeds", stated once.
- **m1:** the loss is read within an arm only, with the schedule named.
- **m2:** the "met" reading carries its unseen verdict in the same sentence.
- **m3:** the table maps "ahead of T2, not of the unigram", and names the four-way selection.
- **m4:** I-s checks the seen sets.
- **m5:** the lanes are balanced (2.24 hours a card), the DEV cost is named, and `uor-pod status` comes first.
- **Nit:** "either hinge" is exactly 1 at equal scores.

## Changes after the D67 pre-run review (PASS WITH CHANGES: 2 majors, 3 minors, 2 nits)

- **M1:** the spend stop follows the pod's price (its price × 4 hours, rounded up to the dollar), a host above $2.00/h is refused, and the cap is $16 for the two pods.
- **M2:** Step 2's fallback is a one-card pod for that seed, with its 6 hours stated as the one exception to the cap.
- **m3:** the seal refuses a store path that already holds files.
- **m4:** "one pin, built on each pod"; the two pods' binary shas may differ.
- **m5:** a cross-TRAIN pair runs: claude-295's D1 cells against its D3 cells, `clean=` D1's seen set, exit 0 with verdicts on all rows and on unseen rows.
- **Nits:** the cost line no longer counts the DEV evaluations twice; the watcher's header names the new budget rule.
