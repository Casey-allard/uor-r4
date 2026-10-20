# #202 — [Compiler] Enforce Chat Template Formatting in Teacher Oracle

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-26T02:18:50Z
- **Updated:** 2026-07-26T02:55:25Z
- **Closed:** 2026-07-26T02:55:25Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/202
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Objective & Root Cause
The `SmolLM2-135M-Instruct` teacher model degrades to a 14.84 bits/token floor because it is prompted with raw text instead of its required instructional chat template (`<|im_start|>user\n...<|im_end|>\n<|im_start|>assistant\n`). Prompting the teacher with unformatted raw strings forces softmax probability extraction into an out-of-distribution high-entropy state, corrupting downstream R4G1 graph candidate selection.

### Implementation Directives
- [ ] Update `crates/uor-r4-core/src/transformerless/teacher.rs` to wrap prompt sequences in the model's native chat template before computing logprobs.
- [ ] Update `uor-r4-model-source` and `r4 evaluate-report` in `crates/uor-r4-graph-cli/src/lib.rs` to format held-out evaluation tokens with identical ChatML formatting.

### BDD Acceptance Criteria (Cucumber / Scenarios)
Feature: Teacher Oracle Chat Template Formatting
Scenario: Extract teacher logprobs for instruction-tuned model
- Given an instruction-tuned teacher model requiring ChatML formatting
- When observation sequences are processed by the teacher oracle
- Then prompt tokens are wrapped in `<|im_start|>user\n...<|im_end|>\n<|im_start|>assistant\n` before probability evaluation
- And the teacher cross-entropy floor is measured below 4.0 bits/token

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

