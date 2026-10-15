# #313 — Implement 16-window zeta QR spectral projection

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T22:10:22Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-07-31T22:20:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/313
- **Merged:** 2026-07-31T22:20:29Z
- **Merge commit:** 0c27ab446aead1bcd33369cdc5334789d7e6ae65
- **Base / Head:** main / UOR-Foundation:issue-246-zeta-projection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- port the 16-window sparse zeta design matrix from the prime-router design
- build deterministic reduced complex QR bases once per process in `uor-r4-core`
- make spectral window selection respond to query text with identical session state
- carry sparse active ranges and QR coefficient states through routing, indexing, and retrieval
- replace the hard-coded spectral eigenvalue vector with six-subwindow covariance eigenvalues
- record the #245 content-reconnection quality comparison and explicitly keep GloVe out because its source is unavailable/tombstoned

## Measurement

Focused harness:

`cargo test -p uor-r4-router --offline --test zeta_projection_quality -- --nocapture`

On the six-sentence/six-query fixture:

- #245 content-reconnection baseline: top-1 0.167, MRR 0.444
- sparse-QR routed path: top-1 1.000, MRR 1.000
- delta: +0.833 top-1, +0.556 MRR

The result is an empirical criterion for this fixture, not a general retrieval guarantee.

## Validation

- `cargo fmt --all --check`
- `git diff --check`
- `cargo check -p uor-r4-graph-format --no-default-features`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo test --workspace --offline`
- query-window responsiveness regression test
- deterministic QR/eigenvalue unit tests

Relates to #246 and builds on #245.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

