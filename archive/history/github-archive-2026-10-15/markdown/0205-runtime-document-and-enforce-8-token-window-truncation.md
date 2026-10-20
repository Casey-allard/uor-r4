# #205 — [Runtime] Document and Enforce 8-Token Window Truncation

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T02:18:57Z
- **Updated:** 2026-07-26T03:18:17Z
- **Closed:** 2026-07-26T03:18:17Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/205
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Objective & Root Cause
The R4G1 zero-multiply prediction kernel is mathematically designed around a sliding 8-token context window (`WINDOW = 8`, dyadic-recency bundle). Downstream callers and UI components assuming long-context window retention (e.g. 128+ tokens) in the micro-prediction loop encounter silent context truncation or state corruption when state vectors exceed 8 tokens. API contracts must explicitly document the 8-token ceiling and enforce sliding truncation across `[t-7..t]` without allocating heap memory on the prediction path.

### Implementation Directives
- [ ] Document the `WINDOW = 8` micro-prediction boundary in `RuntimeState` API contracts in `crates/uor-r4-core/src/transformerless/runtime.rs`.
- [ ] Implement explicit sliding window truncation for input sequences exceeding 8 tokens and emit a `tracing::warn!` log when truncation occurs.

### BDD Acceptance Criteria (Cucumber / Scenarios)
Feature: Document and Enforce 8-Token Window Truncation
Scenario: Sliding window truncation on oversized input sequence
- Given an input token sequence of length 16
- When processed by `RuntimeState::push_tokens`
- Then the active micro-prediction state retains exactly the last 8 tokens
- And a `tracing::warn!` truncation event is logged
- And zero heap allocations occur during state update

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

