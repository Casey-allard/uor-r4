# ORDER: scale-probe, claude-295

Lab: claude. Item: [scale-probe](../../../../roadmap/items/scale-probe.md). Base: `e052abf41`.

## Goal

The item's done-when reads:

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

## Claim check

- **(a) The step.** P5 of Casey's 4 November plan, "the bigger train = `scale-probe`", bounded at one sheet unit and one or two run units. This is the first run unit. In the order decided on #2028 (6634218223) it is step 6, "P5 on real text". The sheet unit (claude-294) merged as `e052abf4104fc33ba28a489cf0dfdc4e94ab70b1`.
- **(b) What it changes in the model.** It trains the one model, recipe (b) trained as served, at widths 256, 512 and 1,024 on a quarter, a half and all of 28M tokens of human-written text, at two seeds. It reads whether the model gets measurably better with width and with data: the slope that says whether this architecture can keep getting closer to the goal. It also times the exported artifact at each width.

## The pin

`e052abf4104fc33ba28a489cf0dfdc4e94ab70b1`, the sheet's merge commit on `main` (the sheet's Base artifact). Every pod fetches it, checks it is that commit and clean, builds `train-in-space` and `fbs-snapshot` there with `--locked --features cuda`, and records each binary's SHA-256.

## Cancel probe (run first)

Run at 20:05Z on 5 Nov, before any pod. Class: instrument (it checks the inputs, the tools and the gates, not the mechanism).

1. **No hold.** No `local/loop/STOP`; #2028 has no `loop:pause` label; #3472's last four comments are claude's FIT, owner log, FIT correction and merge note, with no "hold" from Casey.
2. **The sheet is on `main` at the pin,** FIT recorded (#3472 `6666302038`, corrected in the next comment).
3. **The inputs are in the store** as the sheet seals them: `results/claude/scale-probe-2026-11-05/inputs/` (d1 to d3, each `tokens.u16`, `response_mask.u8` and `manifest.json`, with `SHA256SUMS`) and `census/` (seen-d1 to seen-d3). The local copies' `SHA256SUMS` hash to the sheet's `56a7db05a8f35b01e7231e90f95c2e7a095415263e2c16adcd93a20968c60b9c` and `88929e87931bef0fd91213ffbeb4701038354aa6cceca8be3c1621f478399f67`. The panels' lines in the real-text corpus's `SHA256SUMS` carry the sheet's tokens and mask SHA-256s for all four held-out splits.
4. **The caps:** `uor-pod status` shows 0 pods running and $0/h of $8/h, and 17 RTX 5090 offers from $0.563/h.
5. **The trainer takes the mixes as they are:** `--train-split DIR` reads `DIR/tokens.u16` and `DIR/response_mask.u8` and refuses either file whose SHA-256 is not the one given (`load_named_split`, `in_space_long.rs:11015`). `build_document_windows` pads every window to the context with `<|bos|>` (`hamilton_sheet.rs:1896`), so OASST2's short threads pass `load_train`'s context-long check. `config.json` records the split's path and both SHA-256s, which I-3 reads.

Result: GO.

## Decisions (each checked by the pre-run review)

- **D-1, two phases a pod.** The sheet's Step 4 grades pod A's smoke on the laptop and projects I-2 over all three pods before Step 5 trains. So each pod runs `setup` (Steps 2 to 4 on the pod: the pin, the builds, I-0, the inputs and I-3, the smokes, their scoring), exits, and waits leased. Then `main` runs with the step count and the batch per width fixed for all three pods. The pause is minutes, not hours.
- **D-2, R9 on the laptop.** `fbs-compare` is the serving build's integer program (`crates/uor-r4-integer`). It takes the snapshot and its readouts, and exports the snapshot itself in its default form (`fbs-compare.rs:357`). So the pod writes the snapshot and its plain, export and D11 readouts (`fbs-snapshot --dev DATA`, Step 6), and `fbs-compare --parity-only` runs on the laptop, as claude-240 ran it. R9 then checks the converter's default export against the D11 readout, not R4's raw export. The REPORT says so beside R9's numbers. R9 is reported only, so this moves no bar.
- **D-3, the smokes.** Every pod smokes each width it trains (I-1): pods A and B 1,024 and 256; pod C 512, 256 and 1,024. Only pod A's 256 and 1,024 smokes and pod C's 512 smoke are scored and graded, as the sheet names.
- **D-4, the seal.** Each pod's `/workspace/sp` comes back as one compressed tar with a `SHA256SUMS` inside that covers every file the run wrote. It is put under `results/claude/scale-probe-2026-11-05/runs/`, read back, and only then is the pod taken down. The laptop keeps only the rows files grading needs, because its disk has 13 GiB free.

## Files in scope

- `docs/harness/prep/scale-probe*`
- `docs/labs/scale-probe/**`
- `docs/roadmap/items/scale-probe.md`
- `docs/loop/tasks/scale-probe/**`
- `docs/model/pipeline-map.md`
- `docs/research/notebook/float-training.md`

## Done when

1. **The D67 pre-run review** passes, from a fresh non-author reviewer given the sheet, this ORDER and the job and watch scripts, and is posted on #3472 before any pod goes up. That finishes the item's done-when 1.
2. **The runs** (18 grid runs and the 2 contrast runs, or the stop rule's substitutes) are sealed in the store under `results/claude/scale-probe-2026-11-05/runs/` with a `SHA256SUMS` that was read back. Every pod is down. That is the item's done-when 2.
3. **The REPORT** gives, per width and data size:
   - training positions a second, and the exported artifact's decode tok/s against context, first (R4);
   - the panel's top-1 counts, with the sheet's paired bands across seeds (R1 to R3, R5 to R8);
   - the fitted slope per doubling, and whether it is distinguishable from flat, by the sheet's Bar;
   - I-0 to I-5's results, and R9.

   One row goes in `docs/research/notebook/float-training.md`, and the pipeline map's gap 9 records the slope. That is the item's done-when 3. The item is then set to `status: done`, and the REPORT says `Closes #3472`.

If a reading's outcome is the Bar's row 1 (undecided), the seed-3 runs it names belong to a second run unit, as the sheet's bound allows. This unit then closes done-when 1 and 2, and done-when 3 for every reading it has.

## Stop and ask if

"Ask" means decide, record it here and in the REPORT, and go on; nothing waits.
- **Fewer than three pods fit the caps:** the same cells run on two pods, one after another (the sheet's Cost).
- **I-2 projects past 20 GPU-hours:** every run's steps are cut by the sheet's rule, and the cut is recorded.
- **A smoke runs out of memory or is refused:** the sheet's stop rule (batch 8 and 16,384 steps; 768 for a refused 1,024; a refused 512 stops for repair).
- **An instrument check fails:** the outputs are sealed and the readings wait for the repair (the stop rule). The run is never graded around it.
- **A watcher's time or spend stop fires on a healthy run:** extend it and renew the lease; never cancel it.
- **Casey says "hold":** no further launches, and nothing is named or closed.
- **The laptop's free disk falls under 5 GiB:** downloads pause until my own copies already in the store are cleared.
