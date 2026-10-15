# #2239 — Starling read vs softmax at equal budget: behind at 512, fails at 1536 from eviction (row 4); #2233 re-scoped (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T03:39:15Z
- **Updated:** 2026-10-12T03:44:17Z
- **Closed:** 2026-10-12T03:40:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2239
- **Merged:** 2026-10-12T03:40:16Z
- **Merge commit:** cc1f316bb99c86ffac0ad848fe1060e88194184f
- **Base / Head:** main / UOR-Foundation:claude/starling-context-result
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 1/3 · headline "softmax at runtime: yes" → yes. References #2032.

Murmuration ladder, step 2 ([pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6116807513) with its amendment; FIT). [Record](docs/labs/starling-context-2026-10-12/README.md).

| Context | L (softmax), mean | W03L (starling), mean | Holds? |
|---|---:|---:|---|
| 512 | 0.991 | 0.831 | no (misses the 0.841 bar by 0.010) |
| 1536 | 0.999 | 0.241 | no |

**Decision:** row 4.

**Cause:** the fixed 120 × M store holds 94% of a 512 window but only 31% of 1536. Forcing the source into the kept set lifts recall from 0.53 to 0.83 on the working seed, so selection works and retention doesn't scale.

**Re-scoping:** #2233's KEEP stands against W03, not against softmax. The earlier L figure of 0.617 was at 1,800 steps.

**Skipped runs:** the 4096 runs were skipped (about 2.5 GPU-hours) once row 4 was fixed at 1536.

**Checks:** docs only; the claim-wording gate passes. The runs used binary `c04b2f31f`, `device=cuda`, and every report shows 3,600 steps with no early stop. Outputs are in the cloud store (md5 bbac0fea…). STATUS, ROADMAP and #2028 are updated.

**Next:** ladder step 3, a persistent base-3 tree over the past (HoloBox path-copy, FBS fan-out).

**Would the next step differ if this came out the other way?** Yes. A hold would have sent the line to language (step 4).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5486330681 — 2026-10-12T03:39:16Z — Casey-allard — COMMENTED

Self-review at exact head 8feca3958930dd231de88f8f4b253bc1ff924a4b: docs only; numbers match the archived logs; the rows are scored per the pre-registration; the #2233 re-scope is stated in both records. APPROVE (self).


---
_Comment/review ids in this file: 5486330681_

