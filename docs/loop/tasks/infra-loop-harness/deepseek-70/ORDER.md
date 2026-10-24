# ORDER: infra-loop-harness, deepseek-70 (repair after the counted CHANGES on #3137)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10. Base: `4a388633c`. Repair for PR #3137.

## Why this repair

The independent subagent review at `1bbff7f20` returned a counted **CHANGES** with one blocking consistency finding and one minor: the `gpu-collections` wait count still compared `started_unix` against the six-hour window, so a record started seven hours ago but renewed one hour ago was correctly not offered as stale — yet also not counted as running, and the source printed "no GPU collection record is waiting", falling through to premise/lane while a live GPU run existed. The minor: the stale reason claims to name the last renewal epoch but printed only the start epoch.

## Finished steps (past tense, with their commits)

- **Built the liveness heartbeat** (`1bbff7f20`): optional `last_renewed_unix` written on each successful renewal; `collection_is_stale` uses it with fallback to `started_unix`; synchronous `on_renewed` callback; docs; tests for watcher heartbeat, old start + recent renewal, old renewal, missing heartbeat and old-record compatibility.

## Fix

1. **Wait count:** count `running` uncollected records by `!collection_is_stale(record, now)` instead of `started_unix >= stale_before`, so a live long run is counted while a stale one is offered; remove the now-unused `stale_before`.
2. **Reason:** include `last renewal: <epoch or none>` so the message matches what it claims and a session can judge liveness.
3. **Source-level test:** a record started seven hours ago with a heartbeat one hour ago produces no `GO gpu-collection` and prints "1 GPU collection(s) running".

## Files in scope

- `crates/uor-r4-harness/src/loopnext.rs`
- `crates/uor-r4-harness/tests/loop_next_sources.rs`
- this task folder (the deepseek-69 folder stays immutable history)

## Done when

- The wait count and the stale offer use the same staleness predicate; a recently-renewed old running record is not offered and is counted as running; a stale record is still offered.
- The reason text includes the last renewal epoch when present.
- New source-level test passes; `cargo test -p uor-r4-harness` (all targets), `fmt`, `clippy -D warnings`, `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a fresh independent review records at the new head (owner direction of 2026-10-24T05:29Z).

## Stop and ask if

- A GPU run, model, bar, hold, owner release or a file outside the list is needed.
