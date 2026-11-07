# Prep sheet: (b64L) on TRAIN D4, more human text at width 256 (one-model-end-to-end, claude-306, 7 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-306. The one track is Casey's, 6 Nov: "Yes, one track" (#2028 6675163438). Every claude unit works to make the trunk's no-memory top-1 beat its TRAIN's unigram and T2 on the human held-out panel, beyond the paired band, at every seed.

TEST FITNESS: FIT WITH CHANGES, round 2 (a fresh non-author review, 7 Nov). It found no blockers, and said the sheet is fit once its major is applied. That major and its five minors are applied:
- the outcome map is read first-match, with no overlap and every behind case covered;
- I-r checks the input shas, and I-s checks the lever's seen set;
- a NOT_RUN seed reruns once.

Round 1, also FIT WITH CHANGES, found no blockers; its three majors and six minors are applied. This is the FIT's two-round cap.

**Why.**
- **claude-301 (#3800) met the bar** with (b64L) on TRAIN D3, "met; unseen ahead". Its tokens lever at batch 64 read `ahead`.
- **claude-304 (#3810) stopped its width-1,024 run** by a decision review. The review set the outcome map's default as the next unit: more human text at width 256.
- **claude-305 (#3813) built TRAIN D4** on the laptop for $0: 74,039,248 tokens, 2.65× D3.
  - Each of D3's roots is a byte prefix of D4's.
  - The panel is unchanged.
  - D4's T2 gets 256,443 panel rows (D3's got 256,660), and its unigram gets 37,421.
- **This unit trains (b64L) unchanged on D4**, at claude-301's pin, seeds and steps. The training text is the only thing that changes.

## Question

At (b64L)'s recipe, pin and 201.3M positions, does more distinct human text move the trunk? Each run sees D3 7.2 times, but D4 only 2.72 times. Two readings:
- **The bar:** does (b64L) on D4 stay ahead of D4's T2 and unigram at both seeds?
- **The data lever:** is (b64L) on D4 ahead of claude-301's (b64L) on D3?

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, claude-301's. This keeps the trainer byte-for-byte the same, so the data lever compares text alone.
  - `main` has moved on. claude-303 (#3809) and DeepSeek's #3812 changed `crates/uor-r4-float-train`. They are not in this run.
  - Every pod is brought up with `--ref ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`. A pod that boots at any other source is refused by the job's pin check: claude-304's third pod booted at `main`.
- **The recipe:** claude-301's (b64L), unchanged:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2, plus `--train-split` D4.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.

## Data split

- **TRAIN D4:** sealed by claude-305 at `results/claude/one-model-end-to-end-d4-2026-11-07/`. Its `SHA256SUMS` is `51ae9b9dcb685646fa29b01030680f3907b51e91dff906c11332284071634a22` and covers 39 files.
  - `mix/tokens.u16` `3a5b89f9fd99326eb18f57027eb38f2d8ef2d200e907d0c9a7e20553c78b0169`
  - `mix/response_mask.u8` `9fe9dedeccd599c0b9abd59c1864a414f999bb43f53529ba8e5013d35da10807`
  - `mix/manifest.json` `d98a4461b871c7c8e0be87fead7a9e9dbf45f4c5078cdaf63522012a68c47ba9`
- **D4's roots:** oasst2 3,773,419 tokens (as stored), wiki-00000 22,752,102, wiki-00020 23,962,093 and wiki-00040 23,551,634.
  - OASST2's share of TRAIN falls from 13.5% in D3 to 5.1%.
- **The panel:** claude-295's four held-out folders, unchanged: 1,460,697 no-memory rows, identity `5f88a5b3…`. Pins per folder (tokens, mask):
  - oasst2 `02a04e97…f169e` / `9c5bb656…4833`
  - wiki-00000 `82b3a40c…09f7` / `34fe6b68…386f`
  - wiki-00020 `9b7f1432…d640` / `a6803ab3…e00d`
  - wiki-00040 `c927243b…962d` / `9d95ea6d…a625`

  These are the same pins claude-301's job checks, given there in full.
- **D4's tables** (claude-305's seal, `tables/`):
  - T2 gets 256,443 rows, and the unigram 37,421.
  - Per sub-panel, T2 / unigram: oasst2 29,531 / 4,601; wiki-00000 71,051 / 6,851; wiki-00020 78,401 / 14,538; wiki-00040 77,460 / 11,431.
- **D4's seen set** (`seen/`, `excluded.bin` `511b0289…`): 282,585 rows seen, 1,173,177 unseen and 4,935 short. D3's seen set holds 227,312 seen.
  - D3's text is inside D4, so every row unseen in D4 is also unseen in D3.
- **DEV in-run evaluation:** claude-261's `ft/dev`, at the pins claude-301's job checks.

## Arms

| arm | TRAIN | batch (micro) | steps | positions | source |
|---|---|---|---:|---:|---|
| (b64L-D3), sealed | D3 | 64 (4 × 16) | 16,384 | 201.3M | claude-301, `runs-2/s{1,2}` |
| (b64L-D4) | D4 | 64 (4 × 16) | 16,384 | 201.3M | this unit, seeds 1 and 2 |

There is one arm to train, at two seeds, one pod per seed. The steps are fixed at 16,384: the data lever reads at equal positions, so no gate here cuts them.

## Readings

At seeds 1 and 2:
- **The panel:** (b64L-D4)'s whole-table nearest no-memory top-1, from `scale-probe grade` with `train=` D4.
- **Against D4's T2 and unigram:** `pair table=t2|unigram` with `clean=` D4's seen set, on all rows and on unseen rows, by sub-panel too.
- **The data lever:**
  - a = (b64L-D4), graded on D4.
  - b = claude-301's sealed grades of (b64L-D3), graded on D3, at `local/claude/claude-301/grade/b64L-s{1,2}`.
  - The pair uses no table, and `clean=` D4's seen set.
  - `pair` lets two cells differ in TRAIN. It refuses if a cell's own two seeds differ in TRAIN, and it refuses a `clean=` whose TRAIN is not a1's.
  - **The unseen section decides the lever** (FIT round 1). That is `pair`'s `unseen` block: rows whose context is in neither TRAIN, since D3 is inside D4.
    - D4 sees 282,585 − 227,312 = 55,273 more panel rows than D3. That is about 5× the bar's band of 10,552, and about 3× the lever's widest band of about 17,000 (Minimum test power). So the all-rows lever favours D4 by what it holds, not by what it learned.
    - "In neither TRAIN" holds up to the 8-grams that cross a root boundary in D3's mix, which D4's mix need not hold. The effect is negligible.
    - The all-rows lever is recorded beside the unseen one. An all-rows `ahead` with the unseen lever not `ahead` reads "the gain is on contexts D4 holds".
    - The sub-panels are read the same way, unseen first.
- **The oasst2 sub-panel, read on its own,** for both the bar and the lever. Its share of TRAIN falls 2.6×, so this caveat is named before any reading: if the lever is behind on oasst2 and ahead on the wiki sub-panels, that reads as a change in the mix share, not as "more text hurts".
- **T2's share:** `model_hits − b` over T2's hits, from the T2 pair.
- **The loss** at the last step, read within the arm only.
- **Peak GPU memory,** from `mem.log` inside the cell's `.window`, which is the file I-m gates on.

## Bar

Verdict words are `scale-probe pair`'s, at both seeds. Mixed reads "not distinguishable at these seeds", never as a negative (L-007). The table is claude-298's, with D4's tables:

| (b64L-D4) against D4's T2 (whole panel, all rows) | against D4's unigram | reading |
|---|---|---|
| ahead | ahead | **The one-track bar is met on D4.** The unseen-row verdict is written beside it as `pair` returns it, for example "met; unseen ahead", "met; unseen inside" or "met; unseen behind (the lead is on contexts TRAIN holds)". |
| ahead | not ahead | **Not met.** The grade is checked. |
| mixed or inside | any | **Not distinguishable from T2 at these seeds.** |
| behind | any | **Not met,** for this configuration only (§4.1). |

**The data lever is not a bar.** It reads `ahead`, `behind`, `inside` or `mixed`, and says only which way the trunk moves.

## What each outcome unlocks

"The lever" below is the unseen lever (Readings). The bar's verdicts are claude-298's four:
- "met": ahead of T2 and of the unigram;
- "ahead of T2 only";
- "not distinguishable": T2 `mixed` or `inside`;
- "behind T2".

**The rows are read top to bottom, and the first row that matches decides** (FIT round 2). Each (bar, lever) pair therefore has exactly one next unit.

| # | bar on D4 | unseen lever | the next unit |
|---|---|---|---|
| 1 | ahead of T2 only | any | **Not met.** The grade is checked against claude-305's tables. If it holds, this is a result about D4: the trunk keeps D3, and the next unit is a decision review on the unigram gap. |
| 2 | any other | `behind` | First the grade and I-r are checked. If they hold, the sub-panels' unseen verdicts decide, first match again: |
| 2a | | any wiki sub-panel `behind` | **D4 is a worse TRAIN for this recipe.** The trunk keeps D3. Next: the tokens lever on D3, (b64L) at 32,768 steps. |
| 2b | | oasst2 `behind`, no wiki sub-panel `behind` | The mix share, as named in Readings. Next: D4 with OASST2's share restored to D3's 13.5%. |
| 2c | | no sub-panel `behind` | The pooled lever alone is behind. The trunk keeps D3, and the next unit is a decision review in the REPORT. |
| 3 | met | `ahead` | **Distinct text moves the trunk at equal positions.** D4 replaces D3 as the trunk's TRAIN. Next: more human text again (a D5: more wiki shards, the OASST2 share restored), at width 256. |
| 4 | met | `inside` | **At 201.3M positions, 7.2 passes of D3 cost nothing measurable against 2.72 of D4.** Text is not the limit at this budget. D4 replaces D3, since it holds D3 and loses nothing. Next: the tokens lever on D4, (b64L) at 32,768 steps under its own sheet. claude-301's tokens lever read `ahead`. |
| 5 | met | `mixed` | **Not distinguishable at two seeds.** The trunk keeps D3. The sub-panel levers and the spreads pick the next unit, as a decision review in the REPORT. |
| 6 | not distinguishable | `ahead` or `inside` | **Not met on D4 at these seeds** (configuration only, §4.1). The trunk keeps D3. Next: the tokens lever on D3, since claude-301 met the bar there with room. |
| 7 | not distinguishable | `mixed` | The trunk keeps D3. Next: a decision review in the REPORT. |
| 8 | behind T2 | `ahead`, `inside` or `mixed` | **Not met,** configuration only. The trunk keeps D3. Next: a decision review recorded in the REPORT, before any further unit on D4. |

**A seed that is NOT_RUN** reruns once under this sheet, at the seal root `runs-2/`, before any reading (FIT round 2). If the rerun is NOT_RUN for the same class of cause (out of memory, divergence, a gate), that is recorded as the result. The table is then read as "not met on D4, configuration only" (row 8's next unit). No reading is made on one seed.

## Instrument checks

These are registered here, before any pod goes up.
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin.
  - At least one test must run, and none may fail.
  - The three micro-batch tests of claude-300 must pass by name.
- **I-3:**
  - **D4:** the seal's `SHA256SUMS` must have this sheet's sha, its three `./mix/` lines must check OK, and each mix file must match the shas above.
  - **The panel:** the four held-out folders and the DEV files must match claude-301's pins.
  - **Each run's `config.json`:** must name D4's tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed, and the recipe's flags, with no turn memory.
- **I-p, the pin (carried over from claude-301's job):** the job refuses any SHA argument but the pin (exit 2). The source it builds must be a clean checkout at exactly the pin, or it exits before building (exit 3). The new part is Step 3's `--ref` on `uor-pod up`, since `up` defaults to `main` (claude-304's third pod). The laptop gate test mutates both exits.
- **I-1, the smoke:** a 128-step run at `--eval-every 32` on D4, at the pod's seed, exits 0 with a finite loss.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. Otherwise the arm is NOT_RUN at that seed.
  - claude-301's (b64L) smokes read 27,083 / 26,988 of 32,607 MiB at I-m (83.1% / 82.8%). Its full runs peaked at 27,179 / 27,276.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB. Otherwise the arm is NOT_RUN at that seed.
  - What it cannot see, as in claude-301's sheet: growth slower than about 16 MiB a step passes I-f. The backstop is the run's own out-of-memory stop.
- **I-t, recorded only:** r = (train_seconds at step 128 − at step 64) / 64, from the smoke. The projected wall is W = used + 1.05 × r × 16,384 + 1,800 s, written to `it.txt`.
  - No gate acts on W.
  - If W exceeds 18,000 s, the 5 h a pod that the cap assumes, the watcher's limits are extended under the charter's extend rule, and the REPORT says so.
- **I-4:** every graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-5:** each scored checkpoint's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes, as in claude-301's job.
- **I-s:** D4's seen set names D4's `tokens_sha256` and the panel identity. `pair` refuses otherwise.
- **I-r, run on the laptop before any lever is read** (widened in FIT round 1):
  - claude-301's (b64L-D3) grades are re-paired against D3's T2, with `clean=` D3's seen set (`local/claude/claude-295/seen/d3`). The `scale-probe` build used is the one that grades D4, and its binary sha is recorded.
  - Every `d`, `band`, `spread` and `verdict` field of the result must equal claude-301's `pair/b64L-vs-t2.json`. That covers the whole panel (+75,038 / +85,590, band 10,552), the unseen rows (+56,353 / +64,841, band 8,488) and every sub-panel, so a wrong or stale exclusion path fails it.
  - Its `inputs_grade_sha256` must also equal the reference's: `a1`, `a2`, `table`, and `clean_seen_sha256` `53000ba1…`.
- **I-s on the lever** (FIT round 2): the data lever's `inputs_grade_sha256.clean_seen_sha256` must be the sha256 of D4's `seen.json` (`d5443f95…`), and that file's `excluded_sha256` must be `511b0289…`. Otherwise the lever is not read.
  - If it fails, the data lever is not read. (b64L-D4)'s bar still stands.
- **Not repeated: claude-301's I-e.** It is the equivalence check of accumulation against the whole batch on the real recipe.
  - It passed at this pin, recipe and binary, on both of claude-301's hosts: worst gap 1.3 × 10⁻⁷, control 6.8 × 10⁻⁵ / 1.85 × 10⁻⁵.
  - It tests the trainer's arithmetic, which this unit does not change. I-0 still runs claude-300's three micro-batch tests by name.
  - claude-304's I-e ran at another pin, and its control was underpowered there, so it is not cited here.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal and watch scripts.
2. On the laptop: the job's gate tests, with mutations, and `bash -n` on every script.
3. `uor-pod status`, recorded. The cross-lab caps hold with these two pods included: at most 4 pods and at most $8/h. Otherwise wait for room. Then two Vast pods with one RTX 5090 each, one per seed, at most $1.15/h each. Each is leased and watched, and is brought up with `--count 1 --ref ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`. Vast machines 48282 and 48276 are avoided (claude-304: driver 580.95.05, no SSH).
4. On each pod, in order:
   1. I-p, I-0, the build, the inputs and I-3.
   2. The smoke with I-m and I-f, then I-t (recorded only).
   3. (b64L-D4) for 16,384 steps.
   4. The scoring: claude-301's `--score-rows`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-d4-train-2026-11-07/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop:
   1. I-r.
   2. `grade` with `train=` D4.
   3. `pair` against D4's T2 and unigram.
   4. The data lever, and the oasst2 sub-panel on its own.

## Minimum test power

- **The paired band** is `max(2·√(b+c), the two-seed spread)`, `scale-probe pair`'s.
- **claude-301's (b64L) on D3:**
  - Against D3's T2, d was +75,038 / +85,590 with a band of 10,552. That band is all spread, so the bar here has about 7× headroom.
  - On the oasst2 sub-panel alone, the band was 1,515 against d +13,342 / +14,857.
- **The data lever** pairs D4's seed s with D3's seed s. That pairing is arbitrary, since the order of the data differs with TRAIN. Its spread |d₁ − d₂| can therefore run from near the McNemar floor to about the sum of both arms' own spreads: about 8,500 + 8,500 = 17,000 unseen rows if D4's spread is like D3's.
  - The McNemar floor, 2·√(b+c), is about 1,100 rows over the whole panel.
  - So an `inside` on the lever means "within that band", not "no effect".

## Cost

- **claude-301's measured (b64L) runs:** 10,924 s and 12,008 s of training on RTX 5090s at $0.986 and $1.00/h, about $2.99 and $3.34.
  - Its run-2 pods billed $4.84 and $5.48, with (b64S) at 5,444 / 6,035 s.
  - Taking claude-301's two pod totals, less both of its arms' training, leaves about 0.4 h a pod. On those pods that time covered setup, I-e, two smokes and two scorings. Here there is one smoke, one scoring and no I-e.
- **Here:**
  - Setup, the inputs, the smoke and the scoring: about 0.5 h.
  - Training: about 3.0–3.4 h.
  - So about 3.5–4 h a pod, or $3.50–4.60 at $1.00–1.15/h, and $7–9.20 for both.
- **The cap:** 2 pods × 5 h × $1.15 = $11.50. The charter's extend rule holds: a healthy run is never cancelled for a self-set budget.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m or I-f fails:** the arm is NOT_RUN at that seed, by the outcome map.
- **An out-of-memory error, divergence (a non-finite loss) or a trainer exit other than 0:** the arm is NOT_RUN at that seed. Each is classified as an instrument fault or a mechanism result before it is called anything (§4.2). A divergence on D4 at a recipe that trained on D3 is read as a result about D4 and recorded.
- **A pod never reaches SSH within 1,800 s:** it is deleted, and one retry runs on another machine. After two failures for a seed, that seed waits for a new RTX 5090 offer inside $1.15/h, and then reruns under this sheet, as the outcome map says. No reading is made on one seed meanwhile.
- **Always:** the job seals on every exit, and the watcher takes the pod down after `seal=0`.

## Changes from claude-301's sheet

1. **TRAIN:** D4 in place of D3, with its seal, shas, tables and seen set. The bar is read against D4's tables.
2. **One arm,** (b64L), at a fixed 16,384 steps. (b64S) and its levers are dropped.
3. **I-t records only.** No gate cuts steps, because the lever needs claude-301's step count.
4. **I-e is not repeated** (reason under Instrument checks). **I-r** is new. **I-p** is claude-301's pin check, named, with `--ref` added on `up`.
5. **The readings:** the data lever against claude-301's sealed grades, and the oasst2 sub-panel read on its own, with the mix-share caveat named in advance.
6. **The seal path** is `results/claude/one-model-end-to-end-d4-train-2026-11-07/runs/s{1,2}/`.
7. **FIT round 1:**
   - the unseen lever decides;
   - the outcome map is a table that covers every combination;
   - I-r checks every field;
   - the stop rule and the outcome map agree on a failed seed.
8. **FIT round 2:**
   - the outcome map is first-match;
   - I-r checks the input shas;
   - I-s checks the lever's seen set;
   - a NOT_RUN seed reruns once, to `runs-2/`.
9. **D67:**
   - the seal root is a parameter (`runs` or `runs-2`);
   - the watcher's budget is PRICE × 5 h with no rounding up, so the two pods stay inside $11.50.
