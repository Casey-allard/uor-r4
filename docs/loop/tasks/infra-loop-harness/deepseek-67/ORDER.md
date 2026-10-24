# ORDER: infra-loop-harness, deepseek-67 (stale GPU collection records)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10. Base: `38bec04ba`. `References #2904`.

## Goal

The new `gpu-collections` source (merged in #3124) offers a finished uncollected record and waits on any record still marked `running`. Claude's review of #3124 noted the remaining hole: **a killed watcher leaves no note**, so such a record stays `running` forever, the source reports "running; none is finished and uncollected", and the later sources (premise probes, lane, `any`) are starved. Add a bounded staleness rule:

- A `GpuCollection` still `running` with `started_unix` older than `STALE_COLLECTION_SECS` (six hours) is **stale**: `loop next` offers it as `GO gpu-collection <id>` with a reason saying it is stale and the pod may be gone, so a session can collect it or mark it collected with `loop drive --collect`.
- A record started within the window still waits, as before.
- Both registration docs state the rule.
- Host tests cover the stale offer, the fresh wait, and a mutation (removing the stale predicate) that turns the stale test red.

No GPU run, model, bar, hold or owner release is involved.

## Cancel probe (run first)

At `38bec04ba`: `grep -n 'STALE_COLLECTION_SECS' crates/uor-r4-harness/src/loopnext.rs` is empty, and the gpu-collections block offers only `record.status != "running"` while counting every running record as a wait. The task stands. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopnext.rs`
- `crates/uor-r4-harness/tests/loop_next_sources.rs`
- `docs/harness/README.md` and `docs/loop/README.md` (shared registration files)
- this task folder

## Done when

- `STALE_COLLECTION_SECS` is a documented public constant; a running record older than it is offered as stale work (not a wait), and a recently-running record still waits.
- Tests in the gated `loop_next_sources` suite pin: stale offered with a stale reason; fresh running waits with the running count; the mutation removing the stale predicate turns the stale test red (verified by hand in the REPORT).
- Both shared registration docs describe the stale rule; `cargo test -p uor-r4-harness` (all targets), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a counted other-vendor exact-head review records at the head.

## Stop and ask if

- A GPU run, model, bar, hold, owner release or a file outside the list is needed; a heartbeat-style liveness signal would replace the age heuristic (a larger change, not this unit).
