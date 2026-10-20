# #51 — Add fixed-capacity multi-timescale `RuntimeState` skeleton

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-22T03:02:05Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-07-22T17:47:41Z
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/51
- **Merged:** 2026-07-22T17:47:41Z
- **Merge commit:** a59e34016306f4f962fea14bd3856e523aea4879
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-multi-timescale-runtime-state
- **Reviews / inline review comments:** 2 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Phase 5 calls for a bounded `RuntimeState` skeleton: token state wired now, with reserved local/segment/session levels for Phase 8 compiler-generated updates. This change introduces that fixed-capacity state model without widening runtime scope beyond the existing prediction path.

- **RuntimeState shape**
  - Adds `transformerless::runtime_state` with fixed-capacity state primitives:
    - `TokenState<const CAP: usize>` for live token-history tracking
    - `ReservedState<const CAP: usize>` for bounded local / segment / session storage
    - `RuntimeState` wrapper and `RuntimeStateLevel` selectors
  - Exposes explicit capacities for token, local, segment, and session levels.

- **Phase 5 token-state wiring**
  - Replaces the runtime’s heap-backed recency buffer with `RuntimeState` token storage.
  - Keeps current behavior limited to token recurrence penalties while moving the storage model to caller-owned, bounded state.

- **Phase 8 reservation hooks**
  - Adds `SemanticStateSlot` and `ReservedStateUpdate` as the reserved update surface for future compiler-generated state programs.
  - Tracks bounded inserts, saturation behavior, and last applied program id per reserved level.

- **API surface**
  - Re-exports the new runtime-state types from `transformerless::mod` so downstream code can construct and own state explicitly as the runtime evolves.

- **Focused coverage**
  - Adds unit coverage for:
    - fixed-capacity FIFO token retention
    - reserved local / segment / session update hooks and bounded storage semantics
  - Updates the allocation-census path to clear token state through the new API.

```rust
let mut state = RuntimeState::<4, 2, 2, 2>::default();
state.record_token(11);
state.apply_update(
    RuntimeStateLevel::Local,
    ReservedStateUpdate {
        program_id: 7,
        slot: SemanticStateSlot {
            region_id: 1,
            token: 11,
            score_q: ScoreQ::from_raw(5),
            age: 0,
        },
    },
);
```

## Timeline (6 entries; chronological: conversation comments, reviews, review comments)

### review 4751385663 — 2026-07-22T05:54:10Z — auser — COMMENTED

Looks like a good commit


### review-comment 3627803441 — 2026-07-22T05:58:10Z — Copilot

_on crates/uor-r4-core/src/transformerless/runtime_state.rs:45_

`capacity()` doesn’t use `self`, which is likely to trigger `clippy::unused_self` (and the repo runs clippy with `-D warnings`). Either make this an associated const/fn or explicitly allow the lint on this method to keep the `state.capacity()` call style.

### review 4751405567 — 2026-07-22T05:58:11Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR introduces a fixed-capacity, multi-timescale `RuntimeState` model for the transformerless runtime, replacing the heap-backed recency buffer with bounded token state while reserving local/segment/session state hooks for later compiler-generated updates.

**Changes:**
- Added `transformerless::runtime_state` with `TokenState`, `ReservedState`, `RuntimeState`, and update selectors/surfaces.
- Wired runtime recurrence penalties to use `RuntimeState` token history instead of `Vec`-backed `recent`.
- Updated allocation census test to clear token state via the new API and reformatted a long `println!` argument list.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 4 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-core/tests/allocation_census.rs | Clears runtime token-history through `RuntimeState` during allocation census measurement. |
| crates/uor-r4-core/src/transformerless/runtime.rs | Replaces `recent: Vec<u32>` with `state: RuntimeState` and routes recurrence penalties through it. |
| crates/uor-r4-core/src/transformerless/runtime_state.rs | New fixed-capacity state primitives plus initial unit tests. |
| crates/uor-r4-core/src/transformerless/mod.rs | Re-exports runtime-state types/constants and declares the new module. |







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3627803460 — 2026-07-22T05:58:11Z — Copilot

_on crates/uor-r4-core/src/transformerless/runtime_state.rs:110_

Same as `TokenState::capacity()`: this method doesn’t use `self` and may trip `clippy::unused_self` under `-D warnings`. Consider making it an associated const/fn or allow the lint here.

### review-comment 3627803472 — 2026-07-22T05:58:11Z — Copilot

_on crates/uor-r4-core/src/transformerless/runtime_state.rs:239_

`RuntimeState` exposes `ReservedState` only by shared reference, and provides `apply_update`, but there’s no public way to clear/reset local/segment/session without constructing a brand-new `RuntimeState`. Since `ReservedState` already has `clear()`, consider exposing a level-targeted clear to support reusing caller-owned state across boundaries (e.g., new segment/session) without reinitialization.

### review-comment 3627803501 — 2026-07-22T05:58:11Z — Copilot

_on crates/uor-r4-core/src/transformerless/runtime_state.rs:309_

The reserved-state path’s FIFO saturation behavior and `updates_applied()` counter aren’t currently exercised. Since `ReservedState::apply` has special logic when `len == CAP`, adding a small assertion here would lock in the bounded-storage semantics described in the PR (keep last CAP updates, evict oldest, and track last program id/count).


---
_Comment/review ids in this file: 4751385663, 4751405567, 3627803441, 3627803460, 3627803472, 3627803501_

