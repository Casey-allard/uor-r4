# #1811 — AGENTS.md: the 09-06 standing authorization defers to the 10-05 GPU pod rules

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T22:56:04Z
- **Updated:** 2026-10-09T13:09:05Z
- **Closed:** 2026-10-06T22:57:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1811
- **Merged:** 2026-10-06T22:57:05Z
- **Merge commit:** 7c92674a26a3628bc0b3f0fb87348484688680fa
- **Base / Head:** main / UOR-Foundation:claude/retire-paid-compute-line
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The 2026-09-06 standing authorization said it authorizes "neither destructive deletion nor paid/external compute". A lab read that as forbidding `uor-pod` pods, although the 5 October owner rules ("GPU pods" section) authorize `uor-pod up` within the caps, and left capacity idle. This one sentence now defers to those later rules: pods within the caps need no further ask, and any other paid or external compute still needs the owner. Docs only. References #820.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6026953734 — 2026-10-06T22:56:07Z — Casey-allard

Exact-head self-review at `c756cb724ba8e794d39a4c5723d2a45ebd10bcca`: one-sentence AGENTS.md edit, read in full; `check_claim_wording.py` rc 0; no policy change beyond clarifying precedence (owner-confirmed on 10-06 that pods within caps need no ask). APPROVE.


---
_Comment/review ids in this file: 6026953734_

