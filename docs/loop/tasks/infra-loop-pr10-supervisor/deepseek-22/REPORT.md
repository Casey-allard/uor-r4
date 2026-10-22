# REPORT: infra-loop-pr10-supervisor, deepseek-22 (repair: pin the lease-skip fix)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 10. Base: `35750bfecc9b32feadef8e761f876f556e370a52`. Under D59 this setup PR's exact-head review comes from Claude's fresh-context reviewers.

## What did not work, first

The `deepseek-21` review accepted the lease-skip fix and the `GIT_*` change, and found that the new test did not pin them:

- **The lease test passed without the fix.** It asserted only `notices.len() >= 2`; the fake clock stepped 3,600 s per read and the child ended on a wall-clock sleep, so with the two lines present the run produced about 15 notes and without them about 29 - both cleared `>= 2`. Fixed: the clock now steps **600 s** per read (a 7,200 s window is exactly 12 reads), the child ends when the pause hook has counted **30 polls**, and the test asserts **exactly 2** skip notices (the 7,200 s and 14,400 s windows).
- **Negative-gate evidence, observed in this worktree:** with `before = after; next_check = next_check.saturating_add(CHECK_IN_SECONDS);` deleted from the lease branch, the test fails with `left: 19, right: 2` - 19 skip notes in the same 30-poll run, exactly the per-poll spam the code fix prevents. Restoring the two lines makes it pass. This is the evidence the previous test could not give.
- **The all-`GIT_*` strip was too broad.** It also removed transport/auth variables (`GIT_SSH_COMMAND`, `GIT_ASKPASS`), so a lab whose push needs them could be refused. Fixed: `strip_git_env` keeps the transport/auth variables and strips the repository-locating ones (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_COMMON_DIR`, `GIT_NAMESPACE`, `GIT_PREFIX`) plus every `GIT_CONFIG*`.
- **Other LOWs from earlier records stay deferred and named:** stale-lock takeover, SIGINT handling, `--interval` parse errors, README launch examples, real card directory, dedupe ordering, WIP-failure line, `cli`-level test. This round keeps to the one required assertion and the one low.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/tests/loop_drive.rs` | test | ~70 changed | deterministic lease test: 600 s clock steps, poll-counted child end, exact notice count of 2 |
| `crates/uor-r4-harness/src/loopdrive.rs` | code | ~10 changed | `strip_git_env` keeps transport/auth variables and strips repository-locating ones plus `GIT_CONFIG*` |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-22/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** one test rewritten to be a real planted assertion, plus a one-function correction of the strip list.

## Measurements

- **`cargo test -p uor-r4-harness --test loop_drive`:** 16 passed, 0 failed, three consecutive runs (3.21 s, 3.17 s, 2.89 s).
- **Negative gate:** with the two lease-skip lines removed, the same test fails with **19 skip notes where 2 are expected**; with them restored it passes. The exact run log is in this REPORT's second bullet.
- **Lease semantics:** a silent client with `exempt_lease: true` is never carded and no close record is written; the only effects are the two skip notices at the two windows.
- **Gates:** six GREEN at the code head; `--break all` CAUGHT, `loop-drive` red with the item's sentence.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --full`: six GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: six CAUGHT.
- `cargo test -q -p uor-r4-harness --test loop_drive`: 16 passed (three runs); the negative-gate run failed as intended.
- `loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: an assertion that cannot fail is not evidence; the test now fails when the fix is removed (19 vs 2), which is the proof the review asked for.
- **working-discipline**: exact counts and three repeated runs instead of a threshold.
- **scope-guard**: the test, one function and the task folder.

## Still open

- **LOW follow-ups** from earlier records, unchanged and named.
- **A real Claude Code after-stop run** is still structural rather than observed.
- **`loop render --pr` and the compaction hooks** remain other build-plan rows.
- **Live close actions** (WIP push, card post) are best effort and not exercised in CI.

## Plain summary

- **The lease test now proves the fix.** It opens exactly two windows on a 600-second clock, ends the client after 30 polls, and asserts exactly two skip notes; deleting the two advancing lines makes it fail with 19 notes instead of 2.
- **The git strip keeps what a push needs.** Only variables that locate a repository or override config are removed; `GIT_SSH_COMMAND` and `GIT_ASKPASS` survive.
- **Everything else holds:** 16 drive tests pass three runs in a row, six gates are green, and the planted fault still turns the `loop-drive` gate red with the item's own sentence.