# #2071 — VSA record correction: the scale is trained, codes are not; retire rule withdrawn

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T18:32:55Z
- **Updated:** 2026-10-09T18:34:38Z
- **Closed:** 2026-10-09T18:34:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2071
- **Merged:** 2026-10-09T18:34:17Z
- **Merge commit:** 3662f4852e4336ce526125bbb66d5d947e52d622
- **Base / Head:** main / UOR-Foundation:claude/vsa-record-correction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This corrects #2070's VSA record under the delivery cadence's honesty rule:
- **The scale is trained.** The independent review said nothing trains the VSA weight, but the code scout found the scale trained by gradient and Adam in `jepa_trainer.rs`. Only the codes are fixed hashes, and the frozen-artifact critique stands.
- **The retire rule is withdrawn.** The owner withdrew "retire below 0.005" on 9 October: if the test misses, VSA is made more native, not retired.

Both the record and its `current-state.md` entry carry the correction; the original text stays visible with a correction note.

References #2029

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6086939857 — 2026-10-09T18:32:56Z — Casey-allard

Exact-head self-review at `091635539feb0313f634bf7b988975f79163c5c6`: two-file docs correction; claim wording passes.


---
_Comment/review ids in this file: 6086939857_

