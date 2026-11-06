# Prep sheet: does the trunk learn human text? The objective against fixed targets (one-model-end-to-end, claude-296, 6 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-296. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit goes to making the trunk learn human text, until the trunk's no-memory top-1 beats its TRAIN's unigram and T2 on the human held-out panel.

TEST FITNESS: FIT (round 2, a fresh non-author review, 6 Nov 02:33Z: no blockers; its one major and four minors are applied below. Round 1 was NOT FIT; see the end.)

**Why.** claude-295 (scale-probe, #3472) trained the one model on human-written text for the first time; every corpus before it was the templated M-world.
- **The loss did not fall.** At (256, D1), `train_loss_mean` is 1.00386 at step 1,024 and 1.00335 / 1.00302 at step 8,192 (seeds 1 / 2). The same recipe on templated text falls from 1.00280 to 0.82460.
- **The panel.** On the 1,460,697-row human panel, (256, D1) is right on 18,402 / 23,139 rows. Its TRAIN's unigram gets 37,421 and T2 253,018.
- **What the trunk predicts** is the targets' token mix spread across contexts. On wiki-00000, w256-d1-s1 is right on 4,620 rows; a context-blind guess with the same mix gets 2,275, and always naming TRAIN's commonest token (201, the unigram's guess) 6,851. The panel's own commonest target there, token 14, is 10,845 rows (claude-295 REPORT).
- **The lead, a hypothesis.** The objective, `relu(1 + max_r s(r) − s(target))` against the whole table's single nearest wrong token (`in_space_long.rs:1089-1093`, `1104-1133`), pushes the query away from its nearest token whenever that token is not this row's target.
  - Where the next token is nearly fixed by the context, as in templated text, the query can settle on it.
  - Where many tokens can follow, even the likeliest one is "not the target" on most rows, and the query may have no point to settle on.

## Question

1. **Instrument or mechanism (§4.2).** Give the same trunk, recipe, loader and inputs a fixed target per context: T2's own pick, the commonest token after the previous one in TRAIN. Does the trunk then predict human text's real next tokens better than the control and than the unigram?
   - **Yes:** target alignment holds, and the trunk uses at least the last input token on human text. claude-295's flat loss is then the objective under the real targets, a mechanism of the objective.
   - **No:** the fault is upstream of the objective, in the loader, the mask, the inputs or the trunk. That is an instrument candidate.
   - A pass does not show that the trunk uses longer context. It also cannot tell ambiguity apart from the smaller set of distinct targets, which is reported (I-m).
2. Does the real-target failure move with the learning rate (0.003), or with averaging the hinge over 8 rivals (`--objective nearest`)?

## Base artifact

- **The pin:** this unit's head, which carries `--train-targets` (Code, below).
- **The recipe:** claude-295's `RECIPE` exactly, `--objective cell --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192`, with `--width 256 --hamming-read 4 --batch 16 --steps 8192 --eval-every 1024` and `--train-split` D1 with its two shas. Each arm changes one thing.
- **The control:** claude-295's sealed (256, D1) runs at seeds 1 and 2 (`results/claude/scale-probe-2026-11-05/runs/A`). Their grades are `grade/w256-d1-s{1,2}`, made by the same instrument on the same panel. They are not rerun.
- **The binaries:** `train-in-space` and `fbs-snapshot` are built on the pod at the pin, with their sha256s recorded. The scale-probe instrument (`docs/labs/scale-probe/instrument`) grades on the laptop at the pin.

## Data split

- **TRAIN** is D1, from `results/claude/scale-probe-2026-11-05/inputs/d1`: `tokens.u16` `21cb8d3b…` and `response_mask.u8` `8c2f4d11…`. It has 6,865,470 scored targets.
- **The fixed targets** are computed from TRAIN only, inside the trainer, before step 0. The panel is never read for them.
- **The panel** is claude-295's four held-out panels, 1,460,697 no-memory rows, I-4 identity `5f88a5b3…`:

  | panel | tokens | mask |
  |---|---|---|
  | oasst2 | `02a04e97…` | `9c5bb656…` |
  | wiki-00000 | `82b3a40c…` | `34fe6b68…` |
  | wiki-00020 | `9b7f1432…` | `a6803ab3…` |
  | wiki-00040 | `c927243b…` | `9d95ea6d…` |

  Every arm is graded against the panel's **real** next tokens, so all arms and the control compare like with like.
- **DEV** is claude-261's templated DEV, in-run only and read for no bar.

## Arms

All arms are at (256, D1), at seeds 1 and 2, because `scale-probe pair` reads two seeds a side.

| arm | change from the recipe |
|---|---|
| (c) | none: claude-295's sealed runs, not rerun |
| (m1) | `--train-targets mode:1` |
| (lr3) | `--lr 0.003` |
| (near) | `--objective nearest` |

**`mode:K`, the fixed target.** The code takes K from 1 to 4; this unit runs only K = 1.
- **What the counts are built from.** The whole split's `train_tokens` and `train_mask`, before `load_train` drops windows with fewer than 4 scored targets. The replacement then goes through each usable window's `scored_corpus_positions`, so a target's fixed value never depends on which windows survive the filter.
- **Eligible positions.** A scored TRAIN position `p` is one past its document's BOS at `d` (`p > d`), whose token is not BOS and whose mask is 1. This is `build_document_windows`'s rule (`hamilton_sheet.rs:1870`).
- **The contexts.** For `k` from `min(K, p − d)` down to 1, `s_k` is the `k` tokens before `p`. The context may include the document's own BOS, as T2's previous token does at a document's first target, and never reaches past it.
- **The counts.** `count(s_k, t)` counts **scored positions only**: those whose `k` preceding tokens are `s_k` and whose token is `t`. **`p` counts itself.**
- **The target at `p`** is the `t` with the largest `count(s_k, t)`, at the largest `k` where `s_k` occurs at 2 or more scored positions, `p` included. At `k = 1`, one occurrence is enough. Ties go to the smallest id.
- **So `mode:1` is T2's own pick.** It follows the same tie and scope rules as the scale-probe instrument's T2 (`docs/labs/scale-probe/instrument/src/lib.rs:380-443`).
- **What stays unchanged:** inputs, windows and masks. Only the scored targets the loss reads are replaced.

## Readings

For every arm, at seeds 1 and 2:
- **The panel:** whole-table nearest no-memory top-1 against the real targets (`scale-probe grade`).
- **The paired readings** (`scale-probe pair`, the scale-probe sheet's band rule, `clean=` D1's seen set), on all rows and on unseen rows:
  - the arm against the control (`b1`, `b2` = claude-295's `w256-d1-s{1,2}`);
  - the arm against the unigram (`table=unigram`);
  - the arm against T2 (`table=t2`).
- **Where T2 is right, how often the arm is too:** `model_hits − b` from the T2 pair, over T2's own hits. `b` counts the rows where the arm is right and T2 wrong (`lib.rs:979`), so `model_hits − b` counts the rows where both are right. This is how much of T2's pick the arm learned. It is read for every arm and for the control, on all rows only: the pair JSON has no `model_hits` on unseen rows.
- **The panel's best constant guess:** how many of the 1,460,697 rows have the panel's own commonest real target, counted from the sealed rows. A trunk that always names one token can beat TRAIN's unigram by naming this one (on wiki-00000, token 14 is right on 10,845 rows, against TRAIN's unigram token 201 on 6,851 and w256-d1-s1 on 4,620), so Q1 reads against it too.
- **The prediction mix** on wiki-00000: the distinct tokens predicted, the hits, and the hits a context-blind guess with the same mix would get. This is the claude-295 REPORT's diagnostic.
- **The loss** (`train_loss_mean`), read only within an arm, from step 1,024 to step 8,192. (m1)'s loss is against its own targets, so it is never compared with another arm's.

## Bar

Every verdict word is `scale-probe pair`'s: ahead, behind, inside or mixed, beyond the band at both seeds. Mixed reads as "not distinguishable at these seeds", never as a negative (L-007).

**Q1, read on (m1).**

| (m1) against the control | (m1) against the unigram | reading |
|---|---|---|
| ahead | ahead, and above the panel's best constant guess at both seeds | **Mechanism, the objective.** Under fixed targets the trunk predicts human text's real next tokens beyond the control, TRAIN's unigram and any single constant token. No one-token answer gets there, so this shows use of the input. claude-295's flat loss is then the objective under the real targets. The next unit is an objective that settles on the likeliest token, or the longer-context targets with their table's own ceiling. |
| ahead | ahead, but not above the best constant guess at both seeds | **Partial.** (m1) beats TRAIN's unigram but could do it by naming one token, so the call is not made. The next unit reads (m1)'s prediction mix first. |
| ahead | inside, behind or mixed | **Partial.** Fixed targets lift the trunk but not past the unigram, so the call is not made. The next unit inspects both the inputs on human text and the objective. |
| inside or mixed | ahead | **Not distinguishable from the control.** The control is behind the unigram, so this happens only when the band against the control is very wide. No call is made; the next unit adds seeds to (m1) and the control. |
| inside, behind or mixed | inside, behind or mixed | **Instrument candidate.** Before any objective work, the next unit inspects the loader, the mask and the trunk's inputs on human text. |
| behind | ahead | **Instrument candidate**, as above. (m1) behind the control yet ahead of the unigram would itself contradict the control's place behind the unigram, so the pair readings are checked first. |

**Q2, for each of (lr3) and (near).**
- **Moves:** ahead of the control.
- **Reaches the unigram:** ahead of the unigram, reported separately.
- **Does not move:** inside, behind or mixed against the control. That closes the configuration only (§4.1).

**Not read here.** The one-track bar, beating T2, is out of reach in this unit:
- (m1) is bounded by T2, since T2's pick is what it learns;
- no arm here uses longer context.

The longer-context targets (`mode:3`) move to the next unit, which first measures the `mode:3` table's own panel top-1 on the laptop as the ceiling. A trunk beating T2 under `mode:3` would reproduce a TRAIN backoff table, and is claimed as that.

**§3.1.** (m1) is a diagnostic. Adopting count-table targets as a training recipe needs its own decision card, which argues §3.1. This sheet adopts nothing.

## Instrument checks

- **I-0:** `cargo test -p uor-r4-float-train` passes on the pod at the pin, the new tests included, or nothing trains.
- **I-1:** each arm's 64-step smoke exits 0 with a finite loss.
- **I-3:** D1's and the panel's shas match this sheet.
- **I-m:** for (m1), the trainer prints:
  - the split's scored positions, counted before the window filter. This must equal 6,865,470, D1's scored count on the scale-probe sheet, or the run stops before training;
  - the scored targets in the usable windows, which the loss reads;
  - how many of those it replaced;
  - the distinct targets before and after.
  - For K ≥ 2 it also prints, per k, the targets whose window offset is below k − 1, so the trunk cannot see their whole context. This does not apply to K = 1.
  - The unit tests pin, on a hand-made corpus:
    - the self-count;
    - counting scored positions only;
    - backoff at fewer than 2 occurrences;
    - the BOS and the document boundary;
    - ties;
    - that unscored and BOS positions are never replaced;
    - that `observed` leaves every window byte-identical;
    - that the counts come from the whole split, not the usable windows: a position in a dropped window still counts;
    - that at every scored position, `mode:1`'s target equals a brute-force T2 written in the test: the commonest scored token after `tokens[p − 1]`, ties to the smallest id.
- **I-4:** every graded checkpoint scores the same 1,460,697 rows (identity `5f88a5b3…`).
- **I-5:** each scored checkpoint's `.i5` reads `same`.

## Steps

1. On the laptop: the new tests and a stub-CUDA `cargo check` at the pin, then commit.
2. This sheet gets FIT round 2, and the job script gets the D67 pre-run review.
3. One RTX 5090 on Vast (`uor-pod`, a verified datacenter host), leased, with a watcher armed.
4. On the pod at the pin: setup, I-0, I-3, then the three arms' 64-step smokes (I-1).
5. Train the six runs two at a time, side by side on the one GPU (P3(a)): (m1) s1 with (m1) s2, then (lr3) s1 with s2, then (near) s1 with s2.
6. Score each step-8,192 checkpoint with claude-295's exact `--score-rows` command, one panel per process, as `rescore-295.sh score` does.
7. Seal to `results/claude/one-model-end-to-end-human-text-2026-11-06/`, read back, then take the pod down.
8. On the laptop: `scale-probe grade` against D1's tables, then `pair` as in Readings.

## Minimum test power

- **The floors on the panel's 1,460,697 rows.** The floor is `⌈2·√(b+c)⌉`. claude-295's (256, D1) seed 1 had b+c = 46,401 against the unigram, so the floor was 431 rows.
  - Against the control, the seed spread is |(A1 − A2) − (C1 − C2)| (`lib.rs:930`), with C1 − C2 = −4,737 (18,402 / 23,139). The band is therefore about 5,000 rows if the arm's two seeds land close together, and up to about 10,000 if they differ as much as the control's do but the other way round. Q1 and Q2 stay decidable either way.
  - Reaching the unigram from the control is a gain of 14,282–19,019 rows: 1.5 to 4 bands, depending on the arm's own seed spread.
  - (m1)'s ceiling is T2's 253,018.
- **Seeds:** two a side, as `pair` reads.

## Code in this unit

- **`--train-targets observed|mode:K`**, with K from 1 to 4. `observed` is the default and leaves training as it was.
  - It goes in `crates/uor-r4-float-train/src/bin/train-in-space.rs` and `in_space_long.rs`: `LongConfig::train_targets`, applied in `load_train` after `build_document_windows`. It is recorded in `config.json`.
  - It is refused with the turn memory, the reply table and `--score-rows`: it is a training input only.
- **The I-m print and the tests** listed under Instrument checks.

## Cost

- **One RTX 5090** at about $0.563/h, for about 2 hours:
  - setup, about 15 minutes;
  - three pairs of runs side by side, about 25 minutes each;
  - six scorings, about 3 minutes each;
  - the seal.
- **The total** is about $1.20, capped at $3 and 4 hours. Arms still running past the cap are NOT_RUN.

## Stop rule

- **Before training:** a failure of I-0, I-3 or I-m stops everything. It is an instrument item.
- **An arm fails:** a failed smoke (I-1), a non-finite loss or an out-of-memory error makes that arm NOT_RUN, with the failing step. The other arms go on.
- **Side by side runs out of memory:** the pair goes back to one at a time, recorded.
- **A checkpoint fails I-4 or I-5:** its grades are not read and the arm is NOT_RUN at that seed. It is never graded around.
- No arm is retuned after it is seen.
- **Export gap:** no claim here builds on these arms, so none is owed. Any recipe adopted later owes it (§3.7).

## Changes after FIT round 1 (NOT FIT: 2 blockers, 3 majors, 4 minors)

- **Blocker 1: the loss compared across different targets.** The deciding readings are now panel pairs against the real targets, and the loss is read only within an arm. Q1's outcome table is complete.
- **Blocker 2: Q2 tested "fixes".** "Moves" is now ahead of the control; reaching the unigram is reported separately.
- **Major 3: Q3 had no ceiling.** Q3 and (m3) move to the next unit, with the mode table's own panel ceiling. (m1)'s bound by T2 is stated.
- **Major 4: Q1's "yes" overclaimed.** Q1's "yes" is reworded to target alignment and use of at least the last input token, and the distinct targets are reported.
- **Major 5: §3.1.** The adoption line is removed, and adopting needs a decision card.
- **Minors.** Self-counting and scored-only counting are stated and pinned in tests. The hidden-context count for K ≥ 2 is added. The stop rule covers I-1, I-4 and I-5, and mixed seeds. The export gap line is added.
- **The arms.** All arms run at two seeds, because `pair` reads two a side. (m3) moved out, which keeps six runs.

## Changes after FIT round 2 (FIT: 0 blockers, 1 major, 4 minors)

- **Major: what the counts are built from.** They come from the whole split, before the window filter, and the replacement goes through `scored_corpus_positions`. I-m prints both the split's scored total (6,865,470) and the usable windows' total. A test pins `mode:1` to a brute-force T2 at every scored position.
- **Minor: the band.** The band against the control is |(A1 − A2) − (C1 − C2)|, not the control's spread alone, so it runs from about 5,000 to about 10,000 rows.
- **Minor: the collapse claim.** One constant token can beat TRAIN's unigram, so Q1 also reads against the panel's best constant guess, and the table has a row for it.
- **Minor: unseen rows.** The where-T2-is-right reading is on all rows only.
- **Minor: the misfiled cell.** Inside or mixed against the control but ahead of the unigram now reads as "not distinguishable from the control".
