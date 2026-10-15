# #204 — [Store] Deprecate Legacy u16 Store and Enforce u32 Recompile

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T02:18:54Z
- **Updated:** 2026-07-26T03:13:10Z
- **Closed:** 2026-07-26T03:13:10Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/204
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Objective & Root Cause
The codebase underwent a vocabulary migration from 16-bit token IDs (`u16`) to 32-bit token IDs (`u32`). However, `runtime::parse_store_legacy_u16` remains in the runtime initialization path, silently reading stale pre-migration 16-bit store files from `.uor-models/`. Reading legacy 16-bit store files silently truncates 32-bit token IDs, corrupting candidate graph lookup tables and inflating held-out cross-entropy loss to 37.15 bits/token.

### Implementation Directives
- [ ] Deprecate `parse_store_legacy_u16` and enforce 32-bit token integer alignment (`u32`) on standard R4G1 artifact load paths.
- [ ] Add `r4 store purge-legacy` CLI subcommand in `crates/uor-r4-graph-cli` to purge stale 16-bit store files and force a clean 32-bit recompile.

### BDD Acceptance Criteria (Cucumber / Scenarios)
Feature: Deprecate Legacy u16 Store
Scenario: Reject 16-bit store file during standard runtime load
- Given a legacy R4G1 store file compiled with 16-bit token IDs
- When loaded by the standard `uor-r4-api` engine
- Then loading fails fast with `LoadError::LegacyStoreDeprecated`
- And instructions to run `r4 store purge-legacy` are output

### Verification & Implementation Gates (from r4_graph_compiler_implementation_plan.md)
- [ ] Gate A (Correctness): Unit tests pass and reject malformed inputs.
- [ ] Gate B (Runtime Contract): 0 allocations per token on hot paths (tests/allocation_census.rs).
- [ ] Gate E (Reproducibility): Deterministic output maintained (kappa_reproduction.rs).

### Required Documentation Updates
- [ ] Update docs/transformerless/BASELINE.md or GLOSSARY.md (if definitions change).
- [ ] Ensure architectural ADRs or plan docs reflect this constraint.
- [ ] Update Agent documentation (AGENTS.md) if operational rules change.

Epic Tracker: Relates to #201


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

