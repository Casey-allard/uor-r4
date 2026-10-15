# #201 — [Epic] Token-Index Mismatch Resolution & Diagnostic Pivot

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T02:18:40Z
- **Updated:** 2026-07-26T03:44:45Z
- **Closed:** 2026-07-26T03:44:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/201
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# [Epic] Token-Index Mismatch Resolution & Diagnostic Pivot

## Diagnostic Context & Senior Architectural Finding (BRANCH B Verified)

A senior architectural evaluation of baseline evaluation metrics (R4G1 Compiled: 37.15 bits/token, Teacher Floor: 14.84 bits/token) was conducted alongside a **TinyStories Baseline Partition Test**.

### Diagnostic Partition Results (Gate E / κ-Reproduction)
* **TinyStories Baseline Test**: `cargo test -p uor-r4-core --release --offline --test kappa_reproduction -- --ignored` passed **100% cleanly** against the certified `llama2.c` reference (`model.bin` 60.8MB).
* **Partition Result**: **BRANCH B Triggered**. The evaluation harness and core zero-multiply runtime kernel are mechanically sound (reproducing Top-1 28.9%, Argmax Agreement 31.7%, 6.54 bits/token on TinyStories). The elevated entropy (37.15 bits/token) is strictly isolated to the Hugging Face observation extraction and token-indexing pipeline.

---

## Architectural Doctrine & Context Window Specification

- **Correction on 8-Token Window (`WINDOW = 8`)**: The 8-token rolling window is a declared parameter of the current "Chain 2" context encoder, NOT an immutable architectural invariant.
- **Phase 8 Roadmap Mapping**: Context extension beyond 8 tokens is explicitly planned for **Phase 8 (Multi-Timescale Context Abstraction Layer)** via hierarchical local, segment, and session state levels using compiler-generated update programs.

---

## Completed Diagnostic & Phase 2/3 Items

- [x] #193 - `feat(ui/wasm): Web Worker offloading and WASM event-loop isolation for non-blocking browser UI` (Merged in #199)
- [x] #194 - `feat(api/server): Unified InferenceRequest & InferenceResponse payload protocol` (Merged in #200)
- [x] #203 - `[Runtime] Enforce Strict tokenizer_cid Verification` (PR #206 submitted)
- [x] #202 - `[Compiler] Enforce Chat Template Formatting in Teacher Oracle` (PR #207 submitted)
- [x] #204 - `[Store] Deprecate Legacy u16 Store and Enforce u32 Recompile` (PR #208 submitted)
- [x] #205 - `[Runtime] Document and Enforce 8-Token Window Truncation` (PR #209 submitted)
- [x] #210 - `[Router] Implement Dynamic Engine Fallback Pipeline and D4 Policy` (PR #211 submitted)
- [x] #212 - `[Frontend] W(3,3) Phase Field UI & WebSocket Telemetry Integration` (PR #213 submitted)

---

## Completed Downstream Roadmap Execution (Phase 4 & 5)

- [x] #195 - `feat(standards): UOR-Framework attestation envelope and uor-addr CID serialization across server endpoints` (PR #214 submitted)
- [x] #197 - `feat(compiler/eval): Scaled D3 observation corpus compilation and M.V.G. evaluation certification` (PR #215 submitted)

---

## Verification & Implementation Gates
- **Gate A (Correctness)**: All unit tests pass and malformed token/store inputs are rejected fast.
- **Gate B (Runtime Contract)**: Zero allocations per token on hot prediction paths (`tests/allocation_census.rs`).
- **Gate E (Reproducibility)**: Deterministic output maintained across environments (`kappa_reproduction.rs` verified).

## Documentation Impact
- `docs/transformerless/BASELINE.md`
- `docs/r4_graph_compiler_implementation_plan.md`
- `AGENTS.md`

## Comments (1)

### comment 5081846390 — 2026-07-26T03:44:43Z — Casey-allard

### Epic #201 Execution Complete 🚀

All diagnostic, runtime fallback, UI, attestation, and evaluation certification tasks under Master Epic #201 have been implemented, verified across all workspace quality gates (94 BDD scenarios green, clippy clean, no_std ladder passing), and submitted as PRs:

- **Data Integrity & Token Indexing:** #203 (PR #206), #202 (PR #207), #204 (PR #208), #205 (PR #209)
- **Runtime Resilience:** #210 (PR #211)
- **Frontend & Visualisation:** #212 (PR #213)
- **Standards & Attestation:** #195 (PR #214)
- **M.V.G. Evaluation Certification:** #197 (PR #215)

The repository state is fully synchronized and certified against Gate C expectations.


---
_Comment/review ids in this file: 5081846390_

