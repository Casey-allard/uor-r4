# #2021 — Verified restorable bundle for the six archived DeepSeek branches (split) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:27:23Z
- **Updated:** 2026-10-09T14:18:23Z
- **Closed:** 2026-10-09T14:15:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2021
- **Merged:** not merged
- **Merge commit:** ef298a9d91f982eb922992ba7af91c89b739af8c
- **Base / Head:** main / UOR-Foundation:deepseek/branch-archive-clean
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The patch-only archive was not reliably restorable: a test on 2026-10-09 applied 0 of 6 as plain patches and 1 of 6 with a three-way apply, because the branches were cut from older merge bases, main has moved since, and some content is already on main. RECORDING work is not the same as being able to RESTORE it, so this adds the reliable form. deepseek-branches.bundle is VERIFIED: 'git bundle verify' reports a complete history, and fetching it into a scratch clone restored all six branches with tips matching the originals (entry-isolation 1d98faaea, d20-run d5172e7f6, and four more). It is ~115 MB because it carries the complete object closure of the six branches, which is exactly what makes it independent of this repository's history; GitHub rejects any single file above 100 MB, so it is stored as ds-bundle.part00 (90 MB) and ds-bundle.part01 (25 MB) with the reassembly command, the failed approaches and the size documented in INDEX.txt. The branch is rebuilt from efcfe3cd0, so the oversized single-file blob never enters branch history. References #820

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6082709529 — 2026-10-09T14:15:36Z — Casey-allard

Superseded by the light archive PR, which keeps main to an index and moves the 115 MB bundle to iCloud (icloud:UOR-R4/results/deepseek/ds-bundle-icloud.tar, tar md5 3d6233391fa48623e18589e2778d326a, bundle md5 14d17ccd22f1ee196e1479866823be4f). Reasons: GitHub rejects files above 100 MB, two-part storage invites reassembly mistakes, and the project's own storage decision puts bulk material in iCloud with main holding the index. The new PR is 428 KB and carries the same patches plus a fuller index including the failed restoration approaches. References #820


---
_Comment/review ids in this file: 6082709529_

