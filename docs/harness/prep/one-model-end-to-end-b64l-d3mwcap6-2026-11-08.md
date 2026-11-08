# Prep sheet: (b64L) on D3 + a reply-capped `mw-balp/train` repeated six times, the cap at × 1's exposure (one-model-end-to-end, claude-322, 8 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-322. The step is Track A of Casey's "Both in parallel" (#2028 6715318106), under "Yes, pipeline first" (#2028 6709947526): the run is judged by the served-reply score.

TEST FITNESS: FIT WITH CHANGES, round 1 (a fresh non-author review, 8 Nov, before any pod). No blocker; every number checked matched but one. Applied:
1. **The exposure has two measures.** Scored EOS **per window seen** match × 1's. Scored EOS **as a share of scored targets** do not: they are 1.7 times × 1's. The second is the measure the pinned loss weights. "Matched" now means the first only, and both are shown.
2. **Every change from capped × 3 pushes toward more stopping**, so only a fall (row 5) reads one way. Rows 2 and 6a now say what they cannot separate.
3. Row 5 needs both seeds below 122. A mixed reading goes to the new row 5m.
4. The census view's manifest is named.
5. The first-line count's matching rules are stated.
6. 21,100 is corrected to about 21,200.
7. The cost arithmetic is shown.

**Why.**
- **(b64L) on D3 alone never stops.** claude-309 (#3828): 0 of 256 EOS stops at both seeds.
- **mw-balp's short replies are what made (b64L) stop,** but its repeats are what the stopped replies collapsed onto:
  - D3 + mw-balp × 1 (claude-315): 177 and 122 stops, collapse 62 and 75;
  - D3 + mw-balp × 3 (claude-319): 234 and 204 stops, collapse 88 and 101.
- **Capping each repeated reply at 8 scored occurrences took stopping away.** D3 + the capped split × 3 (claude-321, #3855): **1 and 0** of 256 stops. Its replies wrote `\n\nAssistant:` instead of EOS in 116 and 171 rows.
- **But that run changed two things at once.** It saw about 0.82 scored EOS per window, against 1.06 for × 1. A fall from 177 / 122 to 1 / 0 is larger than that exposure change alone suggests, but the two could not be separated.
- **claude-321's decision review chose the × 6 capped mix,** already counted: **1.07 scored EOS per window seen**, × 1's density per window, with the cap kept. As a share of scored targets, the measure the loss weights, the EOS is 1.33%, against 0.78% for × 1 (see "Repetition and exposure").

## Question

At (b64L)'s recipe and pin, with TRAIN changed to D3 + the capped `mw-balp/train` × 6:
- does the served model stop on EOS on `m2-fresh256` at **both** seeds?
- do its stopped replies stay varied, at both seeds?
- what does it cost on the held-out human text, against (b64L-D3)?
- does the first line still repeat a prompt value, and does it pick the asked-for one more often (descriptive)?

**What the comparison can and cannot say.**
- **Against claude-315's × 1:** the scored EOS per window seen is matched (1.07 against 1.06). The EOS share of scored targets is not: 1.33% against 0.78%. The stock replies are capped, and D3's passes fall from 4.73 to 2.51.
- **Against claude-321's capped × 3:** the cap is the same and the file is a superset (three more copies). Four things change together, and each pushes toward more stopping:
  - the EOS per window rises from 0.82 to 1.07;
  - the EOS share rises from 0.77% to 1.33%;
  - D3's passes fall from 3.67 to 2.51;
  - each capped reply now appears 6 times, not 3. The census puts 424,056 of the 444,761 scored EOS (95.3%) in repeated replies.
- **So a rise in stopping cannot be given to any one of them.** A fall would be against all four, and reads one way (row 5).

Neither comparison isolates one change. The REPORT says so beside every reading.

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, as in claude-301, claude-306, claude-312, claude-314, claude-317, claude-318 and claude-321. Every pod is brought up with `--ref` at the pin, and the job refuses any other source.
- **The recipe:** claude-301's (b64L), unchanged:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2.
  - Only `--train-split`, `--train-sha256` and `--train-mask-sha256` change.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.
- **The served artifact:** `cpu-fbs-export` (`724a3b78…`) on the laptop, from each seed's sealed `models/step-016384`, with `--raw-gates --raw-transition`.
- **The scorer:** `served-reply-panel` at bin `be3d4cb0…` (#3828), unchanged.

## Data split

**TRAIN `d3mwcap6`,** in the store at `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/train/`, put by claude-322 and never overwritten. The stored copy was fetched back and its three hashes matched (`shasum -c`, 8 Nov):
- `tokens.u16` `0273173e16e805706a9b3e816c3de46b824a53daf91be3b812be360f8e8e75a7`: 73,044,895 tokens after the 64-byte header;
- `response_mask.u8` `da506e850145f55b746929e108e0fd240ddf753f6a90b23976c98e159a1cb293`;
- `manifest.json` `442ead2c92ae9a2e6b0b500ebbc12eb6e472f91c96465b6c1347fa80ac232ae9`, schema `uor-r4-scale-probe-mix/v1`.

**How it was built:** `scale-probe mix fraction=1/1` (bin `31610a00…`) over ten roots, in this order:
1. D3's four roots, whole, fetched from the store (`results/claude/real-text-corpus-2026-11-05/{oasst2,wiki-00000,wiki-00020,wiki-00040}/train/`), each checked by the tool against its own two shas;
2. then claude-321's k = 8 capped split (`3dee858f…` / `445e8744…`, [`reply-cap`](../../labs/one-model-end-to-end/reply-cap-2026-11-08/README.md)), whole, six times, under the root names `cap1` to `cap6`.

The build log is `k-table/d3mwcap6-mix.log`. **All three shas equal the ones claude-321's sheet recorded for its counted × 6 mix** before that mix was deleted. So the build is reproducible byte for byte, manifest included, and claude-321's census of it applies.

- **A first attempt was refused, and is recorded:** `scale-probe mix` refuses claude-314's `d3-rebuild` (D3 as one mix) as a root, because a mix's manifest has no `source`. A root must carry its corpus's provenance, so D3 is read as its four roots.

**The mix's census** (`reply-census`, `k-table/d3mwcap6.census.json`, from claude-321's count of the same bytes). That census was taken on claude-321's split `census-view6`. Its manifest, `9458e854…`, is not this mix's `442ead2c…`. Its tokens and mask equal this mix's two shas, so its counts apply.

| part | documents | tokens | scored targets | scored EOS | top reply | usable windows |
|---|---:|---:|---:|---:|---:|---:|
| D3 | 18,016 | 27,968,113 | 27,416,554 | 20,729 | n/a | 153,253 |
| capped `mw-balp/train`, each copy | 60,000 | 7,512,797 | 991,367 | 70,672 | 8 | 44,028 |
| **the mix** | **378,016** | **73,044,895** | **33,364,756** | **444,761** | **50** | **417,421** |
| claude-321's capped × 3 | 198,016 | 50,506,504 | 30,390,655 | 232,745 | 26 | 285,337 |
| claude-315's × 1 | 78,016 | 35,480,910 | 29,842,479 | 233,697 | n/a | 221,471 |

- The documents, tokens and scored targets are also `scale-probe mix`'s own printed totals. The scored targets are 27,416,554 + 6 × 991,367, and the EOS 20,729 + 6 × 70,672.
- The census reads 82,329 distinct replies. The top reply, 50, is 6 copies × 8 plus D3's own occurrences.
- **The two exposure measures, per row of the table:**

  | part | scored EOS per usable window | EOS share of scored targets | scored targets per window |
  |---|---:|---:|---:|
  | the mix | 1.07 | 1.333% | 79.9 |
  | claude-321's capped × 3 | 0.82 | 0.766% | 106.5 |
  | claude-315's × 1 | 1.06 | 0.783% | 134.7 |
- **The window count is per part, then summed.** The trainer cuts each BOS-delimited document on its own, so a mix's count is the sum of its parts'. D3's 153,253 is 357,907 − 3 × 68,218, the trainer's own logged counts on `d3mw3` and on `mw-balp/train`. The capped copy's 44,028 is `window-count`'s (claude-321, checked against those same two logs), and claude-321's capped × 3 run logged 153,253 + 3 × 44,028 = 285,337 as predicted. Here, 153,253 + 6 × 44,028 = **417,421**. It gates the run (I-3).

**mw-balp's provenance** is as claude-314 recorded: the two shas, BOS 0, the store path and the tar `data/ft-balp.tar` (`b1d22113…`). Its licence and its generator are **not recorded**. This is open and is not decided here.

**The other inputs, unchanged from claude-321's sheet:**
- **DEV in-run evaluation:** claude-261's `ft/dev`, at the pins claude-306's job checks.
- **The no-memory panel:** claude-295's four held-out folders, at claude-306's per-folder pins: 1,460,697 rows, identity `5f88a5b3…`.
- **The served panel:** `m2-fresh256`, inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`, checked by the bin against its manifest.

## Repetition and exposure

The trainer samples windows uniformly by epoch (`epoch_order`). So, over 16,384 × 64 = 1,048,576 windows seen:
- **D3's passes:** about **2.51**, against 3.67 for claude-321's capped × 3, 4.73 for (b64L-d3mw) and 6.84 for (b64L-D3). D3 is 36.7% of the windows (53.7% in capped × 3). **This moves the held-out reading the wrong way:** fewer D3 passes should cost more held-out text, whatever the cap does.
- **The capped copies' share:** 63.3% of the windows seen, but only 17.8% of the scored targets.
- **EOS targets seen:** about **1.07** scored EOS per window seen, about 1.12 million over the run. This is an upper bound, as in claude-321's sheet: the EOS in chunks the trainer drops (under 4 scored targets) are never seen, and the overstatement is under 1%. That compares with:
  - 0.82 for claude-321's capped × 3, which stopped 1 and 0;
  - 1.06 for (b64L-d3mw) (× 1), which stopped 177 and 122;
  - 1.84 for claude-319's uncapped × 3, which stopped 234 and 204.
- **What the loss weights:** the pinned trainer's objective (`voronoi_loss` in `crates/uor-r4-float-train/src/in_space_long.rs` at the pin) is the token hinge's `mean_all` plus the address hinge's `mean_all`, each over the batch's scored rows. So EOS carry a weight in each step's loss equal to their share of the batch's scored targets. That share is **1.333%** here, against **0.783%** for × 1 and **0.766%** for claude-321's capped × 3: 1.7 times × 1's. The capped copies' replies are short, so a window holds 79.9 scored targets, against 134.7 in × 1.
  - **On that measure, claude-321's capped × 3 already matched × 1** (0.77% against 0.78%) and still stopped 1 and 0. So "the exposure" claude-321 could not separate from the cap was the per-window count, not the loss's share.
- **Repeated stock replies:** a kept reply occurrence is scored about 15.1 times over the run (6 copies × 2.51). The mix's top reply, 50 occurrences, is scored about 126 times. In × 1, mw-balp's top reply (4,470 occurrences) was scored about 21,200 times (4,470 × 4.735 passes).

**So this run reads one way only.** Every change from capped × 3 pushes toward more stopping (above). So a rise cannot be given to the per-window exposure, the EOS share, D3's lower passes or the 6× repetition. A fall below × 1's lower seed at both seeds (row 5) is against all four. It says that the long-tail repetition the cap removes (mw-balp's top reply scored about 21,200 times in × 1, against 126 here) carried × 1's stopping. This is recorded before any run.

## Arms

| arm | TRAIN | batch (micro) | steps | source |
|---|---|---|---:|---|
| (b64L-D3), sealed | D3 | 64 (4 × 16) | 16,384 | claude-301 `runs-2/s{1,2}`; served scores claude-309 |
| (b64L-d3mw), sealed | D3 + mw-balp × 1 | 64 (4 × 16) | 16,384 | claude-315 `runs/s{1,2}`; served scores and grades claude-315 |
| (b64L-d3mw3), sealed | D3 + mw-balp × 3 | 64 (4 × 16) | 16,384 | claude-319 `runs/s{1,2}`; served scores and grades in `local/claude/claude-318/` |
| (b64L-d3mwcap), sealed | D3 + capped mw-balp × 3 | 64 (4 × 16) | 16,384 | claude-321 `runs/s{1,2}`; served scores and grades claude-321 |
| (b64L-mw), sealed | `mw-balp/train` | 64 (4 × 16) | 16,384 | claude-313 `runs/s{1,2}`; served scores and grades claude-313 |
| **(b64L-d3mwcap6)** | the mix | 64 (4 × 16) | 16,384 | this sheet's run unit, seeds 1 and 2 |

One arm to train, at two seeds, one pod per seed. The steps are fixed: no gate cuts them.

**The two references, on the same measures:**

| arm | seed | EOS per window / EOS share | EOS stops | collapse (EOS-stopped) | complete | pairs | held-out d against (b64L-D3) | first line: asked-for / other |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| (b64L-d3mw), × 1 | 1 | 1.06 / 0.783% | 177 | 62 | 0 | 0 | −14,975 (behind) | 2 / 0 |
| (b64L-d3mw), × 1 | 2 | 1.06 / 0.783% | 122 | 75 | 0 | 0 | −23,833 (behind) | not read |
| (b64L-d3mwcap), capped × 3 | 1 | 0.82 / 0.766% | 1 | 1 | 0 | 0 | −18,236 (inside) | 25 / 21 |
| (b64L-d3mwcap), capped × 3 | 2 | 0.82 / 0.766% | 0 | 0 | 0 | 0 | −72,308 (behind) | 76 / 36 |
| (b64L-d3mwcap6), this run | 1, 2 | 1.07 / 1.333% | | | | | | |

## Readings

At seeds 1 and 2, from `served-reply-panel` on (b64L-d3mwcap6)'s exported `step-016384`, on the same measures as claude-321's sheet:
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows only, the count of the most common reply text, beside the number of EOS-stopped rows it is over. Also recorded: distinct reply texts over all rows, and the most common reply over all rows.
- **Reply length:** the median of `rows[].reply_ids` lengths.
- **EOS stops per stratum** (length2, length4, length8, reassert, update).

**The held-out pairs:**
- `scale-probe grade` with `train=` the mix, on all 1,460,697 rows, with no `clean=`.
- **Deciding:** `scale-probe pair`, (b64L-d3mwcap6) against claude-301's sealed (b64L-D3) grades `e17a13548d09304d901a93683cadb6c7695200261e86770e319003bfc976523a` and `3170c779def7ef376fdc11c96232f3f6ddb2809c436855121fd24be8ed7ab5a1`.
  - Each seed's position is `behind`, `inside` or `ahead` at the pair's band, max(⌈2·√(b+c)⌉, the two-seed spread).
  - The tool's single pooled verdict over both seeds is that position when the seeds agree, and `mixed` when they do not (`assess` in `docs/labs/scale-probe/instrument/src/lib.rs`).
  - **"Behind" in the outcome map** means the pooled verdict `behind`, or `mixed` with at least one seed `behind`. Every other verdict is "not behind". Both seeds' positions are recorded.
- **Deciding between rows 3a and 3b:** the **mean pooled d** over the two seeds of that same pair, against the cost ceiling below.
- **Descriptive,** each against pinned grades that `read-322.sh` checks before it pairs:
  - claude-321's capped × 3 grades, `aeceb4dfe4d699ddfaf10a599df3be659bbf97fe032e81ed31d0532886435e68` and `b1691a98766083516adbda06f25ff6398a3c698a702c21b58abb90269562752b`: the same cap, three more copies;
  - claude-315's (b64L-d3mw) grades, `e4714275acf1fe01063b04e63aff73436614a586a6bb8e5cfa9b55bda45bad85` and `b9160b63e4f3b03892245a3b6027098c5b5eb8a9696915b4e2548fbae158d2c2`: the same EOS per window seen;
  - claude-319's uncapped × 3 grades, `583bfc8734d5761f40a7305816c2031fc3f3faf7242fa30231284bfb2ad40d2e` and `d1b7d0e271cc267790a4ce04106e3bfafce5915f7e2e07b2343ce34f0be56c1c`;
  - claude-313's (b64L-mw) grades (`395ea543…`, `aa725ed9…`), and the mix's own T2 and unigram.
- As in claude-321's sheet, `train=` builds only the T2 and unigram baselines. It does not change which rows are graded or what counts as a nearest hit; I-4 pins the row set.

Recorded beside them, descriptive only:
- **The first-line count** (claude-321's REPORT review), by `firstline.jq` (`b15c778d…`, pinned by `read-322.sh`). For each row, the reply text up to its first newline, lower-cased, is checked for the row's own accepted answers (`labels.json`, a trailing full stop dropped), and separately for those of the other case with the same `pair_id`, which is the prompt's other value. It prints the rows that hold only the asked-for value, only the other value, and both, and the row count (256).
  - **The match is a substring match** (jq `contains`) of the first line against each accepted answer, so a short answer inside a longer word counts.
  - **A reply that starts with `\n` has an empty first line** and holds neither value.
  - On claude-321's scores it reproduces 25 / 21 / 0 and 76 / 36 / 0; on claude-315's seed 1, 2 / 0 / 0; on claude-318's seed 1, 0 / 0 / 0.
- **The role-header count:** rows whose reply text contains `Assistant:`, and `User:`, among EOS-stopped rows and over all 256.
- the DEV curve (`dev_nearest.hits` every 1,024 steps), against (b64L-D3), (b64L-mw), (b64L-d3mw) and (b64L-d3mw3), with (b64L-d3mwcap)'s seed-1 end (6,133) from claude-321's REPORT, since its seals are in the store only;
- the loss at the last step, within the arm only.

## Bars

Fixed before any run. They are claude-312's, unchanged, with the earlier arms' figures beside them.
- **Stopping:** at least **128 of 256** EOS stops at **both** seeds. (b64L-D3): 0 and 0; (b64L-d3mw): 177 and 122; (b64L-d3mw3): 234 and 204; (b64L-d3mwcap): 1 and 0; (b64L-mw): 253 and 244.
- **Collapse:** a seed is collapsed if, among its EOS-stopped rows, one reply text covers **at least 64**. (b64L-d3mw): 62 and 75 (collapsed); (b64L-d3mw3): 88 and 101 (both collapsed); (b64L-mw): 73 (collapsed) and 40; (b64L-d3mwcap): not readable at 1 and 0 stopped rows.
- **The panel's bar:** at least 104 rows complete and at least 52 pairs at both seeds. Every model scored so far has 0.
- **The held-out cost:** `behind` (b64L-D3) on the pooled verdict above. (b64L-d3mw) was behind by −14,975 and −23,833; (b64L-d3mw3) by −155,433 and −110,880; (b64L-d3mwcap) mixed, −18,236 (inside) and −72,308 (behind).

**The cost ceiling, a decision rule, not a bar,** unchanged from claude-318's sheet: a mean pooled d against (b64L-D3) **below −58,212**, three times claude-315's mean (−19,404). It only chooses between rows 3a and 3b.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | a seed NOT_RUN after its rerun, for an instrument fault (a gate, out of memory, setup, a lost pod) | **`blocked`.** Repair the instrument and rerun the seed under this sheet. No reading on one seed. |
| 0b | a seed NOT_RUN after its rerun, for a mechanism cause (a non-finite loss at a recipe that trains on both parts alone) | **`diagnostic stop`,** recorded as a result about (b64L) on the mix. Next: a decision review in the REPORT. |
| 0c | I-c or I-xc fails, or an export or score fails on the laptop with both seals present | **`blocked`, no pod.** Repair the export or scorer and re-read from the seals. |
| 1 | the panel's bar met at both seeds | **The served model answers.** Collapse and the pairs are still recorded. Next: the trunk trains on the mix under its own sheet, at the next scale. |
| 2 | stopping met, neither seed collapsed, and the pair is not `behind` | **This TRAIN stops, varies and keeps D3's held-out text.** Consistent with the exposure, but not separable from D3's lower passes, the higher EOS share or the 6× repetition (above). Recall is the remaining gap. Next: a decision review in the REPORT on the recall lever, with the first-line count as its starting reading. |
| 3a | stopping met, neither seed collapsed, the pair is `behind`, and the mean d is at or above −58,212 | **This TRAIN stops and varies, at a cost inside the ceiling.** Next: a decision review on the recall lever, with this TRAIN as the base. The REPORT says the cost cannot be split between D3's passes (2.51) and mw-balp's content. |
| 3b | stopping met, neither seed collapsed, the pair is `behind`, and the mean d is below −58,212 | **This TRAIN stops and varies, at a cost past the ceiling.** D3's passes are the lowest yet (2.51). Next: a decision review between restoring D3's passes (more steps at this mix, under its own sheet and cost) and a decode-time lever. |
| 4 | stopping met, a seed collapsed | **At × 1's exposure, capping at k = 8 does not stop the collapse.** The REPORT looks the collapsed text up in `k-table/k8.census.json`'s top list and in D3, so whether it is one of mw-balp's repeated replies rests on that lookup. Next: a decision review between k = 1 at a matched exposure (counted under its own sheet) and a decode-time lever. |
| 5 | stopping not met, and **both** seeds below 122 EOS stops ((b64L-d3mw)'s lower seed) | **At × 1's EOS per window, and a higher EOS share, the capped mix still stops on fewer rows than × 1.** This is the clean direction: every change from capped × 3 pushed toward more stopping. So the long-tail repetition of the stock replies, which the cap removes, carried the stopping, not the EOS density. First the REPORT classes it as instrument or mechanism (§4.2), from I-3, I-c and I-xc. Next: a decision review in the REPORT, led by the decode-time read (claude-321's Still open: these seals and claude-321's re-scored with an end-of-turn rule at `\n\nAssistant:`, on the laptop). |
| 5m | stopping not met, one seed below 122 and the other at 122 or more, and neither seed collapsed | **Mixed: one seed at or above × 1's lower seed (122), one below it.** No direction is read. The REPORT gives both seeds' stops beside × 1's and capped × 3's. Next: row 6a's decision review, with this reading named as mixed. |
| 6a | stopping not met, both seeds at 122 or more, and neither seed collapsed | **The capped mix stops about as often as × 1, without collapsing.** Consistent with the exposure, but not separable from D3's lower passes, the higher EOS share or the 6× repetition. The REPORT gives the collapse counts beside the number of EOS-stopped rows they are over, and the stops at × 1, capped × 3 and capped × 6 side by side. Next: a decision review between more capped copies (× 8 or more, counted under its own sheet first) and a decode-time lever. |
| 6b | stopping not met, at least one seed at 122 or more, and a seed collapsed | **The cap at × 1's exposure neither lifts both seeds nor stops the collapse.** Next: row 4's decision review. |

A seed that is NOT_RUN reruns once, sealing to the new root `runs-2/s{1,2}/` (L-002). A second NOT_RUN reads by row 0a or 0b, by the class of its last cause.

## Instrument checks

Registered before any pod goes up. They are claude-321's, with TRAIN swapped:
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin, with at least one test run, none failed, and claude-300's three micro-batch tests passing by name.
- **I-p, the pin:** the job refuses any SHA argument but the pin (exit 2), and a source that is not a clean checkout at the pin (exit 3).
- **I-3:**
  - the mix's three files match the shas above. The likeliest wrong file is claude-321's capped × 3 mix, the store's other capped TRAIN: the gate tests feed it its tokens (claude-318's `d3mw3` file, byte for byte), its mask `423d4f77…` and its manifest `8f73bac1…`, and expect a refusal for each. They also feed the uncapped × 3 mask and manifest, and claude-314's × 1 files;
  - the panel folders and the DEV files match claude-306's pins;
  - each run's `config.json` names the mix's tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed and the recipe's flags, with no turn memory;
  - the trainer's "usable TRAIN windows" line in the smoke reads **417,421**. Any other count, or none, makes the arm NOT_RUN at that seed before the 16,384-step run: the job refuses it, and it reads as row 0a. The gate tests include claude-321's 285,337 as a refusal.
- **I-1, the smoke:** 128 steps at `--eval-every 32` on the mix, at the pod's seed. It exits 0 with a finite loss and writes `models/step-000064`.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. The mix is 22.5M tokens larger than claude-321's (about 45 MB of tokens and 23 MB of mask), so claude-321's smoke peaks are the reference, with that much added.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB.
- **I-t, recorded only:** r = (train_seconds at step 128 − at step 64) / 64; W = used + 1.05 × r × 16,384 + 300 s, written to `it.txt`.
- **The watcher:** MAXSEC 14,400 s from the pod's start, a budget of PRICE × 4 h, and PRICE at most $0.70/h. Neither limit stops anything (D67): past either, the watcher logs one NOTICE and keeps watching and renewing the lease, so `seal=0` still takes the pod down.
  - **A stall pauses the lease.** When the pod's line is unchanged for 30 checks, the watcher logs a STALL NOTICE and stops renewing, so a hung pod's bill is bounded. A changed line clears the notice and renewals resume.
- **I-5:** the no-memory scoring's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes.
- **I-4:** each graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-c, the scorer controls, on the laptop before any (b64L-d3mwcap6) reading,** with `served-reply-panel` at `be3d4cb0…`. Each control's `rows` must equal the reference output's, field by field:
  - **Negative:** claude-308's seed-1 (b64L-D3) artifact (`9a370566…`) against claude-309's output `d02ec432…`: 0 EOS stops.
  - **Positive:** claude-313's seed-1 (b64L-mw) artifact (`1c9ae43c…`) against its output `5175c29b…`: 253 EOS stops.
- **I-x, the export:** each artifact's sha256 is recorded, and `served-reply-panel` accepts it.
- **I-xc, the export control, on the laptop before any (b64L-d3mwcap6) reading:** `cpu-fbs-export` (`724a3b78…`, `--raw-gates --raw-transition`) re-exports claude-313's sealed seed-1 `step-016384`. The artifact's sha256 must equal `1c9ae43c…`.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal, watch, seed and read scripts.
2. On the laptop:
   1. The job is claude-321's `job-321.sh`, with TRAIN swapped to this mix:
      - `check_train` checks this mix's three shas;
      - the folder fetch is `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/train/`;
      - the window gate reads 417,421;
      - every other line, `end()` and the traps included, is claude-321's, as reviewed there.
   2. Its gate tests and mutations, `bash -n` on every script, I-c and I-xc.
   3. The Vast adapter `uor-r4-pod-provider`, built from fresh `main` into `local/pod-provider-target` (`uor-pod status` cannot list Vast pods without it).
3. `uor-pod status`, recorded. The cross-lab caps must hold with these two pods included: at most 4 pods and $8/h. Then, per seed, `seed-322.sh SEED`:
   - one RTX 5090 Vast pod, at most $0.70/h, brought up with `--count 1 --ref` at the pin and leased under session claude-322;
   - its watcher started in a detached `screen` before any script ships;
   - the job, chain and seal scripts shipped, their hashes checked on the pod against this sheet's, then the chain started;
   - avoiding Vast machines 48282 and 48276.
4. On each pod, in order:
   1. I-p, I-0, the build, the inputs and I-3.
   2. The smoke with I-m and I-f, then I-t.
   3. (b64L-d3mwcap6) for 16,384 steps.
   4. The no-memory scoring of `step-016384`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop, `read-322.sh`:
   1. `ctrl`: I-xc and I-c again; then `serve`: export each seal's `models/step-016384`, I-x, the panel, the role headers and the first-line count.
   2. `grade`: `scale-probe grade` with `train=` the mix, then the pairs.
   3. `dev`: the DEV curves.

## Minimum test power

- **Stopping** is a count. At × 1 the seeds read 177 and 122, a spread of 55; at capped × 3, 1 and 0.
  - The EOS seen per window, 1.07, matches × 1's; the EOS share of scored targets is 1.7 times × 1's. That stopping should then near × 1's is an **inference**, not a measured rule: the cap also changes which replies carry the EOS, and D3's passes fall.
  - A seed within 32 of 128 (96 to 160) is reported as "near the bar". The row still follows the bar.
- **Collapse** is a count over EOS-stopped rows. The seed spreads so far are 33 (b64L-mw), 13 (b64L-d3mw) and 13 (b64L-d3mw3) about the bar of 64. A reading within 16 of 64 (48 to 80) is reported as "near the bar". The row still follows the bar.
  - A model that stops rarely has few EOS-stopped rows, and a collapse count over few rows is small by construction. So the REPORT gives the collapse count beside the number of EOS-stopped rows it is over.
- **The held-out pair** uses `scale-probe pair`'s band. claude-315's was 8,858, its two-seed spread.
  - **The scorer is not bit-reproducible across pod instances.** claude-317's I-q found 33 of 14,287 DEV nearest picks differ between two pods scoring the same checkpoint, a DEV hit difference of 3 of 3,188. Each seed here is scored on its own pod, so that noise is inside the two-seed spread.
- **The cost ceiling** is compared on the mean of two seeds. A mean within one band of −58,212 is reported as "near the ceiling". The row still follows the ceiling.
- **The first-line count** is descriptive. At capped × 3 the asked-for and other values were 25 / 21 and 76 / 36: a seed-to-seed spread far larger than the gap at seed 1. No row reads it.

## Cost

- **claude-321's measured runs:** $3.01 for both pods (8,864 s and 8,838 s).
- **Here:** the step time does not depend on TRAIN, since each step reads 64 windows padded to 192. The fetch and load are about 68 MB larger. So about $3 for both at claude-321's rates. At the $0.70/h ceiling, the same seconds cost 8,864 + 8,838 = 17,702 s = 4.92 h × $0.70 = **$3.44**, plus a few minutes for the larger fetch.
- **The budget notice, not a cap:** 2 pods × 4 h × $0.70 = $5.60. A healthy run is never cancelled for a self-set budget. If a slow card bills past $5.60, the REPORT says by how much. The ORDER's bound is $8.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m, I-f or I-3's window count fails:** the arm is NOT_RUN at that seed.
- **An out-of-memory error, a non-finite loss, or a trainer exit other than 0:** NOT_RUN at that seed, classified as instrument or mechanism before it is called anything (§4.2).
- **A pod lost mid-run, or one that never reaches SSH within 1,800 s:** an instrument NOT_RUN. It is deleted, and one retry runs on another machine.
- **The no-memory scoring fails or I-5 fails:** the served readings stand. Rows 2, 3a and 3b then cannot be told apart; the REPORT reads row "2 or 3" and re-scores from the seal.
- **Always:** the job seals on every exit, including a TERM, INT or HUP, and the watcher takes the pod down after `seal=0`.

## Changes from claude-321's sheet

1. **TRAIN:** D3 + the k = 8 capped `mw-balp/train` × 6, at the store path and shas above, in place of capped × 3. The tokens are a new file.
2. **The build** reads D3's four roots from the store and reproduces claude-321's counted × 6 shas.
3. **The window gate** is 417,421, the sum of the measured per-part counts.
4. **The outcome map** is rewritten for the exposure question: rows 2, 5 and 6a read the cap against the exposure, and row 3b names D3's passes.
5. **The descriptive pairs** add claude-321's capped × 3. **The first-line count** is a scripted reading (`firstline.jq`), checked on four earlier scores.
6. **I-3's gate tests** take claude-321's capped × 3 mask and manifest, and its window count, as refusals.
7. **The pods** are brought up by `seed-322.sh`, one per seed, under lease session claude-322, after the Vast adapter is built.
8. **The seal path** is `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/runs/s{1,2}/`, and `runs-2/` for a rerun.
9. **The exposure** is given on two measures, per window seen and as a share of scored targets, and the outcome map reads only a fall one way (the FIT).
10. **The watcher's screen** is named `c322-wS<SEED>` (the pre-run review).

## Pre-run review

**GO**, 8 Nov: the D67 pre-run review, one round, a fresh-context reviewer, on the scripts in `local/claude/claude-322/` against claude-321's.
- **Every hunk in the six scripts and four gate tests is an intended change.** `end()`, the traps, the watcher's stall, renew and down logic, and the seed script's $0.70/h refusal are byte-identical to claude-321's.
- **The seal and TRAIN checks hold.** The seal root is new, and claude-321's sealed root is untouched (L-002). The TRAIN check refuses claude-321's capped × 3 files, the uncapped × 3 mask and manifest, and claude-314's × 1 mix.
- **No MAJOR finding.** MINOR 1 is applied: `seed-322.sh` named the watcher's screen `c321-wS`, now `c322-wS`, with its new hash below. MINOR 2, `firstline.jq`'s extra `rows` field, is now stated under Readings. MINOR 3 needs no action: the ×1 and uncapped wrong-file cases are still asserted directly in `test-gates.sh`.

**The gates at the current hashes, 8 Nov:**
- `gt/test-gates.sh` 50 ok;
- `gt/mutate.sh` 18 of 18 caught, including claude-321's capped × 3 tokens, mask, manifest and window count;
- `gt/test-seal.sh` 7 ok;
- `gt/test-watch.sh` 5 of 5 mutations caught;
- `bash -n` ok on every script.

**The script hashes** (sha256):
- `job-322.sh` `70ebc96570dae9d0a4de1e33a860ca295e15d3cb5ccc56537fb32790d506434b`
- `chain-322.sh` `441a531c7ecff3a6aa9071865f1d59f6c2602d7768a033e1f4e2faa84ffbba5c`
- `seal-322.sh` `dca62ba2b097ae5ffcf516c6e0130b95b4bc0e77a135bafa38ea799bf2d9ec79`
- `watch-322.sh` `fb9fda42e19316274f243acb3c9b8a787515c99e85c94a4879459e354e7dc891`
- `seed-322.sh` `8bd593a58856165ce1007eafb7d73533a8cd8d65a63648856147655f97ba0106` (after the screen rename; the reviewed copy was `aacc57cc…`, and the rename is its only change)
- `read-322.sh` `5ddbb0d84bf6db65cded422fd7abd76d8bad8f18176b6fcab1c451c3f8356e06`
- `firstline.jq` `b15c778db57c2e65902e34987cf15748213128ab5d9bb72230f97f0ea58fb702`
