# #1187 — feat(native_geometric): qualify generalized multi-step reasoning (#955)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T04:19:57Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-09-08T04:21:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1187
- **Merged:** 2026-09-08T04:21:43Z
- **Merge commit:** 8cc948a10d1947b8f599cbe140e055b12229d474
- **Base / Head:** main / UOR-Foundation:issue-955-multi-step-reasoning
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
This PR implements and qualifies generalized multi-step reasoning, counterfactual dependency tracking, and constraint preservation for the native geometric language model, fulfilling Roadmap Position 06 ([#955](https://github.com/UOR-Foundation/uor-r4/issues/955)) under Programme Tracker [#820](https://github.com/UOR-Foundation/uor-r4/issues/820).

### Key Deliverables
1. **Multi-Step Dependency Construction (DAGs)**:
   - Implemented `ReasoningChain`, `ReasoningStep`, and `MultiStepReasoningEngine` in `crates/uor-r4-core/src/native_geometric/multi_step_reasoning.rs`.
   - Enabled compositional DAG execution where step $k$ causally takes intermediate state from preceding step $j$ ($j < k$).
2. **Counterfactual Intermediate State Mutation**:
   - Implemented `intervene_intermediate` to mutate intermediate states ($17 \to 20$) and causally propagate changes across downstream transitive dependencies ($34 \to 40$, $24 \to 30$).
   - Verified deterministic sensitivity of terminal outcomes ($24 \to 30$).
3. **Intermediate State Ablation**:
   - Implemented `ablate_intermediate` verifying that removing an essential intermediate dependency step halts downstream execution, proving causal load-bearing dependence on intermediate state.
4. **Constraint Preservation Across Transformations**:
   - Implemented `ConstraintPolicy` (`Range { min, max }`, `NonNegative`) checked across intermediate DAG steps.
   - Surfaced constraint violations whenever an intermediate calculation or terminal state breaches invariant bounds.
5. **Relational Transitive Deduction**:
   - Implemented `execute_transitive_relation` chaining durable memory facts ($A \to B \to C$) using versioned records in `DurableSession`.
6. **Standalone Compiling Rust Code Synthesis**:
   - Implemented `generate_rust_code` emitting standalone Rust programs verifying the reasoning DAG.
   - Verified clean compilation with `rustc --edition=2021` and execution with exit code 0.
7. **Novel Operand Transfer**:
   - Evaluated novel operand values on identical multi-step DAG structure, confirming generalized arithmetic routing and constraint compliance.
8. **Invariant & Allocation Safety**:
   - In-tree allocation census tests pass with 0 runtime heap allocations on `#![no_std]` hot path.
   - Integer serving kernel retains zero float types and zero forbidden arithmetic opcodes (`*`, `/`, `%`).
9. **Resource Accounting & Policy Compliance**:
   - Recorded milestone in `docs/integration/current-state.md` with 4.500s model charge (cumulative 5,327.918 / 5,550s; 222.082s remaining).
   - Preserved 128 MiB storage stop margin.
   - Clean `cargo fmt --check` and `scripts/check_claim_wording.py` (0 violations).

References #955, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

