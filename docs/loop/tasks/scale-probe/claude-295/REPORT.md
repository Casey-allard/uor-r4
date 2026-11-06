# REPORT: scale-probe, claude-295 (the first run unit)

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [scale-probe](../../../../roadmap/items/scale-probe.md) (#3472). Pin: `e052abf4104fc33ba28a489cf0dfdc4e94ab70b1`. PR #3750.

Widths 256 and 512 are trained, sealed and graded; width 1,024 did not run (the decision card below). Every number below names its source file or comment.

**Claim check.** This unit is P5 of Casey's 4 November plan, "the bigger train = `scale-probe`", bounded at one sheet unit and one or two run units. It is the first run unit. It trains the one model, recipe (b) trained as served, at widths 256, 512 and 1,024. The data is a quarter, a half and all of 28M tokens of human-written text, at two seeds. It reads whether the model gets measurably better with width and with data.

## What did not work, first

- **On human-written text, the training loss does not fall.** This is the unit's main result. It is not yet classified as instrument or mechanism (§4.2).
  - Width 256, all 6 grid cells: `train_loss_mean` is 1.00381–1.00390 at step 1,024 and 1.00296–1.00362 at step 8,192 (`progress.jsonl` in each cell's run folder, sealed in `runs/A`).
  - Width 512, all 6 grid cells: 1.00322–1.00331 at step 1,024, rising to 1.0037–1.0041 at step 4,096, and 1.00343–1.00381 at step 8,192 (`progress.jsonl` in each cell's run folder, sealed in `runs/C`). So every 512 cell ends higher than it starts. D1 at 512 makes 3.41 passes over its TRAIN.
  - Width 512 is not measurably better on the panel at these two seeds: R6 (512 against 256 at D3) reads `inside`, which is not distinguishable at this power, not "no difference" (L-007). Every 512 cell is behind its TRAIN's unigram (Width 512, below).
  - The same recipe on templated text (the contrast cell, 256, mw-balp TRAIN) falls from 1.00280 to 0.82460 over the same 8,192 steps.
  - On the held-out panel every width-256 cell is `behind` its own TRAIN's unigram table, by 12,460–25,917 rows (`pair/w256-dX-vs-unigram.json`). So always naming TRAIN's commonest token would beat it: the trunk predicts tokens in roughly their mix (below), not the commonest one.
  - The templated cell, which never saw human text, scores 14,282 on the same panel, inside the real-text cells' range of 11,504–24,961.
  - **What the trunk predicts:** the mix of human text's tokens, spread across contexts, not the likeliest token for each context. These counts were computed on the laptop from the sealed no-memory rows.
    - On wiki-00000 (396,045 rows), w256-d3-s1 predicts 2,823 distinct tokens; the targets hold 3,081.
    - Most of its commonest predictions are the targets' commonest tokens at about their own rates: token 223 is predicted 10,585 times and is the target 8,979 times; token 265, 9,397 and 9,191; token 85, 8,052 and 8,197.
    - Two are off their rates. Token 201, TRAIN's commonest, is predicted 10,189 times and is the target 6,851 times, about 49% over. Token 14, the panel's commonest target (10,845 times), is predicted 7,761 times.
    - It is right on 5,335 rows. A guess blind to context, drawn with the same token mix, would be right on 2,193 (`Σ` over tokens of the target count × the predicted count / rows). So the context lifts it 2.4×. Always naming TRAIN's commonest token (201, the unigram's guess) is right on 6,851, and T2 on 71,600.
    - The same holds for w256-d1-s1 (4,620 against 2,275) and w256-d3-s2 (6,618 against 2,262), and on oasst2 (1,699–2,608 against 1,089–1,122).
    - **This is a lead for the classification, not a classification.** A nearest-token hinge against the best rival works where the next token is nearly fixed by the context, as in the templated text. Where many tokens can follow, it may match token frequencies rather than pick the likeliest one. It has not been tested.
  - It is classified next, instrument before mechanism (§4.2). Casey's one-track direction of 6 November (#2028 6675163438) gives that to one-model-end-to-end's next unit (claude-296): a fixed-target test that splits instrument from mechanism. This replaces the plan the decision card below gave scale-probe's second run unit.
  - Nothing is closed: no configuration and no mechanism (§4.1).
- **R1, R2 and R3, the deciding readings, are NOT RUN.** All three read width 1,024 at D3, and width 1,024 did not run in this unit. That is not a negative.
- **Width 1,024 does not fit a 32 GB RTX 5090** (attempt 1, #3472).
  - It ran out of memory at batch 16 (peak 31,952 MiB, the whole card) and at batch 8 (29,456 MiB). The sheet's stop rule covers only a width that fails at batch 16, so the job stopped at EXIT 5, as written.
  - Classification: instrument (the GPU's memory). No run trained.
  - The repair kept the recipe and moved width 1,024 to a 96 GB RTX PRO 6000 (my decision review, logged on #3472).
- **The 96 GB card never came up on Vast.**
  - Pod D failed to start three times. All three hosts belonged to one operator, Vast host 120840 in Texas.
  - The cause was measured on the third start: the host was RUNNING with its SSH port open, and its SSH server refused our key (`Permission denied (publickey)`). My first guess, an image still pulling, was wrong; the correction is on #3472.
  - About $3 went on the three starts.
  - The sheet's stop rule replaces a failed pod "once". Pod D's second replacement was a departure, logged before it ran (#3472 6669956916).
- **The lab has no route of its own to a 96 GB card.**
  - Under `uor-pod`'s own filters (a verified datacenter host, reliability above 0.98), the only RTX PRO 6000 Server host is 120840.
  - One RTX PRO 6000 Workstation host qualifies, but `uor-pod`'s GPU table has no row for that edition, and `scripts/pod/uor-pod` is an owner path.
  - Runpod by hand needs `--owner-approved`.
  - Both were asked of Casey on #2028 (6670812323 and its addendum 6672201438). The decision card below withdraws that ask until the second run unit needs it.
- **R9 does not compare the form this model serves.**
  - `fbs-compare --parity-only` always exports the snapped trunk. These models train as served, so their served artifact is the raw-gate export.
  - At (256, D3, seed 1), compare exits 0 on the snapped artifact `cab98c6b…`:
    - single tokens: 130 of 4,096 agree, and 2,752 positions are over the gap bar;
    - sequences: 181 of 5,795 agree, with 4,868 positions over the bar;
    - the largest query gap is 15.9998 (`r9/w256-d3-s1/compare.json`, store `results/claude/scale-probe-2026-11-05/r9/w256-d3-s1`).
  - At (512, D3, seed 1) it exits 0 on the snapped artifact `b22468ba…`:
    - single tokens: 152 of 4,096 agree, with 2,962 positions over the bar;
    - sequences: 160 of 5,795 agree, with 5,786 over;
    - the largest query gap is 15.9998 (`r9/w512-d3-s1/compare.json`, store `results/claude/scale-probe-2026-11-05/r9/w512-d3-s1`).
  - That measures snapping, not the served form. So R9 is NOT_RUN in the served form at widths 256 and 512, as the stop rule says, failing at the step "no parity tool reads the raw-gate export". This is an instrument item and does not move the Bar.
  - A raw-export mode for `fbs-compare` is tooling for its own unit, not this one.
- **`uor-pod`'s exact-driver option can never match on Vast.**
  - Vast's API returns nothing for a `driver_version=` predicate, though the same offers report that driver when unfiltered.
  - `uor-pod` also filters by driver locally and rechecks at create, so only the server-side copy of the filter is at fault.
  - It is a tool item for its own unit (FRICTION).
- **My D30 slip.** Three read-only offer searches used the Python `vastai` CLI on PATH, not `uor-pod`'s Rust adapter. Nothing was created or changed. Every later query used the adapter.
- **Width 512's whole-process scoring ran out of memory on the 5090.**
  - The scorer's GPU memory grows with each split it scores in one process: w512-d1-s1 went 12.3 → 16.4 → 27.2 → 30.0 GB and failed on its sixth split (22:58Z).
  - w512-d1-s1, w512-d2-s1 and w512-d2-s2 failed this way. w512-d1-s2, w512-d3-s1 and w512-d3-s2 scored whole.
  - Classification: instrument (scoring memory), not the trained model. The checkpoints are untouched.
  - The repair, `rescore-295.sh`, scores the same checkpoint with the same binary, flags and seed, one panel per process. It passed two fresh review rounds.
  - All three rescored with EXIT 0, at 14.8–18.8 GB peak and about 7 minutes a cell. Each process's DEV rows agreed, and I-5 reads `same` on every panel (`sp2/rescore/NAME/`, sealed in `runs/C`).
- **I-2 rests on a projected 1,024 term.** The sheet projects I-2 from each smoke, and the 1,024 smoke has not run. At the measured 5090 ratios (×1.97 per width doubling in step time, ×1.87 in scoring), the 20 runs come to 19.83 GPU-h, under 20, so there is no cut.
- **The laptop's disk fell twice.**
  - At 23:21Z on 5 November it was 2.6 GiB free. The largest folder is another lab's build folder (17 GiB), noted for that lab on #2028 (6672142338).
  - The ORDER's rule paused downloads under 5 GiB until store-sealed local copies were cleared. It is 9.2 GiB free now.

## The decision on width 1,024

My decision card is on #3472 (6674681832), after two fresh decision reviews, logged for Casey to reverse. The 96 GB ask on #2028 is withdrawn for now (6674683811).

**Superseded in part.** After this card, Casey's one-track direction of 6 November (#2028 6675163438) made "the trunk learns human text" the only track. The classification moves to one-model-end-to-end (claude-296), as a fixed-target test, and width 1,024's cells wait until the trunk beats its TRAIN's unigram and T2 on the human panel. The card's plan, as decided, is kept below as the record.

- **The decision.** Width 1,024's 7 cells move to the second run unit, which first classifies the real-text training loss.
- **Why defer.** Read now, R1 and R2 would rest on an unclassified instrument: its real-text loss does not fall, and its cells are behind the unigram (§4.2).
  - At 256, D3 against D1 reads `ahead` on all rows and `mixed` on the unseen rows. So Bar row 5 is possible at 1,024, but its meaning would depend on that unclassified loss.
- **What the card planned for the second unit, in order** (claude-296 now sets its own arms): at most 6 GPU runs at (256, D1, seed 1) on one 5090.
  1. The loader and mask, on the laptop with no GPU: what the trainer reads from D1's TRAIN against mw-balp's.
  2. The learning rate: at most two other rates, plus one longer-positions arm.
  3. The objective's margin and rivals: at most two settings.
- **What the outcome does to width 1,024:**
  - a loader or mask defect taints every reading here, and the grid reruns with 1,024;
  - an arm that makes the loss fall reruns the grid with that recipe and 1,024;
  - if every arm stays flat, it is a mechanism result for 256 and 512 only, and 1,024 stays NOT RUN.
- **The ORDER's last paragraph**, which covers row 1's seed-3 runs, is extended to these cells. The item stays `ready`, and this REPORT says `References #3472`, because the deciding slopes need width 1,024.

## Departures from the sheet's pod plan

- The sheet puts seed 1 on pod A, seed 2 on pod B and width 512 on pod C. Since attempt 1, each width has its own GPU:
  - width 256 is on pod A (a 5090: 6 grid cells and the contrast cell);
  - width 512 is on pod C (a 5090: 6 grid cells);
  - width 1,024 moves to the second run unit.
- The cells are unchanged, and each is covered once.

## What changed

Docs only, 4 files, about +330 lines (`loop pr-check` size: tests 0, code 0, docs 4 files, generated 0). No code changed: the instrument and the trainer are the pin's, `e052abf41`.

- **The ORDER** and this REPORT, under `docs/loop/tasks/scale-probe/claude-295/`.
- **One notebook row** in `docs/research/notebook/float-training.md`.
- **Gap 9 in `docs/model/pipeline-map.md`** now says the slope is unread (width 1,024 did not run) and that the trunk's loss does not yet fall on human text, with the 256 and 512 numbers.
- **The item stays `ready`.** This REPORT says `References #3472`, because the deciding slopes need width 1,024.
- **Off the repository:**
  - the sealed runs, `results/claude/scale-probe-2026-11-05/runs/A` and `runs/C`, and R9's `r9/w256-d3-s1` and `r9/w512-d3-s1`, all in the store;
  - the job, watch, rescore, seal, pick and grading scripts in `local/claude/claude-295/`, uncommitted.

## Measurements

### Instruments

- **I-0:** 377 passed, 0 failed, on pods A and B in attempt 1 (#3472). Pods A and C passed setup in attempt 2; their counts are in the sealed setup logs (`sp2/setup`).
- **I-1 and I-4, the smokes,** graded on the laptop:
  - width 256 (pod A): exit 0; 1,460,697 of 1,460,697 rows;
  - width 512 (pod C): exit 0; 1,460,697 of 1,460,697 rows;
  - both have the same pooled row identity, `5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb`.
  - Every graded width-256 checkpoint has that identity too.
- **I-2's inputs** (the 5090): 0.169 s a step at 256 and 0.333 at 512. Scoring takes 179 s and 335 s.
- **I-5:** all 7 width-256 checkpoints report `same` on all 5 checks (`A-main/w256-*-scored.i5`).
- **The scoring repair's equivalence** (on pod A, 01:16Z, 6 November):
  - The checkpoint was w256-d1-s1's step 8,192, which had scored whole.
  - Scored again one panel per process, its 10 row sets matched: 4 panels and DEV, each with memory and without. The row identities and the files were the same byte for byte.
  - The peak GPU memory was 8,176 MiB (`sp2/rescore/w256-d1-s1/equiv.txt`, sealed in `runs/A`).
  - So the per-panel rescore stands in for whole-process scoring.

### Width 256 (pod A, sealed and read back to `runs/A`; pod A down at 01:29Z)

Training took 1,474–1,508 s per grid cell and 1,015 s for the contrast cell. Scoring took 175–180 s.

**The panel's whole-table nearest top-1** (1,460,697 rows; `grade/w256-*/grade.json`):

| cell | seed 1 | seed 2 | its TRAIN's T2 | unigram |
|---|---|---|---|---|
| D1 (a quarter) | 18,402 | 23,139 | 253,018 | 37,421 |
| D2 (a half) | 20,247 | 11,504 | 255,504 | 37,421 |
| D3 (all) | 20,061 | 24,961 | 256,660 | 37,421 |

**The paired readings at 256** (`scale-probe pair`, `pair/*.json`):

| pair | all rows | rows unseen in the cleaner mix |
|---|---|---|
| each cell vs its T2 | behind (d −229,879 to −244,000) | behind |
| each cell vs the unigram | behind (d −12,460 to −25,917) | behind |
| D3 vs D1 | **ahead**: d +1,659 / +1,822, band 371 / 395 | mixed |
| D3 vs D2 | inside: band 13,643 | inside |
| D2 vs D1 | inside: band 13,480 | inside |

- **The seen census** (step 8, `seen/`): unseen rows 1,326,589 against D1, 1,279,552 against D2 and 1,228,450 against D3, with 4,935 short rows throughout. These are exactly the sheet's census counts.
- **R5's data slope at 256** is read from its end pair, D3 against D1: ahead on all rows, so it is distinguishable from flat at this width. It is mixed on the unseen rows. Per doubling of data that is half the end pair's `d`: about +830 / +911 rows at seeds 1 / 2. It is a gain of about 1,700 of 1.46M rows while the model is behind the unigram; it is not a measure of learning human text.
- **R7, by source,** (256, D3, seed 1). Each source is listed as model / T2 / unigram:
  - oasst2 (184,542 rows): 1,699 / 30,868 / 4,601;
  - wiki-00000 (396,045 rows): 5,335 / 71,600 / 6,851;
  - wiki-00020 (443,252 rows): 6,525 / 77,379 / 14,538;
  - wiki-00040 (436,858 rows): 6,502 / 76,813 / 11,431.

  The model is under the unigram on every source.
- **R8, the contrast cell** (256, mw-balp TRAIN, seed 1):
  - On its own templated DEV (14,287 rows) it is right on 7,676, against its T2's 6,302 and its unigram's 1,226. Against T2 that is d +1,374 over a floor of 117: it learns its own text.
  - The score rows' DEV count, 7,676, equals the in-run `dev_nearest` count.
  - On the human panel it gets 14,282, against mw's T2 28,263 and unigram 1,132 (`grade/w256-mw-s1{,-dev}`).
- **R4, tok/s** (laptop, Apple M1, on battery, `r4/w256-d3-s1.toks.json`):
  - The export of (256, D3, seed 1) is `w256-d3-s1.uorlut`: 2,156,416 bytes, sha256 `30326134…`.
  - At its context of 192 (a prefill of 128, then 64 decode steps): prefill 1,370 tok/s and decode 1,336 tok/s with the read; 1,428 and 1,438 without it.
  - The prefills of 512, 2,048, 8,192 and 32,704 are skipped: the artifact's 192-token context cannot hold them.
  - These were measured on battery power, so they are a floor for this machine, not its plugged-in rate.

### Width 512 (pod C)

Pod C's main ended at 03:25:36Z with EXIT 6, as expected with three cells unscored. Its finish rescored them (all EXIT 0), ran R9, and sealed `runs/C` at 03:51:38Z, read back from the store and checked. Pod C was down at 03:56:33Z.

Training took 2,875–2,932 s per cell. Whole-process scoring took 335–337 s, and the per-panel rescore about 7 minutes a cell.

- **What was graded.** The laptop graded each cell's sealed no-memory rows:
  - All 100 picked files match the seal's `SHA256SUMS.files`, and each rescored cell's 28 files match its own.
  - Every grade's recorded `rows_file_sha256` matches a sealed file: 12 of 12 whole-scored, and 4 of 4 for each rescored cell.
  - Every checkpoint scores the panel's 1,460,697 rows (row identity `5f88a5b3…`, I-4), and I-5 reads `same` on every panel of every cell.

**The panel's whole-table nearest top-1** (`grade/w512-*/grade.json`):

| cell | seed 1 | seed 2 | at width 256 (seeds 1 / 2) | its TRAIN's T2 | unigram |
|---|---|---|---|---|---|
| D1 (a quarter) | 18,295 | 15,284 | 18,402 / 23,139 | 253,018 | 37,421 |
| D2 (a half) | 13,058 | 13,078 | 20,247 / 11,504 | 255,504 | 37,421 |
| D3 (all) | 20,227 | 15,553 | 20,061 / 24,961 | 256,660 | 37,421 |

**The paired readings at 512** (`pair/w512-*.json`):

| pair | all rows | rows unseen in the cleaner mix |
|---|---|---|
| each cell vs its T2 | behind (d −234,723 to −242,446) | behind |
| each cell vs the unigram | behind (d −17,194 to −24,363) | behind |
| D3 vs D1 | mixed: d +1,932 / +269, band 1,663 | inside |
| D3 vs D2 | mixed: d +7,169 / +2,475, band 4,694 | mixed |
| D2 vs D1 | mixed: d −5,237 / −2,206, band 3,031 | mixed |

- **R5's data slope at 512** is read from its end pair, D3 against D1: mixed, so it is not distinguishable from flat at this width and these seeds. Per doubling it is about +966 / +135 rows at seeds 1 / 2.
- **R6, width 512 against width 256 at D3** (`pair/w512-d3-vs-w256-d3.json`, clean = D3's seen set): **inside** on all rows (d +166 / −9,408, band 9,574) and on unseen rows (d +261 / −6,942, band 7,203). Doubling the width is not distinguishable from no change at these seeds.
- **R6's curvature** is named at width 1,024 (across widths at D3, and across data at 1,024), so it is NOT RUN. For information only, the data curvature `D3 − 2·D2 + D1` from the top-1 counts is −2,031 / +25,092 at 256 and +12,406 / +4,681 at 512 (seeds 1 / 2). At 256 the seeds disagree in sign.
- **R7, by source,** (512, D3, seed 1), each as model / T2 / unigram:
  - oasst2: 2,116 / 30,868 / 4,601;
  - wiki-00000: 5,083 / 71,600 / 6,851;
  - wiki-00020: 6,495 / 77,379 / 14,538;
  - wiki-00040: 6,533 / 76,813 / 11,431.

  The model is under the unigram on every source, as at 256 (1,699 / 5,335 / 6,525 / 6,502).
- **R4, tok/s at 512** (laptop, Apple M1, on AC power this time, `r4/w512-d3-s1.toks.json`):
  - The export of (512, D3, seed 1) is `w512-d3-s1.uorlut`: 4,302,016 bytes, sha256 `6e83e676…`. It was made by the same command as 256's, whose export this command reproduces byte for byte (`30326134…`).
  - At its context of 192: prefill 691 tok/s and decode 673 tok/s with the read; 745 and 750 without it.
  - Width 256, timed again in the same AC session (`r4/w256-d3-s1.ac.toks.json`): prefill 1,344 and decode 1,329 with the read; 1,419 and 1,417 without. So width 512 serves at about half of 256's rate, from twice the bytes. The 256 battery figures above are within 2% of these.
  - The prefills of 512 and longer are skipped, as at 256.
- **R9 at 512:** the snapped form, NOT_RUN in the served form (the negatives above).

## Checks run at the head

- **The head** is the final commit of this PR: the width-512 sections, the notebook row, gap 9's 512 lines, and the two exact-head review rounds' fixes. No code changed after the pin.
- **The laptop gates,** run at the head before the review record:
  - `loop report --check --order docs/loop/tasks/scale-probe/claude-295/ORDER.md`: PASS;
  - `loop pr-check --local`: PASS (order, scope, banned-presence, commit-order, report, owner-paths, rule-sources and rendered-lock). Its two D76 advisories are the ORDER's size (7,857 B, over its 4,096 B budget) and `docs/start/FILEMAP.md`'s stale rows, which this PR does not touch.
- **The pods:** A down at 01:29Z, D's three starts deleted, and C down at 03:56:33Z, each after its seal was read back from the store. `uor-pod status` then showed 0 running pods.
- **GPU spend:** about $6.93 on pods A and C (the watcher's $6.65 at 03:26Z, plus pod C's last 30 minutes at $0.563/h), and about $3 on pod D's three starts.
- **Reviews, each in a fresh context:**
  - the decision review of the width-1,024 card, two rounds, posted at #3472 6674681832;
  - the exact-head review and a claims check of this REPORT, the notebook row and gap 9, recorded by `loop review` at the final head.

## Skills used

None applied.

## Still open

- Width 1,024 and R1–R3: scale-probe's second run unit. Under Casey's one-track direction of 6 November (#2028 6675163438), it waits until the trunk beats its TRAIN's unigram and T2 on the human panel. The classification of the flat loss is one-model-end-to-end's next unit (claude-296).

## Plain summary

This unit trained the project's one model on real, human-written text for the first time: Wikipedia and OASST2 replies, 28M tokens. It ran two sizes (256 and 512 numbers per position), three amounts of data and two seeds each, on two rented GPUs, and graded every model on 1.46M held-out positions.

The model did not learn the text. Its training loss stayed flat at about 1.0035 for all 8,192 steps, while the same recipe on the project's templated text falls to 0.82. On held-out text, every model guesses the next token right less often than simply always naming TRAIN's single commonest token, and far less often than a table of "which token usually follows this one". The bigger model is not measurably better than the smaller one at these two seeds, and serves at half the speed.

What the model predicts is telling: roughly the right mix of tokens, scattered across contexts, rather than the likeliest token for each context. One hypothesis, not yet tested, is the training objective. It pushes each guess away from whichever token is nearest whenever that token is not the right answer. In ambiguous text that happens on most rows, even for the best guess.

The planned question, how much better the model gets per doubling in size, needs the largest size, 1,024, which did not fit the 32 GB card. A 96 GB card never started. That size waits. By Casey's direction, the next unit (one-model-end-to-end) trains on fixed targets to tell a measuring fault from a method fault, and every later unit works on getting the model to learn human text. GPU time cost about $10.

References #3472.
