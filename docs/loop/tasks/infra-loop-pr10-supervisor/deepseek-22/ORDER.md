# ORDER: infra-loop-pr10-supervisor, deepseek-22 (repair: pin the lease-skip fix with an exact count)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `35750bfecc9b32feadef8e761f876f556e370a52`.

## Goal

Answer the counted other-vendor CHANGES at `35750bf` (Claude, D59). The code fix is accepted; the **new test did not pin it** (it asserted `notices.len() >= 2`, and the clock stepped 3,600 s with a wall-clock-ended child, so 15 notes with the fix and 29 without both passed). Fix: rewrite `a_leased_session...` deterministically - clock steps 600 s per read (a 7,200 s window is exactly 12 reads), the child ends when the pause hook counts 30 polls, and the test asserts the **exact** notice count of 2 (7,200 s and 14,400 s windows). Show in REPORT that removing the two lease-skip lines makes it fail (observed: 19 notes vs expected 2).

LOW in the same push: keep the transport/auth `GIT_*` variables (`GIT_SSH_COMMAND`, `GIT_ASKPASS`, ...) and strip only the repository-locating ones plus `GIT_CONFIG*`.

## History

- deepseek-21 `67076f6` advanced the lease-skip window and stripped every `GIT_*`; the review at `35750bf` accepted both and required only a test that actually fails without the fix, plus keeping auth variables.

## Cancel probe (run first)

Class **instrument**. The CHANGES at `35750bf` is counted at the head; deleting the two lease-skip lines and running the old test still passed, which is the defect. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, the shared registration files (`src/lib.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`) and this task folder.

## Done when

- The lease test steps 600 s, ends the child by poll count, asserts exactly 2 skip notices, and **fails with the two skip lines removed** (evidence in REPORT: 19/2).
- `strip_git_env` keeps `GIT_SSH_COMMAND`/`GIT_ASKPASS` and strips repository-locating variables and `GIT_CONFIG*`.
- 16 drive tests pass repeatedly; `AGENTS.md` 71 lines; six gates GREEN, `--break all` CAUGHT, `report --check` and `pr-check --local` PASS; receipts at the final head.

## Stop and ask if

- A real Claude Code after-stop run is needed (structural only).
- Other LOWs from earlier records are needed now (they stay deferred and named).