# #310 — Design: learned per-dimension weighting for signature assignment (A-W rows)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-31T21:34:36Z
- **Updated:** 2026-08-01T21:32:59Z
- **Closed:** 2026-08-01T21:32:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/310
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Design note: `docs/learned_signature_weighting_design.md` (PR to follow).

Motivated by a review of `sanity/renegade` (renegade-ml): three transferable ideas for the compilation and assignment paths —

1. **Per-dimension importance weighting** for masked-Hamming assignment: `d_w(s,p) = Σ_j popcount((s XOR p) AND m_j) << e_j`, a strict generalization of the shipped path (b=1, e=0 is today's distance). Weights learned at compile time (contract §4), quantized to power-of-two exponents via the libm-free IEEE-exponent trick.
2. **Exact k-means indexing** (Hamerly/Elkan) as the only acceptable E-step acceleration — preserves byte-identical artifacts; approximate NN explicitly excluded from compiler outputs.
3. **Validation discipline**: held-out rows for the region count k0 (k0/2, k0, 2k0 on a content-addressed split) and explicit invalidation triggers for the update store.

DoD: design note reviewed (⚑ sign-offs); Phase A adds A-W(2)/A-W(4) certifier-side rows to the #243 decomposition matrix; ships only if it beats A-binary per the note's Empirical Criterion, else recorded negative result.


## Comments (2)

### comment 5148248313 — 2026-07-31T23:16:03Z — auser

## Phase A measurement: A-W rows — recorded negative result

Fresh-era rows-only run (`R4_CERTIFY_ROWS_ONLY=1`, 150k/754 corpus regenerated after /tmp loss; equality witness 512/512; op census mul-free). Within-run matrix, top-1 / agreement:

| Row | top-1 | agreement |
|---|---|---|
| A-f32 (ceiling) | 32.1 | 36.4 |
| A-dot-po2x2 | 30.7 | 34.3 |
| A (shipped, shift-add dot) | 30.5 | 34.0 |
| A-R∘G(4) | 29.4 | 33.4 |
| **A-W(4)** | **28.3** | **32.0** |
| A-G(2) | 28.0 | 31.6 |
| **A-W(2)** | **28.0** | **31.6** |

**Verdict against the design note's Empirical Criterion** (≥25% of the A-f32 gap over the uniform sign-Hamming baseline): A-W delivers ~5% of the gap (+0.1–0.3pp). **Ships nothing runtime-side; the rows stand as the recorded negative result**, per the note's exit rule.

Why this is informative, not just negative:

- **Sign-space is saturated.** Every assignment variant confined to the 288-bit sign signature — A-G(2/3/4) graded ladders, A-W(2/4) margin-weighted Hamming — lands at 28.0–28.8. Per-dimension importance weighting moves nothing because the bottleneck is not *which* bits are trusted.
- The 30+ rows (dot, residual, f32) all carry **magnitude and/or residual** information; A-W is a fourth independent confirmation of #243's decomposition.
- The "fold weighting into G's ladders" alternative framing is moot (A-G hits the same wall), and with TLA6 dot tables now shipped, weighted Hamming targeted a legacy path regardless.
- Caveat: this exercised the margin-reliability signal. The MI signal could differ in principle, but the ceiling appears structural. Recommend closing unless a reviewer objects.

Branch `issue-310-aw-rows` (commits `986d1fb`, `7d16264`): the A-W rows (retained as the record), plus two by-products of the measurement setup —

1. **Streaming top-k in `softmax_top3_sample`/`softmax_top8_sample`** (`compiler.rs`): the per-token 32k-entry Vec + full sort replaced by O(vocab·N) streaming selection. Byte-identical corpus (old/new differential on recs + hidden prefixes), equivalence witnessed by `tests/softmax_topk.rs`, **+20–25% gen throughput** measured interleaved under load.
2. **Recorded negative result in `model-source` `matmul`**: a rayon row-parallel exact matmul (bit-identical per-row accumulation) was tried and reverted — per-matmul fork-join costs ~1.5 ms of worker-scheduling latency on a load-average->100 machine, ~10× slower end-to-end (interleaved A/B: 176–250 tok/s serial vs 16–40 tok/s parallel). Comment in code; parallelism in corpus gen must be story-level (new era) or not at all.

Local gates green: `cargo test --workspace`, `clippy --workspace --all-targets --all-features -D warnings`, `fmt --check`, no_std ladder.


### comment 5153542269 — 2026-08-01T21:32:59Z — auser

Closing per the recorded negative result above (2026-07-31 comment): A-W(2)/A-W(4) rows ran against the design note's Empirical Criterion and delivered ~5% of the A-f32 gap — below the ≥25% shipping bar, so nothing ships runtime-side per the note's exit rule. Rows retained as the record, merged in PR #316 (merge commit 700c307963). The streaming top-k by-product shipped in the same PR.


---
_Comment/review ids in this file: 5148248313, 5153542269_

