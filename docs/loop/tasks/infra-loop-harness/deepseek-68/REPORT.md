# REPORT: infra-loop-harness, deepseek-68 (tidy the gpu-collections source)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904, setup row 10. Base: `99cdddf60`. `References #2904`.

## What did not work, first

- **The first run failed its own existing test.** Changing the stale reason from "is stale running (… no live watcher)" to "is treated as stale" broke `a_stale_running_record_is_offered_not_waited_on_forever`, which asserted the old phrase. The test was updated to the new wording — the behavior it pins (a stale record is offered) is unchanged.
- **The boundary cannot be tested through the binary without a race.** `loop next` calls `unix_now()` at invocation, so a test that starts a record "exactly six hours ago" can be one second stale by the time the binary runs. The rule was extracted into a pure `collection_is_stale(record, now)` and unit-tested with a fixed `now`; the binary-level tests still cover the offered/waiting paths.
- **One verifier note is deliberately not fixed here.** Replacing the age heuristic with a liveness heartbeat needs a watcher-contract change (the watcher writes a tick file each interval); it stays a named follow-up in #3134's REPORT.
- **The docs need no change:** they already say "a record still running after six hours is treated as stale", which matches the reworded code.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopnext.rs` | `collection_is_stale` (public, clock-injectable) used by the source; stale reason names the epoch and says "treated as stale" without claiming there is no watcher; the comment states the stale rule; the wait count ignores collected records |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | new clock-free boundary test (six hours not stale, +1 s stale, future not stale, epoch 0 stale, non-running not stale); stale-offer test assertion updated to the new wording |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| `loop_next_sources` | 13 passed, 0 failed (12 + new boundary test) |
| `loop_next` | 33 passed, 0 failed |
| boundary table | `now-21600` not stale; `now-21601` stale; future not stale; `0` stale without underflow; finished not stale |
| behavior | stale record still offered with a `GO gpu-collection`; recent running still waits; collected records no longer counted as waits |

## Checks run at the head

`cargo fmt --check`, `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local`, `loop gates --full` — see the closing line.

## Skills used

- `solution-integrity`: the verifier's overclaim became a precise reason; the boundary is tested where it is deterministic instead of flaking through a clock; remaining note left as a stated follow-up.
- `scope-guard`: one source, one test file, task folder.

## Still open

- **Liveness heartbeat** (watcher writes a tick each interval) would replace the age heuristic; named follow-up.
- **No real GPU collection recorded yet**; the first leased run writes one.

## Plain summary

- Corrected the stale-collection message and comment, stopped counting already-collected records as waits, and pinned the six-hour boundary with a clock-free unit test.
