# #11 — R4G1 wire-format RFC and GraphView validation invariants

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-21T21:24:02Z
- **Updated:** 2026-07-22T02:17:41Z
- **Closed:** 2026-07-21T22:51:50Z
- **Labels:** graph-compiler
- **Milestone:** Phase 0: Baseline and contracts
- **Assignees:** auser, afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/11
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Draft the R4G1 container spec (sections HEAD/CODE/NODE/EDGE/ROUT/EMIT/EXCT/PROV/CERT, offset/alignment/checksum rules) and the stage-1/stage-2 validation invariants (plan §5 Phase 0/1, §6 A1/T8). Draft at docs/transformerless/R4G1.md; freeze widths in Phase 1. Plan: docs/r4_graph_compiler_implementation_plan.md §9.1

## Comments (1)

### comment 5041071998 — 2026-07-22T02:17:41Z — auser

Both halves now implemented (commit e700432, local main pending push):

1. RFC: docs/transformerless/R4G1.md (draft line) — PR #36 added the draft/stable version gate; §4.1 + §5 now record the concrete v0 layouts.
2. crates/uor-r4-graph-format: stage-1 structural validation (header/section table, checked offsets, unknown-mandatory rejection, CID integrity) and stage-2 semantic validation (HEAD 224-byte payload, PackedNode/PackedEdge range resolution, reverse-index existence, HEAD-bound honesty, ROUT v0 bytecode, EMIT/EXCT storage descriptors). GraphView is constructed only after both stages; borrowed zero-copy views; no_std ladder holds.

57 tests in the crate (25 stage-1 + 32 stage-2), one rejection test per invariant. Theorem-7 reverse-index check is the documented v0 existence approximation (per-node range wiring comes with the EDGE producer in Phase 4). Full workspace green, clippy -D warnings clean, κ-reproduction green.


---
_Comment/review ids in this file: 5041071998_

