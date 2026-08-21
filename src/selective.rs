//! Typed selective-prediction surface vocabulary (#839 phase 1, RF-30).
//!
//! The shared, dependency-free vocabulary of the #838 typed status schema
//! (`docs/selective_prediction_spec_838.md` §2/§5/§6) as served in
//! **legacy-coverage mode**: canonical kebab-case labels, the OpenAI-surface
//! snake-case rewrite, the deployed-policy → coverage mapping, and the
//! fail-closed selective-calibration presence probe. Every production surface
//! (CLI, native HTTP, OpenAI-compatible, WASM) reads its labels from here so
//! no two surfaces can drift.
//!
//! This module deliberately has no dependencies and no `cfg` gating: the WASM
//! boundary uses the same constants as the native server. Phase 1 serves the
//! schema in legacy-coverage mode only — the evidence axis, confidence
//! values, and calibrated-mode execution are #839 phase 2, gated on the
//! frozen release bar; nothing here fabricates either (spec §6).

/// Canonical outcome labels (spec §2).
pub const STATUS_SUPPORTED_ANSWER: &str = "supported-answer";
pub const STATUS_ABSTENTION: &str = "abstention";
pub const STATUS_HARD_INCOMPATIBILITY: &str = "hard-incompatibility";

/// Canonical coverage-axis labels (spec §2).
pub const COVERAGE_COVERED: &str = "covered";
pub const COVERAGE_DISTRIBUTIONALLY_NOVEL: &str = "distributionally-novel";

/// The only abstention cause legal in legacy-coverage mode (spec §6).
pub const CAUSE_DISTRIBUTIONALLY_NOVEL: &str = "distributionally-novel";

/// OpenAI-compatible error envelope vocabulary (spec §5): the vendored
/// error `type`, and the typed codes produced by the deterministic
/// `-` → `_` rewrite of the kebab labels.
pub const OPENAI_ERROR_TYPE: &str = "uor_selective_prediction";
pub const OPENAI_CODE_ABSTENTION_DISTRIBUTIONALLY_NOVEL: &str =
    "uor_abstention_distributionally_novel";
pub const OPENAI_CODE_INCOMPATIBLE_ARTIFACT: &str = "uor_incompatible_artifact";

/// The versioned optional selective-prediction calibration sidecar filename,
/// looked for beside the active compiled artifact. DISTINCT from the
/// compiler-side `hamming_calibration.json` (a hamming-radius conversion
/// input), which is untouched by the selective-prediction contract.
///
/// Spec §6 semantics as lowered in phase 1: ABSENT → legacy-coverage mode
/// (today's D4 policy through the typed schema). PRESENT → hard
/// incompatibility for every request that would consult the artifact — no
/// executable calibrated mode exists until phase 2 clears its frozen gate,
/// and corrupt-or-unknown calibration data must never silently degrade to
/// the always-serve legacy surface.
pub const SELECTIVE_CALIBRATION_FILE: &str = "selective_calibration.bin";

/// Map a deployed D4 policy status label (`uor_r4_api::engine::PolicyStatus::
/// label`) to the spec §2 coverage-axis label. `exact_context` and `graph`
/// are resolved readings (`covered`); `novel` is the D4 novel reading
/// (`distributionally-novel`). The reserved `contradictory` arm is a
/// resolved-but-disagreeing reading, so it reports `covered` on the coverage
/// axis (the scorer does not produce it today; the evidence-axis cause it
/// would imply is calibrated-mode vocabulary that legacy mode must not
/// mint — spec §6).
pub fn coverage_for_policy_label(label: &str) -> Option<&'static str> {
    match label {
        "exact_context" | "graph" | "contradictory" => Some(COVERAGE_COVERED),
        "novel" => Some(COVERAGE_DISTRIBUTIONALLY_NOVEL),
        _ => None,
    }
}

/// The deterministic kebab → snake rewrite for OpenAI-compatible
/// `error.code` values (spec §5).
pub fn snake(label: &str) -> String {
    label.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spec §5: the OpenAI code is exactly the snake rewrite of the kebab
    /// cause, and the canonical constants agree with the rewrite.
    #[test]
    fn openai_codes_are_the_snake_rewrite_of_the_kebab_labels() {
        assert_eq!(
            format!("uor_abstention_{}", snake(CAUSE_DISTRIBUTIONALLY_NOVEL)),
            OPENAI_CODE_ABSTENTION_DISTRIBUTIONALLY_NOVEL
        );
        assert_eq!(snake(STATUS_HARD_INCOMPATIBILITY), "hard_incompatibility");
    }

    /// The coverage mapping is total over the deployed policy label space
    /// and maps only `novel` to the novel coverage reading.
    #[test]
    fn coverage_mapping_covers_the_policy_label_space() {
        assert_eq!(
            coverage_for_policy_label("exact_context"),
            Some(COVERAGE_COVERED)
        );
        assert_eq!(coverage_for_policy_label("graph"), Some(COVERAGE_COVERED));
        assert_eq!(
            coverage_for_policy_label("novel"),
            Some(COVERAGE_DISTRIBUTIONALLY_NOVEL)
        );
        assert_eq!(
            coverage_for_policy_label("contradictory"),
            Some(COVERAGE_COVERED)
        );
        assert_eq!(coverage_for_policy_label("unknown"), None);
    }
}
