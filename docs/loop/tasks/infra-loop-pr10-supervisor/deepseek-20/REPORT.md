# REPORT: infra-loop-pr10-supervisor, deepseek-20 (repair: repeating windows, identity refusal, checked git)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 10. Base: `8415f914ac2a937882f134ec03ddcdb70d70aa6c`. Under D59 this setup PR's exact-head review comes from Claude's fresh-context reviewers.

## What did not work, first

The `deepseek-19` review was right about all four remaining defects, and fixing them exposed two more:

- **One check-in only.** The single `checked` flag meant a session productive for its first two hours and silent afterwards was never closed, and the before-observation was never refreshed. Fixed: `next_check = started_unix + k*7200`; after a Continue the observation becomes the new before and the window advances. A productive-then-silent fake client is now closed at the **second** window (test), while a one-window progressing client continues with a note and exits cleanly.
- **An empty identity could hang the supervisor.** If `ps -o lstart=` returned nothing, the record kept an empty identity; at a Close, `terminate_session` returned Ok without signalling and `child.wait()` blocked forever. Fixed at both ends: `start_session` refuses a session whose identity could not be read (killing the fresh group first), and `reap_child` bounds the post-terminate wait (poll, one re-KILL, then refuse) so no code path waits unbounded.
- **The identity string was locale- and time-zone-dependent.** `ps -o lstart=` prints local time and a localized month, so a watcher started under another environment could read a live session as dead. Fixed: `ps` runs with `LC_ALL=C` and `TZ=UTC`.
- **The WIP helper ignored git failures.** Only spawn errors were caught; a commit refused by the hook or a push with no upstream returned `Ok(())`. Fixed: add, commit and push each check `status.success()` and return the stderr; the push names `origin` and the branch. Tests cover the refusals (`main`, a non-lab branch, a foreign task folder, no ORDER), the `Lab:`/`Order:` trailers, and that only the task folder is committed (`unrelated.txt` stays untracked).
- **Test isolation.** `GIT_*` is stripped in the test git helper and the session child, and every test repository must sit under the temp dir; otherwise a run from a git hook could write into the repository being committed.
- **Two earlier REPORT claims were false and are corrected here:** the scoping/refusal rules were not tested (they are now, via `commit_and_push_wip`), and the fake-loop tests are `run_sessions`-level, not `cli`-level; `cli` itself remains untested (a named LOW).
- **Deferred LOWs** (not claimed): stale-`drive.lock` takeover after a crash and SIGINT handling; unparseable `--interval` silently becoming 60; README examples omitting `-- <cmd {prompt}>`; the card naming `close-*/` instead of the real directory; dedupe before rather than after a failed post; the card not saying when the WIP commit failed; and a `cli`-level test.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | code | ~40 changed | repeating windows with refreshed observations, empty-identity refusal, `reap_child`, UTC/locale-stable `ps`, checked git results with `push origin <branch>`, `GIT_*` stripped from the session child |
| `crates/uor-r4-harness/tests/loop_drive.rs` | test | +~200 | three new tests (two-window productive-then-silent close; WIP refusals; scoped WIP commit with trailers) plus `GIT_*` stripping and temp-dir assertion in the helper; 15 tests total |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-20/{ORDER,REPORT}.md` | docs | small | this repair order and report, plus receipts |

**Size:** one module repaired at the points the review named; no public API was removed (only `Checked`-flag behaviour and git error handling changed).

## Measurements

- **`cargo test -p uor-r4-harness --test loop_drive`:** 15 passed, 0 failed, three consecutive runs (2.89 s, 2.91 s, 3.01 s).
- **The two windows:** a fake client that commits once and then sleeps is closed at the second window (one card, reason starts `stalled:`); a fake client that commits and exits at the first window continues with a note and produces no card.
- **WIP scope:** `git show --name-only` lists only the task-folder file; `unrelated.txt` remains untracked; the commit message carries `Lab: deepseek` and the ORDER path; the push failure is returned, not swallowed.
- **Gates:** six GREEN at the code head; `--break all` CAUGHT, `loop-drive` red with the item's sentence.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --full`: six GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: six CAUGHT.
- `cargo test -q -p uor-r4-harness --test loop_drive`: 15 passed (three runs).
- `loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: each review item was reproduced before changing code; the empty-identity hang was fixed at both the start and the wait.
- **working-discipline**: three consecutive test runs and a fresh-process `--break` run rather than a single green.
- **scope-guard**: only the module, its tests, the task folder and (already-registered) shared files.

## Still open

- **A real Claude Code after-stop run** is still structural: the supervisor waits only on its tracked child, but the exclusion has not been exercised against that client.
- **The LOW follow-ups** listed above (stale lock, SIGINT, interval parse, README examples, card directory/failure line, dedupe, `cli` test).
- **`loop render --pr` and the compaction hooks** remain other build-plan rows.
- **Live close actions** (WIP push, card post) are best effort and not exercised in CI; their refusals and wording are tested.

## Plain summary

- **A session that works and then stalls is now closed at the right window.** The check-in repeats every two hours with a refreshed observation; the second window closes a silent client, while a progressing one keeps going with a note.
- **The close can no longer hang or lie:** a session with no readable identity is refused at start, the post-terminate wait is bounded, and the WIP helper checks git's answers and pushes only to `origin <branch>` after committing the active ORDER's folder with its trailers.
- **The tests now cover what the earlier REPORT claimed:** the WIP refusals, the scoped staging and trailers, and both two-window outcomes.