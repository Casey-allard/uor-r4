# REPORT: infra-loop-harness, claude-137 (widen the item's scope to take codex out of the registry)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2904.

## What did not work, first

- **The registry change could not be made in one PR.** The plan was a single PR that took codex out of `docs/harness/labs.json` and moved its tests to four labs. The guard refused the first test edit, for two reasons:
  - it takes the unit's scope from the item on `origin/main`;
  - no item's `scope:` holds `crates/uor-r4-harness/src/labs.rs`.

  The refusal: "REFUSED: crates/uor-r4-harness/src/labs.rs is outside active ORDER … scope [PLAN.md, …]". So this unit only widens the scope, and the registry change follows as claude-138.
- **A lab cannot write `docs/harness/labs.json` at all.** The guard refuses every lab write to an owner-only path, approval or not: "REFUSED: docs/harness/labs.json is an owner-only path". `loop pr-check` accepts an owner-path change only when the ORDER quotes the owner's approval. So Casey makes that one-line edit on claude-138's branch.
- **The first scope was too narrow.** The first fresh review returned CHANGES with three Required findings. I had listed only the files that name the labs. It found more that read the real registry:
  - `check_all_receipts` in `loopguard.rs` walks a hard-coded `CLIENTS` list of five, so it would fail once codex left the registry;
  - `tests/loop_guard.rs` copies the real registry into its fixture and runs codex as a lab;
  - an `onboard.rs` test expects "codex" in a refusal;
  - `docs/harness/README.md` says "all five committed receipts".

  The scope now holds all four, and the ORDER records how the receipt gate follows the bar.
- **The item still reads "five-lab" in two places.** The title is "Build the five-lab loop harness", and "Why it matters" says "Five labs from five vendors". No gate reads them, and the ORDER kept this unit to the scope line and the bar.

## What changed

Size: docs 1 file (+2/−2), plus this task folder.

- **`docs/roadmap/items/infra-loop-harness.md`, the `scope:` line.** It gains ten globs, and nothing else: the files that read the real registry and name or count its labs, the loop page that lists the labs, and the archive. The ORDER gives the reason for each.
  - `docs/harness/labs.json`
  - `crates/uor-r4-harness/src/labs.rs`
  - `crates/uor-r4-harness/src/receipt.rs`
  - `crates/uor-r4-harness/tests/lab_registry_cli.rs`
  - `crates/uor-r4-harness/src/onboard.rs`
  - `crates/uor-r4-harness/src/loopguard.rs`
  - `crates/uor-r4-harness/tests/loop_guard.rs`
  - `docs/harness/README.md`
  - `docs/loop/README.md`
  - `archive/history/branch-archive/codex-d63-finite-probe-scope-20261026.*`
- **The receipt-gate decision (D68), built in claude-138.**
  - `CLIENTS` keeps codex's adapter.
  - `check_all_receipts` walks the registry's labs, so it verifies four.
  - Codex's committed receipt stays as unchecked history.
  - The guard tests for the codex adapter register codex in their own fixture.
- **`docs/roadmap/items/infra-loop-harness.md`, the done-when.** "The five labs (Kimi, Claude, Codex, DeepSeek, Antigravity) are onboarded" becomes "The four labs (Kimi, Claude, DeepSeek, Antigravity)". Codex's onboarding receipt and its past units stay in the repository as history.
- **`docs/roadmap/README.md`:** unchanged. A fresh `xtask roadmap render` wrote it identically.
- **Codex's #3169** was closed with a note at 2026-10-26T21:36:57Z, at the same instruction. Its three commits are saved byte-identically outside the repository. claude-138 adds them to the archive.

## Measurements

None. The decision's evidence:
- the two guard refusals;
- the guard's scope rule (`loopguard.rs`, `check_repo_rel_write`: "The item as it is on `origin/main`, so a branch cannot widen its own scope");
- the loop page's "Widening scope takes a separate item PR";
- Casey's two quotes in the ORDER.

## Checks run at the head

See the exact-head review record on the PR, which covers `xtask roadmap check` (95 items OK), `loop report --check` and `loop pr-check --local`.

## Skills used

- **holding-the-goal.** The refusal was read as the lock working. The fix widens this item's scope through its own reviewed PR, rather than working around the guard.

## Still open

- **claude-138, the registry change:**
  - the tests and hints move to four labs;
  - #3169's patch and its record go into the archive;
  - Casey edits `docs/harness/labs.json` on that branch, including its note's "The registry of the five labs";
  - confirm that `loop drive --client codex` is refused. Its receipt check only tests that `docs/harness/onboarding/codex.json` exists (`loopdrive.rs:1504`), and that file stays;
  - the loop page's title and its "## The five labs" table.
- **For the owner:**
  - `AGENTS.md`'s `UOR_LAB=<antigravity|claude|codex|deepseek|kimi>`, an owner path his instruction did not name;
  - the item's title and its "Why it matters" prose.

## Plain summary

Casey asked for codex to be taken out of the list of labs, because it now works on a different project. The project's lock only lets a task touch the files its roadmap item lists, as that list stands on the main branch, and no item listed the files this needs. This change adds exactly those files to the item's list, and drops codex from the item's finish line, so the next task can make the change itself.
