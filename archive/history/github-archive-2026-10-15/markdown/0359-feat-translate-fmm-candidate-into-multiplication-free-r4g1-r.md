# #359 — feat: translate FMM candidate into multiplication-free R4G1 runtime tables

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-03T01:15:37Z
- **Updated:** 2026-08-03T04:06:20Z
- **Closed:** 2026-08-03T04:06:20Z
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/359
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

Issue #290 and merged PR #356 established a deterministic certifier-side FMM candidate and fixed-point feasibility scorer. The current implementation is explicitly not deployed: it uses widened integer products in certifier code and does not emit or serve an R4G1 translation table.

## Objective

Translate the fixed-point FMM candidate into a versioned packed R4G1 representation and an allocation-free serving path, while preserving the deployed-kernel invariants.

## Scope

- define the packed FMM/translation-table section and versioning rules in `uor-r4-graph-format`
- add deterministic compiler/exporter code that emits the section from the fixed-point candidate
- add borrowed `GraphView` accessors and a runtime scorer using only the allowed integer operations
- integrate the scorer behind an explicit engine/runtime capability or feature flag; do not silently replace the incumbent path
- add round-trip, malformed-input, deterministic-rebuild, no_std, fuzz, zero-multiply, and allocation-census coverage
- measure artifact size, serving latency, and teacher-forced parity against the incumbent R4G1 scorer on the §5.2 novel-context protocol
- document the decision gate: adopt only if quality, cost, and invariant checks pass

## Invariants

- no multiply, divide, or float in the deployed kernel
- allocation-free steady-state prediction
- deterministic artifact bytes for identical inputs
- `#![forbid(unsafe_code)]` in the format/runtime paths

## Non-goals

- reworking the certifier-side Jacobi/eigendecomposition prototype
- claiming deployed FMM quality before the packed runtime measurements pass
- removing or weakening the incumbent R4G1 fallback

## Definition of done

A checked-in packed representation, compiler/exporter, borrowed runtime view, and guarded serving path with focused tests and recorded parity/size/latency results. Close only with the merge commit and evidence comment.

Related: #290, #356.

## Comments (1)

### comment 5162169818 — 2026-08-03T04:06:15Z — auser

Completed by merged PR #361: https://github.com/UOR-Foundation/uor-r4/pull/361 (merge commit `79a4766facaafbf3deda7ee69664df1e9e3172d2`). The packed FMM section, deterministic exporter, borrowed GraphView accessors, guarded allocation-free runtime path, and focused malformed-input/determinism/no-multiply/allocation coverage are implemented. Follow-up PR #362 also preserves the context-conditioned path alongside FMM.


---
_Comment/review ids in this file: 5162169818_

