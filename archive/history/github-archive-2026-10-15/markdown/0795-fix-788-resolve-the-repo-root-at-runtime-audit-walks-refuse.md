# #795 — fix(#788): resolve the repo root at runtime; audit walks refuse an empty scan set

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T18:17:02Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-08-18T18:38:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/795
- **Merged:** 2026-08-18T18:38:13Z
- **Merge commit:** 36fa79862e322558aeef96f616654dc047edda15
- **Base / Head:** main / UOR-Foundation:issue-788-runtime-repo-root
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #788. Closes the AUD-VER-001/-002 gate-integrity findings: runtime repo-root resolution (env-var walk + cwd fallback + loud failure), empty-walk refusal in both audit walkers with seeded falsifier tests (the pre-fix behavior on the same fixture was a vacuous 0-second pass), and the AGENTS.md operational notes (stale-worktree rlib hazard, --no-fail-fast). Verified: all register gates green from the live tree via cargo AND via direct binary invocation from the repo root; direct invocation from an unrelated directory fails loudly instead of walking a wrong tree.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

