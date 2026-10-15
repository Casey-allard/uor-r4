# #314 — Add auditable provenance to geometric coordinates

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T22:23:11Z
- **Updated:** 2026-07-31T23:24:21Z
- **Closed:** 2026-07-31T23:24:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/314
- **Merged:** 2026-07-31T23:24:21Z
- **Merge commit:** a18d1e79e5e2270ec9f8ad725af86d4af61a29a9
- **Base / Head:** main / UOR-Foundation:issue-259-provenance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
- attach source-text, projection-configuration, and vocabulary-state uor-addr κ addresses to stored geometric coordinates
- verify all provenance links before serving corpus entries
- expose the provenance chain through resonance retrieval results
- add focused tests and implementation documentation

## Validation
- cargo fmt --all --check
- git diff --check
- cargo check -p uor-r4-graph-format --no-default-features
- cargo check -p uor-r4-graph-format --no-default-features --features alloc
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo test --workspace --offline

PR #313/#246 is now merged into `main`; this PR contains the remaining #259 provenance layer.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

