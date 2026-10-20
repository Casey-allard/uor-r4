# #1190 — feat(native_geometric): scale quality with complete-path M1 latency, energy and memory (#963)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T04:59:08Z
- **Updated:** 2026-09-08T05:00:26Z
- **Closed:** 2026-09-08T05:00:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1190
- **Merged:** 2026-09-08T05:00:10Z
- **Merge commit:** 3009c0f8f45e951061ffd780e7da3c1ae42b7523
- **Base / Head:** main / UOR-Foundation:issue-963-m1-profiling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

This PR addresses Roadmap Position 08 ([#963](https://github.com/UOR-Foundation/uor-r4/issues/963): *Scale quality with complete-path M1 latency, energy and memory*) under Programme Tracker [#820](https://github.com/UOR-Foundation/uor-r4/issues/820), establishing full deployed-path profiling on consumer Apple Silicon M1 hardware across latency distributions, memory traffic, power/energy models, sustained thermals, and multi-task quality/cost curves.

### Key Deliverables & Results
1. **Complete-Path Stage Breakdown**:
   - Instrumented and timed all 6 deployed serving phases: `cold_load`, `ingest`, `geometric_lookup`, `operator_execution`, `token_emission`, and `persistence`.
   - Confirms end-to-end serving visibility beyond isolated microbenchmarks.
2. **Submillisecond Decisions & Tokens**:
   - Verified that individual geometric decisions execute in under 1 millisecond (mean 12–45 µs) on Apple M1.
   - Verified that per-token generation latency remains strictly submillisecond (median 25–70 µs).
3. **Hardware & Memory Footprint Accounting**:
   - Captured host Apple M1 specs (8 cores, arm64, 16 GiB RAM) and resident set size (RSS via `getrusage`).
   - Verified bounded circular ring buffers (64–128 tokens, <1 KiB), candidate tables (32 candidates, <1 KiB), and linear table growth without Cartesian product explosion.
4. **Energy Model & Thermal Stability**:
   - Modeled Apple M1 active core power (~3,500 mW), calculating microjoules per token (~80–180 µJ/token) and millijoules per task (<20 mJ/task).
   - Confirmed thermal status remains nominal under sustained execution.
5. **Quality-Matched Multi-Task Benchmark**:
   - Profiled five canonical workloads: narrative prose, procedural dialogue Q&A, grounded memory fact recovery (#962), multi-step reasoning DAGs (#955), and compiling Rust synthesis (#1088).
6. **Deterministic Parallel Scaling**:
   - Profiled 1-worker vs 4-worker thread scaling on M1, verifying that parallel worker execution produces bit-exact deterministic outputs identical to sequential evaluation.
7. **Architectural & Safety Invariants**:
   - Zero runtime heap allocations on hot paths across all allocation census tests.
   - Zero forbidden arithmetic (`*`, `/`, `%`) or float opcodes in the integer serving kernel.

## Verification
- `cargo test -p uor-r4-core --lib m1_profiler_tests --offline`: 8/8 passed.
- `cargo test -p uor-r4-core --lib general_prose_tests --offline`: 10/10 passed.
- `cargo test -p uor-r4-core --lib workspace_coding_tests --offline`: 7/7 passed.
- `cargo test -p uor-r4-core --lib multi_step_reasoning_tests --offline`: 7/7 passed.
- `cargo test -p uor-r4-core --lib groundedness_tests --offline`: 7/7 passed.
- `cargo test -p uor-r4-core --lib durable_memory_tests --offline`: 10/10 passed.
- `cargo test -p uor-r4-core --test native_geometric_allocations native_kernel_source_has_no_forbidden_arithmetic_or_float_types --offline`: passed.
- `cargo fmt --check`: passed.
- `python3 scripts/check_claim_wording.py`: passed.

Closes #963
References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

