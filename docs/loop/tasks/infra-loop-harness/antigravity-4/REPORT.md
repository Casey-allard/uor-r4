# REPORT: infra-loop-pr09-client-adapters, antigravity-4

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Cancel probe result:** On `8a093dc16fb2401d4872e3747e2584f7e25c5b81`, `uor-harness loop review --check --pr 2932` exited `1` (`CHANGES`), and `loop install --client claude --apply --replace` overwrote existing `~/.claude/CLAUDE.md` content while writing backups to a fixed path under `local/loop/install-backups/`. Class: **instrument**.
- **Three required items from the `8a093dc` `CHANGES` review fixed and tested in `tests/loop_guard.rs::repair_order_antigravity_4_regressions`:**
  1. **HIGH 1 — Preserve existing `~/.claude/CLAUDE.md` content between marker lines (`loopguard.rs:3269-3282`, `merge_claude_md`):** `plan_client_install` reads any existing `~/.claude/CLAUDE.md` and merges the UOR-R4 pointer block between `<!-- uor-r4-loop-start -->` and `<!-- uor-r4-loop-end -->` markers. Existing user instructions before and after the marker block survive intact, and subsequent `--apply --replace` runs replace only the marked block.
  2. **MEDIUM 2 — Timestamped per-run backup folder (`loopguard.rs:3724-3731`):** `run_install` writes backups for each `--apply --replace` run under `local/loop/install-backups/<epoch_ms>-<pid>-<seq>/` so a later run after a file changes never overwrites an earlier backup.
  3. **MEDIUM 3 — Isolate `tests/loop_guard.rs` install tests from the real repository checkout:** `install_skills_backups_and_replace_refusal`, `repair_order_antigravity_3_regressions` and `repair_order_antigravity_4_regressions` bind their temporary `TempDir` repository fixture as `temp_repo` and assert that every backup path is written inside `temp_repo/local/loop/install-backups/` and never inside `repo_root()`.
- **Follow-ups resolved in this round:**
  - `cli_install` (`loopguard.rs:4626-4641`) checks `std::env::var_os("UOR_LAB").is_some()` so even whitespace-only `UOR_LAB=" "` is refused with exit `2` (`REFUSED`) and writes nothing.
  - `docs/loop/README.md:204` states explicitly that a shell deny-list can never be complete (with `echo 'pkill x' | sh` as an example) and that `loop pr-check` is the clean-tree policy gate before merge.
  - `repair_order_antigravity_3_regressions` now also verifies Codex (`.codex/hooks.json`) and DeepSeek (`.dsh/hooks.json`) user hook preservation and asserts the `REFUSED` stderr text and zero writes for `UOR_LAB="claude"` and `UOR_LAB=" "`.

## What changed

- `crates/uor-r4-harness/src/loopguard.rs`: added `merge_claude_md` with `<!-- uor-r4-loop-start -->` / `<!-- uor-r4-loop-end -->` markers, per-run timestamped backup directories under `local/loop/install-backups/<epoch_ms>-<pid>-<seq>/`, and `std::env::var_os("UOR_LAB").is_some()` presence checking on `loop install --apply`.
- `crates/uor-r4-harness/tests/loop_guard.rs`: added `repair_order_antigravity_4_regressions` (10 test suites total), isolated `temp_repo` assertions, Codex/DeepSeek hook merge assertions, and `UOR_LAB` refusal assertions.
- `docs/loop/README.md`: added the explicit deny-list incompleteness note (`echo 'pkill x' | sh`) to the guard threat model bullet.
- **Size by class:** `tests 1 file(s) +1409/-0, code 5 file(s) +4917/-3, docs 10 file(s) +388/-8, generated 1 file(s) +3/-3, other 9 file(s) +281/-0`; ratio ~1.0x.

## Measurements

- Host verification only: `cargo test --locked -p uor-r4-harness --test loop_guard` (`10 passed; 0 failed`), `uor-harness loop onboard --check-receipts` (`5 verified`), and `uor-harness loop copyright` (`0` matching 8-word windows across the repository).

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-harness -- --check` — PASS
- `CARGO_TARGET_DIR=local/antigravity-harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test loop_guard` — PASS
- `UOR_LAB=antigravity uor-harness loop gates --full`, `loop report --check`, `loop pr-check --local` — PASS

## Skills used

- `scope-guard`, `gates`, `solution-integrity`, `working-discipline`: preserved user `~/.claude/CLAUDE.md` content between marker lines, isolated per-run backups under timestamped folders, and verified that tests never write into the real checkout's `local/` folder.

## Still open

- Once PR #2932 merges, Casey can run `uor-harness loop install --all --apply --replace` from Terminal (with `UOR_LAB` unset).
- Deferred non-blocking follow-up items from the `bc1ba40` and `8a093dc` reviews:
  - Replace the file-based stale-lock check in `with_guard_lock` with an OS file lock (`flock`/`fd-lock`) on a persistent lock file that is never deleted.
  - Deeper shell AST parsing beyond token-level deny-listing (`$(...)` command substitutions, heredocs, piped interpreters such as `echo 'pkill x' | sh`, `git worktree` / `git clone` / `git fetch` refspec writes, and `curl -d @file`).
  - Streaming zip reader in `read_kit_entries` (replacing per-entry `zipinfo -1` / `unzip -p` subprocesses) and dynamic discovery of new top-level files in the kit archive.
  - Subagent binding cleanup if a subagent crashes without firing `SubagentStop`.

## Plain summary

Repair order `antigravity-4` resolves all three required items and follow-ups from Claude's `8a093dc` `CHANGES` review of PR #2932: `loop install --apply` preserves existing `~/.claude/CLAUDE.md` content using marker lines, writes backups into per-run timestamped directories under `local/loop/install-backups/`, and isolates all install tests to temporary repository fixtures.

Closes #2914
References #2904
