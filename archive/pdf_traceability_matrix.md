# Hologram / R⁴ PDF-to-Implementation Traceability Matrix

**Version:** 1.0.0  
**Date:** 2026-07-24  
**Source Baseline:** `docs/hologram_formal_analysis_direction.md` §§1–17; `docs/formal_vocabulary.md`; GitHub Issues #11–#34, #122–#137.

---

## 17-Section PDF Traceability Matrix

| PDF Section | Concept / Recommendation | Issue Mapping | Code / Document Location | Evidence Artifact | Overlaps & Dependencies | Claim Class | Proof Status | Primary Owner |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **§1** | Holographic Architecture & Formal Vocabulary | #11–#15, #123 | `docs/formal_vocabulary.md` | `scripts/check_claim_wording.py` | Overlaps #124, #137 | Definition | N/A | Casey Allard |
| **§2** | Semantic State Manifold & Dynamics | #16–#20, #124 | `uor-r4-graph-compiler::semantic_state` | `crates/uor-r4-graph-compiler/src/semantic_state.rs` | Depended by §3, §12 | Definition | N/A | Casey Allard |
| **§3** | Bounded Trajectories & Future State Planning | #131 | `uor-r4-graph-compiler::future_state_planner` | `crates/uor-r4-graph-compiler/src/future_state_planner.rs` | Depends on §2 | Objective | N/A | Casey Allard |
| **§4** | Multiple Edge Algebras over One Graph | #21–#25, #125 | `uor-r4-graph-format::stage2` | `crates/uor-r4-graph-format/src/stage2.rs` | Overlaps §9 | Definition | N/A | Alex Flom |
| **§5** | Holographic Partial Reconstruction | #26–#30, #126 | `uor-r4-graph-certify::holographic_encoding` | `crates/uor-r4-graph-certify/src/holographic_encoding.rs` | Overlaps #108 | Definition | N/A | Alex Flom |
| **§6** | Predictive Entropy & Information Bottleneck | #31–#34, #127 | `uor-r4-graph-compiler::induction` | `crates/uor-r4-graph-compiler/src/induction.rs` | Overlaps §7 | Objective | N/A | Alex Flom |
| **§7** | Lossy Semantic Compression & Rate-Distortion | #136 | `uor-r4-graph-compiler::induction` | `crates/uor-r4-graph-compiler/src/induction.rs` | Depends on §10, §6 | Objective | N/A | Casey Allard |
| **§8** | Unsupervised Behavioral Probes | #128 | `uor-r4-graph-compiler::behavioral_probes` | `crates/uor-r4-graph-compiler/src/behavioral_probes.rs` | Overlaps §2 | Empirical Criterion | Verified | Casey Allard |
| **§9** | Graph Invariant Ownership & Loader Matrix | #135 | `uor-r4-graph-format::invariant_ownership` | `crates/uor-r4-graph-format/src/invariant_ownership.rs` | Overlaps §4, §14 | Guarantee | Verified | Casey Allard |
| **§10** | Reference Compiler IR & Differential Loss | #129 | `uor-r4-graph-compiler::reference_compiler_ir` | `crates/uor-r4-graph-compiler/src/reference_compiler_ir.rs` | Overlaps §11 | Definition | N/A | Casey Allard |
| **§11** | Lower Semantic Regions & Boolean Masks | #130 | `uor-r4-graph-compiler::lower_semantic_regions` | `crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs` | Depends on §10 | Guarantee | Verified | Casey Allard |
| **§12** | Typed Semantic Transition Dynamics & Preconditions | #124, #131 | `uor-r4-graph-compiler::semantic_state` | `crates/uor-r4-graph-compiler/src/semantic_state.rs` | Overlaps §2, §3 | Definition | N/A | Casey Allard |
| **§13** | Decoupled Semantic Reasoning & Language Emission | #134 | `uor-r4-graph-compiler::semantic_emission_decoupling` | `crates/uor-r4-graph-compiler/src/semantic_emission_decoupling.rs` | Depends on §2, #131 | Definition | N/A | Casey Allard |
| **§14** | Structural Proof Matrix & Guaranteed Horizon | #132 | `uor-r4-proof-model::structural_guarantees` | `crates/uor-r4-proof-model/src/structural_guarantees.rs` | Depends on §9 | Guarantee | Verified | Casey Allard |
| **§15** | Living Formal Monograph | #133 | `docs/hologram_r4_formal_monograph.md` | `docs/hologram_r4_formal_monograph.md` | Synthesizes §§1–14 | Definition | N/A | Casey Allard |
| **§16** | Comprehensive PDF Traceability Matrix | #137 | `docs/pdf_traceability_matrix.md` | `docs/pdf_traceability_matrix.md` | Maps §§1–17 | Definition | N/A | Casey Allard |
| **§17** | Research Sequence & Roadmap Integration | #122–#137 | `docs/hologram_formal_analysis_direction.md` | `docs/hologram_formal_analysis_direction.md` | Integrates #11–#137 | Definition | N/A | Casey Allard |

---

## Verification & Audit Invariants

1. **Complete 17-Section PDF Coverage:** Every section §1–§17 in the formal direction document is explicitly mapped to an issue and valid code/doc path.
2. **Path Existence & Evidence Requirement:** All evidence artifact links correspond to existing code files or documentation artifacts.
3. **Claim Class & Proof Status Alignment:** Claims follow `docs/formal_vocabulary.md` categories. `Guarantee` and `Empirical Criterion` carry proof statuses (`Verified`), while `Definition` and `Objective` carry structural status `N/A`.
