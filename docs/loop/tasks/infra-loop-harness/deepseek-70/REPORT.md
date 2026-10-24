# REPORT: infra-loop-harness, deepseek-70 (repair after the counted CHANGES on #3137)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904, setup row 10. Repair for PR #3137.

## What did not work, first

- **The reviewer's blocking finding was correct.** The wait count was still a second, inconsistent staleness computation based on `started_unix`. My tests only exercised `collection_is_stale` directly, so the source-level contradiction was invisible: a seven-hour-old run renewed an hour ago was not offered (right) but was reported as none (wrong), letting `loop next` fall through to the premise probe while a GPU run was live. Fixed by using the same `!stale(record)` predicate as the offer and adding a source-level test.
- **The minor was real.** The reason claimed to name the last renewal epoch but printed the start epoch. Fixed by including `last renewal: <epoch or none>` in the reason rather than weakening the REPORT claim.
- **The heartbeat remains an age bound under a blocked watcher:** a live watcher whose renew command hangs writes no tick, so the record eventually ages out. That is the safe direction (offer rather than wait forever) and is unchanged.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopnext.rs` | wait count uses `!stale(record)` (uncollected, running); `stale_before` removed; stale reason now names the last renewal epoch when present |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | source-level test: old start + recent heartbeat -> no GO, "1 GPU collection(s) running" |
| this folder | repair ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| renewed-7h-old live record | no `GO gpu-collection`, prints "1 GPU collection(s) running" (new test) |
| stale record | still offered as `GO gpu-collection` (existing test) |
| reason text | includes last renewal epoch or `none` |
| `loop_next_sources` / `loop_drive` / `loop_next` | 14 / 28 / 33 passed; see closing line for the full suite |

## Checks run at the head

`cargo fmt --check`, `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local`, `loop gates --full` — see the closing line.

## Skills used

- `solution-integrity`: the second staleness computation was the defect the direct tests missed; the repair makes the offer and the wait share one predicate and tests the source path.
- `scope-guard`: one source, one gated test file, task folder.

## Still open

- **The heartbeat cannot detect a live-but-blocked watcher** (no tick); the six-hour bound remains.
- **No real GPU collection recorded yet.**

## Plain summary

- The reviewer was right: a live long GPU run was counted as nothing. The wait now uses the same staleness predicate as the offer, the reason names the last renewal, and a source-level test covers the case.
