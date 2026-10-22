# REPORT: infra-loop-pr10-supervisor, deepseek-21 (repair: the lease-skip window)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 10. Base: `53d48e9face805fa10a852e33d4389a940cdb395`. Under D59 this setup PR's exact-head review comes from Claude's fresh-context reviewers.

## What did not work, first

The `deepseek-20` review found one real defect and two lows:

- **The lease-skip branch never advanced `next_check`.** A leased session is never closed - correct - but after the first two-hour window every 1-second pause re-observed the repository (spawning git) and printed "leased session; close skipped" again: about 3,600 lines and git spawns per hour for the whole leased GPU run the supervisor exists to watch. Fixed: the skip now advances `next_check += CHECK_IN_SECONDS` and refreshes `before`, so it happens once per window. The skip and check-in notes go through a `DriveHooks::notice` hook, which the CLI prints and tests count; a new `exempt_lease: true` test sees the skip once per window and no card.
- **Two LOWs rode along:** the test helper now runs `git init -b main` (so the `main` refusal is really tested) and adds a non-lab (`kimi/x`) branch refusal case; and every `GIT_*` variable is stripped from git children - observation, session and test - because a named list of eight misses `GIT_CONFIG_PARAMETERS` and others.
- **Not claimed:** the other LOW follow-ups from the last review (stale-lock takeover, SIGINT handling, `--interval` parse errors, README launch examples, real card directory, dedupe ordering, WIP-failure line, `cli`-level test) remain named, deferred items; this round deliberately keeps to the one required line and the two lows.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | code | ~30 changed | lease-skip window advance and observation refresh; `DriveHooks::notice`; `strip_git_env` removes every `GIT_*` key in observation, session and WIP git calls |
| `crates/uor-r4-harness/tests/loop_drive.rs` | test | +~60 | new `exempt_lease` skip-per-window test; `git init -b main`; `kimi/x` refusal; all-git-env stripping in the test helper; notices wired into every hooks construction; 16 tests |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-21/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** a one-window fix plus the two named lows in the same push; no public API removal.

## Measurements

- **`cargo test -p uor-r4-harness --test loop_drive`:** 16 passed, 0 failed, three consecutive runs (2.91 s, 2.90 s, 3.02 s).
- **Lease skip:** a fake silent client with `exempt_lease: true` produces one skip notice per 7,200 s window (at least two in the test), no PM card and no close record.
- **WIP refusals:** `main`, `kimi/*` and a foreign task folder are refused; the commit stages only the task folder, carries `Lab:`/`Order:`, and pushes `origin <branch>`.
- **Gates:** six GREEN at the code head; `--break all` CAUGHT, `loop-drive` red with the item's sentence.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --full`: six GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: six CAUGHT.
- `cargo test -q -p uor-r4-harness --test loop_drive`: 16 passed (three runs).
- `loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: the review's reproduction (a leased run polluting hours of output) was fixed at the window arithmetic, and the note is now observable rather than inferred from stdout.
- **working-discipline**: three consecutive test runs and a `--break all` run at the head.
- **scope-guard**: the module, its tests and the task folder only.

## Still open

- **LOW follow-ups** from the previous record, unchanged and named: stale-lock takeover on a dead pid, SIGINT handling, `--interval` parse errors, README launch examples, real card directory, dedupe before/after a failed post, WIP-failure line, `cli`-level test.
- **A real Claude Code after-stop run** is still structural rather than observed.
- **`loop render --pr` and the compaction hooks** remain other build-plan rows.
- **Live close actions** (WIP push, card post) are best effort and not exercised in CI; their refusals, trailers and wording are tested.

## Plain summary

- **A leased GPU run now produces one skip note per two-hour window, not one per second.** The window advances on a skip, and the observation refreshes, so the supervisor stops re-spawning git and re-printing for hours.
- **The low fixes make the earlier claims true:** the `main` refusal is tested on `main`, a non-lab branch is refused, and every inherited `GIT_*` variable is stripped from git children.
- **Sixteen drive tests pass three runs in a row**, the six gates are green and every planted fault is caught, including the item's own sentence for the stalled-client close.