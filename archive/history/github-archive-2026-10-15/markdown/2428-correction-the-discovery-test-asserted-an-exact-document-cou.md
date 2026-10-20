# #2428 — Correction: the discovery test asserted an exact document count (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T03:32:29Z
- **Updated:** 2026-10-14T03:32:42Z
- **Closed:** 2026-10-14T03:32:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2428
- **Merged:** not merged
- **Merge commit:** 3621c4aa359a4057e4adbc8a4bd4d17b91523191
- **Base / Head:** main / UOR-Foundation:deepseek/discovery-count-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**Correction: a test asserted an exact document count, and it was mine.** Tooling only.

Decision: **KEEP** (tooling).

## What broke, and why CI caught it and I did not

`doccheck::tests::discovery_agrees_with_python_on_the_real_repo` asserted `docs.len() == 927`. **CI failed with `left: 928`.**

**The count is not a property of the code.** It changes **whenever any lab adds a Markdown file** — **including the record I was writing in that same unit.** My local checkout was at 927 and CI checked out the merged head, so **the assertion passed here and failed there**, which is the worst way for a test to be wrong: **it agreed with the tree I was looking at rather than the tree being merged.**

## The fix is not "update 927 to 928"

**A count that must be edited whenever a doc is added is not a test of discovery — it is a test of whether someone remembered to update it.** The number is gone; **what the rule actually promises is asserted instead**:

- `AGENTS.md` sorts **first**, then `CONFORMANCE.md`, `CONTRIBUTING.md`;
- **the exclusions hold on real data**, not only in a fixture — no `docs/history/`, no `current-state.md`, **no dated record**, nothing outside the root and `docs/`;
- **every reported invocation is in a doc discovery returned.**

## Third time in this port that a fixture encoded a transient

The previous two: the `prep.rs` citation list stale against the registry, and the `prlint` fixture **captured while the tree was dirty**, encoding a finding that no longer existed.

**All three share one shape: an assertion true of the tree at the moment it was written, and not a property of the code.** **The first two I found by reasoning about them. This one CI found for me** — and the difference is that **I asserted an exact number about a quantity other people control.**

```
cargo test -p uor-r4-harness                    -> 379 passed
cargo fmt --check / clippy -D warnings          -> clean / 0 findings
python3 -m unittest discover -s tests/harness   -> 332 passed
python3 scripts/check_no_softmax.py             -> OK — 1641, baseline 1641
parity.py                                       -> exit 0
```

Record: `docs/labs/discovery-count-fix-2026-10-14/README.md`

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record + current-state entry carried, presence asserted before commit
- [x] Branch deleted after merge

### Stated rather than hidden

- **The assertion still names the second and third documents.** A new root-level Markdown file sorting before `CONFORMANCE.md` would fail it — **I moved it from a number that changes weekly to a name that changes rarely, not to something that cannot change.**
- **The properties are weaker than the count was.** The count would have caught a discovery that merged or split a doc; **the properties do not, and nothing else does either.**
- **`check` still delegates.** This unit fixes my regression and **moves no coverage.**

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6160270196 — 2026-10-14T03:32:42Z — Casey-allard

Folded into #2427, which carries the ported code this fixes. Closing to keep one PR per change.


---
_Comment/review ids in this file: 6160270196_

