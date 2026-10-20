# #802 — fix(#790-4): remove the dead FallbackRouter pipeline and the legacy engine resolver

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T21:15:14Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-08-18T21:36:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/802
- **Merged:** 2026-08-18T21:36:08Z
- **Merge commit:** 772b68b83b974d8faa384def2119d961c4ee71a8
- **Base / Head:** main / UOR-Foundation:issue-790-4-dead-serving-code
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #790 (item 4, individual PR per the convention). Removes the audit-confirmed dead serving code (FallbackRouter pipeline + satellite types + never-selected policy/status variants + the legacy resolver) and repoints the BDD scenarios at the live resolver semantics. Live cascade behavior byte-identical; router 18+2 tests green, workspace clippy -D warnings, wasm32 lib, bdd builds.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

