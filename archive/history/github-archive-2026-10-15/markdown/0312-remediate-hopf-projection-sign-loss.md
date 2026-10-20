# #312 — Remediate Hopf projection sign loss

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-31T21:47:00Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-07-31T21:57:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/312
- **Merged:** 2026-07-31T21:57:56Z
- **Merge commit:** c8eab90e0d1dd75f42edc33a859b365b83651d73
- **Base / Head:** main / UOR-Foundation:issue-306-hopf-projection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- replace the sign-restricted four-block L2 projection feeding Hopf addressing
- project each 128-dimensional block onto a deterministic signed Blake3-derived direction
- normalize the signed four-vector before sector assignment
- add a regression test and document the same-protocol D3 remeasurement

## Measurement

Using the same 596 held-out articles and 1,500 routing samples as #303:

- distinct sectors: 7/512 (1.4%) baseline -> 43/512 (8.4%)
- u_delta: [0.4783, 0.5029] -> [0.0001, 1.0000]
- u_alpha: [0.6169, 0.6260] -> [0.1406, 0.8031]
- chi_u: [0.4799, 0.5419] -> [0.0001, 0.4741]

This removes the sign-restricted reachable region and increases observed occupancy by 6.1x on the pinned protocol. It does not claim full-sector reachability or serving-quality improvement.

## Validation

- cargo fmt --check
- git diff --check
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo test --workspace --offline
- cargo check -p uor-r4-graph-format --no-default-features
- cargo check -p uor-r4-graph-format --no-default-features --features alloc
- ignored D3 occupancy harness: passed

Relates to #306; follows the baseline measurement in #303.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

