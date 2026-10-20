# #419 — feat(certify,cli): #399 — A-mode infill serving surface (skeleton fill + graph infill subcommand)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-05T00:07:53Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-08-05T00:46:49Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/419
- **Merged:** 2026-08-05T00:46:49Z
- **Merge commit:** 3f3aec4fc3bf5fca74d3b5d38df5ce18c0e52b0d
- **Base / Head:** main / UOR-Foundation:i399-amode
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #399. Productizes the VALIDATED A-mode: anchors given as inputs (drift-immune by construction), free positions filled with the fused scorer (+4.2pp live record, M2 row).

**Surface:** `infill_fill(scorer, artifacts, rotations, skeleton: &[Option<u32>])` — any pattern of givens; nearest following anchor within distance 1..=3 keys the FWDA row, farther/absent falls back to base scoring; givens consumed verbatim into context. `next_skeleton_anchor` exposes the anchor rule. CLI: `r4 graph infill --artifact <score.r4g1> --skeleton "12,_,_,_,99,_,_,_,7" [--teacher <TLA>]` → filled ids + fill summary. Runtime engine default path and witness/replay contract untouched.

**Tests:** 5 new non-ignored (givens verbatim, live-row flip, absent-FWDA == base scoring exactly, far-anchor fallback, CLI end-to-end + fail-closed args). Workspace fmt/clippy/test-compile clean.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

