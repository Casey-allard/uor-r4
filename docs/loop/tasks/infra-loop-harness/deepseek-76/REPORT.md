# REPORT: reconcile harness registry-aware checks after current milestone status, deepseek-76

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Base: `43809a41a`.

## What did not work, first

- `cargo test --locked -p uor-r4-harness` started with 13 failing tests after legitimate milestone progress changed the live registry. The failures were fixture and heartbeat assumptions, not failed product checks: several tests still named pre-M1 lines and counts, the archived PR-lint fixture still expected the old line, and the heartbeat fixture let unrelated active milestones leak into a one-milestone scenario.
- `pivot_at_three` printed a hard-coded `3` denominator. That matched the old registry shape by accident and would misreport any current or future line whose denominator is not three.
- Documentation examples referred to a retired top-level claim-wording subcommand and used shell semicolons inside command fences, so the documented-command parser treated prose as an executable command.

## What changed

- Updated registry-aware unit and golden-fixture inputs to the current M1 line, `saved discrete six-bar close` at `6/6`, while preserving the historical evaluation-integrity and route/action/learning lines as historical prose.
- Changed the pivot diagnostic to use the registry or declared denominator instead of hard-coding `/3`.
- Reset all unrelated milestone statuses inside the heartbeat fixture before activating the one milestone under test.
- Replaced stale command examples in four existing task reports with commands and prose that remain valid.
- Reconciled `docs/labs/session-goal.md` with the current M1 registry line without presenting the completed 6/6 line as new progress or resetting historical evidence.

No milestone target, acceptance bar, hold, model result, serving path or training path changed.

## Measurements

No model compute and no corpus run. This is a test-harness and documentation reconciliation.

## Checks run at the head

- Initial reproduction: `CARGO_TARGET_DIR=target/deepseek-harness-reconcile /home/callard/.cargo/bin/cargo test --locked -p uor-r4-harness` -> 13 failing tests.
- After repair: the same command -> 677 library tests passed, 1 ignored; all integration suites passed; 1 doc-test passed.
- `git diff --check` -> pass.
- Formatting, Clippy, roadmap, harness and loop gates are run before the review request and recorded on the PR.

## Skills used

working-discipline, scope-guard and solution-integrity.

## Still open

- PR #3550's required harness CI can run once this prerequisite lands on `main`; its CPU pilot remains a bounded negative and does not qualify a milestone.
- The wider CPU-training-method repair inventory and isolated repairs proceed after this prerequisite is verified on fresh `main`.

## Plain summary

The harness was red because its test fixtures still described a registry state that legitimate milestone progress had replaced. This repair makes the checks follow the current registry, fixes one denominator bug exposed by that change, and leaves every owner-set bar and held path untouched.
