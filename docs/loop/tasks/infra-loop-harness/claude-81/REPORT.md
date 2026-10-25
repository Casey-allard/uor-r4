# REPORT: infra-loop-harness, claude-81

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904.

## What did not work, first

- **The first plan was to replace the five required "acknowledgement" checks with one real check. That was wrong, and I dropped it.** The acknowledgements are not empty: `required-transport` waits for the CI `policy` job (`loop pr-check --ci`) and writes its verdict into each summary. `loop pr-check --pr` and `loop queue` accept a PR only when that says `policy: pass`. That wait is the real merge gate, so it stays. What costs time is the rules inside it, which this unit addresses.
- **I didn't plan to touch `release.yml` or `deploy.yml`, and didn't.** #2870's release work will repoint `release.yml`, so deleting it now would cost that issue a rebuild. Only `relative-query-check.yml` is deleted; it is tied to a September branch.
- **The fresh review found a gap in a hard rule that is older than this PR.** `banned-presence` skipped added files (`status == 'A'`), so a new `.rs` file could bring banned code in unseen. This unit fixes it: an added file is counted against an empty base.
- **One harness test fails in a worktree, unrelated to this change.** `loop_guard::each_clients_canary_write_is_refused` gets the refusal reason "cross-worktree write refused: .worktrees/claude-gates-reset belongs to another lab" instead of the owner-only reason it expects. The guard infers the owning lab from the worktree path. It is environmental and comes from main.

## What changed

- **`crates/uor-r4-harness/src/loopprcheck.rs`:** a new `pub const ADVISORY` lists `scope`, `commit-order`, `rule-sources`, `rendered-lock`, `read-path` and `filemap`. `evaluate()`, shared by `loop pr-check` and `loop queue`, turns a FAIL from any of them into WARN and prefixes the summary with `advisory (D76):`. The details still print. `order`, `banned-presence`, `report`, `owner-paths`, `review-record` and `required-checks` stay hard.
- **`crates/uor-r4-harness/tests/loop_prcheck.rs`:**
  - The 26 assertions of `FAIL <rule>` for the six rules now expect `WARN <rule>`.
  - Where a demoted rule was the fixture's only failure, the expected exit code goes from 1 to 0. That covers the three commit-order tests, the read-path and filemap tests, and the filemap test's first check. Every other exit code stays 1, because its fixture still fails a hard rule (`order` or `report`).
  - The new `an_out_of_scope_change_warns_but_passes_d76` checks a green PR with one out-of-scope file: it gets `WARN scope`, `advisory (D76)` and `verdict: PASS`.
  - The new `a_new_file_with_banned_code_fails_the_presence_rule`: an added file containing banned code fails `banned-presence`.
  - The new `the_project_owner_paths_still_cover_agents_charter_decisions_and_ci_d76` reads the real `docs/harness/owner-paths.json`. It confirms that `AGENTS.md`, `CHARTER.md`, `DECISIONS.md` and `.github/**` stay under the hard `owner-paths` rule now that `rendered-lock` and `read-path` are warnings.
  - Six test names that said "fails" or "refuses" for a demoted rule now say "warns".
- **`banned-presence`:** an added file is now counted against an empty base instead of being skipped.
- **`.github/workflows/docs.yml`:** the Links, Claim-wording and Generated catalog/figures steps get `continue-on-error: true`. Archive verify stays hard. **Moved to a follow-up PR**: under main's pre-D76 rules, CI's policy job would fail these two files on scope. Once D76 is on main, scope only warns.
- **`.github/workflows/relative-query-check.yml`:** deleted.
- **`docs/integration/DECISIONS.md`:** D76, with the owner's words. **`docs/loop/README.md`:** the policy line names the split.
- **Ruleset `19597522`:** after merge, set `strict_required_status_checks_policy: false` and the merge queue's minimum wait to 0. That is recorded on #2904.

## Measurements

- `cargo test --locked -p uor-r4-harness --test loop_prcheck`: 50 passed, 0 failed.
- **Negative gates.** Each was set aside from the branch, run, then restored:
  - With the advisory change set aside from `origin/main`, `an_out_of_scope_change_warns_but_passes_d76` FAILS. Restored, it passes.
  - With the added-file change set aside, `a_new_file_with_banned_code_fails_the_presence_rule` FAILS. Restored, it passes. The existing `owner_paths_need_the_entry_a_quoted_answer_and_a_time` still passes, so owner paths still fail a PR.
- **The full harness suite, run with `--no-fail-fast`.** Plain `cargo test` stops at the first failing target, so CI's first run on this PR found two more test files asserting `FAIL commit-order` and `FAIL scope`: `loop_repair_orders.rs` and `loop_shared_files.rs`. Both now expect WARN, and in the one test whose only failure was commit-order the expected exit code goes to 0.
- **What is left failing.** The environmental `loop_guard` test fails, and `read_path_warns_on_every_over_budget_item_and_the_warning_line` failed once under the parallel full run. That test passes alone and in its own target (50/50), so it is load-sensitive, not a regression.

## Checks run at the head

- `cargo fmt -p uor-r4-harness -- --check`: PASS (after one `cargo fmt`). `cargo clippy -p uor-r4-harness --all-targets -D warnings`: PASS.
- `uor-harness loop report --check`, `loop pr-check --local` and `loop gates --quick`: PASS, run with this head's binary (`target/claude-gates/debug/uor-harness`).

## Skills used

- None.

## Still open

- The ruleset change, made after merge and recorded on #2904.
- The plan reset (one `PLAN.md`, the held items released) and ranking `loop next` by unblock count are their own units. Both are announced on #2028.

## Plain summary

Bookkeeping rules no longer stop a merge; they print a warning. The rules that protect the project still stop one: banned operators, owner-only files, the ORDER and REPORT, and the exact-head review. Broken links and stale catalogs are reported but don't block a PR.
