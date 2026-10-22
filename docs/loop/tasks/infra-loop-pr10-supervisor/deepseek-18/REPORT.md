# REPORT: infra-loop-pr10-supervisor, deepseek-18 (loop drive, watchers, kill switch, the soft check-in)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 10. Base: `f1c78da8e`. This is a loop setup PR: its exact-head review comes from Claude's fresh-context reviewers (D59).

## What did not work, first

- **The supervisor is unix-only.** It puts a session in its own process group and signals that group, which is a unix facility; on a non-unix target it refuses with a named error instead of pretending. The items's launcher is the owner's macOS laptop, so this is the honest boundary rather than a defect.
- **The loop has no tool-call telemetry.** The loop page defines stalled as "no commits and no tool calls", but the repository records commits, gate ledger rows and working-tree state, not tool calls. Progress therefore means new commits, new gate runs or a changed working tree; the ORDER's stop-and-ask names this fallback and the REPORT states it. It is conservative in the right direction (a session editing all day counts as progressing), not exact.
- **The after-stop summary session is excluded by construction, not observed against a real Claude Code run.** The supervisor waits only on the child it started and records and signals only that pid/group, so a client's later summary process is invisible to it. The tests prove the process-group behaviour with shells; they do not run Claude Code, which the owner-only client would need.
- **The close actions that leave the machine are best effort, not unit-tested.** On a stalled close the supervisor writes the REPORT stub and PICKUP from git state (tested), commits and pushes the WIP with `git` (live-only), and posts the PM card on #2028 through `gh` (wording tested; posting live-only). A failure is logged and the supervisor continues; nothing is force-pushed or deleted.
- **Two rows of the build-plan entry are not in this item's done-when and are not built here.** `render refresh` is step 14 (`loop render --pr`, still to build); the compaction hooks are per-client and P-1 records that the DeepSeek harness has none, so the supervisor handles session boundaries instead and `docs/harness/README.md` must not claim a hook that does not exist.
- **The PR 9 adapters do not carry a launch line or a model.** They name the headless command (`headless`, `hook_entry`), not the argv or pinned model, so `loop drive` takes both from the caller (`--client`, `--model`, and the command after `--`) and records what it was given; it does not invent a launcher.
- **The lease watcher does not lease pods.** The owner path stays `scripts/pod/uor-pod`; `--watch --lease` only runs the caller's renew command on an interval, notes each renewal and stops when that command reports the lease ended, never cancelling the run.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | code | ~980 | `loop drive`: kill switch and every other stop through `loopnext::run`; fresh session per unit from the caller's argv with one `{prompt}`; own process group; pid/pgid/client/model/prompt-SHA record; only the recorded group is signalled; soft 2-hour check-in (`classify`, `check_in`); close with REPORT stub, PICKUP, WIP commit/push and PM card; `--watch` session watcher and `--watch --lease` detached lease watcher; the prompt composer |
| `crates/uor-r4-harness/tests/loop_drive.rs` | test | ~260 | 10 tests: planning (progress, soft check-in, stalled vs progressing, spiralled, placeholder, prompt), the real short session and its record, process-group termination, both watchers, the PM card wording, the close stub files, the real repo observation |
| `crates/uor-r4-harness/src/loopgates.rs` | code | +1 | `loop drive` dispatch |
| `crates/uor-r4-harness/src/lib.rs` | code | +1 | module registration |
| `crates/uor-r4-harness/src/doccheck.rs` | code | +9 | `loop drive` in the subcommand table; `--prompt`, `--max-sessions`, `--pid`, `--interval`, `--lease` in the flag table |
| `docs/harness/gates.json` | generated | +30 | the `loop-drive` gate and its planted fault |
| `docs/harness/README.md` | docs | +27 | the `loop drive` section: stops, launch argv, pid record, soft check-in, both watchers, the gate |
| `docs/loop/README.md` | docs | +2/−2 | build-plan row 15 now built; the supervisor paragraph and the check-in paragraph name `loop drive` |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-18/{ORDER,REPORT}.md` | docs | small | this order (with `Setup row: 10`) and report, plus receipts |

**Size:** about 1,300 changed lines across the module, its tests, the gate and the docs; the item estimated the supervisor as the largest loop PR, and the process layer is deliberately thin next to the pure planner.

## Measurements

- **The gate set:** six gates GREEN at the head, including the new `loop-drive`; `loop gates --break all` is CAUGHT on all six.
- **The item's planted sentence:** `--break loop-drive` prints `red with "a fake stalled client past 2 hours is closed and carded"`, and the fake progressing client continues with a note in the same test file.
- **`loop drive` tests:** 10 passed, 0 failed; they run a real short shell session, terminate a real `sleep` in its own process group, exercise both watchers and write/read the close artifacts.
- **Harness lib suite:** 667 tests passed at the code commit before the lease/card extension; the extension adds no lib test but builds clean and the integration suite above is rerun at the head.
- **The docs checks:** `uor-harness check` reports no doccheck or flag error at the head; the new flags are in the table and the documented `loop drive` command resolves.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check` (the `fmt-harness` gate): GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --full`: six GREEN.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: six CAUGHT, each with its declared text.
- `cargo test -q -p uor-r4-harness --test loop_drive`: 10 passed.
- `uor-harness check`: no error or warning other than the pre-existing free-disk band and spending-class info.
- `loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: the pure planner was separated from the process layer before writing either, and the classification limit (no tool-call telemetry) is stated instead of hidden.
- **scope-guard**: only the module, its tests, the gate, the shared registration files and the two docs the build plan names.
- **working-discipline**: the planted-fault run is the acceptance test, not the green run alone.

## Still open

- **Wiring `loop drive` into every client's headless launcher** is not part of this done-when; the command currently takes the launch argv from its caller, by design.
- **`loop render --pr` (step 14) and the per-client compaction hooks** remain separate build-plan rows; P-1 records no deepseek hook.
- **A real Claude Code after-stop run** should be exercised once the owner runs the supervisor for that client; the exclusion is structural, not tested against that client.
- **The `--lease` renew command** is the caller's `uor-pod renew` invocation; the supervisor never renews a lease itself.

## Plain summary

- **The loop now has a supervisor.** `loop drive` asks `loop next` first (kill switch included), starts one fresh headless session per unit from an explicit launch command, records the pid/group it started, signals only that group, and writes a session record; nothing else is ever killed.
- **The 2-hour rule is soft, as D53 says.** A session with commits or gate activity past two hours continues with a note; one with nothing to show is closed with a REPORT stub, a PICKUP from git state, its WIP committed and pushed, and a card to the PM. The gate's planted fault removes that close, and the gate goes red with the item's own sentence.
- **Leased GPU runs get a watcher that outlives the session** when started detached: it renews on an interval, notes each renewal, and stops when the lease ends without ever cancelling the run.