# ORDER: infra-loop-pr10-supervisor, deepseek-21 (repair: the lease-skip window)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `53d48e9face805fa10a852e33d4389a940cdb395`.

## Goal

Answer the counted other-vendor CHANGES at `53d48e9` (Claude, D59), which covers one required line: **the lease-skip branch never advanced `next_check`**, so a leased session re-observed, spawned git and printed "close skipped" every pause for the whole run. Fix: on a lease skip, advance `next_check += 7200` and refresh `before`; route the skip and check-in notes through a `notice` hook so the CLI prints them and tests can count them. Test: `exempt_lease: true` with a silent client past two windows sees the skip once per window and is never carded.

LOWs in the same push: `git init -b main` in the test helper and a non-lab (`kimi/x`) branch refusal case; strip **every** `GIT_*` variable (not only the eight named ones) from observation, session and test git children, because `GIT_CONFIG_PARAMETERS` and others otherwise survive.

## History

- deepseek-18 `c04f5bbf8` built the supervisor; deepseek-19 `81b51ea` added the live poll, scoped WIP, identity guards and the fake-clock tests; deepseek-20 `3f97590` added repeating windows, bounded reaping, UTC `ps`, checked git and test isolation. The review at `53d48e9` accepted those and required the lease-skip window advance and the two lows.

## Cancel probe (run first)

Class **instrument**. The CHANGES at `53d48e9` is counted at the head; reading the lease-skip branch shows `next_check` untouched, so the test that counts skip notes reproduces the defect. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, the shared registration files (`src/lib.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`) and this task folder.

## Done when

- A lease skip advances the window and refreshes the observation; the skip note appears once per 7,200 s window, not once per poll.
- The notices are observable (`DriveHooks::notice`); the CLI prints them; tests record them.
- The test helper uses `git init -b main` and a non-lab branch case; every `GIT_*` variable is stripped from git children in production and tests.
- `AGENTS.md` 71 lines; six gates GREEN, `--break all` CAUGHT, `report --check` and `pr-check --local` PASS; 16 drive tests pass repeatedly; receipts at the final head.

## Stop and ask if

- A real Claude Code after-stop run is needed (the exclusion remains structural).
- A stale-lock takeover or SIGINT handling is needed now (both stay named LOWs, out of this round).