# #407 — Compile-path quality: retrained same-budget RVQ beats the shipped codes by +2pp / −0.5 bits (#395 v1 control finding)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-04T13:06:30Z
- **Updated:** 2026-08-04T18:35:34Z
- **Closed:** 2026-08-04T18:35:34Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/407
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation (spun out of #395 v1, PR #404)

While isolating the lattice variable, the retraining control produced a result that deserves its own track: a freshly trained residual VQ at the SAME budget as the shipped compile (4 stages × 256 centroids, deterministic k-means on a construction subsample, 8 Lloyd iterations) beats the shipped artifact's compiled codes by **+2.0pp top1 / −0.52 WB bits** (36.8%/7.9266 vs 34.8%/8.4461, identical store build and eval; PR #404's `rvq-f32` arm, reproduced across PR #406's arms).

That is free performance sitting in the compile path — orthogonal to every geometry question. Either the shipped compile's codebook training is leaving quality on the table (iterations, initialization, subsampling, stage residual handling), or the fixture artifact predates a training improvement and needs recompiling, or the harness training differs from the compile path in some way worth understanding (in which case the delta is diagnostic, not free).

## Scope

1. Diff the harness k-means (PR #404) against the shipped compile's codebook training path — identify the differing choices.
2. A/B the differing choices individually at fixed budget (batch per Rule 1) on the fixture corpus.
3. If real: recompile the era artifact with the improved recipe, re-pin κs per measurement discipline, and record the certify-row deltas; if diagnostic: record why the harness number overstates.

## Non-goals

No lattice/geometry changes (tracked in #395); no schema changes; κ re-pinning only with the full record per discipline.


## Comments (2)

### comment 5181931251 — 2026-08-04T16:36:37Z — Casey-allard

## Step 1 complete: the diff is found, and it's stark

Shipped compile path (`compiler.rs`): context codebooks trained via `sampled_kmeans_rvq(CTX_SAMPLE = 6_000, CTX_ITERS = 6)` — **6,000 sampled context vectors for 1,024 centroids (4 stages × 256), ~23 samples per centroid**, 6 Lloyd iterations, xorshift-shuffled sample. The +2pp/−0.52-bit harness control (PR #404) trained on ~50,000 strided construction vectors with 8 iterations, same K/stages/residual scheme.

Primary suspect: **sample starvation** (6k for 1,024 centroids), with iterations secondary. Step-2 attribution sweep is running now — (samples, iters) ∈ {6k, 50k} × {6, 8} with identical machinery, store build, and eval, shipped artifact as the reference row (single batched run per Rule 1). If 50k@6 recovers most of the delta, the fix is a one-constant compile change (CTX_SAMPLE) + recompile + κ re-pin per discipline; results and the recommendation to follow today.

### comment 5182267035 — 2026-08-04T17:09:31Z — Casey-allard

## Step 2 results (PR pending): attribution SPLIT — the bigger share is era self-consistency, not sample size

| arm | top1 | agree | WB bits | keys |
|---|---|---|---|---|
| shipped era artifact | 34.8% | 39.1% | 8.4461 | 179,068 |
| shipped recipe replica (6k samples, 6 iters) | 36.2% | 40.7% | 7.9820 | 190,270 |
| 6k, 8 iters | 36.1% | 40.6% | 8.0284 | 189,385 |
| 50k, 6 iters | 36.7% | 41.3% | 7.9794 | 192,463 |
| 50k, 8 iters | **36.9%** | **41.4%** | **7.9515** | 191,862 |

Attribution: **(a) +1.4pp** appears just by retraining the SHIPPED recipe (6k/6it) against the current corpus's own bundles — an era/self-consistency effect (the pinned artifact's codebooks fit an older pipeline/corpus state); **(b) +0.5–0.7pp** from CTX_SAMPLE 6k→50k (the starvation share — real but the smaller half); **(c) iterations ≈ noise** at 6k, +0.2pp at 50k.

**Recommendation (close-out path):** bump `CTX_SAMPLE` 6_000 → 50_000 (compile cost: minutes) and recompile the era artifact against the current fixture corpus — expected ≈36.9%/7.95 vs today's 34.8%/8.45 on the same eval. Both steps require the κ re-pin ceremony per measurement discipline (era fixtures review) — flagging that as the board-sign-off step. Sweep harness in the PR for reproducibility.


---
_Comment/review ids in this file: 5181931251, 5182267035_

