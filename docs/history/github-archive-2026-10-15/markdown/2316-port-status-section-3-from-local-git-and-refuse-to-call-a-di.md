# #2316 — Port status section 3 from local git, and refuse to call a dirty branch deletable (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T07:16:23Z
- **Updated:** 2026-10-13T07:26:17Z
- **Closed:** 2026-10-13T07:18:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2316
- **Merged:** 2026-10-13T07:18:02Z
- **Merge commit:** 62f0b69011eb52c9898ee9bb04d0942190c109bc
- **Base / Head:** main / UOR-Foundation:harness/rust-status-delivery
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only.

## Result

Decision: **KEEP** (tooling).

**DELIVERY now renders in Rust, resolved from local git alone**: the checked-out branch, `origin/main`'s
commit and subject, the last-fetched warning, the branches holding **only finished work** with the exact
delete command, the branches with **unmerged work**, and — stated rather than omitted — that it **cannot
name the merging PR**, because it proves a branch finished by content (`git cherry`) rather than by asking
GitHub.

That is a deliberate trade, not a shortcut: a lab offline, on a pod, or rate-limited still learns whether
finished work is sitting on a branch — the question the owner's delete-after-merge rule makes
load-bearing.

## It found real drift immediately

**Four local branches hold zero commits `main` lacks and are still present:** `bpb-split`,
`inference-fix`, `pcopy-accessor`, `repro-fix`. `AGENTS.md` separately names `bpb-split` as outstanding,
belonging to its owner. They are reported with the exact command and **not touched** — the view reports,
the owner decides.

## And it caught itself about to be dangerous

The first version classified `codex/geometric-path-repair` as **finished** and offered a delete — because
its commits are all already upstream — **while its worktree holds twelve uncommitted changes.**

**Finished-by-commits is not safe-to-delete.** This is the same lesson as the guarded branch delete three
rounds ago, arriving in a new place: a rule that reasoned about one kind of "done" and missed the other
copy of the work.

Held branches are now reported under a **"do NOT delete"** heading **with no fix command** — because a fix
line under that heading is exactly what a lab would paste. **A test asserts the renderer never emits a fix
for a held branch**, so the data and the advice cannot drift apart.

```
cargo fmt / clippy -D warnings        -> clean, 0 findings
cargo test -p uor-r4-harness          -> 86 passed + 1 doc-test
python3 -m unittest                   -> 315 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check  -> 51 rules, 0 errors
python3 scripts/check_no_softmax.py   -> OK (1648, unchanged)
python3 scripts/harness/parity.py     -> all nine comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/harness-rust-port-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

**Clause (a) of the objective is still only partly met, and this PR does not close it.** Delivery is now
proven by content in Rust, but **PR numbers, merge-queue state and the live issue reconciliation remain
Python-only**. The Rust `status` is not yet a command a lab can rely on alone for GitHub truth, and the
rendered output says so rather than implying otherwise.

- **Section 4 (GITHUB) is still unported.** Three of four sections now render.
- The delivery view reads the **last-fetched** `origin/main`, so it can call a branch finished from a
  stale ref. The note is printed every run; it is not corrected automatically.
- A branch checked out in a worktree is reported but **not** evaluated for whether that worktree is
  buildable or mid-operation — only whether it is dirty. A clean worktree in the middle of a rebase would
  read as deletable.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

