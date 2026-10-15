# #1780 — geometric-stack: head-vs-representation probe — a plain linear readout already reaches served accuracy (negative result)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T03:52:50Z
- **Updated:** 2026-10-09T13:20:27Z
- **Closed:** 2026-10-06T03:54:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1780
- **Merged:** 2026-10-06T03:54:14Z
- **Merge commit:** 2b21c9a32982b22d02f13dea9e0d8d1421c4dd78
- **Base / Head:** main / UOR-Foundation:deepseek/head-vs-rep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Measurement-only. Adds three modes to an existing example (`head-probe`, `hidden-blocks`, `seal-root`; +441/-1, the deletion being a now-redundant `use candle_core::Device;`). No serving change, no artifact, no `.lut`, no Python product dependency. Release build exits 0.

## Two premise corrections, found by reading the source before running

**1. The 9-parameter `OUTPUT_MIX` head is NOT the served head.** `shared_core.rs` line 1: *"No retained-model dispatch uses it."* Inside that file `branch_score()` tries `angular_tree`, then `calibrated`, and only then falls through to the 9-param sign walk — it is the **schema-/1 fallback**. A prior council cited `shared_core.rs:47-48` as evidence that "the defect is the architecture". That describes a path nothing shipped uses.

**2. The `P − A = 0.5423`-bit measurement could not be reproduced or extended.** The `olx-form-2` artifact root symlinks to `/Volumes/UOR-Workspace`, which is **not mounted** (`diskutil list external` is empty); `learner/transferable_lexical.rs` on main hashes `59801a61…`, not the probe's bound `03f83eb8…b250`. Clean blocker, reported rather than worked around.

## Measured

Artifact `chat-100m2-C`, sha256 `3d20344c…`, 96,023,745 params. On its **own** fixed development panel (all three panel files hash-match the run's `inputs.dev`), the recorded `response_mean_nll` `0.09831474578196771` reproduces to **3.2e-8**.

**The served head is the tied embedding** — `Params::head() -> embedding.weight`, dense 4096×1024, 4,194,304 params, **rank 1024/1024**. Weight tying and readout width are already in place.

| arm (held-out, panel D) | top-1 | recovered |
|---|---|---|
| served head | 94.29% (0.1516 bits) | — |
| **Arm L** plain linear softmax over `h` | 93.14% | **98.8%** |
| **Arm K** pure 1-NN over `h`, zero parameters | 88.00% | **93.3%** |

On a harder corpus panel (`chat-v0-p2/heldout`, 6,144 held-out rows): served 3.4269 bits / 53.76%, Arm L 89.8%, pure 1-NN 48.0%. **No fitted readout ever beat the served head**, and Arm L's validation NLL rises from epoch 1 while fit NLL falls to 0.013, on a fit budget **250× smaller** than the head's own supervised visits.

## The rank result

**1025 nonzero singular values, not 9.** The participation ratio *does* land at **9.05** — but that is a coincidence of the concentration statistic, and the direct causal control settles it:

> **Truncating the served head to rank 9 costs 6.10 bits and 52 points of top-1** — 9.53 bits / 1.8% versus 3.43 bits / 53.8%.

A rank-limited head is **not** the binding defect in this artifact. Reaching for the participation ratio alone would have "confirmed" a defect that isn't there.

## Consequence

**Do not fund geometry, routing or typed operators at the prediction step on this evidence.** A plain linear map of `h`, and even a zero-parameter 1-NN, already reaches served accuracy on the panel the project selects on. This does **not** show the head has headroom, and does **not** exonerate the representation as a *capacity* question — it shows representation is not the binding defect here and rank is not either.

D11: Arm L's shape is already shipped as a D11 GPTQ artifact (`d11-chat/balp-export/model.lut`); **Arm K is not D11-shippable as measured** (dense f16 state store plus a cosine inner product). Neither arm's fitted weights are a deliverable.

Cost: ~31 min machine time, CPU-only, peak 6.25 GB RAM, 1.92 GB new storage, no GPU, no external cost. Sealed roots: `head-vs-rep-analysis2-20261006-035136` plus six probe roots. References #820, #973, #1552, #1746.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

