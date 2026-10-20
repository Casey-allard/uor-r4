# #924 — feat(#843): the deployed bounded planner, allocation-free and P-4 only

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-22T16:06:17Z
- **Updated:** 2026-08-22T17:39:26Z
- **Closed:** 2026-08-22T16:50:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/924
- **Merged:** 2026-08-22T16:50:15Z
- **Merge commit:** 978ecb0781ade7f8d59ee9cbf0b10a094f89f3ef
- **Base / Head:** main / UOR-Foundation:issue-843-deployed-planner
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem and outcome

Increment 5 of 6 for issue #843 (item B of S4 tracker #826). Adds the normative deployed-serving planner of `docs/bounded_semantic_transitions_spec_843.md` §6, wires it into the #831 `R4Engine` path, and emits the self-contained witness of §7 into caller-owned bytes.

No capability is claimed here — that is increment 6's measurement. What is established is that the machinery is total, deterministic, bounded, allocation-free, P-4-only, and honest at every boundary.

## Scope and non-goals

**Three bounded strategies over one search core.** `BreadthFirst` and `BestFirstBeam` are the same layered sweep differing only in which candidates survive the frontier bound; `IterativeDeepening` trades repeated expansions for depth-bounded memory. They read the same sections, execute the same operation set and run under the same `PlanBudget` — so a difference between them is a difference of *search order* and nothing else, which is what makes the §8 comparison a comparison of mechanism rather than of effort.

`R4Engine::plan_bounded` resolves `PSCH`/`PTRN`/`PGOL` from the artifact. **Non-goals:** the arm/null measurement, the three-way differential and the verdict — increment 6.

## Acceptance-criteria status

Advances *"the production path meets fixed horizon/frontier, bytes, latency, P-4, allocation, no_std, and determinism budgets"* — every clause of it is asserted below rather than described. The interventional comparison and the witness-replay-at-scale criteria belong to increment 6.

## Tests and verification

11 tests in `crates/uor-r4-graph-runtime/tests/plan_843.rs`, 2 in `plan_allocation_census_843.rs`. Full local ladder: fmt, clippy `-D warnings`, `cargo test --workspace --no-fail-fast` (183 suites), the `no_std` ladder both ways, the `wasm32-unknown-unknown` lib build, `xtask validate` (R1/R4/R5/gnaf, 32 ids), `repo-conformance` R2/R3, `cargo deny check bans`, and the claim-wording gate.

## Evidence artifacts

**Allocation-free, measured.** A counting global allocator reports **0 allocations, 0 bytes** for a whole planning episode *and* **0 allocations, 0 bytes** for emitting its witness. The witness encoder was lifted into `core` for exactly this reason: the offline and deployed producers now share one encoder, and the deployed one writes into a caller-provided buffer.

**P-4, machine-checked at source.** A scan of the planner's own source finds no float type, no `sqrt`/`pow`, and **no runtime multiply or divide**. Six compile-time constant expressions — array lengths and budget constants, all with SCREAMING_SNAKE or literal operands — are exempted and **printed by the test**, so the exemption is visible rather than silent: a constant expression emits no instruction and is not a deployed-kernel operation. The scan asserts it can still fire on a runtime multiply between two non-constant operands, so it is evidence rather than decoration.

**Bounded lookup.** The visited set is an open-addressed probe over `2 × PLAN_VISITED_MAX` slots with a checked bound of 16, hashed by the multiply-free add/rotate/xor mixer already normative for the packed skip-mix tables — unseeded, so identical inputs hash identically on every platform. A membership test costs at most sixteen fixed-offset reads whatever the set holds, and exceeding the bound is `Decline(capacity)` rather than a degrade into an unbounded scan.

**Every boundary is a typed decline, never a truncated plan.** An exhausted expansion budget, a horizon or frontier past the frozen capacity, a full visited set, an arity mismatch, a forbidden initial state and an unreachable goal each decline with their own reason, and a decline emits no plan. Each has its own test.

## Negative and unavailable results

**This document's own estimate was wrong, and is corrected rather than accommodated.** §4.2 said the caller-owned scratch was "measured at ≤ 64 KiB" — written before the structure existed. `size_of::<PlanScratch>()` is **81 548 bytes** (≈ 79.6 KiB): visited nodes ≈ 48 KiB, the considered-candidate buffer 16 KiB, the open-addressing index 8 KiB, the rest frontier, scores and path. **No frozen capacity was shrunk to make the old number true.** The spec is corrected, and a test asserts the size against a declared envelope so it cannot drift unnoticed again.

## Compatibility and migration

Additive. One new public module, one new `R4Engine` method, two new `const`s on existing format types. `plan_bounded` returns `Ok(None)` when the artifact carries no planning section — so every artifact produced before #843 serves exactly as before, which is what absent-section identity means — and `Err` when a planning section is present but is not a product of the bytes, so an episode fails closed rather than planning on a partially-read table.

The only non-additive change is internal: `WitnessDraft` and the witness encoder moved from the `alloc`-gated builder module to `core`, and `build_witness` now delegates to them. The public signature is unchanged and the 16 format tests pass untouched.

## Conformance effects

None. `CONFORMANCE.md` is byte-identical at 32 ids and no RF id is added. RF-33 stays unregistered unless increment 6 actually lowers an arm.

## Claim-boundary changes

None. The planner exists and is bounded; whether it *plans better than a null* is not asserted anywhere in this diff.

## Intentionally excluded

No generated file, model, cache, credential or user-owned file is staged. The maintainer's day-branch checkout is untouched; all work is in a dedicated worktree.

References #843


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

