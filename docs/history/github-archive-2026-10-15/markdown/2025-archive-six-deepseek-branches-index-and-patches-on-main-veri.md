# #2025 — Archive six DeepSeek branches: index and patches on main, verified bundle in iCloud (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:14:54Z
- **Updated:** 2026-10-09T14:18:25Z
- **Closed:** 2026-10-09T14:16:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2025
- **Merged:** 2026-10-09T14:16:50Z
- **Merge commit:** 46ec04c059ed24d4d8dbae80d008b8172c0793e3
- **Base / Head:** main / UOR-Foundation:deepseek/branch-archive-light
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Ancestry said 141 of 146 deepseek branches were unmerged and content diffs said nearly all; both are wrong because deliveries here are squash-merged. Patch-id comparison (git cherry) shows only SIX carried work not already on main, preserved here as per-branch patches with a full commit index: aa-dispatch, bf16-phase2b-gate, d20-control, d20-run, entry-ceiling, entry-isolation. Status is RETAINED, NOT ACTIVATED - an archive for restoration, not a promotion of code into the live tree. Restoration uses the bundle, VERIFIED rather than asserted (git bundle verify reports a complete history; a scratch-clone fetch restored all six branches with tips matching the originals). The patches alone are not reliably applicable (0 of 6 plain, 1 of 6 three-way) and that is recorded. The bundle lives in ICLOUD, not the repository: 115 MB, complete object closure, GitHub rejects files above 100 MB, and the project's storage decision puts bulk in iCloud while main holds the index. INDEX.txt records the iCloud object, both md5s and the restore commands. This PR is 428 KB and supersedes #2021, which stored the bundle as split parts on a branch. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

