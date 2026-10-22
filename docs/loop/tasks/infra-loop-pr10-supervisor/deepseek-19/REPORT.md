# REPORT: infra-loop-pr10-supervisor, deepseek-19 (repair: live check-in, safe WIP, guarded signalling)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 10. Base: `13430063e06a3e05fbb09836c38e4df4ec9c0b41`. This is a loop setup PR: its exact-head review comes from Claude's fresh-context reviewers (D59).

## What did not work, first

The review at `13430063e` was right about the central defect, and the repair found two more along the way:

- **The check-in ran only after the client exited**, so a stalled session was never stopped and every long session looked stalled. Fixed: the session loop now takes the before-observation at launch, polls the live child with `try_wait`, uses an injected clock, and at 7200 s classifies live progress. Only a session classified Stalled is terminated; a Progressing one keeps running with a note. The previous-stall count is carried in so the second consecutive stall is spiraled. The fake-clock tests pin both the closed-and-carded silent client and the continues-with-a-note committing one.
- **The WIP close could stage anything and push anywhere.** Fixed: it refuses a branch that is not `<lab>/*` or is `main`, requires `--order`, stages only that ORDER's task folder (refused if the folder is not `<lab>-<n>`), writes the `Order:` trailer, and strips `GIT_*` from every git child.
- **Signals needed guards before the close path could call them.** Fixed: `terminate_session` refuses pgid 1 or less, a pgid that is not the recorded pid, and the supervisor's own group; `session_alive` now checks pid **plus** the recorded `ps -o lstart=` identity, so a reused pid cannot be signalled and the watchers cannot renew a lease for a dead session. `signal_group` refuses pgid 1/0 as a last gate.
- **A spawn whose record write failed left an untracked child.** Fixed: the child's own group is killed and the error returned; there is no retry that could start a second session.
- **The old test raced a fast child.** Fixed with a sleeping child and no kill-0 race; a record read is asserted before the wait.
- **The planted fault only exercised a match arm.** Fixed with CLI-level fake-clock tests that drive `run_sessions` with a fake card sink: a silent client is **closed and carded**, a committing one continues with a note. The gate's `--break` still goes red with the item's own sentence.
- **Two diagnostics I hit while fixing it** may help the next lab: the session's own log/record under `local/loop/drive/` first counted as "working tree changed" (fixed by ignoring the drive directory and by the test repo's `local/` ignore), and the progressing test was timing-dependent until the fake clock waited for the client's commit marker.
- **LOW items are covered:** the per-lab O_EXCL `drive.lock`; `{prompt}` refused as argv[0]; `--interval` floor 1; `--prompt-file` parsed; a fresh Go per session prompt; the record-root comment; the PM card names `close-*/` and skips a duplicate head+reason; `gates.json` newline; the README says unix-only; receipts at the final head; this REPORT corrects the earlier claims.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | code | rewrote ~1,150 | live check-in loop (`run_sessions` + `DriveSpec`/`DriveHooks`), identity-checked liveness and signal guards, scoped WIP commit, card dedupe, per-lab lock, new CLI flags |
| `crates/uor-r4-harness/tests/loop_drive.rs` | test | 12 tests | planner, real process round-trips, both watchers, PM card wording, close artifacts, and the two fake-clock CLI-level tests |
| `crates/uor-r4-harness/src/doccheck.rs` | code | +3 | `--prompt-file`, `--exempt-lease`, and the count |
| `docs/harness/gates.json` | generated | newline | kept the gate, restored the trailing newline |
| `docs/harness/README.md` | docs | +6 | `--order`, `--exempt-lease`, unix-only, `--prompt-file`, `--watch --lease` |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-19/{ORDER,REPORT}.md` | docs | small | this repair order and report, plus receipts |

**Size:** one module rewritten around the live session loop plus its tests; the pure planner functions are unchanged in behaviour.

## Measurements

- **`cargo test -p uor-r4-harness --test loop_drive`:** 12 passed, 0 failed — including the fake-clock silent client (code 1, one card beginning `stalled:`, a close record written) and the fake-clock committing client (code 0, no card, no close record).
- **Gates:** six GREEN at the code head `ca669b4`; `--break all` CAUGHT on six, `loop-drive` red with **"a fake stalled client past 2 hours is closed and carded"**.
- **`uor-harness check`:** no doccheck/flag error at the head; the documented flags resolve.
- **The previous REPORT's false claims were corrected rather than repeated:** the check-in runs live now, and the measurements above are the ones the gate and the test file produce at this head.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: GREEN (`fmt-harness`).
- `UOR_LAB=deepseek uor-harness loop gates --full`: six GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: six CAUGHT, `loop-drive` with the item's sentence.
- `cargo test -q -p uor-r4-harness --test loop_drive`: 12 passed.
- `loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: the review's reproduction was treated as the specification; the fix changes the loop structure rather than adding a branch, and the two new test defects were fixed at their cause.
- **scope-guard**: the module, its tests, the gate, the shared registration files and the two docs only.
- **holding-the-goal**: no blocker was claimed; the two timing diagnoses were fixed instead of skipped.

## Still open

- **A real Claude Code after-stop run** is still structural only: the supervisor waits on its tracked child and never lists other processes, but the exclusion has not been exercised against that client.
- **The lease renew command** is the caller's `uor-pod renew`; the supervisor never renews a lease itself and never cancels a run.
- **`loop render --pr` and the per-client compaction hooks** remain separate build-plan rows (P-1: no deepseek hook).
- **Live close actions** (WIP push, PM card post) are best effort and not exercised in CI; the wording, scoping and refusal rules are tested.

## Plain summary

- **The 2-hour rule now acts while the session is alive.** A session with commits or gate activity keeps running with a note; a silent one is terminated at the check-in and its close record, WIP commit and PM card are written. The fake-clock tests exercise both outcomes, and the planted fault still turns the gate red with the item's own sentence.
- **The close is safe:** it refuses to commit on `main` or a foreign branch, stages only the active ORDER's folder with its trailer, strips inherited git state, and signals only a process whose pid and start identity match the record.
- **The watcher and lock close the remaining edges:** liveness is pid plus start identity, a second drive for one lab is refused, and a leased run is never closed or renewed by the wrong process.