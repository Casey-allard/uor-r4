# #1942 — Panel v2: repair the instrument defects before spending on the target (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:34:20Z
- **Updated:** 2026-10-09T13:21:29Z
- **Closed:** 2026-10-08T17:36:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1942
- **Merged:** 2026-10-08T17:36:20Z
- **Merge commit:** 6e8ac4493237e29344156c5c95f0c5ce5e583f0d
- **Base / Head:** main / UOR-Foundation:deepseek/panel-v2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The review showed the frozen six-item compliance panel cannot separate the two live explanations for the instruction-precision failure - data support (only 4 eligible one-word episodes after the context filter) against token-level MLE length bias (~250:1 continuation mass after Yes.) - because two items are weakly posed and the probe reads the separator, making it the wrong instrument for a second-token failure. Panel v2 removes the echo item, reads the first TWO content tokens per reply, and covers three yes/no, three numeric, one list, two factual, two retention pairs and one follow-up. Retention and follow-up are kept as the regression guard. The running arms were launched with the earlier panel and remain valid for it. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

