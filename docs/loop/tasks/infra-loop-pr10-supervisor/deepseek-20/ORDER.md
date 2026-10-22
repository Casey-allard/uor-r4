# ORDER: infra-loop-pr10-supervisor, deepseek-20 (repair: repeating windows, identity refusal, checked git)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `8415f914ac2a937882f134ec03ddcdb70d70aa6c`.

## Goal

Answer the counted other-vendor CHANGES at `8415f91` (Claude, D59). Required:

1. **HIGH - repeat the check-in.** One `checked` flag allowed a single 7,200 s check-in; a session productive for its first window and then silent forever was never closed. Fix: `next_check = started + k*7200`, refresh the observation after each Continue, and test a productive-then-silent client that is closed at the 4-hour window.
2. **HIGH - an empty identity hangs the supervisor.** If `ps -o lstart=` returns nothing, `terminate_session` returned Ok without signalling and `child.wait()` blocked. Fix: refuse to start when `lstart` is empty (kill the fresh group), and make the post-terminate wait bounded: `reap_child` polls, re-KILLs once, then refuses rather than waiting unbounded.
3. **MEDIUM - identity independent of environment.** `ps` runs with `LC_ALL=C` and `TZ=UTC`, so a watcher started under another locale or zone reads the same start time.
4. **HIGH - checked git results.** The WIP helper caught only spawn errors. Fix: check `status.success()` for add, commit and push; push with `git push origin <branch>`; test refusals on `main`, a non-lab branch, a foreign task folder, no ORDER, plus the trailers and the scoped staging.
5. **MEDIUM - test isolation.** `GIT_*` is stripped in the test git helper and in the session child; every test repository must sit under the temp dir.
6. **REPORT corrections:** the earlier claims that scoping and refusal rules were tested, and the CLI-level wording, were wrong; this repair tests `commit_and_push_wip` and calls the fake-loop tests `run_sessions`-level.

LOW follow-ups (not required this round; named in REPORT): stale-lock takeover, SIGINT handling, interval parse errors, README `-- <cmd {prompt}>` examples, real card directory, dedupe-after-post, WIP-failure line in the card, `cli`-level test.

## History

- deepseek-18 `c04f5bbf8` built the supervisor; deepseek-19 `81b51ea` added the live poll, scoped WIP, identity guards and the fake-clock tests. The review at `8415f91` accepted those and required the repeating window, the empty-identity refusal, UTC `ps`, checked git results, test isolation and the REPORT corrections.

## Cancel probe (run first)

Class **instrument**. The CHANGES at `8415f91` is counted at the current head; reading the session loop shows the single `checked` flag and the unchecked git helper, and the two-window behaviour reproduces with a fake clock. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, the shared registration files (`src/lib.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`) and this task folder.

## Done when

- The check-in repeats every 7,200 s with a refreshed observation; a productive-then-silent client is closed at the second window; one-window progress continues with a note.
- An empty `lstart` refuses the session (fresh group killed) and no code path waits unbounded on a live child.
- `ps` is locale/zone independent; git add/commit/push failures are returned and the push names `origin` and the branch.
- Tests cover the WIP refusals and trailers, strip `GIT_*`, and keep repos under the temp dir; 15+ tests pass repeatedly.
- `AGENTS.md` 71 lines; six gates GREEN, `--break all` CAUGHT, `report --check` and `pr-check --local` PASS; receipts at the final head.

## Stop and ask if

- Takeover of a stale lock needs an owner decision (it does not: the LOW is deferred, not claimed).
- A real Claude Code after-stop run is needed (the exclusion remains structural).