# REPORT: infra-loop-pr04-gates, claude-13 (loop PR 4: loop gates runner, gates.json, ledger and breaks)

Lab: claude. Base: `e61746628`. Order: [ORDER.md](ORDER.md), committed as `918a845a4` before the work. Its cancel probe found no `gates.json` and no `loop` subcommand.

Fresh-context Opus builders wrote the code: one for the first head, then one builder per commit for each review fix. The main session re-ran every gate. It also wrote this REPORT, because a harness hook does not let subagents write REPORT files.

## What did not work, first

- **No `sha2`.** Adding it would change `Cargo.lock`, which is outside the ORDER's files. Content hashes use `git hash-object --stdin` instead, so the PR adds no dependency. For the same reason there is no `libc`, and so no `O_NOFOLLOW` (see Still open).
- **`cargo xtask links` cannot run as one gate.** CI runs it at the base to record a baseline, then again with `--baseline`. The tree already has 931 broken links, so a planted broken link stayed green, which is a gate defect. Links stays out of `gates.json` until the runner can carry a base baseline.
- **The `xtask` cargo alias does not resolve in a scratch worktree.** Gate commands use `cargo run -q --locked -p <package>` instead.
- **The first head failed the full harness suite**, with 678 passed and 1 failed. The `harness.documented_commands` rule found new usage lines naming `uor-harness loop gates`, a command its tables did not know. The main session registered `loop` and its `gates` action.
- **Round 1: two fresh-context reviews of `e7986fcef` asked for changes.** They were fixed in three commits, one builder each:
  - **Containment (`0f3cbccb2`).**
    - A break's `file` was joined to the scratch path unchecked, and gate ids were used in file paths unchecked. Absolute, `..` and `.` paths and malformed ids are now refused when `gates.json` loads.
    - Each ledger row is one `write_all`.
    - An empty selection exits 2.
    - The repo-wide `git worktree prune` was dropped.
  - **Verdict precision (`7495beef6`).**
    - A break was CAUGHT whenever the gate went red with text that a parse error, or an already-red tree, would also print. A break now runs the gate once before planting the fault. If the gate is already red, the result is REFUSED.
    - Each `expect` is text only the planted fault produces: `+fn planted_break() {}` for the two fmt gates, and for claim-wording the planted sentence itself (see `docs/harness/gates.json`).
    - The working-tree diff no longer depends on an external diff tool, textconv or colour settings (`--no-ext-diff --no-textconv --no-color`). Other diff settings, such as rename detection, can still shape it.
    - Hash sections are length-prefixed, and a missing file is marked distinctly from an empty one.
    - A gate whose `reads` match no file is REFUSED.
    - A death by signal records `signal N`.
    - `--break` refuses an uncommitted `gates.json`.
  - **Tests and tidy (`4f8e5b404`).**
    - The `--break all` test uses a distinct marker per gate.
    - The `--watch` test asserts the message that points to PR 11.
    - New tests cover a missing break, the real `gates.json`, `--quick`, the ledger fields and the `--keep-going` exit code.
- **Round 2: a fresh-context delta review of `5aac998bb` found that the round-1 sweep removed live scratch worktrees, including other labs'.**
  - The sweep matched any lab and any run. The reviewer reproduced it: a Claude `--break` force-removed a registered Kimi worktree that held an untracked file.
  - The same review found that no test reached the symlink guard or the sweep.
  - Fixed in `fb9aaf335`:
    - The sweep now removes a scratch worktree only when three things hold: its path names this lab; its run's pid is not this process; and `kill -0` reports "No such process" for that pid.
    - New tests run the symlink guard and the sweep. Each guard's negative gate fails its test.
    - Refusals now happen before the run folder is created.
    - The containment checks run right before the write, after the pre-check.
    - The pre-check keeps its own log, `<id>.pre.log`.
    - A pre-check refusal now reads "not green".

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopgates.rs` (new), `lib.rs`, `main.rs`, `doccheck.rs` | code | 908 | `uor-harness loop gates [--quick\|--full\|--keep-going\|--status\|--break G\|all]`. Other `loop` actions and `--watch` exit 2, and `--watch` points to loop PR 11. A bad `UOR_LAB` exits 2 through `lab_or_refuse`. `doccheck.rs` registers `loop`, `gates` and five flags. |
| `crates/uor-r4-harness/tests/loop_gates_run_rules.rs` (new) | tests | 609 | 29 tests over temporary git repositories. Every `GIT_*` variable is stripped, and each repository is asserted to sit inside its own temp directory. |
| `docs/harness/gates.json` (new) | data | 41 | Schema version 1: `id`, `proves`, `command` (argv), `reads` (globs), `quick`, and `break {file, append, expect}`. Gates: `fmt-claim-wording`, `fmt-harness`, `claim-wording`. |
| `docs/harness/README.md`, `docs/loop/README.md` | docs | 13 | Usage, verdicts, and the loop page's runner line. |

That is about 1,570 lines, not counting this ORDER and REPORT: about 1.75 times the 900-line estimate, after two review rounds of fixes.

**Design calls:**
- **Run verdicts.** Each gate ends as one of:
  - GREEN;
  - RED;
  - REFUSED: the gate cannot run, or its reads match no file;
  - UNKNOWN: an earlier gate was not green and `--keep-going` was not given;
  - VOID: its read files changed during the run.

  Any result other than GREEN stops the run.
- **Break verdicts.**
  - CAUGHT: green before the break, then red with the declared text once the fault is planted.
  - REFUSED: not green before the break, or red for another reason.
  - DEFECT: still green.
- **Exit codes.**
  - 0 when every gate is green or caught.
  - 1 otherwise.
  - 2 for a usage error, a bad lab, an unreadable or invalid `gates.json`, an empty selection, or `--break` with an uncommitted `gates.json`. These refusals create no run folder.
- **Logs and ledger.**
  - Logs and the true exit code or signal go under `local/<lab>/gates/<ts>-<pid>/`. The pre-check writes `<id>.pre.log` and `<id>.pre.exit`.
  - The ledger, `local/<lab>/gates.jsonl`, gets one row per run gate: gate, verdict, seconds, commit, working-tree hash, exit code, signal and timestamp.
  - `--status` shows the latest row for each gate, marked STALE when the working-tree hash differs.
- **Hashes.** Every section is length-prefixed, and a missing file is marked distinctly.
  - The working-tree hash covers `HEAD`, `status --porcelain=v1 -z --untracked-files=all`, `diff HEAD --binary --no-ext-diff --no-textconv --no-color`, and the untracked, unignored files.
  - The read-file hash covers every tracked file, and every untracked unignored file, that matches the gate's `reads` globs.
- **Breaks.**
  - A break appends to a file in a detached scratch worktree at `local/<lab>/gates/<run>/scratch/wt-<id>`.
  - Absolute, `..` and `.` paths are refused when the file loads.
  - Right before the write, a symlink target is refused, and so is any target whose canonical parent lies outside the scratch worktree.
  - A Drop guard removes the scratch worktree.
- **Sweep.** At the start of `--break`, the runner removes only this lab's scratch worktrees whose run pid is dead. That means `kill -0` says "No such process", and the pid is not this process's own. Nothing else is touched.
- **Finding cargo.** A leading `cargo` resolves to `$CARGO`, then `~/.cargo/bin/cargo`, then `PATH`.
- **Environment.** Gates and the runner's git calls run without `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE`.

## Measurements

None. This is a host-only tool.

## Checks run at the head

The main session re-ran each of these at `fb9aaf335` after the builders. Every later commit changes this REPORT only.

- `cargo test -q --locked -p uor-r4-harness --test loop_gates_run_rules`: 29 passed.
- **Negative gates.** Each one set aside only its own fix's lines, rebuilt, and restored a byte-identical copy afterwards. In each case the named test failed with the lines set aside and passed once they were restored:
  - UNKNOWN after red: `later_gates_read_unknown_after_a_red`.
  - Load-time containment: `a_break_file_outside_the_tree_exits_2`.
  - The green-at-HEAD pre-check: `a_break_on_a_gate_already_red_at_head_is_refused`.
  - The `--watch` refusal: `watch_and_other_actions_are_refused`.
  - The sweep's lab condition, which the main session also re-ran: `the_sweep_removes_only_this_labs_dead_runs_scratch_worktrees`.
  - The sweep's pid condition: the same test.
  - The symlink refusal: both symlink tests. With the refusal set aside, the file outside the repository changed.
- **A shared worktree would be caught.** Planting gate 1's marker into gate 2 turned gate 2 REFUSED and failed `break_all_turns_each_gate_red_for_its_own_reason`.
- `UOR_LAB=claude uor-harness loop gates --quick`: all three gates GREEN.
- `UOR_LAB=claude uor-harness loop gates --break all`: all three CAUGHT, each with its own text. No scratch worktree was left under `local/`.
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: 699 passed, 0 failed.
- `cargo fmt -p uor-r4-harness --check` is clean. Clippy shows no new warnings in the touched files.
- Claim-wording passed. `xtask links` found 0 broken links in the changed files. The local copyright check against the Loop Kit found 0 matching 8-word runs.

## Skills used

None of the private kit's skills.

## Still open

- **A check-to-open race remains for the break file.** Without `O_NOFOLLOW`, a gate command could swap the break file for a symlink between the containment check and the open. The check runs immediately before the open, and a test pins a swap made during the pre-check. `O_NOFOLLOW` would close the swap of the final path component; it needs `libc`, which is a `Cargo.toml` change. A swap of a parent directory between the canonical-parent check and the open would need `openat`-style resolution.
- **Gates still to add to `gates.json`:**
  - clippy and library tests for changed crates;
  - `uor-harness check`, `lint check` and `shard-check`;
  - the d23, d25 and d32 ratchets;
  - `archive-verify --base`, `roadmap check` and `docs-catalog --check`;
  - links, which needs a base-baseline mode.

  A "create file" break kind may be needed.
- **Old breaks against new gates.** A changed gate's old breaks do not yet run against the new gate. That needs the merge base's `gates.json`.
- **A pid can be reused.** If a dead run's pid is later taken by an unrelated live process, that run's scratch worktree is never swept. That is the safe direction; such a worktree can be removed by hand.
- **VOID compares snapshots.** It compares a snapshot taken at the start of the run with one at the end, so a file that changes and changes back during the run is not seen.
- **Signal deaths in the summary.** The run summary prints `exit -` for a signal death. The signal is in the reason text and in the ledger.
- **`--watch`** is loop PR 11.

## Plain summary

`uor-harness loop gates` runs the checks listed in `docs/harness/gates.json`. For each one it keeps the true exit code and the log, and records a ledger row that is marked STALE once uncommitted edits appear. It stops at the first result that is not green, unless told to keep going. A gate whose input files changed during the run is voided, and one whose inputs match nothing is refused.

`--break all` copies the repository into a throwaway worktree, with checks that keep each write inside it. There it confirms each gate is green, plants that gate's declared fault, and confirms the gate then turns red with that fault's own message. Leftover throwaway worktrees are cleaned up only when they belong to this lab and to a run that is no longer alive. Tests pin each of these rules.
