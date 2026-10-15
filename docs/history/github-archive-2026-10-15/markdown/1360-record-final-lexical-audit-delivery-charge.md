# #1360 — Record final lexical audit delivery charge

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-23T06:04:47Z
- **Updated:** 2026-10-09T13:17:44Z
- **Closed:** 2026-09-23T06:05:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1360
- **Merged:** 2026-09-23T06:05:52Z
- **Merge commit:** 5ead13e31e8520659d5d94265512dbb9876f1b18
- **Base / Head:** main / UOR-Foundation:codex/ordinary-audit-ledger-final
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

PR #1359 merged the corrected lexical audit and public handoff. This ledger-only follow-up records the complete principal delivery charge once in the protected repository, matching the owner checkout's atomically updated cumulative JSON: 387,615,640 / 390,700,000 ms, with 3,084,360 ms remaining. It also records the new sealed audit's retained bytes and measured physical free space. No model behavior or capability claim changes; no model run was executed in this follow-up. References #820, #963 and #973.

Validation: `git diff --check`; compared the ledger amounts with the live owner JSON. Historical CI names are compatibility acknowledgements, not executed Rust checks.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

