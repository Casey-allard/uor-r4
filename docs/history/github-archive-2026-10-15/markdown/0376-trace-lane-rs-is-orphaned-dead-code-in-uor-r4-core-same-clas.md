# #376 — trace_lane.rs is orphaned dead code in uor-r4-core (same class as the pre-#373 cover_sweep) — delete or revive

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T15:10:23Z
- **Updated:** 2026-08-03T16:08:16Z
- **Closed:** 2026-08-03T16:08:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/376
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

While reviving `cover_sweep.rs` (#373; it was orphaned in `uor-r4-core` — no `mod` declaration, imports unresolvable since the crate restructuring), the same audit found `crates/uor-r4-core/src/transformerless/trace_lane.rs` orphaned the same way: no module declaration anywhere, `use super::cover_sweep::…` (its neighbor, now moved) and lane machinery that nothing compiles or tests. Its text has been kept current by recent commits without ever building.

Maintainer decision wanted: **delete** it, or **revive** it the way #373 revived the sweep (relocate to a crate where its dependencies resolve, wire a command, let its tests run). No position taken here — filing so the dead file is tracked rather than rediscovered.

Refs #373, #374.


## Comments (1)

### comment 5168661556 — 2026-08-03T15:52:02Z — Casey-allard

## Decision (maintainer-approved): delete

`trace_lane.rs` comes out. Rationale: dead since the restructuring, never compiled, and its kept-textually-current state imposes a phantom tax on every refactor while misleading readers — the exact failure mode that hid the sweep harness' orphaning until #373. Git history preserves the file completely; if the lane-comparison capability is wanted later, revival is the same mechanical move #373 made for the sweep (relocate to `uor-r4-graph-cli`, where its `cover_sweep` dependency now actually lives — its imports already point at a module that has moved out from under it). Deletion PR references the final SHA for easy revival.


---
_Comment/review ids in this file: 5168661556_

