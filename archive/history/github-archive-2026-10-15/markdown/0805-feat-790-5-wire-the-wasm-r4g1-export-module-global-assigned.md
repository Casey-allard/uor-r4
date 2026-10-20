# #805 — feat(#790-5): wire the wasm r4g1 export — module global assigned, bundle installer exported, static-mode install

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T21:24:23Z
- **Updated:** 2026-10-09T13:08:58Z
- **Closed:** 2026-08-18T21:45:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/805
- **Merged:** 2026-08-18T21:45:19Z
- **Merge commit:** 5a4175b09b6b3ef760d0b3f0c20b54b8d7148a20
- **Base / Head:** main / UOR-Foundation:issue-790-5-wire-wasm-export
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #790 (item 5, per the maintainer's wire-it decision). Exports the atomic bundle installer to wasm (verified in wasm-pack bindings), assigns the module global both frontends gate on, and installs a static-mode bundle when one is served alongside the dashboard (score.r4g1 preferred, honest geometric fallback when absent). Gates: fmt, clippy -D warnings, wasm32 lib, wasm-pack build green, node --check on the worker, bdd builds.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

