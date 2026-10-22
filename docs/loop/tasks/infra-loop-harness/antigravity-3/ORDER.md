# ORDER: infra-loop-pr09-client-adapters, antigravity-3 (repair: ten required items from bc1ba40 review)

Lab: antigravity. Item: [infra-loop-pr09-client-adapters](../../../../roadmap/items/infra-loop-pr09-client-adapters.md) (#2914). Setup row: 9. Base: `a8ff5690e` (`origin/main`), repairing `antigravity-2` (PR #2932) after the counted `CHANGES` record at `bc1ba40e627157ab8634bad7fe688e87991665b9` (`claude`, `uor-review v1`, fresh context).

## Goal

Fix all ten required items from the `bc1ba40` `CHANGES` review record (plus cheap follow-ups) and state the guard's threat model in `docs/loop/README.md`:
1. **HIGH 1 — Global flags and `cargo run` before `uor-harness`:** Skip global flags (`--repo`, `--json`, `-v`, `-q`) before reading `uor-harness` subcommands, treat `cargo run ...` invoking `uor-r4-harness`/`uor-harness` as `uor-harness`, and match `--apply` or `--apply=...`.
2. **HIGH 2 — Shell keywords, `eval` and `exec`:** Strip `do`, `then`, `else`, `elif`, `if`, `while`, `until`, `for ... in ...`, `case ... in`, `{`, `}`, `!`, `exec` and `coproc` prefixes, and recursively re-check `eval`'s joined arguments through `check_command`.
3. **HIGH 3 — `git config` hook/include/push overrides:** Refuse any `git config` write whose key is `core.hookspath`, `include.path`, `includeif.*.path` or `remote.*.push` (any case), plus `-c include*.path=...`, `-c remote.*.push=...`, and `GIT_CONFIG_GLOBAL`/`GIT_CONFIG_SYSTEM`.
4. **HIGH 4 — Symlink plus `..` path resolution:** Canonicalize the longest existing prefix on disk before applying remaining relative components so `local/antigravity/d2/../CHARTER.md` via a symlink resolves to `CHARTER.md` and is refused.
5. **MEDIUM 5 — Preserve existing user hooks on `loop install --apply --replace`:** Merge each hook event's array in `~/.claude/settings.json`, `~/.codex/hooks.json` and `~/.dsh/hooks.json` (keeping non-`uor-harness` entries), strip only `uor-harness` `[[hooks]]` tables in `~/.kimi-code/config.toml`, and return `Err` if an existing config file fails to parse.
6. **MEDIUM 6 — Refuse `--apply` whenever `UOR_LAB` is set:** Refuse `--apply` unconditionally whenever `UOR_LAB` is set, and canonicalize `--home` before the repo-local check.
7. **MEDIUM 7 — Exempt `SHARED_FILES` in `check_repo_rel_write`:** Include `crate::loopclaim::SHARED_FILES.contains(&rel)` in `always_allowed`.
8. **MEDIUM 8 — Keep Claude subagent bindings across `Stop` and `SessionStart`:** Clear a binding only at that agent's `SubagentStop`.
9. **MEDIUM 9 — Refuse `--mirror`, `--all` and `heads/main` pushes:** Refuse `git push` with `--mirror` or `--all`/`--branches`, any destination ending in `heads/main`, and `-c remote.*.push=...`.
10. **MEDIUM 10 — Fail closed on guard lock contention:** Return `Err` when the lock cannot be acquired, and rename a stale lock to a unique path before unlinking.
11. **Docs & follow-ups:** State the guard's threat model in `docs/loop/README.md`, and cover the cheap follow-up items listed in the `bc1ba40` review.

## Cancel probe (run first)

Is `bc1ba40` still `CHANGES` on PR `#2932` and do `uor-harness --repo /x loop install --apply`, `eval git push origin main`, `git config core.hooksPath /dev/null` and the `d2 -> ../../docs` symlink traversal still exit `0` on `bc1ba40`? Result: yes (`loop review --check --pr 2932` exits `1`, and all four HIGH probes reproduce). Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs`, `crates/uor-r4-harness/tests/loop_guard.rs`, `docs/harness/onboarding/**`
- `docs/loop/README.md`, `docs/harness/README.md` (shared docs)
- `docs/loop/tasks/infra-loop-harness/antigravity-{1,2,3}/**`

## Done when

- All 10 required items have negative tests in `tests/loop_guard.rs` that fail on `bc1ba40` and pass on HEAD, and `loop gates --full`, `loop report --check` and `loop pr-check --local` pass.

## Stop and ask if

- Any file outside the scope above must change.
