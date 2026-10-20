# #347 — test: enforce canonical D2 reproducibility (#265)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T20:13:53Z
- **Updated:** 2026-10-09T14:17:13Z
- **Closed:** 2026-08-02T20:27:31Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/347
- **Merged:** 2026-08-02T20:27:31Z
- **Merge commit:** dcd0c2ac7eee2a8bf913674033eb0923dde63363
- **Base / Head:** main / UOR-Foundation:issue-265-d2-acceptance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Completes the remaining acceptance work for #265:

- records `canonical_deterministic` in the κ fixture metadata
- requires `TLESS_CANONICAL_DETERMINISTIC=1` for the Gate E κ reproduction test
- removes Linux's `continue-on-error` from canonical κ reproduction in CI
- updates AGENTS.md, the D2 design note, and BASELINE.md

## Evidence

The 500k/TLA7 fixture was recompiled with canonical deterministic math on 2026-08-02. All recorded κs matched the existing pin, including the 1,346,836-byte container (`blake3:ef6a20f3…`), so no fixture bytes changed.

Local validation:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- graph-format no_std checks with and without alloc
- canonical release κ reproduction
- deterministic rebuild tests

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

