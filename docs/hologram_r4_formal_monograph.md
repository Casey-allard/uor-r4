# Hologram / R⁴ Formal Monograph & Implementation Specification

> **Historical formal specification.** This monograph remains the scoped
> specification for the preserved R4G1 graph programme. It is not a proof or
> design of the current route-native attention, inference, correctness, or
> reasoning path. See the
> [Geometric Intelligence Programme](geometric_intelligence_programme.md).

**Version:** 1.0.0  
**Date:** 2026-07-24  
**Source Baseline:** Hologram Formal Analysis Direction PDF §§1–17; `docs/formal_vocabulary.md`; GitHub Issues #123–#137.

---

## Executive Summary & Design Invariants

The R⁴ Holographic Graph Compiler program generalizes transformerless table-native neural execution into a multiresolution, overlapping semantic graph architecture. This monograph consolidates the mathematical definitions, compiler pipeline stages, runtime execution contracts, and formal proof obligations governing the system.

---

## Section 1: Problem Statement and Non-Goals

### 1.1 Problem Statement
Modern autoregressive transformer language models require floating-point matrix multiplications ($O(N^2)$ to $O(N)$ memory/latency overhead) and fail to provide deterministic, allocation-free execution on edge devices. R⁴ compiles teacher model behaviors into multiplication-free, bitwise integer table-native graphs with witnessed runtime execution contracts.

### 1.2 Explicit Non-Goals & Disavowals
- **No Human-Level Reasoning Claim:** The system does not claim general artificial intelligence or human cognition.
- **No Exact Teacher Equivalence:** The graph compiler approximates teacher predictive distributions under bounded rate-distortion loss, not exact functional identity.
- **No Floating-Point Runtime Hot Path:** Floats and dynamic allocations are strictly confined to offline compilation and certification.

---

## Section 2: Semantic State Spaces and Holographic Projections

### 2.1 State Space $S$
**Definition 1 (Semantic State Space):** A semantic state $s \in S$ consists of a latent floating-point vector projection $v \in \mathbb{R}^D$, a Boolean signature bitmask $b \in \{0, 1\}^K$, confidence $c \in [0, 1]$, and structural attributes.

- **Implementation Module:** [`crates/uor-r4-graph-compiler/src/semantic_state.rs`](crates/uor-r4-graph-compiler/src/semantic_state.rs)  
- **Issue:** #124  
- **Claim Class:** Definition  

### 2.2 Holographic Encoding $H(x)$
**Definition 2 (Holographic Encoding & Partial Reconstruction):** Observation $x$ maps into an overlapping family of sub-projections $H(x) = \{h_0, h_1, \dots, h_k\}$ across progressive depth tiers. Partial reconstruction $R(H_k(x))$ exhibits monotonic divergence reduction as projection depth $k$ increases.

- **Implementation Module:** [`crates/uor-r4-graph-certify/src/holographic_encoding.rs`](crates/uor-r4-graph-certify/src/holographic_encoding.rs)  
- **Issue:** #126  
- **Claim Class:** Definition  

---

## Section 3: Graph Induction & Multi-Edge Algebras

### 3.1 Unified 9-Edge Algebra
**Definition 3 (Unified 9-Edge Algebra):** All node identities are shared across a single node space. Multi-edge connections are typed into 9 discriminants:
1. `Semantic` (0)
2. `Causal` (1, DAG enforced)
3. `Temporal` (2)
4. `Constraint` (3)
5. `GoalProgress` (4)
6. `Evidence` (5)
7. `Refinement` (6)
8. `Forward` (7)
9. `Reverse` (8)

- **Implementation Module:** [`crates/uor-r4-graph-format/src/records.rs`](crates/uor-r4-graph-format/src/records.rs)  
- **Issue:** #125  
- **Claim Class:** Definition  

---

## Section 4: Predictive Entropy & Information Bottleneck

### 4.1 Information-Bottleneck Objective
**Objective 1 (Information Bottleneck Rate-Distortion):** The offline compiler objective optimizes:
$$J_{\text{IB}} = I(Z; X) - \beta \cdot I(Z; Y_{\text{future}})$$
balancing surface detail compression $I(Z;X)$ against predictive future utility $I(Z; Y_{\text{future}})$.

- **Implementation Module:** [`crates/uor-r4-graph-compiler/src/semantic_state.rs`](crates/uor-r4-graph-compiler/src/semantic_state.rs)  
- **Issue:** #127  
- **Claim Class:** Objective  

---

## Section 5: Unsupervised Behavioral Probes & Anti-Memorization

### 5.1 Counterfactual Behavioral Probes
**Guarantee 1 (Counterfactual Probe Sensitivity):** `InterventionRecord` evaluates baseline vs perturbed contexts under 5 intervention types (`ContextAblation`, `SurfaceVariation`, `EntitySubstitution`, `TemporalChange`, `GoalChange`). Zero sensitivity under goal changes triggers `MemorizationDetected` rejection.

- **Implementation Module:** [`crates/uor-r4-graph-compiler/src/behavioral_probes.rs`](crates/uor-r4-graph-compiler/src/behavioral_probes.rs)  
- **Issue:** #128  
- **Claim Class:** Guarantee  

---

## Section 6: Future-State Optimization & Bounded Planning

### 6.1 Bounded Graph Planner
**Guarantee 2 (Bounded Graph Planning & Constraint Safety):** Finite action trajectory search $s_0 \xrightarrow{a_1} s_1 \dots \xrightarrow{a_k} s_k \in G$ enforcing forbidden region safety constraints ($s_i \notin C$) at every step within horizon $H_{\max}$.

- **Implementation Module:** [`crates/uor-r4-graph-compiler/src/future_state_planner.rs`](crates/uor-r4-graph-compiler/src/future_state_planner.rs)  
- **Issue:** #131  
- **Claim Class:** Guarantee  

---

## Section 7: Reference Intermediate Representation (IR)

### 7.1 5-Stage Reference Pipeline
**Definition 4 (5-Stage Reference IR Pipeline):** Standalone floating-point research surface (`ReferenceGraphIr`) operating via 5 explicit stages: Teacher Probing $\to$ Region Induction $\to$ Transition Discovery $\to$ Objective Optimization $\to$ Lowering Preparation.

- **Implementation Module:** [`crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs`](crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs)  
- **Issue:** #129  
- **Claim Class:** Definition  

---

## Section 8: Boolean / Integer Lowering & R4G1 Format

### 8.1 Boolean Region & Q8.8 ScoreQ Lowering
**Guarantee 3 (Boolean Predicate & Q8.8 Score Lowering):** Region predicates lower into 64-bit bitmasks (`XOR` + `POPCOUNT`), while transition scores lower into quantized Q8.8 (`i16`) ScoreQ tables with explicit saturation. Emits traceable `LoweringWitness`.

> **Disambiguation Note:** The compiler's Q8.8 `i16` fixed-point arithmetic score representation is used for internal region decision tables, distinct from the packed wire-format `uor_r4_graph_format::ScoreQ` (`i32` Q16.16 newtype) stored in R4G1 artifacts.

- **Implementation Module:** [`crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs`](crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs)  
- **Issue:** #130  
- **Claim Class:** Guarantee  

---

## Section 9: Structural Proofs & Proof Matrix

### 9.1 Executable Proof Verifiers
**Guarantee 4 (Structural Executable Proof Obligations):** Programmatic verification of output determinism, bounded memory/frontier resources, and constraint safety preservation ($s_i \notin C$) without panic-based assertions.

- **Implementation Module:** [`crates/uor-r4-proof-model/src/structural_guarantees.rs`](crates/uor-r4-proof-model/src/structural_guarantees.rs)  
- **Issue:** #132  
- **Claim Class:** Guarantee  

---

## Section 10: Decoupled Semantic Reasoning & Language Emission

### 10.1 Decoupled Architecture
**Guarantee 5 (Decoupled Semantic State Reasoning & Language Emission):** Pure state transition planning (`SemanticReasoningEngine`) is separated from token emission (`LanguageEmissionAdapter`), preventing token-looping degeneration.

- **Implementation Module:** [`crates/uor-r4-graph-compiler/src/semantic_emission_decoupling.rs`](crates/uor-r4-graph-compiler/src/semantic_emission_decoupling.rs)  
- **Issue:** #134  
- **Claim Class:** Guarantee  

---

## Section 11: Graph Invariant Ownership & Validation

### 11.1 Invariant Ownership Matrix
**Guarantee 6 (Explicit Graph Invariant Ownership):** Matrix assigning invariant validation responsibility across Format Loader, Compiler, Certifier, and Runtime.

- **Implementation Module:** [`crates/uor-r4-graph-format/src/invariant_ownership.rs`](crates/uor-r4-graph-format/src/invariant_ownership.rs)  
- **Issue:** #135  
- **Claim Class:** Guarantee  

---

## Section 12: Rate-Distortion Semantic Compression

### 12.1 Semantic Compression Function
**Objective 2 (Rate-Distortion Semantic Compression):** Formalizes compilation as lossy compression $C: \Theta \to G$ balancing rate terms $R(k)$ against distortion metrics $D(k)$.

- **Implementation Module:** [`crates/uor-r4-graph-compiler/src/rate_distortion_compression.rs`](crates/uor-r4-graph-compiler/src/rate_distortion_compression.rs)  
- **Issue:** #136  
- **Claim Class:** Objective  

---

## Section 13: PDF-to-Implementation Traceability

### 13.1 Traceability Audit Matrix
**Guarantee 7 (PDF-to-Implementation Traceability):** Living traceability matrix mapping all 15 formal direction PDF sections to code locations, evidence artifacts, and owners.

- **Implementation Module:** [`crates/uor-r4-proof-model/src/pdf_traceability.rs`](crates/uor-r4-proof-model/src/pdf_traceability.rs)  
- **Issue:** #137  
- **Claim Class:** Guarantee  

---

## Section 14: Complete Traceability & Proof Status Matrix

| Component | Issue | Module Path | Evidence Artifact | Claim Class | Proof Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Formal Vocabulary & Claims | #123 | `docs/formal_vocabulary.md` | `scripts/check_claim_wording.py` | Definition | Verified |
| Semantic State & Dynamics | #124 | `crates/uor-r4-graph-compiler/src/semantic_state.rs` | `features/suites/semantic_state_space.feature` | Definition | Verified |
| Multi-Edge Algebras | #125 | `crates/uor-r4-graph-format/src/records.rs` | `crates/uor-r4-graph-format/tests/stage2.rs` | Definition | Verified |
| Holographic Encoding | #126 | `crates/uor-r4-graph-certify/src/holographic_encoding.rs` | `crates/uor-r4-graph-certify/tests/holographic_encoding_test.rs` | Definition | Verified |
| Information Bottleneck | #127 | `crates/uor-r4-graph-compiler/src/semantic_state.rs` | `features/suites/semantic_state_space.feature` | Objective | ExecutableSpec |
| Behavioral Probes | #128 | `crates/uor-r4-graph-compiler/src/behavioral_probes.rs` | `features/suites/behavioral_probes.feature` | Guarantee | Verified |
| Reference IR Pipeline | #129 | `crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs` | `features/suites/reference_compiler_ir.feature` | Definition | Verified |
| Boolean & Q8.8 Lowering | #130 | `crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs` | `features/suites/lower_semantic_regions.feature` | Guarantee | Verified |
| Bounded State Planner | #131 | `crates/uor-r4-graph-compiler/src/future_state_planner.rs` | `features/suites/future_state_planner.feature` | Guarantee | Verified |
| Structural Proof Model | #132 | `crates/uor-r4-proof-model/src/structural_guarantees.rs` | `crates/uor-r4-proof-model/tests/proof_matrix_audit.rs` | Guarantee | Verified |
| Formal Monograph | #133 | `docs/hologram_r4_formal_monograph.md` | `features/suites/formal_monograph.feature` | Definition | ExecutableSpec |
| Decoupled Emission | #134 | `crates/uor-r4-graph-compiler/src/semantic_emission_decoupling.rs` | `features/suites/separate_semantic_emission.feature` | Guarantee | Verified |
| Invariant Ownership | #135 | `crates/uor-r4-graph-format/src/invariant_ownership.rs` | `features/suites/graph_invariant_ownership.feature` | Guarantee | Verified |
| Rate-Distortion Compression | #136 | `crates/uor-r4-graph-compiler/src/rate_distortion_compression.rs` | `features/suites/rate_distortion_compression.feature` | Objective | Verified |
| PDF Traceability Matrix | #137 | `crates/uor-r4-proof-model/src/pdf_traceability.rs` | `features/suites/pdf_traceability_matrix.feature` | Guarantee | Verified |

---

## Section 15: Issue Dependency Graph

```
                   [#123 Formal Vocabulary]
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
  [#124 Semantic State]               [#125 Edge Algebras]
            │                                     │
    ┌───────┴────────┐                    ┌───────┴────────┐
    ▼                ▼                    ▼                ▼
[#126 Hologram] [#127 Bottleneck] [#128 Probes]   [#129 Ref IR]
    │                │                    │                │
    └───────┬────────┘                    └───────┬────────┘
            ▼                                     ▼
    [#130 Lowering]                     [#131 State Planner]
            │                                     │
            └──────────────────┬──────────────────┘
                               ▼
                   [#132 Structural Proofs]
                               │
            ┌──────────────────┼──────────────────┐
            ▼                  ▼                  ▼
    [#133 Monograph]   [#134 Decoupled]   [#135 Invariants]
                               │                  │
                               └────────┬─────────┘
                                        ▼
                           [#136 Rate-Distortion]
                                        │
                                        ▼
                             [#137 Traceability]
```

---

## Section 16: Review Checklist vs Repos & Specifications

| Target Artifact | Conformance Criterion | Review Status |
| :--- | :--- | :--- |
| `docs/hologram_formal_analysis_direction.pdf` | Covers §§1–17 recommendations & equations | Pass |
| `docs/r4_graph_compiler_implementation_plan.md` | Aligns with compiler phase milestones | Pass |
| `docs/transformerless/R4G1.md` | Preserves R4G1 packed container RFC invariants | Pass |
| Threat Model (`docs/threat_model.md`) | Enforces anti-memorization & safety bounds | Pass |
| Proof Matrix (`crates/uor-r4-proof-model`) | Zero unverified entries in production baseline | Pass |

---

## Section 17: Known Negative Results & Disavowals

1. **Unconstrained Dense Float Execution:** Running dense floating-point matrix multiplication on resource-constrained edge microcontrollers fails deterministic, zero-allocation latency guarantees.
2. **Unbounded Graph Traversal:** Graph planning without strict horizon bounds ($H_{\max}$) and degree limits causes exponential frontier memory explosion.
3. **Greedy Token Emission Without Semantic Decoupling:** Emitting tokens directly from raw state representations without intermediate `SemanticStatus` reasoning induces repetitive token loops ("how this works like im 5...").

---

## Section 18: Legacy-Preserving Migration Path

The R⁴ Holographic Graph Compiler architecture coexists with the legacy transformerless engine. Packed R4G1 artifacts are read via allocation-free borrowed `GraphView` interfaces. The HTTP chat server resolves synthesis engines dynamically, defaulting to R4G1 while keeping legacy TLA/TLS accessible for regression validation.

---

## Section 19: Empirical Certification & Quality Gates

**Empirical Criterion 1 (Gate C Quality & Certification Thresholds):** All compiled graph artifacts must pass Gate C quality thresholds:
- **Bits-Per-Token:** $\le 3.50$
- **Top-1 Agreement:** $\ge 29.70\%$
- **Determinism:** Identical input bytes $\implies$ identical artifact bytes
- **No-Std Format:** `cargo check -p uor-r4-graph-format --no-default-features`
- **Claim Wording:** `python3 scripts/check_claim_wording.py` passes without unlinked machine-verified claims
