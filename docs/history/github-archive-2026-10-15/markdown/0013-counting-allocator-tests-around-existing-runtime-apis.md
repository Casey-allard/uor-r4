# #13 — Counting-allocator tests around existing Runtime APIs

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-21T21:24:05Z
- **Updated:** 2026-07-21T21:37:24Z
- **Closed:** 2026-07-21T21:37:24Z
- **Labels:** graph-compiler
- **Milestone:** Phase 0: Baseline and contracts
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/13
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Allocation census harness for transformerless Runtime (parse, predict, predict_witness, generate_greedy_into at crates/uor-r4-core/src/transformerless/runtime.rs:497). Harness lands in crates/uor-r4-core/tests/allocation_census.rs; feeds BASELINE.md. Plan §9.3

## Comments (1)

### comment 5039297444 — 2026-07-21T21:37:23Z — auser

Done in commit 7cf402a: crates/uor-r4-core/tests/allocation_census.rs (std-only counting allocator, no source changes, full suite green).

Fresh census on real .uor-models/compiled/smollm2-135m-instruct artifacts (deterministic across runs):
- Hot path (assign_window + predict + predict_witness + generate_greedy_into, 32 tokens, preallocated buffers): 0 allocations, 0 bytes — asserted
- Runtime::new: 0 allocations
- Op census: 144,498 avg integer ops/token (adds 48,526 / xors 36,864 / shifts 11,662 / compares 1,334 / table-reads 46,112)
- Parse side (report-only): artifacts 18 allocs / 1.71 MB; store 57,092 allocs / 4.69 MB for a 494 KB TLS1 container (~9.5x, per-key Vec + BTreeMap nodes) — the Phase 1 packed R4G1 layout target
- Write path add_evidence x64: 563 allocs (known; becomes patch epochs in Phase 9)

Numbers folded into docs/transformerless/BASELINE.md section 3.3.


---
_Comment/review ids in this file: 5039297444_

