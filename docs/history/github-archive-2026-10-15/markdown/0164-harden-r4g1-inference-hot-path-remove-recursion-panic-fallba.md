# #164 — Harden R4G1 inference hot path: remove recursion/panic fallbacks and broaden zero-allocation kernel coverage

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-24T18:29:43Z
- **Updated:** 2026-10-09T14:16:58Z
- **Closed:** 2026-07-24T20:23:32Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/164
- **Merged:** 2026-07-24T20:23:32Z
- **Merge commit:** 5127c71d9ad4c82d9034b32e77d24d054dcc94f4
- **Base / Head:** main / UOR-Foundation:copilot/implement-allocation-free-kernels
- **Reviews / inline review comments:** 1 / 2
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR tightens the runtime toward the R4G1 inference contract (bounded, allocation-free, recursion-free behavior on packed immutable graph arrays). It specifically addresses recursion in target collection, panic-based fallbacks, and incomplete allocation-census coverage across runtime kernels.

- **Hot-path traversal made iterative (no recursion)**
  - Replaced recursive graph walks in `check_node_emits` and `collect_target_leaf_nodes` with fixed-capacity explicit stacks.
  - Preserves bounded behavior while avoiding call-stack growth on adversarial graph shapes.

- **Panic fallbacks removed from prediction paths**
  - Replaced `unreachable!("level 0 is always populated")` in transformerless prediction functions with safe fallback behavior for empty/malformed store states.
  - Added explicit empty-store coverage in tests to validate non-panicking behavior.

- **Kernel allocation contract expanded**
  - Extended allocation census assertions from `predict_token`-only to include:
    - `predict_token`
    - `predict_distribution`
    - `predict_candidates`
    - `step`
  - Uses caller-owned fixed buffers to keep steady-state measurement aligned with runtime constraints.

- **Bounded candidate path hardening**
  - Guarded candidate iteration by `min(node_count, node_scores.len())` to prevent out-of-bounds indexing in bounded caller-buffer scenarios.

- **Multiplication operator removal in Cayley-Dickson path**
  - Replaced direct multiply operators with shift/add based helpers for hash/scalar product operations in runtime-side Cayley-Dickson code.

```rust
// before: recursive traversal
collect_target_leaf_nodes(base_graph, edge.dst.0, out, out_len);

// after: bounded iterative traversal
let mut stack = [0u32; 256];
let mut stack_len = 1;
stack[0] = start_id;
while stack_len > 0 && *out_len < out.len() {
    stack_len -= 1;
    let node_id = stack[stack_len];
    // process node, push children into stack
}
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #159

## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### review 4775957248 — 2026-07-24T18:55:43Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR hardens the R4G1 runtime inference hot path toward the recursion-free, panic-free, allocation-free contract by converting recursive traversals to bounded iterative walks, removing `unreachable!` fallbacks from transformerless prediction, extending zero-allocation census coverage across more kernels, and removing multiplication operators from the runtime Cayley–Dickson scoring path.

**Changes:**
- Replaced recursive node traversal helpers in `uor-r4-graph-runtime` with fixed-capacity explicit stacks and added a bound when iterating nodes from caller-provided buffers.
- Replaced `unreachable!` panic fallbacks in transformerless prediction with a safe fallback path and added an empty-store test.
- Expanded allocation census to measure `predict_token`, `predict_distribution`, `predict_candidates`, and `step` together using caller-owned fixed buffers.

### Reviewed changes

Copilot reviewed 5 out of 5 changed files in this pull request and generated 2 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-graph-runtime/src/engine.rs | Converts recursive graph traversal helpers to iterative, fixed-capacity stack-based traversal; bounds candidate scanning by `min(node_count, node_scores.len())`. |
| crates/uor-r4-graph-runtime/src/cayley_dickson.rs | Removes multiplication usage in hashing and scalar product via shift/add helpers. |
| crates/uor-r4-core/src/transformerless/runtime.rs | Replaces panic-based “level 0 always populated” assumptions with a safe fallback prediction path. |
| crates/uor-r4-core/tests/{allocation_census.rs,window_paths.rs} | Extends allocation census to more kernels and adds an empty-store non-panicking regression test. |




<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**crates/uor-r4-graph-runtime/src/engine.rs:433**
* `collect_target_leaf_nodes` has the same cycle issue as the updated `check_node_emits`: it prevents duplicates only among nodes currently on the stack. In the presence of cyclic child edges (especially among nodes with `emission_len == 0`, which are never recorded in `out`), this loop can run forever because nodes can be popped and then pushed again later. Please add a bounded fixed-capacity visited set (independent from `out`) to guarantee termination.
```
                if let Some(edge) = base_graph.edge((start + i) as u32) {
                    let dst = edge.dst.0;
                    if stack_len < stack.len() && !stack[..stack_len].contains(&dst) {
                        stack[stack_len] = dst;
                        stack_len += 1;
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3647538390 — 2026-07-24T18:55:43Z — Copilot

_on crates/uor-r4-graph-runtime/src/engine.rs:241_

The new iterative traversal only de-duplicates nodes that are currently on the stack (`!stack[..stack_len].contains(&dst)`). If the graph contains a cycle (e.g. A→B→A), a node can be popped and then re-pushed later, leading to an infinite loop in `check_node_emits` (unbounded runtime on adversarial graph shapes). This needs a bounded, fixed-capacity "visited" set (or other cycle break) in addition to the stack check.

This issue also appears on line 429 of the same file.

### review-comment 3647538425 — 2026-07-24T18:55:43Z — Copilot

_on crates/uor-r4-graph-runtime/src/cayley_dickson.rs:90_

`saturating_mul_small` returns `0` for any `rhs` outside { -2, -1, 0, 1, 2 }. Since `CayleyDicksonVector.coords` is `pub`, callers can construct vectors with arbitrary coordinate values, and `endomorphism_scalar_product` would silently compute an incorrect scalar product in those cases. If the intent is to keep the function correct for all `i32` without using multiplication, implement a shift/add-based saturating multiply fallback (or make the invariant explicit by constraining the type/fields).

### comment 5073670268 — 2026-07-24T19:33:25Z — Copilot


> ## Pull request overview
> 
> This PR hardens the R4G1 runtime inference hot path toward the recursion-free, panic-free, allocation-free contract by converting recursive traversals to bounded iterative wa...

Addressed both review points in the latest commit:

1. Added fixed-capacity `visited` sets (independent of stack contents) in both iterative traversals in `engine.rs` to guarantee termination on cyclic child graphs.
2. Replaced the limited-operand `saturating_mul_small` implementation with a shift/add saturating multiply that is correct for all `i32` inputs without using multiplication operators.

Validation run after changes: `cargo fmt --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, plus the graph-format no-default-feature checks.


---
_Comment/review ids in this file: 5073670268, 4775957248, 3647538390, 3647538425_

