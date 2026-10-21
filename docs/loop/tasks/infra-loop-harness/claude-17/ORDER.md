# ORDER: infra-loop-pr07b-session, claude-17 (loop PR 7b: loop next, arrive, report and close)

Lab: claude. Item: [infra-loop-pr07b-session](../../../../roadmap/items/infra-loop-pr07b-session.md) (#2911). Setup row: 7b. Base: `8c2c76e42`.

## Goal

The item's done-when:
- `loop next` picks work in the loop page's order and stops on its listed stops.
- `loop arrive` fetches, then checks the binary, hooks, notes and read path.
- `loop report --check` checks the REPORT's sections.
- `loop close --session` does step 9, and `loop close --verify` does step 13.
- Templates are in our own words.
- The planted fault: **a REPORT missing a section is refused.**

## Cancel probe (run first)

Do `next`, `arrive`, `report` or `close` exist under `loop`? Result at `8c2c76e42`: no. The loop actions are gates, render, claim, suborder, ask and pr-check. The task stands. Class: instrument.

## Architecture calls (Claude, D52)

1. **Missing sources are named, never guessed.** `next` reads the sources that exist: the kill switch (`local/loop/STOP`, or the label `loop:pause` on #2028), the lab's own PRs needing repair, PRs awaiting its review, and ready, unclaimed, scoped items. It prints the sources not built yet and the PR that builds them: owner directions and probes (PR 8, PLAN), GPU collections (PR 10), lanes (no `lane:` key exists).
2. **Exit codes.** `next`: 0 GO, 1 STOP, 2 cannot run. The other commands: 0 ok (warnings allowed), 1 a check failed, 2 cannot run.
3. **One REPORT checker.** `report --check` uses `loopprcheck::report_problems`. `report --new` writes the template, and `report --receipts` copies this head's gate results from `local/<lab>/gates.jsonl` into the task folder.
4. **GitHub goes through a trait with a fake.** Only `close` writes to GitHub: the PR body, and marking the PR ready.
5. **`close --verify` deletes only what is provably the lab's.** All four must hold: GitHub reports MERGED, the squash commit `(#N)` is on `origin/main`, the PR's files there equal its head, and the worktree is clean and under `.worktrees/<lab>-*`. The remote delete is leased to the PR head. Hygiene follows the loop page.
6. **Templates.** ORDER, REPORT and PICKUP templates go in `docs/loop/templates/`, written in our own words and checked by the local copyright check.

## Files in scope

- `crates/uor-r4-harness/src/**` and `crates/uor-r4-harness/tests/**`
- `docs/loop/templates/**` (new)
- `docs/harness/README.md` and `docs/loop/README.md`: usage, and what is built.
- `docs/loop/tasks/infra-loop-harness/claude-17/**`

## Done when

- Each command's refusals and writes are pinned by tests, with GitHub faked. That includes the planted fault: a REPORT missing a section is refused.
- `close --verify` never touches a branch, worktree or PR it cannot prove is this lab's.
- The full harness suite passes, as do `pr-check --local`, claim-wording, links and the local copyright check.

## Stop and ask if

- A command needs a GitHub write other than the PR body or ready state.
- Hygiene `--apply` would touch anything outside its documented guards.
- Any file outside scope needs to change.
