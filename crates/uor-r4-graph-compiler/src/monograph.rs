//! Hologram/R4 Formal Monograph Validator & Traceability Harness
//!
//! Specification & Source: `docs/hologram_formal_analysis_direction.md` PDF §§14–17;
//! `docs/formal_vocabulary.md`; GitHub Issue #133.
//!
//! This module provides programmatic validation for the formal monograph:
//! - Section completeness verification across all 19 monograph sections.
//! - Traceability link validation connecting implementation modules to proof matrix entries.
//! - Verification of explicit non-goals and claim-wording boundaries.

/// Monograph validation report metrics. `verified` is `true` only when every
/// required section, module traceability link, and non-goal disavowal is
/// present; otherwise the count fields report how many of each were found (R5 —
/// a failed validation is a measured report, not a raised error).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonographValidationReport {
    pub total_sections_verified: usize,
    pub total_modules_linked: usize,
    pub non_goals_disavowed: usize,
    pub verified: bool,
}

/// Monograph Traceability Verifier.
pub struct MonographTraceabilityVerifier;

impl MonographTraceabilityVerifier {
    /// Validate full formal monograph markdown text. Total: always produces a
    /// [`MonographValidationReport`]; `verified` is `true` only when every
    /// required section, module link, and non-goal disavowal is present.
    pub fn validate_monograph_text(content: &str) -> MonographValidationReport {
        let required_sections = [
            "Section 1: Problem Statement and Non-Goals",
            "Section 2: Semantic State Spaces and Holographic Projections",
            "Section 3: Graph Induction & Multi-Edge Algebras",
            "Section 4: Predictive Entropy & Information Bottleneck",
            "Section 5: Unsupervised Behavioral Probes & Anti-Memorization",
            "Section 6: Future-State Optimization & Bounded Planning",
            "Section 7: Reference Intermediate Representation (IR)",
            "Section 8: Boolean / Integer Lowering & R4G1 Format",
            "Section 9: Structural Proofs & Proof Matrix",
            "Section 10: Decoupled Semantic Reasoning & Language Emission",
            "Section 11: Graph Invariant Ownership & Validation",
            "Section 12: Rate-Distortion Semantic Compression",
            "Section 13: PDF-to-Implementation Traceability",
            "Section 14: Complete Traceability & Proof Status Matrix",
            "Section 15: Issue Dependency Graph",
            "Section 16: Review Checklist vs Repos & Specifications",
            "Section 17: Known Negative Results & Disavowals",
            "Section 18: Legacy-Preserving Migration Path",
            "Section 19: Empirical Certification & Quality Gates",
        ];

        let required_modules = [
            "semantic_state.rs",
            "records.rs",
            "holographic_encoding.rs",
            "behavioral_probes.rs",
            "reference_compiler_ir.rs",
            "lower_semantic_regions.rs",
            "future_state_planner.rs",
            "structural_guarantees.rs",
            "semantic_emission_decoupling.rs",
            "invariant_ownership.rs",
            "rate_distortion_compression.rs",
            "pdf_traceability.rs",
        ];

        let required_non_goals = [
            "No Human-Level Reasoning Claim",
            "No Exact Teacher Equivalence",
            "No Floating-Point Runtime Hot Path",
        ];

        let sections_count = required_sections
            .iter()
            .filter(|sec| content.contains(**sec))
            .count();
        let modules_count = required_modules
            .iter()
            .filter(|mod_name| content.contains(**mod_name))
            .count();
        let non_goals_count = required_non_goals
            .iter()
            .filter(|ng| content.contains(**ng))
            .count();

        let verified = sections_count == required_sections.len()
            && modules_count == required_modules.len()
            && non_goals_count == required_non_goals.len();

        MonographValidationReport {
            total_sections_verified: sections_count,
            total_modules_linked: modules_count,
            non_goals_disavowed: non_goals_count,
            verified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monograph_validation_passes() {
        let content = include_str!("../../../docs/hologram_r4_formal_monograph.md");
        let report = MonographTraceabilityVerifier::validate_monograph_text(content);

        assert_eq!(report.total_sections_verified, 19);
        assert_eq!(report.total_modules_linked, 12);
        assert_eq!(report.non_goals_disavowed, 3);
        assert!(report.verified);
    }
}
