# #700 — serving/#655-B1: promote uor-matmul to a production dep + teacher-shaped exact-matmul parity harness

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T12:01:46Z
- **Updated:** 2026-08-14T12:25:41Z
- **Closed:** 2026-08-14T12:25:41Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/700
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

First half of #655 sub-B (the arithmetic migration), split to de-risk the CID-changing switch. **No behavior or CID change** — the teacher still uses its current `cblas_sgemm`/hand-rolled path.

## What
- Promote `uor-matmul` (rev `b13c98449948174f590e337c4dc25dfc394a07d0`) from the dev-only parity pin (#622, graph-cli) to a **production dependency** of `uor-r4-model-source`.
- Add a differential parity harness (`tests/uor_matmul_gemm_float_parity.rs`) over teacher-shaped f32 operands that establishes the exact numerical relationship the switch (B2) will introduce.

## Findings (the harness asserts/measures)
- `uor_matmul::slice::gemm_float` returns the **correctly-rounded exact** product: it is bit-identical to an f64-accumulated reference on every tested shape, and run-to-run deterministic (portable, unlike per-machine Accelerate — a determinism *improvement* for κ-reproducibility).
- Against a naive sequential-f32 accumulation (what `cblas_sgemm`/hand-rolled approximates): most cells differ, with large relative deltas at cancellation cells where naive f32 is the inaccurate one. **This is exactly what B2's κ re-pin encodes** — the teacher moves from a per-machine rounded sum to the portable correctly-rounded exact value.
- Transitive deps pulled in are only `uor-matmul{,-codec,-gemm,-kernels}` — pure Rust, no BLAS/GPU; the forbidden-crate dependency/inference audits stay green.

## Follows: #655-B2
Wire `gemm_float` into the teacher (`matmul`/`matmul_fast`/`matmul_batched`) in place of the `cblas_*` FFI + hand-rolled fallbacks, delete the duplicate arithmetic, tighten the census guard to drop the sanctioned teacher site, extend the P-4 audit to the pinned dep source, and open a new artifact era + κ re-pin (pre-approved). That is the CID-changing step.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

