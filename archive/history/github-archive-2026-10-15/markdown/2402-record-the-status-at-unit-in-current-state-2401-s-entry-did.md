# #2402 — Record the status --at unit in current-state (#2401's entry did not land)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T00:37:38Z
- **Updated:** 2026-10-14T00:39:23Z
- **Closed:** 2026-10-14T00:39:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2402
- **Merged:** 2026-10-14T00:39:04Z
- **Merge commit:** 0cceea6f0f43cd838a5f5016383c4fe0ae6c7cf9
- **Base / Head:** main / UOR-Foundation:deepseek/current-state-at
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

**Correction: #2401's current-state entry did not land.** Tooling only.

Decision: **KEEP** (tooling).

## What happened

PR #2401's commit message stated it carried a current-state entry. **It did not.** The insert script's anchor did not match, the `assert` raised, **and the commit proceeded anyway** — so the message claimed a record the commit does not contain.

**This is the same silent no-op class as the mis-edited `parity` call site in that same unit: twice in one PR.** An edit that lands nowhere, and an edit that lands in the wrong place, **both look exactly like an edit that worked.**

## Fixed

The entry is written at the head of `docs/integration/current-state.md`: the defect, the fix, the mis-edited call site, the honest note that I could not prove the fix by reverting it, and the limitations.

**`git diff --stat` shows 8 insertions and no deletions**, so nothing was lost. And **my first two "verification" greps reported 0 because the pattern was wrong, not because the file was** — the heading spans an em dash, and I had to simplify the pattern to see it. **That is the same trap that produced two false alarms earlier in this session**, which is worth stating rather than presenting the check as having worked first time.

Record: `docs/labs/status-clock-injection-2026-10-13/README.md` (from #2401)

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

