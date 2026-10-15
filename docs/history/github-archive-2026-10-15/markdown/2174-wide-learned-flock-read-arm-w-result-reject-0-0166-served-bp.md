# #2174 — Wide learned flock read (arm W) result: REJECT (+0.0166 served BPB, 15 % slower); support size is not the gap (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T19:07:22Z
- **Updated:** 2026-10-10T20:51:14Z
- **Closed:** 2026-10-10T20:50:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2174
- **Merged:** 2026-10-10T20:50:59Z
- **Merge commit:** a5f83132a6c426e44f985abbb2cb3442e0ba0e98
- **Base / Head:** main / UOR-Foundation:claude/wide-flock-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: wide learned flock read (owner-funded new line) · count 1/3 · headline softmax at runtime yes → yes (served 0.886838 BPB unchanged)

**Result PR of the line's one decisive run**, [pre-registered](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6099092880), with no preparation PR (all code was already on main). Decision test: a KEEP would have made the wide learned flock the default served read. This REJECT, together with its support-size finding, sends the next run to training the rank read rather than widening it.

## Result

Held-out e5f400b0, 512 windows × 384, stream basis. Served = D11 on the same windows.

| arm | float BPB (s1/s2) | served BPB (s1/s2) | v4 (s1/s2) | Δ served vs A | D11 speed vs A (back to back) |
|---|---|---|---|---:|---|
| A softmax (#2150) | 0.877118 / 0.877277 | 0.886817 / 0.886702 | 6 / 3 | — | 228.4 tok/s |
| L learned `flock:8:8` (#2160) | 0.895331 / 0.896049 | 0.904403 / 0.905052 | 3 / 3 | +0.0180 | — |
| **W learned `flock:32:32`** | **0.894703 / 0.894072** | **0.903271 / 0.903509** | 3 / 6 | **+0.0166** | **195.2 tok/s (−15 %)** |

- **REJECT:** the BPB bar (0.01) and the cost clause both miss.
- **Finding:** widening the support from 17 to 66 sources buys only 0.0013 BPB, so the gap is the rank weighting or its training, not support size.
- **D22:** this closes the `flock:32:32` configuration, not the mechanism. Next, per the [brief](https://github.com/UOR-Foundation/uor-r4/blob/main/docs/mechanisms/flock-rank-reads.md): a rank-consistent training gradient.

## Artifacts and cost
- **Record:** `docs/labs/softmax-free-read-2026-10-10/README.md` § New line, plus a current-state entry with **Next:**. README and STATUS rows updated.
- **Archives:**
  - cloud-store `claude/wide-flock-20261010` (checkpoints, MD5 `f662f1d1…`);
  - `claude/wide-flock-eval-20261010` (evaluation, MD5 `fd9fd4d6…`);
  - `claude/chat-data-control-20261008` (the data streams, so later evaluations need no pod).
- **Compute:** pod about 2.4 h on 2×4090, about $4.30, plus a 3-minute data pod, both deleted. Laptop CPU for the evaluation.

## Checklist
- [x] Numbers read from the sealed `evaluation.json`, `report.json` and grade reports in the archives
- [x] Record, current-state entry, README and STATUS updated
- [x] `check_claim_wording.py` passes

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6101143117 — 2026-10-10T19:07:35Z — Casey-allard

Exact-head self-review at `c1e40d1dc7eb1dbff76f3e450408b5531739e922`: docs and record only, based on fresh origin/main. Every number was read from the sealed reports in the cloud-store archives, and the claim-wording gate passes. No code change.

### comment 6102036009 — 2026-10-10T20:49:46Z — Casey-allard

Exact-head self-review at `ea19cd0ceb4810e117f12b3ea66869d69ba1727b`: merged origin/main (current-state.md keeps every entry); docs only, and the claim-wording gate passes.


---
_Comment/review ids in this file: 6101143117, 6102036009_

