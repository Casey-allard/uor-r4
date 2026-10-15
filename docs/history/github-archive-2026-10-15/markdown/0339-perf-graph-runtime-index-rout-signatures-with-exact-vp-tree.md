# #339 — perf(graph-runtime): index ROUT signatures with exact VP-tree (#277)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-01T22:04:53Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-01T22:22:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/339
- **Merged:** 2026-08-01T22:22:42Z
- **Merge commit:** a4a394fa71d877a81b8dce1b67f6d4eb0a06d6a3
- **Base / Head:** main / UOR-Foundation:issue-277-vptree
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Implements the first serving-path phase of #277.

- Builds a deterministic VP-tree over ROUT prototypes at runtime load time.
- Uses exact masked-Hamming pruning only when all indexed nodes share one mask.
- Falls back to the existing linear scan for varying masks, preserving exact semantics.
- Returns the exact nearest node with canonical smallest-node tie-breaking.
- Preserves the first eight radius matches in canonical node-ID order.
- Keeps the prediction query allocation-free and multiplication-free.
- Leaves artifact bytes unchanged.
- Adds a benchmark harness that checks 32 deterministic queries for exact equality before timing.

## Verification

- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo check -p uor-r4-graph-format --no-default-features`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc`
- Exact-nearest, radius-filter, and tie-break unit tests.
- Existing R4G1 runtime tests and allocation census pass; steady-state prediction remains zero-allocation.
- Benchmark results on available 255-node fixtures: approximately 0.94–1.18x tree/scan speed, with exact result equality.

The available fixtures do not yet demonstrate a reliable throughput win, so #277 remains open for larger-node measurements and/or further tree-layout optimization rather than claiming completion.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

