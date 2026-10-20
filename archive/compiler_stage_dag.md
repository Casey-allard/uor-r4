# Compiler Stage Ownership and Parallelization DAG (v0.1.0)

> **Preserved graph-compiler DAG.** This remains the stage map for the
> historical TLA/R4G1 compiler only. It is not the build sequence for current
> geometric intelligence; use the
> [Geometric Intelligence Programme](geometric_intelligence_programme.md).

**Normative Specification — Issue #166**  
*Source: `docs/r4_graph_compiler_implementation_plan.md` §§4.1, 5; `docs/transformerless/R4G1.md` §7*

---

## 1. Overview & Concurrency Taxonomy

The R⁴ Holographic Graph Compiler pipeline is structured as a directed acyclic graph (DAG) of processing stages. To guarantee **D2 Normative Reproducibility** (canonical artifact byte-equality regardless of thread count or execution scheduling), every compiler stage is assigned to exactly one of four concurrency classes:

1. **`Parallel-Safe`**: Embarrassingly parallel computation over independent content-addressed shards or inputs. Outputs preserve strict positional input order.
2. **`Parallel with Deterministic Merge`**: Parallel candidate discovery or extraction followed by an explicit, stable reduction (content-addressed sorting, deduplication, or ordered merge).
3. **`Bounded Parallel`**: Parallel execution constrained by memory budget limits (`#169`) or external teacher-probe rate limits (`#170`/`#174`).
4. **`Sequential Canonical Finalization`**: Strictly single-threaded execution spine protecting canonical form, node/edge ID assignment, artifact packing, root hashing, and signing.

---

## 2. Stage Inventory & Concurrency Matrix

| Stage ID | Stage Name | Concurrency Class | Module / Crate | Boundary Owner Issue | Determinism Justification |
| --- | --- | --- | --- | --- | --- |
| `S01` | Corpus Partitioning | `Parallel-Safe` | `observation_text` | `#170` | Deterministic BLAKE3 shard partitioning (`mod 5`). |
| `S02` | Teacher-Probe Request Prep | `Bounded Parallel` | `observation` | `#170` | Rate-limited batching over teacher oracle endpoints. |
| `S03` | Trace Normalization | `Parallel-Safe` | `observation` | `#170` | Positional mapping over observation vectors. |
| `S04` | Contextual Feature Extraction | `Parallel-Safe` | `observation` | `#170` | Pure function of token sequences and window bounds. |
| `S05` | Behavioral Fingerprinting | `Parallel-Safe` | `behavioral_probes` | `#170` | Content-addressed SHA-256/BLAKE3 hashing of activation traces. |
| `S06` | Paraphrase & Counterfactual Analysis | `Parallel with Deterministic Merge` | `perturbation` | `#170` | Parallel perturbation generation + stable index merge. |
| `S07` | Distance & Divergence Calculation | `Parallel-Safe` | `quantum_cover` | `#171` | Pure matrix distance/KL-divergence over state pairs. |
| `S08` | Nearest-Neighbor Search | `Parallel with Deterministic Merge` | `quantum_cover` | `#171` | Parallel top-k candidate search + deterministic tie-breaker. |
| `S09` | Recursive Clustering | `Parallel with Deterministic Merge` | `quantum_cover` | `#171` | Parallel density partitioning + ordered cluster merge. |
| `S10` | Region Proposal | `Parallel with Deterministic Merge` | `induction` | `#171` | Parallel cover proposal + stable sort by density/gain. |
| `S11` | Overlap Discovery | `Parallel with Deterministic Merge` | `induction` | `#171` | Parallel intersection check + canonical pair sorting. |
| `S12` | Parent/Child Discovery | `Sequential Canonical Finalization` | `induction` | `#171` | Single-threaded hierarchy tree/dag construction. |
| `S13` | Transition Discovery | `Parallel with Deterministic Merge` | `semantic_state` | `#171` | Parallel transition verification + ordered graph edge insertion. |
| `S14` | XOR-Polynomial Search | `Bounded Parallel` | `routing` | `#172` | Bounded multicore search over routing polynomials + min-degree selection. |
| `S15` | Routing-Program Search | `Bounded Parallel` | `routing` | `#172` | Bounded search for multiplication-free routing steps. |
| `S16` | Mask & Threshold Search | `Parallel-Safe` | `lower_semantic_regions` | `#172` | Pure bitwise signature mask optimization per region. |
| `S17` | Radius Calibration | `Parallel-Safe` | `lower_semantic_regions` | `#172` | Independent fixed-point radius search per signature. |
| `S18` | Collision Analysis | `Bounded Parallel` | `lower_semantic_regions` | `#172` | Memory-bounded pairwise collision verification. |
| `S19` | Shortlist-Recall Evaluation | `Parallel with Deterministic Merge` | `shortlist_evaluator` | `#173` | Parallel candidate scoring + deterministic tie-breaking. |
| `S20` | Region Emission Compilation | `Parallel with Deterministic Merge` | `pack` | `#173` | Parallel table construction + ordered deduplication. |
| `S21` | Residual Compilation | `Parallel with Deterministic Merge` | `residual` | `#173` | Parallel residual calculation + stable accumulator order. |
| `S22` | Quantization Analysis | `Bounded Parallel` | `rate_distortion_compression` | `#173` | Rate-distortion evaluation bounded by memory budget. |
| `S23` | Empirical Certification | `Bounded Parallel` | `performance_certificate` | `#173` | Witness counter evaluation over test fixtures. |
| `S24` | Graph-Fragment Construction | `Sequential Canonical Finalization` | `graph` | `#173` | Single-threaded assembly of immutable graph fragments. |
| `S25` | Artifact Section Construction | `Sequential Canonical Finalization` | `pack` | `#173` | Single-threaded layout of R4G1 header, METADATA, & TABLES. |
| `S26` | Canonical Sorting & ID Assignment | `Sequential Canonical Finalization` | `pack` | `#167` | Stable sorting by CID; sequential node/edge ID assignment. |
| `S27` | Final Offset Calculation & Packing | `Sequential Canonical Finalization` | `pack` | `#167` | Single-threaded byte offset alignment and table packing. |
| `S28` | Root Hashing & Signing | `Sequential Canonical Finalization` | `pack` | `#167` | Final BLAKE3 root hash calculation over packed artifact bytes. |

---

## 3. Parallel Boundary Ownership Edges

Every transition across a parallel boundary is owned by a dedicated issue in the Phase 4 compiler parallelization plan:

1. **`#170` (Observation & Shards)**: Owns stages `S01` through `S06`.
2. **`#171` (Clustering & Region Induction)**: Owns stages `S07` through `S13`.
3. **`#172` (XOR-Polynomial & Routing Synthesis)**: Owns stages `S14` through `S18`.
4. **`#173` (Emission, Quantization, & Packing)**: Owns stages `S19` through `S25`.
5. **`#167` (Normative Reproducibility)**: Owns stages `S26` through `S28` (the canonical finalization spine).

---

## 4. Sequential Canonical Finalization Spine Protection

Stages `S12` and `S24`–`S28` constitute the **Sequential Canonical Finalization Spine**. 
- No parallel scheduling or worker execution order is permitted within these stages.
- All candidate nodes, edges, region signatures, and emission tables must be reduced into a single, content-addressed ordered sequence prior to entering stage `S26`.
- The final artifact byte representation output by stage `S28` is strictly deterministic ($A_{\text{bytes}}(\text{input}) \equiv A_{\text{bytes}}'(\text{input})$ across all thread configurations).
