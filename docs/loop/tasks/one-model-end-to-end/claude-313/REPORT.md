# REPORT: one-model-end-to-end, claude-313

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Sheet: [one-model-end-to-end-b64l-mwbalp-2026-11-07.md](../../../../harness/prep/one-model-end-to-end-b64l-mwbalp-2026-11-07.md) (claude-312, #3831). Two Vast RTX 5090 pods at $0.563/h, $2.43 in all.

**Outcome row 3: stopping met, a seed collapsed.** The corpus makes (b64L) stop: 253 and 244 of 256 replies stop on EOS, against 0 and 0 on D3. Seed 1's stopped replies collapse past the bar: 73 of them are the same text, against the bar of 64.

## What did not work, first

- **Seed 1 collapsed.** Among its 253 EOS-stopped replies, 73 are " Got it, I'll remember that." and 23 more are " Got it, I'll remember them." The bar is 64. Seed 2's most common EOS-stopped reply occurs 40 times, under the bar.
- **No row is complete and no pair counts: 0 of 256 and 0 of 128 at both seeds.** The panel's bar is 104 and 52. The replies stop but do not answer, as claude-259's (b) and (h) did (claude-311).
- **The swap costs the held-out human text heavily.** On the no-memory panel, (b64L-mw) is **behind** claude-301's (b64L-D3) on every split at both seeds: pooled over the four held-out splits (oasst2 and three wiki shards), d = −304,197 and −307,764 rows. Against mw-balp's own tables it is **inside** T2 and **ahead** of unigram. That is consistent with mw-balp alone giving up what D3 bought. This unit does not separate corpus content from corpus size: mw-balp has 7.5M tokens, D3 28.0M.
- **Seed 1's DEV curve peaks and falls.** Its `dev_nearest.hits` peak is 8,672 at step 8,192, and it ends at 8,246. That reads as over-fitting at the 10.6–17.5 passes the sheet predicted. Seed 2 still rises at the end (8,838).
- **Tokens per second is the bin timer's figure, not decode speed:** 248 and 328 tok/s. The timer re-parses the artifact on every row (`served-reply-panel.rs:204`, claude-311). It matches claude-309's scope, not a decode measurement.

## What changed

- **No code.** Two seeds trained on the pods, then the sheet's Step 6 on the laptop. The scripts were the four the pre-run review pinned: `job-312.sh` `fcbaae10…`, `chain-312.sh` `9fcef8ef…`, `seal-312.sh` `3653c9bf…` and `watch-312.sh` `8d77c59c…`. On-pod hashes were checked equal before launch. The laptop reader `read-313.sh` is in `local/claude/claude-313/` (not committed).
- **Pods:** `vast-59617111` (seed 1) and `vast-59617166` (seed 2). Both were leased as `claude/claude-312`, the session `watch-312.sh` renews. Each watcher took its own pod down on `seal=0`: seed 2 at 22:06:14Z for $1.20, and seed 1 at 22:08:14Z for $1.23. No TIME or SPEND NOTICE fired, and 0 pods were left running.
- **Seals:** `results/claude/one-model-end-to-end-b64l-mwbalp-2026-11-07/runs/s{1,2}/`. Each was fetched to the laptop and passed `shasum -c SHA256SUMS`. Neither seed needed a rerun.
- **The export needed `--raw-gates --raw-transition`.** `cpu-fbs-export` (`724a3b78…`) refuses a train-as-served checkpoint without both flags. The reader was fixed and the export rerun. Nothing had been scored with the wrong flags.

## Measurements

**The served replies on `m2-fresh256`** (`served-reply-panel` at bin `be3d4cb0…`, greedy, 64-token cap, rendering `one-user-turn/v1`):

| Seed | Artifact sha256 (I-x) | Stops EOS / BOS / cap / context | Complete | Pairs | Most common EOS-stopped | Distinct (all) | Most common (all) | Median reply | Output sha256 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `1c9ae43c3db8917766a06160075d0683bcfa116dcac42bdec24ac06bfbcb6ff0` | 253 / 0 / 3 / 0 | 0 | 0 | **73** | 120 | 73 | 11 | `5175c29bcaa5bdb492f8215e41f8e3c2b16c9abe1a3e4861324df921738090e7` |
| 2 | `99239ea331e9b16ae163762fb7f580e93c3fc8f2b13a51b01ae29443ad539adc` | 244 / 0 / 12 / 0 | 0 | 0 | 40 | 104 | 40 | 18 | `468d44696830f920309393d277c7e5180471e5a30881e4728968038bbbf23973` |
| (b64L-D3), claude-309 | | 0 / 0 / 256 / 0 | 0 | 0 | 0 | | | | |
| claude-259 (b) s1, claude-311 | | 242 / 0 / 14 / 0 | 0 | 0 | 129 | 24 | 129 | 14 | |

Both artifacts are 2,156,416 bytes. Seed 1's EOS-stopped collapse is spread over the strata: the top reply's count per stratum is 15, 18, 23, 10 and 7 (length2, length4, length8, reassert, update). Seed 2's most common reply is " I'm a helpful ass N,ingitalot'll have four there anything I." (40 times).

**The bars:**
- **Stopping** (at least 128 of 256 EOS stops at both seeds): **met**, 253 and 244.
- **Collapse** (at least 64 identical EOS-stopped replies at a seed): **seed 1 collapsed** (73); seed 2 did not (40).
- **The panel** (at least 104 complete and 52 pairs at both seeds): **not met**, 0 and 0.

**The no-memory panel** (`scale-probe grade`, `train=` `mw-balp/train` `3dee858f…`/`b5e7352d…`, no `clean=`; grades `395ea543…` and `aa725ed9…`). Descriptive only:

| Pair | Pooled verdict | Pooled d, seed 1 / seed 2 | Band | Output sha256 |
|---|---|---|---|---|
| (b64L-mw) vs claude-301's (b64L-D3) (`e17a1354…`, `3170c779…`) | behind (every split) | −304,197 / −307,764 | 3,567 | `ab60da6d3395b807749e551f2ae0dad06a806a3c5b29ab0f00855fb50e6bf84c` |
| (b64L-mw) vs mw-balp's T2 | inside | −762 / +6,223 | 6,985 | `91bbdbcb47e853b03ebad87d95782425bbd52967c56d5385b2104ec5913e1fae` |
| (b64L-mw) vs mw-balp's unigram | ahead | +26,369 / +33,354 | 6,985 | `88f64e7fc0b4c20f30932e5d7c66d672aee00af2a26fbcf4f0c563e12a9d7df3` |

**The DEV curve** (`progress.jsonl` `dev_nearest.hits`, every 1,024 steps):

| Step | 1,024 | 4,096 | 8,192 | 12,288 | 16,384 | Peak |
|---|---|---|---|---|---|---|
| (b64L-mw) s1 | 6,904 | 8,439 | **8,672** | 8,278 | 8,246 | 8,672 at 8,192 |
| (b64L-mw) s2 | 7,013 | 8,221 | 8,528 | 8,773 | 8,838 | 8,838 at 16,384 |
| (b64L-D3) s1 | 2,745 | 1,281 | 2,326 | 3,093 | 3,185 | 3,219 at 13,312 |
| (b64L-D3) s2 | 2,751 | 2,564 | 3,299 | 3,518 | 3,458 | 3,518 at 12,288 |

The (b64L-D3) curves come from the d3-accum seals `results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s{1,2}/`. Both passed `shasum -c`.

**The loss** at step 16,384: 0.573 (seed 1) and 0.445 (seed 2).

**The smoke:** I-f PASS at both seeds (26,640→26,832 and 26,672→26,704 MiB). I-t recorded 0.405 and 0.399 s a step, with W 7,534 and 7,420 s.

## Checks run at the head

- **On-pod script hashes** before launch: all three of `job-312.sh`, `chain-312.sh` and `seal-312.sh` equal the pins at both pods.
- **The job's own checks, from the seals:**
  - `EXIT` is 0 at both seeds.
  - `.i3` reads `ok` at both.
  - The no-memory `.i5` reads `same` on all five splits (oasst2, wiki-00000, wiki-00020, wiki-00040, dev) at both.
  - `score.exit` is 0 at both.
- **I-c:** the scorer controls were run by claude-312 at `be3d4cb0…`. The negative (claude-309's `d02ec432…`) and the positive (claude-311's `19849a18…`) both came back row-equal. The scorer binary used here hashes to `be3d4cb0…`.
- **I-x:** both artifacts' sha256 are recorded above. The scorer accepted both (exit 0), and each output's `inputs.artifact_sha256` equals the export's.
- **The panel files and tokenizer** pass the bin's own manifest checks (`inputs.*_sha256` in each output).
- **The collapse, distinct and median figures** are from `jq` over each output's `rows[]`.
- **The `mw-balp/train` copy** the grade read hashes to `3dee858f…` and `b5e7352d…` (the reader refuses otherwise).
- **`loop report --check`** on this REPORT.

## Decision review (row 3)

The sheet's row 3 asks: next, a larger dialogue corpus, or the mix?

**The mix: D3 + `mw-balp/train`, at (b64L)'s recipe, under its own sheet.** Reasons, from this unit's evidence:
1. **The trunk cannot give up D3.** mw-balp alone is behind (b64L-D3) on every held-out split by about 304,000 rows, pooled over the four held-out splits. The one-track bar is the no-memory panel against TRAIN's T2 and unigram on human text (claude-306's sheet). A mix keeps D3's held-out gains; a larger dialogue corpus alone would repeat this loss.
2. **The collapse here is milder than claude-259's,** at a different recipe:
   - 120 and 104 distinct replies, against 7–24;
   - a most common reply of 73 and 40, against 128–152.
   One seed is under the bar. The comparison crosses recipes, so this is the lab's judgement, not a measurement: more varied text is the cheaper lever to try before a bigger dialogue corpus.
3. **Seed 1's DEV curve reads as over-fitting mw-balp** (its peak at step 8,192, then a fall). A mix lowers the passes over the dialogue documents at the same steps.
4. **The store's dialogue root holds no larger corpus.** `uor-store ls w2-inputs/2026-10-25/driver-data/` lists only `mw-balp/train`, `ft/dev` and the tokenizer. Building a larger one is its own corpus unit; the mix needs only a mixing step on two stored roots.

What a pass and a fail would each unlock: the mix sheet should hold this unit's two bars fixed (stopping at both seeds; collapse under 64 at both seeds). Its held-out pair should be against D3's (b64L). If the mix keeps stopping and passes collapse, the next is recall (the panel's complete bar). If stopping falls under 128, the mix ratio is the lever.

This is the claude lab's own decision review (§5.1), logged for Casey, who can reverse it.

## Skills used

- None beyond the loop's own commands.

## Still open

- **The D3 + mw-balp mix sheet,** under its own unit: the ratio, its EOS rate per million, and the bars above fixed before any run.
- **The scorer's timer** still measures re-parse plus decode (claude-311). A decode-only figure would make tokens per second comparable.
- **DeepSeek's #3833** (EOS rank, per-turn rendering) is unmerged. This unit scored with the pinned `be3d4cb0…` under `one-user-turn/v1`.
- **`watch-312.sh` has no exec bit.** It was run as `bash watch-312.sh`, which leaves its bytes and hash unchanged. The first launch exited 126 and left both pods about a minute unwatched (FRICTION).

## Plain summary

We retrained the same small model on a dialogue corpus instead of plain human text, at two seeds, on two rented GPUs for $2.43. It now ends its replies on its own (253 and 244 of 256, against none before). But the replies don't answer the prompts: one seed says "Got it, I'll remember that." 73 times. And the model lost much of what it had learned about ordinary human text. The next step is to train on a mix of the two corpora.
