# ORDER: infra-loop-harness, claude-23 (loop next skips items with unfinished dependencies)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7b. Base: `a2ada4542`.

## Goal

`loop next` (step 1 of [the loop page](../../../README.md)) offers the first ready, scoped item that `loop claim` accepts, and it ignores the item's `depends_on:`. On 22 Oct it offered `d11-exporter-float-trunk`, whose `depends_on` includes `owner-decision-float-path-scope` (#2875, an open owner decision), and Kimi and DeepSeek both had to set it aside by hand.

Casey's direction, in Claude's chat on 22 Oct: "Yes: build the loop next fix (skip items whose depends_on is not done) through the delivery path."

The planted fault: **an item whose `depends_on` names an item that is not `status: done` is never offered.**

## Cancel probe (run first)

At `a2ada4542`, does `loop next` read `depends_on`? No. `ItemFields` has no such field, and the `any` loop checks only status, scope and `loop claim`. The task stands. Class: instrument.

## Calls

- `ItemFields` gains `depends_on`, parsed from the same inline `[a, b]` form as `scope:`.
- In the `any` source, an item counts as waiting while any of its dependencies is missing from `origin/main` or is not `status: done`. A waiting item is never offered, and the `any` detail names it with what it waits on.
- `loop claim` is unchanged. Whether it should refuse such an item is a separate call.

## Files in scope

- `crates/uor-r4-harness/src/loopnext.rs` and `src/loopclaim.rs` (the parser only), and `crates/uor-r4-harness/tests/loop_next*.rs`
- `docs/harness/README.md` (the `loop next` paragraph)
- `docs/loop/tasks/infra-loop-harness/claude-23/**`

## Done when

- A ready, scoped item with an unfinished dependency is not offered; with the dependency `done`, it is.
- A missing dependency counts as unfinished.
- The full harness suite passes, as do `pr-check --local`, claim-wording, links and the copyright check.

## Stop and ask if

- `loop claim` or another tool would have to change.
- Any file outside scope must change.
