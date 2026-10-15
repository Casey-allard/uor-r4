# #803 — fix(#790-6): one store root for every artifact and state path; sanitized compiled-bundle names

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T21:19:30Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-18T21:40:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/803
- **Merged:** 2026-08-18T21:40:30Z
- **Merge commit:** 51e88efee785c31a609c406108528c8dadba919d
- **Base / Head:** main / UOR-Foundation:issue-790-6-store-path-resolution
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #790 (item 6, individual PR). Single store-root resolution (env-aware) for the ask-path artifact preference and all three persisted state files, readers and writers together; compiled-bundle paths sanitize names like manifest reads do. Default behavior byte-identical (verified live: bare ask unchanged). Gates: fmt, clippy -D warnings, wasm-router lib 250 green, wasm32 lib.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

