# Prep sheet: (b64L) on claude-322's ×6 capped mix + the current-state corpus (one-model-end-to-end, claude-325, 8 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-325. The step is Track A of Casey's "Both in parallel" (#2028 6715318106), under "Yes, pipeline first" (#2028 6709947526): the run is judged by the served-reply score.

TEST FITNESS: pending (see "FIT" at the end). No pod comes up before it and the pre-run review are filled.

**Why.**
- **claude-322's served replies answered the job and home questions with friend and dog templates.** Counted by `bare.jq` below, 73 of seed 1's 109 stops open "Your friend's name is …" and 28 "Got it, your friend is …"; at seed 2, 1 and 36 of 86.
- **No reply held the asked-for value.** 0 complete and 0 pairs at both seeds. Stops 109 and 86 of 256; collapse 9 and 6. Held-out text behind (b64L-D3) at both seeds, d −15,456 and −25,627.
- **The mix never taught the current-state task, or the bare `{value}.` reply.**
- **claude-324 built a 40,000-row current-state corpus in the panel's served form:**
  - one user message, the statements then the question, joined by spaces;
  - one assistant reply, exactly `{value}.`, then EOS;
  - values of 2 to 8 words from a 423-word list, with no word (singular or plural) in the panel's literals.

## Question

At (b64L)'s recipe and pin, with TRAIN changed to claude-324's mix (claude-322's ×6 capped mix, then the corpus):
- does the bare reply appear, and does `complete` move above claude-322's 0?
- does stopping hold at claude-322's level or better, at both seeds, without collapse?
- what does the corpus cost on the held-out human text, against (b64L-D3) and against claude-322's (b64L-d3mwcap6)?

**What the comparison can and cannot say.**
- **Against claude-322:** the recipe, the pin and the ten roots are the same. Only the corpus is added. So a change in the bare reply or in `complete` is read as the corpus's, since only the corpus teaches that form. A change in stopping or on held-out text is read as the mix's.
- **But the corpus also moves D3's passes** from 2.51 to about 2.29, and the capped copies' share of windows from 63.3% to about 57.8% (see "Repetition and exposure"). A held-out cost cannot be split between the corpus and D3's fewer passes.

## Base artifact

Unchanged from claude-322's sheet.
- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`. Every pod is brought up with `--ref` at the pin, and the job refuses any other source.
- **The recipe:** claude-301's (b64L), unchanged:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2.
  - Only `--train-split`, `--train-sha256` and `--train-mask-sha256` change.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.
- **The served artifact:** `cpu-fbs-export` (`724a3b78…`) on the laptop, from each seed's sealed `models/step-016384`, with `--raw-gates --raw-transition`.
- **The scorer:** `served-reply-panel` at bin `be3d4cb0…` (#3828), unchanged.

## Data split

**TRAIN `d3mwcs`,** claude-324's mix, in the store at `results/claude/one-model-end-to-end-currentstate-2026-11-08/train/`:
- `tokens.u16` `b34e72542273f3ac1b040f7ec5c940b07b132898d6862e6df5155b2b9962c191`: 76,368,774 tokens;
- `response_mask.u8` `a8aaa73bbd6c8ee6759cf6ea2a2e46a8eb620885274c1ba75f962d0535b1f50c`;
- `manifest.json` `5efff97971db6a05148f01153c626b4f7ef8177d084bee0da096a93542e6ec13`.

**How it was built:** `scale-probe mix fraction=1/1` (bin `31610a00…`) over eleven roots, in this order:
1. claude-322's ten roots: D3's four (`oasst2`, `wiki-00000`, `wiki-00020`, `wiki-00040`), then the k = 8 capped split as `cap1` to `cap6`;
2. then `currentstate`, the corpus.

**The corpus:** tokens `85641489ddc9f3af966545667cb558215dcbf3f36049205a139ca3e2146a79c2`, mask `febada2f1ed021c8553607a80b5fc09729e234f4070fa9cdb48717c95448b443`.

| part | documents | tokens | scored targets | scored EOS | usable windows |
|---|---:|---:|---:|---:|---:|
| claude-322's ×6 capped mix | 378,016 | 73,044,895 | 33,364,756 | 444,761 | 417,421 (logged) |
| the corpus | 40,000 | 3,323,879 | 474,325 | 40,000 | about 40,000 (expected) |
| **the mix** | **418,016** | **76,368,774** | **33,839,081** | **484,761** | **about 457,421 (expected)** |

- The corpus's rows hold 4 to 32 scored targets each, and its longest document is 188 tokens.
- **Its share of the mix:** 9.6% of documents, 4.4% of tokens, 1.4% of scored targets.
- **The window count:** the trainer cuts each BOS-delimited document on its own, so a mix's count is the sum of its parts'. Each corpus document (at most 188 tokens, under the 192 context) should add exactly one window. So 417,421 + 40,000 = 457,421 is expected. It is not yet measured; I-3 records the trainer's own count.

**The other inputs, unchanged from claude-322's sheet:** DEV (claude-261's `ft/dev`), the no-memory panel (claude-295's four folders, 1,460,697 rows, identity `5f88a5b3…`), and the served panel `m2-fresh256` (inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`). mw-balp's licence and generator are still not recorded.

## Repetition and exposure

The trainer samples windows uniformly by epoch. Over 16,384 × 64 = 1,048,576 windows seen, at about 457,421 windows:
- **Passes:** 1,048,576 / 457,421 ≈ **2.29** per window, against 2.51 in claude-322. D3 is 153,253 / 457,421 ≈ 33.5% of the windows (36.7% in claude-322).
- **The corpus:** 40,000 / 457,421 ≈ **8.7%** of the windows seen, about 91,700 windows. Each corpus row is seen about 2.29 times.
- **Scored EOS per window seen:**
  - claude-322: 444,761 / 417,421 = 1.07;
  - here: (444,761 + 40,000) / 457,421 = 484,761 / 457,421 ≈ **1.06**.
  - **So the count of scored EOS per window is matched, not the stopping it gives.** claude-322 showed that a matched count need not give matched stopping (×1's 177 and 122 against its own 109 and 86). The bound is the same as claude-322's sheet: EOS in chunks the trainer drops are never seen, under 1%.
- **The EOS share of scored targets:** 484,761 / 33,839,081 ≈ **1.43%**, against 1.333% in claude-322. The corpus's replies are short, so this share rises a little. Scored targets per window fall from 79.9 to about 74.0.

## Arms

| arm | TRAIN | batch (micro) | steps | source |
|---|---|---|---:|---|
| (b64L-D3), sealed | D3 | 64 (4 × 16) | 16,384 | claude-301 `runs-2/s{1,2}`; served scores claude-309 |
| (b64L-d3mwcap6), sealed | claude-322's ×6 capped mix | 64 (4 × 16) | 16,384 | claude-322 `runs/s{1,2}`; served scores and grades claude-322 |
| **(b64L-d3mwcs)** | the mix | 64 (4 × 16) | 16,384 | this sheet's run unit, seeds 1 and 2 |

One arm to train, at two seeds, one pod per seed. The steps are fixed: no gate cuts them.

**The comparison arm, on the same measures:**

| arm | seed | EOS per window / EOS share | EOS stops | collapse (EOS-stopped) | complete | pairs | held-out d against (b64L-D3) |
|---|---:|---:|---:|---:|---:|---:|---:|
| (b64L-d3mwcap6) | 1 | 1.07 / 1.333% | 109 | 9 | 0 | 0 | −15,456 (behind) |
| (b64L-d3mwcap6) | 2 | 1.07 / 1.333% | 86 | 6 | 0 | 0 | −25,627 (behind) |
| (b64L-d3mwcs), this run | 1, 2 | 1.06 / 1.43% | | | | | |

## Readings

At seeds 1 and 2, from `served-reply-panel` on (b64L-d3mwcs)'s exported `step-016384`, on claude-322's measures:
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows only, the count of the most common reply text, beside the number of EOS-stopped rows it is over. Also: distinct reply texts and the most common reply over all rows.
- **Reply length:** the median of `rows[].reply_ids` lengths.
- **EOS stops per stratum** (length2, length4, length8, reassert, update).

**The held-out pairs:**
- `scale-probe grade` with `train=` the local copy `local/claude/claude-324/mix`, on all 1,460,697 rows, with no `clean=`.
- **Deciding:** `scale-probe pair`, (b64L-d3mwcs) against claude-301's sealed (b64L-D3) grades `e17a13548d09304d901a93683cadb6c7695200261e86770e319003bfc976523a` and `3170c779def7ef376fdc11c96232f3f6ddb2809c436855121fd24be8ed7ab5a1`. The band, the per-seed positions, the pooled verdict and "behind" are as in claude-322's sheet.
- **Descriptive, named before the run:** the pair against claude-322's pinned (b64L-d3mwcap6) grades, `local/claude/claude-322/grade/b64Ld3mwcap6-s1/grade.json` `24d69af0455c9b18ab819bd73355e30ffa2f56db191f83e899ce0d8d4b092f7e` and `-s2/grade.json` `4407e2c2baf24ec94b1ba823316c109868f1fdd4322b4667ff136792518d3ec3`. This is the mix's cost on held-out text at the same recipe: the corpus, confounded with D3's fewer passes (2.29 against 2.51) and the capped copies' smaller share. Row (f) reads its mean d against the ceiling.

**The bare-reply count, deciding rows (c) and (e),** named before the run, by `bare.jq` (hash below). It counts among each seed's EOS-stopped replies, lower-cased and trimmed:
- **short:** one line ending in `.`, with no other `.`, `,`, `?` or `!`, of 1 to 8 words;
- **bare:** a short reply whose first whole word is not a sentence frame (your, you, got it, i, my, it, that, ok, okay, sure, yes, no). Because it matches whole words, "okra" and "thatch" are not frames;
- **span**, recorded with row (c): a bare reply whose words before the `.` occur as whole consecutive words in the prompt, with the prompt's punctuation read as spaces. A cut value such as "basalt lo" is not a span.

A bare reply holding the wrong value counts here, and not in `complete`. The frame list is closed, so a framed reply it does not list ("Her name is Basalt.") counts as bare: the REPORT quotes every bare reply word for word beside the count, and names any such reply. Re-read by the same file, claude-322 gives bare 0 and 0, span 0 and 0, and short 68 and 16. So bare above 0 is above claude-322.

**Recorded beside them, descriptive only:**
- **The first-line count,** by `firstline.jq` (`b15c778db57c2e65902e34987cf15748213128ab5d9bb72230f97f0ea58fb702`, the same file as claude-322's), with claude-322's matching rules.
- **The role-header count:** rows whose reply contains `Assistant:`, and `User:`, among EOS-stopped rows and over all 256.
- **The opening template count:** stopped replies that begin "Your friend's name is" or "Got it, your friend is", by `bare.jq`, beside claude-322's 73 and 28 (seed 1) and 1 and 36 (seed 2).
- the DEV curve, beside the earlier arms and claude-322's; the loss at the last step, within the arm only.

## Bars

Fixed before any run. They are claude-322's, unchanged.
- **Stopping:** at least **128 of 256** EOS stops at **both** seeds. (b64L-d3mwcap6): 109 and 86.
- **Collapse:** a seed is collapsed if, among its EOS-stopped rows, one reply text covers **at least 64**. (b64L-d3mwcap6): 9 and 6.
- **The panel's bar:** at least 104 rows complete and at least 52 pairs at both seeds. Every model scored so far has 0.
- **The held-out cost:** `behind` (b64L-D3) on the pooled verdict. (b64L-d3mwcap6): −15,456 and −25,627, behind.

**The cost ceiling, a decision rule, not a bar,** unchanged: a mean pooled d **below −58,212**. Here it reads row (f), against claude-322's grades.

## What each outcome unlocks

Read in table order: 0a, 0b, 0c, a, d, b, c, e. The first row that matches decides. Row (f) and the held-out row are read beside whichever of them matches.

| # | reading | the next unit |
|---|---|---|
| 0a | a seed NOT_RUN after its rerun, for an instrument fault (a gate, I-3's window count, out of memory, setup, a lost pod) | **`blocked`.** Repair the instrument and rerun the seed under this sheet. No reading on one seed. |
| 0b | a seed NOT_RUN after its rerun, for a mechanism cause (a non-finite loss at a recipe that trains on both parts alone) | **`diagnostic stop`,** recorded as a result about (b64L) on the mix. Next: a decision review in the REPORT. |
| 0c | I-c or I-xc fails, or an export or score fails on the laptop with both seals present | **`blocked`, no pod.** Repair the export or scorer and re-read from the seals. |
| a | complete ≥ 104 and pairs ≥ 52, stops ≥ 128, no collapse, and the held-out pair against (b64L-D3) not `behind`, at both seeds | **The panel bar is met on this panel.** Next: a fresh-panel confirm under its own sheet. |
| d | row (a) not met, and either a seed is collapsed (one stopped text covers 64 or more), or stops are below claude-322's at both seeds (under 109 at seed 1 and under 86 at seed 2) | **The mix disturbs stopping.** This is the mix, not the corpus alone: D3's passes and the capped copies' share also fall. The REPORT first classes it as instrument or mechanism (§4.2), from I-3, I-c and I-xc. Next: a decision review in the REPORT. |
| b | rows (a) and (d) not met, and complete above 0 at a seed | **The corpus lever works.** Next: the corpus's share (more rows, or repeats) under its own sheet. |
| c | complete 0 at both seeds, and bare above 0 at a seed (claude-322: 0 and 0) | **The bare form is learned; recall is the gap.** The REPORT gives span. Span above 0 means chat values are copied, but not the asked-for one. Span 0 means the values do not come whole from the chat. Next: the recall lever (writer admission, copy keys: claude-323's result). |
| e | complete 0 and bare 0 at both seeds | **No change from claude-322 in the reply form: the corpus share is too small to move it.** Next: a repeat factor under its own sheet. |
| f | the mean pooled d against claude-322's (b64L-d3mwcap6) below −58,212 | **The mix's held-out cost is named,** beside the row above. The REPORT says it cannot be split from D3's fewer passes (2.29 against 2.51). |

- **The held-out row is decided against (b64L-D3),** as in claude-322's sheet, and is recorded beside every row.
- **Seeds are read as each row is written:** "at a seed" means either seed, and "at both" means both. In order (d), (b), (c), (e), the rows cover every run that is not 0a to 0c or (a), so exactly one matches first. The REPORT gives both seeds' figures, and says "mixed" when the seeds differ.
- A seed that is NOT_RUN reruns once, sealing to the new root `runs-2/s{1,2}/` (L-002). A second NOT_RUN reads by row 0a or 0b, by the class of its last cause.

## Instrument checks

Registered before any pod goes up. They are claude-322's, with TRAIN swapped:
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin, with at least one test run, none failed, and claude-300's three micro-batch tests passing by name.
- **I-p, the pin:** the job refuses any SHA argument but the pin (exit 2), and a source that is not a clean checkout at the pin (exit 3).
- **I-3:**
  - the mix's three files match the shas above. The likeliest wrong file is claude-322's ×6 capped mix (tokens `0273173e…`, mask `da506e85…`, manifest `442ead2c…`): the gate tests feed it, and expect a refusal for each;
  - the panel folders and the DEV files match claude-306's pins;
  - each run's `config.json` names the mix's tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed and the recipe's flags, with no turn memory;
  - **the window count:** the trainer's "usable TRAIN windows" line in the smoke must read exactly **457,421** (D3's 153,253, six times the capped copy's 44,028, and the corpus's 40,000: `build_document_windows` in `hamilton_sheet.rs` gives a document one window per 192 targets, and keeps a window with at least one scored target; every corpus document is at most 188 tokens and has a scored reply). Any other count, or none, makes the arm NOT_RUN at that seed before the 16,384-step run (row 0a), an instrument finding to classify before any reading. claude-322's 417,421 is a refusal.
- **I-1, the smoke:** 128 steps at `--eval-every 32` on the mix, at the pod's seed. It exits 0 with a finite loss and writes `models/step-000064`.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. The mix is about 3.3M tokens larger than claude-322's, so claude-322's smoke peaks are the reference.
- **I-f, I-t, the watcher, I-5, I-4, I-c, I-x and I-xc:** as in claude-322's sheet, unchanged. The watcher keeps MAXSEC 14,400 s, a budget of PRICE × 4 h, PRICE at most $0.70/h, and the stall pause (D67).

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal, watch, seed and read scripts.
2. On the laptop:
   1. The job is claude-322's `job-322.sh`, with TRAIN swapped to this mix:
      - `check_train` checks this mix's three shas;
      - the folder fetch is `results/claude/one-model-end-to-end-currentstate-2026-11-08/train/`;
      - the window gate reads exactly 457,421 (see I-3);
      - every other line, `end()` and the traps included, is claude-322's.
   2. Its gate tests and mutations, `bash -n` on every script, I-c and I-xc.
   3. The Vast adapter `uor-r4-pod-provider`, built from fresh `main` into `local/pod-provider-target`.
3. `uor-pod status`, recorded. The cross-lab caps must hold with these two pods included: at most 4 pods and $8/h. Then, per seed, `seed-325.sh SEED`:
   - one RTX 5090 Vast pod, at most $0.70/h, brought up with `--count 1 --ref` at the pin and leased under session claude-325;
   - its watcher started in a detached `screen` (`c325-wS<SEED>`) before any script ships;
   - the job, chain and seal scripts shipped, their hashes checked on the pod against this sheet's, then the chain started;
   - avoiding Vast machines 48282 and 48276.
4. On each pod, in order: I-p, I-0, the build, the inputs and I-3; the smoke with I-m and I-f, then I-t; (b64L-d3mwcs) for 16,384 steps; the no-memory scoring of `step-016384`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-b64l-d3mwcs-2026-11-08/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop, `read-325.sh`:
   1. `ctrl`: I-xc and I-c again; then `serve`: export each seal's `models/step-016384`, I-x, the panel, the role headers and the first-line count.
   2. `grade`: `scale-probe grade` with `train=` `local/claude/claude-324/mix`, then the pairs against (b64L-D3) and claude-322's pinned grades.
   3. `dev`: the DEV curves.
   4. The bare-reply count and the opening template count, printed by `serve` from `bare.jq` for each seed's served output.

## Minimum test power

- **Stopping** is a count. claude-322's seeds read 109 and 86, a spread of 23. A seed within 32 of 128 (96 to 160) is reported as "near the bar". The row still follows the bar.
- **Collapse:** a reading within 16 of 64 (48 to 80) is "near the bar". The REPORT gives the collapse count beside the number of EOS-stopped rows it is over.
- **`complete`** is a count of 256. claude-322 read 0 and 0, so any complete row at a seed is above it. Row (b) needs only that; it does not claim a rate.
- **The bare-reply count** decides rows (c) and (e) only by whether it is above 0 at a seed, where claude-322 read 0 and 0. No row reads its size.
- **The held-out pair** uses `scale-probe pair`'s band. The scorer is not bit-reproducible across pod instances (claude-317's I-q); each seed is scored on its own pod, so that noise is inside the two-seed spread.
- **The cost ceiling** is compared on the mean of two seeds. A mean within one band of −58,212 is "near the ceiling". The row still follows the ceiling.

## Cost

- **Expected:** two RTX 5090s at about $0.563/h, about 2.3 h of wall each: 2 × 2.3 h × $0.563 ≈ $2.59, **about $3**. At the $0.70/h ceiling, 4.6 h × $0.70 = $3.22.
- **The caps:** the lab's claude-323 pods are running ($1.12/h). With these two, four pods at about $1.12 + 2 × $0.563 ≈ $2.25/h, inside 4 pods and $8/h.
- **The budget notice, not a cap:** 2 pods × 4 h × $0.70 = $5.60. A healthy run is never cancelled for a self-set budget. The ORDER's bound is $8.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m, I-f or I-3's window count fails:** the arm is NOT_RUN at that seed.
- **An out-of-memory error, a non-finite loss, or a trainer exit other than 0:** NOT_RUN at that seed, classified as instrument or mechanism before it is called anything (§4.2).
- **A pod lost mid-run, or one that never reaches SSH within 1,800 s:** an instrument NOT_RUN. It is deleted, and one retry runs on another machine.
- **The no-memory scoring fails or I-5 fails:** the served readings stand. The held-out row is then not read; the REPORT re-scores from the seal.
- **Always:** the job seals on every exit, including a TERM, INT or HUP, and the watcher takes the pod down after `seal=0`.

## Changes from claude-322's sheet

1. **TRAIN:** claude-324's mix, claude-322's ten roots then `currentstate`, at the store path and shas above.
2. **The window gate** is exactly 457,421, in place of 417,421.
3. **The exposure:** passes 2.29 (from 2.51), scored EOS per window 1.06 (from 1.07), EOS share 1.43% (from 1.333%), the corpus 8.7% of windows seen.
4. **The comparison arm** is claude-322's (b64L-d3mwcap6), with its pinned grades. The earlier descriptive pairs are dropped from the sheet's readings.
5. **The readings add** the bare-reply count, the opening template count, and the pair against claude-322's grades.
6. **The outcome map** is rewritten as rows (a) to (f) for the corpus question; rows 0a, 0b and 0c are kept.
7. **I-3's gate tests** take claude-322's ×6 mix as the refusal.
8. **The scripts** are claude-325's: claude-322's with renames, the new MIXR and its three shas in `job-325.sh`, the new seal root, and new hash pins in `seed-325.sh`. `read-325.sh` is claude-323's `read-323.sh` without its copy-key gate (I-k), grading with `train=` `local/claude/claude-324/mix` and pairing against claude-322's pinned grades.
9. **The pods** are brought up by `seed-325.sh` under lease session claude-325; the watcher's screen is `c325-wS<SEED>`.
10. **The seal path** is `results/claude/one-model-end-to-end-b64l-d3mwcs-2026-11-08/runs/s{1,2}/`, and `runs-2/` for a rerun.
11. **The cost** counts claude-323's two running pods against the caps.

**The script hashes** (sha256, in `local/claude/claude-325/`):
- `job-325.sh` `a9599810d926b1683d068b5110c0f24ec3c7e69ebbe2011023a5e603bbb45b21`
- `chain-325.sh` `c0a13147fae0053965f351c0eced1113bc385df3e4ab3c0baf45217c41f6c1a0`
- `seal-325.sh` `4cf1f375257539b004468e2ee9c93a330263b54dbd19797e8dccf58fbaa2632d`
- `watch-325.sh` `93ca699ff7bbfc8e65c91bdfaedb457cfc3ddee4eccc2b28504e5b16c27f14a6`
- `seed-325.sh` `6d69f16b3fd1a2f716d1398061f9a821669c2f93dedc98133c1445e9d7bc9a8c`
- `read-325.sh` `4f33533df07267ab5384914c0265477cc42ce232d95b454483120c2cc8d32038`
- `firstline.jq` `b15c778db57c2e65902e34987cf15748213128ab5d9bb72230f97f0ea58fb702` (the same file as claude-322's)
- `unwrap.jq` `4bb7ef1bc57bf39fbe15c5e98e9b414ca1d1c820f2a56067e0e51d783ebe9db5`
- `bare.jq` `6b143d056b9007b535427cae700c231666debcd79c9f8e5deb1a15ea4be5226a`

## FIT

[claude] Two fresh-context reviewers, neither the sheet's author, 8 Nov.

**Round 1, at `4400a4182`: FIT WITH CHANGES.** The arithmetic, the bars and the window count checked out.
- **MAJOR 1:** row (c) read "bare above claude-322", and claude-322 already read bare 2 at seed 2. The cause was prefix matching in the frame list, which let "ok" match "okra". So no change would have read as row (c). Fixed: `bare.jq` now matches whole words and counts only EOS-stopped replies. claude-322 re-reads bare 0 and 0, and the bare count is named as deciding rows (c) and (e).
- **MAJOR 2:** no row held collapse, and a loss of stopping could read as row (b). Fixed: row (d) carries collapse and stopping below claude-322 at both seeds, and is read before (b), (c) and (e).
- **MINOR 1–6:** overclaims fixed:
  - "the mix" in place of "the corpus" for stopping and held-out changes;
  - "the count of scored EOS is matched" in place of "exposure is held";
  - the window clause now cites `build_document_windows`;
  - row (b) reworded;
  - the mixed-seed rule now covers every run.

  All fixed at `587eca092`.

**Round 2, at `587eca092` with the rewritten `bare.jq`: FIT.**
- Both MAJORs hold.
- The rows after 0a–0c and (a) partition every run:
  - (d) covers any collapse, and stopping below claude-322 at both seeds;
  - (b) needs complete above 0;
  - (c) and (e) are complements on bare.
- 17 synthetic replies probed `bare.jq`. It handles "okra", "Okay.", "It's", "Iris.", leading space, newlines and a missing period as intended. Three MINORs were applied before any pod:
  - the `bare.jq` header named `span` as deciding (corrected, and re-pinned);
  - a closed frame list counts "Her name is Basalt." as bare, so the REPORT quotes every bare reply word for word;
  - "do not come whole from the chat", and "count" in place of "share".

  Re-read after the re-pin, claude-322 is unchanged: s1 stopped 109, short 68, bare 0, span 0, templates 73 and 28; s2 stopped 86, short 16, bare 0, span 0, templates 1 and 36.

## Pre-run review

[claude] D67, a fresh-context reviewer, 8 Nov: **GO, no findings.**
- **The diffs:** every hunk of the six scripts against claude-322's (`job`, `chain`, `seal`, `watch`, `seed`) and claude-323's (`read`) is an intended change: renames, MIXR and its three shas, the window gate 457,421, the seal root, the seed pins, the bare pin. The seal, watch, trap and price-refusal logic is byte-identical.
- **The pins:** each hash above equals `shasum -a 256` of its file. The local mix `local/claude/claude-324/mix` matches the three TRAIN shas.
- **The gate tests**, all passing:
  - `gt/test-gates.sh`: 56 ok, with claude-322's ×6 mix and the 417,421 count as refusals;
  - `gt/mutate.sh`: 18 of 18 mutants caught;
  - `gt/test-seal.sh`: 10 ok, with its seed-pin check shown to fail on a broken pin;
  - `gt/test-watch.sh`: 5 of 5 caught.

  `bash -n` passes on every script.
- **After the review,** only `bare.jq` (rewritten, then its header comment) and `read-325.sh`'s pin line changed. Round 2 of the FIT read both; `read-325.sh` greps `bare` only at its pin and its print line.
- **The whole-project chain:** Track A of "Both in parallel" under "pipeline first" (#2028). The inputs are in the store and on the laptop. The bars are unchanged, and each outcome row names its next unit. The lease and caps are checked at step 3.
