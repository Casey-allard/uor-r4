# #1558 — Record D18 outcome D (owner-confirmed 1 October)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T01:51:43Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-10-01T01:52:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1558
- **Merged:** 2026-10-01T01:52:48Z
- **Merge commit:** f0502629f3f7d050ec42e6399e83058a169b22e3
- **Base / Head:** main / UOR-Foundation:codex/d18-outcome-d
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Docs only. This updates STATUS.md, the current state and D18 with outcome D, as D18 §8 requires at each outcome.

- **A1's pre-registered kill was met.** Every arm scored below 0.5 MQAR recall at d16 on the development cell, the transformer control at 2× steps included (it scored 0).
- **Pointer arms copy recency, not the queried key.** They score 0.28–0.44, falling as about 1/N, and do not beat the untrained most-recent-value rule (0.36).
- **The owner confirmed outcome D** at about 01:50 UTC.
- **§8 count:** up to 317 positions with a 32-token reply budget.
- **Merged:** #1553–#1557.
- **Next:** the exact-log plus prime-sieve memo.

Full tables, roots and binaries: #1552.

**Checks:** `git diff --check`; `python3 scripts/check_claim_wording.py` (passed); link targets exist. No code changes, so no compile or test run applies beyond the queue's checks.

References #1552

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5923125245 — 2026-10-01T01:51:57Z — Casey-allard

**Claude lab, exact-head self-review at `e40f691b`: APPROVE** (owner rule, #820). Docs only: 3 Markdown files and no compiled source, the same class as #1553. I checked every number against the sealed reports cited on #1552: the arm table, the R-recency row, the §8 count and the census figures. The outcome wording matches the owner's 01:50 UTC ruling. `git diff --check` and `check_claim_wording.py` pass.

### comment 5923155431 — 2026-10-01T01:54:59Z — Casey-allard

Merged as f0502629f3f7d050ec42e6399e83058a169b22e3. Verified: `git merge-tree --write-tree c190c7c2 e40f691b` equals the squash tree (ed22b3de).


---
_Comment/review ids in this file: 5923125245, 5923155431_

