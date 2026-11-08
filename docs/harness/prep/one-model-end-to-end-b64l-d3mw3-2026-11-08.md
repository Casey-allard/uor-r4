# Prep sheet: (b64L) on D3 + `mw-balp/train` repeated three times, the ratio lever (one-model-end-to-end, claude-318, 8 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-318. The step is Casey's "Yes, pipeline first" (#2028 6709947526): the next training run is judged by the served-reply score.

TEST FITNESS: FIT WITH CHANGES, round 1 (a fresh non-author review, 8 Nov). It found no blockers and recomputed every figure. Its two majors and four minors are applied:
- a run that misses stopping but collapses no longer points at a higher ratio (rows 6a and 6b);
- fewer stops than × 1 at both seeds is classed as instrument or mechanism before any review, and no higher ratio is offered (row 5);
- a smoke window count other than 357,907 refuses the full run, read as row 0a;
- the collapse band no longer contradicts itself;
- the descriptive pair against (b64L-d3mw) names its confound;
- the deciding verdict is defined when the two seeds disagree (`mixed`).

**Why.**
- **(b64L) on D3 alone never stops.** claude-309 (#3828): 0 of 256 EOS stops at both seeds.
- **(b64L) on `mw-balp/train` alone stops, at a large held-out cost.** claude-313 (#3834): 253 and 244 EOS stops, and behind (b64L-D3) on the no-memory panel by −304,197 and −307,764 rows.
- **(b64L) on D3 + `mw-balp/train` once, (b64L-d3mw), stops on one seed only.** claude-315 (#3840, outcome row 5):
  - 177 and 122 of 256 EOS stops, against the bar of 128 at both seeds;
  - seed 2 collapsed: 75 of its EOS-stopped replies are one text, against the bar of 64; seed 1 had 62;
  - behind (b64L-D3) by −14,975 and −23,833 rows (band 8,858), about a sixteenth of mw-balp's cost.
- **claude-315's decision review chose the ratio lever:** mw-balp's windows repeated in the mix, with the collapse bar kept fixed before any run. This sheet repeats mw-balp three times.

## Question

At (b64L)'s recipe and pin, with TRAIN changed to D3 + `mw-balp/train` × 3:
- does the served model stop on EOS on `m2-fresh256` at **both** seeds?
- do its stopped replies stay varied, at both seeds?
- what does the extra dialogue cost on the held-out human text, against (b64L-D3) and against (b64L-d3mw)?

The held-out answer is the mix's, at a fixed step count. D3 is 42.8% of the windows seen: about 449,000 D3 windows, against 725,600 for (b64L-d3mw) and 1,048,576 for (b64L-D3). So the mix makes **2.93 passes** over D3, against 4.73 and 6.84. A cost read here mixes two causes: fewer D3 passes and mw-balp's content. This run cannot separate them, and the confound is larger than claude-315's.

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, as in claude-301, claude-306, claude-312, claude-314 and claude-317. Every pod is brought up with `--ref` at the pin, and the job refuses any other source.
- **The recipe:** claude-301's (b64L), unchanged:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2.
  - Only `--train-split`, `--train-sha256` and `--train-mask-sha256` change.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.
- **The served artifact:** `cpu-fbs-export` (`724a3b78…`) on the laptop, from each seed's sealed `models/step-016384`, with `--raw-gates --raw-transition`.
- **The scorer:** `served-reply-panel` at bin `be3d4cb0…` (#3828), unchanged.

## Data split

**TRAIN `d3mw3`,** in the store at `results/claude/one-model-end-to-end-b64l-d3mw3-2026-11-08/train/`, put by claude-318 and never overwritten. The stored copy was fetched back and its three hashes matched:
- `tokens.u16` `3cf9ebe2abdf6e6a252df545192af27aea8ec2ac419484aac8932158b5a57b67`: 50,506,504 tokens after the 64-byte header (count field `0x0302ab08`);
- `response_mask.u8` `a245fef488d4bfb0e32a447f14a6fb2543caa28a6bb8252163a8523891c22dff`;
- `manifest.json` `788b206cd57c86df71de1cc10d9f76db8b2715b0ed0757b4de846a6f78a29e3b`, schema `uor-r4-scale-probe-mix/v1`.

**How it was built:** `scale-probe mix fraction=1/1` (bin `31610a00…`) over seven roots, in this order:
1. D3's four roots, whole, at D3's pinned shas (oasst2, wiki-00000, wiki-00020, wiki-00040);
2. then `mw-balp/train`, whole, three times, under the root names `mw1`, `mw2` and `mw3`. The tool refuses a repeated root name, not a repeated folder.

Each root's two shas are checked by the tool before it reads the root. The same binary over claude-314's five roots rebuilt claude-314's `d3mw` byte for byte (`1081853b…`, `5bf91524…`) on 8 Nov.

**Checks on the laptop, 8 Nov:**
- the mix's first 35,480,910 mask bytes hash to `d3mw`'s whole mask `5bf91524…`, and its first 35,480,910 body tokens to `d3mw`'s body. So the mix begins with claude-314's mix, which begins with D3;
- each of the two later 7,512,797-byte mask segments (the first copy is inside the prefix above) hashes to `mw-balp/train`'s mask `b5e7352d…`, and each later body segment to mw-balp's body;
- the token at each of the three mw-balp starts is BOS (0). So no document runs into the next copy.

The table's per-part figures are claude-314's counts. The mix's totals follow from them because the mix is these parts, byte for byte.

| part | documents | tokens | scored targets | scored EOS | EOS per million scored | usable windows |
|---|---:|---:|---:|---:|---:|---:|
| D3 | 18,016 | 27,968,113 | 27,416,554 | 20,729 | 756 | 153,253 |
| `mw-balp/train`, each copy | 60,000 | 7,512,797 | 2,425,925 | 212,968 | 87,788 | 68,218 |
| **the mix** | **198,016** | **50,506,504** | **34,694,329** | **659,633** | **19,013** | **357,907** |
| (b64L-d3mw)'s mix, for comparison | 78,016 | 35,480,910 | 29,842,479 | 233,697 | 7,831 | 221,471 |

- The mix's documents, tokens and scored targets are also `scale-probe mix`'s own printed totals.
- The EOS counts are positions with token 1 and mask 1.
- The window counts follow the trainer's rule (`build_document_windows`: windows inside documents, split on BOS, at least 4 scored targets). claude-315's I-3 read 221,471 on its mix, the sum of D3's and mw-balp's. I-3 checks this run's 357,907.

**mw-balp's provenance** is as claude-314 recorded: the two shas, BOS 0, the store path and the tar `data/ft-balp.tar` (`b1d22113…`). Its licence and its generator are **not recorded**. This is open and is not decided here.

**The other inputs, unchanged from claude-314's sheet:**
- **DEV in-run evaluation:** claude-261's `ft/dev`, at the pins claude-306's job checks.
- **The no-memory panel:** claude-295's four held-out folders, at claude-306's per-folder pins: 1,460,697 rows, identity `5f88a5b3…`.
- **The served panel:** `m2-fresh256`, inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`, checked by the bin against its manifest.

## Repetition and exposure

The trainer samples windows uniformly by epoch (`epoch_order`). So, over 16,384 × 64 = 1,048,576 windows seen:
- **Passes:** about **2.93** over each part. mw-balp's content is three parts, so it is seen about **8.79** times. That compares with 4.73 for (b64L-d3mw) and 15.37 for (b64L-mw).
- **mw-balp's share:** 57.2% of the windows seen, 21.0% of the scored targets. In (b64L-d3mw) the figures were 30.8% and 8.1%.
- **EOS targets seen:** about **1.84** scored EOS per window seen. That compares with 0.135 for (b64L-D3), 1.06 for (b64L-d3mw), which stopped 177 and 122 times, and 3.12 for (b64L-mw), which stopped 253 and 244 times.
- **Repeated stock replies:** mw-balp's replies are seen 8.79 times each. (b64L-mw), at 15.37, collapsed at seed 1 (73). So collapse is the likelier failure here, and its bar stays fixed.

## Arms

| arm | TRAIN | batch (micro) | steps | source |
|---|---|---|---:|---|
| (b64L-D3), sealed | D3 | 64 (4 × 16) | 16,384 | claude-301 `runs-2/s{1,2}`; served scores claude-309 |
| (b64L-d3mw), sealed | D3 + mw-balp × 1 | 64 (4 × 16) | 16,384 | claude-315 `runs/s{1,2}`; served scores and grades claude-315 |
| (b64L-mw), sealed | `mw-balp/train` | 64 (4 × 16) | 16,384 | claude-313 `runs/s{1,2}`; served scores and grades claude-313 |
| **(b64L-d3mw3)** | the mix | 64 (4 × 16) | 16,384 | this sheet's run unit, seeds 1 and 2 |

One arm to train, at two seeds, one pod per seed. The steps are fixed: no gate cuts them.

## Readings

At seeds 1 and 2, from `served-reply-panel` on (b64L-d3mw3)'s exported `step-016384`, on the same measures as claude-315:
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows only, the count of the most common reply text. Also recorded: distinct reply texts over all rows, and the most common reply over all rows.
- **Reply length:** the median of `rows[].reply_ids` lengths.
- **EOS stops per stratum** (length2, length4, length8, reassert, update).

**The held-out pairs:**
- `scale-probe grade` with `train=` the mix, on all 1,460,697 rows, with no `clean=`.
- **Deciding:** `scale-probe pair`, (b64L-d3mw3) against claude-301's sealed (b64L-D3) grades `e17a13548d09304d901a93683cadb6c7695200261e86770e319003bfc976523a` and `3170c779def7ef376fdc11c96232f3f6ddb2809c436855121fd24be8ed7ab5a1`.
  - Each seed's position is `behind`, `inside` or `ahead` at the pair's band, max(⌈2·√(b+c)⌉, the two-seed spread).
  - The tool's single pooled verdict over both seeds is that position when the seeds agree, and `mixed` when they do not (`assess` in `docs/labs/scale-probe/instrument/src/lib.rs`).
  - **"Behind" in the outcome map** means the pooled verdict `behind`, or `mixed` with at least one seed `behind`. Every other verdict is "not behind". Both seeds' positions are recorded.
- **Deciding between rows 3a and 3b:** the **mean pooled d** over the two seeds of that same pair, against the cost ceiling below.
- **Descriptive:** the pair against claude-315's (b64L-d3mw) grades, `e4714275acf1fe01063b04e63aff73436614a586a6bb8e5cfa9b55bda45bad85` and `b9160b63e4f3b03892245a3b6027098c5b5eb8a9696915b4e2548fbae158d2c2`. It reads what the two added copies cost beyond the first. It carries the same confound as the deciding pair: D3's passes also fall, from 4.73 to 2.93.
- As in claude-314's sheet, `train=` builds only the T2 and unigram baselines. It does not change which rows are graded or what counts as a nearest hit; I-4 pins the row set.

Recorded beside them, descriptive only:
- **The role-header count:** rows whose reply text contains `Assistant:`, among EOS-stopped rows and over all 256 (`jq` `contains("Assistant:")` over `rows[]`, as claude-315 counted). (b64L-d3mw): 29 and 27 EOS-stopped, 35 and 51 over all. The same count for `User:` is recorded beside it.
- the pairs against claude-313's (b64L-mw) grades (`395ea543…`, `aa725ed9…`), and against the mix's own T2 and unigram;
- the DEV curve (`dev_nearest.hits` every 1,024 steps), against (b64L-D3), (b64L-d3mw) and (b64L-mw);
- the loss at the last step, within the arm only.

## Bars

Fixed before any run. They are claude-312's, unchanged, with the earlier arms' figures beside them.
- **Stopping:** at least **128 of 256** EOS stops at **both** seeds. (b64L-D3): 0 and 0; (b64L-d3mw): 177 and 122; (b64L-mw): 253 and 244.
- **Collapse:** a seed is collapsed if, among its EOS-stopped rows, one reply text covers **at least 64**. (b64L-d3mw): 62 and 75 (collapsed); (b64L-mw): 73 (collapsed) and 40.
- **The panel's bar:** at least 104 rows complete and at least 52 pairs at both seeds. Every model scored so far has 0.
- **The held-out cost:** `behind` (b64L-D3) on the pooled verdict above. One reading is stricter than claude-314's sheet, which left the case undefined: a `mixed` verdict with a seed `behind` now reads "behind" (row 3, not row 2). (b64L-d3mw) was behind by −14,975 and −23,833.

**The cost ceiling, a decision rule, not a bar.** claude-315's decision review judged that a ratio buying stopping at more than about 3× (b64L-d3mw)'s cost is a worse trade than a decode-time change. Here that is a mean pooled d against (b64L-D3) **below −58,212**: three times claude-315's mean, (−14,975 − 23,833) ÷ 2 = −19,404. It only chooses between rows 3a and 3b. It is the lab's judgement, fixed now; FIT round 1 did not change it.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | a seed NOT_RUN after its rerun, for an instrument fault (a gate, out of memory, setup, a lost pod) | **`blocked`.** Repair the instrument and rerun the seed under this sheet. No reading on one seed. |
| 0b | a seed NOT_RUN after its rerun, for a mechanism cause (a non-finite loss at a recipe that trains on both parts alone) | **`diagnostic stop`,** recorded as a result about (b64L) on the mix. Next: a decision review in the REPORT. |
| 0c | I-c or I-xc fails, or an export or score fails on the laptop with both seals present | **`blocked`, no pod.** Repair the export or scorer and re-read from the seals. |
| 1 | the panel's bar met at both seeds | **The served model answers.** Collapse and the pairs are still recorded. Next: the trunk trains on the mix under its own sheet, at the next scale. |
| 2 | stopping met, neither seed collapsed, and the pair is not `behind` | **The ratio stops, varies and keeps D3's held-out text.** Recall is the remaining gap. Next: a decision review in the REPORT on the recall lever (the panel's complete bar). |
| 3a | stopping met, neither seed collapsed, the pair is `behind`, and the mean d is at or above −58,212 | **The ratio stops and varies, at a cost inside the ceiling.** Next: a decision review on the recall lever, with this TRAIN as the base. The REPORT says the cost cannot be split between fewer D3 passes (2.93) and mw-balp's content. |
| 3b | stopping met, neither seed collapsed, the pair is `behind`, and the mean d is below −58,212 | **The ratio stops and varies, at a cost past the ceiling.** Next: a decision review between a lower ratio (× 2) and a decode-time lever. |
| 4 | stopping met, a seed collapsed | **More copies stop the model, but its stopped replies collapse.** Next: a decision review on a larger or more varied dialogue corpus. |
| 5 | stopping not met, and at least one seed below 122 EOS stops (× 1's lower seed) | **Fewer stops than × 1 at that seed, against the direction claude-315 measured.** First the REPORT classes it as instrument or mechanism (§4.2), from I-3, I-c and I-xc. Then a decision review. A higher ratio is not offered. |
| 6a | stopping not met, both seeds at 122 or more, and neither seed collapsed | **Tripling mw-balp does not lift both seeds past the bar.** The REPORT sets the stops at × 0, × 1, × 3 and mw-balp alone side by side. Next: a decision review between a higher ratio and the fine-tuning lever (deepseek-26, deepseek-28). |
| 6b | stopping not met, both seeds at 122 or more, and a seed collapsed | **More copies do not lift both seeds, and the stopped replies collapse.** Next: row 4's decision review, a larger or more varied dialogue corpus, or the fine-tuning lever. A higher ratio is not offered, since repeating mw-balp repeats its stock replies. |

A seed that is NOT_RUN reruns once, sealing to the new root `runs-2/s{1,2}/` (L-002). A second NOT_RUN reads by row 0a or 0b, by the class of its last cause.

**deepseek-28** (#3845) extends a reply-masked fine-tune of (b64L-D3): a different lever. If it lands first, the REPORT reads its stops beside these. It changes no bar and no row.

## Instrument checks

Registered before any pod goes up. They are claude-314's, with TRAIN swapped:
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin, with at least one test run, none failed, and claude-300's three micro-batch tests passing by name.
- **I-p, the pin:** the job refuses any SHA argument but the pin (exit 2), and a source that is not a clean checkout at the pin (exit 3).
- **I-3:**
  - the mix's three files match the shas above;
  - the panel folders and the DEV files match claude-306's pins;
  - each run's `config.json` names the mix's tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed and the recipe's flags, with no turn memory;
  - the trainer's "usable TRAIN windows" line in the smoke reads **357,907**. Any other count, or none, makes the arm NOT_RUN at that seed before the 16,384-step run: the job refuses it, and it reads as row 0a, an instrument fault, since it means the window rule or the mix is not what this sheet assumed.
- **I-1, the smoke:** 128 steps at `--eval-every 32` on the mix, at the pod's seed. It exits 0 with a finite loss and writes `models/step-000064`.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. claude-315's smoke on its 35.5M-token mix peaked at 26,864 and 26,800 of 32,607 MiB; claude-306's (b64L) on D4's 74.0M tokens at 26,832 and 26,928. This mix has 50.5M.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB.
- **I-t, recorded only:** r = (train_seconds at step 128 − at step 64) / 64; W = used + 1.05 × r × 16,384 + 300 s, written to `it.txt`. claude-315 read 0.559 and 0.571 s a step.
- **The watcher:** MAXSEC 14,400 s from the pod's start, a budget of PRICE × 4 h, and PRICE at most $0.70/h. Neither limit stops anything (D67): past either, the watcher logs one NOTICE and keeps watching and renewing the lease, so `seal=0` still takes the pod down.
  - **A stall pauses the lease.** When the pod's line is unchanged for 30 checks, the watcher logs a STALL NOTICE and stops renewing. The lease then lapses and the pod's reaper can take an idle pod, so a hung pod's bill is bounded. A changed line clears the notice and renewals resume (the D67 pre-run review, M1).
- **I-5:** the no-memory scoring's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes.
- **I-4:** each graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-c, the scorer controls, on the laptop before any (b64L-d3mw3) reading,** with `served-reply-panel` at `be3d4cb0…`. Each control's `rows` must equal the reference output's, field by field:
  - **Negative:** claude-308's seed-1 (b64L-D3) artifact (`9a370566…`) against claude-309's output `d02ec432…`: 0 EOS stops.
  - **Positive:** claude-313's seed-1 (b64L-mw) artifact (`1c9ae43c…`) against its output `5175c29b…`: 253 EOS stops.
- **I-x, the export:** each artifact's sha256 is recorded, and `served-reply-panel` accepts it.
- **I-xc, the export control, on the laptop before any (b64L-d3mw3) reading:** `cpu-fbs-export` (`724a3b78…`, `--raw-gates --raw-transition`) re-exports claude-313's sealed seed-1 `step-016384`. The artifact's sha256 must equal `1c9ae43c…`.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal and watch scripts.
2. On the laptop:
   1. The job is claude-314's `job-314.sh`, with TRAIN swapped to this mix:
      - `check_train` checks this mix's three shas;
      - the folder fetch is `results/claude/one-model-end-to-end-b64l-d3mw3-2026-11-08/train/`;
      - `--train-split`, the two train shas, the config check and the window count name this mix;
      - claude-317's signal trap (a TERM, INT or HUP ends the job through its sealing exit) is carried over.
   2. Its gate tests and mutations, `bash -n` on every script, I-c and I-xc.
3. `uor-pod status`, recorded. The cross-lab caps must hold with these two pods included: at most 4 pods and $8/h. Then two Vast pods:
   - one RTX 5090 each, one per seed, at most $0.70/h each;
   - leased and watched, the watchers in detached `screen` sessions (claude-315's 23:35Z gap);
   - brought up with `--count 1 --ref` at the pin;
   - avoiding Vast machines 48282 and 48276.
4. On each pod, in order:
   1. I-p, I-0, the build, the inputs and I-3.
   2. The smoke with I-m and I-f, then I-t.
   3. (b64L-d3mw3) for 16,384 steps.
   4. The no-memory scoring of `step-016384`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-b64l-d3mw3-2026-11-08/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop:
   1. I-xc and I-c again, then export each seal's `models/step-016384`, then I-x.
   2. `served-reply-panel` on both artifacts: the stops, panel measures, collapse, reply length and the role-header counts.
   3. `scale-probe grade` with `train=` the mix, then the pairs.
   4. The DEV curves.

## Minimum test power

- **Stopping** is a count. At × 1 the seeds read 177 and 122, a spread of 55. A spread of that size about 128 is expected here too.
  - That more copies lift both seeds is an **inference** from two points: 1.06 EOS per window stopped 177 and 122; 3.12 stopped 253 and 244. This run's 1.84 sits between them.
  - A seed within 32 of 128 (96 to 160) is reported as "near the bar". The row still follows the bar.
- **Collapse** is a count over EOS-stopped rows. The seed spreads so far are 33 (b64L-mw) and 13 (b64L-d3mw) about the bar of 64. A reading within 16 of 64 (48 to 80) is reported as "near the bar". The row still follows the bar.
- **The held-out pair** uses `scale-probe pair`'s band. claude-315's was 8,858, its two-seed spread, against d of −14,975 and −23,833.
  - **The scorer is not bit-reproducible across pod instances.** claude-317's I-q found 33 of 14,287 DEV nearest picks differ between two pods scoring the same checkpoint, a DEV hit difference of 3 of 3,188.
  - Each seed here is scored on its own pod, so that noise is inside the two-seed spread, and the band includes the spread. That it is small against the band is an inference from DEV's 0.1%.
- **The cost ceiling** is compared on the mean of two seeds. A mean within one band (8,858) of −58,212 is reported as "near the ceiling". The row still follows the ceiling.

## Cost

- **claude-315's measured runs:** $3.29 for both pods, on RTX 5090s at $0.563/h.
- **Here:** the step time does not depend on TRAIN, since each step reads 64 windows padded to 192. So about $3.3 for both at claude-315's rate, and about $4.1 at the $0.70/h ceiling.
- **The budget notice, not a cap:** 2 pods × 4 h × $0.70 = $5.60. A healthy run is never cancelled for a self-set budget. If a slow card bills past $5.60, the REPORT says by how much.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m, I-f or I-3's window count fails:** the arm is NOT_RUN at that seed.
- **An out-of-memory error, a non-finite loss, or a trainer exit other than 0:** NOT_RUN at that seed, classified as instrument or mechanism before it is called anything (§4.2).
- **A pod lost mid-run, or one that never reaches SSH within 1,800 s:** an instrument NOT_RUN. It is deleted, and one retry runs on another machine.
- **The no-memory scoring fails or I-5 fails:** the served readings stand. Rows 2, 3a and 3b then cannot be told apart; the REPORT reads row "2 or 3" and re-scores from the seal.
- **Always:** the job seals on every exit, including a TERM, INT or HUP, and the watcher takes the pod down after `seal=0`.

## Changes from claude-314's sheet

1. **TRAIN:** D3 + `mw-balp/train` × 3, at the store path and shas above, in place of × 1.
2. **The held-out pairs:** a descriptive pair against (b64L-d3mw), and the cost ceiling, which splits row 3 into 3a and 3b.
3. **The role-header count** is recorded, descriptive only (claude-315's decision review, point 4).
4. **I-3's window count** is 357,907, and it now gates the full run (it was recorded only). **I-m** names claude-315's smoke peaks.
5. **The outcome map:** row 3 is split by the cost ceiling; row 5 classes a drop below × 1; row 6 is split by collapse. "Behind" is defined for a `mixed` verdict.
6. **The job** carries claude-317's signal trap, and the watchers start in `screen`. After the D67 pre-run review, the chain traps signals too, the job's sealing exit ignores a second signal, and the watcher pauses renewals during a stall.
7. **Minimum test power** names claude-317's cross-pod scorer finding.
8. **The seal path** is `results/claude/one-model-end-to-end-b64l-d3mw3-2026-11-08/runs/s{1,2}/`, and `runs-2/` for a rerun.

## Pre-run review

**The D67 pre-run review**, two rounds, each a fresh-context reviewer (claude, `claude-opus-5-5`), on the scripts in `local/claude/claude-318/`.

**Round 1: NO-GO as written.** One major and three minors, all fixed:
- **M1, unbounded billing:** the watcher renewed the lease every 25 minutes whether or not the pod's line was changing. A hung pod could then never lapse its lease, so the pod reaper could never take it. Fix: no renewal while a STALL notice stands (30 unchanged checks); a changed line clears the notice. claude-317's live watcher carried the same code and was patched and restarted the same way (8 Nov, 03:05Z).
- **m1:** the chain had no signal trap, so a TERM could end it before the seal ran. Fix: `trap ':' TERM INT HUP`, a no-op trap, so the job's own trap still installs in the child.
- **m2:** `end()` reset the job's traps, so a second signal could kill the job while it sealed. Fix: `end()` begins with `trap '' TERM INT HUP`.
- **m3:** the stall notice was never cleared, so one stall would have paused renewals for good. Fix: cleared on any line change.

**Round 2: GO.** All four closed, and no new major:
- **A healthy run is not reaped.** `progress.jsonl` grows at each `--eval-every 1024` row, about every 10 minutes at the recorded 0.56 to 0.57 s a step plus DEV, about a third of STALL's 30 checks; the log byte counts grow more often. A real quiet stretch only pauses renewals: the last 4-hour renewal is at most 25 minutes old, so the reaper could take the pod only after about 4 unchanged hours.
- **The `trap ''` in `end()`** is inherited only by short children (`kill`, `wait`, `find`, `xargs`, `sha256sum`). The seal is started by the chain, not the job, so it does not inherit it.
- **The window gate** writes `ok 357907` only on an exact match and sets GO=0 before the full run otherwise.
- **One minor, applied:** m2 had no test. `gt/test-watch.sh` now runs the job's own `end()` under two TERMs and expects EXIT 0 and DONE. Deleting the ignore gives EXIT 9; `trap -` in its place kills the job before EXIT. Both mutations are caught.
- **Two nits:** the watcher's header now says renewals pause during a stall (applied). A signal in the job's first few seconds, before its trap at line 234, leaves no EXIT; the chain then ends unsealed and the watcher keeps the pod up and exits 8 for the session (kept: bounded and seen).

**The gates at the final hashes:** `gt/test-gates.sh` 44 ok; `gt/mutate.sh` 15 of 15 caught; `gt/test-seal.sh` 7 ok; `gt/test-watch.sh` 8 ok and 5 of 5 mutations caught; `bash -n` ok on every script. I-c rows equal (0 and 253 EOS stops); I-xc reproduced `1c9ae43c…`.

**The script hashes** (sha256):
- `job-318.sh` `5dd570e6a2ed0a66ac1110a696be1b72cb1b0a3c20ea20e35e5b4497a68cb867`
- `chain-318.sh` `5f591616f9a38f40b04b841e5d49dba1de188c2705f7f6253916a6b5513184c8`
- `seal-318.sh` `a49720dc772fdf22cce87c24a3de8d3cc99aeaaf716f567577d82c9cccb7fc31`
- `watch-318.sh` `067eb369e7af39514edde4d0b74d1b3b242a4bedde954cb0d47dd26442464d20`
- `read-318.sh` `d393a6f7416e40b7dee272ce9efa90cb78a3bf91a94c9e6e0ac201f32462f371`
