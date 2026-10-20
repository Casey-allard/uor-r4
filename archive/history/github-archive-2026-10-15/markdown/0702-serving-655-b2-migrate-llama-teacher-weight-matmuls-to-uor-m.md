# #702 — serving/#655-B2: migrate Llama teacher weight matmuls to uor-matmul exact GEMM

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T13:11:10Z
- **Updated:** 2026-08-14T13:34:21Z
- **Closed:** 2026-08-14T13:34:21Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/702
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Second half of #655 sub-B (the arithmetic migration), scoped by the #700/#701 dry-run to exactly the change proven safe.

## What
- `uor-r4-model-source/src/lib.rs`: the shared Llama teacher weight matmuls `matmul` (matrix–vector) and `matmul_batched` (matrix–matrix) now call `uor_matmul::slice::gemm_float` (the pinned exact GEMM). GPT-2's tied lm-head reuses `matmul_batched`, so it is migrated too.
- Removed: the Accelerate `#[link]` FFI (`cblas_sgemv`/`cblas_sgemm`), both `matmul_fast` variants, the hand-rolled canonical loop, and all SIMD dot helpers (`dot_neon`, `dot_avx2_fma`, `dot_portable`, `dot_fast`). The per-backend `fast_matmul_backend()` collapses to one returning `"uor-matmul exact GEMM"`. Net −254 lines.
- Census guard tightened: the teacher site is de-sanctioned and a guard asserts it owns **no** `cblas_*`; the scan now enforces zero library-BLAS in the production chain. Census doc updated.

## Why it's safe (dry-run, #700)
The exact GEMM is correctly-rounded and byte-identical across targets (a determinism *improvement* over per-machine Accelerate). The full κ-sensitive suite was run with the switch in place:
- model-source (incl. 27 numpy-parity), graph-compiler observe/induction/reproducibility/route_fit, graph-certify (29), serving BDD (100 scenarios / 333 steps incl. teacher-label replay + P-4 census) — **all pass**.
- Two full GPT-2 124M compiles are **byte-identical** (determinism holds).
- Compile cost is modest (~+37% model-source; ~100s per 124M compile), not prohibitive.

No test pins a pre-B2 teacher-derived κ constant, so **no fixture re-pin is required**. The teacher-arithmetic era change (post-B2 CIDs differ) is recorded by the teacher's own κ and documented in the census doc — never retroactively assigned to pre-B2 artifacts (Casey pre-approved).

## P-4 note
`uor-matmul` enters only the compile-time teacher, not the deployed R4G1 runtime kernel, so it adds nothing to the P-4 deployed-inference audit surface; the crate-dependency audits confirm no BLAS/GPU crate enters the tree.

## Follows: #655-B3
Migrate GPT-2 `conv1d`/`conv1d_batched` and the attention-scoring accumulations (`Q·K` / `·V`) — the remaining hand-rolled f32 matrix ops (not library-BLAS, so not guard-flagged; tracked in the census doc).

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

