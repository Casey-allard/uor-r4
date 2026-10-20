# UOR-R4 Quantum Geometric Transformerless LLM — Master Architecture & Implementation Plan
*Based on N. Furey (2026), "Standard Model Symmetries and the Nested Embeddings of $\mathbb{R} \subset \mathbb{C} \subset \mathbb{H} \subset \mathbb{O}$" (arXiv:2607.18450v1)*

---

> **Status (updated 2026-08-26): historical/dormant research plan.** This file
> does not determine current implementation order. The active architecture is
> the [Geometric Intelligence Programme](geometric_intelligence_programme.md).
> Reuse a mechanism here only when a live #820 programme stage names it, makes
> it reachable in the token path, and supplies a non-degenerate control.
> Statements below that say “outperforms,” “O(1),” “guaranteeing,” or describe
> a future engine are historical hypotheses/proposals, not established current
> product claims. Their dated negative and dormant results remain authoritative
> at the scopes recorded in the implementation-status table.

## Implementation Status (dated deferral record — issue #249)

*Added 2026-07-28. This plan reads as committed architecture; this section
records what is actually wired into the inference path as of `main` today,
so readers do not mistake dormant capability for load-bearing structure.
Nothing listed here is removed — these modules are planned capability,
kept deliberately (decision recorded on issue #249).*

| Module | State on `main` | Deferral / activation condition |
|---|---|---|
| `cd_space.rs` (Cayley-Dickson nested embedding, Phases 1-2) | Implemented, unit-tested, **no callers in the compile/runtime/scoring path** | Deferred until a phase gate consumes `V`-substrate states; revisit after #243 settles the address space |
| `endomorphism.rs` (`End(V) ≅ Cl(0,8)` store, Phase 2) | Implemented, unit-tested, callers only in `cd_space` tests | Same gate as `cd_space` |
| `lie_jordan.rs` (Lie/Jordan split kernels, Phase 3) | Implemented, self-referential only | Same gate; hot-path adoption additionally requires a P-4 kernel-scan review |
| `bott_fock.rs` (O(1) context fold, Phase 4) | Implemented, unit-tested, **unused by the runtime** — the shipped context is the hard 8-token window | **Measured 2026-08-07 (#424, `docs/context_horizon_424.md`); no longer the priority candidate.** The flagged A/B is not reachable: the shipped decay `cell - (cell >> 2)` bounds influence to 63 tokens with ~90% of the mass inside the existing window, and a lossless replay of the same inductive bias caps the fold at +0.16pp of top-1 — one standard error — against a +1.02pp long-range ceiling for any carrier on this corpus. Mechanism sound, constant wrong: retune the decay to `>> 7` before reconsidering activation. |
| `quantum_cover.rs` (Von Neumann density clustering, Phase 5) | Implemented in `uor-r4-graph-compiler`, zero callers outside its own tests; present in the stage DAG registry but not the `compile()` critical path | Deferred until Phase-5 cover induction is scheduled |

**Naming disambiguation** (recorded here because it has already caused
confusion): the live `uor-r4-graph-runtime/src/cayley_dickson.rs` used by
`syntactic_morphism_score` is a golden-ratio hash tie-breaker and is *not*
the Cayley-Dickson algebra of this plan (`uor-r4-core/.../cd_space.rs`,
dormant). When this plan's phases activate, one of the two should be
renamed. *(Addendum 2026-08-05: `syntactic_morphism_score` was measured dead
and removed in PR #410 — #400 A/B, 0/1,998 executions; the graph-runtime
`cayley_dickson.rs` module remains, now without serving callers.)*

---

## Executive Summary & Strategic Vision

The goal of the **UOR-R4 project** is to build a completely local, CPU-first, multiplication-free, transformerless Large Language Model (LLM) that outperforms traditional transformer architectures. 

Traditional transformer models suffer from two fundamental bottlenecks:
1. **Quadratic $O(N^2)$ Compute & Memory Scaling**: Attention matrices ($K, Q, V$) require storing and multiplying dense floating-point matrices over all context tokens.
2. **Lack of Algebraic Structure**: Token representations live in arbitrary vector spaces without non-associative division algebraic symmetries or quantum-state compression principles.

By synthesizing the mathematical physics of **N. Furey (2026)** with the **UOR-R4 Holographic Graph Compiler**, we introduce the **R4 Prime Geometric Quantum Transformerless Engine**:
- **Cayley-Dickson Nested Embedding Substrate**: Token and context states are embedded in the 16D real / 8D complex vector space $V = e_i \mathbb{O} \oplus e_5 \mathbb{H} \oplus e_6 \mathbb{C} \oplus e_7 \mathbb{R} \oplus \mathbb{R}$ with nested inclusions $\mathbb{R} \subset \mathbb{C} \subset \mathbb{H} \subset \mathbb{O} \subset V$.
- **Endomorphic Multiplication Algebra & $Cl(0,8)$ Store**: State transitions are represented as elements of $\text{End}_\mathbb{R}(V) \cong Cl(0,8) \cong M_{16}(\mathbb{R})$ with 256 degrees of freedom.
- **Bott Periodic Fock Space Context Abstraction**: Utilizing 8-periodicity $Cl(s,t) \cong Cl(s_0,t_0) \otimes M_{16}(\mathbb{R})^{\otimes n}$, multi-token context histories are folded into fixed 256-dimensional tensor states. This eliminates $O(N^2)$ KV-caches, achieving **$O(1)$ space and $O(1)$ time context abstraction** (the *quantum context advantage*).
- **Lie-Jordan Splitting & Allocation-Free Hot Path**: Context operations split into anti-Hermitian Lie symmetries $\mathcal{L}_\Delta = \{u \mid u^\dagger = -u\}$ (transition operations $[u,v]$) and Hermitian Jordan observables $\mathcal{H}_\Delta = \{h \mid h^\dagger = h\}$ (token state probabilities $\{h,s\}$). Hot-path execution uses the universal product $m(a,b) = ab + ba^\dagger$ over bitwise integer tables without floating-point arithmetic or heap allocations.
- **Von Neumann Maximum-Entropy Density Bounds**: Region cover induction and residual emission tables are governed by equipartition charge density operators $\sum \frac{1}{n} I_{n \times n}$, guaranteeing information-theoretic efficiency and noise elimination.

---

## Mathematical Architecture (Furey 2026 Mapping)

```
========================================================================================
                         BOTT PERIODIC FOCK SPACE CONTEXT LAYER
   F_s0,t0 = Cl(s0,t0)|0⟩ ⊕ [Cl(s0,t0) ⊗ M16(R)]|p1⟩ ⊕ [Cl(s0,t0) ⊗ M16(R)⊗2]|p1,p2⟩ ...
                                 ↓ (8-Periodicity Reduction)
                        Fixed-Size 256D State Matrix M16(R)
========================================================================================
                                         │
                                         ▼
========================================================================================
                  CAYLEY-DICKSON NESTED EMBEDDING SUBSTRATE (V)
                   V = e_i O ⊕ e_5 H ⊕ e_6 C ⊕ e_7 R ⊕ R  (16D / 8C)
                             R ⊂ C ⊂ H ⊂ O ⊂ V
========================================================================================
                                         │
                                         ▼
========================================================================================
                  ENDOMORPHIC MULTIPLICATION ALGEBRA End_R(V) ≅ Cl(0,8)
                               256 Degrees of Freedom
                                         │
                       ┌─────────────────┴─────────────────┐
                       ▼                                   ▼
             Centralizer Δ_SM                    Electroweak Break Δ_LE
    C ⊕ M3(C) ⊕ M2(C) ⊕ C ⊕ R           C ⊕ M3(C) ⊕ C ⊕ C ⊕ C ⊕ R
========================================================================================
                                         │
                                         ▼
========================================================================================
                         LIE-JORDAN SPLITTING m(a,b) = ab + ba†
             Symmetries L_Δ (Lie [u,v])  │  Observables H_Δ (Jordan {h,s})
========================================================================================
                                         │
                                         ▼
========================================================================================
                  VON NEUMANN MAXIMUM-ENTROPY EQUIPARTITION COVER
                     Density Operator ρ = Σ (1/n) I_{n × n}
                       ScoreQ Quantized Emission Residuals
========================================================================================
```

---

## Phased Implementation Roadmap

### Phase 1: Cayley-Dickson Vector Space & Division Algebraic State Substrate
**Goal**: Replace raw 1D signatures in `uor-r4-core` with 16D real / 8D complex Cayley-Dickson state vectors $V = e_i \mathbb{O} \oplus e_5 \mathbb{H} \oplus e_6 \mathbb{C} \oplus e_7 \mathbb{R} \oplus \mathbb{R}$.

- **Module**: `crates/uor-r4-core/src/transformerless/cd_space.rs`
- **Data Structures**:
  ```rust
  pub struct CayleyDicksonVector {
      pub octonion: [f32; 8],  // e_i O
      pub quaternion: [f32; 4],// e_5 H
      pub complex: [f32; 2],   // e_6 C
      pub real: f32,           // e_7 R
      pub scalar: f32,         // R
  }
  ```
- **Nested Inclusions**: Implement $\mathbb{R} \subset \mathbb{C} \subset \mathbb{H} \subset \mathbb{O} \subset V$ projection and embedding functions.
- **Verification**: Unit tests in `crates/uor-r4-core/tests/cd_space.rs` asserting exact multiplication tables ($e_i e_j = -\delta_{ij} + f_{ijk} e_k$) and norm preservation.

---

### Phase 2: Endomorphic Multiplication Algebra & $Cl(0,8)$ Matrix Operator Store
**Goal**: Construct $\text{End}_\mathbb{R}(V) \cong Cl(0,8) \cong M_{16}(\mathbb{R})$ to represent state transitions and region operations as 256-dimensional real matrix endomorphisms.

- **Module**: `crates/uor-r4-core/src/transformerless/endomorphism.rs`
- **Data Structures**:
  ```rust
  pub struct EndomorphismAlgebra {
      pub left_octonion: [f32; 64],  // L_O ≅ Cl(0,6)
      pub left_quaternion: [f32; 16],// L_H ≅ Cl(0,2)
      pub full_quaternion: [f32; 16],// B_H ≅ Cl(3,1)
      pub matrix: [f32; 256],        // End_R(V) ≅ M16(R)
  }
  ```
- **Operator Generators**: Implement $L_x(y) = xy$ and $R_x(y) = yx$ for octonions, quaternions, and sedenion Cayley-Dickson elements.
- **Verification**: Machine-checked algebraic isomorphisms asserting $B_{\mathbb{H}} \cong L_{\mathbb{H}} \otimes_{\mathbb{R}} R_{\mathbb{H}}$ and $L_{\mathbb{O}} \cong Cl(0,6)$.

---

### Phase 3: Lie-Jordan Splitting & Allocation-Free Hot Path Engine
**Goal**: Split endomorphism operators into Lie symmetries $\mathcal{L}_\Delta$ (anti-Hermitian) and Jordan observables $\mathcal{H}_\Delta$ (Hermitian), executing inference hot paths via universal product $m(a,b) = ab + ba^\dagger$ over packed integer tables.

- **Module**: `crates/uor-r4-core/src/transformerless/lie_jordan.rs`
- **Data Structures**:
  ```rust
  pub struct LieJordanState {
      pub symmetries: Vec<i8>,   // L_Δ anti-Hermitian generators
      pub observables: Vec<u8>,  // H_Δ Hermitian probability density
  }
  ```
- **Universal Product Kernel**:
  ```rust
  #[inline(always)]
  pub fn universal_product(a: u8, b: u8, dagger: bool) -> u8 {
      // XOR/AND/shift integer kernel - 0 floats, 0 allocations
      if dagger { a ^ (b.rotate_left(1)) } else { a & b }
  }
  ```
- **Verification**: Machine-checked source scan enforcing zero `f32`/`f64` types and zero `*`/`/` arithmetic operators in `lie_jordan.rs` hot path. `allocation_census.rs` asserting zero heap allocations in steady state.

---

### Phase 4: Bott Periodic Fock Space Context Abstraction Layer
**Goal**: Implement Bott Periodic Fock Space $\mathcal{F}_{s_0,t_0} = \bigoplus_{n=0}^\infty Cl(s_0, t_0) \otimes M_{16}(\mathbb{R})^{\otimes n}$ to compress $N$-token context windows into fixed 256-dimensional tensor states ($O(1)$ space, $O(1)$ time).

- **Module**: `crates/uor-r4-core/src/transformerless/bott_fock.rs`
- **Context Folding Engine**:
  ```rust
  pub struct BottFockContextStore {
      pub state_matrix: [i16; 256], // Folded M16(R) context matrix
      pub token_count: usize,
  }

  impl BottFockContextStore {
      pub fn append_token(&mut self, token_embedding: &[i16; 16]) {
          // Applies 8-periodicity tensor contraction Cl(s,t) ≅ Cl(s0,t0) ⊗ M16(R)^n
          // Constant O(1) time update without KV-cache expansion!
      }
  }
  ```
- **Verification**: Benchmark test `bott_fock_scaling.rs` measuring context update latency and RAM usage for $N = 1,000$, $N = 10,000$, $N = 100,000$, and $N = 1,000,000$ context tokens (asserting strict $O(1)$ constancy).

---

### Phase 5: Von Neumann Entropy Equipartition & Quantum Cover Induction
**Goal**: Implement the maximum-entropy quantum density operator $\rho = \sum \frac{1}{n} I_{n \times n}$ to bound multiresolution region covers, optimize candidate emission quantization, and eliminate noise.

- **Module**: `crates/uor-r4-core/src/transformerless/quantum_cover.rs`
- **Equipartition Density Operator**:
  ```rust
  pub fn von_neumann_entropy(density: &[f32]) -> f32 {
      // S(ρ) = - Tr(ρ ln ρ)
      density.iter().map(|&p| if p > 1e-9 { -p * p.ln() } else { 0.0 }).sum()
  }
  ```
- **Cover Induction & Emission Residuals**:
  - Filter candidate region pairs by minimum von Neumann entropy gain $\Delta S \ge S_{\min}$.
  - Quantize emission residuals into `ScoreQ` integer tables.
- **Verification**: Gate C evaluation comparing top-1 agreement and bits/token on held-out D3 corpus partition.

---

### Phase 6: Façade, Local Server, and Benchmarking Suite
**Goal**: Expose the Quantum Geometric Transformerless Engine through `uor-r4-wasm-router`, local CLI (`r4 transformerless generate`), and Wasm dashboard.

- **Modules**: `src/tless_uor.rs`, `src/lib.rs`, `src/main.rs`.
- **CLI Commands**:
  - `r4 transformerless cd-compile` — Compile teacher model into Cayley-Dickson $Cl(0,8)$ graph artifact.
  - `r4 transformerless quantum-eval` — Run full Gate C evaluation with Bott Periodic Fock Context scaling.
- **Verification**: End-to-end Wasm smoke tests, local server `/api/tless/generate` HTTP benchmarks, and complete CI workflow integration.

---

## Verification & Acceptance Matrix

| Gate | Scope | Target Metric / Requirement | Verifier |
| :--- | :--- | :--- | :--- |
| **Gate A** | Build Integrity | 100% pass across all 46 workspace crates | `cargo test --workspace --offline` |
| **Gate B** | Static Invariant | 0 floats, 0 multiplies, 0 divides in deployed hot path | `transformerless/mod.rs` P-4 machine scan |
| **Gate C** | Quality Baseline | Top-1 agreement $\ge 31.7086\%$, bits/token $\le 9.8612$ | `score_report.json` on D3 partition |
| **Gate D** | Allocation Census | 0 heap allocations in steady-state prediction loop | `allocation_census.rs` |
| **Gate E** | $\kappa$-Reproduction | Bit-exact artifact reproduction from content-addressed inputs | `kappa_reproduction.rs` |
| **Gate F** | Quantum Context | $O(1)$ time & memory scaling for $N = 10^3 \dots 10^6$ context tokens | `bott_fock_scaling.rs` benchmark |

---

## Conclusion & Action Plan

By grounding UOR-R4 in N. Furey's **nested division algebraic inclusions ($\mathbb{R} \subset \mathbb{C} \subset \mathbb{H} \subset \mathbb{O} \subset V$)**, **$Cl(0,8)$ endomorphism multiplication algebras**, and **Bott Periodic Fock space context abstraction**, we unlock the theoretical and engineering foundation for a **truly transformerless, ultra-fast, local LLM**.

Execution begins immediately upon maintainer approval of this master plan.
