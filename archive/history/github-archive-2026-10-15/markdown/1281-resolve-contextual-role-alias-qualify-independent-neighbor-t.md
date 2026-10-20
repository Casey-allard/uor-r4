# #1281 — Resolve contextual role alias, qualify independent neighbor transfer, and synchronize master plan

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-17T06:38:06Z
- **Updated:** 2026-10-09T13:17:11Z
- **Closed:** 2026-09-17T06:39:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1281
- **Merged:** 2026-09-17T06:39:15Z
- **Merge commit:** 41a938290f5264d90ee27084fe1f4e89d6adf45f
- **Base / Head:** main / UOR-Foundation:codex/contextual-role-repair
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- **Crate Qualification Result:** Passes official crate qualification test (`native_geometric::dependent_language::independent_neighbor_report::independent_neighbor_evaluate_report`) with **2,304/2,304 (100%)** (`PASS_INDEPENDENT_NEIGHBOR_TRANSFER`).
- **Failing Case Recovery:** Recovers 100% (288/288) of failing interior-`will` cases across `novel-will-novel`, `will-novel-novel`, etc. (96 valid, 96 missing, 96 conflict).
- **Preservation & Zero Regression:** All 7,680 retained cases pass bitwise equal:
  - 492/492 unknown-neighbor retained cases
  - 300/300 styled retained cases
  - 200/200 earlier sealed traces
  - 6,688/6,688 candidate/current Full traces (including 512/512 typed unresolved)
- **Runtime Invariants:**
  - 0 runtime matrix products or mathematical linear maps at serving.
  - 0 steady-state heap allocations on hot path (verified via `tests/native_geometric_allocations.rs`).
  - 0 compiler warnings (`cargo check --all-targets` clean).
  - Clean `cargo fmt --check` and clean `scripts/check_claim_wording.py`.
- **Mechanism:**
  - Implements `crates/uor-r4-core/src/native_geometric/dependent_language/contextual_role.rs` to disambiguate the opposite-role observation alias where source key `(13, 64, 64)` conflated interior name-`will` (content) with auxiliary `will` (context).
  - Implements `crates/uor-r4-core/src/native_geometric/hopf_metric.rs` providing zero-allocation Hopf state trajectory and serving projections.
- **Master Plan Synchronization:**
  - Commits the 10 comprehensive audit and knowledge base documents in `docs/integration/review-2026-09-16/`.
  - Commits pre-registered experiment cards `docs/integration/cards/P1-ground-truth-m1.md` and `docs/integration/cards/P4-geometry-controls.md`.
  - Updates `docs/integration/current-state.md` and `docs/integration/project-track.md` with the Two-Pillar architecture (Exact Addressed Memory Substrate + Learned Generator with Empirical M1 Baselines) and Cards P1–P6 re-sequencing.

References #973, #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

