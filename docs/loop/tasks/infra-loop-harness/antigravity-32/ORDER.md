# ORDER: infra-loop-harness, antigravity-32 (fix carded loop_drive parallel-load flake, #3058)

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10 (`loop drive` test suite in `crates/uor-r4-harness/tests/loop_drive.rs`). Base: `d99279c42`. `References #3058` (the PM closes it after merge).

## Goal

Fix the parallel-load flake in `crates/uor-r4-harness/tests/loop_drive.rs::a_leased_session_is_never_closed_and_notes_exactly_once_per_window` (#3058):
1. **Why it flakes under parallel load:** At poll 30 (`jump = 18,000 s`, 2.5 two-hour windows), `pause` writes `root.join("stop")` and sleeps `2 ms` per subsequent poll while waiting for the child shell (`while [ ! -f stop ]; do sleep 0.01; done`) to finish its `10 ms` `/bin/sleep 0.01` and exit. Under full-suite parallel CPU load, `/bin/sleep 0.01` plus process teardown exceeds `6 * 2 ms = 12 ms`, so the supervisor reaches poll 36 (`jump = 21,600 s`), opening a third two-hour window where `Observation::take` sees untracked `?? stop` in `root` and emits a `"real progress (working tree changed)"` note (or a third skip note).
2. **Fix:** Place the stop sentinel inside `local/loop/drive/stop` (ignored by `Observation::take`) and cap the fake clock's `+600 s` advance once the 30-poll test window (`18,000 s`) has completed while the supervisor waits for the child process to reap, and verify both:
   - a child that takes `40 ms` (`20` extra `2 ms` polls) to exit after `stop` is written still records `notices.len() == 2` and `all("close skipped")`;
   - removing `next_check = next_check.saturating_add(CHECK_IN_SECONDS)` in `loopdrive.rs:775` still fails the test (`notices.len() == 19 != 2`).

## Cancel probe (run first)

Class instrument (harness test; no GPU). Adding a `40 ms` post-stop teardown delay (`sleep 0.04`) to the child shell in `a_leased_session_is_never_closed_and_notes_exactly_once_per_window` reproduces the parallel-load failure deterministically (`every note is the lease skip` / `notices.len() > 2`) before the fix.

## Files in scope

- `crates/uor-r4-harness/tests/loop_drive.rs`
- `docs/loop/tasks/infra-loop-harness/antigravity-32/**`

## Done when

- `cargo test -p uor-r4-harness --test loop_drive` and full `cargo test -p uor-r4-harness` pass deterministically (including with post-stop teardown delay and under parallel load), mutation check (`next_check` advance removed) fails as expected, and `uor-harness loop pr-check --local` and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside `crates/uor-r4-harness/tests/loop_drive.rs` and this task folder must change.
