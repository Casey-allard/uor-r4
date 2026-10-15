# #190 — Deterministic parallel compilation for residuals/emissions and canonical certificate fragment assembly

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-25T04:23:16Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-07-25T05:13:45Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/190
- **Merged:** 2026-07-25T05:13:45Z
- **Merge commit:** 69272ba85a27ef15794522fe930504870034d908
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-parallel-emission-residuals
- **Reviews / inline review comments:** 1 / 4
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This change addresses parallelization in the residual/emission/quantization/certification compilation layer while preserving canonical artifact determinism. The core requirement is that thread scheduling can change throughput, but must not change section bytes, offsets, or certificate CIDs.

- **Residual quantization + delta compilation (deterministic under parallelism)**
  - Added threaded APIs in `residual.rs`:
    - `quantize_logprobs_with_threads(...)`
    - `compute_residual_deltas_with_threads(...)`
  - Kept existing sequential APIs as wrappers (`...with_threads(..., 1)`).
  - Enforced canonical residual processing via stable key ordering and token dedup before delta computation, eliminating order-sensitive accumulation from input interleavings.

- **Emission packing with canonical parallel assembly**
  - Added `pack_emission_tables_with_threads(...)` in `pack.rs`.
  - Parallelized per-region fragment preparation, then performed deterministic merge by canonical region index.
  - Preserved dedup semantics and stable range assignment independent of worker completion order.

- **Certificate claim-fragment compilation with canonical CID materialization**
  - Added `Certificate::new_from_claim_fragments(...)` in `certificate.rs`.
  - Claim fragments can be assembled in parallel, then canonicalized via sort + dedup before CID computation.
  - Ensures certificate payload bytes and resulting CIDs remain invariant across thread counts/interleavings.

- **Determinism-focused coverage additions**
  - Added focused tests for:
    - quantization thread-count invariance
    - residual-delta canonical/thread-invariant output
    - emission packing thread-count invariance
    - canonical claim-fragment assembly yielding identical certificate bytes/CIDs

```rust
let cert = Certificate::new_from_claim_fragments(
    source_cid,
    corpus_cid,
    graph_cid,
    metric_cid,
    op_cid,
    benchmark_cid,
    &claim_fragments,
    attestation,
    threads, // 1, 2, 4, ...
)?;
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #173

## Timeline (5 entries; chronological: conversation comments, reviews, review comments)

### review-comment 3649435674 — 2026-07-25T04:38:59Z — Copilot

_on crates/uor-r4-graph-compiler/src/residual.rs:27_

`quantize_logprobs` now panics on mismatched `tokens`/`logprobs` lengths via `expect(...)`. Previously it used `zip` and would deterministically truncate to the shorter slice; introducing a panic here makes this public API less robust for recoverable input errors.

### review 4778427679 — 2026-07-25T04:39:00Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR introduces thread-bounded, deterministic-by-construction parallelism in the graph compilation layer (residual quantization/deltas, emission packing) and in certificate claim-fragment assembly, with tests intended to ensure byte/CID invariance across thread counts.

**Changes:**
- Add `*_with_threads(...)` APIs for quantization, residual delta computation, and emission packing, keeping sequential wrappers.
- Canonicalize residual delta computation inputs (sort + dedup) and emission packing assembly order (sort fragments by region index) to preserve deterministic outputs under parallel execution.
- Add determinism-focused unit tests covering thread-count invariance for quantization, residual deltas, emission packing, and certificate claim-fragment assembly.

### Reviewed changes

Copilot reviewed 4 out of 4 changed files in this pull request and generated 4 comments.

| File | Description |
| ---- | ----------- |
| crates/uor-r4-graph-compiler/src/residual.rs | Adds threaded quantization + residual-delta APIs and determinism tests; introduces canonicalization + executor-based mapping. |
| crates/uor-r4-graph-compiler/src/pack.rs | Adds threaded emission packing with deterministic fragment merge and a thread-invariance test. |
| crates/uor-r4-graph-certify/src/certificate.rs | Adds canonical claim-fragment constructor with parallel cloning + canonical sort/dedup prior to CID materialization. |
| crates/uor-r4-graph-certify/tests/certificate_test.rs | Adds a test asserting canonical/thread-invariant claim-fragment assembly yields identical certificates. |




<details>
<summary>Comments suppressed due to low confidence (1)</summary>

**crates/uor-r4-graph-certify/src/certificate.rs:164**
* `canonical_sort_and_dedup_claims` currently uses `sort_by_key(Self::claim_key)` where `claim_key` allocates by cloning `name` and `slice_label` for every comparison/key extraction (and twice again during `dedup_by_key`). This can become a significant allocation hot spot when assembling many claim fragments; it’s avoidable by sorting/deduping via comparisons on borrowed fields.
```
    fn canonical_sort_and_dedup_claims(claims: &mut Vec<EmpiricalClaim>) {
        claims.sort_by_key(Self::claim_key);
        claims.dedup_by_key(|c| Self::claim_key(c));
    }
```
</details>



---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3649435692 — 2026-07-25T04:39:00Z — Copilot

_on crates/uor-r4-graph-compiler/src/residual.rs:101_

`map_with_threads` treats `threads == 0` as sequential because of the `threads <= 1` check. This prevents using the executor’s documented `RayonExecutor::new(0)` behavior (auto = `available_parallelism()`), and makes `threads=0` inconsistent across call sites.

### review-comment 3649435702 — 2026-07-25T04:39:00Z — Copilot

_on crates/uor-r4-graph-compiler/src/pack.rs:101_

`map_with_threads` treats `threads == 0` as sequential because of the `threads <= 1` check. This blocks the executor’s `RayonExecutor::new(0)` auto-thread behavior and makes `threads=0` unusable for “use all cores”.

### review-comment 3649435712 — 2026-07-25T04:39:00Z — Copilot

_on crates/uor-r4-graph-certify/src/certificate.rs:144_

`claims_from_fragments_with_threads` treats `threads == 0` as sequential due to `threads <= 1`. This prevents using the executor’s `RayonExecutor::new(0)` auto-thread behavior (available_parallelism), which is likely what callers expect when passing 0.

This issue also appears on line 161 of the same file.


---
_Comment/review ids in this file: 4778427679, 3649435674, 3649435692, 3649435702, 3649435712_

