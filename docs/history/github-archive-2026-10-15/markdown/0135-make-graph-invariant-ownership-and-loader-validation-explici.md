# #135 — Make graph invariant ownership and loader validation explicit

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-24T05:29:24Z
- **Updated:** 2026-07-24T19:07:07Z
- **Closed:** 2026-07-24T19:07:07Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/135
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #122
Extends #11, #17, #20, and #22.
Source: PDF §9.

Turn the PDF's graph-invariant list into a single ownership and validation matrix. Existing R4G1 validation covers many wire-level cases, but the formal direction requires every invariant to have a declared owner and evidence path.

## Invariant inventory

- bounded node degree, overlap count, and active frontier width;
- valid/aligned ranges and no dangling references;
- deterministic node/edge ordering and canonical serialization;
- no duplicate evidence contributions;
- provenance and reverse-index completeness;
- fixed-width arithmetic and no unsafe score overflow;
- refinement acyclicity unless explicitly declared otherwise;
- bounded candidate/frontier work and declared fallback limits.

## Acceptance criteria

- A versioned matrix maps each invariant to compiler construction, packer, loader, runtime, certifier, property test, fuzz target, or formal proof.
- R4G1 stage-1/stage-2 validation rejects malformed artifacts for every loader-owned invariant.
- Compiler-side failures are structured and occur before artifact emission.
- The proof-status matrix distinguishes proven, witnessed, empirically checked, assumed, and unproven invariants.
- Boundary fixtures cover empty graphs, maximum bounds, duplicate contributions, cycles, overflow, and malformed reverse indexes.

## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

