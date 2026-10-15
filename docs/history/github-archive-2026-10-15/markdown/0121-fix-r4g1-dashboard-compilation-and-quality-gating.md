# #121 — Fix R4G1 dashboard compilation and quality gating

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-24T01:54:29Z
- **Updated:** 2026-10-09T14:16:55Z
- **Closed:** 2026-07-24T03:53:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/121
- **Merged:** 2026-07-24T03:53:05Z
- **Merge commit:** 13e8a98cae5e082e6dd726bc349abc3901ec0508
- **Base / Head:** main / UOR-Foundation:agent/r4g1-dashboard-quality
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
- report live R4G1 compilation progress and explain the build stages
- keep the selected router reflected in the dashboard status
- compile cover observations in parallel with deterministic ordering
- use an explicit same-corpus TLA quality profile for dynamic Hugging Face builds
- retain the pinned absolute floor for pinned fixture reports

## Validation
- cargo test --workspace --offline
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo fmt --all -- --check
- cargo check -p uor-r4-graph-format --no-default-features
- cargo check -p uor-r4-graph-format --no-default-features --features alloc

Closes the dashboard compilation and quality-gate issues.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

