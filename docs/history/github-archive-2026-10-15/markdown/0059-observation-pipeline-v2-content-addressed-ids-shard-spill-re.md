# #59 — Observation pipeline v2: content-addressed IDs, shard spill/resume, teacher trace surface

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-22T18:29:14Z
- **Updated:** 2026-07-22T18:58:38Z
- **Closed:** 2026-07-22T18:58:38Z
- **Labels:** graph-compiler
- **Milestone:** Phase 2: Overlapping multiresolution regions
- **Assignees:** auser
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/59
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Phase 2 foundation (plan §5 Phase 2, §4.1 rules): content-addressed sample IDs (blake3 over context tokens), deterministic shard partitioning/spill/resume with manifest, observe CLI writing v3 48-byte anchor records into shards, and the optional TeacherOracle trace surface (final hidden state + canonical top-k) that cover induction consumes. Hard rules: existing corpus formats and κ-reproduction stay byte-identical; T-invariant merges in shard-ID order. In flight (assigned), first slice lands via subagent build + gate verification.

## Comments (1)

### comment 5050230083 — 2026-07-22T18:58:38Z — auser

Landed in 2b1d060 (pushed to main).

- observe.rs: sample_id (blake3 over LE window tokens — content-addressed, iteration-independent), shard_of (first-N-bits partition), ObservationShardWriter + manifest.json (per-shard blake3 kappa), state.bin generator checkpoint, crash-safe resume (completed shards skipped, incomplete resumed at deterministic prefix), merge_shards byte-identical regardless of shard completion order (T-invariance proven in tests).
- TeacherOracle trace surface: default hidden_state() -> None and top_k() -> 0; real implementations for LlamaOracle + HuggingFaceLlamaOracle from retained State.x/logits with canonical ordering (total_cmp prob desc, lower token id on ties).
- CLI: r4 transformerless observe [--source DIR|--checkpoint BIN] [--seconds N] [--target N] [--shards N] [--out DIR].
- Bonus fix: latent debug-build u32 overflow panic in the legacy top-3 weight rounding (compiler.rs) — value-identical to release behavior, kappa gate confirms.
- Verified locally: 43 suites, clippy -D warnings, fmt, kappa-reproduction (2 passed) all green. Unblocks #60 (cover induction): the per-shard write/finish API is the partition-before-parallelism contract of plan §4.1.


---
_Comment/review ids in this file: 5050230083_

