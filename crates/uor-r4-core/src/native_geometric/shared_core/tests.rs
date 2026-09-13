use super::*;
use std::sync::OnceLock;

mod calibrated;
mod calibrated_audit;
mod diagnostic;

fn model() -> &'static SharedCore {
    static MODEL: OnceLock<SharedCore> = OnceLock::new();
    MODEL.get_or_init(|| SharedCore::initialized(7341).expect("canonical fixture"))
}

#[test]
fn shared_core_streaming_causality_and_checkpoint_replay() {
    let m = model();
    let mut s = m.session(Intervention::Full);
    let input = b"Record: Ada lives in Lima.\nWhere is Ada?\n";
    for &byte in input {
        let before = s.checkpoint().unwrap();
        let prediction = s.predict();
        assert_eq!(before, s.checkpoint().unwrap());
        let mut restored = m.restore(&before).unwrap();
        assert_eq!(prediction, restored.predict());
        s.observe(byte).unwrap();
        restored.observe(byte).unwrap();
        assert_eq!(s.checkpoint().unwrap(), restored.checkpoint().unwrap());
        assert!(s.last_source().is_none_or(|p| p + 1 < s.seen()));
    }
}

#[test]
fn shared_core_all_bytes_and_evicted_occurrences_are_bounded() {
    let mut s = model().session(Intervention::Full);
    for byte in 0..=255 {
        let before = s.work();
        s.observe(byte).unwrap();
        let token = s.predict();
        assert!(token <= EOS);
        assert!(s.work().candidates - before.candidates <= 32);
        assert!(s.work().output_decisions - before.output_decisions <= 9);
        assert!(s.work().products - before.products <= 100);
        assert!(s
            .last_source()
            .is_none_or(|p| p >= s.seen().saturating_sub(33)));
    }
    assert!(model().restore(&s.checkpoint().unwrap()).is_ok());
}

#[test]
fn shared_core_snapshot_rejects_wrong_artifact_and_resident_identity() {
    let m = model();
    let mut s = m.session(Intervention::Full);
    for byte in 0..40 {
        s.observe(byte).unwrap();
    }
    let mut wire: serde_json::Value = serde_json::from_slice(&s.checkpoint().unwrap()).unwrap();
    wire["artifact"] = serde_json::json!("other");
    assert!(m.restore(&serde_json::to_vec(&wire).unwrap()).is_err());
    wire["artifact"] = serde_json::json!(m.cid);
    wire["state"]["last_source"] = serde_json::json!(39);
    assert!(m.restore(&serde_json::to_vec(&wire).unwrap()).is_err());
    wire["state"]["last_source"] = serde_json::json!(0);
    assert!(m.restore(&serde_json::to_vec(&wire).unwrap()).is_err());
    wire["state"]["last_source"] = serde_json::Value::Null;
    wire["state"]["ring"][8]["position"] = serde_json::json!(999);
    assert!(m.restore(&serde_json::to_vec(&wire).unwrap()).is_err());
}

#[test]
fn shared_core_wire_roundtrip_and_invalid_parameter_rejection() {
    let m = model();
    let bytes = m.to_bytes().unwrap();
    let restored = SharedCore::from_bytes(&bytes).unwrap();
    assert_eq!(bytes, restored.to_bytes().unwrap());
    assert_eq!(m.cid, restored.cid);
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    wire["parameters"][0] = serde_json::json!(120);
    assert!(SharedCore::from_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
}

#[test]
fn shared_core_context_disabled_removes_only_reads() {
    let m = model();
    let mut s = m.session(Intervention::ContextDisabled);
    for &byte in b"a b c d e" {
        s.observe(byte).unwrap();
        assert_eq!(s.last_source(), None);
    }
    assert_eq!(s.work().candidates, 0);
    assert_eq!(s.seen(), 9);
    assert!(s.work().products > 0);
}

#[test]
fn shared_core_joint_fit_preserves_parent_and_accounts_all_proposals() {
    let m = model();
    let before = m.to_bytes().unwrap();
    let data = vec![b"a a a".to_vec()];
    let (candidate, report) = m
        .fit(
            &data,
            FitConfig {
                seed: 31,
                max_proposals: 36,
                max_seconds: 10,
            },
        )
        .unwrap();
    assert_eq!(report.proposals, 36);
    assert_eq!(report.proposals_by_family, [4; 9]);
    assert_eq!(
        report.accepted,
        report.accepted_by_family.iter().sum::<usize>()
    );
    assert!(report.after.mean_nll <= report.before.mean_nll);
    assert_eq!(
        candidate
            .evaluate(&data, Intervention::Full)
            .unwrap()
            .mean_nll,
        report.after.mean_nll
    );
    assert_eq!(before, m.to_bytes().unwrap());
}

#[test]
fn shared_core_runtime_source_excludes_linear_predictor_and_float_work() {
    let source = include_str!("../shared_core.rs");
    let kernel = source
        .split("// SHARED_CORE_INTEGER_KERNEL_BEGIN")
        .nth(1)
        .unwrap()
        .split("// SHARED_CORE_INTEGER_KERNEL_END")
        .next()
        .unwrap();
    for forbidden in [
        "f32",
        "f64",
        ".scores",
        "score_candidate",
        "gate_eighths",
        "Vec::",
        "vec![",
        "softmax(",
        ".rows[",
    ] {
        assert!(!kernel.contains(forbidden), "{forbidden}");
    }
    assert!(!kernel.contains(" * "));
    assert!(!kernel.contains(" / "));
    let scan = crate::transformerless::source_scan::scan_for_forbidden_arith_and_floats(kernel);
    assert!(scan.offenders.is_empty(), "{:?}", scan.offenders);
    assert!(scan.allowed.is_empty());
}

mod credit_assignment;

mod tied_learning;

mod full_objective;

mod final_emission;

mod angular_tree_experiment;

mod decoder_validation;

mod tree_state_pair;

mod coupled_profile;

mod active_sensitivity;
