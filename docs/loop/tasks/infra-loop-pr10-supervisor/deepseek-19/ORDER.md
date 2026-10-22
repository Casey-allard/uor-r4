# ORDER: infra-loop-pr10-supervisor, deepseek-19 (repair: live check-in, safe WIP, guarded signalling)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `13430063e06a3e05fbb09836c38e4df4ec9c0b41`.

## Goal

Answer the counted other-vendor CHANGES at `13430063e` (Claude, D59). Required:

1. **HIGH — check-in while the session is live.** `child.wait()` blocked, so every long session looked stalled and none was stopped. Fix: before-observation at launch, `try_wait` polling with an injected clock, at 7200 s classify live progress and terminate only a Stalled session (never a healthy or leased one); keep progressing sessions with a note; count consecutive stalls so spiraled is reachable. Fake-clock tests: committing client continues, silent one is closed and carded.
2. **HIGH — scoped, legal WIP commit.** Refuse `main` or a non-`<lab>/*` branch; stage only the active ORDER's task folder (refuse otherwise); add the `Order:` trailer; strip `GIT_*`.
3. **MEDIUM — identity-checked signalling.** Refuse pgid 1/0, the own group, and a pgid that is not the recorded pid; verify pid+lstart before any signal; watcher liveness by pid+identity.
4. **MEDIUM — a spawn with no record** is killed on its own pid; the flaky test uses a sleeping child; the gate's planted fault is exercised at CLI level with a fake card sink.

LOW, all in scope: per-lab O_EXCL lock; `{prompt}` refused as argv[0]; `--interval` floor; `--prompt-file`; fresh Go per session; comment fix; card dedupe/`close-*`; `gates.json` newline; unix-only README; receipts at head; corrected REPORT measurements.

## History

- deepseek-18 `c04f5bbf8` built the supervisor; the review at `13430063e` accepted the launch, stop, group-signal, lease and card mechanics and required the live check-in, scoped WIP, identity guards, watcher liveness, child cleanup and the CLI-level planted-fault test.

## Cancel probe (run first)

Class **instrument**. The CHANGES record is counted at the current head; `loopdrive.rs` had `child.wait()` before both observations (`:744-756`) and `tests/loop_drive.rs:110` raced a fast child — both reproduced before the repair. The task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, the shared registration files (`src/lib.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`) and this task folder.

## Done when

- The live check-in continues real progress with a note, terminates and cards a stalled session, exempts leased ones, and spiraled is reachable.
- WIP close is branch- and scope-checked with the `Order:` trailer and `GIT_*` stripped; signals are identity-checked and refuse pgid 1/own group; catchers and watchers use pid+lstart; a spawn without a record is killed.
- The CLI-level fake-clock tests pin closed-and-carded and continues-with-a-note; the flaky record test no longer races; the lows are fixed.
- `AGENTS.md` 71 lines; six gates GREEN, `--break all` CAUGHT, `report --check` and `pr-check --local` PASS; receipts at the final head.

## Stop and ask if

- The active ORDER cannot be found for the WIP commit (refuse the close action and card instead).
- A real Claude Code after-stop run is needed to prove the exclusion (the structural fix remains).