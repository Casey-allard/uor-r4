# #519 — feat(#514): scale_sweep.sh — the saturation-sweep harness (β calibration)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T00:56:30Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-09T01:15:49Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/519
- **Merged:** 2026-08-09T01:15:49Z
- **Merge commit:** 27303528dfb54f3cacbe7f12b352ddd52c5a6a44
- **Base / Head:** main / UOR-Foundation:feat/514-sweep-harness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Second #514 deliverable. `scripts/scale_sweep.sh` sub-samples one observation corpus to several record counts (via the existing `scripts/mc1_subsample_corpus.py`, truncating to a story-run boundary so the 80/20 train_cut split is preserved), runs compile-recorded → cover → score at each, and prints `records | held_out | top1_rule12 | exct_miss_%`. The knee (smallest N past which top-1 and EXCT-miss flatten) is that teacher's needed scale; log(N_knee) vs log(S) calibrates the recommend-scale β. Observe once at the top scale — the sweep reuses that corpus.

Validated: mc1_subsample_corpus.py on the wiki-360m corpus (21,235 → 9,882 at a story boundary, valid records/meta). docs/scaling_law.md points at the real scripts.

Remaining #514: β calibration from real knees — runs on #516's observe (in progress on dev hardware). Refs #514.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

