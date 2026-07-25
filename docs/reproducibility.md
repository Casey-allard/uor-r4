# Normative Specification: Canonical Artifact Byte Equality Under Compiler Parallelism

**Version:** 0.1.0  
**Status:** Normative Specification  
**Issue:** #167  
**Parent:** #122  
**Depends On:** #165 (Deterministic Compiler Executor), #166 (Compiler Stage Ownership & Parallelization DAG)  
**Normative Invariant Source:** R4G1.md §7 (D2: "identical graph content + format version ⇒ identical bytes", Gate E), plan §4.1, AGENTS.md determinism invariant  

---

## 1. Primary Invariant Statement

> **Normative Invariant (verbatim)**:
> Parallel execution may change compilation time, but must not change the canonical graph artifact produced from the same pinned inputs, compiler version, configuration, and target-independent compilation mode.

All R4 compiler pipeline stages (whether `Parallel-Safe`, `Parallel with Deterministic Merge`, `Bounded Parallel`, or `Sequential Canonical Finalization`) must conform to this invariant.

---

## 2. Required Execution & Reduction Mechanisms

To enforce strict byte-identical canonical outputs across sequential and parallel multicore executions, the R4 graph compiler mandates the following mechanisms:

1. **Stable Input Ordering & Shard Numbering**:
   - Observations, traces, and samples are partitioned into deterministic shards based strictly on content-addressed IDs or stable positional indices.
   - Dynamic worker scheduling order or thread dispatch timing must never dictate shard numbering.

2. **Stable Candidate Ordering & Tie-Breaking**:
   - At every candidate selection point (e.g., region induction, XOR-polynomial candidate ranking, routing program selection), tie-breaking must follow a deterministic total order.
   - For candidate scores, ties are broken strictly by candidate ID / content hash (`ScoreQ` fixed-point score descending, then candidate ID ascending).

3. **Deterministic Reductions & Sorting**:
   - Parallel map-reduce operations must partition reductions by content-addressed key and perform deterministic ordered merges.
   - All canonical outputs (nodes, edges, regions, certificates) must undergo stable canonical sorting prior to binary encoding into R4G1 format.

4. **Floating-Point & Summation Policy**:
   - Order-sensitive floating-point parallel reductions are strictly forbidden for canonical output calculations.
   - Compiler stages computing canonical graph quantities must use fixed-order summation or integer/fixed-point accumulator types (`ScoreQ`, saturating integer arithmetic).
   - Compiler-internal float heuristics (e.g., loss diagnostics) may use floating-point operations provided canonical topology, IDs, offsets, and binary payload bytes do not depend on summation order.

5. **Random-Seed Policy**:
   - All pseudo-random processes (such as clustering initialization or randomized graph exploration) must consume pinned random seeds recorded in the compile report.
   - Seed generation and PRNG state transitions must be positionally deterministic and independent of thread scheduling or multicore worker counts.

6. **Thread-Count Invariance**:
   - The compiled graph topology, node/edge IDs, section offsets, payload bytes, content CIDs, and performance/verification certificates must be 100% bit-identical across any thread count $N \in \{1, 2, 4, 8, \dots\}$.

---

## 3. Forbidden Nondeterministic Patterns

The following implementation patterns are explicitly forbidden within the R4 graph compiler pipeline:

- **Forbidden Pattern 1**: Assigning canonical node or edge IDs during unordered parallel discovery.
- **Forbidden Pattern 2**: Using thread completion order or worker arrival sequence as semantic ordering.
- **Forbidden Pattern 3**: Mutating or appending to shared global graph data structures from parallel worker threads.
- **Forbidden Pattern 4**: Iterating over non-deterministic hash map key orders to emit canonical output structures.
- **Forbidden Pattern 5**: Order-sensitive floating-point parallel reductions feeding canonical graph attributes.
- **Forbidden Pattern 6**: First-finished-candidate-wins race conditions in candidate selection.

---

## 4. Verification Harness & Compliance

The compiler crate (`uor-r4-graph-compiler::reproducibility`) provides `ParallelReproducibilityHarness`. The harness executes:

$$\text{Harness}(\text{inputs}) \implies \text{Artifact}_{\text{seq}} \equiv \text{Artifact}_{\text{par}, N=1} \equiv \text{Artifact}_{\text{par}, N=2} \equiv \text{Artifact}_{\text{par}, N=4} \equiv \text{Artifact}_{\text{par}, N=\text{max}}$$

The harness asserts:
1. `ArtifactBytes(seq) == ArtifactBytes(par, N)` (100% byte equality)
2. `Digest(ArtifactBytes(seq)) == Digest(ArtifactBytes(par, N))` (BLAKE3 digest parity)

The proof model (`uor-r4-proof-model::structural_guarantees`) currently provides an executable-spec check via `verify_parallel_reproducibility_compliance`; end-to-end compiler artifact-byte parity remains validated by compiler-path tests.

---

## 5. Traceability & Claim Classification

- **Formal Claim Classification**: Canonicalization and deterministic output under parallelism is a **Guarantee** with **Structural** proof status (harness-backed).
- **CPU Portability**: Reproducibility is strictly CPU-native and GPU-free (zero CUDA, OpenCL, Metal, WebGPU, BLAS, or hardware vendor dependencies).
