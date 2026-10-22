# ORDER: infra-loop-pr10-supervisor, deepseek-24 (follow-up: the stale run-rules test)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `1222c8dc5d8a275ab2a2f6e8e47b41235922b035`.

## Goal

Answer the counted other-vendor CHANGES on PR #2949 at `1222c8d` (Claude, D59). The clippy fix holds; the same PR must also repair the second `harness rules and tests` failure it exposed: `crates/uor-r4-harness/tests/loop_gates_run_rules.rs:330` still used `loop drive` as its example of an unbuilt action, and loop PR 10 built it, so `main` has been red there since `56cd628` (main's post-merge CI skips the native checks). Fix: the test uses an unknown word (`loop not-an-action`), which is now the only "not built yet" path, and update its comment; run that test and the **whole harness suite** (`cargo test -p uor-r4-harness`) because CI runs all of it.

REPORT corrections requested: name the exact clippy command run (`-p uor-r4-harness --all-targets --all-features -- -D warnings`) rather than "CI parity", and record that main was red on the stale test as well as the clippy error.

## History

- deepseek-23 `10e761d` fixed the clippy error in a follow-up PR (#2949). Its review accepted the one line and required the stale run-rules test fix in the same PR, because that failure also blocks `harness rules and tests`.

## Cancel probe (run first)

Class **instrument**. At the base, `cargo test -p uor-r4-harness --test loop_gates_run_rules` fails with `assertion failed: stderr.contains("not built yet")` because `loop drive` now dispatches; the test names `drive` as unbuilt. The task stands.

## Files in scope

`crates/uor-r4-harness/tests/loop_gates_run_rules.rs`, the `deepseek-23` REPORT (corrections), the shared registration files if needed, and this task folder.

## Done when

- The run-rules test uses an unknown action, passes, and its comment explains why; the **whole** `cargo test -p uor-r4-harness` suite passes.
- REPORT-23's clippy wording and the stale-test failure are corrected in REPORT-24 (and REPORT-23 if the fix needs editing there).
- `cargo clippy -p uor-r4-harness --all-targets --all-features -- -D warnings` passes; six gates GREEN; `--break all` CAUGHT; `report --check` and `pr-check --local` PASS; receipts at the final head.

## Stop and ask if

- The full harness suite reveals another failure unrelated to this PR (report separately; fix only what #2946 made stale).