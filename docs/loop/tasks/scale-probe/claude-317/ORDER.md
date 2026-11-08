# ORDER: scale-probe, claude-317

Lab: claude. Item: [scale-probe](../../../../roadmap/items/scale-probe.md) (#3472). Base: `8903c5551`.

## Goal

**Register and run the width slope on the recipe that learns human text.** Track B of Casey's 8 Nov decision, "Both in parallel" (#2028 6715318106).

Why now:
- claude-295 ran the scale probe on the old recipe. Its training loss did not fall on human text at either width, so that run had no slope to read.
- claude-296 classed that result as the objective (mechanism).
- claude-301's (b64L) is now ahead of D3's T2 at both seeds, by +75,038 and +85,590 held-out rows against a band of 10,552.
- So (b64L) is the first recipe on which a width slope means anything.

The run:
- (b64L) at widths 128, 256 and 512, on TRAIN D3, with every other flag fixed.
- **Heads scale with width:** `--hamming-read` is width / 64. A Hamming head holds at most 64 coordinates (`flock_read_hamming.rs` ~996), so width 512 cannot keep 4 heads. Width 256 at 4 heads is claude-301's sealed (b64L) itself, re-paired with I-r and not retrained.
- Graded on claude-295's four human held-out panels, against each other and against D3's T2 and unigram.

The item's done-when, which this ORDER covers for widths 128–512 at one corpus size:

> 1. **A prep sheet** `docs/harness/prep/scale-probe-<date>.md` with a fresh FIT and the D67 pre-run review, both posted on this item's issue. It registers:
>    - the arms, the cost and the stop rule;
>    - the panel: the real-text corpus's own held-out splits, cut by source id, with a census of their 8-token overlap with each TRAIN mix (claude-294's decision card). P-dev stays reserved for `one-model-end-to-end`'s run;
>    - the bars, in advance.
> 2. **The runs,** sealed in the store with a SHA256SUMS that was read back. The pods are down.
> 3. **The REPORT** gives, per width and corpus size:
>    - training tokens per second, and served decode tokens per second against context on the exported artifact;
>    - the panel's reply scores, with paired bands across seeds;
>    - the fitted slope, and whether it is distinguishable from flat.
>
>    It also has one notebook row in `float-training.md`, and the pipeline map's gap 9 records the slope.

**Claim check:**
- (a) The step is Casey's Track B, the P5 scale slope.
- (b) What it changes is the answer to "does this architecture get better with size", read on the served recipe. It does not change the served model.

## Cancel probe (run first)

Run 8 Nov, about 01:10Z, at `origin/main` `3d3bbb6a1`. The outcome: **no cancel.**

1. **No open PR trains a width sweep.** The only open training PR is claude-315's #3840, which keeps width 256 and changes the data. Classed: no overlap.
2. **claude-301's width-256 control is sealed and readable.**
   - The store lists `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s1/` (SHA256SUMS, list and tarball), and the laptop holds claude-301's `grade/b64L-s{1,2}`.
   - Classed: instrument present.
3. **The geometry accepts the arms.**
   - `HammingRead::new` (`crates/uor-r4-float-train/src/flock_read_hamming.rs` ~989) requires at least 2 heads, heads that divide the width, and a head width that is a multiple of 4 and at most 64.
   - Width 128 at 2 heads, 256 at 4 and 512 at 8 each give a head width of 64.
   - Every width-checking site (`flock_read.rs`, `flock_read_opening.rs`, `fbs_export.rs`, `geometric_stack_native.rs`, `native_vector_writer.rs`) needs only a multiple of 4 or 8.
   - The pod's smoke is the live check.
4. **Width 1,024 is out of this unit.**
   - On the old recipe it did not fit a 32 GB card (claude-295).
   - `uor-pod`'s 80 GB cards (A100, H100, H200) are owner-forbidden, and no 96 GB route passes its filters.
   - So 1,024 needs a 48 GB card (L40S or RTX 6000 Ada, explicit tier) and its own unit, and only if this slope rises.

## Files in scope

- `docs/harness/prep/scale-probe*`
- `docs/labs/scale-probe/**`
- `docs/roadmap/items/scale-probe.md`
- `docs/loop/tasks/scale-probe/**`
- `docs/model/pipeline-map.md`
- `docs/research/notebook/float-training.md`

Run scripts live in `local/claude/claude-317/`, uncommitted, as for claude-301 and claude-315. Their hashes are recorded in the sheet.

## Done when

1. **The sheet** `docs/harness/prep/scale-probe-b64l-width-2026-11-08.md`, with a fresh non-author FIT and the D67 pre-run review, both posted on #3472. It registers:
   - the arms, the gates, the cost, the stop rule;
   - the bars and the outcome map, all before any pod goes up.
2. **The runs:** widths 128 and 512 at seeds 1 and 2, plus the (w128h4) contrast at seed 1, sealed in the store with a SHA256SUMS that was read back. The pods are down.
3. **The REPORT, per width:**
   - training tokens per second;
   - served decode tokens per second, from `served-reply-panel`'s new decode fields on the exported artifact;
   - held-out top-1 rows;
   - the paired verdicts 512 against 256 and 256 against 128;
   - margins over T2 and the unigram;
   - the slope reading.
4. **The records:** one notebook row in `float-training.md`, and the pipeline map's gap 9 records the slope.

## Stop and ask if

Nothing waits on Casey (§5.1). Stop and decide (never stop and wait) if:
- **Width 512 does not fit at any registered micro-batch:** NOT_RUN at that seed, instrument, and the 48 GB card route becomes the next unit.
- **Spend would pass $30,** the ORDER's notice line. It is not a cap: the charter's extend rule applies to a healthy run.
- **A gate fails:** the sheet's stop rule applies, and the result is classified instrument versus mechanism before it is named.
- **Bound:** this unit, plus one repair unit if a gate fails as an instrument.
