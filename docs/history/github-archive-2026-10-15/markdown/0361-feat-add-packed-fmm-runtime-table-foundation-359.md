# #361 — feat: add packed FMM runtime table foundation (#359)

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-08-03T01:24:30Z
- **Updated:** 2026-10-09T14:17:14Z
- **Closed:** 2026-08-03T02:47:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/361
- **Merged:** 2026-08-03T02:47:24Z
- **Merge commit:** 79a4766facaafbf3deda7ee69664df1e9e3172d2
- **Base / Head:** main / UOR-Foundation:issue-359-fmm-runtime
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
- define the optional versioned FMM R4G1 section (FMM1)
- add zero-copy validated GraphView access and coefficient-row iterators
- fold fixed-point basis/factor products into compiler-side per-bit/per-token coefficients
- wire FMM section generation into the normal scored-artifact and cover-sweep exporters
- add a multiplication-free, allocation-free packed runtime evaluator with caller-owned scratch
- expose FMM footprint, rank, and candidate count in scored-artifact reports

## Design boundary

The compiler/certifier performs the floating-point low-rank decomposition and folds it into signed fixed-point coefficient rows. Serving selects a coefficient sign from each query bit and performs only table reads, saturating integer add/subtract, comparison, and bounded recent-token checks. The existing certifier-side low-rank scorer remains the compiler-side source of those coefficients.

FMM remains an optional artifact section. It does not replace masked-Hamming routing or the existing exact-context/graph scorer; the #290 novel-context accuracy and cost decision remains a separate measured gate.

## Validation

- cargo test --workspace --offline
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo fmt --check
- cargo check -p uor-r4-graph-format --no-default-features --offline
- cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline
- end-to-end R4G1 FMM artifact parsing and runtime parity
- deterministic tie-breaking and recent-token penalty coverage
- steady-state zero-allocation runtime census
- packed footprint and serving-work measurement: 2340 FMM bytes; 2439 artifact delta; 576 sign/add updates per query on the fixture
- GitHub CI: workspace nextest, BDD, docs, no-std, deterministic rebuild, canonical κ, audit, fuzz smoke, wasm parity, Gate C, and Kani all green

Related: #290, #356.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

