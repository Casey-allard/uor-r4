# ORDER: infra-loop-pr10-supervisor, deepseek-18 (loop drive, watchers, kill switch, soft check-in)

Lab: deepseek. Item: [infra-loop-pr10-supervisor](../../../../roadmap/items/infra-loop-pr10-supervisor.md) (#2915). Setup row: 10. Base: `f1c78da8e`.

## Goal

Build the supervisor the item's done-when describes: `loop drive` starts a fresh headless session per unit with the lab's caller-supplied client and model, stopping on the loop page's stops (kill switch first, via `loop next`); a watcher outlives a session for a leased GPU run, renews the lease and never cancels a healthy run; the 2-hour check-in is soft per D53 (real progress continues with a note; a spiraled or stalled session closes with WIP committed and pushed, a REPORT stub, a PICKUP from git state and a PM card); only the supervisor's own pid/group is signalled and a client's after-stop summary is not a lab session. The planted fault turns the gate red: a fake stalled client past two hours is closed and carded, while a fake progressing one continues with a note.

## Cancel probe (run first)

Class **instrument**. At base `f1c78da8e`, `loop drive` answers "not built yet (see the build plan)"; `loopnext::run` already returns `Decision::{Go,Stop}` and checks killing stops including the kill switch file and #2028 label; the PR 9 adapters and `heartbeatcard`/`githubcards` exist. The done-when conditions are absent, so the task stands.

## Files in scope

`crates/uor-r4-harness/src/loopdrive*.rs`, `crates/uor-r4-harness/src/githubcards.rs`, `crates/uor-r4-harness/src/heartbeatcard.rs`, `crates/uor-r4-harness/tests/loop_drive*.rs`, the shared registration files (`src/lib.rs`, `src/main.rs`, `src/loopgates.rs`, `src/doccheck.rs`, `docs/harness/README.md`, `docs/loop/README.md`, `docs/harness/gates.json`, `docs/roadmap/README.md`) and this task folder.

## Done when

- `loop drive --client <c> --model <m> [--prompt <p>] [--max-sessions <n>] -- <launch argv with {prompt}>` runs one fresh session per unit: `loop next` decides every start and stop (kill switch included, not re-implemented), the launch is in its own process group, pid/pgid/client/model/prompt-SHA are recorded under `local/loop/drive/<lab>/`, and only that recorded pid/group is signalled.
- The soft 2-hour check-in: at 7200 s a progressing session continues with a note; a stalled or spiraled one closes with WIP commit/push, REPORT stub, PICKUP from git state and a PM card; `--watch` follows a session and `--watch --lease` renews a leased GPU run until the lease ends without cancelling it.
- The `loop-drive` gate runs the tests; its planted fault turns it red with "a fake stalled client past 2 hours is closed and carded". `AGENTS.md` stays 71 lines. Docs/flags updated.
- `loop gates --full` GREEN, `--break all` CAUGHT, `report --check` and `pr-check --local` PASS.

## Stop and ask if

- A client needs a launcher or credential the adapter data does not carry (use caller argv; stop rather than invent).
- The check-in needs tool-call telemetry the repository does not record (fallback: commits, gate runs, working tree; named in REPORT).
- The kill switch would need touching the owner's STOP file or #2028 label directly.