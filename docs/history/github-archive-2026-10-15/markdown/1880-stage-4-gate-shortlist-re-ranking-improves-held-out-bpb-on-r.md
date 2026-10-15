# #1880 — Stage 4 gate: shortlist re-ranking improves held-out BPB on real text, against a uniform-boost control (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T07:10:24Z
- **Updated:** 2026-10-09T13:21:04Z
- **Closed:** 2026-10-08T07:12:01Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1880
- **Merged:** 2026-10-08T07:12:01Z
- **Merge commit:** 064fe86dd2bbb76a9a187a987fb89b27d0e081fa
- **Base / Head:** main / UOR-Foundation:deepseek/lut4-shortlist
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this changes

The first Stage 4 form corrected all 4,096 candidates; the best of all 1,056 two-input terms moved held-out BPB by **+0.0000061** (slightly worse). The plan says the term is a *selected read over the shortlist feature vector* — and one bit shared by half the vocabulary is diluted across 4,096 candidates. This form re-ranks the baseline's own **top-K** instead, with richer features.

- 26-bit features: each of the six additive term groups quantised at **four** thresholds (not one sign and one magnitude bit), plus two **context-level** bits (any engram active, context-length bucket) that let a gate condition a candidate feature on the context — an interaction the additive scorer cannot express at all.
- Shortlist re-ranking, with the target-in-shortlist rate reported (it bounds what any re-ranker can achieve).
- A **uniform-boost control**: a constant `+scale/2` on the same shortlist and nothing else.

## Result — the gate improves held-out BPB on real text

Artifact `7023c450…`; trained on `chat-v0-p2/train` (205 fit + 51 scale-selection tail), held out on `chat-v0-p2/heldout` — a separate real-text file, 256 positions, top-64 shortlist, scale chosen on the disjoint tail.

| form | bits/byte | delta vs baseline |
|---|---:|---:|
| baseline | 4.211070 | — |
| soft (learned) | 4.194177 | **−0.016893** |
| hard (served truth tables) | 4.191505 | **−0.019565** |

Discretisation gap 0.002672 bits/byte; top-1 change rate 0.000 (the term redistributes mass, it does not change the argmax); gate utilisation entropy 0.323 bits of 4.

## The control that makes it credible

A constant boost of `+scale/2` on every shortlist candidate **loses 0.048235 bits/byte** at the same scale, and worsens monotonically with scale (256: +0.0010 … 8192: +0.0482). It hurts because the target is outside the top-64 for **80 %** of held-out rows, so mass moved onto the shortlist is usually wasted. **Learned minus uniform boost: −0.067801 bits/byte.** The gain comes from what the term learned, not from adding mass.

## Scope, and one known defect

One artifact, one corpus pair, 256 positions, debug build, **no paired confidence interval** — the control is the strongest evidence here, not the absolute delta. No serving default changed. Utilisation is low and is reported, not celebrated.

**Known defect, not fixed here:** `Lut4Network::cone_sizes()` reports `0` for this trained network while the served artifact demonstrably varies its output per candidate. The cone accounting for a constant-looking final gate needs correcting before cone size is quoted as a result. Recorded in the evidence document and listed as the next action, with a paired bootstrap CI and a shortlist-size sweep.

Evidence: `docs/evidence/native_geometric_lut4_shortlist_rerank_2026-10-08.txt`. Verification: `cargo build -p uor-r4-core --bin lut4-prose` clean; `cargo test -p uor-r4-core lut4` 6 passed; `rustfmt` clean.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

