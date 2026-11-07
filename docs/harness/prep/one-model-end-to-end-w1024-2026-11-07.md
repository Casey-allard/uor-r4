# Prep sheet: width 1,024 on (b64L)'s recipe (one-model-end-to-end, claude-302, 7 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-302. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit goes to making the trunk's no-memory top-1 beat its TRAIN's unigram and T2 on the human held-out panel, beyond the paired band, at every seed.

TEST FITNESS: FIT WITH CHANGES by a fresh non-author review on 6 Nov, at sheet head `31a213c7`. It is FIT after its Finding 1, the outcome map. That finding and Findings 2 to 6 are applied in this revision.

**Why.** claude-301 (#3800) met that bar with (b64L): width 256, batch 64 as four micro-batches of 16, 16,384 steps on TRAIN D3, "met; unseen ahead". The outcome map of claude-298's sheet, which claude-301's applies word for word, names the next unit: width 1,024 on that recipe, with unseen rows read again. Width 1,024 has never trained on this trainer. In claude-295 it ran out of the 32 GB RTX 5090's memory at batch 16 (31,952 MiB) and at batch 8 (29,456 MiB).

## Question

At TRAIN D3 and (b64L)'s recipe, with only the width changed from 256 to 1,024:
- **Does width 1,024 meet the one-track bar at both seeds?** That is, ahead of D3's T2 and unigram beyond the paired band, with the unseen-row verdict beside it.
- **Does width 1,024 get more held-out rows right than width 256?** Read against claude-301's sealed (b64L), at equal steps and equal tokens.

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, claude-301's. This is the same source pin as (b64L)'s, rebuilt on the PRO 6000 pods. So the width lever compares two builds of one trainer's source, not one binary. The build hashes ptxas output, which is not deterministic, and the toolkit image may differ. Each pod records its `nvcc --version` and the binary's sha256 beside claude-301's, and the REPORT does not claim the binaries are identical.
  - `main` has moved since, by DeepSeek's #3803–#3806. Over the trainer they add `query_cache.rs`, the `fbs-query-cache` binary, and code in `score_rows` and `train-in-space.rs`, all behind `--query-cache-out`. None of it is needed here, and pinning `ff5156bcc` keeps the claim free of it.
- **The recipe:** (b64L)'s, with the width and its heads changed:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 1024 --hamming-read 16 --batch 64 --micro-batch 16 --steps 16384 --eval-every 1024`, plus `--train-split` D3.
  - `--hamming-read` is the width over 64, as in scale-probe's sheet (`flock_read_hamming.rs` `HammingRead::new`). Each Hamming head is 64 wide, and half the heads are copy heads.
  - The learning rate is not retuned. A wider trunk may want a different rate; that is a question for a later unit, not this one (Stop rule).
  - Turn memory, memory dropout and the reply cut stay off, as in (b64L).
- **The binaries:** `train-in-space` is built on each pod at the pin, with its sha256 recorded.
- **The control:** claude-301's sealed (b64L) at seeds 1 and 2, its grades in `local/claude/claude-301/grade/b64L-s{1,2}`, and its seals `runs-2/s1` `f7852f25…` and `runs-2/s2` `1f24f03a…`. (b64L) trained and scored on RTX 5090s, and (w1024) runs on RTX PRO 6000s. The card is therefore a known second difference in the lever. Its likely effect on top-1 is small, through a reduction order that depends on the card's SM count, but it is not zero, and the REPORT names it.

## Data split

claude-301's, unchanged:
- **TRAIN D3:** `tokens.u16` `ed8f2537…`, `response_mask.u8` `ec6e520c…`.
- **The panel:** claude-295's four held-out panels, 1,460,697 no-memory rows, identity `5f88a5b3…`.
- **D3's tables:** T2 gets 256,660 panel rows and the unigram 37,421.
- **D3's seen set:** 227,312 seen, 1,228,450 unseen.

## Arms

| arm | width | heads (Hamming) | batch (micro) | steps | positions |
|---|---|---|---|---|---|
| (b64L), claude-301, sealed | 256 | 4 | 64 (4 × 16) | 16,384 | 201.3M |
| (w1024) | 1,024 | 16 | 64 (4 × 16, or 8 × 8 by I-m) | 16,384 | 201.3M |

Seeds 1 and 2, one pod per seed, each one RTX PRO 6000 (96 GB, sm_120, `--gpu pro6000`).

## Readings

For (w1024), at seeds 1 and 2:
- **The panel:** whole-table nearest no-memory top-1 (`scale-probe grade`).
- **Against D3's T2 and unigram** (`pair table=t2|unigram`, `clean=` D3's seen set): on all rows and on unseen rows, by sub-panel too.
- **The width lever:** (w1024) against (b64L) (`pair`, no table, `clean=` D3's seen set), with (w1024) always the A side. This lever inherits (b64L)'s spread of 10,552, so it reads `ahead` or `behind` only on a move beyond the pair's own band at both seeds.
- **The spread:** the `spread` field of (w1024)'s T2 pair, read beside (b64L)'s 10,552.
- **T2's share:** `model_hits − b` over T2's hits.
- **The loss** within the arm, with the parting step: the first progress row (every 1,024 steps) at which the two seeds' `train_loss_mean` differ by more than 0.005. It is read beside (b64L)'s 3,072.
- **Peak GPU memory,** from `mem.log` within the arm's window.
- **Speed:** training tokens a second, from each run's `train_context_tokens_per_second`, written beside (b64L)'s. The two were measured on different cards, so this comparison is not a width cost.

## Bar

claude-298's Bar, word for word. Its verdict words are `scale-probe pair`'s, at both seeds, and "mixed" reads as "not distinguishable at these seeds", never as a negative (L-007).

| (w1024) against D3's T2 (whole panel, all rows) | against D3's unigram | reading |
|---|---|---|
| ahead | ahead | **The one-track bar is met** by (w1024). The unseen-row verdict is written beside it in the same sentence: "met; unseen ahead", or "met; unseen behind (the lead is on contexts TRAIN holds)". |
| ahead | not ahead | **Not met.** The grade is checked. |
| mixed or inside | any | **Not distinguishable from T2 at these seeds.** |
| behind | any | **Not met** for this configuration only (§4.1). |

**The width lever is not a bar.** It reads `ahead`, `behind`, `inside` or `mixed`, and says only which way the trunk moves.

## What each outcome unlocks

- **(w1024) meets the bar, and the width lever is `ahead`:** width 1,024 becomes the trunk's human-text recipe. The next unit is more human text at width 1,024. That unit is chosen because the tokens lever at batch 64 was `ahead` in claude-301.
- **(w1024) meets the bar, and the lever is `inside` or `mixed`:** width 1,024 is not distinguishable from width 256 here. Width 256 stays the trunk's recipe because it is four times cheaper. The next unit is more human text at width 256.
- **The lever reads `behind`, or (w1024) reads `behind` against T2:** width 1,024 at this recipe is worse, for this configuration only (§4.1). This holds whether or not the bar is met. Width 256 stays the trunk's recipe. Width 1,024 is next read under its own sheet with a learning rate set for its width, and the next main unit is more human text at width 256.
- **(w1024) is `mixed` or `inside` against T2:** not distinguishable from T2 at these seeds, and there is no width reading either way (L-007). Width 256 stays the trunk's recipe, and the next unit is more human text at width 256.
- **(w1024) is ahead of T2 but not ahead of the unigram:** the grade is checked, as the Bar says. That is an instrument step, not a reading. No outcome is drawn until the check is done.
- **(w1024) is NOT_RUN at one seed or both:** no reading, because the bar needs both seeds. Each missing seed is rerun under this sheet, after its gate's cause is recorded.
- **Anything else:** the lever and the spreads pick the next unit. That choice is recorded as a decision review in the REPORT.
- **In every case, the served-form check (§3.7) stays owed** before any serving claim on the adopted recipe (ORDER, Decision).

## Instrument checks

claude-301's I-0, I-1, I-3, I-4, I-5, I-s and I-r, unchanged except as noted. The other gates are registered here, before any pod goes up.
- **I-0** runs the trainer's CUDA tests at the pin, claude-300's three accumulation tests by name among them.
- **I-3** requires `"width": 1024`, `"hamming_read": 16`, `"batch": 64` and the micro-batch I-m chose in each run's `config.json`.
- **I-e, equivalence at width 1,024.** These are claude-301's runs at the new width, on the pod's own seed: four 4-step runs at `--batch 16 --eval-every 1`:
  - (whole);
  - (micro4), four micro-batches of 4;
  - (micro2), eight micro-batches of 2, so that eight forwards are checked directly (the review's Finding 6; it costs seconds);
  - (read0), whole with `--hamming-read 0`.
  - **Pass:** (micro4)'s and (micro2)'s `train_loss_mean` are each within 10⁻⁶ relative of (whole)'s at every step, 1 to 4.
  - **Fail:** the pod trains nothing, and the unit becomes a diagnosis.
  - **The control is recorded, not gated.** In claude-301, I-e on the real recipe passed at width 256 with a powered control, 6.8 × 10⁻⁵ and 1.85 × 10⁻⁵ at step 4. The micro-batch code path does not depend on the width. Here (read0)'s gap is written at every step. If its step-4 gap is under 10⁻⁵, the REPORT says that I-e at width 1,024 was underpowered, and the equivalence rests on claude-301's.
- **I-m, the memory gate and its one fallback.** The smoke is 128 steps of (w1024) at `--micro-batch 16`, `--eval-every 32`. Its peak in `mem.log` within the smoke's window must be under 90% of the card.
  - If the peak is at or over 90%, or the smoke runs out of memory, one second smoke runs at `--micro-batch 8`, eight micro-batches of 8. That second smoke is held to the same 90%.
  - If both fail, (w1024) is NOT_RUN at that seed.
  - The full run uses the micro-batch whose smoke passed.
  - At micro-batch 8 the full run makes eight forwards a step, and I-e's (micro2) checks eight forwards directly. claude-300's CUDA test checks the weighting for any divisor, and the step's loop (`step_gradient`) is one loop for every divisor.
- **I-f, memory flat across steps,** is claude-301's, read on the smoke that passed I-m. It splits at the step-64 row; the peak after the split must be at most the peak before it plus 1,024 MiB.
- **I-t, the time gate, which never cuts steps.** The rate r comes from steps 64 to 128 of the smoke that passed I-m. W = used + 1.05 × r × 16,384 + 3,600 s for scoring at width 1,024.
  - **The scoring term.** Width 256's scoring took 268 s at seed 1 and 351 s at seed 2. That is the sum of the four panels' `score.seconds` for (b64L) in claude-301's seals. At claude-295's ×1.87 a doubling, width 1,024 should score in about 940–1,230 s, so 3,600 s leaves about three times the margin.
  - If W is at most 57,600 s (16 h), the run goes ahead at 16,384 steps.
  - If W passes 57,600 s, (w1024) is NOT_RUN at that seed, and the rate is recorded for the next sheet. With about 2,000 s used, that is r above (57,600 − 3,600 − 2,000) / (1.05 × 16,384) ≈ 3.0 s a step.
  - Steps are never cut. The width lever is read at equal steps and tokens or not at all.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job and watch scripts.
2. `uor-pod status`. Then two Vast pods with one RTX PRO 6000 each, one per seed (`--gpu pro6000`, at most $2.00/h each), leased, each with a watcher.
3. On each pod: setup, I-0, I-3, I-e at the pod's own seed, the smoke or smokes with I-m and I-f, then I-t.
4. The arm (w1024), then the scoring: claude-298's `--score-rows`, one panel per process.
5. Seal each pod to `results/claude/one-model-end-to-end-w1024-2026-11-07/runs/s{1,2}/`, read the seal back, then take the pod down.
6. On the laptop: I-r (claude-301's (b64L) against D3's T2 must give d +75,038 / +85,590), then `grade` and `pair` as in the Readings.

## Minimum test power

The McNemar floor is about 1,100 rows. (b64L)'s band of 10,552 is its seed spread, so the width lever needs a move of more than about 10,000 rows at both seeds.

## Cost

- **The rate, a projection:** (b64L) ran 0.63–0.70 s a step on a 5090. claude-295 measured ×1.97 in step time per width doubling, so width 1,024 should run about 2.5–2.7 s a step. The PRO 6000 is the same Blackwell generation, with more memory. I-t measures the real rate before the full run.
- **The time:** about 2,000 s already used at the gate (the build, I-0, the inputs, I-e at width 1,024 and the smoke), plus 1.05 × 2.6 × 16,384 = 44,700 s, plus 3,600 s for scoring. That is W ≈ 50,300 s, about 14 h, under I-t's 16 h.
- **Spend:** at most 2 pods × 17 h × $2.00 = $68; likelier 2 × 14 h × $1.50 ≈ $42. The ORDER's cap is **$70**. The watchers' stops are set at 17 h and $34 a pod. The charter's extend rule holds: a healthy run is extended, never cancelled.

## Stop rule

claude-301's, with these changes:
- An I-e failure (the micro-batch match, not its control) stops the pod before any training.
- If I-m fails at both micro-batches, or I-f fails, or I-t's W passes 16 h, (w1024) is NOT_RUN at that seed. These are instrument results, recorded with their numbers, and none is a reading of width.
- An out-of-memory error in the full run's training is an instrument fault. (w1024) is NOT_RUN at that seed.
- I-m does not gate `--score-rows` decoding at width 1,024. An out-of-memory error in scoring is also an instrument fault, but it does not make the run NOT_RUN. The final checkpoint is sealed with the run, and it is scored again from the seal on a fresh pod under this sheet, with the same flags.
- **The learning rate is not retuned in this unit.** A width-1,024 loss that does not fall, or falls more slowly than (b64L)'s, is read with the lever and recorded. It is not a reason to change the rate mid-unit.

## Changes from claude-301's sheet

1. The width is 1,024 with `--hamming-read 16`. There is one arm; (b64L) is the sealed control.
2. The card is an RTX PRO 6000 (96 GB). I-m has one registered fallback, to micro-batch 8.
3. I-e runs at width 1,024, with a fourth run, (micro2), that checks eight forwards. Its control is recorded but not gated, and the reason is given.
4. I-t never cuts steps. Its bound is 16 h, and its scoring term is 3,600 s.
5. The Readings, Bar and outcome map are written for one arm and a width lever.
6. The cost and the cap are new: $70 for the unit.
7. The seal path is `results/claude/one-model-end-to-end-w1024-2026-11-07/runs/`.
