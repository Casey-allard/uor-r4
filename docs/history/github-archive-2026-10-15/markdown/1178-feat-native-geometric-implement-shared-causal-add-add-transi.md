# #1178 — feat(native_geometric): implement shared causal Add->Add transitions (#1140)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T00:55:21Z
- **Updated:** 2026-10-09T14:17:10Z
- **Closed:** 2026-09-08T00:58:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1178
- **Merged:** 2026-09-08T00:58:37Z
- **Merge commit:** 6ce9682de8427aaae2ff0ddd1b3a5dd2fba63d52
- **Base / Head:** main / UOR-Foundation:issue-1140-causal-add-to-add-transition
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Addresses Milestone **#1140** (`Learn shared state transitions and compositional emission`). Implements the shared causal Add -> Add state transition where Operator 2 consumes Operator 1's actual committed result and retains Operator 1's `write_id` as an explicit operand dependency in its `ValueDerivation.operand_ids` (`[write_id_1, next_id]`).

### Architectural Mechanisms
1. **Transition Tracking & Source Synchronization**:
   - Added `source_refreshes: u64` to `ValueWork` to account for source synchronizations.
   - Added `operations_committed: u8` to `ValueState` (with `#[serde(default, skip_serializing_if = "is_zero_u8")]`) tracking multi-stage commits within a transition sequence.
   - Implemented `refresh_sources` in `ValueState` and `refresh_value_sources` in `Session` to synchronize `self.sources` from `self.records` and clear `self.consumed` without runtime heap allocations.
2. **Causal Operand Dependency Binding**:
   - When Operator 1 commits at `cursor == 0`, its derived record is committed with its `write_id`.
   - After source refresh, Operator 2 evaluates the refreshed source view containing Operator 1's output, selects Operator 1's intermediate record, and binds its `write_id` in `ValueDerivation.operand_ids`.
3. **Causal Load-Bearing Intervention**:
   - Validated via ablation testing: removing Operator 1's committed intermediate record causes Operator 2's prediction to diverge (`d_intervened.value != 22`), proving that the intermediate geometric state is strictly load-bearing.
4. **Zero Heap Allocation Invariant**:
   - Verified that `predict`, `observe`, and `refresh_value_sources` execute with **0 heap allocations** on the hot path in `native_geometric_allocations`.
5. **Rust Execution Conformance**:
   - Chained multi-operator Rust program compiles via `rustc --edition=2021` and executes with exit code 0.

### Verification Results
- `cargo check -p uor-r4-core --all-targets --offline`: PASS
- `cargo test -p uor-r4-core --lib native_value_causal_add_to_add_transition --offline`: PASS
- `cargo test -p uor-r4-core --lib native_value_chained_rust_execution --offline`: PASS
- `cargo test -p uor-r4-core --test native_geometric_allocations native_typed_value_ingest_write_emission_and_eviction_are_allocation_free --offline`: PASS (0 allocations)
- `cargo test -p uor-r4-core --test native_geometric_allocations native_kernel_source_has_no_forbidden_arithmetic_or_float_types --offline`: PASS
- `cargo test -p uor-r4-core --lib native_word_copy --offline`: PASS (13/13)
- `cargo fmt --check`: PASS
- `python3 scripts/check_claim_wording.py`: PASS

References #1140, #973

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

