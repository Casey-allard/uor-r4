# ORDER: infra-loop-pr10-supervisor, deepseek-23 (follow-up: the merged clippy error)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `56cd6288488c4f34c2d1fce0ddb3ff6ade47594b`.

## Goal

Repair the one defect the merged PR #2946 left in `main`: the pull-request CI job `harness rules and tests` runs `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and the merged `signal_group` used `&format!("--")`, which clippy denies as `useless use of format!`. `main`'s own CI skips the native checks, so the merge stayed green while every later PR's clippy would fail. Fix the line (use `"--"`), prove CI parity locally with the same clippy command, and land a follow-up PR that `References #2915` (the item is already `done`; this PR does not reopen it).

## History

- deepseek-18..22 built and repaired `loop drive`; PR #2946 merged as `56cd62884` with a counted Claude APPROVE at `5ad07edb5`. The merge-head PR CI was red on clippy only; the merged content is otherwise verified (state/squash/content).

## Cancel probe (run first)

Class **instrument**. At the base, `cargo clippy -p uor-r4-harness --all-targets --all-features -- -D warnings` fails with `useless use of format!` at `loopdrive.rs:473`; the line is a string literal formatted as `--`. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, the shared registration files if needed (`src/lib.rs`, `src/main.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`) and this task folder.

## Done when

- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes locally (CI parity), and the signal helper still sends the same argv (`kill -TERM -- -pgid`).
- The `loop-drive` gate still passes and its planted fault is CAUGHT; six gates GREEN; `report --check` and `pr-check --local` PASS; receipts at the final head.
- A follow-up PR `References #2915` with a counted other-vendor record (D59: Claude's fresh-context reviewers) merges, verifies on fresh `main`, and its branch/worktree are removed.

## Stop and ask if

- The clippy parity run reveals a second, unrelated error (report it as a separate finding; fix only this one here).
- The item's `done` status would need changing (it does not; this is a follow-up repair).