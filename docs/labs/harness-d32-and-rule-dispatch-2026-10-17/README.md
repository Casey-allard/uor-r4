# `uor-r4-harness` Un-Dispatched Rule Implementation, Adapter & Dispatch Reconciliation (`harness.*`, `prlint.gates_declared`, `receipts.skill_usage_recorded`) — 17 October 2026

**Scope:** Source-only harness rule implementation, adapter, and dispatch reconciliation (`antigravity` / `ag1`, milestone `#2029`). Zero model compute (`model_compute_executed: false`, `apply_calls: 0`, `$0` GPU spend). Training, M2, and Stage B remain **HELD**; Q4 remains **NOT_EXECUTABLE**; H1 magnitude remains **NOT_APPROVED**. No `#[cfg(feature = "cuda")]` or `required-features = ["cuda"]` target was modified, preserving all pinned `crates/uor-r4-discrete` and instrument SHA-256 digests.

---

## What Was Found

A cross-module audit of `crates/uor-r4-harness` comparing the **41 repo-scope rules** registered in `checkdispatch::RULES` (`crates/uor-r4-harness/src/checkdispatch.rs:22-67`) and `checkdispatch::MODULE_RULES` (`crates/uor-r4-harness/src/checkdispatch.rs:69-162`) — alongside the **53 total entries** across 10 modules in `ruleinventory::INVENTORY` (`crates/uor-r4-harness/src/ruleinventory.rs:26-170`, which includes the 12 individual `prlint.*` PR-body gates in addition to the 41 repo-scope rules) — against the **nine** per-module `all(...)` functions and **four** direct `mistakes::*` calls in `checkdispatch::run_scoped`'s `per_module` table (`crates/uor-r4-harness/src/checkdispatch.rs:230-247`) identified **five registered rules in `checkdispatch::RULES` that were never invoked by their module dispatcher**:

1. **`harness.owner_rules_applied` (`crates/uor-r4-harness/src/checkdispatch.rs:37,103`, `crates/uor-r4-harness/src/ruleinventory.rs:160`):**
   - Registered in `RULES`, `MODULE_RULES["harness"]`, and `INVENTORY["harness"]`, and implemented in Python (`scripts/harness/checkrules.py:238-303` `_check_owner_rules_applied`), but **had no Rust implementation in `crates/uor-r4-harness/src/harnessrules.rs`**. Because `checkdispatch::run_scoped` (`crates/uor-r4-harness/src/checkdispatch.rs:266-278`) synthesizes `message: "ok"` whenever a registered rule ID produces no `Finding`, `uor-harness check` unconditionally reported `harness.owner_rules_applied: ok` without ever comparing `docs/harness/milestones.json`'s `all`-scoped `owner_rules` against `crate::prep::APPLICABLE_RULES`.
2. **`harness.entry_point` (`crates/uor-r4-harness/src/checkdispatch.rs:34,100`, `crates/uor-r4-harness/src/ruleinventory.rs:167`):**
   - Implemented in `crates/uor-r4-harness/src/doccheck.rs:182-210` (`doccheck::entry_point`), but never called by `harnessrules::all` (at base `fa62a670a` lines `103-113`; post-change lines `98-111`).
3. **`harness.documented_commands` (`crates/uor-r4-harness/src/checkdispatch.rs:33,99`, `crates/uor-r4-harness/src/ruleinventory.rs:156`):**
   - Implemented in `crates/uor-r4-harness/src/doccheck.rs:902-930` (`doccheck::documented_commands_rule`; at base `fa62a670a` lines `795-823`), but never called by `harnessrules::all` (at base `fa62a670a` lines `103-113`; post-change lines `98-111`). In addition, within `crates/uor-r4-harness/src/doccheck.rs`, the static `GROUPS` and `FLAGS` tables were marked `#[allow(dead_code)]` while `commands_in_text` discarded `flags` (`let _ = &flags;`) and `resolve` checked only `known.contains(&head.as_str())`, allowing invalid subcommands (`uor-harness team list`) or misspelled flags (`uor-harness status --refreshh`) to pass.
4. **`prlint.gates_declared` (`crates/uor-r4-harness/src/checkdispatch.rs:55,141`, `crates/uor-r4-harness/src/ruleinventory.rs:103`):**
   - Registered in `RULES`, `MODULE_RULES["prlint"]`, and `INVENTORY["prlint"]`, with `DECLARED_GATES` defined in `crates/uor-r4-harness/src/prgates.rs:1896-2044`, while `prlintrules::all` (at base `fa62a670a` lines `265-268`; post-change lines `347-351`) only invoked `interlock_holds(paths, claims)`.
5. **`receipts.skill_usage_recorded` (`crates/uor-r4-harness/src/checkdispatch.rs:62,150`, `crates/uor-r4-harness/src/ruleinventory.rs:67`):**
   - Implemented as an intentional no-op returning `Vec::new()` in `crates/uor-r4-harness/src/receiptsrules.rs:329-361` (matching Python's informational placeholder in `scripts/harness/checkrules.py`), but not called by `receiptsrules::all` (at base `fa62a670a` lines `186-190`; post-change lines `183-188`).

---

## Changes Made (Implementation + Adapter + Dispatch Wiring)

1. **`crates/uor-r4-harness/src/doccheck.rs` (`:1-20, 480-715, 850-980`):**
   - Activated `GROUPS` (`11` subcommand groups, adding `("comment", &["edit"])`) and `FLAGS` (`76` flags across `cli.py` and `main.rs`'s `SUBCOMMAND_FLAGS` / `SUBCOMMAND_VALUE_FLAGS`), removing `#[allow(dead_code)]` and `let _ = &flags;`.
   - Added `pub fn resolve_with_flags(words: &[String], flags: &[String], known: &[&str]) -> (bool, String)` (`:669-712`) and wired `commands_in_text` (`:850-869`) to validate top-level commands against `ACCEPTED`, subcommands against `GROUPS`, and flags against `FLAGS` (exempting bare prose `"-"` after `RUST_ONLY_COMMANDS` so `the_table_agrees_with_pythons_live_parser` agrees 100% with `scripts/harness/doccheck.py` across all 143 documented invocations in the repository).
   - **Disclosed architectural distinction from Python:** Python's `scripts/harness/doccheck.py` introspects `cli.py`'s live `argparse` tree per subcommand (and short-circuits `RUST_ONLY_COMMANDS` without checking their subcommands or flags), whereas Rust's `doccheck` resolves against the static `ACCEPTED`, `GROUPS`, and global `FLAGS` union tables and checks flags on both shared and Rust-only commands.
   - Added unit test `subcommands_and_flags_are_validated_against_groups_and_flags` (`:940-977`).

2. **`crates/uor-r4-harness/src/harnessrules.rs` (`:98-168, 428-516`):**
   - **New rule implementation:** `pub fn owner_rules_applied(regs: &Registries) -> Vec<Finding>` (`:115-142`), checking that every `all`-scoped `owner_rules` entry in `regs.owner_rules` (excluding `OWNER-*` sub-clauses, matching `scripts/harness/checkrules.py:260-268`) appears in `crate::prep::APPLICABLE_RULES`.
   - **Dispatcher adapters:** `pub fn entry_point(paths: &StatePaths) -> Vec<Finding>` (`:150-155`) and `pub fn documented_commands(paths: &StatePaths) -> Vec<Finding>` (`:163-168`), adapting `crate::doccheck::entry_point` and `crate::doccheck::documented_commands_rule` to `checkdispatch::run_scoped`'s convention (forwarding `error` / non-empty-`fix` findings and returning `Vec::new()` when clean).
   - **Dispatch wiring & tests:** Wired `owner_rules_applied(regs)`, `entry_point(paths)`, and `documented_commands(paths)` into `harnessrules::all(paths, regs)` (`:98-111`), and added unit and dispatch-level tests (`an_unapplied_all_scoped_owner_rule_is_reported`, `entry_point_reports_stale_cli_and_stays_silent_when_clean`, `documented_commands_reports_unknown_command_and_stays_silent_when_clean`, and `all_and_checkdispatch_surface_owner_rules_entry_point_and_documented_commands`).

3. **`crates/uor-r4-harness/src/prlintrules.rs` (`:258-351, 526-653`):**
   - **New rule implementation:** `pub fn gates_declared_for(declared: &[&str], inventory_prlint: &[&str]) -> Vec<Finding>` (`:276-325`) and `pub fn gates_declared() -> Vec<Finding>` (`:329-344`), reconciling `crate::prgates::DECLARED_GATES` against `crate::ruleinventory::INVENTORY["prlint"]` and wired into `prlintrules::all(paths, claims)` (`:347-351`).
   - **Disclosed semantic distinction from Python:** Python's `scripts/harness/prlint.py:874-899` (`_gate_ids`) and `scripts/harness/prlint.py:1026-1048` (`check_gates` in `register_rules`) AST-walk `check_pr()` and `_check_softmax()` in `scripts/harness/prlint.py` to extract every gate ID emitted in code and reconcile that live set against `LOGICAL_RULES` / `GATE_RULES`. In Rust, `prlintrules::gates_declared` reconciles the two static tables `prgates::DECLARED_GATES` and `ruleinventory::INVENTORY["prlint"]` at `uor-harness check` time, while the "declared in `DECLARED_GATES` and actually reachable/emitted by `prgates::check_pr()`" direction is enforced at `cargo test` time by `prgates::gates_declared_tests::the_bodies_exercise_every_declared_gate_that_a_body_can_reach` (`crates/uor-r4-harness/src/prgates.rs:1933-2043`).
   - Added unit and dispatch-level tests `gates_declared_reconciles_declared_gates_with_inventory` and `all_and_checkdispatch_invoke_gates_declared` (asserting both clean invocation and injected-mismatch `Finding` emission through `prlintrules::all` and `checkdispatch::run_scoped`).

4. **`crates/uor-r4-harness/src/receiptsrules.rs` (`:6-17, 183-188, 329-401`):**
   - Updated the module header table and commentary (`:6-17`) to reflect that `receipts.skill_usage_recorded` is ported and wired into `receiptsrules::all(paths)` (`:183-188`) as an explicit no-op matching Python, and added dispatch test `all_and_checkdispatch_invoke_skill_usage_recorded`.

5. **Synthesized-`ok` severity note (`checkdispatch::run_scoped`, `crates/uor-r4-harness/src/checkdispatch.rs:266-278`) & `docs/harness/claims.json` `head` field (`:9-15`):**
   - When a rule returns no `Finding`, `checkdispatch::run_scoped` synthesizes an `"ok"` record using the rule's declared severity in `checkdispatch::RULES` (`"error"` for `harness.entry_point` and `prlint.gates_declared`, `"warn"` for `harness.owner_rules_applied` and `harness.documented_commands`, `"info"` for `receipts.skill_usage_recorded`), whereas Python's clean `RuleResult` uses `warn` for `harness.entry_point` and `info` for `prlint.gates_declared`. `checkdispatch::summarize` only counts findings with `message != "ok"`, so failure/warning totals are unaffected, while `uor-harness check --json` consumers observe the `checkdispatch::RULES` severity on synthesized `"ok"` records.
   - In `docs/harness/claims.json:9-15`, `head` is set to `"fa62a670a"` (the `origin/main` base commit at claim/merge-forward time; a tracked file inside a commit cannot embed that commit's own SHA-1).

---

## Verification

- `cargo test -p uor-r4-harness --lib doccheck -- --include-ignored`
- `cargo test -p uor-r4-harness`
- `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`
- `cargo fmt -p uor-r4-harness -- --check`
- `python3 scripts/check_claim_wording.py`
