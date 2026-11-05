# ORDER: scale-probe, claude-294

Lab: claude. Item: [scale-probe](../../../../roadmap/items/scale-probe.md) (#3472). Base: `9577aebc9`.

## Goal

The item's done-when reads:

> 1. **A prep sheet** `docs/harness/prep/scale-probe-<date>.md` with a fresh FIT and the D67 pre-run review, both posted on this item's issue. It registers:
>    - the arms, the cost and the stop rule;
>    - the panel, drawn fresh by `heldout-panel` at a new seed with its own census. P-dev stays reserved for `one-model-end-to-end`'s run;
>    - the bars, in advance.
> 2. **The runs,** sealed in the store with a SHA256SUMS that was read back. The pods are down.
> 3. **The REPORT** gives, per width and corpus size:
>    - training tokens per second, and served decode tokens per second against context on the exported artifact;
>    - the panel's reply scores, with paired bands across seeds;
>    - the fitted slope, and whether it is distinguishable from flat.
>
>    It also has one notebook row in `float-training.md`, and the pipeline map's gap 9 records the slope.

**This unit is done-when 1, the sheet.** The runs (done-when 2) and the REPORT (done-when 3) are the next units, under P5's bound of 1–2 run units.

- **Claim check, (a) the step:** Casey's plan of 4 November, P5, "The bigger train = `scale-probe`. [sheet 1 unit; run 1–2 units]", in the order decided on #2028 ([6634218223](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6634218223)): step 6, "P5 on real text", after the real-text corpus (done, claude-292 and claude-293) and P4 (done, claude-286 to claude-288).
- **Claim check, (b) the model:** no code changes. The sheet fixes what the next unit trains: the one model's recipe at widths 256, 512 and 1,024, on human-written text in place of the templated M-world, and the bars that say whether it gets better with width and data. That slope decides whether the trunk is scaled up next, which is the plan's question about reaching a GPT-4-class chatbot.

## Decision card (the Data line)

The decided order says: "P5's Data line changes to real text through a decision card in its sheet unit. The templated data stays as one contrast cell." This unit makes that card, on its own fresh decision review, logged on #3472 as `[claude] owner log:` and open to Casey's reversal:
- **Data:** the real-text corpus of claude-293 (OASST2 and Wikipedia, about 28M TRAIN tokens), at several fractions, in place of "the corpus at several fractions". This is under the order's interim §3.10 reading, human-written text only; if Casey answers otherwise, that is a new configuration.
- **Panel:** the corpus's own held-out splits, cut by source id, in place of a panel drawn by `heldout-panel`, which draws only templated M-world conversations. The order implies a panel taken from the corpus ("the corpus panel scores…").
  - No model has read them. Only claude-293's census, and its memory probe, have read them; the probe read an earlier build of `wiki-00000/heldout`.
  - **The overlap is counted.** `scale-probe seen` counts, against each cell's TRAIN (D1, D2, D3), the held-out scored positions whose preceding 8 tokens appear in that TRAIN.
  - **The deciding readings R1 to R3** are read on all rows and on the rows unseen in D3, the largest TRAIN. A deciding reading that holds on one row set and not the other is read as mixed.
  - P-dev stays reserved.
  - **The contrast cell,** reported only, is scored on DEV and on the same four panels. It is reported beside claude-152's and claude-153's DEV points, which used other recipes and steps. (Revised in this unit before the card was logged; the first draft drew a new templated panel for it.)
- **Scores:** an exact count of whole-table nearest top-1 hits by the float model trained as served, against unigram and T2 tables built from each cell's own TRAIN, with bytes per token under the pinned tokenizer.
  - The decided order says "integer top-1 against unigram and T2 count tables". This card departs from those words: the deciding count is the float model's whole-table nearest top-1, not §4.8's integer top-1 through the D11 export. The slope is a float-model measurement; R9's parity checks the export, and the D11 export's panel top-1 is owed before any width is served or any claim is made.
  - Bits per byte waits for a lawful likelihood (§4.8).
- **Reversal:** restore the item's Data line and done-when 1's panel text, taking `docs/roadmap/items/scale-probe.md` from this unit's merge-base with `origin/main`. The sheet's "hold" stops any launch. The mixes, the census, and any checkpoints and rows already sealed stay in the store. A run already made stays as its own configuration (§4.1), and the old data becomes a new configuration.

## Cancel probe (run first)

The unit is cancelled if any of these holds:
1. A `scale-probe` sheet or labs folder already exists on `main`.
2. Another open order or PR holds a file in this scope.
3. The real-text corpus is not sealed in the store with a `SHA256SUMS` read back.
4. The kill switch is on.
5. `depends_on: [one-model-end-to-end]` means the recipe is still undecided.

**Result: PASS, go** (run at `823f012a4`, 5 Nov ~18:10Z).
1. No `docs/harness/prep/scale-probe*` and no `docs/labs/scale-probe/` exist on `main` (`9577aebc9`).
2. Of the open PRs, only this unit's #3744 touches `pipeline-map.md`, `float-training.md` or a `scale-probe` path; `loop claim` found no overlapping order.
3. `results/claude/real-text-corpus-2026-11-05/` is in the store (`uor-store ls`). Its 61-line `SHA256SUMS` was read back in claude-293, and the local copy in `local/claude/rtc/` checks 61 of 61 OK.
4. `loop next`: "kill-switch: off".
5. **Classed instrument** (a dependency label, not a result). `one-model-end-to-end` is `ready`, not `done`, because its 21-run training and verdict have not run. But the recipe the line trains and serves today is fixed:
   - claude-266's (b), which P2b trained (claude-275) and claude-281 classified;
   - served as claude-243's a7r and P4's carry exports (claude-288).

   Casey's plan puts P5 after P4 and waits on no verdict. So this probe measures recipe (b) without its turn memory (the sheet's Arms say why). A recipe that item picks later is a new configuration of this probe, not a fault in it.

## Files in scope

- `docs/harness/prep/scale-probe*`
- `docs/labs/scale-probe/**`
- `docs/roadmap/items/scale-probe.md`
- `docs/loop/tasks/scale-probe/**`
- `docs/model/pipeline-map.md`
- `docs/research/notebook/float-training.md`

## Done when

1. **The instrument** `docs/labs/scale-probe/instrument/` exists: a standalone crate (its own `[workspace]`, like `docs/labs/one-model-end-to-end/instrument`), bin `scale-probe`, laptop counting only. It has three commands:
   - `mix` writes the nested TRAIN fractions;
   - `grade` scores one checkpoint's `rows-*.json` against unigram and T2 tables built from that cell's own TRAIN;
   - `pair` gives b, c, d, the band and the verdict for two cells at two seeds.

   Its tests pass, with a negative gate for each refusal.
2. **The mixes** D1, D2 and D3 (¼, ½ and all of claude-293's four TRAIN roots) are built on the laptop. They are stored under `results/claude/scale-probe-2026-11-05/inputs/` with a `SHA256SUMS`, read back.
3. **The sheet** `docs/harness/prep/scale-probe-2026-11-05.md`:
   - it is in the project's shape (`uor-harness check` passes `prep.sheet_shape`);
   - it pins the mixes' and the panels' SHA-256s;
   - a fresh non-author FIT reads `TEST FITNESS: FIT`, posted on #3472 and recorded in the sheet.
4. **The decision card** is logged on #3472 as `[claude] owner log:`, on its own fresh decision review, and the item file's Data and panel lines say real text.
5. **A REPORT,** negatives first, and the in-unit exact-head review.

**The D67 pre-run review is the run unit's,** right before launch. It checks the lease and the caps as they stand then, as P2b's sheet did (claude-276). Done-when 1 is met there.

## Stop and ask if

Stop and decide, never stop and wait:
- **The trainer cannot take a cell as registered** (width 1,024, the corpus fractions, or the host memory the turn memory needs at 28M tokens): narrow the cell, write why in the sheet, and keep the three widths if the writer's cap allows them.
- **The projected cost passes the caps** (4 pods, $8/h): cut positions equally across cells, never drop a seed.
- **The FIT or the pre-run review finds a blocker:** fix the sheet and review the delta, at most two rounds.
