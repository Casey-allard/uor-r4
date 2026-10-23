# REPORT: infra-loop-harness, antigravity-32 (fix carded loop_drive parallel-load flake, #3058)

Lab: antigravity. Order: [ORDER.md](ORDER.md). Item: `infra-loop-harness` (#2904). Base: `d99279c42`. `References #3058`.

## What did not work, first

- **Uncapped fake clock after poll 30 plus `root.join("stop")` working-tree pollution (`crates/uor-r4-harness/tests/loop_drive.rs:605–640`):** At poll 30 (`jump = 18,000 s`, 2.5 two-hour windows), `pause` wrote `root.join("stop")` and slept `2 ms` per subsequent poll while waiting for the child shell (`while [ ! -f stop ]; do sleep 0.01; done`) to finish its `10 ms` `/bin/sleep 0.01` and exit. Under full-suite parallel CPU load, `/bin/sleep 0.01` plus process teardown exceeded `6 * 2 ms = 12 ms`, so the supervisor reached poll 36 (`jump = 21,600 s`), opening a third two-hour window where `Observation::take` saw untracked `?? stop` in `git status --porcelain` and emitted `"2-hour check-in: real progress (working tree changed); continuing"`, failing both `notices.iter().all(|note| note.contains("close skipped"))` and `assert_eq!(notices.len(), 2)`. Adding a `40 ms` post-stop teardown delay (`sleep 0.04`) reproduced the failure deterministically (`0 passed; 1 failed` in `0.88 s`).
- **Fix:** Moved the stop sentinel to `root.join("local/loop/drive/stop")` (which `Observation::take` ignores per `loopdrive.rs:136`), capped the fake clock's `+600 s` step at the 30-poll horizon (`18,000 s`, matching `the_fake_clock_continues_a_committing_client_with_a_note` at line 402), and kept the `40 ms` post-stop teardown delay (`sleep 0.04`) in the child shell so the test permanently exercises multi-poll reap latency after poll 30. Removing `next_check = next_check.saturating_add(CHECK_IN_SECONDS)` in `loopdrive.rs:775` still fails the test deterministically (`left: 21, right: 2`).

## What changed

- **Tests (`+9 / -6` lines in 1 file):**
  - [`crates/uor-r4-harness/tests/loop_drive.rs`](../../../../../crates/uor-r4-harness/tests/loop_drive.rs) (`+9 / -6`): Placed the stop sentinel under `local/loop/drive/stop`, capped `jump` at `18_000` (`30 * 600 s`), and retained `sleep 0.04` after the loop so parallel-load reap latency is exercised on every run.
- **Docs (`+66 / -0` lines across 2 files):** Added [`ORDER.md`](ORDER.md) (`+28 / -0`) and [`REPORT.md`](REPORT.md) (`+38 / -0`).

## Measurements

- **Cancel probe (before fix, with `sleep 0.04` teardown delay):** `cargo test -p uor-r4-harness --test loop_drive a_leased_session_is_never_closed_and_notes_exactly_once_per_window` -> `FAILED` (`every note is the lease skip: [..., "2-hour check-in: real progress (working tree changed); continuing"]`).
- **Planted mutation check (with `next_check = next_check.saturating_add(CHECK_IN_SECONDS)` removed from `loopdrive.rs:775`):** `FAILED` (`assertion left == right failed: one skip note per two-hour window, not one per poll: left: 21, right: 2`).
- **With fix (`loop_drive` and full `uor-r4-harness` suite):** `24 / 24` in `tests/loop_drive.rs` (`4.27 s`) and `674 / 674` unit tests + all integration tests in `uor-r4-harness` (`0 failed`).

## Checks run at the head

- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo clippy -p uor-r4-harness --all-targets -- -D warnings` -> `0 warnings`.
- `CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-harness` -> `PASS` (`0 failed`).
- `uor-harness loop report --check --order docs/loop/tasks/infra-loop-harness/antigravity-32/ORDER.md`, `uor-harness loop pr-check --local`, `uor-harness loop gates --full`, and `uor-harness check` -> `PASS` / `GREEN`.

## Skills used

- `solution-integrity`: Diagnosed the exact 12 ms / 6-poll race between `pause`'s `2 ms` sleep and the child shell's `10 ms` `/bin/sleep 0.01` + `root.join("stop")` `git status --porcelain` side effect, and kept the `40 ms` post-stop teardown delay in the test so the race stays permanently covered.
- `scope-guard`: Modified only `crates/uor-r4-harness/tests/loop_drive.rs` and `docs/loop/tasks/infra-loop-harness/antigravity-32/`.

## Still open

- The PM closes card [#3058](https://github.com/UOR-Foundation/uor-r4/issues/3058) after merge, and umbrella item [`docs/roadmap/items/infra-loop-harness.md`](../../../../roadmap/items/infra-loop-harness.md) ([`#2904`](https://github.com/UOR-Foundation/uor-r4/issues/2904)) stays `status: ready` until every row in the loop build plan is complete. References #2904, #3058.

## Plain summary

This task (`antigravity-32`) fixes the parallel-load flake in `crates/uor-r4-harness/tests/loop_drive.rs::a_leased_session_is_never_closed_and_notes_exactly_once_per_window` reported in card `#3058`. When full-suite CPU load delayed the child shell's `sleep 0.01` exit by more than `12 ms` (`6` parent polls of `2 ms`) after poll 30 wrote `root.join("stop")`, the uncapped fake clock reached poll 36 (`21,600 s`, a third two-hour window) and `Observation::take` saw `?? stop` in `git status --porcelain`, emitting a `"working tree changed"` progress note. Moving the stop sentinel into `local/loop/drive/stop` (ignored by `Observation::take`), capping the fake clock at the 30-poll horizon (`18,000 s`), and keeping a `40 ms` post-stop teardown delay makes the test deterministic under arbitrary parallel load while preserving the `next_check` mutation failure (`left: 21, right: 2`).
