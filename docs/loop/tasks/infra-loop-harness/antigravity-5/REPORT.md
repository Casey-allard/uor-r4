# REPORT: infra-loop-pr09-client-adapters, antigravity-5

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Cancel probe result:** On `91065253db03d7746e2ae4105cadaf64270b13a3`, `uor-harness loop review --check --pr 2932` exited `1` (`CHANGES`), and `merge_claude_md` (`loopguard.rs:3639-3658`) checked only `existing.find(CLAUDE_MD_START)` and `existing.find(CLAUDE_MD_END)` without counting markers, so a stray `<!-- uor-r4-loop-start -->` marker with no end marker caused run 1 to append a second block and run 2 to pair the stray start with the new end and delete the user's text between them. Class: **instrument**.
- **Required item and LOW follow-up from the `9106525` `CHANGES` review fixed and tested in `tests/loop_guard.rs::repair_order_antigravity_5_regressions`:**
  1. **Refuse malformed `CLAUDE.md` marker states (`merge_claude_md`, `loopguard.rs:3643-3686`):** `merge_claude_md` counts `<!-- uor-r4-loop-start -->` (`start_count`) and `<!-- uor-r4-loop-end -->` (`end_count`) and accepts only `(0, 0)` or `(1, 1)` with `start < end`. Any other state (a stray start or end marker, two or more blocks, or an end marker before a start marker) returns `Err("REFUSED: malformed UOR-R4 marker state in ...")` naming the file path and marker problem, so `loop install` exits `2` and writes nothing.
  2. **LOW follow-up (`loopguard.rs:3649-3657`):** In the `(0, 0)` case, if an unmarked `CLAUDE_MD_BODY` appears inside a larger `~/.claude/CLAUDE.md`, `merge_claude_md` wraps that block in place between `<!-- uor-r4-loop-start -->` and `<!-- uor-r4-loop-end -->` instead of requiring it to be the entire file.

## What changed

- `crates/uor-r4-harness/src/loopguard.rs`: updated `merge_claude_md` to return `Result<String, String>`, validate `(start_count, end_count)` as `(0, 0)` or `(1, 1)` with `start < end`, and migrate embedded unmarked pointers in place.
- `crates/uor-r4-harness/tests/loop_guard.rs`: added `repair_order_antigravity_5_regressions` (11 test suites total) covering all three malformed states (stray start marker, two blocks, and end before start) plus embedded unmarked pointer migration.
- **Size by class:** `tests 1 file(s) +1553/-0, code 5 file(s) +4950/-3, docs 12 file(s) +472/-8, generated 1 file(s) +3/-3, other 10 file(s) +286/-0`; ratio ~1.0x.

## Measurements

- Host verification only: `cargo test --locked -p uor-r4-harness --test loop_guard` (`11 passed; 0 failed`), `uor-harness loop onboard --check-receipts` (`5 verified`), and `uor-harness loop copyright` (`0` matching 8-word windows across the repository).

## Checks run at the head

- `~/.cargo/bin/cargo fmt -p uor-r4-harness -- --check` — PASS
- `CARGO_TARGET_DIR=local/antigravity-harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test loop_guard` — PASS
- `UOR_LAB=antigravity uor-harness loop gates --full`, `loop report --check`, `loop pr-check --local` — PASS

## Skills used

- `scope-guard`, `gates`, `solution-integrity`, `working-discipline`: validated `CLAUDE.md` marker counts so malformed marker states fail closed with exit `2` and zero writes, and added regression tests for all three malformed states.

## Still open

- Once PR #2932 merges, Casey can run `uor-harness loop install --all --apply --replace` from Terminal (with `UOR_LAB` unset).
- Deferred non-blocking follow-up items from the `bc1ba40` and `8a093dc` reviews:
  - Replace the file-based stale-lock check in `with_guard_lock` with an OS file lock (`flock`/`fd-lock`) on a persistent lock file that is never deleted.
  - Deeper shell AST parsing beyond token-level deny-listing (`$(...)` command substitutions, heredocs, piped interpreters such as `echo 'pkill x' | sh`, `git worktree` / `git clone` / `git fetch` refspec writes, and `curl -d @file`).
  - Streaming zip reader in `read_kit_entries` (replacing per-entry `zipinfo -1` / `unzip -p` subprocesses) and dynamic discovery of new top-level files in the kit archive.
  - Subagent binding cleanup if a subagent crashes without firing `SubagentStop`.

## Plain summary

Repair order `antigravity-5` resolves the single required item and LOW follow-up from Claude's `9106525` `CHANGES` review of PR #2932: `merge_claude_md` counts start and end markers in `~/.claude/CLAUDE.md`, refuses any state other than `(0, 0)` or `(1, 1)` with start before end (exiting `2` and writing nothing), and migrates embedded unmarked pointers in place.

Closes #2914
References #2904
