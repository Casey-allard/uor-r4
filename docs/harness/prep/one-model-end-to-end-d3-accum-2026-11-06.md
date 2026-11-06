# Prep sheet: batch 64 on D3 by gradient accumulation (one-model-end-to-end, claude-301, 6 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-301. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit goes to making the trunk's no-memory top-1 beat its TRAIN's unigram and T2 on the human held-out panel, beyond the paired band, at every seed.

TEST FITNESS: FIT WITH CHANGES, round 2 (a fresh non-author review of the amendment to I-e after run 1, 6 Nov: no blockers; its two majors and four minors are applied in I-e). Round 1: FIT WITH CHANGES (a fresh non-author review, 6 Nov: no blockers; its three majors and eight minors are applied below).

**Why.** This is claude-298's sheet, [`one-model-end-to-end-d3-batch-2026-11-06`](one-model-end-to-end-d3-batch-2026-11-06.md) (FIT WITH CHANGES, round 1), run again with batch 64 built by gradient accumulation.
- **claude-298 (#3796) could not answer its question.** Every cell at batch 64 ran out of the RTX 5090's 32 GB.
  - The smokes peaked at 32,108 / 31,696 MiB.
  - The full runs climbed from about 22 GB to 31–32 GB, then stopped within 84 s.
  - The verdict was blocked, an instrument fault, and the sheet's Stop rule named gradient accumulation as the next step.
- **claude-300 (#3799) built it.** `--micro-batch M` runs each step's windows forward and backward M at a time, weights each forward's loss by its share of the step's scored rows, and makes one optimizer step on the summed gradients.
  - A CUDA test shows four micro-batches of two give one batch of eight's loss and gradients, to 10⁻⁵ on the loss and 10⁻⁴ of each gradient's largest entry. Weighting each 1/k fails the test, by a relative 1.3 × 10⁻⁴ on the loss.
  - That test is the only check of the gradient's scale. Its fixture does not run the real recipe's Hamming read or served path; I-e below checks the real recipe's forward and loss.
- **What it does not change:** claude-298's question, arms, readings, bar and outcome map all stand. This sheet repeats only what is needed to run, and lists every change from claude-298's sheet under "Changes from claude-298's sheet".

## Question

claude-298's, unchanged. At TRAIN D3 and (near)'s recipe, with only the batch changed from 16 to 64:
- **Is the seed spread gradient noise?** Read on (b64L) against claude-297's (n3L).
- **Does the trunk pass D3's T2 at both seeds?**

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f` (`main` after #3799).
  - Between claude-298's pin (`1f2bd76ce58503f25ef24e415dd193bf57086ddb`) and this one, `crates/uor-r4-float-train` changes only by #3799. The other crates, `docs/labs/scale-probe`, `Cargo.toml` and `Cargo.lock` are unchanged; `git log 1f2bd76c..ff5156bc` over those paths lists only #3799.
  - With no `--micro-batch`, #3799 leaves the step as it was: one forward, no scaling, the same rows, mask and draws. Only `train_seconds` grows slightly, because the batch is assembled inside the step clock. So claude-297's sealed (n3L) at batch 16 stays this unit's control (I-r).
- **The recipe:** claude-298's, plus `--micro-batch 16`:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024`, plus `--train-split` D3.
  - Turn memory, memory dropout and the reply cut are all off in this recipe, so #3799's per-micro-batch memory and reply rows never run. This unit's equivalence claim does not extend to recipes that use them.
- **The binaries:** `train-in-space` is built on each pod at the pin, with its sha256 recorded.

## Data split

claude-298's, unchanged:
- **TRAIN D3:** `tokens.u16` `ed8f2537…`, `response_mask.u8` `ec6e520c…`.
- **The panel:** claude-295's four held-out panels, 1,460,697 no-memory rows, identity `5f88a5b3…`.
- **D3's tables:** T2 gets 256,660 panel rows and the unigram 37,421.
- **D3's seen set:** 227,312 seen, 1,228,450 unseen.

## Arms

claude-298's, each run as batch 64 in four micro-batches of 16. Seeds 1 and 2. On each pod (b64L) trains first, then (b64S).

| arm | batch (micro) | steps | positions |
|---|---|---|---|
| (n3L), claude-297, sealed | 16 | 32,768 | 100.7M |
| (b64L) | 64 (4 × 16) | 32,768, or 16,384 by I-t | 402.7M or 201.3M |
| (b64S) | 64 (4 × 16) | 8,192 | 100.7M |

## Readings, Bar, What each outcome unlocks

claude-298's sheet's sections of these names apply word for word, with its numbers: (n3L)'s spread 37,975, the narrowing threshold 18,988, and the lever floor of about 38,000 rows. Two additions to the outcome map:
- **If I-t cuts (b64L) to 16,384 steps,** the equal-steps reading of the noise question is lost. (b64L) then makes half (n3L)'s updates at four times its batch. The spread is still read and the bar still applies. "Batch narrows the spread" may then be read only as "a larger batch at half the updates narrows it", and the REPORT says so.
- **(b64S) is read only if it ran at both seeds.** Each pod applies I-t to its own rate, and I-t drops (b64S) near the expected rate (above about 0.70 s a step), so it may run at one seed only. Without (b64S) at both seeds, "batch narrows the spread" is read on (b64L) alone: its spread under 18,988, and its parting step later than (n3L)'s. The REPORT names which reading applied.

## Instrument checks

claude-298's I-0, I-1, I-t, I-3, I-4, I-5, I-s and I-r, unchanged except as noted, plus three new gates. They are registered here, before any pod goes up.
- **I-0** includes claude-300's three tests by name, as passed: `micro_batches_weighted_by_their_scored_rows_give_the_whole_batchs_step` (CUDA), `a_micro_batch_is_refused_unless_it_divides_the_batch_and_is_written_only_when_set` and `each_micro_batch_names_the_whole_batchs_rows_at_its_own_place`.
- **I-3** also requires `"micro_batch": 16` in each run's `config.json` (the field is left out when it is 0).
- **I-e, equivalence on the real recipe.** Three 4-step runs at TRAIN D3, at the pod's own seed (round 2), `--batch 16 --eval-every 1`, on the same card, one after the other. Batch 16 is the batch that fits whole. Seed 1's pod replays run 1's rows on a new host; seed 2's pod tests rows the gate was not set on, and is the run that tests the amended gate.
  - **The runs:** (whole) with no `--micro-batch`, (micro4) with `--micro-batch 4`, and the sensitivity control (read0), whole with `--hamming-read 0`.
  - **What each row tests:** the row at step 1 is the loss before any update. It tests the forward and the loss weighting across micro-batches. The rows at steps 2 to 4 test the gradients of the first three steps, but only through AdamW's update, which ignores the gradient's scale; the scale is pinned only by claude-300's CUDA test.
  - **Why the read is silent early.** The read's output weight starts at zero (`Var::zeros((2, width))`, `flock_read_hamming.rs` ~993), so at step 1 the read adds exactly 0 to the output, and its rotations and margins get no gradient through it. Their first effect on the loss is at step 3. At step 2 the read's whole effect in run 1 was 7.0 × 10⁻⁷, under the 10⁻⁶ tolerance. So steps 1 and 2 test the rest of the forward and the loss weighting, and the gate's power over a read that crosses windows rests on steps 3 and 4: in run 1 the read's effect was 10× the tolerance at step 3 and 68× at step 4. The REPORT gives (read0)'s gap over the tolerance at every step.
  - **Pass:** (micro4)'s `train_loss_mean` is within 10⁻⁶ relative of (whole)'s at every step, 1 to 4. The measured gaps are recorded.
  - **The control:** (read0)'s step-4 loss must differ from (whole)'s by at least 10⁻⁵ relative, 10× the tolerance. If it does not, the read barely moves the loss within four steps, and I-e is too weak to see a read that crosses windows. I-e then fails as underpowered.
  - **Fail:** the micro-batched forward of the real recipe is not window-independent, or I-e is underpowered. Either way the pod trains nothing and the unit becomes a diagnosis.
  - **Run 1 and the amendment (round 2).** As first registered, I-e held (micro4) to 10⁻⁵ at step 1 and 10⁻⁴ at steps 2 to 4, and the control to 10⁻⁴ at step 1. Run 1 (both pods, 6 Nov, sealed at `runs/s{1,2}/`, $0.38) stopped at I-e as underpowered, with these rows, identical to every digit on both hosts:
    - (micro4) against (whole): 1.3 × 10⁻⁸, 8.4 × 10⁻⁹, 8.9 × 10⁻⁸ and 4.8 × 10⁻⁹ relative at steps 1 to 4.
    - (read0) against (whole): 0 at step 1, then 7.0 × 10⁻⁷, 1.0 × 10⁻⁵ and 6.8 × 10⁻⁵ at steps 2 to 4.
    - The read leaves the loss before the first update unchanged to the last bit, so a step-1 control could never pass. That was a fault in the control's design (instrument), not a reading of the mechanism.
    - The amendment sets the control at step 4, the last step and the one where the read has moved the loss most; at step 3 run 1's control would read 9.97 × 10⁻⁶, just under 10⁻⁵. It tightens the tolerance to 10⁻⁶ at every step, 11× above run 1's largest gap. Both hosts agreeing to every digit shows the run is deterministic, so the gap is not host noise.
    - It is calibrated on run 1's I-e rows, which are an instrument's rows, not the unit's outcome: no arm trained and nothing was graded, and the tolerance moved tighter, not looser. Seed 1's pod in run 2 re-reads those rows, so its pass is run 1's reading read again; seed 2's pod is the new test. The REPORT says which is which, and gives the margins on each pod's rows.
- **I-m, the memory gate.** It is read from each arm's smoke: the largest memory used in `mem.log` within that smoke's start and end times, against the card's total, both recorded in each sample. `mem.log` samples the card every 2 s, with the line count of the running smoke's progress file.
  - **Pass:** the peak is below 90% of the card.
  - **Fail:** that arm is NOT_RUN at that seed, before its full run.
- **I-f, memory flat across steps.** It is read from (b64L)'s smoke, which runs 128 steps at `--eval-every 32`, in place of claude-298's 64.
  - The samples are split at the time of the smoke's step-64 progress row. Both halves contain evaluations; the first also contains startup.
  - **Pass:** the peak after the split is at most the peak before it plus 1,024 MiB.
  - **Fail:** memory grows with steps. Both arms are NOT_RUN at that seed.
  - **What it cannot see:** over about 64 steps, 1,024 MiB detects growth of about 16 MiB a step or more. Slower growth passes I-f and can still fill the card over 16,384 steps. The backstop is the run's own out-of-memory stop, which makes that arm NOT_RUN by the stop rule; the other arm still runs. The job has no stop of its own on memory.
- **I-t** takes its rate r from steps 64 to 128 of (b64L)'s smoke, over 64 steps. Its projection adds the time the job has already used at the gate: W = used + 1.05 × r × (the steps left to train) + 1,800 s. The rule, the 1.05 factor, the 1,800 s for scoring and the 21,000 s bound are claude-298's.
  - If W with (b64L) alone at 16,384 steps still passes 21,000 s (r above about 1.05 s), (b64L) runs 16,384 steps anyway. The watcher's time and spend stops then fire, and the charter's extend rule applies.

## Steps

1. The ORDER at the pin; this sheet's FIT; the D67 pre-run review of the job and watch scripts.
2. `uor-pod status`. Then two Vast pods with one RTX 5090 each, one per seed, at most $1.15/h each, leased, each with a watcher. There is no two-card fallback.
3. On each pod: setup, I-0, I-3, I-e (at the pod's own seed), the smokes with I-m and I-f, then I-t.
4. The arms, (b64L) then (b64S), then the scoring: claude-298's `--score-rows`, one panel per process.
5. Seal each pod to `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s{1,2}/` (run 1 holds `runs/s{1,2}/`, never written over), read the seal back, then take the pod down.
6. On the laptop: I-r, then `grade` and `pair` as in claude-298's Readings.

## Minimum test power

claude-298's. The McNemar floor is about 1,100 rows; (n3L)'s band of 37,975 is all spread.

## Cost

- **The rate:** claude-298's smokes measured 0.564 / 0.540 s a step at whole batch 64, before running out of memory. Four forwards of 16 should cost about the same compute, plus four launches of each kernel instead of one: about 0.6–0.8 s a step.
- **The time already used at the gate:** about 1,200 s (the build and I-0, the inputs, I-e and the two smokes).
- **At 0.65 s,** W = 1,200 + 1.05 × 0.65 × (8,192 + 32,768) + 1,800 = 30,955 s, which is over 21,000. So (b64L) runs 16,384 steps, and W = 19,773 s.
- **At 0.8 s,** W with (b64L) at 16,384 is 23,643 s, still over. So (b64S) is NOT_RUN by the gate, and (b64L) runs 16,384 steps alone, W = 16,763 s.
- **Spend:** at most 2 × 6 h × $1.15 = $13.80, within the ORDER's $14; likelier about $10. The charter's extend rule holds.

## Stop rule

claude-298's, with these changes:
- An I-e failure stops the pod before any training.
- An I-m or I-f failure makes the arm, or both arms, NOT_RUN at that seed, as I-m and I-f say.
- An out-of-memory error at batch 64 by accumulation is again an instrument fault. It makes that arm NOT_RUN at that seed, and the other arm still runs. It leaves the question open, and the next unit reads where the memory goes before trying again.

## Changes from claude-298's sheet

1. The pin is `main` after #3799, and the recipe adds `--micro-batch 16`. Base artifact records that turn memory, dropout and the reply cut are off.
2. New gates I-e (with its control), I-m and I-f. I-0 names claude-300's three tests, and I-3 requires `micro_batch` 16.
3. (b64L)'s smoke runs 128 steps, and I-t's rate is read from its steps 64 to 128. I-t's projection adds the time already used, and says what happens above about 1.05 s a step.
4. (b64L) trains before (b64S) on each pod.
5. The outcome map's notes: reading the spread when I-t cuts (b64L) to 16,384 steps, and reading "narrows" on (b64L) alone when (b64S) did not run at both seeds.
6. The Cost section's model, and the out-of-memory line in the Stop rule.
7. The seal path is `results/claude/one-model-end-to-end-d3-accum-2026-11-06/`: run 1 at `runs/`, the run after the round-2 amendment at `runs-2/`.
8. Round 2: the control of I-e at step 4, its tolerance 10⁻⁶ at every step, and I-e at the pod's own seed (see I-e).
