# #698 — serving/#655-A: matrix-operation census + source-level CI guard (production chain)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T11:14:46Z
- **Updated:** 2026-08-14T11:39:15Z
- **Closed:** 2026-08-14T11:39:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/698
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Sub-issue A of #655 (see decomposition roadmap). Foundational, no behavior/CID change.

## Goal
Land the generated matrix-operation ownership report the epic prescribes ("First land a generated matrix-operation ownership report"), plus a CI guard that fails if a NEW conventional library-BLAS matrix op enters the production `model-source → source/graph compiler → certifier → bundle → runtime → API` chain.

## Why a source scan (the gap)
`dependency_audit.rs` (Cargo.lock crates) and `inference_audit.rs` (manifest deps) already forbid GPU/tensor/BLAS *crates*. But the teacher's `cblas_sgemv`/`cblas_sgemm` reach system Accelerate via `#[link(name = "Accelerate", kind = "framework")]` + `extern "C"` — an FFI symbol, not a crate dependency — so the crate-level audits can't see it. A source-level scan closes that hole.

## Inventory (production chain)
- `crates/uor-r4-model-source/src/lib.rs` — teacher (Llama) executor: `matmul` (hand-rolled), `matmul_fast` (macOS `cblas_sgemv` FFI / off-macOS hand-rolled), `matmul_batched` (macOS `cblas_sgemm` FFI / off-macOS hand-rolled), `dot`/`dot_fast`. **Conventional — migrate to uor-matmul under #655-B.**
- `crates/uor-r4-model-source/src/gpt2.rs` — GPT-2 executor: Conv1D `x@W`, attention accumulation, `matmul_batched` reuse. **Conventional — migrate under #655-B.**
- `crates/uor-r4-core/src/transformerless/compiler.rs:60` — `vvexpf` vForce Accelerate FFI is a vector *exp*, NOT a matrix op → out of census scope (tracked separately if all-Accelerate removal is later wanted).
- Deployed R4G1 / TLA runtime (`uor-r4-graph-runtime`, `uor-r4-graph-format`, `uor-r4-api`) — table-native, no float matmul (P-4 enforced). **eliminated-at-compile.**

## Deliverable
- `docs/matrix_operation_census.md` — the classified inventory (`eliminated-at-compile` / `uor-matmul-owned` / `dev-only-oracle` / `conventional-to-migrate`), file:line anchored.
- A CI guard test scanning production-chain crate `src/` for library-BLAS matrix tokens (`cblas_`, `matrixmultiply`, `openblas`, `dgemm`) and asserting they appear ONLY at the census-sanctioned teacher site; a planted conventional GEMM elsewhere fails CI. Modeled on the existing `architecture_boundary.rs` source scan.

## Non-goals (deferred to #655-B)
No replacement of any matrix op, no uor-matmul wiring, no CID change. Hand-rolled fallbacks are inventoried in the doc; the mechanical guard covers library-BLAS FFI/crates (the additively-detectable class).

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

