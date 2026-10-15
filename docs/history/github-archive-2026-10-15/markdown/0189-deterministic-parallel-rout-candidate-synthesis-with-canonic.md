# #189 — Deterministic parallel ROUT candidate synthesis with canonical reduction

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-25T04:23:07Z
- **Updated:** 2026-10-09T14:17:00Z
- **Closed:** 2026-07-25T05:13:45Z
- **Assignees:** afflom, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/189
- **Merged:** 2026-07-25T05:13:45Z
- **Merge commit:** 0dc1e1d4d8f9c98e3d384e50810257d34d5c034b
- **Base / Head:** main / UOR-Foundation:copilot/uor-foundation-parallel-xor-synthesis
- **Reviews / inline review comments:** 4 / 6
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR implements CPU-parallel routing-program candidate search with deterministic winner selection, so ROUT output is invariant to evaluation order and thread count. It formalizes canonical candidate ordering, total-order tie-breaking, and bounded batch evaluation for routing synthesis.

- **Deterministic candidate model + total order**
  - Introduces canonical routing candidate keys (`tap_set`, polynomial id, mask word/bits, threshold, radius, DAG id).
  - Enforces stable global sort and dedup by key before evaluation.
  - Defines deterministic comparison over `(quality desc, collision/cost asc, canonical key asc)`.

- **Order-independent reduction**
  - Adds canonical `reduce_best` logic that selects by comparator, never by completion order.
  - Makes winner selection invariant under candidate evaluation permutation.

- **Bounded CPU-parallel evaluation**
  - Uses existing compiler executor abstraction (`SequentialExecutor` / `RayonExecutor`) for candidate evaluation.
  - Batches candidate work using `CompilerMemoryBudget`-derived limits (bounded in-flight behavior, no unbounded queueing).
  - Keeps synthesis explicitly CPU-only (no GPU/tensor runtime path).

- **ROUT synthesis integration + scaling surface**
  - Replaces stub routing synthesis with deterministic candidate derivation/evaluation/selection.
  - Adds a lightweight `benchmark_candidate_scaling` surface for candidates/sec thread sweeps.

```rust
// Deterministic total order:
// 1) quality_gain desc
// 2) collision_penalty asc
// 3) runtime/artifact/cache asc
// 4) canonical key asc
fn compare_evaluations(a: &CandidateEvaluation, b: &CandidateEvaluation) -> Ordering {
    b.score.quality_gain.cmp(&a.score.quality_gain)
        .then_with(|| a.score.collision_penalty.cmp(&b.score.collision_penalty))
        .then_with(|| a.score.runtime_cost.cmp(&b.score.runtime_cost))
        .then_with(|| a.score.artifact_size_cost.cmp(&b.score.artifact_size_cost))
        .then_with(|| a.score.cache_cost.cmp(&b.score.cache_cost))
        .then_with(|| a.key.cmp(&b.key))
}
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #172

## Timeline (10 entries; chronological: conversation comments, reviews, review comments)

### review 4778419190 — 2026-07-25T04:35:11Z — copilot-pull-request-reviewer[bot] — COMMENTED

## Pull request overview

This PR replaces the prior ROUT synthesis stub with a deterministic, CPU-parallel candidate generation/evaluation/selection pipeline in `uor-r4-graph-compiler`, aiming to make routing-program selection invariant to evaluation order and thread count.

**Changes:**
- Adds canonical candidate keys + a documented total-order comparator for deterministic winner selection.
- Introduces bounded batched evaluation using the compiler executor abstraction (sequential vs Rayon).
- Adds unit tests for ordering/tie-break/reduction determinism and a lightweight candidate-scaling benchmark surface.







---

💡 <a href="/UOR-Foundation/uor-r4/new/main?filename=.github/instructions/*.instructions.md" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Add Copilot custom instructions</a> for smarter, more guided reviews. <a href="https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot" class="Link--inTextBlock" target="_blank" rel="noopener noreferrer">Learn how to get started</a>.

### review-comment 3649430028 — 2026-07-25T04:35:11Z — Copilot

_on crates/uor-r4-graph-compiler/src/routing.rs:173_

`evaluate_candidate` allocates and mutates a `BTreeSet` per candidate to detect `prev` collisions. This is O(n log n) with many small allocations and can dominate runtime/memory when observations are large, undermining the intended bounded/batched parallel evaluation.

### review-comment 3649430036 — 2026-07-25T04:35:11Z — Copilot

_on crates/uor-r4-graph-compiler/src/routing.rs:335_

`synthesize_routing_program` silently falls back to `OP_HALT` on any error from `choose_best_for_threads`. That can mask deterministic synthesis failures (budget derivation/executor creation/evaluation) and produce artifacts with routing effectively disabled, making failures harder to detect and debug.

### review-comment 3649430044 — 2026-07-25T04:35:11Z — Copilot

_on crates/uor-r4-graph-compiler/src/routing.rs:371_

`benchmark_candidate_scaling` computes throughput via floating-point seconds and truncating cast to `u64`, which can produce `0` on slow/loaded systems and make `benchmark_rows_report_positive_throughput` flaky. Integer nanosecond arithmetic avoids FP and allows clamping to a minimum positive throughput when `evaluated > 0`.

### review 4778466247 — 2026-07-25T04:53:24Z — copilot-swe-agent[bot] — COMMENTED

### review 4778466248 — 2026-07-25T04:53:24Z — copilot-swe-agent[bot] — COMMENTED

### review 4778466251 — 2026-07-25T04:53:24Z — copilot-swe-agent[bot] — COMMENTED

### review-comment 3649466870 — 2026-07-25T04:53:24Z — Copilot

_on crates/uor-r4-graph-compiler/src/routing.rs:173 · in reply to review-comment 3649430028_

Implemented. I replaced per-candidate `BTreeSet` insertion with fixed-size bitmap tracking in `evaluate_candidate`, eliminating per-candidate heap churn while keeping deterministic behavior.

### review-comment 3649466871 — 2026-07-25T04:53:24Z — Copilot

_on crates/uor-r4-graph-compiler/src/routing.rs:335 · in reply to review-comment 3649430036_

Implemented. `synthesize_routing_program` now returns `Result<Vec<u8>, String>` and `emit_r4g1` propagates failures instead of silently falling back to `OP_HALT`.

### review-comment 3649466872 — 2026-07-25T04:53:24Z — Copilot

_on crates/uor-r4-graph-compiler/src/routing.rs:371 · in reply to review-comment 3649430044_

Implemented. `benchmark_candidate_scaling` now uses integer nanosecond arithmetic and clamps to a minimum positive throughput when `evaluated > 0`, removing the float/truncation flake risk.


---
_Comment/review ids in this file: 4778419190, 4778466247, 4778466248, 4778466251, 3649430028, 3649430036, 3649430044, 3649466870, 3649466871, 3649466872_

