# #699 — serving/#655-A: matrix-operation census + source-level CI guard

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T11:19:12Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-14T11:39:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/699
- **Merged:** 2026-08-14T11:39:13Z
- **Merge commit:** 97ee5fe4021da346e67af8c5e00f519b030b152f
- **Base / Head:** main / UOR-Foundation:i698-matrix-op-census
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #698. First sub-issue of #655 (serving epic). No behavior or CID change.

## What
- **`docs/matrix_operation_census.md`** — the generated ownership report for every matrix-like operation in the production `model-source → compiler → certifier → bundle → runtime → API` chain, classified `eliminated-at-compile` / `uor-matmul-owned` / `conventional-to-migrate` / `dev-only-oracle`. Deployed R4G1/TLA inference is eliminated-at-compile (P-4 zero-multiply/float); the `model-source` teacher GEMM sites (`cblas_sgemv`/`cblas_sgemm` FFI + hand-rolled fallbacks) are the `conventional-to-migrate` targets for #655-B.
- **`crates/uor-r4-model-source/tests/matrix_operation_census.rs`** — a source-level CI guard.

## Why a source scan
The teacher reaches system Accelerate through `#[link(name = "Accelerate", kind = "framework")]` + `extern "C"` — an FFI **symbol**, not a crate dependency — so `dependency_audit` (Cargo.lock crates) and `inference_audit` (manifest deps) structurally cannot see it. The scan closes that gap: it fails CI if a library-BLAS matrix marker (`cblas_*`, `matrixmultiply`, `openblas`, `dgemm`, `intel_mkl`) appears in any production-chain crate `src/` outside the one sanctioned teacher site. The two audit files are allowlisted — they enumerate forbidden BLAS crate names as **denylist data**, the opposite of a matrix op. A second guard asserts the sanctioned site still owns the FFI, so when #655-B removes it the allowlist + census doc are forced to update in lockstep.

## Verification
- Scan passes: 155 production `src/` files, 0 unsanctioned library-BLAS matrix ops.
- Positive control: a planted `cblas_` in `uor-r4-graph-runtime/src/lib.rs` fails the guard with a `file:line` message (reverted).
- `cargo fmt --check`, `cargo clippy -p uor-r4-model-source --all-targets`: clean.

## Scope
No matrix op is replaced and no uor-matmul is wired here — that is #655-B (the first CID-changing step). Hand-rolled fallbacks are inventoried in the doc; the mechanical guard covers the additively-detectable library-BLAS class.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

