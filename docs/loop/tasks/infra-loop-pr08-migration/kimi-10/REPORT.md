# REPORT: infra-loop-pr08-migration, kimi-10 (repair)

Lab: kimi. Order: [ORDER.md](ORDER.md). Parent: [kimi-9/REPORT.md](../kimi-9/REPORT.md) (the original account, unchanged). References #2904. Closes #2913.

## What did not work, first

- **The M1 headline reconciliation was the owner's call, not mine to make mechanically.** kimi-9's milestones.json edit (the M1 headline → the §4.8 closing bar) broke three repo-content tests that pin the registry against `docs/labs/session-goal.md` (`briefingrules::the_real_repo_agrees_with_its_own_prompts`, `prgates::registry_gate_tests::the_headline_gate_fires_only_on_a_self_contradicting_body`, `prgates::check_pr_tests::every_ported_gate_and_the_dispatcher_agree_with_python` — each verified failing with the edit and passing on the revert at this branch). Which metric heads M1 is a bar decision; carded to the owner on #2028. The edit is reverted; the registry keeps the reply headline.
- **Two gate-rule defects surfaced and were handled in the open.** The owner-paths rule needs a `## Owner approval` section with each changed owner path named, a double-quoted answer and an ISO-UTC time — kimi-9's ORDER carried the approval as a header paragraph with a `~`-prefixed time and failed twice before passing. The rendered-lock rule allowed the views' first commit on no branch at all (its N/A read "absent at base AND head"); fixed in this PR (loopprcheck.rs, a shared file): with no view at the base, the first commit passes on any branch; the steady-state kimi/render rule is unchanged and the existing 28 loop_prcheck tests pass unmodified.
- **The §4.7 stub pattern defeats the links gate's rename detection** (its -M100% cached diff pairs renames only when the old path is deleted; my stubs keep it modified). The 19 outgoing links inside the byte-identical archive copies count as NEW broken and fail the gate — carded to the loop architect on #2028 with the analysis. My own 6 link slips are fixed separately (the two stubs' depths and L-018's pre-existing broken link, repaired in place per the store's correct-in-place rule).
- **The flaky loop-drive gate** (`a_leased_session_is_never_closed_and_notes_exactly_once_per_window`) went red twice in three runs on this branch (nothing here touches loopdrive): a real-progress note races a lease window in the gate's scratch worktree. Carded on #2028 with the log; green on the run whose receipts are bound.

## What changed (this repair)

- `docs/harness/milestones.json` reverted to the base content for M1 (the reply headline stands, consistent with session-goal.md and the pinned tests).
- kimi-9's done-when 7 is amended by kimi-10's ORDER to "verified against §4.8 and any mismatch carded to the owner"; the card is #2028 comment 6351835226.
- `crates/uor-r4-harness/src/loopprcheck.rs` (shared): the rendered-lock rule passes the views' first commit when none exists at the base.
- The ORDER's `## Owner approval` section (both owner paths named, the verbatim quote, the ISO time).

Second round (draft-phase feedback on #2982, 21:47–21:52Z, after merging 105 commits of main):

- `docs/start/FILEMAP.md`: rows for `src-tauri` and its three subdirectories — the desktop client landed on main while this PR was open, and the filemap verifier enumerates every tracked directory two levels deep.
- `crates/uor-r4-harness/tests/loop_migrate.rs`: the unused `Path` import is gone (the one real CI red in the harness job).
- The views re-rendered from the merged head (`loop render --check` byte-exact).
- Verified with a PR-code binary and the CI-equivalent commands, against the diagnosis on the PR: rendered-lock PASSes (the first-commit pass applies — no view exists at the base), owner-paths PASSes (this ORDER's section), and `xtask links --baseline --renames-from` at the merged head reports **0 new broken** with 19 COPIED (both archive copies, byte-identical, under #3007). The diagnosed HANDOFF/stub/L-018 link fixes were already in the tree at 84b10704e; the archive copies must NOT be relinked (byte-identity is the migration's premise).

## Measurements

- The three repo-content tests pass at this head (each named above, run individually).
- `cargo test -p uor-r4-harness --test loop_prcheck`: 28 passed, 0 failed, with the rendered-lock extension in place.
- `uor-harness loop gates --full`: 11/11 GREEN at the head; `--break all`: 11/11 CAUGHT, each red with its declared text (the loop-migrate fault reds with "broken link").
- `uor-harness loop migrate --check`: PASS at the head (the renderer's re-rooting fix landed; HANDOFF.md's link resolves).
- `xtask links --baseline … --renames-from …`: the only NEW-broken links at the head are the 19 inside the byte-identical archive copies, carded as the tool gap.
- The full `cargo test -p uor-r4-harness` suite result at the final head is recorded with the push.

## Checks run at the head

- `loop gates --full`, `--break all`, `loop report --check`, `loop pr-check --local`, `loop migrate --check`, `xtask roadmap check` / `render`, `xtask links` (baseline mode), and the full harness suite — results above and in the receipts.

## Skills used

- `scope-guard`, `solution-integrity`, `working-discipline` (the loop's private skills; no kit text in this repo).

## Still open

- The owner answers the M1-headline card (a registry + session-goal + pinned-tests follow-up rides his answer).
- The links gate's archive-source classification (the carded tool gap) — the loop architect's call.
- The flaky loop-drive gate test (carded with evidence).
- The D37/D30 classification of the legal-construction solve is ruled (PM, #2828); the case file itself is still with the owner.

## Plain summary

This repair keeps the migration honest where its first pass overreached: the M1 headline stays as it is because picking the metric is the owner's bar call, not a mechanical edit, and the three tests that caught it are green again. The gate machinery itself needed two small fixes found by running it for real (the approval-section format and the rendered-lock rule's missing first-commit case), and the archive-move pointers exposed a links-gate gap that is now the loop architect's carded question. Everything else from kimi-9 stands as written there.
