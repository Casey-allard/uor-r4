# #348 — perf: skip VP-tree on small ROUT graphs (#277)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T20:27:25Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-02T20:41:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/348
- **Merged:** 2026-08-02T20:41:16Z
- **Merge commit:** 5f8ba69aedfcb1a86d5215d4b99d2c7deb9c8393
- **Base / Head:** main / UOR-Foundation:issue-277-vptree-cutoff
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

The exact VP-tree ROUT index is useful only after the graph is large enough to amortize its traversal overhead. On the current 363-node serving graph, repeated release benchmarks show the linear masked-Hamming scan is faster:

- linear: 2.50–2.58 µs/query
- VP-tree: 2.65–2.76 µs/query
- 32 deterministic queries × 1,000 iterations
- exact nearest node, distance, active set, and checksum all match

This PR keeps the exact VP-tree for graphs with at least 512 nodes and uses the linear path below that threshold, avoiding a measured small-graph regression. The cutoff is documented as a serving heuristic, not a universal crossover claim.

## Validation

- `cargo fmt --all -- --check`
- `cargo test -p uor-r4-graph-runtime --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- graph-format no_std checks with and without alloc
- `cargo bench -p uor-r4-graph-runtime --bench vp_tree --offline -- ... 1000`

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

