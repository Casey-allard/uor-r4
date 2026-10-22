# REPORT: infra-loop-pr09-client-adapters, antigravity-1

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Planted fault verification (canary write allowed):** Temporarily bypassing the `CHARTER.md` write check in `evaluate_guard` turned `tests/loop_guard.rs::each_clients_canary_write_is_refused` red with `canary for antigravity was not refused: canary write to CHARTER.md for client antigravity was ALLOWED instead of refused` (exit 101). Restoring the path guard returned all 7 tests in `tests/loop_guard.rs` to green.
- **Outside-repository `UOR_LAB` resolution:** Calling `lab_or_refuse(repo, args)` in `main.rs` before dispatching `loop guard` would look up `docs/harness/labs.json` in the caller's working directory and fail with exit 2 whenever a user-level hook fired in another project outside `~/uor-r4`. Fixed in `main.rs` and `loopguard.rs` so `loop guard` checks `in_uor_repo` first and exits `0` immediately outside `~/uor-r4` and its worktrees, then validates `UOR_LAB` / `--lab` against `docs/harness/labs.json` once inside the repository.
- **`default_kit_zip` under `--home` override:** When `loop install` ran with `--home <scratch>` and no explicit `--zip`, looking only in `<home>/Downloads` missed the owner's archive in `$HOME/Downloads`. Fixed `default_kit_zip` to check `--home/Downloads` first and fall back to `$HOME/Downloads`.
- **Live user-level skill frontmatter (`gates/SKILL.md`):** Running `loop onboard --client <C>` against the live `$HOME` skills directories installed during hand-join surfaced `FAIL skills: unquoted YAML colon in description of gates` in `antigravity`, `claude`, `deepseek` and `kimi`, `FAIL skills: missing skill(s)` in `codex`, and `REFUSED hooks` on Kimi's retired second loop hook in `~/.kimi-code/config.toml`. `loop install` now quotes unquoted `description:` values containing `: `, skips `start-here`, and strips Kimi's retired hook when the owner runs `loop install --all --replace`.
- **Roadmap render check in `docs.yml`:** Setting `status: done` in `docs/roadmap/items/infra-loop-pr09-client-adapters.md` without regenerating `docs/roadmap/README.md` failed `cargo xtask roadmap render --check` in the `docs archive and links` workflow on `8b20de895`. Ran `cargo xtask roadmap render` to keep `docs/roadmap/README.md` in sync.

## What changed

- Added `crates/uor-r4-harness/src/loopguard.rs` implementing:
  - `uor-harness loop guard --client <C> [--lab <L>] --event <E> [--input <path>]`: single hook entry point for `antigravity`, `claude`, `codex`, `deepseek` and `kimi`; exits `0` immediately outside the UOR-R4 repository and its worktrees; tracks per-client subagent state from P-1 probe shapes (`agent_id` in Claude Code, root vs child `conversationId` in Antigravity, `SubagentStart`/`SubagentStop` depth per `session_id` in Kimi Code); refuses writes to owner-only paths (`docs/harness/owner-paths.json`), the shared primary checkout (R10), other labs' worktrees and `local/` folders, rendered views outside `kimi/render`, existing `archive/` files, and paths outside the active ORDER or subagent suborder (`local/<lab>/<topic>/orders/<n>.md`); refuses owner release and GPU cap-override commands (D56), forbidden git/process/credential commands, and subagent PR/push/claim commands.
  - `uor-harness loop install (--client <C> | --all) [--home <dir>] [--zip <path>] [--replace] [--dry-run] [--json]`: lists all 42 planned writes across the five clients, installs the six allowed Loop Kit skills (`gates`, `holding-the-goal`, `scope-guard`, `session-close`, `solution-integrity`, `working-discipline`; skipping `start-here` and quoting unquoted YAML `description:` colons), writes Kimi's `<= 1` KB start and handoff pointers while retiring its second loop hook, writes only to gitignored `local/` inside the repository, refuses to overwrite differing files without `--replace`, and backs up replaced files to `local/loop/install-backups/`.
  - `uor-harness loop onboard (--client <C> | --check-receipts) [--home <dir>] [--write-receipt] [--json]`: checks `rules`, `skills`, `hooks`, `canary`, `identity` and `binary` (`PASS`, `FAIL`, `REFUSED`), prints `lane:` and `pickup:`, writes committed receipts to `docs/harness/onboarding/<client>.json`, and verifies all five receipts and live canary refusals with `--check-receipts`.
  - `uor-harness loop copyright [--zip <path>] [--json]`: laptop-only 8-word window check comparing tracked repository markdown and text files against the owner's private Loop Kit zip in memory, storing no kit text or hashes in the repository.
- Wired `loopguard` and its flags in `crates/uor-r4-harness/src/{lib.rs,loopgates.rs,main.rs,doccheck.rs}`.
- Added 7 integration tests in `crates/uor-r4-harness/tests/loop_guard.rs`.
- Added committed onboarding receipts in `docs/harness/onboarding/{antigravity,claude,codex,deepseek,kimi}.json`.
- Documented the four commands in `docs/harness/README.md` and marked row 9 built in `docs/loop/README.md`.
- **Size by class:** `tests 1 file(s) +673/-0, code 5 file(s) +2810/-3, docs 5 file(s) +123/-6, generated 0 file(s) +0/-0, other 5 file(s) +270/-0`; ratio ~1.0x against the expected five-client adapter scope.

## Measurements

- Host verification only (no GPU or model evaluation compute):
  - `uor-harness loop copyright` on the laptop against `~/Downloads/LoopKit-2026-10-12-v9 (1).zip`: `0` matching 8-word windows across all tracked repository `.md` and `.txt` files.
  - `uor-harness loop onboard --check-receipts`: `5` client receipts verified (`antigravity`, `claude`, `codex`, `deepseek`, `kimi`), all `5` live canary writes to `CHARTER.md` refused.
  - `cargo test --locked -p uor-r4-harness --test loop_guard`: `7 passed; 0 failed` in `1.03s`.

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-harness -- --check` — PASS (exit 0)
- `CARGO_TARGET_DIR=local/antigravity-harness-target ~/.cargo/bin/cargo clippy --locked -p uor-r4-harness --all-targets -- -D warnings` — PASS (exit 0)
- `CARGO_TARGET_DIR=local/antigravity-harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test loop_guard` — PASS (7 passed, 0 failed)
- `UOR_LAB=antigravity local/antigravity-harness-target/debug/uor-harness loop onboard --check-receipts` — PASS (5 verified and canary writes refused)
- `UOR_LAB=antigravity local/antigravity-harness-target/debug/uor-harness loop copyright` — PASS (0 matching 8-word windows)
- `UOR_LAB=antigravity local/antigravity-harness-target/debug/uor-harness loop gates --full` — PASS (3/3 gates green)
- `UOR_LAB=antigravity local/antigravity-harness-target/debug/uor-harness loop report --check` — PASS
- `UOR_LAB=antigravity local/antigravity-harness-target/debug/uor-harness loop pr-check --local` — PASS on `order`, `commit-order`, `report`, `owner-paths`, `rule-sources`, `read-path` (`scope` N/A/FAIL until item scopes land on `main`)

## Skills used

- `scope-guard`: kept all edits strictly inside the ORDER's declared files and verified no owner-only path or concurrent PR file (`looparrive.rs`, `gates.json`) was touched.
- `gates`: verified the planted fault (allowing a canary write to `CHARTER.md`) turns `each_clients_canary_write_is_refused` red before restoring the guard.
- `solution-integrity`: ensured `loop guard` checks repository membership before `labs.json` lookup so hooks firing outside `~/uor-r4` exit `0` instead of failing on a missing lab registry.
- `working-discipline`: ran the cancel probe first, recorded negatives first in this report, and verified against the real repository and private Loop Kit archive.

## Still open

- Codex's live P-1 and P-2 probe execution on `#2028` remains owed by `codex`; `loop guard`, `loop install` and `loop onboard` already support `codex` (`~/.codex/hooks.json`, `~/.codex/rules/uor-r4.rules`, `~/.codex/skills`) and keep subagents without a suborder read-only.
- Once PR 9 merges, the owner can run `uor-harness loop install --all --replace` from Terminal to normalize the live `$HOME` skill YAML descriptions and replace the hand-installed hooks with `uor-harness loop guard`.

## Plain summary

This change builds the client adapter commands for the five-lab loop (loop PR 9, issue #2914). Each of the five coding clients (`antigravity`, `claude`, `codex`, `deepseek`, `kimi`) has a different hook format and user-level configuration folder. `uor-harness loop guard` provides a single command that client hooks call before running a tool: if the user is working in a folder outside `~/uor-r4`, it exits immediately so other projects are untouched; inside `~/uor-r4`, it blocks edits to owner-only files (such as `CHARTER.md` and `AGENTS.md`), blocks edits to other labs' worktrees or `local/` folders, enforces narrower file scopes for subagents (using the subagent identifiers discovered in probe P-1), and blocks dangerous or owner-only shell commands. `uor-harness loop install` installs the six allowed Loop Kit skills and hook configs for each client while backing up any replaced files to `local/loop/install-backups/`. `uor-harness loop onboard` verifies a client's setup (including a live canary test that proves a write to `CHARTER.md` is refused) and checks the five committed onboarding receipts in `docs/harness/onboarding/`. Finally, `uor-harness loop copyright` scans repository text on the owner's laptop in 8-word windows against the private Loop Kit zip to ensure no private kit text is ever copied into the repository.

Closes #2914
References #2904
