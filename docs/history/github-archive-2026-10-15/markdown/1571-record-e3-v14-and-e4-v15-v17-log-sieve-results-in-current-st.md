# #1571 — Record E3 v14 and E4 v15–v17 log-sieve results in current state

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T08:49:32Z
- **Updated:** 2026-10-09T13:08:45Z
- **Closed:** 2026-10-01T08:50:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1571
- **Merged:** 2026-10-01T08:50:36Z
- **Merge commit:** 7050f0d3774b8823da1dd1d769f1b70922e42c08
- **Base / Head:** main / UOR-Foundation:codex/log-sieve-status
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Docs only. References #1552.

**Why.** Under D19, `current-state.md` and `STATUS.md` still reported E4 v12 (open relation 25/52) and E3 v11. Cards v13–v17 on #1552 have since finished.

**What changes:**
- `current-state.md`, *Concurrent work*: a card table (v13 batch; v14 E3; E4 v12 → v15 → v16; v17 negative), v16's miss analysis, and a note that v14–v16 used the raw paraphrase union, which the 07:04/07:20 audits found carries label errors.
- `STATUS.md`: the semantic-compiler row moves to v14; the E4 paragraph moves to v16 (36/52) with v17 recorded as a negative.

**Scope.** Every figure is a development-cell diagnostic (one draw, `emit-1`, seed 9101), not a gate claim. E3's original gate still fails. No new run.

**Checks:** `git diff --check` clean; `python3 scripts/check_claim_wording.py` passed.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5377030595 — 2026-10-01T08:49:42Z — Casey-allard — COMMENTED

Exact-head self-review at `f32d064ba81281368fe3eb8ef068a6c373de65cb` (docs only).
- Every figure checked against its #1552 card comment: v13 403/23 (`b9a5f451…`); v14 trunk 1,754/2,098 relation and 2,808/3,664 act, combined 0.949, table 0.900; E4 v12 25/52, v15 30/52, v16 36/52; v17 33/52; MQAR 106/109; oracle 48/52; best trained read 27/52; closed 2/17.
- v16's miss split (14 lookups, 2 emissions) matches the v16 comment.
- The scope line (development cell, one draw, not a gate claim) and the label-audit caveat are kept.
- `git diff --check` clean; `check_claim_wording.py` passed at this head.
Approve for the queue.

### comment 5928057457 — 2026-10-01T08:53:15Z — Casey-allard

Merged as `7050f0d3`. The squash tree `aac1a0ba20cb7a341cf17e795e9a6329172f294d` equals `git merge-tree --write-tree` of the parent `98d5546a` and the reviewed head `f32d064b`.


---
_Comment/review ids in this file: 5928057457, 5377030595_

