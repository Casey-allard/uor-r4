# #703 — serving/#655-B2: migrate Llama teacher weight matmuls to uor-matmul exact GEMM

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T13:14:09Z
- **Updated:** 2026-10-09T13:09:10Z
- **Closed:** 2026-08-14T13:34:20Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/703
- **Merged:** 2026-08-14T13:34:20Z
- **Merge commit:** bde4982bd827b41487eadc66a11cff7518776190
- **Base / Head:** main / UOR-Foundation:i702-teacher-uor-matmul
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #702. Second half of #655 sub-B — the arithmetic migration, scoped by the #700/#701 dry-run to exactly what was proven safe. **Net −254 lines.**

## What
- `matmul` (matrix–vector) and `matmul_batched` (matrix–matrix) in the shared Llama teacher now call `uor_matmul::slice::gemm_float` (pinned exact GEMM). GPT-2's tied lm-head reuses `matmul_batched`, so it's migrated too.
- Removed: the Accelerate `#[link]` FFI (`cblas_sgemv`/`cblas_sgemm`), both `matmul_fast` variants, the hand-rolled canonical loop, and every SIMD dot helper. `fast_matmul_backend()` collapses to one returning `"uor-matmul exact GEMM"`.
- Census guard tightened to zero library-BLAS anywhere (teacher site de-sanctioned + asserted `cblas_*`-free). Census doc updated.

## Why it's safe — the #700 dry-run
`gemm_float` is correctly-rounded exact and **byte-identical across targets** (a determinism improvement — the teacher is now macOS == Linux/CI reproducible, not per-machine). With the switch in place the full κ-sensitive suite passes: model-source (incl. 27 numpy-parity), graph-compiler observe/induction/reproducibility/route_fit, graph-certify (29), and serving **BDD 100 scenarios / 333 steps** (incl. teacher-label replay + the P-4 census). Two full GPT-2 124M compiles are **byte-identical**. Compile cost ~+37% model-source, ~100s/124M compile — modest.

**No fixture re-pin required:** no test pins a pre-B2 teacher-derived κ as a constant. Post-B2 CIDs are a new teacher-arithmetic era, recorded by the teacher's own κ and the "teacher model ready" diagnostic, never retroactively assigned to pre-B2 artifacts (pre-approved).

## P-4
`uor-matmul` enters only the compile-time teacher, not the deployed R4G1 runtime kernel, so the P-4 deployed-inference audit surface is unchanged; dependency audits confirm no BLAS/GPU crate enters the tree.

## Verified
macOS: builds, census guard + numpy parity pass (Accelerate removal confirmed). Linux: full model-source suite, downstream κ suites, 124M canary — all green. `fmt` + `clippy --workspace --all-targets` clean.

## Follows: #655-B3
GPT-2 `conv1d`/`conv1d_batched` + attention scoring (`Q·K` / `·V`) — the remaining hand-rolled f32 matrix ops (not library-BLAS; tracked in the census doc).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

