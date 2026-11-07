# Prep sheet: (b64L) on a D3 + `mw-balp/train` mix, stopping without the held-out cost (one-model-end-to-end, claude-314, 7 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-314. The step is Casey's "Yes, pipeline first" (#2028 6709947526): the next training run is judged by the served-reply score.

TEST FITNESS: FIT WITH CHANGES, round 1 (a fresh non-author review, 7 Nov). It found no blockers. Its two majors and four minors are applied:
- the export gets its own control (I-xc): claude-313's seal is re-exported and must reproduce its artifact, so a faulty export cannot read as row 6;
- the held-out reading is labelled as confounded by fewer D3 passes;
- the stopping-power claim is labelled an inference, with a "near the bar" band;
- the notes on the pair's `train=`, the header and the BOS boundary are stated.

**Why.**
- **(b64L) on D3 never stops.** claude-309 (#3828): 0 of 256 EOS stops at both seeds; every reply ran to the 64-token cap.
- **(b64L) on `mw-balp/train` alone stops, but costs the held-out text.** claude-313 (#3834), at the same recipe and pin:
  - 253 and 244 of 256 EOS stops;
  - seed 1 collapsed, with 73 identical EOS-stopped replies against the bar of 64; seed 2 had 40;
  - on the no-memory panel it was **behind** (b64L-D3) on every held-out split, pooled d −304,197 and −307,764 rows (band 3,567).
- **claude-313's decision review chose this mix:** D3 whole plus `mw-balp/train` whole, so the trunk keeps D3's held-out text while it sees mw-balp's stops.

## Question

At (b64L)'s recipe and pin, with TRAIN changed to D3 + `mw-balp/train`:
- does the served model still stop on EOS on `m2-fresh256`?
- do its stopped replies stay varied?
- does it keep (b64L-D3)'s no-memory hits on the held-out human text?

The held-out answer is the mix's, at a fixed step count. D3 is 69.2% of the windows seen: about 725,600 D3 windows, against 1,048,576 for (b64L-D3), so 4.73 passes over D3 against 6.84. A cost read here mixes two causes: fewer D3 passes, and mw-balp's content. This run cannot separate them.

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, claude-301's, claude-306's and claude-312's. Every pod is brought up with `--ref` at the pin, and the job refuses any other source.
- **The recipe:** claude-301's (b64L), unchanged:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2.
  - Only `--train-split`, `--train-sha256` and `--train-mask-sha256` change.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.
- **The served artifact:** `cpu-fbs-export` (`724a3b78…`) on the laptop, from each seed's sealed `models/step-016384`, with `--raw-gates --raw-transition` (claude-313: the export refuses a train-as-served checkpoint without both).
- **The scorer:** `served-reply-panel` at bin `be3d4cb0…` (#3828), unchanged.

## Data split

**TRAIN `d3mw`,** in the store at `results/claude/one-model-end-to-end-b64l-d3mw-2026-11-07/train/`, put by claude-314 and never overwritten:
- `tokens.u16` `1081853b3871ecb66e33fe1f212ae4415cccaba0fb123754949832ef3d2f223f`: 35,480,910 tokens after the 64-byte header;
- `response_mask.u8` `5bf91524fd65c05fc5284e3cbfc1d9d038d54a9eb2de12355b9bf9e9d86c1efa`;
- `manifest.json` `802b60ba80784e50e68f329957ca92471cab59f53f4a36df16e91169da9bfc9f`, schema `uor-r4-scale-probe-mix/v1`.

**How it was built:** `scale-probe mix fraction=1/1` (bin `31610a00…`) over five roots, in this order:
1. D3's four roots, whole, at D3's pinned shas (oasst2, wiki-00000, wiki-00020, wiki-00040);
2. then `mw-balp/train`, whole.

The same tool over the first four roots alone rebuilt D3 byte for byte: tokens `ed8f2537…`, mask `ec6e520c…` (scale-probe sheet, 5 Nov).

**Checks on the laptop:**
- the mix's first 27,968,113 mask bytes hash equal to D3's mask;
- its first 27,968,113 tokens of body, after the 64-byte header, hash equal to D3's body. The headers differ only in their count field;
- its last 7,512,797 mask bytes hash to `mw-balp/train`'s mask `b5e7352d…`;
- its tokens after D3's equal mw-balp's body;
- mw-balp's first token, at the boundary, is BOS (0). So D3's last document ends there and does not run into mw-balp's first.

**mw-balp's provenance:**
- `mw-balp/train` has no manifest in the store, so claude-314 wrote one for the mix's root. It records:
  - the two shas;
  - BOS 0;
  - the store path, and the tar `data/ft-balp.tar` it belongs to (`b1d22113…`).
- Its licence and its generator are **not recorded** in the store or the repository. The mix's manifest says so. This is open and is not decided here.

| part | documents | tokens | scored targets | scored EOS | EOS per million scored | usable windows |
|---|---:|---:|---:|---:|---:|---:|
| D3 | 18,016 | 27,968,113 | 27,416,554 | 20,729 | 756 | 153,253 |
| `mw-balp/train` | 60,000 | 7,512,797 | 2,425,925 | 212,968 | 87,788 | 68,218 |
| **the mix** | **78,016** | **35,480,910** | **29,842,479** | **233,697** | **7,831** | **221,471** |

- The EOS counts are positions with token 1 and mask 1, counted over the mix.
- The window counts come from the trainer:
  - mw-balp's from claude-313's log ("usable TRAIN windows 68218");
  - D3's from 1,048,576 ÷ claude-301's final epoch 6.842124.
  - The trainer cuts windows within documents, splitting on BOS (`build_document_windows`), and keeps a window only if it has at least 4 scored targets. mw-balp's first token is BOS, so the mix's count is their sum. The run's own log line is recorded, and I-3 checks it.

**The other inputs:**
- **DEV in-run evaluation:** claude-261's `ft/dev`, at the pins claude-306's job checks.
- **The no-memory panel:** claude-295's four held-out folders, at claude-306's per-folder pins: 1,460,697 rows, identity `5f88a5b3…`.
- **The served panel:** `m2-fresh256`, inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`, checked by the bin against its manifest.

## Repetition and exposure

The trainer samples windows uniformly by epoch (`epoch_order`). So:
- **Passes:** 16,384 × 64 = 1,048,576 windows seen, about **4.73 passes** over each part. That compares with 6.84 over D3 for (b64L-D3) and 15.37 over mw-balp for (b64L-mw).
- **mw-balp's share:** mw-balp is 30.8% of the windows seen but 8.1% of the scored targets: its documents are short, about 125 tokens each.
- **EOS targets seen:** about 1.06 scored EOS per window seen. That compares with 0.135 for (b64L-D3) and 3.12 for (b64L-mw).
- Where stopping falls between 0 at 0.135 and 244–253 at 3.12 is what this run reads. No prior result places it.

## Arms

| arm | TRAIN | batch (micro) | steps | source |
|---|---|---|---:|---|
| (b64L-D3), sealed | D3 | 64 (4 × 16) | 16,384 | claude-301 `runs-2/s{1,2}`; served scores claude-309 |
| (b64L-mw), sealed | `mw-balp/train` | 64 (4 × 16) | 16,384 | claude-313 `runs/s{1,2}`; served scores and grades claude-313 |
| (b64L-d3mw) | the mix | 64 (4 × 16) | 16,384 | this sheet's run unit, seeds 1 and 2 |

One arm to train, at two seeds, one pod per seed. The steps are fixed: no gate cuts them.

## Readings

At seeds 1 and 2, from `served-reply-panel` on (b64L-d3mw)'s exported `step-016384`, on the same measures as claude-313:
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows only, the count of the most common reply text. Also recorded: distinct reply texts over all rows, and the most common reply over all rows.
- **Reply length:** the median of `rows[].reply_ids` lengths.

**The held-out pair, which decides rows 2 and 3:**
- `scale-probe grade` with `train=` the mix, on all 1,460,697 rows, with no `clean=`.
- Then `scale-probe pair`, (b64L-d3mw) against claude-301's sealed (b64L-D3) grades:
  - `e17a13548d09304d901a93683cadb6c7695200261e86770e319003bfc976523a`
  - `3170c779def7ef376fdc11c96232f3f6ddb2809c436855121fd24be8ed7ab5a1`
- The pooled verdict over the four held-out splits is `behind`, `inside` or `ahead`, at the pair's band, max(2·√(b+c), the two-seed spread).
- The two arms' grades are made with different `train=` arguments: the mix here, D3 for claude-301's. `train=` builds only the T2 and unigram baselines. It does not change which rows are graded or what counts as a nearest hit; I-4 pins the row set. claude-313's pair rested on the same reading.

Recorded beside them, descriptive only:
- the pair against claude-313's (b64L-mw) grades (`395ea54373fd108bc502fc697849b12056b1f2a8dd538edf8c63f0afe8f35e2d`, `aa725ed9cce066a049625ba4b7efb949af189313cc0e444b08d4370d0f94aad2`);
- the pairs against the mix's own T2 and unigram;
- the DEV curve (`dev_nearest.hits` every 1,024 steps), against (b64L-D3) and (b64L-mw);
- the loss at the last step, within the arm only.

## Bars

Fixed before any run. They are claude-312's, unchanged, with claude-313's figures beside them.
- **Stopping:** at least **128 of 256** EOS stops at **both** seeds. (b64L-D3): 0 and 0; (b64L-mw): 253 and 244.
- **Collapse:** a seed is collapsed if, among its EOS-stopped rows, one reply text covers **at least 64**. (b64L-mw): 73 (collapsed) and 40.
- **The panel's bar:** at least 104 rows complete and at least 52 pairs at both seeds. Every model scored so far has 0.
- **The held-out cost:** `behind` (b64L-D3) on the pooled verdict above. (b64L-mw) was behind by about 304,000 rows.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | a seed NOT_RUN after its rerun, for an instrument fault (a gate, out of memory, setup, a lost pod) | **`blocked`.** Repair the instrument and rerun the seed under this sheet. No reading on one seed. |
| 0b | a seed NOT_RUN after its rerun, for a mechanism cause (a non-finite loss at a recipe that trains on both parts alone) | **`diagnostic stop`,** recorded as a result about (b64L) on the mix. Next: a decision review in the REPORT. |
| 0c | I-c or I-xc fails, or an export or score fails on the laptop with both seals present | **`blocked`, no pod.** Repair the export or scorer and re-read from the seals. |
| 1 | the panel's bar met at both seeds | **The served model answers.** Collapse and the pair are still recorded. Next: the trunk trains on the mix under its own sheet, at the next scale. |
| 2 | stopping met, neither seed collapsed, and the pair is not `behind` | **The mix stops, varies and keeps D3's held-out text.** Recall is the remaining gap. Next: a decision review in the REPORT on the recall lever (the panel's complete bar). |
| 3 | stopping met, neither seed collapsed, and the pair is `behind` | **The mix stops and varies, at a held-out cost.** The REPORT sets that cost beside claude-313's −304,000. It cannot say how much comes from fewer D3 passes (4.73 against 6.84) and how much from mw-balp's content. The ratio review takes that as an open input. Next: a decision review on the ratio. |
| 4 | stopping met, a seed collapsed | **The mix stops, but its stopped replies still collapse.** Next: a decision review on a larger or more varied dialogue corpus. |
| 5 | at least 8 EOS stops at some seed, stopping not met (a seed split such as 256 / 0 lands here) | **D3 dilutes the stopping signal below the bar.** Next: the ratio lever, for example mw-balp's windows repeated in the mix, under its own sheet. |
| 6 | fewer than 8 EOS stops at each seed | **The mix does not make (b64L) stop,** configuration only (§4.1). mw-balp alone did, at 3× the EOS targets per window. Next: a decision review in the REPORT on the ratio. |

A seed that is NOT_RUN reruns once, sealing to the new root `runs-2/s{1,2}/` (L-002). A second NOT_RUN reads by row 0a or 0b, by the class of its last cause.

**deepseek-25** (#3838) fine-tunes (b64L-D3) on an OASST2 + M-World mix: a different lever, fine-tuning against pretraining. If it lands first, the REPORT reads its stops beside these. It changes no bar and no row.

## Instrument checks

Registered before any pod goes up. They are claude-312's, with TRAIN swapped:
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin, with at least one test run, none failed, and claude-300's three micro-batch tests passing by name.
- **I-p, the pin:** the job refuses any SHA argument but the pin (exit 2), and a source that is not a clean checkout at the pin (exit 3).
- **I-3:**
  - the mix's three files match the shas above;
  - the panel folders and the DEV files match claude-306's pins;
  - each run's `config.json` names the mix's tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed and the recipe's flags, with no turn memory;
  - the trainer's "usable TRAIN windows" line reads 221,471. Any other count is recorded and classed before any reading, since it means the window rule is not what this sheet assumed.
- **I-1, the smoke:** 128 steps at `--eval-every 32` on the mix, at the pod's seed. It exits 0 with a finite loss and writes `models/step-000064`.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. claude-306's (b64L) on D4, which has 74.0M tokens against the mix's 35.5M, peaked at 26,832 / 26,928 of 32,607 MiB.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB.
- **I-t, recorded only:**
  - r = (train_seconds at step 128 − at step 64) / 64;
  - W = used + 1.05 × r × 16,384 + 300 s, written to `it.txt`;
  - claude-313's (b64L-mw) trained in the same step time as claude-306's runs, since each step reads 64 windows padded to 192.
- **The watcher:** MAXSEC 14,400 s from the pod's start, a budget of PRICE × 4 h, and PRICE at most $0.70/h. Neither limit stops anything (D67): past either, the watcher logs one NOTICE and keeps watching and renewing the lease, so `seal=0` still takes the pod down.
- **I-5:** the no-memory scoring's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes.
- **I-4:** each graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-c, the scorer controls, on the laptop before any (b64L-d3mw) reading,** with `served-reply-panel` at `be3d4cb0…`. Each control's `rows` must equal the reference output's, field by field:
  - **Negative:** claude-308's seed-1 (b64L-D3) artifact (`9a370566…`) against claude-309's output `d02ec432…`: 0 EOS stops.
  - **Positive:** claude-313's seed-1 (b64L-mw) artifact (`1c9ae43c…`) against its output `5175c29b…`: 253 EOS stops.
- **I-x, the export:** each artifact's sha256 is recorded, and `served-reply-panel` accepts it.
- **I-xc, the export control, on the laptop before any (b64L-d3mw) reading:** `cpu-fbs-export` (`724a3b78…`, `--raw-gates --raw-transition`) re-exports claude-313's sealed seed-1 `step-016384`. The artifact's sha256 must equal `1c9ae43c…`, claude-313's served artifact. I-c then scores that artifact. So the export and the scorer are both checked on a model known to stop.
  - **Prep pass, 7 Nov:** the re-export reproduced `1c9ae43c3db8917766a06160075d0683bcfa116dcac42bdec24ac06bfbcb6ff0` byte for byte. The run unit repeats it before reading.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal and watch scripts.
2. On the laptop:
   1. The job is claude-312's `job-312.sh`, with TRAIN swapped to the mix:
      - `check_mw` becomes `check_train`, a check of the mix's three shas;
      - the two mw-balp fetches become one folder fetch of `results/claude/one-model-end-to-end-b64l-d3mw-2026-11-07/train/`;
      - `--train-split`, the two train shas and the config check name the mix.
   2. Its gate tests and mutations, `bash -n` on every script, I-c and I-xc.
3. `uor-pod status`, recorded. The cross-lab caps must hold with these two pods included: at most 4 pods and $8/h. Then two Vast pods:
   - one RTX 5090 each, one per seed, at most $0.70/h each;
   - leased and watched;
   - brought up with `--count 1 --ref` at the pin;
   - avoiding Vast machines 48282 and 48276.
4. On each pod, in order:
   1. I-p, I-0, the build, the inputs and I-3.
   2. The smoke with I-m and I-f, then I-t.
   3. (b64L-d3mw) for 16,384 steps.
   4. The no-memory scoring of `step-016384`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-b64l-d3mw-2026-11-07/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop:
   1. I-xc and I-c again, then export each seal's `models/step-016384`, then I-x.
   2. `served-reply-panel` on both artifacts: the stops, panel measures, collapse and reply length.
   3. `scale-probe grade` with `train=` the mix, then the pairs.
   4. The DEV curves.

## Minimum test power

- **Stopping** is a count. (b64L-D3) stopped on 0 of 512 rows over two seeds; (b64L-mw) on 497 of 512, a seed spread of 9.
  - That the bar decides here is an **inference**. This run's EOS exposure, 1.06 per window, sits between those ends, where no spread has been measured.
  - A seed within 32 of 128 (96 to 160) is reported as "near the bar", as collapse is near 64. The row still follows the bar.
- **Collapse** is a count over EOS-stopped rows. (b64L-mw)'s two seeds read 73 and 40, a spread of 33 about the bar of 64. A reading near 64 is not decisive, and the REPORT says so.
- **The held-out pair** uses `scale-probe pair`'s band, as claude-313's did: there it was 3,567 rows against a d of about −304,000. A cost of a tenth of claude-313's would be far outside claude-313's band of 3,567. This run's own band is not yet measured, so this is an inference.

## Cost

- **claude-313's measured runs:** $2.43 for both pods, on RTX 5090s.
- **Here:** the step time does not depend on TRAIN, since each step reads 64 windows padded to 192. So about $2.4 for both at claude-313's rate, and about $3.0 at the $0.70/h ceiling ($2.43 × 0.70 / 0.563).
- **The budget notice, not a cap:** 2 pods × 4 h × $0.70 = $5.60. A healthy run is never cancelled for a self-set budget. If a slow card bills past $5.60, the REPORT says by how much.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m or I-f fails:** the arm is NOT_RUN at that seed.
- **An out-of-memory error, a non-finite loss, or a trainer exit other than 0:** NOT_RUN at that seed, classified as instrument or mechanism before it is called anything (§4.2).
- **A pod lost mid-run, or one that never reaches SSH within 1,800 s:** an instrument NOT_RUN. It is deleted, and one retry runs on another machine.
- **The no-memory scoring fails or I-5 fails:** the served readings stand. Rows 2 and 3 then cannot be told apart; the REPORT reads row "2 or 3" and re-scores from the seal.
- **Always:** the job seals on every exit, and the watcher takes the pod down after `seal=0`.

## Changes from claude-312's sheet

1. **TRAIN:** the D3 + `mw-balp/train` mix in place of `mw-balp/train`, at the store path and shas above, with its manifest checked too.
2. **The held-out pair** against (b64L-D3) now decides between rows 2 and 3. In claude-312's sheet it was descriptive.
3. **I-c's positive control** is claude-313's (b64L-mw) seed 1, at this recipe, in place of claude-259's (b).
4. **I-3 adds the window count,** since the mix's sum rests on the trainer's window rule. **I-xc** is new: an export control on claude-313's seal.
5. **The outcome map** has a ratio row (5) and a no-stop row (6) in place of claude-312's recipe suspects. The recipe is now known to stop on mw-balp.
6. **The seal path** is `results/claude/one-model-end-to-end-b64l-d3mw-2026-11-07/runs/s{1,2}/`, and `runs-2/` for a rerun.

## Pre-run review

**GO** (a fresh D67 review, 7 Nov: no blockers, no majors), for the run unit, on these scripts in `local/claude/claude-314/`. Its two minors are applied:
- a gate test and a mutation (`fetch-other`) pin the TRAIN fetch to `TR`'s folder. A wrong fetch path was already safe, since `check_train` refuses before training, but it would have cost the build;
- the window count's variable is `win`, no longer sharing `w` with I-t's wall.

The reviewer confirmed:
- `uor-store get REMOTE/ DIR` writes `DIR/REMOTE/`, so the fetch lands in `TR`;
- the trainer at the pin reads and sha-checks `TR`'s two files;
- the seal's empty-root probe is on `.../runs/sN`, so the stored `train/` cannot block it;
- the watcher differs from claude-312's only in names.

- `job-314.sh` `ce50f73894eb7bd5421d089ef7b86651c6974e78636867938069e96edb7374d2`
- `chain-314.sh` `7ca161a4fcdea32f215a106840343f6df6b36a631bde67065f1a2730875c166c`
- `seal-314.sh` `5a56d59f0eecbdc00336b95800e363bf440b19b2b70d5ffb550b5afde97cbf7e`
- `watch-314.sh` `565e5c3dd5f6c85bdc354f1b57bad74249bd77ba9648250642ae9dd7a4f61ebe`

**Laptop gates at these hashes:**
- `gt/test-gates.sh`: 29 ok, 0 FAIL, including the window-count line on claude-313's real smoke log and the fetch target;
- `gt/mutate.sh`: 10 of 10 mutations caught. Each produces only the FAIL lines its own gate owns. The new ones drop the manifest check and change the fetch target;
- I-c: both controls' rows equal their references (0 and 253 EOS stops); I-xc: the re-export reproduced `1c9ae43c…`;
- `gt/test-seal.sh`: 7 ok;
- `bash -n` ok on every script.
