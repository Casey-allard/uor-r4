# ORDER: infra-loop-pr07a-claim, claude-15 (loop PR 7a: loop claim, suborder and ask)

Lab: claude. Item: [infra-loop-pr07a-claim](../../../../roadmap/items/infra-loop-pr07a-claim.md) (#2910). Setup row: 7a. Base: `32092fca6`.

## Goal

This is loop PR 7a from the build plan. Under D52 as amended in #2919, Claude builds every setup PR while it runs alone. The item's done-when reads:

> `loop claim` is refused unless the item is ready and unclaimed, its scope overlaps no open order (notebook pages excepted) and the lab is under its open-PR limit; it creates the worktree, commits the ORDER, opens a draft PR and lists again. `loop suborder` refuses a scope outside the parent's. `loop ask --scope` and `--blocked` write the two stop-and-ask cards. The planted fault turns the check red: **an overlapping claim is refused.**

## Cancel probe (run first)

Does any `loop claim`, `suborder` or `ask` already exist?

Result at `32092fca6`: `uor-harness` has only `loop gates`. Claims today use `uor-harness team claim` and `docs/harness/claims.json`, which check neither item readiness nor scope overlap. The task stands. Class: instrument.

## Architecture calls

These are Claude's calls as architecture lead (D52). They are reversible and recorded in the REPORT.

1. **What makes an item claimed.** An item is claimed when an open order names it. Open orders are found in two places:
   - the lab's own worktrees under `.worktrees/`, each holding an ORDER whose PR is not merged;
   - the open draft PRs, listed through `gh`.

   All GitHub access goes through one small trait. Tests use a fake, and never touch the network.
2. **Scope comes from the ORDER.** The order's scope is its item's `scope:` globs, narrowed if the ORDER says so. When an item has no `scope:` yet (PR 2 made the key optional), `loop claim` refuses with "item has no scope: line". It does not guess.
3. **Overlap is decided by literal prefix.** Two globs overlap when the literal part of one, the text before its first wildcard, is a path prefix of the other's literal part. This is conservative: it may refuse a claim that would not really collide, but it never misses a real overlap. Notebook pages under `docs/research/notebook/` are excepted.
4. **The open-PR limit comes from `docs/harness/labs.json`** (`open_pr_limit`).
5. **Claim writes happen only after every refusal check passes.** They come in the step 4 order from the loop page: worktree `.worktrees/<lab>-<topic>` from fresh `origin/main`, then the ORDER commit, then the draft PR, then a re-list. If the re-list shows a lower-numbered draft PR for the same item, the newer claim is withdrawn: its draft PR is closed and its worktree removed, and only what this claim created is touched.
6. **`loop suborder --parent <order> --scope <globs>`** refuses any glob not inside the parent's scope. It writes `local/<lab>/<topic>/orders/<n>.md`.
7. **`loop ask --scope` and `loop ask --blocked`** write their cards to `local/<lab>/<topic>/asks/<n>.md`, with the fields the loop page names. They post nothing.

## Files in scope

- `crates/uor-r4-harness/src/**`: new modules for claim, suborder and ask, and their wiring under `loop` in `main.rs`, plus the `doccheck.rs` tables for the new actions and flags.
- `crates/uor-r4-harness/tests/**`
- `docs/harness/README.md`: usage.
- `docs/loop/README.md`: only to say what is built.
- `docs/loop/tasks/infra-loop-harness/claude-15/**`

## Done when

- `loop claim <item>` refuses with exit 2 and a reason when:
  - the item is not ready;
  - the item is already claimed;
  - the item has no scope;
  - its scope overlaps an open order (notebook pages excepted);
  - the lab is at its open-PR limit.
- Otherwise `loop claim <item>` creates the worktree, commits the ORDER, opens the draft PR, re-lists and settles a race. In tests, GitHub is a fake.
- The planted fault holds, pinned by a test: an overlapping claim is refused.
- `loop suborder` refuses a scope outside the parent's and writes the sub-order.
- `loop ask --scope` and `--blocked` write their cards.
- Every refusal and every write is pinned by a test.
- The full harness suite passes, as do claim-wording, links and the local copyright check.

## Stop and ask if

- A real GitHub write would be needed in a test.
- Any file outside scope needs to change.
