# #735 — graph-format/graph-runtime/graph-certify/#643: packed R4G1 lowering for MsaStructuredSelectorV1

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T17:11:23Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-08-16T17:38:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/735
- **Merged:** 2026-08-16T17:38:26Z
- **Merge commit:** 9c9bdfba27234c8245b5e2d006af158ae664c860
- **Base / Head:** main / UOR-Foundation:issue-643-msa-selector-packed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Slice 2 of #643: the packed R4G1 lowering of `msa-structured-selector/1`, mirroring the #604 `r4-route-attention/1` two-stage plan (reference semantics first, packed lowering second) and its three-crate split.

**Shared substrate, new:** `crates/uor-r4-graph-format/src/msa_selector.rs` — the canonical `MSA1` instance wire layout (candidate rows of id + role_rank + cascade_position + contribution), `MsaSelectorView` (zero-copy validated parse), `build_msa_selector_instance`, and `MsaSelectorOpCensus`. This crate has no dependency on either `uor-r4-graph-certify` or `uor-r4-graph-runtime` — it's the only place both can share one wire format without a dependency cycle, exactly the `route_attention` precedent. Bounds are shared with `r4-route-attention/1`'s existing 64 candidate / 8 top-M caps.

**Classification is precomputed, not recomputed on the deployed path.** The certify-side reference classifies via `candidate_id mod 11`. The packed lowering is P-4-scanned (no `* / %` permitted) and cannot recompute that modulus, so `build_msa_selector_instance` computes it ONCE at instance-build time and bakes `role_rank`/`cascade_position` into the row bytes — the packed kernel only ever reads a declared table. This matches the registry record's own declared `permitted_operation_class`: `"deployed-integer-table-read-compare-add-no-runtime-modulo"`.

**Packed kernel, new:** `crates/uor-r4-graph-runtime/src/msa_selector.rs` (P-4-scanned, added to `uor-r4-core`'s `p4_contract_owned_graph_runtime_source_scan` list) — `MsaSelectorState` (caller-owned bounded selection slots, the same `StepState` epoch discipline `RouteState` uses) and `msa_selector_step`: declared-table classification read, bounded top-M insertion with the lowest-index tie rule, selection-order saturating `ScoreQ` aggregation. No query parameter — classification is query-independent by construction. Zero allocation in steady state.

**Refactored:** `crates/uor-r4-graph-certify/src/msa_selector.rs` — `MsaSelectorReference` now builds through the shared `build_msa_selector_instance` substrate instead of its own bespoke byte layout from slice 1, so `MsaSelectorReference::new` returns `Result<Self, NotAProduct>` (was `Option<Self>`), and the reference and the new `run_packed()` operate on byte-identical instances — a true differential comparison, not two encodings that happen to agree. Removed the now-redundant local `MsaSelectorOpCensus` (re-exported from `uor_r4_graph_format` instead).

**Tests:** `crates/uor-r4-graph-certify/tests/msa_selector_643.rs` grew from 17 to 26 tests (all passing): bit-for-bit reference/packed agreement on a pinned fixture and a shape grid, packed-state epoch reuse, tie-break/saturation/cap property tests mirroring `route_attention_604.rs`, source scans confirming the packed lowering and the view/census substrate are integer-only (with the one documented build-time modulus exempted and explained), and a registry-identity cross-check.

**Ledger:** registered `msa-structured-selector-dormant` in `model/ledger.toml`, mirroring `r4-route-attention-dormant` — constructible and differentially tested, referenced by no serving path, restating the #643 pre-registered A/B run contract (already posted on the issue) as the claim's exit/activation gate.

## Scope / deferred

This is slice 2 of #643. NOT yet done: the actual A/B evaluation run against `r4-route-attention/1` — no held-out evaluation harness wires either target operator into a shared loop yet.

## Test plan

- [x] `cargo fmt --check` (relevant crates)
- [x] `cargo clippy --all-targets --all-features -- -D warnings` (relevant crates)
- [x] Full `cargo test` across `uor-r4-graph-format`, `uor-r4-graph-runtime`, `uor-r4-graph-certify`, `uor-r4-core`, `uor-r4-model-source` — zero regressions, 26/26 new tests passing
- [x] `cargo run -p xtask -- audit-limits` (R5 gate) — clean
- [x] `p4_contract_owned_graph_runtime_source_scan` (P-4 no-multiply/divide/modulo source scan) — clean, now covers `msa_selector.rs`
- [x] `repo-model` ledger consistency tests — clean (new claim well-formed)

References #643.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

