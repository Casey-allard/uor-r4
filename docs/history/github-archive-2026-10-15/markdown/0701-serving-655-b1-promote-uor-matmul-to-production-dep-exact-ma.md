# #701 — serving/#655-B1: promote uor-matmul to production dep + exact-matmul parity harness

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T12:03:39Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-14T12:25:40Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/701
- **Merged:** 2026-08-14T12:25:40Z
- **Merge commit:** 7219e95761ccddb6f2ff23fae32b268968bf9594
- **Base / Head:** main / UOR-Foundation:i700-uor-matmul-parity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #700. First half of #655 sub-B (arithmetic migration), split to de-risk the CID-changing switch. **No behavior or CID change** — the teacher still uses its current `cblas_sgemm`/hand-rolled path.

## What
- Promote `uor-matmul` (rev `b13c9844`) from the dev-only parity pin (#622, graph-cli) to a **production dependency** of `uor-r4-model-source`.
- Add `tests/uor_matmul_gemm_float_parity.rs`, a differential harness over teacher-shaped f32 operands.

## Findings
- `uor_matmul::slice::gemm_float` returns the **correctly-rounded exact** product — bit-identical to an f64-accumulated reference on every tested shape, and run-to-run deterministic. **Verified byte-identical on Linux and macOS**, which is the portability #655 wants (the current Accelerate path is only per-machine deterministic).
- Against naive sequential-f32 (what `cblas_sgemm` approximates): 2800/3232 cells differ, max relative error 5.1e-1 at a cancellation cell where naive f32 is the inaccurate one. This is exactly the teacher-arithmetic change **B2's κ re-pin will encode** — a move from a per-machine rounded sum to the portable correctly-rounded exact value.
- Transitive deps added: only `uor-matmul{,-codec,-gemm,-kernels}` — pure Rust, no BLAS/GPU. The forbidden-crate `dependency_audit` / `inference_audit` stay green; the #698 census guard is unaffected (it scans workspace `src/`, not cargo deps).

## Verification
- `cargo test -p uor-r4-model-source` — full suite passes (incl. the new harness), on Linux and macOS.
- `cargo fmt --check`, `cargo clippy -p uor-r4-model-source --all-targets` — clean.
- `dependency_audit` / `inference_audit` — pass with uor-matmul in the tree.

## Next: #655-B2 (CID-changing)
Wire `gemm_float` into `matmul`/`matmul_fast`/`matmul_batched` in place of the `cblas_*` FFI + hand-rolled fallbacks; delete the duplicate arithmetic; tighten the #698 census guard to drop the sanctioned teacher site; extend the P-4 audit to the pinned dep source; open a new artifact era + κ re-pin (pre-approved).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

