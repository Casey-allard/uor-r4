# #797 — feat(#785-C3): decode witness on ask/chat — engine, tier split, plain-path depths; name the R4G1 downgrade (#790-7)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T18:40:19Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-18T19:01:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/797
- **Merged:** 2026-08-18T19:01:02Z
- **Merge commit:** ab7b3bced35350d8604b0f4fab37badb9f2114a4
- **Base / Head:** main / UOR-Foundation:issue-785-c3-decode-witness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #785 (C3). References #790 (item 7's log half).

Additive observability, no decode behavior change: ChatAnswer carries a DecodeWitness (serving engine + per-candidate-query node/non-node split via the #785-C1 buffer contract + the plain path's per-token resolution depths), logged at info; a present-but-unparsable graph now warn-logs its downgrade to the plain path instead of silently falling through. End-to-end test drives both engines through ask() on the core fixture artifacts. Live: the canonical bundle's digit attractor is attributably row-tier (engine=r4g1-beam, non_node=381, node_path=0).

Gates: fmt, clippy -D warnings (workspace), wasm-router lib 242 green, wasm32 lib build.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

