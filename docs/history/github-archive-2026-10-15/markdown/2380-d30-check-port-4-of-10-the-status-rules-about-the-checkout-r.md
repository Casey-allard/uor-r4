# #2380 — D30 check-port 4 of 10: the status rules about the checkout (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T21:00:08Z
- **Updated:** 2026-10-13T21:02:52Z
- **Closed:** 2026-10-13T21:02:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2380
- **Merged:** 2026-10-13T21:02:33Z
- **Merge commit:** f79722af294f5eb0c8d9c9496f0ecac4084546c1
- **Base / Head:** main / UOR-Foundation:deepseek/status-rules
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**D30 check-port 4 of 10.** Tooling only.

Decision: **KEEP** (tooling).

## The two rules in this unit

| rule | severity | Python | Rust |
|---|---|---|---|
| `status.branch_claimed` | warn | **1 warn** | **1 warn** |
| `status.no_orphan_branches` | warn | **2 warn** | **2 warn** |

Coverage reads **16 of 40 rules** (harness 3, multilab 6, prep 5, status 2).

These are the two `status` rules about **what this working copy is holding** — what the port can verify offline. They cover the two ways a checkout accumulates state the project says it should not: an **unclaimed branch** and an **orphan branch**.

## Why the other two `status` rules are not here

`heartbeat_receipts` and `registry_vs_github` both read **status cards posted as issue comments**. **They are the only two rules in `check` whose subject is the GitHub conversation rather than the checkout** — porting them means `gh` pagination, comment parsing and the headline grammar together.

**That is a unit of work, not a line.** Riding it along here would produce two rules that *look* ported and **cannot be verified** — the failure this sequencing exists to avoid.

## The distinction each rule turns on

**`branch_claimed` — claims are how a branch becomes answerable.** A branch checked out with no claim is work no lab can be asked about: another lab cannot tell whether it is active, abandoned, or scratch, and the board will not show it. `main`/`master` are exempt because the shared checkout is not work — it is the base every branch comes from.

**`no_orphan_branches` — proof by content, not ancestry.** **The project squash-merges, and a squash merge leaves no ancestry link** — so an ancestry check would call a merged branch *unmerged* and **this rule would never fire on the merges the project actually performs.** `githelp::merge_state` does the content comparison, and the rule reports only `Merged`.

**An `Unknown` branch is left alone, because a rule that guesses "merged" tells a lab to delete work that may be live.**

A branch that is **checked out** is not an orphan — it is in use — and the rule **reuses `githelp`'s own `checked_out` flag rather than forming a second opinion about what "in use" means**, since **two definitions of one rule is the defect class this line keeps producing.**

```
cargo fmt --check                               -> clean
cargo clippy --all-targets -- -D warnings       -> 0 findings
cargo test -p uor-r4-harness                    -> 250 passed (7 new)
python3 -m unittest discover -s tests/harness   -> 319 passed
parity.py                                       -> exit 0, rule inventory: 16 implemented
```

Record: `docs/labs/port-check-status-2026-10-13/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried
- [x] Branch deleted after merge

### Stated rather than hidden

- **`check` still delegates.** 4 of 10 modules; six remain.
- **A genuinely orphaned branch whose content comparison returns `Unknown` is silently kept** — the safe direction, but **a miss, not a pass.**
- **This covers branches, not worktrees.** Python's rule also names **clean worktrees whose branch is merged**; that half needs `githelp`'s worktree listing consolidated, and it is recorded rather than half-done.
- **The agreement measurement is a hand-run probe**, read and removed — nothing in CI compares per-rule finding counts.
- **The fix text differs from Python's** (`uor-harness git clean-merged` vs `python3 … cli.py`). The counts agree, but a script scraping the fix string sees different words — **and the Rust string is the one that will still be valid after the port.**

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

