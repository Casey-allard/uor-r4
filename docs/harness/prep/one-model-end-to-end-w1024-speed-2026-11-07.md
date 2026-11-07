# Prep sheet: width 1,024 trained, at the fastest micro-batch and a 20-hour wall (one-model-end-to-end, claude-304, 7 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-304. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438).

TEST FITNESS: FIT WITH FIXES by a fresh non-author review on 7 Nov, at sheet head `1b0861aa4`. All seven findings are applied in this revision:
1. the watcher's money stop moves to $44, so that the 21 h stop acts first;
2. a smoke with no memory sample drops its size instead of stopping the smokes;
3. sizes 64 and 32 keep 20% memory headroom, and a long-run out-of-memory error is named among the outcomes;
4. the micro-batch is named as a difference in the width lever;
5. the rate the bound admits, and the thin margin, are stated;
6. the scoring-term and card wording is corrected, and Casey's words are cited;
7. the job's stale comment is fixed.

The D67 pre-run review (GO WITH FIXES) found the same watcher and sampler faults. It also found that a smoke run where no size was ever read would be recorded as an I-m outcome. That case is now an instrument fault, which ends the job unregistered (exit 6).

**This sheet amends [claude-302's](./one-model-end-to-end-w1024-2026-11-07.md), as [claude-303's](./one-model-end-to-end-w1024-rerun-2026-11-07.md) did.** claude-302's Question, Data split, Arms, Readings, Bar, outcome map, Minimum test power, I-0, I-3, I-f, I-s, I-r and Stop rule all apply word for word, except where a numbered change below says otherwise. claude-303's I-0 test, I-e tolerance (10⁻⁵) and memory diagnosis apply as claude-303's sheet set them. The seeds, the card and the price cap are claude-302's.

**Why.** claude-303 (#3809) repaired width 1,024's memory. I-m passed at micro-batch 16 at both seeds, 38,621 and 38,571 of 97,887 MiB. Then I-t stopped both seeds:
- at 3.578 and 3.636 s a step, W was 66,641 and 67,618 s, against claude-302's bound of 57,600 s;
- claude-302's sheet had projected 2.5–2.7 s a step from claude-295's ×1.97 per width doubling, but the real rate is about 1.4 times that.

Two levers remain that leave the run's steps and tokens alone: a faster step, and a longer wall. This sheet registers both before any run.

## Changes from claude-302's and claude-303's sheets

1. **The smokes: micro-batch 64, 32 and 16, the fastest kept.** claude-302's I-m smoke and its one fallback (16, then 8) are replaced.
   - Three smokes run in this order: micro-batch 64 (the whole batch, one forward a step; `train-in-space.rs`'s doc: "M equal to `--batch` is one forward"), 32, and 16. Each is 128 steps of (w1024) at `--eval-every 32`.
   - **Each smoke passes or drops out.** It must pass I-1 and I-m, its peak in its own `mem.log` window under 90% of the card; at micro-batch 64 and 32 the bound is 80%, so that a size chosen for speed keeps memory headroom for the 16,384-step run that 128 steps cannot show. It must also pass I-f, its peak after the step-64 row at most 1,024 MiB above its peak before. Its rate is (train seconds at step 128 − at step 64) / 64, and must be readable.
   - **An out-of-memory error,** a miss on any of these, or a memory window with no reading, drops that size, and the next smoke runs.
   - **An I-1 failure that is not out of memory** may be a script fault. It stops the smokes, and (w1024) is NOT_RUN at that seed by I-1.
   - **The size that trains** is the one with the smallest rate; on equal rates, the larger one. If no size is left, (w1024) is NOT_RUN at that seed by I-m.
   - **Why these sizes.** At micro-batch 16, peak memory was 38.6 of 97.9 GB. At batch 16, splitting into micro-batches lowered the peak by only 224 MiB (seed 2) to 8,768 MiB (seed 1). Fewer forwards a step may run faster, but claude-303 measured neither 32 nor 64, so this sheet predicts no rate. Micro-batch 16 is kept as the last size, and it is known to pass I-m.
   - **What it costs.** Three smokes take about 25 minutes, against claude-302's one. It is counted in I-t's "used".
2. **I-t's bound: 72,000 s (20 h), up from 57,600 s (16 h).** W is claude-302's: used + 1.05 × rate × 16,384 + 3,600 s for scoring. Over 72,000 s, (w1024) is NOT_RUN at that seed and the rate is recorded. Steps are still never cut. This is a stop-rule change, so it is a decision card, decided by the review below.
   - **What it admits.** With about 3,000 s used by the three smokes, the bound passes any rate up to (72,000 − 3,600 − 3,000) / 17,203 ≈ 3.80 s a step.
   - **The margin is thin.** At claude-303's rates (3.578 and 3.636), micro-batch 16 passes on this change alone, by only 4–6%; a host about 5% slower stops it again. The smokes are the margin, not only a speed-up.
3. **The pin is `main` at claude-303's merge,** `2578cd6b4d10d2d2beae24dcdc6692424d56e1f0`.
   - Its trainer is claude-303's pin `fd90aa1c16e2af35d7fa52d101726e7fe6927953`, byte for byte: `git diff fd90aa1c 2578cd6b -- crates/` is empty.
   - claude-303's branch is deleted, so `main` is what the pods can fetch.
   - The width lever compares builds, as claude-302's Base artifact says. The REPORT names every difference from (b64L)'s run: the build (`variables_only`, and DeepSeek's #3803–#3806, which run only under `--query-cache-out` and which no run passes), the card, and the micro-batch. (b64L) trained 4 × 16; if micro-batch 64 wins, (w1024) trains one forward a step. The trainer's doc (`train-in-space.rs`: "0, the default, or M equal to `--batch` is one forward, as before") makes 64 the no-flag path. claude-303's I-e compared micro 4 and 2 with the whole batch at batch 16 and found the losses equal within 1.84 × 10⁻⁶.
4. **The seal path** is `results/claude/one-model-end-to-end-w1024-speed-2026-11-07/runs/s{1,2}/`. No earlier seal is touched (L-002).

### The decision review for Change 2 (the wall)

I read the whole chain before deciding; Casey can reverse this. The points:
- **Where 57,600 s came from.** claude-302's sheet set it as headroom over a projected W of about 50,300 s (14 h). It came from claude-302's cost section, not from any owner cap or decision.
- **The owner's caps.** At most 4 pods and $8/h across all labs. This run uses 2 pods at at most $2.00/h each, so $4/h. No owner rule caps a run's hours. AGENTS.md says a healthy run is extended, never cancelled, because a self-set budget ran out.
- **The alternatives.**
  - Cutting steps would break the width lever, which is read at equal steps and tokens or not at all (claude-302's I-t).
  - A cheaper card is not on offer at this width's memory: the 5090 has 32 GB, and micro-batch 16 peaks at 38.6 GB.
  - A faster 80 GB card (H100, B200) would change the card as well as the width, a third difference in the lever, and costs more an hour than the $2.00 cap claude-302 set. Neither is tried here.
  - A third choice is to wait for a faster trainer. That parks the one track behind work no unit has scoped.
- **The cost.**
  - Worst case: 2 pods × 21 h × $2.00 = $84.
  - Likelier, at claude-303's rates and micro-batch 16: W ≈ 68,900 s, so about 2 × 19.5 h × $1.78 ≈ $69.
  - Casey's frustration on record is about time without a trained model ("it seems like its been over 3 days and we still dont really have a trained model"), not about $15 more spend.
- **What could go wrong.**
  - A 20-hour run lasts longer, which gives the host longer to fail. The watchers renew the lease and seal on every exit.
  - With the scoring, the wall could pass 21 h. The 3,600 s scoring term is about three times claude-302's projected scoring time (940–1,230 s). That projection used the per-doubling method that underestimated training time by 1.4×; at 1.4× it is still under 3,600 s.

**Decision:** raise the bound to 72,000 s. It is logged on #2028 for Casey at launch.

## The required blocks, as amended

Each block is claude-302's, word for word, except for the changes it names.

### Base artifact

claude-302's recipe, at Change 3's pin, with `--micro-batch` set by Change 1's smokes. The control is claude-301's sealed (b64L), unchanged.

### Data split

claude-302's, unchanged.

### Steps

claude-302's six steps, with these changes:
- Step 3's smokes are Change 1's three, and I-t uses Change 2's bound.
- I-0 also runs claude-303's `a_steps_gradients_keep_only_the_variables_a_detached_input_leaves_none`.
- I-e is claude-303's, at 10⁻⁵, with its memory diagnosis recorded.
- Step 5's seal path is Change 4's.
- The watchers' stops are 21 h and $44 a pod.

### Minimum test power

claude-302's, unchanged: the width lever needs a move of more than about 10,000 rows at both seeds.

### Bar

claude-302's, unchanged.

### Stop rule

claude-302's, with Change 1's smoke rules in place of its I-m fallback, and I-t at Change 2's 72,000 s.

### Cost

- **The cap is $88 for this unit:** 2 pods × $44, the money stop; 21 h at the $2.00/h price cap is $42, so the 21 h stop acts first and the money stop is only a backstop.
- **The likely spend is about $69** (the decision review).
- **The watchers stop each pod** at 21 h or $44. A run still healthy near 21 h is extended (AGENTS.md), with the lease renewed and noted.
- **No more than 2 pods,** at $4/h in all, inside the owner's caps.

## What a pass and a fail unlock

- **Both seeds train and score:** claude-302's Readings, Bar and outcome map decide the next unit, word for word.
- **A seed is NOT_RUN by I-t at 72,000 s:** the rate is recorded. The next unit is the trainer's step time at width 1,024, profiled with `--profile`, before any longer wall.
- **A seed is NOT_RUN by I-m (no size left):** that contradicts claude-303's micro-batch 16 pass. The next unit diagnoses the difference before any rerun.
- **An out-of-memory error in the 16,384-step run** at a size the smokes chose: the seed is NOT_RUN with nothing read about width. The next unit reruns at the next smaller candidate.
- **An I-1 failure:** the job or the trainer is at fault. The next unit fixes it, and nothing is read about width.
