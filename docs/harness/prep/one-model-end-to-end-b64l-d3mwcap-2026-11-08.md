# Prep sheet: (b64L) on D3 + a reply-capped `mw-balp/train` repeated three times, the cap lever (one-model-end-to-end, claude-321, 8 Nov 2026)

Lab: claude. Item: one-model-end-to-end (#3407). Order: claude-321. The step is Track A of Casey's "Both in parallel" (#2028 6715318106), under "Yes, pipeline first" (#2028 6709947526): the run is judged by the served-reply score.

TEST FITNESS: FIT WITH CHANGES, round 1 (a fresh non-author review, 8 Nov). It found no blockers and recomputed every figure. Its three majors and five minors are applied:
- rows 2, 3a, 3b, 5 and 6a no longer credit the cap with what the window and exposure changes could also explain;
- row 3b names the × 2 option's lower exposure (about 0.67 EOS per window);
- the EOS-per-window figure is marked an upper bound;
- the mix census is said to attest the tokens and mask, not the manifest;
- `read-321.sh` checks the three base grades' hashes before it pairs;
- row 4's claim about the collapsed text rests on a census lookup.

**Why.**
- **(b64L) on D3 alone never stops.** claude-309 (#3828): 0 of 256 EOS stops at both seeds.
- **mw-balp's short replies are what made (b64L) stop.** On `mw-balp/train` alone, claude-313: 253 and 244 stops. On D3 + mw-balp × 1, claude-315: 177 and 122. On D3 + mw-balp × 3, claude-319 (#3851): **234 and 204**.
- **mw-balp's repeats are what the stopped replies collapsed onto.** claude-319's × 3 collapsed at both seeds: **88 and 101** of its EOS-stopped replies are one text, against the bar of 64. claude-320's census (#3851):
  - 72.4% of mw-balp's EOS targets close a reply seen more than once;
  - its top reply occurs 4,470 times in one copy, 13,410 times in claude-319's mix.
- **The × 3 mix also cost the most held-out text so far:** behind (b64L-D3) by −155,433 and −110,880 rows.
- **claude-320's decision review chose the cap lever:** keep every mw-balp text as context, but score each distinct reply at most k times. OASST2, the only more varied stored corpus, carries about 36 times fewer EOS per scored token, so it cannot replace mw-balp's stopping signal.

## Question

At (b64L)'s recipe and pin, with TRAIN changed to D3 + the capped `mw-balp/train` × 3:
- does the served model stop on EOS on `m2-fresh256` at **both** seeds?
- do its stopped replies stay varied, at both seeds?
- what does it cost on the held-out human text, against (b64L-D3), and beside claude-319's uncapped × 3?

**What the comparison can and cannot say.** The capped mix keeps claude-319's tokens byte for byte; only the mask differs. Its scored EOS count, 232,745, is within 0.4% of claude-315's × 1 mix (233,697). So:
- against claude-319's × 3, the cap is the only change in the file, but the trainer's window count changes too (below), so D3's passes and the EOS seen per window change with it;
- against claude-315's × 1, the EOS count is matched, but the stock replies are capped and the windows differ.

Neither comparison isolates the cap alone. The REPORT says so beside every reading.

## Base artifact

- **The pin:** `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`, as in claude-301, claude-306, claude-312, claude-314, claude-317 and claude-318. Every pod is brought up with `--ref` at the pin, and the job refuses any other source.
- **The recipe:** claude-301's (b64L), unchanged:
  `--objective nearest --retain-rates 1 --decay-mult 1 --lr 0.001 --lr-end 0 --train-as-served 1 --context 192 --width 256 --hamming-read 4 --batch 64 --micro-batch 16 --eval-every 1024 --steps 16384`, seeds 1 and 2.
  - Only `--train-split`, `--train-sha256` and `--train-mask-sha256` change.
- **The binary:** `train-in-space`, built on each pod at the pin, with its sha256 recorded.
- **The served artifact:** `cpu-fbs-export` (`724a3b78…`) on the laptop, from each seed's sealed `models/step-016384`, with `--raw-gates --raw-transition`.
- **The scorer:** `served-reply-panel` at bin `be3d4cb0…` (#3828), unchanged.

## Data split

**TRAIN `d3mwcap`,** in the store at `results/claude/one-model-end-to-end-b64l-d3mwcap-2026-11-08/train/`, put by claude-321 and never overwritten. The stored copy was fetched back and its three hashes matched (`shasum -c`, 8 Nov):
- `tokens.u16` `3cf9ebe2abdf6e6a252df545192af27aea8ec2ac419484aac8932158b5a57b67`: 50,506,504 tokens after the 64-byte header. **The same file as claude-318's `d3mw3`.**
- `response_mask.u8` `423d4f77d1cff168beda14d826ed1de9a2bc16010c961e17f1ef5c4c21232d32`;
- `manifest.json` `8f73bac12fd90fad08d0f726d3370598ea007347b27ea905147d89ff5d4fcf35`, schema `uor-r4-scale-probe-mix/v1`.

### The cap

`reply-cap` (claude-321, `docs/labs/one-model-end-to-end/reply-cap-2026-11-08/`, commit `a2d31821e0dccd8277d4c35f8047755098292730`) over claude-314's checked `mw-balp/train` (`3dee858f…` / `b5e7352d…`):
- a **reply** is a maximal run of mask-1 positions, as in `reply-census`;
- walking the file in order, the j-th occurrence of each distinct EOS-ending reply keeps its mask if j ≤ k; every later occurrence has its **whole run** set to 0;
- the tokens are copied byte for byte. Every reply is still in the context; only its scoring changes.

**The k table.** `reply-census` on each capped split (`k-table/k{1,2,3,4,6,8}.census.json`). mw-balp has 212,968 EOS-ending replies, 61,616 of them distinct, and every candidate keeps all 61,616:

| k | E(k): EOS kept | 3·E(k) | EOS in a repeated reply | top reply | scored targets |
|---:|---:|---:|---:|---:|---:|
| 1 | 61,616 | 184,848 | 0 | 1 | 899,280 |
| 2 | 64,407 | 193,221 | 5,582 (8.7%) | 2 | 929,120 |
| 3 | 65,979 | 197,937 | 7,154 (10.8%) | 3 | 944,949 |
| 4 | 67,128 | 201,384 | 8,303 (12.4%) | 4 | 956,385 |
| 6 | 68,999 | 206,997 | 10,174 (14.7%) | 6 | 974,908 |
| 8 | 70,672 | 212,016 | 11,847 (16.8%) | 8 | 991,367 |
| uncapped | 212,968 | 638,904 | 72.4% | 4,470 | 2,425,925 |

**The pick, by the ORDER's rule:** the smallest k in {1, 2, 3, 4, 6, 8} with 3·E(k) ≥ 212,968 (claude-315's × 1 mw-balp EOS count). **No k meets it.** k = 8 comes closest (212,016, 952 short, 0.45%), and the rule then takes **k = 8**. The capped split's mask is `445e8744faeb7cb8a58208775e078b2a808f728aa22d4c9bcedb9912ba758976`.

Most of mw-balp's EOS targets are repeats: capping at k = 8 keeps 70,672 of 212,968 (33.2%). Its scored targets fall from 2,425,925 to 991,367 (40.9%).

### The mix

**How it was built:** `scale-probe mix fraction=1/1` (bin `31610a00…`) over seven roots, in this order:
1. D3's four roots, whole, at D3's pinned shas (oasst2, wiki-00000, wiki-00020, wiki-00040, from claude-314's checked copies);
2. then the capped split, whole, three times, under the root names `cap1`, `cap2` and `cap3`.

Each root's two shas are checked by the tool before it reads the root. The build log is `k-table/d3mwcap-mix.log`.

**Checks on the laptop, 8 Nov:**
- the mix's `tokens.u16` is byte-identical to claude-318's `d3mw3` (`3cf9ebe2…`): the cap moves no token;
- the mix's first 27,968,113 mask bytes are identical to `d3mw3`'s, so D3's part is unchanged;
- each of the three later 7,512,797-byte mask segments hashes to the capped split's mask `445e8744…`;
- `reply-census` on the mix (`k-table/d3mwcap.census.json`) reads 30,390,655 scored targets = 27,416,554 + 3 × 991,367, and 232,745 EOS = 20,729 + 3 × 70,672. It was run on a view folder (the mix's tokens and mask, with a manifest carrying `eos_id`), so it attests the tokens and mask but not the manifest; the manifest `8f73bac1…` is gated by `TR_MAN` in the job.

| part | documents | tokens | scored targets | scored EOS | EOS per million scored | top reply | usable windows |
|---|---:|---:|---:|---:|---:|---:|---:|
| D3 | 18,016 | 27,968,113 | 27,416,554 | 20,729 | 756 | n/a | 153,253 |
| capped `mw-balp/train`, each copy | 60,000 | 7,512,797 | 991,367 | 70,672 | 71,287 | 8 | 44,028 |
| **the mix** | **198,016** | **50,506,504** | **30,390,655** | **232,745** | **7,658** | **26** | **285,337** |
| claude-318's `d3mw3` (uncapped × 3) | 198,016 | 50,506,504 | 34,694,329 | 659,633 | 19,013 | 13,410 | 357,907 |
| claude-315's (b64L-d3mw) mix (× 1) | 78,016 | 35,480,910 | 29,842,479 | 233,697 | 7,831 | n/a | 221,471 |

- The mix's documents, tokens and scored targets are also `scale-probe mix`'s own printed totals.
- The EOS counts are positions with token 1 and mask 1. The mix's top reply, 26, is 3 copies × 8 plus D3's own occurrences.
- **The window counts are measured, not summed.** `window-count` (claude-321, beside `reply-cap`) restates the trainer's rule:
  - `build_document_windows` (`crates/uor-r4-float-train/src/hamilton_sheet.rs`) cuts each BOS-delimited document's targets into chunks of at most `context`;
  - `train_windows` (`in_space_long.rs`) keeps a chunk with at least 4 scored targets.
  - It reads the trainer's own logged counts on `d3mw3` (357,907) and on `mw-balp/train` (68,218). On the capped copy it reads 44,028: the cap takes 24,190 of mw-balp's 68,218 usable windows below 4 scored targets, and the trainer drops them. The chunks with any scored target fall from 68,641 to 44,681.
  - I-3 checks this run's **285,337**.

**mw-balp's provenance** is as claude-314 recorded: the two shas, BOS 0, the store path and the tar `data/ft-balp.tar` (`b1d22113…`). Its licence and its generator are **not recorded**. This is open and is not decided here.

**The other inputs, unchanged from claude-318's sheet:**
- **DEV in-run evaluation:** claude-261's `ft/dev`, at the pins claude-306's job checks.
- **The no-memory panel:** claude-295's four held-out folders, at claude-306's per-folder pins: 1,460,697 rows, identity `5f88a5b3…`.
- **The served panel:** `m2-fresh256`, inputs `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, tokenizer `d36d3e87…`, checked by the bin against its manifest.

## Repetition and exposure

The trainer samples windows uniformly by epoch (`epoch_order`). So, over 16,384 × 64 = 1,048,576 windows seen:
- **D3's passes:** about **3.67**, against 2.93 for claude-319's × 3, 4.73 for (b64L-d3mw) and 6.84 for (b64L-D3). The cap drops windows, so D3 is a larger share (53.7%) than in × 3 (42.8%). **This is a second change beside the cap, and it moves the held-out reading the other way:** more D3 passes should cost less held-out text, whatever the cap does.
- **The capped copies' share:** 46.3% of the windows seen, but only 9.8% of the scored targets. In × 3 the figures were 57.2% and 21.0%.
- **EOS targets seen:** about **0.82** scored EOS per window seen, about 855,000 over the run. This is an upper bound: it divides all EOS by the usable windows, and the EOS in the 653 chunks per capped copy that the trainer drops (under 4 scored) are never seen. The overstatement is under 1%. That compares with:
  - 1.84 for claude-319's × 3 (about 1.93 million), which stopped 234 and 204;
  - 1.06 for (b64L-d3mw) (about 1.11 million), which stopped 177 and 122;
  - 0.135 for (b64L-D3), which never stopped.
- **Repeated stock replies:** a kept reply occurrence is scored about 11.0 times over the run (3 copies × 3.67). The mix's top reply, 26 occurrences, is scored about 96 times, against about 39,300 for × 3's top reply.

**So stopping is the likelier failure here, not collapse.** The run sees fewer EOS targets per window than (b64L-d3mw), whose seed 2 missed the stopping bar. The ORDER's k rule matched the EOS *count* to × 1, not the EOS per window seen. This is recorded before any run. It is the reason row 6a names the next lever already counted (below).

**The counted next lever, if stopping is lost.** The same build with the capped split **six** times (`cap1` to `cap6`) was counted on the laptop on 8 Nov and then deleted:
- tokens `0273173e16e805706a9b3e816c3de46b824a53daf91be3b812be360f8e8e75a7`, mask `da506e850145f55b746929e108e0fd240ddf753f6a90b23976c98e159a1cb293`, manifest `442ead2c92ae9a2e6b0b500ebbc12eb6e472f91c96465b6c1347fa80ac232ae9`;
- 73,044,895 tokens, 378,016 documents, 33,364,756 scored, 444,761 EOS, top reply 50;
- 417,421 usable windows: **1.07 EOS per window seen**, (b64L-d3mw)'s density, and D3 passes 2.51.

It is not trained here. `scale-probe mix` rebuilds it byte for byte from the same roots.

## Arms

| arm | TRAIN | batch (micro) | steps | source |
|---|---|---|---:|---|
| (b64L-D3), sealed | D3 | 64 (4 × 16) | 16,384 | claude-301 `runs-2/s{1,2}`; served scores claude-309 |
| (b64L-d3mw), sealed | D3 + mw-balp × 1 | 64 (4 × 16) | 16,384 | claude-315 `runs/s{1,2}`; served scores and grades claude-315 |
| (b64L-d3mw3), sealed | D3 + mw-balp × 3 | 64 (4 × 16) | 16,384 | claude-319 `runs/s{1,2}`; served scores and grades in `local/claude/claude-318/` |
| (b64L-mw), sealed | `mw-balp/train` | 64 (4 × 16) | 16,384 | claude-313 `runs/s{1,2}`; served scores and grades claude-313 |
| **(b64L-d3mwcap)** | the mix | 64 (4 × 16) | 16,384 | this sheet's run unit, seeds 1 and 2 |

One arm to train, at two seeds, one pod per seed. The steps are fixed: no gate cuts them.

**claude-319's × 3, the cap's reference**, on the same measures:

| seed | EOS stops | collapse (EOS-stopped) | complete | pairs | held-out d against (b64L-D3) |
|---:|---:|---:|---:|---:|---:|
| 1 | 234 | 88 | 0 | 0 | −155,433 (behind) |
| 2 | 204 | 101 | 0 | 0 | −110,880 (behind) |

## Readings

At seeds 1 and 2, from `served-reply-panel` on (b64L-d3mwcap)'s exported `step-016384`, on the same measures as claude-318's sheet:
- **Stops:** EOS, BOS, cap and context counts, of 256.
- **The panel's own measures:** rows complete (of 256) and pairs both complete and distinct (of 128).
- **Collapse:** among the EOS-stopped rows only, the count of the most common reply text. Also recorded: distinct reply texts over all rows, and the most common reply over all rows.
- **Reply length:** the median of `rows[].reply_ids` lengths.
- **EOS stops per stratum** (length2, length4, length8, reassert, update).

**The held-out pairs:**
- `scale-probe grade` with `train=` the mix, on all 1,460,697 rows, with no `clean=`.
- **Deciding:** `scale-probe pair`, (b64L-d3mwcap) against claude-301's sealed (b64L-D3) grades `e17a13548d09304d901a93683cadb6c7695200261e86770e319003bfc976523a` and `3170c779def7ef376fdc11c96232f3f6ddb2809c436855121fd24be8ed7ab5a1`.
  - Each seed's position is `behind`, `inside` or `ahead` at the pair's band, max(⌈2·√(b+c)⌉, the two-seed spread).
  - The tool's single pooled verdict over both seeds is that position when the seeds agree, and `mixed` when they do not (`assess` in `docs/labs/scale-probe/instrument/src/lib.rs`).
  - **"Behind" in the outcome map** means the pooled verdict `behind`, or `mixed` with at least one seed `behind`. Every other verdict is "not behind". Both seeds' positions are recorded.
- **Deciding between rows 3a and 3b:** the **mean pooled d** over the two seeds of that same pair, against the cost ceiling below.
- **Descriptive:**
  - the pair against claude-319's uncapped × 3 grades, `583bfc8734d5761f40a7305816c2031fc3f3faf7242fa30231284bfb2ad40d2e` and `d1b7d0e271cc267790a4ce04106e3bfafce5915f7e2e07b2343ce34f0be56c1c`. Same tokens, capped mask; it also carries D3's passes rising from 2.93 to 3.67;
  - the pair against claude-315's (b64L-d3mw) grades, `e4714275acf1fe01063b04e63aff73436614a586a6bb8e5cfa9b55bda45bad85` and `b9160b63e4f3b03892245a3b6027098c5b5eb8a9696915b4e2548fbae158d2c2`. Matched EOS count; D3's passes fall from 4.73 to 3.67.
- As in claude-318's sheet, `train=` builds only the T2 and unigram baselines. It does not change which rows are graded or what counts as a nearest hit; I-4 pins the row set.

Recorded beside them, descriptive only:
- **The role-header count:** rows whose reply text contains `Assistant:`, and `User:`, among EOS-stopped rows and over all 256.
- the pairs against claude-313's (b64L-mw) grades (`395ea543…`, `aa725ed9…`), and against the mix's own T2 and unigram;
- the DEV curve (`dev_nearest.hits` every 1,024 steps), against (b64L-D3), (b64L-mw), (b64L-d3mw) and (b64L-d3mw3);
- the loss at the last step, within the arm only.

## Bars

Fixed before any run. They are claude-312's, unchanged, with the earlier arms' figures beside them.
- **Stopping:** at least **128 of 256** EOS stops at **both** seeds. (b64L-D3): 0 and 0; (b64L-d3mw): 177 and 122; (b64L-d3mw3): 234 and 204; (b64L-mw): 253 and 244.
- **Collapse:** a seed is collapsed if, among its EOS-stopped rows, one reply text covers **at least 64**. (b64L-d3mw): 62 and 75 (collapsed); (b64L-d3mw3): 88 and 101 (both collapsed); (b64L-mw): 73 (collapsed) and 40.
- **The panel's bar:** at least 104 rows complete and at least 52 pairs at both seeds. Every model scored so far has 0.
- **The held-out cost:** `behind` (b64L-D3) on the pooled verdict above. (b64L-d3mw) was behind by −14,975 and −23,833; (b64L-d3mw3) by −155,433 and −110,880.

**The cost ceiling, a decision rule, not a bar,** unchanged from claude-318's sheet: a mean pooled d against (b64L-D3) **below −58,212**, three times claude-315's mean (−19,404). It only chooses between rows 3a and 3b. claude-319's × 3 was past it (mean −133,157).

## What each outcome unlocks

Read top to bottom; the first row that matches decides.

| # | reading | the next unit |
|---|---|---|
| 0a | a seed NOT_RUN after its rerun, for an instrument fault (a gate, out of memory, setup, a lost pod) | **`blocked`.** Repair the instrument and rerun the seed under this sheet. No reading on one seed. |
| 0b | a seed NOT_RUN after its rerun, for a mechanism cause (a non-finite loss at a recipe that trains on both parts alone) | **`diagnostic stop`,** recorded as a result about (b64L) on the mix. Next: a decision review in the REPORT. |
| 0c | I-c or I-xc fails, or an export or score fails on the laptop with both seals present | **`blocked`, no pod.** Repair the export or scorer and re-read from the seals. |
| 1 | the panel's bar met at both seeds | **The served model answers.** Collapse and the pairs are still recorded. Next: the trunk trains on the mix under its own sheet, at the next scale. |
| 2 | stopping met, neither seed collapsed, and the pair is not `behind` | **This TRAIN stops, varies and keeps D3's held-out text.** Not the cap's alone: the windows and D3's passes changed with it. Recall is the remaining gap. Next: a decision review in the REPORT on the recall lever (the panel's complete bar). |
| 3a | stopping met, neither seed collapsed, the pair is `behind`, and the mean d is at or above −58,212 | **This TRAIN stops and varies, at a cost inside the ceiling.** Next: a decision review on the recall lever, with this TRAIN as the base. The REPORT says the cost cannot be split between D3's passes (3.67) and mw-balp's content. |
| 3b | stopping met, neither seed collapsed, the pair is `behind`, and the mean d is below −58,212 | **This TRAIN stops and varies, at a cost past the ceiling.** Next: a decision review between fewer capped copies (× 2) and a decode-time lever. × 2 would see about 0.67 EOS per window (162,073 over 241,309 windows), so it risks losing stopping. |
| 4 | stopping met, a seed collapsed | **Capping at k = 8 does not stop the collapse.** The REPORT looks the collapsed text up in `k-table/k8.census.json`'s top list and in D3, so whether it is one of mw-balp's repeated replies rests on that lookup. Next: a decision review between k = 1 (`k-table/k1`, every reply scored once) and a decode-time lever. |
| 5 | stopping not met, and at least one seed below 122 EOS stops ((b64L-d3mw)'s lower seed) | **Fewer stops than × 1.** The EOS count is matched, but the EOS seen per window is not (0.82 against 1.06), so the cap and the exposure cannot be separated. First the REPORT classes it as instrument or mechanism (§4.2), from I-3, I-c and I-xc. Then a decision review between the counted × 6 capped mix (1.07 EOS per window) and a decode-time lever. |
| 6a | stopping not met, both seeds at 122 or more, and neither seed collapsed | **Neither seed collapsed, but this TRAIN's EOS exposure (0.82 per window) does not lift both seeds past the bar.** The REPORT gives the collapse counts beside the number of EOS-stopped rows they are over, and does not credit the cap: a seed with 122 to 127 stops has few rows to collapse over, and the exposure changed too. The REPORT sets the stops at × 0, × 1, × 3, × 3 capped and mw-balp alone side by side. Next: the counted × 6 capped mix under its own sheet. |
| 6b | stopping not met, both seeds at 122 or more, and a seed collapsed | **The cap neither lifts both seeds nor stops the collapse.** Next: row 4's decision review. More copies are not offered, since they repeat the capped replies too. |

A seed that is NOT_RUN reruns once, sealing to the new root `runs-2/s{1,2}/` (L-002). A second NOT_RUN reads by row 0a or 0b, by the class of its last cause.

**deepseek-34** (#3854, merged) found that the first reply token is never in the top 32 on DeepSeek's M-World content-ratio checkpoint: a different trunk, corpus and panel. It changes no bar and no row here; the REPORT reads the first-token measure only if the scorer already records it.

## Instrument checks

Registered before any pod goes up. They are claude-318's, with TRAIN swapped:
- **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib` passes on each pod at the pin, with at least one test run, none failed, and claude-300's three micro-batch tests passing by name.
- **I-p, the pin:** the job refuses any SHA argument but the pin (exit 2), and a source that is not a clean checkout at the pin (exit 3).
- **I-3:**
  - the mix's three files match the shas above. The mix shares its tokens with claude-318's `d3mw3`, so **the mask and the manifest are what tell the two apart**: the gate tests feed it `d3mw3`'s mask and manifest and expect a refusal;
  - the panel folders and the DEV files match claude-306's pins;
  - each run's `config.json` names the mix's tokens and mask shas, width 256, read 4, batch 64, micro-batch 16, 16,384 steps (128 for the smoke), the pod's seed and the recipe's flags, with no turn memory;
  - the trainer's "usable TRAIN windows" line in the smoke reads **285,337**. Any other count, or none, makes the arm NOT_RUN at that seed before the 16,384-step run: the job refuses it, and it reads as row 0a.
- **I-1, the smoke:** 128 steps at `--eval-every 32` on the mix, at the pod's seed. It exits 0 with a finite loss and writes `models/step-000064`.
- **I-m, from the smoke:** the largest `mem.log` sample inside the smoke's window is below 90% of the card. The mix has the same 50.5M tokens as claude-319's, so claude-319's smoke peaks are the reference.
- **I-f, from the smoke:** the peak after the step-64 row is at most the peak before it plus 1,024 MiB.
- **I-t, recorded only:** r = (train_seconds at step 128 − at step 64) / 64; W = used + 1.05 × r × 16,384 + 300 s, written to `it.txt`. claude-315 read 0.559 and 0.571 s a step.
- **The watcher:** MAXSEC 14,400 s from the pod's start, a budget of PRICE × 4 h, and PRICE at most $0.70/h. Neither limit stops anything (D67): past either, the watcher logs one NOTICE and keeps watching and renewing the lease, so `seal=0` still takes the pod down.
  - **A stall pauses the lease.** When the pod's line is unchanged for 30 checks, the watcher logs a STALL NOTICE and stops renewing, so a hung pod's bill is bounded. A changed line clears the notice and renewals resume.
- **I-5:** the no-memory scoring's `.i5` reads `same` on every sub-panel, and DEV's rows agree across the four per-panel processes.
- **I-4:** each graded checkpoint scores the 1,460,697 rows with identity `5f88a5b3…`.
- **I-c, the scorer controls, on the laptop before any (b64L-d3mwcap) reading,** with `served-reply-panel` at `be3d4cb0…`. Each control's `rows` must equal the reference output's, field by field:
  - **Negative:** claude-308's seed-1 (b64L-D3) artifact (`9a370566…`) against claude-309's output `d02ec432…`: 0 EOS stops.
  - **Positive:** claude-313's seed-1 (b64L-mw) artifact (`1c9ae43c…`) against its output `5175c29b…`: 253 EOS stops.
- **I-x, the export:** each artifact's sha256 is recorded, and `served-reply-panel` accepts it.
- **I-xc, the export control, on the laptop before any (b64L-d3mwcap) reading:** `cpu-fbs-export` (`724a3b78…`, `--raw-gates --raw-transition`) re-exports claude-313's sealed seed-1 `step-016384`. The artifact's sha256 must equal `1c9ae43c…`.

## Steps

1. The ORDER; this sheet's FIT; the D67 pre-run review of the job, chain, seal, watch and seed scripts.
2. On the laptop:
   1. The job is claude-318's `job-318.sh`, with TRAIN swapped to this mix:
      - `check_train` checks this mix's three shas;
      - the folder fetch is `results/claude/one-model-end-to-end-b64l-d3mwcap-2026-11-08/train/`;
      - the window gate reads 285,337;
      - every other line, `end()` and the traps included, is claude-318's, as reviewed there.
   2. Its gate tests and mutations, `bash -n` on every script, I-c and I-xc.
3. `uor-pod status`, recorded. The cross-lab caps must hold with these two pods included: at most 4 pods and $8/h. Then, per seed, `seed-321.sh SEED`:
   - one RTX 5090 Vast pod, at most $0.70/h, brought up with `--count 1 --ref` at the pin and leased under session claude-321;
   - its watcher started in a detached `screen` before any script ships;
   - the job, chain and seal scripts shipped, their hashes checked on the pod against this sheet's, then the chain started;
   - avoiding Vast machines 48282 and 48276.
4. On each pod, in order:
   1. I-p, I-0, the build, the inputs and I-3.
   2. The smoke with I-m and I-f, then I-t.
   3. (b64L-d3mwcap) for 16,384 steps.
   4. The no-memory scoring of `step-016384`, one panel per process, then I-5.
5. Seal each pod to `results/claude/one-model-end-to-end-b64l-d3mwcap-2026-11-08/runs/s{1,2}/`. The root must be empty first (L-002). Read the seal back, then take the pod down.
6. On the laptop, `read-321.sh`:
   1. `ctrl`: I-xc and I-c again; then `serve`: export each seal's `models/step-016384`, I-x, and the panel.
   2. `grade`: `scale-probe grade` with `train=` the mix, then the pairs.
   3. `dev`: the DEV curves.

## Minimum test power

- **Stopping** is a count. At × 1 the seeds read 177 and 122, a spread of 55; at × 3, 234 and 204, a spread of 30.
  - The EOS seen per window, 0.82, is below × 1's 1.06. That stopping falls with it is an **inference** from three points (0.135: 0 and 0; 1.06: 177 and 122; 1.84: 234 and 204), not a measured rule, and the cap also changes which replies carry the EOS.
  - A seed within 32 of 128 (96 to 160) is reported as "near the bar". The row still follows the bar.
- **Collapse** is a count over EOS-stopped rows. The seed spreads so far are 33 (b64L-mw), 13 (b64L-d3mw) and 13 (b64L-d3mw3) about the bar of 64. A reading within 16 of 64 (48 to 80) is reported as "near the bar". The row still follows the bar.
  - A model that stops rarely has few EOS-stopped rows, and a collapse count over few rows is small by construction. So the REPORT gives the collapse count beside the number of EOS-stopped rows it is over.
- **The held-out pair** uses `scale-probe pair`'s band. claude-315's was 8,858, its two-seed spread.
  - **The scorer is not bit-reproducible across pod instances.** claude-317's I-q found 33 of 14,287 DEV nearest picks differ between two pods scoring the same checkpoint, a DEV hit difference of 3 of 3,188. Each seed here is scored on its own pod, so that noise is inside the two-seed spread.
- **The cost ceiling** is compared on the mean of two seeds. A mean within one band of −58,212 is reported as "near the ceiling". The row still follows the ceiling.

## Cost

- **claude-319's measured runs:** $2.98 for both pods.
- **Here:** the step time does not depend on TRAIN, since each step reads 64 windows padded to 192. So about $3 for both, and about $4.1 at the $0.70/h ceiling.
- **The budget notice, not a cap:** 2 pods × 4 h × $0.70 = $5.60. A healthy run is never cancelled for a self-set budget. If a slow card bills past $5.60, the REPORT says by how much. The ORDER's bound is $8.

## Stop rule

- **I-p, I-0, I-3 or an input fetch fails:** the pod trains nothing. Fix the instrument and rerun at a new root.
- **The smoke, I-m, I-f or I-3's window count fails:** the arm is NOT_RUN at that seed.
- **An out-of-memory error, a non-finite loss, or a trainer exit other than 0:** NOT_RUN at that seed, classified as instrument or mechanism before it is called anything (§4.2).
- **A pod lost mid-run, or one that never reaches SSH within 1,800 s:** an instrument NOT_RUN. It is deleted, and one retry runs on another machine.
- **The no-memory scoring fails or I-5 fails:** the served readings stand. Rows 2, 3a and 3b then cannot be told apart; the REPORT reads row "2 or 3" and re-scores from the seal.
- **Always:** the job seals on every exit, including a TERM, INT or HUP, and the watcher takes the pod down after `seal=0`.

## Changes from claude-318's sheet

1. **TRAIN:** D3 + the k = 8 capped `mw-balp/train` × 3, at the store path and shas above, in place of the uncapped × 3. The tokens are the same file.
2. **The k table and the pick,** by the ORDER's rule (no k met it; k = 8).
3. **The window count is measured** by `window-count`, checked against the trainer's own logs, and gates the run at 285,337.
4. **Exposure:** the capped mix sees fewer EOS per window than × 1, and more D3 passes than × 3. Both are named before the run, with the counted × 6 capped mix as row 6a's next lever.
5. **The descriptive pairs** add claude-319's × 3. **The outcome map** rows 3b, 4, 5, 6a and 6b name the cap's own next levers.
6. **I-3's gate tests** add `d3mw3`'s mask and manifest as refusals, since the two mixes share their tokens.
7. **The pods** are brought up by `seed-321.sh`, one per seed, under lease session claude-321.
8. **The seal path** is `results/claude/one-model-end-to-end-b64l-d3mwcap-2026-11-08/runs/s{1,2}/`, and `runs-2/` for a rerun.

## Pre-run review

**The D67 pre-run review**, one round, a fresh-context reviewer (claude, `claude-opus-5-5`), on the scripts in `local/claude/claude-321/` against their reviewed parents (claude-318's job, chain, seal, watch and read scripts, and claude-319's seed script).

**Round 1: GO.** No major.
- **Every diff hunk is intended:** the names, the TRAIN store path, mask and manifest (the tokens are unchanged), the window gate 285,337 at both sites, the lease session claude-321, the seal root, and read-321's added pair and DEV line. `end()`, the traps, the watcher's stall, renew and down logic, and the seed script's $0.70/h refusal are untouched.
- **The pins:** the seed script's three hashes equal the shipped scripts'.
- **The store:** `…/runs/` and `…/runs-2/` do not exist (L-002); `…/train/` holds the three files.
- **Minor 1, applied:** a comment in `job-321.sh` still summed the window count to 357,907. It now reads 3 × 44,028 + 153,253 = 285,337, and the job's hash and the seed script's pin moved with it.
- **Minor 2, kept:** the one-off miss of the `trap -` mutation is a race in the test, not a gap in the gate: `end()` and the traps are byte-identical to claude-318's, and `mutate.sh`'s `no-signal-trap` also covers the trap.

**The gates at the final hashes, 8 Nov:**
- `gt/test-gates.sh` 47 ok;
- `gt/mutate.sh` 17 of 17 caught, including the uncapped mix's mask and manifest;
- `gt/test-seal.sh` 7 ok;
- `gt/test-watch.sh` 8 ok and 5 of 5 mutations caught;
- `bash -n` ok on every script;
- I-c: rows equal (0 and 253 EOS stops); I-xc reproduced `1c9ae43c…`.

**The script hashes** (sha256):
- `job-321.sh` `d37527274765a1077a01bad46c409a3d09071d6717ffa38b2cbd07bf5f723a09`
- `chain-321.sh` `797ad522e507e607d704033085ac6408c1be48b1927c418559ba45ef21f50823`
- `seal-321.sh` `cc8121ba1c9062e4de16986e8262688fbf2fe0af8d7f4abc72b944b8666343a0`
- `watch-321.sh` `7b0603fa4458f7c5b5826b3a5133fcd82b6b775f249cf64004c40ab7e872af5d`
- `seed-321.sh` `2b981359e635152a4de9c9bd7ee8ccb5eaca684a0ecc05af49753862dab17604`
- `read-321.sh` `c931d9c3703e891cbb9fc59be5f4152b98deadec12172334a5012fd23ab754d9`
