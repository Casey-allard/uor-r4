# REPORT: infra-loop-pr09-client-adapters, antigravity-2

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Round 1 (`antigravity-1`, `e7b768e04`) exact-head review findings (CHANGES on PR #2932):**
  - **HIGH 1 — Regex command matcher in `loop guard`:** `check_command` in `e7b768e04` matched raw shell substrings with narrow regexes (`\bgit\s+(?:-C\s+\S+\s+)?...`), which missed `git -c core.hooksPath=/dev/null commit -m x`, `git push origin feat:main`, `/usr/bin/pkill foo`, `git -c a=b push --force`, `git push -u origin main`, and `git commit -nm x`. Replaced with a shell segment tokenizer (`tokenize_shell_segments` and `check_command_segment`) that splits pipelines and subshells, strips path prefixes and wrappers (`env`, `command`, `nohup`, `time`, `nice`, `sudo`, `timeout`, `stdbuf`, `xargs`, `sh -c`), skips all `git` global flags while refusing `-c core.hooksPath=...`, checks `git commit` short flag clusters for `-n`, and parses `git push` refspecs and options (`-u`, `--set-upstream`, `--delete`, `src:dst`) while permitting `--force-with-lease`.
  - **HIGH 2 & Owner decisions on `#2028` — `loop install` plan-only default and `git install-hooks` refusal:** `loop install` wrote user-level files under `~` whenever `--dry-run` was omitted. Per Casey's two owner decisions on `#2028`, `loop install` now prints the planned write list by default without writing any file, requires `--apply` (`uor-harness loop install --all --apply --replace`) to write, refuses `--apply` against `$HOME` when `UOR_LAB` is set, and `loop guard` refuses both `uor-harness loop install --apply` and `uor-harness git install-hooks` from lab sessions.
  - **MEDIUM 1 — `gh` merge and credential/environment checks:** Replaced fixed-spacing `"gh pr merge"` and `cat`-only credential regexes with tokenized `check_gh_command` (refusing `gh pr merge` without a 40-hex `--match-head-commit` and `gh api .../merge`), universal credential-path checks across all readers and flags (`~/.netrc`, `~/.git-credentials`, `.env*`, `auth.json`, `credentials.json`, `~/.aws/credentials`, `~/.runpod/config.toml`, `~/.vast_api_key`, `~/.config/gh/hosts.yml`, `~/.npmrc`, `~/.pypirc`, `~/.cargo/credentials*`, `~/.ssh/id_*`), and environment dump refusals (`printenv`, bare `env`, `export -p`, `declare -p`, bare `set`, and `$...KEY...`/`$...TOKEN...` expansions).
  - **MEDIUM 2 — Historical ORDER folders in `find_active_order`:** Scanning all `docs/loop/tasks/<folder>/<lab>-<n>/ORDER.md` picked closed historical folders on `origin/main` (such as `claude-20`). Updated `find_active_order` to resolve only from `local/<lab>/guard/active-order` or an ORDER added/modified on the branch relative to `origin/main` (or an open folder without `REPORT.md` in standalone test fixtures).
  - **MEDIUM 3 & MEDIUM 4 — Subagent bindings, Antigravity session state, and advisory locking:** Added `with_guard_lock` (`local/<lab>/guard/.lock`) and `atomic_write`, bound Claude Code `agent_id`s at `SubagentStart` in `local/<lab>/guard/subagent-bindings.json` without falling back to the highest-numbered `<n>.md` for unbound subagents, and tracked Antigravity session state in `antigravity-session.json` so a killed root session is replaced on the next top-level `SessionStart` and `PreToolUse` without a prior `SessionStart` fails closed as a subagent.
  - **MEDIUM 5 — Shell file-modifying commands:** Added `check_shell_file_mods` and `git rm`/`git mv`/`git restore`/`git checkout --` path validation so `rm`, `cp`, `mv`, `install`, `ln`, `touch`, `truncate`, `mkdir`, `chmod`, `chown`, `sed -i`, `perl -i`, `ruby -i`, and `dd of=...` are checked through `check_write_path`.
  - **LOW 1, LOW 2, LOW 3:** Allowed `git push --force-with-lease`, checked `find_uor_roots` before `--client` or JSON validation so hooks outside `~/uor-r4` always exit `0`, and updated `docs/harness/onboarding/*.json` to report `N/A` for hooks (and `codex` skills) pending the owner's `uor-harness loop install --all --apply --replace` while stating explicitly that the canary is evaluated in-process against `loop guard`.
- **Planted fault verification (canary write allowed):** Temporarily bypassing the `CHARTER.md` write check in `evaluate_guard` turned `tests/loop_guard.rs::each_clients_canary_write_is_refused` red with `canary for antigravity was not refused: canary write to CHARTER.md for client antigravity was ALLOWED instead of refused` (exit 101). Restoring the path guard returned all 8 tests in `tests/loop_guard.rs` to green.
- **Live `loop copyright` catch on `origin/main` (`kimi-1/REPORT.md`):** Running `uor-harness loop copyright` after merging `origin/main` caught 1 real 8-word Loop Kit overlap in `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md:37` (word offset 554, overlapping `LoopKit/skills/scope-guard/SKILL.md`, landed in PR #2929). Because `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md` is outside `infra-loop-pr09-client-adapters`'s scope (`uor-harness loop pr-check --local` refuses edits to another lab's task folder), left `kimi-1/REPORT.md` untouched in this PR and flagged it under "Still open" for Kimi or the owner.

## What changed

- Updated `crates/uor-r4-harness/src/loopguard.rs` to implement tokenized command analysis (`tokenize_shell_segments`, `check_command_segment`, `check_git_command`, `check_gh_command`, `check_uor_harness_command`, `check_uor_pod_command`, `check_shell_file_mods`), plan-only default `loop install` with owner-only `--apply`, branch-diff active ORDER resolution (`find_active_order`), locked atomic subagent state and bindings (`with_guard_lock`, `atomic_write`, `claim_next_suborder_for_agent`), early outside-repo exit in `evaluate_guard` and `cli_guard`, and `OnboardVerdict::NotApplicable` (`N/A`) in `loop onboard`.
- Added `docs/loop/tasks/infra-loop-harness/antigravity-2/{ORDER.md,REPORT.md}` alongside `antigravity-1/{ORDER.md,REPORT.md}` as the repair chain for PR #2932.
- Expanded `crates/uor-r4-harness/tests/loop_guard.rs` (8 integration tests) covering all review findings and owner decisions.
- Regenerated `docs/harness/onboarding/{antigravity,claude,codex,deepseek,kimi}.json` and updated `docs/harness/README.md` and `docs/loop/README.md` with the full `loop install` file inventory and `--apply` owner workflow.
- **Size by class:** `tests 1 file(s) +877/-0, code 5 file(s) +4375/-3, docs 6 file(s) +252/-6, generated 0 file(s) +0/-0, other 5 file(s) +270/-0`; ratio ~1.0x against the five-client adapter and repair scope.

## Measurements

- Host verification only (no GPU or model evaluation compute):
  - `uor-harness loop copyright` on the laptop against `~/Downloads/LoopKit-2026-10-12-v9 (1).zip`: `0` matching 8-word windows across all files touched in this PR (`antigravity-1` and `antigravity-2`), and `1` out-of-scope hit caught in `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md` (word offset 554, landed on `main` in PR #2929).
  - `uor-harness loop onboard --check-receipts`: `5` client receipts verified (`antigravity`, `claude`, `codex`, `deepseek`, `kimi`), all `5` in-process canary writes to `CHARTER.md` refused.
  - `cargo test --locked -p uor-r4-harness --test loop_guard`: `8 passed; 0 failed`.
  - Done-when probe on `uor-harness loop guard --client claude --lab claude --event PreToolUse`: `--no-verify`, `pkill`, `git -c core.hooksPath=/dev/null commit -m x`, `git push origin feat:main`, `/usr/bin/pkill foo`, `git -c a=b push --force`, `git push -u origin main`, `git commit -nm x`, `uor-harness loop install --all --apply`, and `uor-harness git install-hooks` all exited `2`.

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-harness -- --check` — PASS (exit 0)
- `CARGO_TARGET_DIR=local/antigravity-harness-target ~/.cargo/bin/cargo clippy --locked -p uor-r4-harness --all-targets -- -D warnings` — PASS (exit 0)
- `CARGO_TARGET_DIR=local/antigravity-harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test loop_guard` — PASS (8 passed, 0 failed)
- `UOR_LAB=antigravity ./local/antigravity-harness-target/release/uor-harness loop onboard --check-receipts` — PASS (5 verified and canary writes refused)
- `UOR_LAB=antigravity ./local/antigravity-harness-target/release/uor-harness loop copyright` — `0` hits across all files in this PR (`1` out-of-scope hit flagged in `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md:37`)
- `UOR_LAB=antigravity ./local/antigravity-harness-target/release/uor-harness loop gates --full` — PASS (5/5 gates green)
- `UOR_LAB=antigravity ./local/antigravity-harness-target/release/uor-harness loop report --check` — PASS
- `UOR_LAB=antigravity ./local/antigravity-harness-target/release/uor-harness loop pr-check --local` — PASS

## Skills used

- `scope-guard`: kept all edits inside the declared scope of `infra-loop-pr09-client-adapters` and retained `antigravity-1/{ORDER.md,REPORT.md}` alongside `antigravity-2/{ORDER.md,REPORT.md}` per `3994731aa`.
- `gates`: verified the planted fault on `run_canary` and added regression tests in `tests/loop_guard.rs` for every `CHANGES` finding.
- `solution-integrity`: replaced regex command matching with tokenized shell segment analysis and ensured `find_active_order` never mistakes closed historical task folders on `origin/main` for an active ORDER.
- `working-discipline`: paused for Casey's two owner decisions on `#2028` before changing `loop install --apply` and `git install-hooks` behavior, and verified every command from the review's done-when probe.

## Still open

- `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md:37` (merged in PR #2929) contains an 8-word Loop Kit overlap caught by `uor-harness loop copyright` (`word offset 554`, overlapping `LoopKit/skills/scope-guard/SKILL.md`); it is outside `infra-loop-pr09-client-adapters`'s scope and should be rephrased by `kimi` or the owner.
- Codex's live P-1 and P-2 probe execution on `#2028` remains owed by `codex`; `loop guard`, `loop install` and `loop onboard` support `codex` and keep subagents without a suborder read-only.
- Once PR #2932 merges, Casey can run `uor-harness loop install --all --apply --replace` from Terminal (with `UOR_LAB` unset) to write the user-level hooks, rules and normalized skills across all five clients.

## Plain summary

This repair round (`antigravity-2`) resolves all findings from the first exact-head review of PR #2932 (`e7b768e04`) and implements Casey's two owner decisions from `#2028`. In `uor-harness loop guard`, shell commands are now tokenized across pipelines, subshells and command wrappers rather than matched by fixed regexes, so global `git` flags (`git -c core.hooksPath=/dev/null commit`, `git -c a=b push --force`, `git push -u origin main`, `git push origin feat:main`, `git commit -nm x`), path-prefixed binaries (`/usr/bin/pkill`), environment dumps, arbitrary credential-file reads, and file-modifying shell commands (`rm`, `cp`, `mv`, `sed -i`, `dd of=...`, `git checkout --`) are all caught while `git push --force-with-lease` on a feature branch is allowed. Active ORDER lookup now ignores closed historical task folders on `origin/main`, subagent state and suborder bindings use an advisory lockfile and atomic renames, and hooks firing outside `~/uor-r4` exit `0` before any client or JSON failure. Finally, `uor-harness loop install` is now plan-only by default and requires `--apply` (which is refused from lab sessions alongside `uor-harness git install-hooks`), and the five committed onboarding receipts in `docs/harness/onboarding/` record `N/A` for hooks not yet applied by the owner and state that the canary is evaluated in-process against `loop guard`.

Closes #2914
References #2904
