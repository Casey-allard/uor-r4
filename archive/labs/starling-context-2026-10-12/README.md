# Starling read against the softmax read at equal budget: behind at 512, fails at 1536 because a fixed store evicts old positions (row 4) (12 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Murmuration-read ladder, step 2.
**Line:** starling read · count 1/3 · headline "softmax at runtime: yes" → yes.
**Pre-registration:** [6116807513](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116807513), with the amendment (new row 1b, a teacher-on probe, a validity floor for L, `probe_steps=none`), the probe note, and the skip note. **Test fitness:** FIT after one round (two blocking items fixed). **Preparation:** [#2235](https://github.com/UOR-Foundation/uor-r4/pull/2235) (a d3000 bucket, `reachability=false`).

## Question

Does the starling read keep its recall as the context grows, against softmax attention over every earlier position, at equal training budget? Each query scores at most 13·M candidates and mixes K = 7, with no softmax at runtime.

## Runs

- **Arms:** `arm=stack pattern=rarr key_shift=true`, width 128, 4 heads, 3,600 steps, seeds 1–3.
  - **L:** the L2 softmax read.
  - **W03L:** the provisional KEEP from [#2233](https://github.com/UOR-Foundation/uor-r4/pull/2233).
- **Batch:** 8 at contexts 512 and 1536.
- **Hardware:** one RTX 4090 (pod o7hbm7x6u0wu60), binary `c04b2f31f`, all runs with `device=cuda`. Every report shows 3,600 steps completed with no early stop. About 45 min of GPU time.
- **Probe** (before the runs): at 4096, W03L ran out of memory in training at batch 8, and every arm ran out of memory in the observation-only reachability counters. The counters are now skippable (#2235). Batch 4 completes end to end.
- **4096 skipped:** once two seeds at 1536 fixed row 4 arithmetically, the 9 runs at 4096 were skipped, saving about 2.5 GPU-hours ([note](https://github.com/UOR-Foundation/uor-r4/issues/2032)).
- **Outputs:** `claude-s2-20261012` in the cloud store (md5 bbac0fea…).

## Result (recall = in-class fresh accuracy)

| Context | L, softmax | W03L, starling | Mean L | Mean W03L | Holds (W03L ≥ L − 0.15)? |
|---|---|---|---:|---:|---|
| 512 | 0.998, 0.991, 0.985 | 0.938, 0.605, 0.950 | 0.991 | 0.831 | **no** (bar 0.841; misses by 0.010) |
| 1536 | 1.000, 0.997, 1.000 | 0.185, 0.529, 0.009 | 0.999 | 0.241 | **no** |

**Decision: row 4. The read degrades early.**
- W03L does not hold at 1536.
- Following the pre-registration, the next step is the stability and addressing mechanism before any context scaling.

**Why it fails, from the diagnostics:**
- **The store is too small for the window.** Each query reads among the last M = 4 keys in each of 120 600-cell vertices, so a head holds at most 480 earlier positions. That covers 94% of a 512 window but only 31% of 1536. The 512 bench therefore never tested retention, only selection.
- **Hit rate at 1536:** 0.56, 0.61 and 0.00, against about 0.94 at 512.
- **Recall with the true source forced into the kept set:** 0.31, 0.83 and 0.01, against natural recall of 0.19, 0.53 and 0.01. Most of the loss on the working seeds is eviction.
- **One seed (3) collapsed outright** at 1536: hit rate 0, train NLL about 5.1 throughout.

**Re-scoping [#2233](https://github.com/UOR-Foundation/uor-r4/pull/2233):**
- Its KEEP stands against its own base, W03.
- It does **not** stand against softmax. At equal budget, L reaches 0.991 at 512. The 0.617 it was compared with was L at 1,800 steps.
- The provisional-KEEP re-check (W03L ≥ L − 0.15 at 512) misses by 0.010.

**What carries forward, scoped:**
- The selection machinery works when the source is retained: on the forced-in seeds, 0.83 at 1536 and 0.74–0.99 at 512.
- What is missing is **retention that does not grow linearly with the window.**

## Next

**Ladder step 3: a persistent base-3 tree over the past** (the HoloBox path-copy pattern with FBS fan-out; see the [W33 overview](../../research/w33-overview-2026-10-12.md)).
- Old positions are retained by learned salience within base-3 time blocks, not by recency within a vertex.
- A query reads its own 600-cell cell in O(log₃ n) blocks, so the read cost grows logarithmically while old sources survive.
- It is pre-registered and reviewed before compute, and scored first at 1536 against L at equal budget.
