# #315 — Feed session manifold signatures into the graph bias lane

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T23:13:55Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-07-31T23:24:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/315
- **Merged:** 2026-07-31T23:24:21Z
- **Merge commit:** 0b8c252085e4ac88366d3ca823ddca82886501ad
- **Base / Head:** main / UOR-Foundation:issue-247-session-signature
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Implements the first measured milestone for #247.

## Summary
- add deterministic 288-bit session-signature quantization for persistent 512D manifold state
- add an order-sensitive token-history fallback for the direct chat client
- extend the graph runtime with separate context/session signature lanes
- keep context signatures on ROUT fallback; use session signatures only in the existing emission-affinity bias path
- wire the server's evolved manifold state into transformerless graph generation
- add same-eight-token/different-history A/B regression coverage and documentation

## Design boundary
This is intentionally bias-only. Session signatures do not enter ROUT fallback until held-out calibration proves that session-state prototypes are valid for that path. The deployed lane remains integer-only and reuses masked Hamming plus shift/add arithmetic; floating-point quantization is server-side.

## Validation
- cargo fmt --all --check
- git diff --check
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo check -p uor-r4-graph-format --no-default-features
- cargo check -p uor-r4-graph-format --no-default-features --features alloc
- cargo test --workspace --offline

The full suite passed, including 100 BDD scenarios. Relates to #247.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

