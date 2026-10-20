# Normative Specification: Parallel Observation, Trace, and Evaluation Processing Over Deterministic Shards

> **Preserved teacher-observation contract.** This specification belongs to
> the historical source-model/R4G1 compiler lane. It is not a required stage of
> route-native serving and does not authorize corpus-scale work for the current
> programme. See the [documentation map](README.md).

**Version:** 0.1.0  
**Status:** Normative Specification  
**Issue:** #170  
**Parent:** #122  
**Depends On:** #165 (Executor), #166 (Stage DAG), #167 (Reproducibility), #168 (Jobs Config), #169 (Memory Budget)  
**Normative Invariant Source:** Plan §4.1, #59 (Observation Pipeline v2)  

---

## 1. Content-Addressed Shard Partitioning

Observations, trace documents, and evaluation units are embarrassingly parallel work items. Work items are partitioned into coarse deterministic shards (`par_chunks`-style batching):

1. **Shard ID Determination**:
   - Each shard $k$ carries a deterministic 64-bit ID $S_k$ computed from content hashes or deterministic positional sequence indices:
     $$S_k = \text{fnv1a\_64}(\text{shard\_content\_bytes})$$
   - Shard IDs are strictly independent of worker execution order or thread scheduling.

2. **Work-Unit Coarseness**:
   - Shard sizes obey Issue #169's memory budget constraints. Token-by-token task spawning is strictly prohibited.

---

## 2. Ordered Reduction Semantics

1. **Ordered Reduction Invariant**:
   - Intermediate outputs generated across parallel worker threads are aggregated into final stage results by sorting partial results in ascending order of shard ID $S_k$:
     $$\text{Result} = \text{Reduce}_{k \in \text{sorted\_shard\_ids}}(\text{ShardResult}_k)$$

2. **Thread-Count Invariance**:
   - Executing shard processing across $T=1, 2, 4, N$ worker threads yields 100% bit-identical byte digests and metrics (verified via #167's harness).

3. **No Non-Associative FP Reductions**:
   - Evaluation metrics that feed canonical output employ ordered reductions with fixed-point or integer accumulators to prevent floating-point sum non-associativity drift.

---

## 3. Teacher-Backend Boundary & CPU Portability

1. **Three-Way Architecture**:
   - **CPU Orchestration**: CPU-native shard partitioning, prompt/request preparation, and response analysis.
   - **Batching Interface**: Structured request/response batching interface (`TeacherProbeBatch`).
   - **External Teacher Backend**: External teacher runtime (local runner or remote API).

2. **CPU-Only Correctness Guarantee**:
   - Compiler correctness and supported operation never depend on a GPU backend. Zero CUDA, Metal, OpenCL, Vulkan compute, or accelerator dependencies exist in the compiler pipeline.

---

## 4. Verification Harness & Compliance

The compiler crate (`uor-r4-graph-compiler::observation_shards`) provides `ParallelShardEngine` and `ordered_shard_reduce`.

The proof model (`uor-r4-proof-model::structural_guarantees`) verifies this guarantee via `verify_parallel_observation_shards_compliance`.
