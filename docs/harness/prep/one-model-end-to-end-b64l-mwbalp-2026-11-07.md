# Prep sheet: (b64L) on TRAIN `mw-balp/train`, the corpus lever on stopping (one-model-end-to-end, claude-312, 7 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-312. The step is Casey's "Yes, pipeline first" (#2028 6709947526): the next training run is judged by the served-reply score.

TEST FITNESS: FIT WITH CHANGES, round 1 (a fresh non-author review, 7 Nov). It found no blockers. Its three majors and six minors are applied:
- collapse is counted among EOS-stopped rows only, with (b64L-D3)'s figure on the same measure, since (b64L-D3) seed 1 already repeats one looped reply 76 times;
- I-c gains a positive control, claude-259's (b) seed 1, which stops on EOS;
- repetition is counted in the trainer's document windows, not in positions;
- I-t's W, claude-306's wall figures, the job's I-3 lines, row 4's floor, row 5's suspects and the layout inference are corrected.

**Why.**
- **(b64L) on D3 never stops.** claude-309 (#3828) scored claude-301's sealed (b64L) at 16,384 steps, exported to the integer artifact, on `m2-fresh256`: 0 of 256 rows complete, and every reply ran to the 64-token cap (0 EOS stops) at both seeds.
- **D3 rarely shows EOS as a target.** claude-310 (#3829): 756 scored EOS per million scored targets in D3, against 85,812 in DEV.
- **Models trained on `mw-balp/train` do stop.** claude-311 (#3830): `mw-balp/train` has 87,788 per million, and claude-259's (b) and (h) arms on it stop on EOS on 227–256 of 256 prompts. They answer none, and 128–152 of the 256 replies are one stock line.
- **Those arms differ from (b64L) in recipe too:** objective `cell` against `nearest`, batch 16 against 64, turn memory 8 against none, 8,192 steps against 16,384. So claude-311 could not credit stopping to the corpus.

## Question

At (b64L)'s recipe and pin, with only TRAIN changed from D3 to `mw-balp/train`, does the served model stop on EOS on `m2-fresh256`? And at what cost to DEV and the no-memory panel?

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, claude-301's and claude-306's. Every pod is brought up with `--ref` at the pin, and the job refuses any other source.
- **The recipe:** claude-301's (b64L), unchanged:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2.
  - Only `--train-split`, `--train-sha256` and `--train-mask-sha256` change.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.
- **The served artifact:** `cpu-fbs-export` (claude-308's build, `724a3b78…`) on the laptop, from each seed's sealed `models/step-016384`.
- **The scorer:** `served-reply-panel` at bin `be3d4cb0…` (#3828), unchanged.

## Data split

- **TRAIN `mw-balp/train`,** in the store at `w2-inputs/2026-10-25/driver-data/mw-balp/train/`:
  - `tokens.u16` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, 15,025,658 bytes: 7,512,797 tokens after the 64-byte header;
  - `response_mask.u8` `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, 7,512,797 bytes;
  - 2,425,925 scored targets, 212,968 of them EOS (claude-311's `mwbalp.json`, `d7179a4b…`).
  - The store folder holds these two files only, with no `manifest.json`. The job's I-3 checks the two shas directly. `scale-probe grade` loads TRAIN with `need_manifest = false` (`lib.rs:555`).
  - 60,000 documents (60,000 BOS tokens).
  - This is the corpus claude-259's (b) and (h) trained on. Their served artifacts stop under `served-reply-panel`'s rendering. That the corpus's dialogue layout matches the panel's is an inference from this, not a check of the corpus's encoder.
- **DEV in-run evaluation:** claude-261's `ft/dev`, at the pins claude-306's job checks.
- **The no-memory panel:** claude-295's four held-out folders, at claude-306's per-folder pins: 1,460,697 rows, identity `5f88a5b3…`.
- **The served panel:** `m2-fresh256`, inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`, checked by the bin against its manifest.

## Repetition

The trainer reads document windows, not a flat stream. At the pin, `build_document_windows` (`hamilton_sheet.rs:1888-1896`) cuts each document into chunks of at most 192 targets, never crosses a document start, and pads a short window with BOS.
- **The window count:** at least the 60,000 documents, and at most 60,000 + 7,512,797 / 192 ≈ 99,130. The run's own count is recorded from the trainer's log.
- **Passes:** 16,384 steps × 64 = 1,048,576 windows seen, so about **10.6 to 17.5 passes** over `mw-balp/train`'s windows. D3 at the same steps was about 7.2 (claude-307's sheet, `one-model-end-to-end-d3-tokens-2026-11-07.md`).
- **Scored targets per step are far fewer than on D3.** The mean document is about 125 tokens, so most windows are one short, padded document, and about a third of each document's targets are scored.
- **A stock-line reply is a likely outcome** at this many passes. It is not counted against the stopping bar, since stopping is what this lever reads. It is recorded as its own reading (collapse, below), and it decides the next unit through the outcome map.

## Arms

| arm | TRAIN | batch (micro) | steps | positions | source |
|---|---|---|---:|---:|---|
| (b64L-D3), sealed | D3 | 64 (4 × 16) | 16,384 | 201.3M | claude-301 `runs-2/s{1,2}`; served scores claude-309 |
| (b64L-mw) | `mw-balp/train` | 64 (4 × 16) | 16,384 | 201.3M | this unit, seeds 1 and 2 |

One arm to train, at two seeds, one pod per seed. The steps are fixed: no gate cuts them.

## Readings

At seeds 1 and 2, from `served-reply-panel` on (b64L-mw)'s exported `step-016384`:
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows only, the count of the most common reply text, by `jq` over `rows[]` where the stop is EOS. Also recorded: the distinct reply texts over all 256 rows, and the most common reply over all rows, as in claude-311. (b64L-D3)'s and claude-259's figures are recomputed from their outputs on the same measures.
- **Reply length:** the median of `rows[].reply_ids` lengths.

Recorded beside them, descriptive only:
- **The DEV curve:** `progress.jsonl`'s `dev_nearest.hits` at every 1,024 steps, against claude-301's (b64L-D3). A peak followed by a fall reads as over-fitting at 10.6 to 17.5 passes.
- **The no-memory panel:** `scale-probe grade` with `train=` `mw-balp/train`, which builds that corpus's T2 and unigram. Two pairs, with no `clean=`, on all rows:
  - (b64L-mw) against (b64L-D3)'s sealed grades (claude-301's `e17a1354…`, `3170c779…`): the cost of the swap on human held-out text;
  - (b64L-mw) against `mw-balp`'s own T2 and unigram.
- **The loss** at the last step, within the arm only.

## Bars

Fixed before any run.
- **Stopping:** "the corpus makes (b64L) stop" holds if (b64L-mw) stops on EOS on **at least 128 of 256** rows at **both** seeds. (b64L-D3) stopped on 0 of 256 at both.
- **The panel's bar:** at least 104 rows complete and at least 52 pairs, at both seeds. Reported; it decides row 1 only.
- **Collapse:** a seed is collapsed if, among its EOS-stopped rows, one reply text covers **at least 64**. The count is over EOS-stopped rows, because (b64L-D3) seed 1 already repeats one looped, capped reply 76 times (claude-309). A count over all rows would read a capped loop as the corpus's collapse.
  - **Reference figures on this measure,** computed from the existing outputs: claude-259's (b) seeds 1–3 have 129 / 128 / 128, and its (h) seeds have 152 / 146 / 128. (b64L-D3) has 0 / 0, against 76 / 36 over all rows.

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | a seed NOT_RUN after its rerun, for an instrument fault (a gate, out of memory, setup, a lost pod) | **`blocked`.** Repair the instrument and rerun the seed under this sheet. No reading on one seed. |
| 0b | a seed NOT_RUN after its rerun, for a mechanism cause (a non-finite loss at a recipe that trains on D3) | **`diagnostic stop`,** recorded as a result about (b64L) on this corpus. Next: a decision review in the REPORT. |
| 0c | I-c fails, or an export or score fails on the laptop with both seals present | **`blocked`, no pod.** Repair the export or scorer and re-read from the seals. |
| 1 | the panel's bar met at both seeds | **The served model answers.** Next: the trunk trains on `mw-balp/train` (or a D3 mix) under its own sheet, with the no-memory cost read from this unit. |
| 2 | stopping met, neither seed collapsed | **The corpus makes (b64L) stop, and its replies vary.** Recall is the remaining gap. Next: a D3 + `mw-balp` mix at (b64L)'s recipe, so the trunk keeps D3's held-out gains, under its own sheet. |
| 3 | stopping met, a seed collapsed | **The corpus makes (b64L) stop, and its stopped replies collapse at this recipe too,** as claude-259's did at theirs. Next: a decision review in the REPORT on a larger dialogue corpus or the mix. |
| 4 | at least 8 EOS stops at some seed, stopping not met (a seed split such as 256 / 0 lands here) | **The corpus raises stopping, not to the bar.** Next: a decision review using the stop counts and the DEV curve. |
| 5 | fewer than 8 EOS stops at each seed | **The corpus alone does not make (b64L) stop,** configuration only (§4.1). claude-259's other differences are the next suspects: objective `cell`, turn memory 8, batch 16, 8,192 steps, and whatever makes its served artifacts 51,310,784 bytes against (b64L)'s 2,156,416 at the same width 256. Next: a decision review in the REPORT. |

A seed that is NOT_RUN reruns once, sealing to the new root `runs-2/s{1,2}/` (L-002). A second NOT_RUN reads by row 0a or 0b, by the class of its last cause.

## Instrument checks

Registered before any pod goes up. They are claude-306's, with TRAIN swapped:
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin, with at least one test run and none failed, and claude-300's three micro-batch tests passing by name.
- **I-p, the pin:** the job refuses any SHA argument but the pin (exit 2), and a source that is not a clean checkout at the pin (exit 3, `end 3` after `fetch_src` at `job-312.sh:230`). The laptop gate test checks both: the pin refusal by running the job, with a mutation (`pin-other`), and `fetch_src` as a function on a clean, another-commit and a dirty checkout, with no mutation.
- **I-3:**
  - the two `mw-balp/train` files match the shas above;
  - the panel folders and the DEV files match claude-306's pins;
  - each run's `config.json` names `mw-balp/train`'s tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed and the recipe's flags, with no turn memory.
- **I-1, the smoke:** 128 steps at `--eval-every 32` on `mw-balp/train`, at the pod's seed, exits 0 with a finite loss and writes `models/step-000064`.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. claude-306's (b64L) peaked at 26,832 / 26,928 of 32,607 MiB; TRAIN's size changes no tensor.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB.
- **I-t, recorded only:** r = (train_seconds at step 128 − at step 64) / 64. W = used + 1.05 × r × 16,384 + 300 s, written to `it.txt`. claude-306's own projections were 12,967 / 13,054 s, so W is about 13,000 s. At claude-301's slowest rate, 0.698 s a step, W is about 14,100 s.
  - **The watcher:** MAXSEC 14,400 s (4 h) from the pod's start, and a budget of PRICE × 4 h.
  - **Neither limit stops anything** (D67 review). Past MAXSEC or BUDGET the watcher logs one NOTICE and keeps watching. It keeps renewing the lease every 25 minutes, so `seal=0` still takes the pod down. The REPORT records any NOTICE.
  - If W exceeds MAXSEC less the pod's setup time (`up-sS.epoch`), a NOTICE is likely. On a slow card that is plausible, and the spend can then pass $5.60, still near the ORDER's "about $6".
- **I-5:** the no-memory scoring's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes, as in claude-306's job.
- **I-4:** each graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-c, the scorer controls, on the laptop before any (b64L-mw) reading,** with `served-reply-panel` at `be3d4cb0…`. Each control's `rows` must equal the reference output's, field by field (the rows carry no timing):
  - **Negative:** claude-308's seed-1 (b64L-D3) artifact (`9a370566…`) against claude-309's seed-1 output `d02ec432…`: 0 EOS stops, 256 cap stops.
  - **Positive:** claude-259's (b) seed-1 artifact against claude-311's output `19849a18…`: 242 EOS stops, 14 cap stops. A scorer that never saw EOS would fail this one.
- **I-x, the export:** each exported artifact's sha256 is recorded, and `served-reply-panel` accepts it (it refuses an artifact that serves the writer head).

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal and watch scripts.
2. On the laptop:
   1. The job is claude-306's `job-306.sh`, with TRAIN swapped to `mw-balp/train`, and the seal root and watcher limits changed. The I-3 lines that change:
      - `check_d4` (`job-306.sh:106-112`), which checks D4's root `SHA256SUMS`, its three `./mix/` lines and a `manifest.json`, becomes a check of the two `mw-balp/train` shas;
      - the two D4 fetches (`:250-251`) become fetches of the two files under `w2-inputs/2026-10-25/driver-data/mw-balp/train/`;
      - `--train-split` and the two train shas (`:141`), and the config check (`:168`), name `mw-balp/train`.
   2. Its gate tests and mutations.
   3. `bash -n` on every script.
   4. I-c.
3. `uor-pod status`, recorded. The cross-lab caps must hold with these two pods included: at most 4 pods and $8/h. Then two Vast pods with one RTX 5090 each, one per seed, at most $0.70/h each, leased and watched, brought up with `--count 1 --ref` at the pin. Vast machines 48282 and 48276 are avoided, as in claude-307's sheet.
4. On each pod, in order:
   1. I-p, I-0, the build, the inputs and I-3.
   2. The smoke with I-m and I-f, then I-t.
   3. (b64L-mw) for 16,384 steps.
   4. The no-memory scoring of `step-016384`: claude-306's `--score-rows`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-b64l-mwbalp-2026-11-07/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop:
   1. `cpu-fbs-export` from each seal's `models/step-016384`, then I-x.
   2. `served-reply-panel` on both artifacts; the stops, panel measures, collapse and reply length.
   3. `scale-probe grade` with `train=` `mw-balp/train`, then the two pairs.
   4. The DEV curves against claude-301's.

## Minimum test power

- **The stopping bar** is a count, not a paired difference. (b64L-D3) stopped on 0 of 512 rows over two seeds. A reading of 128 or more at each seed is far outside any seed spread seen on this panel: claude-311's six artifacts ranged 227–256.
- **The panel's bar** needs 104 rows complete. Every model scored so far has 0, so it can be met only by a large change; a miss is the likely outcome and decides nothing past row 1.
- **The no-memory pair** against (b64L-D3) uses `scale-probe pair`'s band, `max(2·√(b+c), the two-seed spread)`. It is descriptive: a `behind` is the expected cost of training on 3.7× fewer tokens, and it decides no row.

## Cost

- **claude-306's measured (b64L) runs,** on RTX 5090s at $0.563/h: 10,409 / 10,489 s of training (its REPORT's table), and $1.81 billed a pod for the whole pod. The step time does not depend on TRAIN: each step reads 64 windows padded to 192.
- **Here:** about $1.81 a pod at $0.563/h and $2.26 at the ceiling of $0.70/h; $3.6–4.5 for both.
- **The budget notice, not a cap:** 2 pods × 4 h × $0.70 = $5.60, inside the ORDER's "about $6". Past it, the watcher logs a NOTICE and keeps watching. The charter's extend rule holds: a healthy run is never cancelled for a self-set budget. So a slow card can bill past $5.60, and the REPORT then says by how much.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m or I-f fails:** the arm is NOT_RUN at that seed.
- **An out-of-memory error, a non-finite loss, or a trainer exit other than 0:** NOT_RUN at that seed, classified as instrument or mechanism before it is called anything (§4.2).
- **A pod lost mid-run, or one that never reaches SSH within 1,800 s:** an instrument NOT_RUN. It is deleted, and one retry runs on another machine.
- **The no-memory scoring fails or I-5 fails:** only that descriptive reading is lost; the served readings stand on the sealed checkpoint.
- **Always:** the job seals on every exit, and the watcher takes the pod down after `seal=0`.

## Changes from claude-306's sheet

1. **TRAIN:** `mw-balp/train` in place of D4, at the store path and shas above.
2. **The deciding reading:** the served artifact's stops on `m2-fresh256`, read on the laptop, in place of the no-memory bar against T2.
3. **The no-memory panel** is graded with `train=` `mw-balp/train` and read descriptively, with no seen set.
4. **I-c and I-x** are new: two scorer controls, one with no stops and one with 242, and the export check.
5. **The watcher** has MAXSEC 14,400 s and a budget of PRICE × 4 h, for one 16,384-step run and one scoring.
6. **The seal path** is `results/claude/one-model-end-to-end-b64l-mwbalp-2026-11-07/runs/s{1,2}/`, and `runs-2/` for a rerun.
7. **D67 review** (a fresh pre-run review of the scripts, 7 Nov: GO WITH CHANGES, no blockers). Applied:
   - passing MAXSEC or BUDGET logs one NOTICE and the watcher keeps watching, so `seal=0` still takes a pod down after 4 h. Before, the watcher exited and a healthy run that finished late would bill idle (L-004). A hung run is still caught by the stall stop;
   - the watcher refuses a PRICE above $0.70/h, the sheet's ceiling, in place of claude-306's $1.15/h (`watch-306.sh:26`);
   - `trained_ok` requires the checkpoint, `models/step-000064` for the smoke and `models/step-016384` for the run, with a gate test and a mutation.

## Pre-run review

**GO**, for the run unit, on these scripts in `local/claude/claude-312/`:
- `job-312.sh` `fcbaae10ae5e504996ac561cb4da7c7d464aa85d9c20926b886d7c7605b395dd`
- `chain-312.sh` `9fcef8ef53f86d0b0ee223d5aa33c31fc68e06a07469f532c7353cb0e8a42fd1`
- `seal-312.sh` `3653c9bf29dcfe16d8cb993296c2e20b3735640fa63db876d5a456f9174c94d5`
- `watch-312.sh` `8d77c59c0e3228ea3b6aa0481346052764f86598d0890c3b3374ff61b7ed59aa`

**The chain, end to end:**
1. The pin's trainer loads `mw-balp/train` through `--train-split` (`load_named_split`, `in_space_long.rs:11214`), which re-checks both shas. It is also the pin's default TRAIN.
2. The job fetches TRAIN into the folder it trains from.
3. The seal is refused into a non-empty root.
4. Export, serving and scoring are laptop tools already merged, with both I-c controls passing.

**Laptop gates at these hashes:**
- `gt/test-gates.sh`: 24 ok, 0 FAIL;
- `gt/mutate.sh`: 8 of 8 mutations caught;
- `gt/test-seal.sh`: 7 ok;
- `bash -n` ok on every script.

**What each result unlocks** is the outcome map above. **The lease:** 2 pods at ≤ $0.70/h inside the 4-pod, $8/h caps, checked with `uor-pod status` at launch.
