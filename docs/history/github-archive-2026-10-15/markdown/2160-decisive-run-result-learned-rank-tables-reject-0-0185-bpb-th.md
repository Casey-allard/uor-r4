# #2160 — Decisive run result: learned rank tables REJECT (+0.0185 BPB); the softmax-free read line stops (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T13:58:59Z
- **Updated:** 2026-10-10T14:00:19Z
- **Closed:** 2026-10-10T14:00:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2160
- **Merged:** 2026-10-10T14:00:02Z
- **Merge commit:** c404b0ca02f2c4f1b783bbfb1b7e26464931be45
- **Base / Head:** main / UOR-Foundation:claude/learned-rank-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: softmax-free served read (M4 item 0) · count 5/3, the result of the single decisive continuation; the line now stops · headline softmax at runtime yes → yes (served 0.886838 BPB unchanged)

**The result of the one decisive run** named by the 3/3 [pivot card](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6096272058), prepared by #2153. Decision test: a KEEP would have made the learned-table read the default served read and met item 0 for reads. This REJECT stops the line and leaves item 0 open.

## Result

Held-out e5f400b0, 512 windows × 384 = 196,608 targets, the stream basis. Served = D11 on the same windows.

| arm | float BPB (s1/s2) | served BPB (s1/s2) | v4 (s1/s2) | Δ float vs A | verdict |
|---|---|---|---|---:|---|
| A softmax (#2150) | 0.877118 / 0.877277 | 0.886817 / 0.886702 | 6 / 3 | — | reference |
| B fixed rank (#2150) | 0.917017 / 0.917037 | 0.925884 / 0.925945 | 3 / 6 | +0.0398 | REJECT |
| **L learned rank** | **0.895331 / 0.896049** | 0.904403 / 0.905052 | 3 / 3 | **+0.0185** | **REJECT** (bar 0.01) |

- The learned profile closes 54 % of the fixed table's gap; the rest is about twice the bar.
- v4: Δ −1.5, inside the tolerance.
- D11 speed: 143.4 tok/s against A's 151.8. That is not a matched timing; it is in the range seen for B and D.
- **Decision:** REJECT. The line stops. The D11 machinery and trainer options are kept. Arm C (copy head on B's read) is not run: its base read is rejected. Item 0 stays open and goes to the owner on #2032. Next: M4 items 1–3 on the softmax-read model.

## Artifacts and cost
- **Record:** `docs/labs/softmax-free-read-2026-10-10/README.md` § Result (decisive run), plus a current-state entry with **Next:**.
- **Results:** cloud-store `claude/learned-rank-20261010` (184,948,736 bytes, MD5 `516294b0…`, verified by download), and the EU-RO-1 volume.
- **Compute:** pod `583vvwhk05yy1p` (2×5090, about 1.9 h, about $4.50), deleted. Laptop CPU for the evaluation.

## Checklist
- [x] Numbers read from the sealed `evaluation.json`, `report.json` and grade reports in the archive
- [x] Record and current-state entry with **Next:**
- [x] The #2028 M4 row is updated after merge (state: line closed); STATUS/ROADMAP headline unchanged
- [x] `check_claim_wording.py` passes

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6098252101 — 2026-10-10T13:59:10Z — Casey-allard

Exact-head self-review at `464cc7a85311ae3df283cc4b4b9ae387ffd89d9f`: docs and record only, based on fresh origin/main 0d4ade12. Every number was read from the sealed evaluation, report and grade files in the cloud-store archive, and the claim-wording gate passes. No code change.


---
_Comment/review ids in this file: 6098252101_

