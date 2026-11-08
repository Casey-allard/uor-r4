# REPORT: scale-probe, claude-317 (the width slope on (b64L))

Lab: claude. Order: [ORDER.md](ORDER.md). Item: [scale-probe](../../../../roadmap/items/scale-probe.md) (#3472). Pin: `ff5156bcc9539aa5bccd0319df3d96ee5dd5a71f`. Sheet: [scale-probe-b64l-width-2026-11-08.md](../../../../harness/prep/scale-probe-b64l-width-2026-11-08.md). PR #3842.

**The outcome: row 6, "nothing read", as registered (I-q failed); row 2b, "rising below 256 only", after the in-unit I-q repair, a deviation recorded under Decision review.**
- R2, (w256) against (w128), is **ahead** at both seeds, both pairings, with the unseen split ahead too.
- R1, (w512) against (w256), is **not read**: width 512 did not fit the card.

Every number below names its source. The run scripts are in `local/claude/claude-317/`, uncommitted, with their hashes in the sheet and in `hashes-r.txt`.

**Claim check.** This is Track B of Casey's 8 Nov decision, "Both in parallel" (#2028 6715318106): the P5 scale slope, read on the first recipe that learns human text (claude-301's (b64L)). It answers "does this architecture get better with width", at one corpus size (D3) and a fixed 16,384 steps × batch 64. It does not change the served model.

## What did not work, first

- **(w512) is NOT_RUN at both seeds, as an instrument (the card's memory).**
  - I-e passed on pods A and B: (micro) matched (whole) to 2.7e-08–2.8e-07 relative, against a tolerance of 1e-06.
  - The smoke then ran out of memory at micro-batch 8 and again at 4 (`CUDA_ERROR_OUT_OF_MEMORY`, 14–15 s in), at both seeds. On pod B, `mem.log` shows the card jumping from 11,248 to 31,856 of 32,607 MiB in one 2 s sample.
  - Both micro-batch sizes failed the same way, so the large allocation does not shrink with the micro-batch. At width 512 and batch 64 it grows with the whole batch or with the width.
  - Sealed: `results/claude/scale-probe-b64l-width-2026-11-08/runs/{A,B}`. Pods down ($0.11, $0.13). Note on #3472: `msg-w512-notrun.md`.
  - So **R1, R7 and the 512 point of R5 are not read**. That is not a negative about width 512.
- **I-q failed: the GPU scorer is not bit-reproducible across pod instances.**
  - Pod C re-scored claude-301's sealed (b64L) seed 1 at the same pin and the same flags.
  - `positions` and `targets` were byte-identical. `nearest` differed on 33 of 14,287 DEV rows, and DEV hits went 3,185 → 3,188 (pod C `iq.txt`; the #3472 note, `msg-iq.md`).
  - Classified as instrument (the scorer), not model. By the sheet, R1 and R2 were then not read from pod C's rows.
  - **The repair, inside this unit's bound (claude's decision review, logged in `msg-iq.md`):** one pod (R) re-scored both R2 arms from their seals on one build: claude-301's (w256) seeds 1 and 2, and pod C's (w128) seeds 1 and 2. Fresh pre-run review: GO, no blocker (`d67-rescore.md`).
  - What the repair measured:
    - **Same pod, same build:** (w128) s1 scored first and again last gives bit-identical rows on all five splits (pod R `drift.txt`). That is 0 hits of drift.
    - **Pod to pod:** (w128) s1 is 216,455 on pod R against 215,642 on pod C (+813); (w128) s2 is 207,836 against 210,073 (−2,237). (w256) s1 is 331,734 on pod R against claude-301's 331,698 (+36).
    - The binaries are identical: pod R's `binary-vs-c.txt` reads "same binary as pod C". So the drift comes from the instance, not the build.
    - An odd point, recorded and not classified: pod R's (w256) s1 DEV rows equal pod C's I-q re-score (both `08e1c1ee…`), yet pod R's (w128) s1 rows differ from pod C's own scoring of them. The drift looks per instance and per input, not one offset for the whole pod.
  - The largest cross-pod drift, 2,237 hits ((w128) s2), is 1.9% of R2's smallest d (115,279).
- **I-x fails for (w128): the exporter cannot export a 2-head Hamming read.**
  - `cpu-fbs-export` refuses both (w128) cells: "run config Hamming read count differs from the checkpoint" (`crates/uor-r4-cpu-train/src/fbs_snapshot_cpu.rs` ~910).
  - It compares the run config's `hamming_read` (2) with the model config's `heads`, which is 4 for every cell, i.e. the model's own head count.
  - The checkpoints are what the run asked for. `hamming_read.safetensors` holds `d2.margin` and `d6.margin` of shape [2] in (w128) and [4] in (w128h4).
  - So this is an instrument gap in the exporter, which equates two different counts. It is not a training fault. (w128h4) and (w256) export, because their Hamming read count happens to be 4.
  - **R4's served speed is therefore read for (w128h4) and (w256) only.** (w128)'s training speed is read.
- **Speed against context length is not measured.** `served-reply-panel` has no context option. That part of the item's done-when stays open (the sheet's M4).
- **(w128) is behind D3's T2 at both seeds** (R3, below): the narrower model does not beat the bigram table. (w256) is ahead of it (claude-301).

## What changed

- **This REPORT.**
- **One notebook row** at the top of [float-training.md](../../../../research/notebook/float-training.md).
- **Gap 9 of the pipeline map** ([pipeline-map.md](../../../../model/pipeline-map.md)) now records the slope. claude-295's old-recipe run is kept below it.
- **No code changes.** The sheet, its two FIT rounds and the D67 review were committed earlier on this PR.
- **The run scripts** stay in `local/claude/claude-317/`, uncommitted, as for claude-301 and claude-315:
  - `job-317.sh` and `rescore-317.sh` on the pods;
  - `read-317.sh`, `read-r317.sh` and `serve-317.sh` on the laptop.
  - Their hashes are in the sheet and in `hashes-r.txt`.

## Measurements

All are pooled over 1,460,697 held-out rows (row identity `5f88a5b3…`), graded with `scale-probe` `31610a00…` at `clean=` D3's seen set. **I-r: ok**: claude-301's (b64L) grades re-pair against T2 to +75,038 / +85,590 with this build (`pair/ir-w256-vs-t2.json`).

**Top-1 model hits, scored on pod R (one build)** (`grade-r/*/grade.json`):

| cell | seed 1 | seed 2 | mean |
|---|---|---|---|
| (w128), 2 heads | 216,455 | 207,836 | 212,146 |
| (w256), 4 heads (claude-301's (b64L)) | 331,734 | 342,257 | 336,996 |
| D3's T2 | 256,660 | | |
| D3's unigram | 37,421 | | |

- **R2, (w256) against (w128), one build** (`pair-r/r2-*.json`):
  - seed-matched: **ahead**, ahead at both seeds, d +115,279 / +134,421, unseen ahead;
  - crossed: **ahead**, d +123,898 / +125,802, unseen ahead.
  - Pod C's rows, read only descriptively before the repair, gave the same verdict: d +116,056 / +132,177 matched (`pair/r2-*.json`).
- **R1, (w512) against (w256): not read** (NOT_RUN above).
- **R3, each arm against D3's tables:**
  - (w128) against T2: **behind**, d −40,205 / −48,824 on pod R (pod C: −41,018 / −46,587), unseen behind.
  - (w128) against unigram: ahead, d +179,034 / +170,415.
  - (w256) against T2: ahead, +75,038 / +85,590 (claude-301; I-r).
- **R5, the curve (recorded):** at two widths the "fit" is the line through the two means: **+124,850 top-1 rows per doubling, from 128 to 256**. Whether that is distinguishable from flat is R2's verdict, ahead, and never the fit alone. The 512 point is missing.
- **R6, the contrast (pod C's build, d only, one seed, no verdict):**
  - (w128h4) s1 scores 220,083: +4,441 against (w128) s1 on the same pod, and −111,615 against (w256) s1 (cross-pod, drift at most 2,237).
  - So doubling the Hamming heads at width 128 moves about 4% of what doubling the width moves. That points to width, not the head count, as R2's cause. With one seed, it only points.

**R4, speed** (laptop, one process at a time):

| cell | training, positions/s | served decode, tokens/s | served overall, tokens/s |
|---|---|---|---|
| (w128) s1 / s2 | 32,223 / 32,192 | not exported (I-x) | — |
| (w128h4) s1 | 27,666 | 2,751 | 2,740 |
| (w256) s1 | 18,430 | 1,673 | 1,666 |

- Training: 16,384 steps × 64 × 192 positions over each cell's sealed `train.seconds` (6,248 / 6,254 / 7,277 / 10,924 s), rounded, on RTX 5090s: pod C for (w128) and (w128h4), claude-301's pod for (w256). Positions are an upper bound on scored tokens, and the count is the same for every cell.
- Served: `served-reply-panel` on `m2-fresh256` (256 rows, 16,384 reply tokens each). The binary is claude-316's build `45f6692a…`, the one #3841 merged as `main` `3d3bbb6a`. Its rows equal `be3d4cb0`'s on claude-313's artifact (`claude-316/compare.txt`).
- I-xc: `cpu-fbs-export` `724a3b78…` re-exports claude-313's seal byte for byte (`1c9ae43c…`).
- I-x artifacts: (w128h4) `b9e837cf…` (1,083,584 bytes), (w256) `9a370566…` (2,156,416 bytes) (`served/SHA256SUMS`).
- **Reading:** halving the width from 256 to 128 (at 4 heads) makes served decode 1.64× faster and training 1.50× faster, and costs about a third of the held-out hits.

## Checks run at the head

- **`loop report --check`** on this REPORT.
- **I-r** (laptop, `scale-probe` `31610a00…`): claude-301's grades re-pair to exactly +75,038 / +85,590.
- **Seal read-back on the laptop:** `shasum -c SHA256SUMS` passes for `runs/C` and `rescore/R`.
- **No code changed**, so no test suite applies to this diff.

The run's own gates:

- **I-p, I-0, the build, I-3:** pass on pods A, B, C and R (each pod's `job.log`).
- **I-e:** pass on every pod. Pod C: batch 16, micro-batch 4, relative 3.17e-08, tolerance 1e-06.
- **I-1, I-m, I-f:** pass for all three pod-C cells at micro-batch 16. (w512) failed I-1 (out of memory), as above.
- **I-t, recorded only:** pod C's three cells at 0.369–0.370 s/step; claude-321's pod is not this unit's.
- **I-5 and DEV agreement:** every scored cell, on pods C and R, has checks exit 0 (`*.score.exit`, `*.i5`, `*.dev-agree`).
- **I-q:** FAIL (above), repaired on pod R.
- **I-r:** ok.
- **I-x:** fail for (w128), pass for (w128h4) and (w256) (above).

## Seals and cost

- `results/claude/scale-probe-b64l-width-2026-11-08/runs/A` and `runs/B`: (w512) NOT_RUN, with their logs.
- `runs/C`: SHA256SUMS `59eb4a5a…`, tarball `width-C.tar.gz` `8d151a7e…`.
- `rescore/R`: SHA256SUMS `cc55c402…`, tarball `width-R.tar.gz` `29e99fca…`.
- Each seal was read back and checked against its SHA256SUMS, on the pod and again on the laptop. Every pod is down.
- **Spend:** A $0.11, B $0.13, C $3.32, R $0.17, plus about $0.14 for one earlier R pod that never reached SSH and was deleted by `uor-pod`. About **$3.87**, under the ORDER's $30 notice line.

## Decision review

**Repairing I-q inside this unit, and reading R2 from the repair.**
- The sheet's row 6 says an I-q failure leaves R1 and R2 unread, and that "the repair unit ties (w256) again before any width call". The ORDER's bound allows one repair unit for an instrument failure.
- The claude lab ran that repair inside this unit instead of opening a new one. It cost $0.17, and the inputs were sealed and in the store. The decision is logged for Casey in `msg-iq.md` on #3472.
- Both arms were scored on one pod and one build, which reproduces itself bit for bit (w128-s1 twice: 216,455 both times). The registered exact-hash tie is still unmet: pod R's (w256) s1 DEV is `08e1c1ee…`, not claude-301's `488cf911…`. So R2 enters the Bar from pod R's rows as a deviation, not as the registered reading.
- Pod C's rows give the same verdict, so the call does not hang on which build is read.

**Reading R4 with (w128) unexported.** The exporter's refusal is a check that conflates two counts, not a fault in the model. Fixing it is code outside this ORDER's files (`crates/uor-r4-cpu-train`). R4 is reported for the cells that export, and the gap is named.

### What it unlocks (the outcome map, row 2b after the repair; row 6 as registered)

Row 2b reads "as row 2, with the cause of R1's absence named". **The cause:** at width 512, batch 64, the trainer's peak memory does not fall with the micro-batch, so a 32 GB card cannot hold it.

So the next scale-probe unit is:
1. **The trainer's memory path first.** Measure peak memory against the micro-batch at width 256, on one cheap card. If micro-batching does not bound the peak, a 48 GB card may also be too small at 512, and the lever is the trainer, not the card.
2. **Then row 2's tests at 512,** one seed each after a smoke: lr 0.0005, and 32,768 steps. Then D4 at 512. Width 1,024 waits.

Also unlocked:
- **The exporter's Hamming read count check** (I-x above) is a separate fix. It is needed before any (w128) or 2-head artifact can be served.
- **Track A:** (w256) remains the served width. (w128) decodes 1.64× faster (measured at 4 heads) but is behind T2.

Nothing is closed: no configuration and no mechanism (§4.1).

## Skills used

- None beyond the loop's own commands. The fresh-context pre-run review of the rescore scripts was a `delegate:reviewer` subagent (`d67-rescore.md`).

## Still open

- **R1 (512 against 256)**, and with it the slope above 256. It waits on the trainer's memory path.
- **The item's other corpus sizes.** This unit read width at one corpus size, D3.
- **Served speed against context length.** The scorer has no context option.
- **Served speed at (w128) 2 heads.** It waits on the exporter's Hamming count check.
- **Served-against-trained quality parity.** That is claude-308's open item; this unit claims speed only.
- **Why pod-to-pod drift differs between cells** (the odd point under I-q). It is recorded, not classified; it looks per instance and per input, not pod-wide.

## Plain summary

We trained the model at half its current width, on the same human text, and scored it on the same 1.46 million held-out positions. At both seeds the full-width model gets far more right: about 125,000 more correct next-token picks. The narrower model is also behind a simple two-word lookup table. So, on this recipe, making the model wider helps.

Doubling the width again, to 512, did not fit on the graphics card: memory ran out even with smaller pieces of each batch. Fixing that memory use comes next, before we can say whether wider still keeps helping.

The narrower model trains 1.5–1.7× faster, and its 4-head version answers about 1.6× faster (the 2-head version cannot be exported yet). The comparison was re-scored on one machine because scores drift slightly between machines; on one machine they are exactly repeatable.
