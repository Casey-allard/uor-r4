# ORDER: infra-loop-harness, deepseek-69 (GPU collection liveness heartbeat)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10. Base: `4a388633c`. `References #2904`.

## Goal

The stale-collection rule merged in #3134/#3135 is an **age heuristic**: a record still marked `running` after six hours is offered as stale, because a killed watcher writes no note. But age cannot distinguish a genuinely long live run from a dead watcher. Add the liveness signal the earlier verifier named as the remaining follow-up:

- Each successful lease renewal updates a new `last_renewed_unix` timestamp on the collection record (atomic rewrite, one per renewal tick).
- `collection_is_stale(record, now)` bases staleness on `last_renewed_unix` when present, falling back to `started_unix` for old records without the field: a live watcher keeps a long run out of the stale path, and a killed watcher ages out six hours after its last renewal.
- Both shared registration docs state the heartbeat semantics.
- Tests: the watcher writes the heartbeat; the boundary/staleness logic covers recent renewal with an old start (not stale), old renewal (stale), and missing heartbeat (falls back to start); an old record written without the new field still parses.

The record schema gains an optional field; old records must keep reading. No GPU run, model, bar, hold or owner release is involved.

## Cancel probe (run first)

At `4a388633c`: `grep -n 'last_renewed_unix' crates/uor-r4-harness/src/loopdrive.rs` is empty, and `collection_is_stale` compares `started_unix` only. The task stands. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopdrive.rs`
- `crates/uor-r4-harness/src/loopnext.rs`
- `crates/uor-r4-harness/tests/loop_drive.rs`
- `crates/uor-r4-harness/tests/loop_next_sources.rs`
- `docs/harness/README.md` and `docs/loop/README.md` (shared registration files)
- this task folder

## Done when

- A successful renewal writes `last_renewed_unix`; the final watcher state keeps it; a record parsed without the field has `None` and falls back to `started_unix` for staleness.
- `collection_is_stale` uses the fallback rule; tests pin recent-renewal-with-old-start not stale, old-renewal stale, missing-heartbeat fallback, and the original boundary/edge cases.
- Both shared registration docs describe the heartbeat semantics; `cargo test -p uor-r4-harness` (all targets), `fmt`, `clippy -D warnings`, `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a review records at the head (per the owner direction of 2026-10-24T05:29Z, an independent subagent review is acceptable when other labs are slow).

## Stop and ask if

- A GPU run, model, bar, hold, owner release, a schema-compatibility break or a file outside the list is needed.
