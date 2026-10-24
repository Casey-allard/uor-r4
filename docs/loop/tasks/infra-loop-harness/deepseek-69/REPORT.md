# REPORT: infra-loop-harness, deepseek-69 (GPU collection liveness heartbeat)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904, setup row 10. Base: `4a388633c`. `References #2904`.

## What did not work, first

- **The first field insertion landed in the wrong parser.** The new `last_renewed_unix` read was added to the `SessionRecord` parser (the same `started_unix: value.get(...)` shape) instead of the collection parser, so the crate failed to compile (E0560/E0063). Caught by the first test run; the line was moved into `collection_from_value` and the session parser restored.
- **The liveness signal is only as good as the watcher's renewal loop.** A watcher that is alive but whose renew command hangs produces no tick; the record then ages out after six hours even though a process exists. That is the same failure direction as before (offer stale work rather than wait forever), and the reason text names the last renewal epoch so a session can judge it.
- **Backward compatibility is explicit rather than assumed:** the new field is optional on read and a record written before this unit parses with `None`, tested with a hand-written old JSON.
- **Docs are the shared registration files:** both the loop page and the harness README state that renewals extend liveness, so the earlier "after six hours" wording is not left stale.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | `GpuCollection.last_renewed_unix` (optional, written/read in the record JSON); `watch_lease_until_renew_fails_with` takes an `on_renewed` callback (old function delegates with a no-op); `watch_lease_collect` updates the heartbeat each renewal |
| `crates/uor-r4-harness/src/loopnext.rs` | `collection_is_stale` uses `last_renewed_unix` with fallback to `started_unix` |
| `crates/uor-r4-harness/tests/loop_drive.rs` | watcher-heartbeat assertion; old-record-without-field compatibility test |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | boundary test extended: old start + recent renewal not stale; old renewal stale; no heartbeat falls back to start |
| `docs/harness/README.md`, `docs/loop/README.md` | heartbeat semantics stated |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| watcher writes heartbeat | `a_lease_watcher_writes_and_updates_a_record_and_collect_marks_it` asserts `last_renewed_unix.is_some()` |
| old record without field | parses, `None`, falls back to start |
| staleness with heartbeat | 10 h start + 1 h renewal → not stale; 7 h since renewal → stale; missing heartbeat + 1 h start → not stale |
| `loop_next_sources` / `loop_drive` / `loop_next` | 13 / 28 / 33 passed, 0 failed |
| full harness suite, clippy, fmt | green (see closing line) |

## Checks run at the head

`cargo fmt --check`, `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local`, `loop gates --full` — see the closing line.

## Skills used

- `solution-integrity`: the heuristic was replaced with a real liveness signal, compatibility was tested, and the remaining failure direction is stated.
- `scope-guard`: two harness sources, two gated test files, two shared registration docs, task folder.

## Still open

- **The heartbeat does not survive a watcher that is alive but blocked** (no tick); the six-hour bound still offers the record, which is the safe direction.
- **No real GPU collection recorded yet**; the first leased run will exercise the heartbeat.

## Plain summary

- GPU collection records now carry the time of their last successful lease renewal, and staleness uses that instead of the start time, so a live long run is not declared stale while a killed watcher still ages out after six hours; old records fall back to their start.
