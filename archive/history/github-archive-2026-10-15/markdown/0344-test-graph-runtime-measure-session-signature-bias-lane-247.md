# #344 — test(graph-runtime): measure session signature bias lane (#247)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T03:40:05Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-08-02T03:54:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/344
- **Merged:** 2026-08-02T03:54:59Z
- **Merge commit:** 967c0297928e9c78665bafc34df0d5657c9f2811
- **Base / Head:** main / UOR-Foundation:issue-247-session-eval
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
- add a deterministic held-out session-signature A/B harness for the graph runtime
- compare context-only, actual story-prefix session bias, and reversed-prefix session bias
- report corpus top-1, teacher-argmax agreement, changed predictions, and changed scores
- document the command and keep session state out of ROUT fallback

## Evidence
- `cargo fmt --all -- --check`
- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- local 17-position probe on the pinned 200k SmolLM2 bundle: session scores changed on 16/17 positions; token choices unchanged in this small sample

The harness is measurement-only and does not claim a quality improvement or enable session signatures for ROUT fallback. Refs #247.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

