# #203 — [Runtime] Enforce Strict tokenizer_cid Verification

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T02:18:52Z
- **Updated:** 2026-07-26T02:43:36Z
- **Closed:** 2026-07-26T02:43:36Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/203
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Objective & Root Cause
If the generation-time tokenizer diverges from the compiled artifact tokenizer, token IDs shift silently. A token ID mapping mismatch causes the R4G1 zero-multiply prediction kernel to look up wrong candidate rows in the packed graph, generating confident nonsense tokens and inflating held-out cross-entropy bits/token.

### Implementation Directives
- [ ] Enforce strict byte equality between loaded `tokenizer.bin` BLAKE3 CID and `R4G1Header::tokenizer_cid` in Stage 1/2 validation inside `crates/uor-r4-graph-format`.
- [ ] In `crates/uor-r4-core/src/transformerless/runtime.rs` and `uor-r4-api`, fail fast with `LoadError::TokenizerCidMismatch` if CIDs diverge.

### BDD Acceptance Criteria (Cucumber / Scenarios)
Feature: Strict Tokenizer CID Validation
Scenario: Reject artifact loading when tokenizer CID diverges
- Given a compiled R4G1 graph artifact with header `tokenizer_cid`
- When a runtime attempts to load the graph with a mismatched `tokenizer.bin`
- Then loading fails immediately with a `LoadError::TokenizerCidMismatch` error
- And no prediction step is executed with mismatched token IDs

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

