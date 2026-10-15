# #356 — feat: add certifier FMM candidate scorer (#290)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-02T23:35:33Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-03T01:06:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/356
- **Merged:** 2026-08-03T01:06:23Z
- **Merge commit:** cc8d3ba03eefc66858d4935de9988b3023ae858c
- **Base / Head:** main / UOR-Foundation:issue-290-fmm-accuracy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
- define reproducible §5.2 novel-context protocol for issue #290
- add deterministic certifier-side low-rank FMM scorer over graph prototypes/emission residuals
- add fixed-point feasibility scorer with caller-buffered selection and S7 parity measurement

## Results
- rank 20 / tolerance 1e-2, retained energy 0.6967
- 96 novel teacher-forced positions: top-1 1.04%, top-8 26.04%, teacher cross-entropy 10.588 bits/token
- fixed-point candidate matched float results and reduced estimated table storage 337,888 B -> 164,056 B
- compared with incumbent R4G1: top-1 unchanged; top-8 +20.83pp; teacher bits/token -3.367

## Scope
- certifier/research path only; not wired into deployed runtime or integer kernel
- fixed candidate currently uses widened integer products in certifier feasibility code; deployment still requires multiplication-free packed translation tables, allocation-free serving integration, and §5.4 cost comparison

## Validation
- cargo fmt --check
- cargo test -p uor-r4-graph-certify --offline
- cargo clippy -p uor-r4-graph-certify --all-targets --all-features --offline -- -D warnings
- cargo check -p uor-r4-graph-format --no-default-features
- cargo check -p uor-r4-graph-format --no-default-features --features alloc
- R4_FMM_POSITIONS=256 cargo test --test bdd --offline -- --name 'S7' --concurrency 1 -v
- workspace suite has a pre-existing R4G1 allocation census failure (2 allocations/594 bytes) outside this change

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

