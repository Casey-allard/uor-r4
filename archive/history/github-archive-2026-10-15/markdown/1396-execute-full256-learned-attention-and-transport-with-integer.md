# #1396 — Execute full256 learned attention and transport with integer numerical kernels

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-25T19:32:51Z
- **Updated:** 2026-10-09T13:17:16Z
- **Closed:** 2026-09-25T19:34:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1396
- **Merged:** 2026-09-25T19:34:07Z
- **Merge commit:** 0b85bc66b5cbb5e8aa4b36dfc84e5535b1518dbd
- **Base / Head:** main / UOR-Foundation:codex/full-context-integer-20260925
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Delivered

Implements full-context integer numerical execution of the accepted learned-code recurrent models: signed-code loading, low-bit affine maps, integer RMS and transport, all-causal attention/NoRead, state updates and normalized vocabulary/copy output. Both quaternion and matched Householder arms are retained. Offline Rust builds bound nonlinear tables once.

Training provenance/evaluation/session context remains **256/256/256**, with **full admission**. No new training, admission sweep or checkpoint selection.

## Measured outcome

- Matched-parent complete source answers retained: **28/32 quaternion, 24/32 ordinary**, zero losses/gains; NoRead **0/32 each**.
- State and probability drift meet the unchanged **0.01** limits for Read and NoRead on four existing full256 calibration windows per mode. This does not rerun/qualify the full natural-language gate.
- First comparison omitted the historical parent's Accelerate features. It and the ordinary drift failure are preserved. The prospective comparator-only rebuild restores historical F32 source outputs; all **4,096 integer probability digests**, 128 source variants and20 continuations are unchanged across builds (elapsed fields excluded).
- Compiled inspection:41 symbol ranges,9,438 instructions; no floating arithmetic/conversion or hardware product/division on numerical model values. Five shape/address multiply/divide instructions remain. No entire-step/executable claim.

## Executed verification

Ten new arithmetic/interface tests and two retained codec tests; optimized build including the historical `metal,cpu-accelerate,reference-accelerate` features; two paired loaded comparisons, about112 seconds each; independent arithmetic/instruction and output reviews; formatting, claim wording, whitespace, JSON and local Markdown file-link checks. DeepSeek numerical review and both Rust model workers ran through RDC on the local M1 while implementation continued.

Source used for both comparisons: `45da66d4a2e77a908411c7084e109f6a17472114`.
Final executable SHA256: `554d95eecc552be72bb2923fbbdba7125df1419c1062546d863dd427cac24b6e`.

## Limits and next direction

This is a dense, allocating prototype in the training crate. Seeded generation retains a floating host sampler; only the declared numerical model-value path is integer. Measured incremental calls are **28–29x slower** than the matched F32 reference. Actual stories remain semantically unreliable. General language, geometric advantage, Hamiltonian dynamics, terminal D5 sparsity and energy remain unqualified.

The canonical plan/current state and frontdoors now advance a **standalone integer serving session with measured complete-generation cost**, fixed full256 access and retained codes. No new model fit or threshold tuning is the next step. All original failures and exact artifacts remain preserved.

Pre-delivery whole-cycle charge36.27 minutes; cumulative543,323,181/547,200,000ms; no extension/deletion. Physical free16.56GiB; conservative current allocation346.41MiB under1.5GiB. Delivery tail will be charged once and recorded on owning issues. Five queue status names are compatibility acknowledgements, not tests.

[Result](docs/integration/integer-execution-result-2026-09-25.md) · [actual outputs](docs/integration/integer-execution-outputs-2026-09-25.md) · [current state](docs/integration/current-state.md).

References #973, #820. Both issues remain open.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

