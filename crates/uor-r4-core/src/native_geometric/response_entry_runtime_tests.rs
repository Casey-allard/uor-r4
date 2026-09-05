//! Mechanical entry, observation, numeric precedence and persistence checks.
//! Fixed rows here establish state laws, not fitted model capability.
use super::completion_types::{CompletionModel, COMPLETION_SCHEMA};
use super::numeral::NUMERAL_CODEC;
use super::response_entry_types::*;
use super::value_types::{ValueFeature, ValueModel, ValueRow, LEXEME_VALUE_SCHEMA, VALUES};
use super::*;

const EMIT: u32 = b'!' as u32 + 2;
const OTHER: u32 = b'?' as u32 + 2;

fn mechanical(copy: bool, continuation: u32) -> Model {
    let documents = [Document {
        id: "response-entry-mechanical-catalog".into(),
        text: "source value query unknown reply 9 17 101 ; : ! ?\n".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 8,
            candidate_limit: 16,
            max_lexical_pieces: 128,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let mut model = trainer.compile().unwrap();
    model.values = Some(ValueModel {
        schema: LEXEME_VALUE_SCHEMA.into(),
        codec: NUMERAL_CODEC.into(),
        capacity: VALUES,
        rows: if copy {
            vec![ValueRow {
                feature: ValueFeature {
                    kind: 0,
                    a: 0,
                    b: 0,
                },
                weight: 4096,
            }]
        } else {
            Vec::new()
        },
        continuation_score: 4096,
        fit_config: [0; 4],
        training: Vec::new(),
    });
    model.refresh_identity().unwrap();
    model.completion = Some(CompletionModel {
        schema: COMPLETION_SCHEMA.into(),
        baseline_artifact: model.artifact_cid.clone(),
        rows: Vec::new(),
        global_postings: Vec::new(),
        fit_config: [0; 4],
        fit_positions: 0,
        training: Vec::new(),
    });
    model.refresh_identity().unwrap();
    model.response_entry = Some(ResponseEntryModel {
        schema: RESPONSE_ENTRY_SCHEMA.into(),
        copy: None,
        baseline_artifact: model.artifact_cid.clone(),
        rows: [(0, EMIT), (16, continuation)]
            .into_iter()
            .map(|(kind, token)| ScoreRow {
                feature: Feature { kind, value: 0 },
                default_score: 0,
                scores: vec![TokenScore { token, score: 1000 }],
                postings: vec![token],
            })
            .collect(),
        global_postings: if continuation == EMIT {
            vec![EMIT]
        } else {
            vec![EMIT, continuation]
        },
        fit_config: [0; 5],
        fit_positions: 0,
        training: Vec::new(),
    });
    model.refresh_identity().unwrap();
    Model::from_bytes(&model.to_bytes().unwrap()).unwrap()
}

fn prefix(model: &Model, control: Control, source: &str) -> Session {
    let mut session = model.session(control).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(source).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    session
}

#[test]
fn native_response_entry_commits_only_selected_observation_and_tracks_actual_mismatch() {
    let model = mechanical(false, EMIT);
    let mut session = prefix(&model, Control::Full, "source 17; query:");
    let boundary = session.response_entry.as_ref().unwrap().boundary.unwrap();
    assert!(!session.response_entry.as_ref().unwrap().active);
    let first = session.predict(&model).unwrap();
    assert_eq!(first.token, EMIT);
    let decision = session.response_entry_decision().unwrap();
    assert_eq!(decision.action, ResponseEntryAction::Enter);
    assert_eq!(decision.boundary_seen, boundary.at_seen);
    assert_eq!(session.predict(&model).unwrap(), first);
    assert_eq!(session.response_entry.as_ref().unwrap().steps, 0);
    assert_eq!(session.work.values.derived_writes, 0);
    session.observe(&model, first.token).unwrap();
    assert!(session.response_entry.as_ref().unwrap().active);
    assert_eq!(session.response_entry.as_ref().unwrap().steps, 1);
    assert_eq!(
        session.response_entry.as_ref().unwrap().last_action,
        ResponseEntryAction::Enter
    );
    session.predict(&model).unwrap();
    session.observe(&model, OTHER).unwrap();
    let entry = session.response_entry.as_ref().unwrap();
    assert_eq!(entry.steps, 2);
    assert_eq!(entry.last, OTHER);
    assert_eq!(entry.last_action, ResponseEntryAction::Base);
    assert_eq!(session.work.response_entry.mismatches, 1);
    let (features, count) = entry.features(
        &model,
        session.values.as_ref().unwrap(),
        Control::Full,
        &mut CompletionWork::default(),
    );
    assert_eq!(count, 16);
    assert!(features[..count].iter().all(|feature| feature.kind >= 16));
    assert!(features[..count].contains(&Feature {
        kind: 17,
        value: u64::from(model.geometry.tokens[OTHER as usize].prime),
    }));
    let restored = model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
    assert_eq!(restored.state(), session.state());
    assert_eq!(session.work.values.derived_writes, 0);
}

#[test]
fn native_response_entry_first_unselected_or_mismatched_observation_closes_boundary() {
    let model = mechanical(false, EMIT);
    for predict in [false, true] {
        let mut session = prefix(&model, Control::Full, "source 17; query:");
        if predict {
            assert_eq!(session.predict(&model).unwrap().token, EMIT);
        }
        session.observe(&model, OTHER).unwrap();
        assert!(session.response_entry.as_ref().unwrap().boundary.is_none());
        assert!(!session.response_entry.as_ref().unwrap().active);
        session.predict(&model).unwrap();
        assert!(session.response_entry_decision().is_none());
        model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
    }
}

#[test]
fn native_response_entry_respects_typed_precedence_and_valid_gate_boundaries() {
    let model = mechanical(true, EMIT);
    let mut numeric = prefix(&model, Control::Full, "source 17; query:");
    let first = numeric.predict(&model).unwrap();
    assert_eq!(first.token, b'1' as u32 + 2);
    assert!(numeric.value_decision().is_some());
    assert!(numeric.response_entry_decision().is_none());
    numeric.observe(&model, first.token).unwrap();
    assert!(numeric.response_entry.as_ref().unwrap().boundary.is_none());
    let model = mechanical(false, EMIT);
    for control in [
        Control::ResponseEntryDisabled,
        Control::ValuesDisabled,
        Control::MemoryDisabled,
    ] {
        let mut session = prefix(&model, control, "source 17; query:");
        session.predict(&model).unwrap();
        assert!(session.response_entry_decision().is_none());
        assert!(session.response_entry.as_ref().unwrap().boundary.is_none());
    }
    let mut empty = prefix(&model, Control::Full, "source without value query:");
    empty.predict(&model).unwrap();
    assert!(empty.response_entry_decision().is_none());
    let mut exhausted = prefix(&model, Control::Full, "source 17; query:");
    exhausted.values.as_mut().unwrap().next_id = u64::MAX;
    exhausted.predict(&model).unwrap();
    assert!(exhausted.response_entry_decision().is_none());
}

#[test]
fn native_response_entry_eos_and_observation_cap_end_without_invented_tokens() {
    let model = mechanical(false, EOS);
    let mut session = prefix(&model, Control::Full, "source 17; query:");
    let first = session.predict(&model).unwrap();
    session.observe(&model, first.token).unwrap();
    let stop = session.predict(&model).unwrap();
    assert_eq!(stop.token, EOS);
    assert_eq!(
        session.response_entry_decision().unwrap().action,
        ResponseEntryAction::Stop
    );
    session.observe(&model, EOS).unwrap();
    assert!(!session.response_entry.as_ref().unwrap().active);
    assert_eq!(session.work.response_entry.stops, 1);
    model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();

    let model = mechanical(false, EMIT);
    let mut session = prefix(&model, Control::Full, "source 17; query:");
    for step in 1..=RESPONSE_ENTRY_STEPS {
        let next = session.predict(&model).unwrap();
        assert_eq!(next.token, EMIT);
        session.observe(&model, next.token).unwrap();
        if step == RESPONSE_ENTRY_STEPS - 1 {
            let restored = model
                .restore_session(&session.checkpoint().unwrap())
                .unwrap();
            assert_eq!(restored.state(), session.state());
        }
    }
    let entry = session.response_entry.as_ref().unwrap();
    assert!(!entry.active);
    assert!(entry.boundary.is_none());
    assert_eq!(entry.last, EMIT);
    assert_eq!(entry.steps, 0);
    assert_eq!(session.work.response_entry.step_limits, 1);
    assert_eq!(session.work.response_entry.stops, 0);
    model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
}

#[test]
fn native_response_entry_snapshot_rejects_foreign_origin_history_and_transient_fields() {
    let model = mechanical(false, EMIT);
    let mut session = prefix(&model, Control::Full, "source 17; query:");
    let initial = session.checkpoint().unwrap();
    model.restore_session(&initial).unwrap();
    let first = session.predict(&model).unwrap();
    session.observe(&model, first.token).unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    let mut bad = wire.clone();
    bad["response_entry"]["boundary"]["query_prime"] = serde_json::json!(u32::MAX);
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
    let mut bad = wire.clone();
    bad["response_entry"]["last"] = serde_json::json!(OTHER);
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
    let mut bad = wire.clone();
    bad["response_entry"]["pending"] = serde_json::Value::Null;
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
    let mut bad = wire.clone();
    bad["response_entry"] = serde_json::Value::Null;
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
    let mut bad = wire.clone();
    bad["response_entry"]["boundary"]["pose"] = serde_json::json!(u16::MAX);
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());

    // A changed, valid model with the same upstream baseline must not accept
    // a forged origin merely by replacing the outer artifact identity.
    let mut no_entry = model.clone();
    no_entry.response_entry.as_mut().unwrap().rows[0].scores[0].score = -1000;
    no_entry.refresh_identity().unwrap();
    let no_entry = Model::from_bytes(&no_entry.to_bytes().unwrap()).unwrap();
    let mut bad = wire;
    bad["response_entry"]["last_action"] = serde_json::json!("base");
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
    bad["response_entry"]["last_action"] = serde_json::json!("enter");
    bad["artifact_cid"] = serde_json::json!(no_entry.artifact_cid);
    assert!(no_entry
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
}

#[test]
fn native_response_entry_uses_disjoint_entry_and_continuation_geometry_namespaces() {
    let model = mechanical(false, EMIT);
    let mut session = prefix(&model, Control::Full, "source 17; query:");
    let (entry_features, entry_count) = session.response_entry.as_ref().unwrap().features(
        &model,
        session.values.as_ref().unwrap(),
        Control::Full,
        &mut CompletionWork::default(),
    );
    assert_eq!(entry_count, RESPONSE_ENTRY_FEATURES);
    assert!(entry_features[..entry_count]
        .iter()
        .all(|feature| feature.kind < 16));
    let first = session.predict(&model).unwrap();
    session.observe(&model, first.token).unwrap();
    let (features, count) = session.response_entry.as_ref().unwrap().features(
        &model,
        session.values.as_ref().unwrap(),
        Control::ResponseEntryGeometryDisabled,
        &mut CompletionWork::default(),
    );
    assert_eq!(count, 6);
    assert!(features[..count]
        .iter()
        .all(|feature| (16..22).contains(&feature.kind)));
}

#[test]
fn native_response_entry_counts_unselected_eos_and_restores_a_new_boundary_after_eos() {
    let model = mechanical(false, EMIT);
    let mut session = prefix(&model, Control::Full, "source 17; query:");
    assert_eq!(session.predict(&model).unwrap().token, EMIT);
    session.observe(&model, EOS).unwrap();
    assert_eq!(session.work.response_entry.base_steps, 1);
    assert_eq!(session.work.response_entry.mismatches, 1);
    assert_eq!(session.work.response_entry.commits, 0);
    assert_eq!(session.work.response_entry.stops, 0);
    model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();

    // The existing begin_response law can establish a new response after an
    // actual EOS. That ready, zero-step boundary is distinct from an active
    // response attempting to cross an EOS in its committed history.
    session.begin_response(&model).unwrap();
    let ready = session.response_entry.as_ref().unwrap();
    assert!(ready.boundary.is_some());
    assert!(!ready.active);
    assert_eq!(ready.last, EOS);
    assert_eq!(ready.steps, 0);
    let mut restored = model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
    assert_eq!(restored.state(), session.state());
    assert_eq!(
        restored.predict(&model).unwrap(),
        session.predict(&model).unwrap()
    );

    let first = session.predict(&model).unwrap();
    session.observe(&model, first.token).unwrap();
    // Without another prediction, the next EOS is a Base observation ending
    // the active component, not a committed Stop decision.
    session.observe(&model, EOS).unwrap();
    assert_eq!(session.work.response_entry.base_steps, 2);
    assert_eq!(session.work.response_entry.mismatches, 1);
    assert_eq!(session.work.response_entry.commits, 1);
    assert_eq!(session.work.response_entry.stops, 1);
    model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
}

#[test]
fn native_response_entry_reuses_no_write_only_during_committed_response() {
    use super::value_types::ValueWork;

    let model = mechanical(false, EMIT);
    let mut session = prefix(&model, Control::Full, "source 17; source 9; query:");
    assert_eq!(session.values.as_ref().unwrap().sources.len(), 2);
    let first = session.predict(&model).unwrap();
    assert!(session.work.values.proposals > 0);
    assert!(session.work.values.additions > 0);
    assert!(session.work.values.feature_lookups > 0);
    session.observe(&model, first.token).unwrap();
    assert!(session.response_entry.as_ref().unwrap().active);
    assert!(session.value_decision().is_none());
    // Restoration rechecks the original selector and retains the same right
    // to reuse NoWrite; it does not serialize a pending prediction or cache.
    session = model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();

    for step in 0..6 {
        let mut reference = session.values.as_ref().unwrap().clone();
        let mut hypothetical_work = ValueWork::default();
        assert!(reference
            .offer(
                &model,
                Candidate {
                    token: OTHER,
                    score: 1234 + step
                },
                Control::Full,
                &mut hypothetical_work,
            )
            .is_none());
        assert!(hypothetical_work.proposals > 0);
        assert!(hypothetical_work.additions > 0);
        assert!(hypothetical_work.feature_lookups > 0);

        let before = session.work.values;
        let prediction = session.predict(&model).unwrap();
        assert_eq!(prediction.token, EMIT);
        assert_eq!(session.predict(&model).unwrap(), prediction);
        assert_eq!(session.work.values, before);
        assert!(session.value_decision().is_none());
        session
            .observe(&model, if step & 1 == 0 { EMIT } else { OTHER })
            .unwrap();
        assert!(session.response_entry.as_ref().unwrap().active);
    }
    assert!(session.work.response_entry.mismatches > 0);
    assert!(session.work.response_entry.commits > 1);

    for _ in 0..RESPONSE_ENTRY_STEPS {
        if !session.response_entry.as_ref().unwrap().active {
            break;
        }
        let before = session.work.values;
        session.predict(&model).unwrap();
        assert_eq!(session.work.values, before);
        session.observe(&model, OTHER).unwrap();
    }
    assert!(!session.response_entry.as_ref().unwrap().active);
    assert_eq!(session.work.response_entry.step_limits, 1);
    let after_cap = session.work.values;
    session.predict(&model).unwrap();
    assert!(session.work.values.proposals > after_cap.proposals);
    assert!(session.work.values.additions > after_cap.additions);
    assert!(session.work.values.feature_lookups > after_cap.feature_lookups);
    assert!(session.response_entry_decision().is_none());

    session.end_response(&model).unwrap();
    for token in model.encode("source 101; query:").unwrap() {
        session.observe(&model, token).unwrap();
    }
    session.begin_response(&model).unwrap();
    assert!(!session.response_entry.as_ref().unwrap().active);
    let before_new_query = session.work.values;
    let first = session.predict(&model).unwrap();
    assert!(session.work.values.proposals > before_new_query.proposals);
    assert!(session.work.values.additions > before_new_query.additions);
    assert!(session.work.values.feature_lookups > before_new_query.feature_lookups);
    assert_eq!(
        session.response_entry_decision().unwrap().action,
        ResponseEntryAction::Enter
    );
    session.observe(&model, first.token).unwrap();
    assert!(session.response_entry.as_ref().unwrap().active);
    assert!(session.value_decision().is_none());
}
