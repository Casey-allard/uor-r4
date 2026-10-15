# #1000 — feat(attention): freeze V4 connection-gauge covariance Phase I

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-29T14:32:04Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-08-29T14:33:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1000
- **Merged:** 2026-08-29T14:33:37Z
- **Merge commit:** b054197acb92e3dd23d88d81bd859379ea8fac67
- **Base / Head:** main / UOR-Foundation:issue-973-connection-gauge-covariance-v4
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and freezes the construction-only Phase I of `ConnectionGaugeCovarianceV4` for #973.

- H4-compatible, oriented alternative-tangent, and fixed-frame plain arms each fit 16/16 construction bindings.
- Current-only remains 8/16 on the balanced same-query construction set.
- All 120 exact H4 frames and 14,400 ordered frame pairs pass the direct tangent and full-connection audit.
- All 39 active representative Q/K/V/O coordinates pass central finite differences.
- Main-arm logits, weights, scores, decisions, gradients, and proposed updates are gauge-covariant within the frozen tolerances.
- Order, value, and deliberate gauge-mismatch controls are live; causal suffix reads remain zero; artifact replay is byte-identical.

Phase-I evidence root: `blake3:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e`.

## Freeze boundary

This PR contains no V4 validation population, labels, predictions, or scores. Those remain `NOT_RUN`. After this Phase-I commit is protected and merged, Phase II derives and publishes the target-free 24-case population from the protected merge SHA in a separate commit before any prediction.

## Decision-bearing verification

- `cargo test -p uor-r4-core --test connection_gauge_covariance_v4_973 --offline --no-fail-fast -- --nocapture` — 8/8 passed
- Existing V3 integration — 7/7 passed
- Existing direct-attention unit slice — 6/6 passed
- WASM library check — passed
- formatting, diff, and claim-wording gates — passed

A strict package clippy invocation also surfaced two pre-existing findings in untouched `uor-r4-model-source` and `uor-r4-graph-runtime` files; the modified attention file has no clippy findings.

References #973

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

