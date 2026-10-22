# ORDER: infra-loop-pr09-client-adapters, antigravity-2 (repair: guard tokenization, subagent binding, plan-only install)

Lab: antigravity. Item: [infra-loop-pr09-client-adapters](../../../../roadmap/items/infra-loop-pr09-client-adapters.md) (#2914). Setup row: 9. Base: `a8ff5690e` (`origin/main`), repairing `antigravity-1` (PR #2932) after the counted `CHANGES` record at `e7b768e043349bc7456ebfb4ab1082949d85a963` (`claude`, `uor-review v1`, fresh context).

## Goal

Address every HIGH, MEDIUM and LOW finding in the `e7b768e04` `CHANGES` review record and build Casey's two owner decisions from `#2028`:
- **HIGH 1:** Tokenize shell commands across `;`, `|`, `||`, `&&`, `&`, newlines, `(...)`, `$(...)`, backticks, env assignments and wrappers (`env`, `command`, `nohup`, `time`, `nice`, `sudo`, `sh -c`, `xargs`), strip `argv[0]` path prefixes (`/usr/bin/pkill`), skip `git` global options (`-C`, `-c`, `--git-dir`, `--work-tree`, `--config-env`), refuse `-c core.hooksPath=...`, `git commit -n` (`-nm x`), and any `git push` refspec targeting `main` (`feat:main`, `-u origin main`, `HEAD:refs/heads/main`, `:main`).
- **HIGH 2:** Make `loop install` plan-only by default (writing only with `--apply`), refuse `loop install --apply` against `$HOME` when `UOR_LAB` is set, refuse `uor-harness loop install --apply` and `uor-harness git install-hooks` in `loop guard`, and document every user-level path (`~/.claude/{settings.json,CLAUDE.md,skills/*}`, `~/.codex/{hooks.json,rules/uor-r4.rules,skills/*}`, `~/.kimi-code/{config.toml,UOR-R4-START.md,UOR-R4-HANDOFF.md}`, `~/.agents/skills/*`, `~/.gemini/config/{plugins/uor-r4-loop/*,skills/*}`, `~/.dsh/{hooks.json,profiles/headless/cordis.patch.yml,skills/*}`, `local/deepseek/headless-overlay.yml`).
- **MEDIUM 1–5:** Tokenized `gh pr merge` / `gh api .../merge` checks; universal credential-path and env-dump (`printenv`, bare `env`, `export -p`, `declare -p`) refusals; `find_active_order` restricted to `local/<lab>/guard/active-order` or the branch's added ORDER relative to `origin/main`; Claude `agent_id` bound to numbered `orders/<n>.md` at `SubagentStart`; Antigravity `SessionStart` replacement and fail-closed `PreToolUse` with advisory lockfile + atomic writes; shell file-modifying commands (`cp`, `mv`, `rm`, `sed -i`, `dd of=`, `git rm`, `git mv`, `git restore`, etc.) checked via `check_write_path`.
- **LOW 1–3:** Allow `git push --force-with-lease`; check `find_uor_roots` before `--client` or JSON validation so hooks outside `~/uor-r4` always exit `0`; record `N/A` in `docs/harness/onboarding/*.json` for unapplied hooks/skills and state that the canary runs in-process on `loop guard`.

## Cancel probe (run first)

Is `e7b768e04` still `CHANGES` on PR `#2932` and do `git -c core.hooksPath=/dev/null commit -m x`, `git push origin feat:main` and `/usr/bin/pkill foo` still exit `0` on `e7b768e04`? Result: yes (`uor-harness loop review --check --pr 2932` exits `1`, and all three probes exit `0` at `e7b768e04`). The repair stands. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopguard.rs`
- `crates/uor-r4-harness/tests/loop_guard.rs`
- `docs/harness/onboarding/**`
- `crates/uor-r4-harness/src/{lib,main,loopgates,doccheck}.rs` (shared registration files)
- `docs/harness/README.md`, `docs/loop/README.md`, `docs/roadmap/README.md` (shared docs)
- `docs/roadmap/items/infra-loop-pr09-client-adapters.md` (`status: done`)
- `docs/loop/tasks/infra-loop-harness/antigravity-1/**`
- `docs/loop/tasks/infra-loop-harness/antigravity-2/**`

## Done when

- All HIGH, MEDIUM and LOW items from `e7b768e04` and Casey's two owner decisions are built and tested in `crates/uor-r4-harness/tests/loop_guard.rs`.
- `uor-harness loop gates --full`, `uor-harness loop onboard --check-receipts`, `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass, and `uor-harness loop copyright` finds `0` hits across all files in this PR's scope.

## Stop and ask if

- Any file outside the scope above must change.
