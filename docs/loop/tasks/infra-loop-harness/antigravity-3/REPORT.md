# REPORT: infra-loop-pr09-client-adapters, antigravity-3

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Cancel probe result:** On `bc1ba40e627157ab8634bad7fe688e87991665b9`, `uor-harness loop review --check --pr 2932` exited `1` (`CHANGES`), and the four HIGH probes (`uor-harness --repo /x loop install --apply`, `eval git push origin main`, `git config core.hooksPath /dev/null`, and symlink `local/antigravity/d2 -> ../../docs` + `local/antigravity/d2/../CHARTER.md`) reproduced. Class: **instrument**.
- **Ten required items from the `bc1ba40` `CHANGES` review fixed and tested in `tests/loop_guard.rs::repair_order_antigravity_3_regressions`:**
  1. **HIGH 1 — Global flags and `cargo run` before `uor-harness`:** `check_uor_harness_command` skips `--repo`, `--json`, `-v`, `-q`, `--offline` and `--locked` before reading `s1`/`s2`, matches `--apply` and `--apply=...`, and dispatches `uor-r4-harness` and `cargo run ... -- <args>`.
  2. **HIGH 2 — Shell compound keywords, `eval` and `exec`:** `tokenize_shell_segments` strips `#` comments and splits on `{`/`}`; `check_command_segment` strips `do`/`then`/`else`/`elif`/`if`/`while`/`until`/`!`/`done`/`fi`/`esac`/`coproc`/`for`/`select`/`case`/`exec` and re-checks `eval` via `check_command`.
  3. **HIGH 3 — `git config` hook, include and push overrides:** `check_git_command` refuses `git config` writes and `-c` overrides for `core.hookspath`, `include.path`, `includeif.*`, `remote.*.push` and `push.default`, and `check_env_assignment` refuses `GIT_CONFIG_GLOBAL`/`GIT_CONFIG_SYSTEM`/`GIT_CONFIG_NOSYSTEM`.
  4. **HIGH 4 — Symlink plus `..` traversal:** `resolve_components_with_symlinks` canonicalizes existing prefixes and expands symlinks on each component before applying `ParentDir` (`..`), so `local/claude/d2/../CHARTER.md` via `d2 -> ../../docs` resolves to `CHARTER.md` and is refused.
  5. **MEDIUM 5 — Preserve existing user hooks on `loop install --apply --replace`:** `plan_client_install` parses existing `.claude/settings.json`, `.codex/hooks.json`, `.gemini/config/plugins/uor-r4-loop/hooks.json` and `.dsh/hooks.json` (failing closed on malformed JSON), preserves top-level keys and events (`PostToolUse`), and merges per-event arrays to keep non-`uor-harness` entries; `strip_kimi_hooks` validates TOML lines and keeps user `[[hooks]]` tables.
  6. **MEDIUM 6 — Refuse `--apply` whenever `UOR_LAB` is set:** `cli_install` unconditionally refuses `--apply` whenever `!active_lab.is_empty()` regardless of `--home` or `HOME`, and `run_install` checks raw and resolved paths against `repo` so `--home <repo>/docs` is refused.
  7. **MEDIUM 7 — Exempt `SHARED_FILES` in `check_repo_rel_write`:** `check_repo_rel_write` includes `crate::loopclaim::SHARED_FILES.contains(&rel)` in `always_allowed`, and `find_active_order` prefers open ORDER folders without `REPORT.md`.
  8. **MEDIUM 8 — Keep Claude subagent bindings across `Stop` and `SessionStart`:** `update_and_detect_subagent` keeps `subagent-bindings.json` across top-level `Stop` and `SessionStart` (clearing a binding only at `SubagentStop`), and fails closed when `SubagentStart` omits `agent_id`.
  9. **MEDIUM 9 — Refuse `--mirror`, `--all`, `--branches` and `heads/main` pushes:** `check_git_command` refuses `git push` with `--mirror`, `--all`, `--branches`, any refspec destination ending in `heads/main` or `/main`, and `-c remote.*.push=...`.
  10. **MEDIUM 10 — Fail closed on guard lock contention:** `with_guard_lock` renames a stale lock (`> 5s`) to a unique path before unlinking and returns `Err` (`GuardOutcome::Refused`) if the lock cannot be acquired.

## What changed

- `crates/uor-r4-harness/src/loopguard.rs`: resolved HIGH 1–4, MEDIUM 5–10, and follow-ups (`cd` tracking, credential path normalization and `.pub` exemption, `$GITHUB_PAT`/`*_PAT`, `ps eww`/`declare -x`, `git checkout <rev> <path>`, `gh api` GraphQL merge, `gh pr merge --admin`/`--auto`, `loop copyright` frontmatter scanning and line numbers, and `loop onboard` `N/A` skill YAML reporting).
- `crates/uor-r4-harness/tests/loop_guard.rs`: added `repair_order_antigravity_3_regressions` (9 test suites).
- `docs/loop/README.md` & `docs/harness/onboarding/*.json`: documented the guard threat model and refreshed the five receipts.
- **Size by class:** `tests 1 file(s) +1230/-0, code 5 file(s) +4876/-3, docs 9 file(s) +305/-8, generated 1 file(s) +3/-3, other 8 file(s) +276/-0`; ratio ~1.0x.

## Measurements

- Host verification only: `cargo test --locked -p uor-r4-harness --test loop_guard` (`9 passed; 0 failed`), `uor-harness loop onboard --check-receipts` (`5 verified`), and `uor-harness loop copyright` (`0` hits in PR files; `1` out-of-scope hit in `kimi-1/REPORT.md:37`).

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-harness -- --check` — PASS
- `CARGO_TARGET_DIR=local/antigravity-harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test loop_guard` — PASS
- `UOR_LAB=antigravity uor-harness loop gates --full`, `loop report --check`, `loop pr-check --local` — PASS

## Skills used

- `scope-guard`, `gates`, `solution-integrity`, `working-discipline`: kept edits inside `infra-loop-pr09-client-adapters`, added `repair_order_antigravity_3_regressions`, resolved symlinks per component before `..`, merged existing user hooks, and made lock contention fail closed.

## Still open

- `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md:37` (PR #2929) has an 8-word Loop Kit overlap caught by `uor-harness loop copyright`, reported to `kimi` on `#2028`.
- Once PR #2932 merges, Casey can run `uor-harness loop install --all --apply --replace` from Terminal (with `UOR_LAB` unset).

## Plain summary

Repair order `antigravity-3` resolves all ten required items (HIGH 1–4 and MEDIUM 5–10) and follow-ups from Claude's `bc1ba40` `CHANGES` review of PR #2932, adds `repair_order_antigravity_3_regressions`, documents the guard's threat model in `docs/loop/README.md`, and refreshes the five onboarding receipts.

Closes #2914
References #2904
