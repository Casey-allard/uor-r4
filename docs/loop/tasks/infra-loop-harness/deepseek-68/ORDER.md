# ORDER: infra-loop-harness, deepseek-68 (tidy the gpu-collections source after post-merge verification)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10. Base: `99cdddf60`. `References #2904`.

## Goal

A fresh-context adversarial verifier confirmed #3134's stale-collection behavior but left four non-blocking notes. This unit fixes the three that are code/tests:

1. **The stale reason overclaims.** It says "no live watcher", which age alone cannot prove (a genuinely long live run would be mislabeled) and prints a raw epoch unlabeled. Say "has been running since epoch N and is treated as stale; collect it or mark it collected if its watcher is gone".
2. **A stale comment.** The gpu-collections comment still says "Running records wait", which is no longer the whole rule; state the stale case.
3. **The wait count includes collected records.** A collected-but-recently-running record makes the source report a wait; count only uncollected recent running records.
4. **Boundary semantics are unpinned.** The strict `<` at the six-hour boundary is only exercised through a live clock. Extract `collection_is_stale(record, now)` and unit-test it clock-free (exactly six hours is not stale, one second more is stale, future start not stale, epoch 0 stale without underflow, only `running` records stale).

The fourth verifier note (a liveness heartbeat instead of an age heuristic) stays a named follow-up, not this unit. Behavior for the existing stale-offer path is otherwise unchanged; docs stay accurate.

## Cancel probe (run first)

At `99cdddf60`: `grep -n 'is stale running' crates/uor-r4-harness/src/loopnext.rs` finds the overclaiming reason; the wait filter has no `!record.collected`; and no `collection_is_stale` helper exists. The task stands. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopnext.rs`
- `crates/uor-r4-harness/tests/loop_next_sources.rs`
- this task folder

## Done when

- The stale reason names the epoch and says the record is treated as stale (no "no live watcher" claim); the comment states the stale rule; the wait count excludes collected records.
- `collection_is_stale` is public, used by the source, and unit-tested on the exact boundary and edge values without a clock race; the existing stale-offer and fresh-wait tests still pass.
- `cargo test -p uor-r4-harness` (all targets), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a counted other-vendor exact-head review records at the head.

## Stop and ask if

- A GPU run, model, bar, hold, owner release or a file outside the list is needed; the heartbeat follow-up would require changing the watcher contract.
