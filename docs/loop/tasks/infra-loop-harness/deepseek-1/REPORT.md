# REPORT: infra-loop-pr11-progress-window, deepseek-1 (loop PR 11: gates --watch and warnings raised to errors)

Lab: deepseek. Order: folded into [deepseek-2/ORDER.md](../deepseek-2/ORDER.md) as history; this folder's original ORDER.md was removed so the PR adds exactly one ORDER, which `loop review --check` and `loop pr-check` both require. It was committed as `1a157a2cd` before any work.
Base: `7864b05`. `origin/main` did not move while this branch was built.
Closes #2916. References #2904.

## What did not work, first

- **Which warnings are "the named warnings" is not written down.** The item and the loop page name no list, so the ORDER names one: the four `loop arrive` warnings (checkout behind, older binary, hooks missing or different, read path above 76,800 B) and the `loop pr-check` read-path warning. A reviewer should judge that reading. No PM answer arrived while the branch was built; if the intended set differs, a repair order should replace this one.
- **Two new CLI tests failed on the fixture, not on the feature.** They exited 2 on "cannot read the lab registry docs/harness/labs.json" because the fixture repository had no lab registry. Added one; both then passed.
- **`harness.documented_commands` failed on the new documentation with "unknown flag --watch" even after the binary accepted it.** The rule keeps its own `FLAGS` list in `doccheck.rs`, and loop PR 4 registered the flag only because it was refused; it was never in that list. Added it (78 to 79 flags); the rule now passes.
- **I rebuilt into the default dev profile while PATH pointed at the release binary**, so one check still ran the old binary and still reported `--watch` unknown. Rebuilt `--release`; the head binary then accepts it. Instrument, not mechanism.
- **One existing test asserted that `--watch` is refused as "loop PR 11".** That is no longer true. It now asserts the refusals for `--watch` with `--quick`, `--status` and `--break`, and keeps the "not built yet" assertion for `loop drive`.
- **`loop arrive` is red on this branch by design:** checkout FAILs because the branch is not in `origin/main` (true of any unmerged branch), and hooks FAIL because the shared checkout's pre-commit hook differs from this binary's. The hooks check stays red until loop PR 12 installs the hook.
- **The window draws on stderr in the same terminal, not in a separate OS window.** The item asks for a window "in a terminal"; the kit's separate-terminal step is not portable to a headless harness and the done-when does not need it.
- **A single gate longer than five minutes closes the window.** The status file does not change while a gate runs, so the five-minute give-up fires; the run itself continues and its exit code is unchanged. See Still open.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopgates.rs` | code | +298/−14 | the progress window: status file written after every gate, the renderer, the open/off decision, the ledger estimate, and the five-minute give-up; `run_gates` gained a progress callback |
| `crates/uor-r4-harness/src/looparrive.rs` | code | +44/−30 | the four former warnings are FAILs; the `Warn` verdict is gone from arrive |
| `crates/uor-r4-harness/src/loopprcheck.rs` | code | +26/−13 | every read-path overrun is a FAIL, whether or not it grew; the read-path warning is gone |
| `crates/uor-r4-harness/src/doccheck.rs` | code | +2/−1 | registers `--watch` |
| `crates/uor-r4-harness/tests/loop_gates_watch.rs` (new) | tests | +216 | the window's rules: N of M, reds, estimate, open only in a terminal full run, give up at 300 s, headless CLI is off |
| `crates/uor-r4-harness/tests/loop_arrive.rs`, `loop_prcheck.rs`, `loop_gates_run_rules.rs` | tests | +23/−11 | the raised severities and the new `--watch` refusals |
| `docs/harness/gates.json` | data | +12 | the `loop-gates-watch` gate and its planted break |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | +21/−4 | `--watch` usage, the window, the raised severities, the read-path budget |
| `docs/harness/claims.json` | generated | +7 | this branch's claim line |
| `docs/loop/tasks/infra-loop-harness/deepseek-1/ORDER.md` | docs | +63 | this task's ORDER |

**Size:** 656 insertions and 72 deletions over 12 files. Against the ORDER's 900-line estimate that is about 0.73. By class (insertions): code 323, tests 239, docs 75, data/generated 19.

**Design calls:**
- The runner writes `local/<lab>/gates/<run-id>/status.json` by writing a temporary file and renaming it, after every gate. Nothing waits for a reader.
- The watcher polls that file every 500 ms and writes one line per change to stderr: `gate N of M`, the gate running now, each RED as it lands, and the estimate. It opens once, closes after the run, and gives up after 300 s without a change, saying so.
- It opens only when `--watch` is given, the run is full (not `--quick`, `--status` or `--break`; those refuse it with exit 2), stdin and stdout are both terminals, and `CI` is unset.
- The estimate is the sum of the remaining gates' latest seconds per gate from `local/<lab>/gates.jsonl`; a first run says "no estimate yet".
- If the window is off, crashes or gives up, the gates run exactly as before; a panic in the watcher cannot change a verdict or the exit code.

## Measurements

- **The four gates at the head, with the shared target directory set:** `loop gates --full` GREEN on all four: fmt-claim-wording 0.6 s, fmt-harness 1.7 s, claim-wording 2.8 s, loop-gates-watch 1.5 s.
- **The planted breaks:** `loop gates --break all` CAUGHT on all four with each gate's own declared text; the new gate went red with `planted_break_watch` at exit 101.
- **The tests:** `cargo test -q --locked -p uor-r4-harness --no-fail-fast` passed on every target after the one test update above; the library target alone ran 658 tests, and the loop targets ran 29 (run rules), 28 (pr-check), 17 (arrive) and 12 (watch).
- **Clippy:** `cargo clippy -q --locked -p uor-r4-harness --all-targets` reported no warning.
- **The check set:** `uor-harness check` exits 0 with 174 rules and 0 errors (140 warnings and one info, all pre-existing coordination and registry notes).
- **The raised severities, measured on this branch:** `loop arrive` FAILs `checkout` (HEAD is not in origin/main) and `hooks` (the shared hook differs), and PASSes or N/As the rest; `loop pr-check --local` fails `read-path` only when a budget is actually exceeded, which this branch does not.
- No research compute, no GPU, no pod and no store were used; the only network was the harness's own `git fetch`.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: clean.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: every target passed.
- `cargo clippy -q --locked -p uor-r4-harness --all-targets`: no warnings.
- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on all four gates.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: CAUGHT on all four gates.
- `UOR_LAB=deepseek uor-harness check`: exit 0.
- `UOR_LAB=deepseek uor-harness check --only harness.documented_commands`: pass.
- `UOR_LAB=deepseek uor-harness loop gates --watch --quick`: exit 2, "needs a full run".
- `cargo run -q --locked -p xtask -- links`: 4789 sources and 19305 links checked; 930 broken links repo-wide, all pre-existing, and none in a changed file (`docs/harness/README.md` and `docs/loop/README.md` do not appear in the report).

## Skills used

- **gates**, read from its private local install (`~/.dsh/skills/gates`), because probe P-1 showed the DeepSeek harness's loader rejects that skill's YAML frontmatter; its progress-window design is the model here, written in our own words.

## Still open

- **The named-warnings interpretation** stands as the ORDER states; a different intended set needs a repair order.
- **A long single gate closes the window** after five minutes without a status change. A runner heartbeat while a gate runs would keep the estimate live; it is not added because the item's own fault is the window giving up after five minutes without change, and the run continues either way.
- **No pty test.** Whether the window opens in a real terminal is decided by a tested function (`watch_decision`) and a headless CLI test; no automated test allocates a pseudo-terminal.
- **The separate-terminal step** of the kit's progress window is not built; the window is drawn on stderr.
- **Hooks stay red in `loop arrive` until loop PR 12** installs the shared pre-commit hook; that is the intended order of the two rows.
- **`loop pr-check --local` still fails `scope` on every PR** until items carry `scope:` lines (Kimi's PR), which the ORDER anticipated.

## Plain summary

- **The gates can now show their progress.** A person starting a full gate run in a terminal can add `--watch`; the runner records each gate's start and finish in a small status file, and a watcher prints one line per change: which gate of how many is running, any red result as it lands, and how long the remaining gates should take based on their last recorded times. It stays out of CI and headless runs, closes when the run ends, and closes itself if the run looks dead for five minutes.
- **The run does not depend on the window.** The gates, their logs and the exit code are identical with the window on, off, crashed or given up.
- **Four warnings became errors.** `loop arrive` now fails instead of warning when the checkout is behind `origin/main`, the binary is older than the checkout's build, the shared git pre-commit hook is missing or different, or the read path is above 75 KB. `loop pr-check` now fails a read-path budget overrun even when the PR did not grow it. The loop relies on these checks at session start and before a push, so they may not be ignored.
- **The new `loop-gates-watch` gate proves the window with its own planted fault**, so a future change that breaks the window turns the gate suite red instead of passing silently.
