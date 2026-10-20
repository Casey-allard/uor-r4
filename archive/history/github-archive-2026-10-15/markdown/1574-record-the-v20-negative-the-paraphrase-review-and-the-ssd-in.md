# #1574 — Record the v20 negative, the paraphrase review and the SSD incident

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T13:59:42Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-10-01T14:00:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1574
- **Merged:** 2026-10-01T14:00:44Z
- **Merge commit:** 498d001eac331f14d6b354aa3d1efe387d718cdf
- **Base / Head:** main / UOR-Foundation:codex/state-after-v20
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Docs only. References #1552, #820.

- **`current-state.md`**, after the grounded-session CLI paragraph, gains three paragraphs:
  - v20's combined relation head as a recorded negative: relation 0.922 vs 0.900, but Correct exact 41/313 vs 272/313, so the session gives 26/52 vs v19's 34/52;
  - the reviewed paraphrase derivative from #1573: 394 rows, with `hometown/update` empty;
  - the 1 October storage incident and the owner's write pause.
- **`STATUS.md`:** the semantic-compiler row notes the v20 negative and the reviewed derivative.

Every figure is from #1552 cards v19–v20 and #1573 (development cell, one draw), and none is a gate claim.

**Checks:** `git diff --check` clean; `check_claim_wording.py` passed.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5380316444 — 2026-10-01T13:59:52Z — Casey-allard — COMMENTED

Exact-head self-review at `5e9d0b3acdc61e21b77cdf97b953e23bfd31ccc5` (docs only).
- Every figure checked against its source:
  - v20 compiler relation 1,934/2,098 (0.922) vs the table's 1,889/2,098 (0.900), and Correct 41/313 vs 272/313: `compiler-save-3/report.json` and `compiler-save-2` as posted on #1552;
  - session 26/52 vs 34/52: the run log and the v19 report;
  - review counts 372/22/260/122 and 394 rows: `decisions.tsv` in #1573;
  - incident facts as posted on #820.
- The scope wording (development cell, one draw, not a gate claim) is kept. The incident is described as a storage fault, not a model result.
- `git diff --check` clean; `check_claim_wording.py` passed.

Approve for the queue.

### comment 5933028371 — 2026-10-01T14:00:59Z — Casey-allard

Merged as `498d001e`. The squash tree equals `git merge-tree --write-tree` of the parent `e126867f` and the reviewed head `5e9d0b3a`.


---
_Comment/review ids in this file: 5933028371, 5380316444_

