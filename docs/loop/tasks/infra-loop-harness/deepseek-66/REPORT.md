# REPORT: infra-loop-harness, deepseek-66 (repair after Claude's counted CHANGES)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904, setup row 10. Repair for PR #3124 at the new head. `References #2904`.

## What did not work, first

- **Claude's blocker was real and reproduced.** My earlier ORDER made `--collect` a return branch before the lock, but it sat *after* the `--client`/`--model`/adapter/launch checks. `loop drive --collect claude-uorpod-1 --artifact a/b` exited 2 with "needs --client", so the supervisor could offer a collection it could never close, every cycle. The review's evidence was correct; I moved the branch next to `--watch`.
- **My own tests hid it.** They marked records collected through the library (`write_collection`/`collect_collection`), not the CLI. The new test runs the binary with only `--collect/--artifact` and then asserts `loop next` stops offering the record, so the blocker now fails the suite.
- **A record could stay `running` forever.** `watch_lease_collect` wrote the running record and then used `?` on the watcher result, so an error (for example a non-spawnable renew program) returned before the update. It now writes `stopped` with the error as a note, then returns the error; a test pins it.
- **Two low findings fixed:** a same-second duplicate id now gets a numeric suffix rather than sharing one file; the drive tests remove their temp roots.
- **The remaining low finding is a documented limit, not fixed:** the atomic write is temp-file-plus-rename and does not fsync; a power loss can lose the newest record. The collection is local operational state, not a sealed result, and `loop next` would simply not offer it; if it ever needs durability, an fsync belongs in the writer.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | `--collect` handled before the drive-mode checks; `watch_lease_collect` writes `stopped` + note on watcher error; duplicate same-second ids get `-2`... suffixes |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | new test: CLI-only `--collect --artifact` succeeds, marks the record, and `loop next` stops offering it |
| `crates/uor-r4-harness/tests/loop_drive.rs` | new test: watcher error leaves a stopped record, never a running one; temp roots cleaned |
| this folder | repair ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| `loop drive --collect <id> --artifact p` alone | exit 0, record marked collected with the artifact (new test) |
| watcher error | record `stopped` with "watcher error" note, never `running` (new test) |
| `loop next` after collect | no `GO gpu-collection`; "no GPU collection record is waiting" (new test) |
| `cargo test -p uor-r4-harness` | all targets green (see closing line) |

## Checks run at the head

`cargo fmt --check`, `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local`, `loop gates --full` — see the closing line below.

## Skills used

- `solution-integrity`: the reviewer's reproduction was accepted and turned into a test; the failed path is repaired rather than explained away; one finding is left as a stated limit.
- `scope-guard`: the two harness sources and the gated test files only.

## Still open

- **No real GPU collection has been recorded yet**; the next leased run writes the first one.
- **The fsync low finding** is documented above and left for the unit that needs durable records.
- **#3123's process miss** (ORDER without a setup row, merged without a counted record) is recorded on #2028; this repair's order names Setup row: 10 and will not be enqueued before a counted pass.

## Plain summary

- Fixed the blocker the reviewer found: the collect command is now reachable on its own and a CLI-level test proves the record closes; a watcher error can no longer leave a collection running forever.
