//! Persistence tests use the public fit APIs on a tiny construction fixture.
//! They establish causal state/serialization behavior, not held-out quality.
use super::*;
use crate::native_geometric::completion_training::ValueCompletionFitConfig;
use crate::native_geometric::completion_types::CompletionAction;
use std::sync::OnceLock;

const PROMPT: &str = "left = 13; right = 4; total:";

fn fixture() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        let catalog = [Document {
            id: "completion-snapshot-catalog".into(),
            text: "left right total copy none 13 4 17 18 answer unknown result ;\n".into(),
        }];
        let mut trainer = Trainer::new(
            Config {
                context_tokens: 8,
                candidate_limit: 16,
                max_lexical_pieces: 128,
                ..Config::default()
            },
            &catalog,
        )
        .unwrap();
        trainer.train_documents(&catalog).unwrap();
        let baseline = trainer.compile().unwrap();
        let examples = [
            ValueExample {
                id: "completion-snapshot-fit-17".into(),
                prompt: PROMPT.into(),
                response: "17;\n".into(),
            },
            ValueExample {
                id: "completion-snapshot-fit-18".into(),
                prompt: "left = 14; right = 4; total:".into(),
                response: "18;\n".into(),
            },
        ];
        let (typed, report) = baseline
            .fit_values_with_lexeme_cues(
                &examples,
                ValueFitConfig {
                    epochs: 64,
                    learning_rate: 0.25,
                    max_features: 4096,
                },
            )
            .unwrap();
        assert_eq!(report.fit_correct, 2);
        let (learned, report) = typed
            .fit_value_completion(
                &examples,
                ValueCompletionFitConfig {
                    epochs: 32,
                    learning_rate: 0.25,
                    max_positions: 32,
                },
            )
            .unwrap();
        assert_eq!(report.matched_numeric, 2);
        assert_eq!(report.upstream_failures, 0);
        assert_eq!(report.positions, 6);
        Model::from_bytes(&learned.to_bytes().unwrap()).unwrap()
    })
}

fn before_numeral(model: &Model) -> Session {
    let mut session = model.session(Control::Full).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(PROMPT).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    session
}

fn emit_digit(session: &mut Session, model: &Model, byte: u8) {
    let prediction = session.predict(model).unwrap();
    assert_eq!(prediction.token, u32::from(byte) + 2);
    assert!(session.value_decision().is_some());
    session.observe(model, prediction.token).unwrap();
}

fn anchored(model: &Model) -> Session {
    let mut session = before_numeral(model);
    emit_digit(&mut session, model, b'1');
    assert!(!session.completion.as_ref().unwrap().active);
    emit_digit(&mut session, model, b'7');
    assert!(session.completion.as_ref().unwrap().active);
    assert_eq!(session.completion.as_ref().unwrap().steps, 0);
    session
}

fn restored(model: &Model, wire: &serde_json::Value) -> Result<Session> {
    model.restore_session(&serde_json::to_vec(wire).unwrap())
}

#[test]
fn native_completion_checkpoint_preserves_mid_suffix_and_empty_input_continuation() {
    let model = fixture();
    let mut original = anchored(model);
    original.observe(model, u32::from(b';') + 2).unwrap();
    original.predict(model).unwrap();
    assert!(original.completion.as_ref().unwrap().pending.is_some());
    let bytes = original.checkpoint().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(wire["schema"], "uor-r4.native-geometric-session/4");
    assert!(wire["completion"].get("pending").is_none());
    let mut resumed = model.restore_session(&bytes).unwrap();
    assert!(resumed.completion.as_ref().unwrap().pending.is_none());
    assert_eq!(resumed.checkpoint().unwrap(), bytes);
    // The core continuation contract consumes an empty input without another
    // response boundary; the HTTP request boundary is tested in the service.
    for token in model.encode("").unwrap() {
        resumed.observe(model, token).unwrap();
    }
    assert!(resumed.completion.as_ref().unwrap().active);
    assert_eq!(resumed.completion.as_ref().unwrap().steps, 1);
    for _ in 0..4 {
        let left = original.predict(model).unwrap();
        let right = resumed.predict(model).unwrap();
        assert_eq!(left, right);
        assert_eq!(original.completion, resumed.completion);
        original.observe(model, left.token).unwrap();
        resumed.observe(model, right.token).unwrap();
        assert_eq!(original.completion, resumed.completion);
        assert_eq!(
            serde_json::to_value(&original.values).unwrap(),
            serde_json::to_value(&resumed.values).unwrap()
        );
        if left.token == EOS {
            assert!(!resumed.completion.as_ref().unwrap().active);
            return;
        }
    }
    panic!("tiny fitted completion fixture did not terminate");
}

#[test]
fn native_completion_checkpoint_rejects_missing_foreign_and_transient_state() {
    let model = fixture();
    let original = anchored(model);
    let wire: serde_json::Value = serde_json::from_slice(&original.checkpoint().unwrap()).unwrap();
    let mut missing = wire.clone();
    missing.as_object_mut().unwrap().remove("completion");
    assert!(restored(model, &missing).is_err());
    let mut null = wire.clone();
    null["completion"] = serde_json::Value::Null;
    assert!(restored(model, &null).is_err());
    let mut transient = wire.clone();
    transient["completion"]["pending"] = serde_json::Value::Null;
    assert!(restored(model, &transient).is_err());
    let mut old_schema = wire.clone();
    old_schema["schema"] = "uor-r4.native-geometric-session/3".into();
    assert!(restored(model, &old_schema).is_err());

    let mut historical = model.clone();
    historical.completion = None;
    historical.refresh_identity().unwrap();
    let session = historical.session(Control::Full).unwrap();
    let bytes = session.checkpoint().unwrap();
    let mut old_wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(old_wire["schema"], "uor-r4.native-geometric-session/3");
    assert!(old_wire.get("completion").is_none());
    assert_eq!(
        historical
            .restore_session(&bytes)
            .unwrap()
            .checkpoint()
            .unwrap(),
        bytes
    );
    old_wire["completion"] = serde_json::Value::Null;
    assert!(restored(&historical, &old_wire).is_err());
}

#[test]
fn native_completion_checkpoint_rejects_reforged_anchor_and_actual_history() {
    let model = fixture();
    let mut original = anchored(model);
    original.observe(model, u32::from(b';') + 2).unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&original.checkpoint().unwrap()).unwrap();
    let mutations = [
        ("/completion/anchor/write_id", serde_json::json!(u64::MAX)),
        ("/completion/anchor/action", serde_json::json!("copy")),
        ("/completion/anchor/at_seen", serde_json::json!(0)),
        ("/completion/anchor/pose", serde_json::json!(u16::MAX)),
        ("/completion/anchor/query_prime", serde_json::json!(0)),
        ("/completion/steps", serde_json::json!(32)),
        ("/completion/active", serde_json::json!(false)),
        ("/completion/last", serde_json::json!(EOS)),
        ("/completion/previous", serde_json::json!(BOS)),
        ("/completion/seen", serde_json::json!(0)),
        ("/work/completion/observations", serde_json::json!(0)),
    ];
    for (pointer, value) in mutations {
        let mut changed = wire.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(restored(model, &changed).is_err(), "accepted {pointer}");
    }
    let mut phase = wire.clone();
    let old = phase["completion"]["anchor"]["phases"][0].as_u64().unwrap();
    phase["completion"]["anchor"]["phases"][0] = ((old + 1) & 65535).into();
    assert!(restored(model, &phase).is_err());
}

#[test]
fn native_completion_checkpoint_cannot_anchor_an_unfinished_numeral() {
    let model = fixture();
    let complete = anchored(model);
    let complete_wire: serde_json::Value =
        serde_json::from_slice(&complete.checkpoint().unwrap()).unwrap();
    let mut partial = before_numeral(model);
    emit_digit(&mut partial, model, b'1');
    let mut wire: serde_json::Value =
        serde_json::from_slice(&partial.checkpoint().unwrap()).unwrap();
    assert!(wire["completion"]["anchor"].is_null());
    wire["completion"]["anchor"] = complete_wire["completion"]["anchor"].clone();
    wire["completion"]["active"] = true.into();
    wire["completion"]["anchor"]["at_seen"] = wire["completion"]["seen"].clone();
    assert!(restored(model, &wire).is_err());
}

#[test]
fn native_completion_checkpoint_follows_mismatched_actual_byte() {
    let model = fixture();
    let mut original = anchored(model);
    let offered = original.predict(model).unwrap();
    assert!(original.completion.as_ref().unwrap().pending.is_some());
    let actual = if offered.token != u32::from(b'?') + 2 {
        u32::from(b'?') + 2
    } else {
        u32::from(b'!') + 2
    };
    let prior_commits = original.work.completion.commits;
    original.observe(model, actual).unwrap();
    assert_eq!(original.work.completion.commits, prior_commits);
    assert_eq!(
        original.completion.as_ref().unwrap().last_action,
        CompletionAction::Base
    );
    assert_eq!(original.completion.as_ref().unwrap().last, actual);
    assert_eq!(original.completion.as_ref().unwrap().steps, 1);
    let bytes = original.checkpoint().unwrap();
    let mut resumed = model.restore_session(&bytes).unwrap();
    assert_eq!(resumed.checkpoint().unwrap(), bytes);
    assert_eq!(
        original.predict(model).unwrap(),
        resumed.predict(model).unwrap()
    );
    assert_eq!(original.completion, resumed.completion);
}

#[test]
fn native_completion_checkpoint_uses_retained_suffix_through_step_cap() {
    let model = fixture();
    let mut original = anchored(model);
    for _ in 0..31 {
        original.observe(model, u32::from(b'?') + 2).unwrap();
    }
    assert_eq!(original.state().retained_tokens, 8);
    assert_eq!(original.completion.as_ref().unwrap().steps, 31);
    let bytes = original.checkpoint().unwrap();
    let mut resumed = model.restore_session(&bytes).unwrap();
    assert_eq!(resumed.checkpoint().unwrap(), bytes);
    original.observe(model, u32::from(b'!') + 2).unwrap();
    resumed.observe(model, u32::from(b'!') + 2).unwrap();
    for state in [&original, &resumed] {
        let completion = state.completion.as_ref().unwrap();
        assert!(!completion.active);
        assert!(completion.anchor.is_none());
        assert_eq!(completion.steps, 0);
        assert_ne!(completion.last, EOS);
        assert_eq!(state.work.completion.step_limits, 1);
    }
    assert_eq!(original.completion, resumed.completion);
    assert!(model
        .restore_session(&resumed.checkpoint().unwrap())
        .is_ok());
    resumed.observe(model, EOS).unwrap();
    resumed.observe(model, u32::from(b'x') + 2).unwrap();
    assert_eq!(
        resumed.completion.as_ref().unwrap().last_action,
        CompletionAction::Base
    );
    assert!(model
        .restore_session(&resumed.checkpoint().unwrap())
        .is_ok());
}
