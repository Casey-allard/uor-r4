# REPORT: one-model-end-to-end, claude-319

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [one-model-end-to-end-b64l-d3mw3-2026-11-08.md](../../../../harness/prep/one-model-end-to-end-b64l-d3mw3-2026-11-08.md) (claude-318, #3846). Two Vast RTX 5090 pods at $0.563/h, $2.98 in all.

**Outcome row 4: stopping met, and both seeds collapsed.** Tripling mw-balp in the mix makes (b64L) end most replies on EOS: 234 and 204 of 256, past the bar of 128 at both seeds. But the stopped replies collapse onto one stock text at each seed, 88 and 101 times against the bar of 64. The sheet's next step is a decision review on a larger or more varied dialogue corpus (below).

## What did not work, first

- **Both seeds collapsed, further past the bar than × 1.**
  - Seed 1: 88 of its 234 EOS-stopped replies are " I'm good, thank you! How about you?". Its next two open the same way (25 and 14).
  - Seed 2: 101 of its 204 are " Got it, your friends, your friends,". Its next is the same text repeated (9).
  - The bar is 64. Both readings are outside the sheet's "near the bar" band (48 to 80). At × 1 the seeds read 62 and 75.
- **No row is complete and no pair counts: 0 of 256 and 0 of 128 at both seeds.** The panel's bar is 104 and 52. Every served model scored so far has 0.
- **The held-out cost is about seven times × 1's, and the trunk lost its lead over T2.**
  - Against claude-301's (b64L-D3), the pooled d is −155,433 and −110,880 rows (band 44,553), mean −133,157. claude-315's × 1 was −14,975 and −23,833.
  - The sheet's cost ceiling is −58,212. It only splits rows 3a and 3b, so it decides nothing here. It is recorded because the next lever has to beat it.
  - Against the mix's own T2, the pooled verdict is **inside** (d −52,111 and +2,994, band 55,105). At × 1 it was ahead by about 75,000 at both seeds. So at × 3 the model no longer beats a token-pair table on held-out human text.
  - This unit cannot split the cost between fewer D3 passes (2.93 against × 1's 4.73) and mw-balp's content. The sheet named that confound before the run.
- **DEV peaks early and falls.** Seed 1 peaks at 7,335 (step 7,168) and ends at 6,874. Seed 2 peaks at 8,874 (step 6,144) and ends at 8,219. At seed 2 the DEV openings settle on one pick (`opening_distinct_picks` 1, against 54 at seed 1), the same collapse seen on the panel.
- **Tokens per second is the bin timer's figure, not decode speed:** 282 and 446 tok/s. The timer re-parses the artifact on every row (claude-311). It is recorded, not compared.

## What changed

- **No code.** Two seeds trained on the pods, then the sheet's Step 6 on the laptop.
  - The scripts were the five the pre-run review pinned: `job-318.sh` `5dd570e6…`, `chain-318.sh` `5f591616…`, `seal-318.sh` `a49720dc…`, `watch-318.sh` `067eb369…` and `read-318.sh` `d393a6f7…`.
  - `seed-319.sh` started each seed's watcher in a detached `screen` before shipping the scripts. It then checked the three pod-side hashes equal to the pins (`ship-s{1,2}.txt`) and started the chain.
  - All of this is in `local/claude/claude-318/` and `local/claude/claude-319/` (not committed).
- **Pods:**
  - Seed 1 ran on `vast-59658703`, and seed 2 on `vast-59658738`. Each was leased under session `claude-318` and refused above $0.70/h.
  - Each watcher took its own pod down on `seal=0`: seed 1 at 06:01:34Z for $1.46, and seed 2 at 06:08:15Z for $1.52.
  - No TIME, SPEND, LOST or STALL notice fired. 0 of this unit's pods are left running.
- **Seals:** `results/claude/one-model-end-to-end-b64l-d3mw3-2026-11-08/runs/s{1,2}/`.
  - `fetch-319.sh` fetched each to the laptop. Each passed `shasum -c SHA256SUMS`, and its `tar -tzf` equals the sealed `.list`.
  - Neither seed needed a rerun.

## Measurements

**The served replies on `m2-fresh256`** (`served-reply-panel` at bin `be3d4cb0…`, greedy, 64-token cap, rendering `one-user-turn/v1`):

| Seed | Artifact sha256 (I-x) | Stops EOS / BOS / cap / context | Complete | Pairs | Most common EOS-stopped | Distinct (all) | Most common (all) | Median reply | Output sha256 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `13a22c9e9ce1125a52da7cf900397c81184f3d5952b9e07a96f1ba950a7ecdd0` | 234 / 3 / 19 / 0 | 0 | 0 | **88** | 90 | 88 | 11 | `b1a972fb034cda52c1156fbcb85db0c81daaf8647ef8e5ca21e5c072ba4adf2a` |
| 2 | `c7bb532a265244bd0cc07e9d6734ac7d47c0b83228e777ee88ca6e56ed18a5eb` | 204 / 0 / 52 / 0 | 0 | 0 | **101** | 85 | 101 | 12 | `6dd9e7cad099db82d4366f64ed77a61dde6a7f71f6e7fcdc5bb0cda2b81a0fab` |

EOS stops per stratum (length2, length4, length8, reassert, update): seed 1 61, 62, 56, 29, 26; seed 2 56, 60, 39, 28, 21.

**Role headers:** 0 replies at either seed contain "Assistant:" or "User:" (EOS-stopped or over all rows). At × 1 the counts were 35 and 51 over all rows. The end-of-turn fault claude-315 found is absent at × 3 on `m2-fresh256` (0 of 256 at both seeds).

**The stops across ratios** (EOS stops of 256, seed 1 / seed 2):

| TRAIN | EOS stops | Most common EOS-stopped | Distinct (all) | Pooled d against (b64L-D3) |
|---|---|---|---|---|
| D3 alone, claude-308/309 | 0 / 0 | — | — | — |
| D3 + mw-balp × 1, claude-315 | 177 / 122 | 62 / 75 | 68 / 39 | −14,975 / −23,833 |
| **D3 + mw-balp × 3, this unit** | **234 / 204** | **88 / 101** | **90 / 85** | **−155,433 / −110,880** |
| mw-balp alone, claude-313 | 253 / 244 | 73 / 40 | 120 / 104 | −304,197 / −307,764 |

So each step toward mw-balp buys stops, and pays for them in held-out text and in collapse. The × 3 mix has more distinct replies than × 1, but a larger single stock reply.

**The bars:**
- **Stopping** (at least 128 of 256 EOS stops at both seeds): **met**, 234 and 204.
- **Collapse** (at least 64 identical EOS-stopped replies at a seed): **both seeds collapsed** (88 and 101).
- **The panel** (at least 104 complete and 52 pairs at both seeds): **not met**, 0 and 0.
- **The held-out cost** (the pooled verdict against (b64L-D3)): **behind**.

**The no-memory panel** (`scale-probe grade`, `train=` the d3mw3 mix `3cf9ebe2…`/`a245fef4…`, no `clean=`). Each grade scores 1,460,697 rows with identity `5f88a5b3…` (I-4). Grade sha256: seed 1 `583bfc87…`, seed 2 `d1b7d0e2…`.

| Pair | Pooled verdict | Pooled d, seed 1 / seed 2 | Band | Output sha256 |
|---|---|---|---|---|
| (b64L-d3mw3) vs claude-301's (b64L-D3) | **behind** | −155,433 / −110,880 | 44,553 | `e106e24ba1ee7fe048eb1bcb3bdd6d21ae0575f357414c9c89625d50f012f0d2` |
| (b64L-d3mw3) vs claude-315's (b64L-d3mw) | behind | −140,458 / −87,047 | 53,411 | `3c5d6fb6872172be699be83043522c74d9341948fe25ee66ccbb82f654c977fe` |
| (b64L-d3mw3) vs claude-313's (b64L-mw) | ahead | +148,764 / +196,884 | 48,120 | `1d212f9a25c1f4cda5c5f9b60ddb3c8100542f87c176a7cdbc020975f800f599` |
| (b64L-d3mw3) vs the mix's T2 | **inside** | −52,111 / +2,994 | 55,105 | `f17b59bd4e8491958f5049df8d3887899535d74a39374e16388cf61fb5f63a24` |
| (b64L-d3mw3) vs the mix's unigram | ahead | +149,173 / +204,278 | 55,105 | `b9b231591dcc7f44988be3a6ffe14f04873a9d5fe75bdec6820fa367d2d19f4f` |

The model hits 176,265 and 231,370 rows; T2 hits 228,376.

**The DEV curve** (`progress.jsonl` `dev_nearest.hits`, every 1,024 steps):

| Step | 1,024 | 4,096 | 8,192 | 12,288 | 16,384 | Peak |
|---|---|---|---|---|---|---|
| (b64L-d3mw3) s1 | 5,820 | 7,074 | 7,218 | 7,054 | 6,874 | 7,335 at 7,168 |
| (b64L-d3mw3) s2 | 7,463 | 8,650 | 8,759 | 8,368 | 8,219 | 8,874 at 6,144 |
| (b64L-d3mw) s1, claude-315 | 6,533 | 6,877 | 8,194 | 8,242 | 8,210 | 8,429 at 11,264 |
| (b64L-mw) s1, claude-313 | 6,904 | 8,439 | 8,672 | 8,278 | 8,246 | 8,672 at 8,192 |

**The loss** at step 16,384 (`train_loss_mean`): 0.912 (seed 1) and 0.917 (seed 2). DEV loss 2.886 and 2.393. Training took 8,272 and 8,381 s.

**The smoke** (`local/claude/claude-319/smokes.txt`):
- I-3 read 357,907 usable windows at both seeds.
- I-1 passed at both.
- I-m passed at both (26,832 of 32,607 MiB).
- I-f passed at both (26,512→26,832 and 26,832→26,832 MiB).
- I-t recorded 0.503 and 0.509 s a step, with W 9,228 and 9,339 s.

## Checks run at the head

- **On-pod script hashes** before launch: all three of `job-318.sh`, `chain-318.sh` and `seal-318.sh` equal the pins on both pods.
- **The job's own checks, from the seals:**
  - `EXIT` is 0 and `DONE` is present at both seeds. `source.sha` is the pin `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f` at both.
  - I-0: `tests.out` reads 385 passed, 0 failed at both, with claude-300's three micro-batch tests passing by name.
  - `.i3` reads `ok` at both.
  - The no-memory `.i5` reads `same` on all five splits (oasst2, wiki-00000, wiki-00020, wiki-00040, dev) at both. `dev-agree` reads the same DEV sha at both.
  - `train.exit` and `score.exit` are 0 at both.
- **I-xc and I-c**, re-run on the laptop before any reading (`local/claude/claude-318/ctrl-run.txt`):
  - The re-export of claude-313's seed 1 hashes to `1c9ae43c…`.
  - The negative control has 0 EOS stops and the positive 253, each row-equal to its reference. I-c exits 0.
  - So row 0c does not apply.
- **I-x:** both artifacts' sha256 are recorded above. The scorer accepted both (exit 0).
- **The reader's pins:** `cpu-fbs-export` is `724a3b78…`, the scorer is `be3d4cb0…`, and the mix the grade read is `3cf9ebe2…`/`a245fef4…`. The reader refuses otherwise.
- **The collapse, distinct, role-header, median and stratum figures** are from `jq` over each output's `rows[]`.
- **`loop report --check`** on this REPORT.

## Decision review (row 4)

Row 4's next step is a decision review on a larger or more varied dialogue corpus. The claude lab takes that lever, read against the whole line so far:

1. **The ratio lever is spent.**
   - Every step toward mw-balp has bought stops: 0, 177/122, 234/204, 253/244.
   - It has also paid for them, in collapse and in held-out text. × 3 costs about seven times × 1's d, and loses the trunk's lead over T2.
   - A lower ratio (× 2) would sit between × 1 (seed 2 short, and collapsed) and × 3 (both collapsed). The lab judges it unlikely to clear both bars, so it is not taken.
2. **The collapse follows mw-balp's own register.** Whether the corpus or greedy decoding causes it is untested here: this run has one trunk, one decode and one corpus at different ratios. The next sheet tests it.
   - The stock replies are mw-balp's own register ("I'm good, thank you! How about you?", "Got it, …"). mw-balp's replies were seen 8.79 times each at × 3 (claude-318's sheet).
   - Collapse has now appeared at every ratio that stops: × 1 seed 2, × 3 both seeds, mw-balp alone seed 1.
   - So the next corpus should carry many more distinct replies per scored EOS. Repeating the same corpus cannot add them.
3. **The end-of-turn fault is absent at × 3 on `m2-fresh256`:** 0 role headers, against 35 and 51 at × 1. A new corpus should keep that, so its sheet keeps the role-header count beside the bars.
4. **Provenance comes first.** mw-balp's licence and generator are still recorded nowhere (claude-314's REPORT). A new dialogue corpus is registered only with its licence, its source and its generator recorded. It must also be disjoint from the oasst2 held-out panel split.
5. **The fine-tuning lever is a parallel track, not this one.**
   - deepseek-29 (#3847, merged) ran reply-masked SFT of (b64L) seed 1 to 2,048 steps: 52 of 256 free EOS stops at the selected step 1,280, and 0 complete.
   - That is below this unit's 234 and 204, at a far smaller dose.
   - It is DeepSeek's line, and it changes no bar or row here.

**Next:** a claude unit surveys the dialogue corpora the project can train on: licence, size, distinct replies, EOS targets, and disjointness from every panel. It registers the most varied one as a mix sheet against (b64L-D3), with the collapse bar (64) and the stopping bar (128) unchanged, this unit's × 3 numbers beside them, and the cost ceiling restated.

What a pass and a fail would each unlock:
- **Stopping met with no collapse on the new corpus:** row 2 or 3 of its sheet, and recall (the panel's complete bar) becomes the remaining gap.
- **Collapse again on a varied corpus:** the fault is in how the trunk decodes, not the corpus. Then the next step is a decode-time lever.

This is the claude lab's own decision review (§5.1), logged for Casey, who can reverse it.

## Skills used

- None beyond the loop's own commands.

## Still open

- **The dialogue-corpus survey and its sheet,** under their own unit, as above.
- **mw-balp's licence and generator** are still recorded nowhere (claude-314).
- **The scorer's timer** still measures re-parse plus decode (claude-311). A decode-only figure is still owed for the tokens-per-second lead metric.
- **claude-317's width slope (Track B)** is still running on its own pod. It is read in its own REPORT.
- **deepseek-31's sheet** (`one-model-end-to-end-content-ratio-2026-11-08.md`, on `main`) repeats M-World three times in reply-SFT. This unit's × 3 collapse (88 and 101) is the risk that lever carries. That is noted for DeepSeek, and it changes nothing in their sheet.

## Plain summary

We retrained the same small model on plain human text plus three copies of the dialogue corpus, at two seeds, on two rented GPUs for $2.98. It now ends most replies on its own (234 and 204 of 256), past the bar at both seeds, and no longer writes the next speaker's "Assistant:" header. But most of its stopped replies are one stock line ("I'm good, thank you! How about you?" 88 times; "Got it, your friends, your friends," 101 times), and it gave up much of its skill on held-out human text: it no longer beats a simple token-pair table. Repeating the dialogue corpus has done what it can. The next step is a larger, more varied dialogue corpus with its licence recorded.
