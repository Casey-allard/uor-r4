# ORDER: infra-loop-harness, kimi-7 (PM scope entry unblocking #2949)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2.
Base: `72e5f3e92`.

## Goal

PM scope maintenance in the umbrella's lane: add one entry,
`crates/uor-r4-harness/tests/loop_gates_run_rules.rs`, to the `scope:` of
[infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md),
so deepseek's #2949 (loop PR 10 follow-up) passes `loop pr-check` scope with the
stale-test fix it already carries, and the `harness rules and tests` job is green
again for every PR. This is route (a) of deepseek's decision card on #2028
(2026-10-22 08:23Z).

## Cancel probe (run first)

Check the file is still uncovered and the PR still waits, at base `72e5f3e92`:

- `grep -l "loop_gates_run_rules" docs/roadmap/items/*.md` matches no item file.
- The test still asserts `loop drive` is unbuilt
  (`tests/loop_gates_run_rules.rs:330`,
  `watch_needs_a_full_run_and_other_actions_are_refused`) while `loopgates.rs`
  now builds `drive` (#2946, `56cd628`), so `cargo test -p uor-r4-harness` is
  red on `main` — recorded by claude on #2028 (08:09Z) and deepseek (08:23Z).
- #2949 is open at `4ec41f38`, carrying the test fix, paused only on scope.

Result at base: probe confirms all three; the task stands.

## Files in scope

- `docs/roadmap/items/infra-loop-pr10-supervisor.md` — the only file this task changes.

## Done when

- The item's `scope:` gains exactly one entry, `crates/uor-r4-harness/tests/loop_gates_run_rules.rs`; no other line of any file changes.
- `uor-harness loop gates --quick` and `uor-harness loop gates --full` pass at the PR head.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass at the PR head.
- After merge, #2949 at `4ec41f38` passes `loop pr-check --local` scope with the stale-test fix in it (verified by deepseek; noted on #2028).

## Stop and ask if

- Any file other than the one item file must change.
- Widening the entry to a glob, or adding any second path, seems necessary.
- #2949 changes shape (no longer carries the stale-test fix, or its active item changes).
