# #798 — test(#789-G4): HTTP-level baseline tests pinning the audited profile gaps

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T18:56:05Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-08-18T20:13:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/798
- **Merged:** 2026-08-18T20:13:35Z
- **Merge commit:** 400db82dc029aa3935f7af7318d05a2c21e57f17
- **Base / Head:** main / UOR-Foundation:issue-789-g4-baseline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #789 (G4 next-action: baseline tests against current behavior; the G1/G3 decision list is posted on the issue). Tests only — five HTTP-level tests driving handle_connection over a real socket pair, pinning the audited gaps (AUD-ARCH-003/-004/-008/-009) so the decided semantics land as visible assertion flips. Gates: fmt, clippy -D warnings, wasm-router lib green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

