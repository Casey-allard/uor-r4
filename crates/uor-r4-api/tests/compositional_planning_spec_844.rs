//! #844 (S4 item A) — compositional-planning benchmark constitution and typed
//! state/action reference semantics. Frozen contract:
//! `docs/compositional_planning_spec_844.md`.
//!
//! This is the machine-checked record for #844. Increment 1 (this file) freezes
//! the `s4-compositional-reasoning` benchmark constitution in the #832
//! capability-suite framework: the primary metric, the promotion statistic, the
//! six held-out generalization split axes, and the six planning baselines. The
//! typed reference model (states/actions/transitions/plan-witnesses) and the
//! F1–F5 deterministic generators/verifiers are the follow-on build increment;
//! their tests append here.
//!
//! Boundary (S4 entry reconciliation, #826): teacher-forced only (S3 #824
//! LIMIT — no free-running generation); honest decline, not calibrated
//! confidence (S2 #823 REVISE). Nothing here is deployed-serving evidence.

use uor_r4_api::capability_suite::{
    builtin_constitution, builtin_manifests, ControlKind, SuiteManifest, Workload,
};

/// The single committed manifest for the S4 compositional-reasoning workload.
fn s4_manifest() -> SuiteManifest {
    builtin_manifests()
        .into_iter()
        .find(|m| m.workload == Workload::CompositionalReasoning)
        .expect("a committed manifest covers the compositional-reasoning workload")
}

#[test]
fn s4_constitution_manifest_validates_and_is_named_primary() {
    let m = s4_manifest();
    assert_eq!(m.validate(), None, "the frozen S4 manifest validates");
    assert_eq!(m.id, "s4-compositional-reasoning");
    // the constitution names it as the S4 primary suite and validates whole.
    let manifests = builtin_manifests();
    assert_eq!(
        builtin_constitution().validate(&manifests),
        None,
        "the constitution validates against the committed manifests"
    );
}

#[test]
fn s4_primary_metric_is_the_frozen_valid_plan_rate() {
    let m = s4_manifest();
    assert_eq!(m.primary_metric, "held-out-valid-plan-rate");
    // the promotion statistic is frozen prose naming the beat-target and the
    // memorization kill; it is intentionally non-empty and specific.
    assert!(m.promotion_statistic.contains("lower bound"));
    assert!(m.promotion_statistic.contains("memorization"));
}

#[test]
fn s4_freezes_the_six_generalization_split_axes() {
    let s = s4_manifest().split;
    assert!(s.by_entity && s.by_topology && s.by_template);
    assert!(s.by_vocabulary && s.by_operator_composition && s.by_horizon);
    assert!(s.leakage_check && s.tamper_check);
    assert_eq!(s.axis_count(), 6, "six disjointness axes are frozen");
}

#[test]
fn s4_freezes_the_six_planning_baselines() {
    let controls = s4_manifest().controls;
    for expected in [
        ControlKind::RetrievalOnly,
        ControlKind::DirectContinuation,
        ControlKind::MemorizedTrajectory,
        ControlKind::ShuffledState,
        ControlKind::ShortestPathOracle,
        ControlKind::TrivialPrior,
    ] {
        assert!(
            controls.contains(&expected),
            "baseline {} must be frozen in the S4 controls",
            expected.label()
        );
    }
    assert_eq!(
        controls.len(),
        6,
        "exactly the six frozen baselines, no more"
    );
}

#[test]
fn new_planning_baseline_labels_are_kebab_case() {
    assert_eq!(ControlKind::RetrievalOnly.label(), "retrieval-only");
    assert_eq!(
        ControlKind::DirectContinuation.label(),
        "direct-continuation"
    );
    assert_eq!(
        ControlKind::MemorizedTrajectory.label(),
        "memorized-trajectory"
    );
    assert_eq!(
        ControlKind::ShortestPathOracle.label(),
        "shortest-path-oracle"
    );
    // ALL is exhaustive and includes the new variants exactly once.
    assert_eq!(ControlKind::ALL.len(), 12);
    for c in ControlKind::ALL {
        assert_eq!(
            ControlKind::ALL.iter().filter(|&&x| x == c).count(),
            1,
            "control {} appears exactly once in ALL",
            c.label()
        );
    }
}
