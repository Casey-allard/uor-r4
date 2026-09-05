//! Mechanical persistence checks. Synthetic score tables here establish state
//! integrity, not fitted language behavior.
use super::*;
use crate::native_geometric::memory_types::{
    MemoryModel, MemoryReadFitConfig, ResponseDecision, OCCURRENCE_MEMORY_SCHEMA,
};

fn fixture(schema: Option<&str>) -> Model {
    let documents = [Document {
        id: "response-checkpoint-fixture".into(),
        text: "red green blue alpha beta gamma".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 8,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let mut model = trainer.compile().unwrap();
    if let Some(schema) = schema {
        model.memory_read = Some(MemoryModel {
            schema: schema.into(),
            baseline_artifact: model.artifact_cid().into(),
            cue_aliases: None,
            config: MemoryReadFitConfig {
                query_tokens: 3,
                source_offsets: 2,
                postings_per_address: 2,
                candidate_limit: 12,
                ..MemoryReadFitConfig::default()
            },
            source_shift: 1,
            posting_shift: 1,
            training: model.construction.clone(),
            rows: Vec::new(),
            fit_positions: 1,
            fit_schedule: None,
        });
        model.refresh_identity().unwrap();
    }
    model
}

fn active_session(model: &Model) -> Session {
    let mut session = model.session(Control::Full).unwrap();
    for token in [BOS, 67, 68, 69, 67, 70, 71, 67, 68, 72, 67, 70, 73, 67] {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    // Two distinct retained occurrences have token 67. Exercise restoration
    // of the earlier chosen occurrence, not a search by its token value.
    let memory = session.memory.as_mut().unwrap();
    let reference = memory.reference_for_sequence(10).unwrap();
    assert_eq!(memory.ring[reference.slot].token, 67);
    assert_eq!(
        memory.ring[memory.reference_for_sequence(13).unwrap().slot].token,
        67
    );
    memory.pending = Some(ResponseDecision {
        token: 67,
        score: 0,
        sequence: Some(reference.sequence),
        slot: Some(reference.slot),
        action: ResponseAction::Requery,
        at_seen: memory.seen,
    });
    session.observe(model, 67).unwrap();
    session
}

#[test]
fn response_checkpoint_rebases_slots_preserves_absolute_state_and_future_work() {
    let model = fixture(Some(RESPONSE_MEMORY_SCHEMA));
    let mut original = active_session(&model);
    let old_slot = original
        .memory
        .as_ref()
        .unwrap()
        .response
        .as_ref()
        .unwrap()
        .selected
        .unwrap()
        .slot;
    original.predict(&model).unwrap();
    let bytes = original.checkpoint().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["schema"], RESPONSE_SESSION_SCHEMA);
    assert!(value["response_memory"]["response"]
        .get("pending")
        .is_none());
    let mut restored = model.restore_session(&bytes).unwrap();
    assert_eq!(restored.state(), original.state());
    assert_eq!(restored.work, original.work);
    assert!(restored.response_decision().is_none());
    let selection = restored
        .memory
        .as_ref()
        .unwrap()
        .response
        .as_ref()
        .unwrap()
        .selected
        .unwrap();
    assert_eq!(selection.sequence, 10);
    assert_ne!(selection.slot, old_slot);
    assert_eq!(restored.checkpoint().unwrap(), bytes);
    for _ in 0..5 {
        let expected = original.predict(&model).unwrap();
        let actual = restored.predict(&model).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(restored.work, original.work);
        let expected_decision = original.response_decision();
        let actual_decision = restored.response_decision();
        assert_eq!(
            actual_decision.map(|decision| (
                decision.token,
                decision.score,
                decision.sequence,
                decision.action,
                decision.at_seen
            )),
            expected_decision.map(|decision| (
                decision.token,
                decision.score,
                decision.sequence,
                decision.action,
                decision.at_seen
            ))
        );
        original.observe(&model, expected.token).unwrap();
        restored.observe(&model, actual.token).unwrap();
        assert_eq!(restored.state(), original.state());
        assert_eq!(restored.work, original.work);
    }
}

#[test]
fn response_checkpoint_retains_query_after_all_query_tokens_are_evicted() {
    let model = fixture(Some(RESPONSE_MEMORY_SCHEMA));
    let mut original = active_session(&model);
    for token in 80..92 {
        original.observe(&model, token).unwrap();
    }
    let memory = original.memory.as_ref().unwrap();
    assert!(memory
        .response
        .as_ref()
        .unwrap()
        .queries
        .iter()
        .all(|entry| entry.sequence < memory.seen - memory.length as u64));
    let bytes = original.checkpoint().unwrap();
    let mut restored = model.restore_session(&bytes).unwrap();
    assert_eq!(restored.state(), original.state());
    assert_eq!(restored.checkpoint().unwrap(), bytes);
    assert_eq!(
        restored.predict(&model).unwrap(),
        original.predict(&model).unwrap()
    );
    assert_eq!(restored.work, original.work);
}

#[test]
fn response_checkpoint_bounds_frozen_queries_by_retained_context() {
    let mut model = fixture(Some(RESPONSE_MEMORY_SCHEMA));
    let memory = model.memory_read.as_mut().unwrap();
    memory.config.query_tokens = 16;
    memory.config.candidate_limit = 64;
    model.refresh_identity().unwrap();
    let mut original = active_session(&model);
    assert_eq!(original.state().response.unwrap().query_tokens, 8);
    let bytes = original.checkpoint().unwrap();
    let mut restored = model.restore_session(&bytes).unwrap();
    assert_eq!(restored.state(), original.state());
    assert_eq!(restored.checkpoint().unwrap(), bytes);
    assert_eq!(
        restored.predict(&model).unwrap(),
        original.predict(&model).unwrap()
    );
    assert_eq!(restored.work, original.work);
}

#[test]
fn response_checkpoint_preserves_stopped_and_cleared_boundaries() {
    let model = fixture(Some(RESPONSE_MEMORY_SCHEMA));
    let mut stopped = active_session(&model);
    stopped.observe(&model, EOS).unwrap();
    stopped.observe(&model, 79).unwrap();
    assert_eq!(
        stopped.state().response.unwrap().last_action,
        ResponseAction::Stop
    );
    let bytes = stopped.checkpoint().unwrap();
    assert_eq!(
        model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
        bytes
    );
    stopped.end_response(&model).unwrap();
    let bytes = stopped.checkpoint().unwrap();
    assert_eq!(
        model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
        bytes
    );
    for control in [Control::MemoryDisabled, Control::ResponseStateDisabled] {
        let mut disabled = model.session(control).unwrap();
        disabled.observe(&model, BOS).unwrap();
        disabled.begin_response(&model).unwrap();
        let bytes = disabled.checkpoint().unwrap();
        assert_eq!(
            model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
            bytes
        );
    }
}

#[test]
fn response_checkpoint_refuses_tampered_state_and_future_references() {
    let model = fixture(Some(RESPONSE_MEMORY_SCHEMA));
    let original = active_session(&model);
    let value: serde_json::Value = serde_json::from_slice(&original.checkpoint().unwrap()).unwrap();
    let seen = original.work.observed_tokens;
    let mutations: [fn(&mut serde_json::Value, u64); 12] = [
        |v, _| v["response_memory"]["origin"]["seen"] = 999.into(),
        |v, _| v["response_memory"]["origin"]["pose"] = 120.into(),
        |v, _| v["response_memory"]["index"][0][0] = usize::MAX.into(),
        |v, seen| v["response_memory"]["index"][0][1] = seen.into(),
        |v, _| v["response_memory"]["response"]["queries"][0]["token"] = u32::MAX.into(),
        |v, _| v["response_memory"]["response"]["queries"] = serde_json::json!([]),
        |v, seen| v["response_memory"]["response"]["references"][0] = seen.into(),
        |v, seen| v["response_memory"]["response"]["selected"] = seen.into(),
        |v, seen| v["response_memory"]["response"]["steps"] = seen.into(),
        |v, _| v["response_memory"]["response"]["selected"] = 9.into(),
        |v, _| v["response_memory"]["response"]["last_action"] = "stop".into(),
        |v, _| v["response_memory"]["response"]["queries"][0]["slot"] = 0.into(),
    ];
    for mutate in mutations {
        let mut invalid = value.clone();
        mutate(&mut invalid, seen);
        assert!(model
            .restore_session(&serde_json::to_vec(&invalid).unwrap())
            .is_err());
    }
    // Visit zero belongs to query token 67 at source distance one. Sequence
    // 10 is an existing prior value, but its preceding cue is token 72.
    let mut wrong_cue = value.clone();
    wrong_cue["response_memory"]["response"]["references"][0] = 10.into();
    assert!(model
        .restore_session(&serde_json::to_vec(&wrong_cue).unwrap())
        .is_err());
    for control in ["memory_disabled", "response_state_disabled"] {
        let mut uncleared_disabled = value.clone();
        uncleared_disabled["control"] = control.into();
        uncleared_disabled["response_memory"]["response"]["active"] = false.into();
        uncleared_disabled["response_memory"]["response"]["selected"] = serde_json::Value::Null;
        uncleared_disabled["response_memory"]["response"]["last_action"] = "stop".into();
        assert!(model
            .restore_session(&serde_json::to_vec(&uncleared_disabled).unwrap())
            .is_err());
    }
    let mut wrong_control = value;
    wrong_control["control"] = "response_state_disabled".into();
    assert!(model
        .restore_session(&serde_json::to_vec(&wrong_control).unwrap())
        .is_err());
}

#[test]
fn response_checkpoint_preserves_legacy_schema_and_refuses_cross_schema_state() {
    for schema in [None, Some(OCCURRENCE_MEMORY_SCHEMA)] {
        let model = fixture(schema);
        let mut session = model.session(Control::Full).unwrap();
        for token in [BOS, 67, 68, 69, 67, 70, 71, 67, 68, 72] {
            session.observe(&model, token).unwrap();
        }
        let bytes = session.checkpoint().unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["schema"], LEGACY_SESSION_SCHEMA);
        assert!(value.get("response_memory").is_none());
        assert_eq!(
            model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
            bytes
        );
        let mut invalid = value;
        invalid["response_memory"] = serde_json::Value::Null;
        assert!(model
            .restore_session(&serde_json::to_vec(&invalid).unwrap())
            .is_err());
    }
    let model = fixture(Some(RESPONSE_MEMORY_SCHEMA));
    let mut session = model.session(Control::Full).unwrap();
    session.begin_response(&model).unwrap();
    let bytes = session.checkpoint().unwrap();
    assert_eq!(
        model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
        bytes
    );
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["schema"] = LEGACY_SESSION_SCHEMA.into();
    value.as_object_mut().unwrap().remove("response_memory");
    assert!(model
        .restore_session(&serde_json::to_vec(&value).unwrap())
        .is_err());
}
