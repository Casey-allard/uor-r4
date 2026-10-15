# #799 — fix(#790-8): CLI state-file writes are atomic and fail loudly

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T18:59:15Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-18T19:47:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/799
- **Merged:** 2026-08-18T19:47:38Z
- **Merge commit:** 8f560156626c271e88b163f4337ad7c4b54924a0
- **Base / Head:** main / UOR-Foundation:issue-790-8-atomic-engine-writes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #790 (item 8, individual PR per the batch convention). Atomic temp-file+rename persists with surfaced errors for the CLI's last_model_name/last_engine state files; falsifier test included. Gates: fmt, clippy -D warnings, wasm-router lib green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

