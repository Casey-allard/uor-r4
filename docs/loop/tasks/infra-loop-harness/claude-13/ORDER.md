# ORDER: infra-loop-pr04-gates, claude-13 (loop PR 4: loop gates runner, gates.json, ledger and breaks)

Lab: claude. Item: [infra-loop-pr04-gates](../../../../roadmap/items/infra-loop-pr04-gates.md) (#2907). Setup row: 4. Base: `e61746628`.

## Goal

Loop PR 4 from the build plan. The owner amended D52 on 21 Oct 2026 (#2919), so Claude builds every setup PR while it runs alone. The item's done-when:

> `loop gates` reads `docs/harness/gates.json`, in which every entry declares a break. It follows the run rules on the loop page: a gate that cannot run is REFUSED, later gates read UNKNOWN after a red, the true exit code is kept, and a change to a gate's read files during the run voids it. The ledger `local/<lab>/gates.jsonl` marks a result STALE after an uncommitted edit. The planted fault turns the check red: **`--break all` turns each gate red for its own reason.**

## Cancel probe (run first)

Does a gate runner or `gates.json` already exist?

Result at `e61746628`: there is no `docs/harness/gates.json` and no `loop` subcommand in `crates/uor-r4-harness/src/main.rs`. Today's checks run one by one, or through `uor-harness check`. The task stands. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/**`: a new module for the runner, and its wiring in `main.rs` and `lib.rs`.
- `crates/uor-r4-harness/tests/**`
- `crates/uor-r4-harness/Cargo.toml`: only for a dependency already in `Cargo.lock`.
- `docs/harness/gates.json` (new).
- `docs/harness/README.md`: the command's usage.
- `docs/loop/README.md`: only the Gates section, and only to say what is now built.
- `docs/loop/tasks/infra-loop-harness/claude-13/**`

`--watch` is PR 11's. It is not built here; the flag may be refused with a pointer to PR 11.

## Done when

- `uor-harness loop gates [--quick|--full|--keep-going|--status|--break G|all]` runs the gates listed in `docs/harness/gates.json`. Every entry names its command, the files it reads and a declared break, with the break's expected failure text.
- The loop page's run rules hold, each pinned by a test:
  - a gate that cannot run is REFUSED;
  - after a red, later gates read UNKNOWN unless `--keep-going` is given;
  - the true exit code is kept, with a log under `local/<lab>/`;
  - a change to a gate's read files between start and end voids the run;
  - the ledger `local/<lab>/gates.jsonl` records gate, verdict, seconds, commit and working-tree hash, and marks a result STALE after an uncommitted edit.
- `--break G` plants G's declared fault in a scratch worktree. The gate must go red with its declared failure text; red for another reason is REFUSED.
- The planted fault holds: `--break all` turns each listed gate red for its own reason.
- No research compute, pod, store or GitHub write happens inside a gate.
- The harness's own tests pass, and so do claim-wording, links and the local copyright check.

## Stop and ask if

- A run rule on the loop page is ambiguous enough that two readings would build different behaviour.
- A gate in `gates.json` would need network, GPU or a GitHub write.
- Any file outside scope needs to change.
