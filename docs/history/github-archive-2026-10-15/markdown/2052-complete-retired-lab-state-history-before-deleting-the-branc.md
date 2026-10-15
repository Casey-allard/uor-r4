# #2052 — Complete retired lab-state history before deleting the branch

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T16:18:13Z
- **Updated:** 2026-10-09T16:19:54Z
- **Closed:** 2026-10-09T16:19:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2052
- **Merged:** 2026-10-09T16:19:24Z
- **Merge commit:** 42196a34b076bc85e33633b5baf08fbdc98b44b2
- **Base / Head:** main / UOR-Foundation:codex/retire-lab-state-tail-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retired coordination branch received two heartbeat commits after #2048. Fresh restoration also exposed that the original bundle came from a shallow checkout and omitted older ancestors. This replaces it with the complete 1,976-commit history through `d9eca2ab1d8240bd854e2ae92bf29f8f1781c765`, retains the two-event delta as an indexed patch, and corrects the restore instructions.

Validation at head `3d74739f2`: fetched the final GitHub branch into an isolated non-shallow repository, restored the new bundle into a second empty bare repository, passed full object verification, and matched final commit/tree and 1,976-commit count. Claim wording and diff whitespace checks pass. Changes are archival only; no model execution or coordination writer was invoked. The original bundle remains in Git history.

After protected merge and fresh-main blob verification, delete `codex/lab-state` with an exact-tip lease, then delete this temporary delivery branch. No standing branch will be recreated. References #2028 and #2048.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

