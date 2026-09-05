//! Causal state and finite operator checks; quality is measured by saved-model
//! generation separately. Internal fixtures deliberately choose model scores.
use super::memory_types::*;
use super::*;

fn fixture(context: usize) -> Model {
    let documents = [Document {
        id: "response-kernel".into(),
        text: "red blue green cue tail".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: context,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let mut model = trainer.compile().unwrap();
    model.memory_read = Some(MemoryModel {
        schema: RESPONSE_MEMORY_SCHEMA.into(),
        baseline_artifact: model.artifact_cid().into(),
        cue_aliases: None,
        config: MemoryReadFitConfig {
            query_tokens: 2,
            source_offsets: 2,
            postings_per_address: 2,
            candidate_limit: 8,
            ..MemoryReadFitConfig::default()
        },
        source_shift: 1,
        posting_shift: 1,
        training: Vec::new(),
        rows: Vec::new(),
        fit_positions: 0,
        fit_schedule: None,
    });
    model.refresh_identity().unwrap();
    model
}

fn prefix(model: &Model, tokens: &[u32]) -> Session {
    let mut session = model.session(Control::Full).unwrap();
    for &token in tokens {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    session
}

#[test]
fn response_prediction_is_idempotent_and_wrong_observation_cannot_choose_a_source() {
    let model = fixture(16);
    let mut session = prefix(&model, &[BOS, 67, 68, 69, 70, 67]);
    let before = session.state();
    let first = session.predict(&model).unwrap();
    let decision = session.response_decision().unwrap();
    assert_eq!(session.state(), before);
    assert_eq!(session.predict(&model).unwrap(), first);
    assert_eq!(session.response_decision(), Some(decision));
    assert_eq!(session.state(), before);
    let wrong = if first.token == 68 { 69 } else { 68 };
    session.observe(&model, wrong).unwrap();
    let state = session.state().response.unwrap();
    assert_eq!(state.steps, 1);
    assert_eq!(state.selected_sequence, None);
    assert_eq!(session.work.response_mismatches, 1);
    assert_eq!(session.response_decision(), None);
    let after = session.state();
    assert!(session.observe(&model, u32::MAX).is_err());
    assert_eq!(session.state(), after);
}

#[test]
fn equal_value_occurrences_commit_distinct_reads_and_causal_successors() {
    let model = fixture(16);
    let session = prefix(&model, &[BOS, 67, 68, 69, 67, 68, 70, 67]);
    let mut selected_next = Vec::new();
    for sequence in [2, 5] {
        let mut state = session.memory.clone().unwrap();
        // Two legitimate model alternatives emit the same token. Select one
        // by its model score; the subsequently observed token gives no position.
        state.composed = vec![ComposedCandidate {
            action: ResponseAction::Requery,
            sequence,
            token: 68,
            score: 1000,
            feature_start: 0,
            feature_count: 0,
        }];
        let mut work = Work::default();
        state.select_response(
            &model,
            Candidate {
                token: 68,
                score: 1000,
            },
            &mut work,
        );
        assert_eq!(state.response.as_ref().unwrap().selected, None);
        state.observe(&model, model.memory_read.as_ref().unwrap(), 68, &mut work);
        assert_eq!(
            state.response.as_ref().unwrap().selected.unwrap().sequence,
            sequence
        );
        state.candidates.clear();
        state.collect_continuation(&model, &mut work);
        assert_eq!(state.candidates.len(), 1);
        let next = state.candidates[0];
        assert_eq!(next.sequence, sequence + 1);
        assert_eq!(next.action, ResponseAction::Continue);
        assert!(next.sequence < state.response.as_ref().unwrap().started_at);
        assert_ne!(next.features[17].value & (1 << 63), 0);
        selected_next.push(next.token);
    }
    assert_eq!(selected_next, vec![69, 70]);
}

#[test]
fn response_query_and_postings_stay_fixed_and_evicted_sources_are_not_read() {
    let model = fixture(8);
    let mut session = prefix(&model, &[BOS, 67, 68, 69, 67]);
    let before = session.memory.as_ref().unwrap().response.clone().unwrap();
    let capacities = {
        let memory = session.memory.as_ref().unwrap();
        (
            memory.candidates.capacity(),
            memory.composed.capacity(),
            memory.composition_features.capacity(),
            before.queries.capacity(),
            before.references.capacity(),
        )
    };
    for _ in 0..20 {
        session.predict(&model).unwrap();
        session.observe(&model, 67).unwrap();
        let memory = session.memory.as_ref().unwrap();
        let response = memory.response.as_ref().unwrap();
        assert_eq!(response.queries, before.queries);
        assert_eq!(response.references, before.references);
        assert_eq!(response.query_pose, before.query_pose);
        assert_eq!(response.query_phases, before.query_phases);
        assert_eq!(
            (
                memory.candidates.capacity(),
                memory.composed.capacity(),
                memory.composition_features.capacity(),
                response.queries.capacity(),
                response.references.capacity()
            ),
            capacities
        );
    }
    session.predict(&model).unwrap();
    assert!(session.memory.as_ref().unwrap().candidates.is_empty());
    assert!(session.work.memory_stale_rejections > 0);
    session.observe(&model, EOS).unwrap();
    let response = session.state().response.unwrap();
    assert!(!response.active);
    assert_eq!(response.last_action, ResponseAction::Stop);
    assert_eq!(response.selected_sequence, None);
    session.begin_response(&model).unwrap();
    let response = session.state().response.unwrap();
    assert!(response.active);
    assert_eq!(response.steps, 0);
    assert!(response.started_at > before.started_at);
}

#[test]
fn response_control_removes_state_without_removing_the_memory_reader() {
    let model = fixture(16);
    let mut session = model.session(Control::ResponseStateDisabled).unwrap();
    for token in [BOS, 67, 68, 69, 70, 67] {
        session.observe(&model, token).unwrap();
    }
    session.begin_response(&model).unwrap();
    session.predict(&model).unwrap();
    assert!(!session.state().response.unwrap().active);
    assert!(session.response_decision().is_none());
    assert!(!session.memory.as_ref().unwrap().composed.is_empty());
    assert_eq!(session.work.response_query_captures, 0);
}

#[test]
fn advancing_response_geometry_changes_scores_inputs_without_changing_admission() {
    let frozen = fixture(16);
    let mut advancing = frozen.clone();
    advancing
        .memory_read
        .as_mut()
        .unwrap()
        .config
        .advance_response_path = true;
    advancing.refresh_identity().unwrap();
    assert_ne!(frozen.artifact_cid(), advancing.artifact_cid());
    let config = &frozen.memory_read.as_ref().unwrap().config;
    assert!(serde_json::to_value(config)
        .unwrap()
        .get("advance_response_path")
        .is_none());
    let tokens = [BOS, 67, 68, 69, 67, 70, 67];
    let mut left = prefix(&frozen, &tokens);
    let mut right = prefix(&advancing, &tokens);
    let captured = right.state().response.unwrap();
    left.predict(&frozen).unwrap();
    right.predict(&advancing).unwrap();
    let left_rows = &left.memory.as_ref().unwrap().candidates;
    let right_rows = &right.memory.as_ref().unwrap().candidates;
    assert_eq!(left_rows.len(), right_rows.len());
    assert!(left_rows
        .iter()
        .zip(right_rows)
        .all(|(a, b)| a.features == b.features));

    // Same observed text and captured evidence, with a changed endpoint only.
    left.observe(&frozen, 71).unwrap();
    right.observe(&advancing, 71).unwrap();
    left.predict(&frozen).unwrap();
    right.predict(&advancing).unwrap();
    let left_rows = &left.memory.as_ref().unwrap().candidates;
    let right_rows = &right.memory.as_ref().unwrap().candidates;
    assert!(!left_rows.is_empty());
    assert_eq!(left_rows.len(), right_rows.len());
    assert!(left_rows.iter().zip(right_rows).all(|(a, b)| {
        a.sequence == b.sequence
            && a.token == b.token
            && a.action == b.action
            && a.features[..5] == b.features[..5]
            && a.features[16..] == b.features[16..]
    }));
    assert!(left_rows
        .iter()
        .zip(right_rows)
        .any(|(a, b)| a.features[5..16] != b.features[5..16]));
    let after = right.state().response.unwrap();
    assert_eq!(after.query_pose, captured.query_pose);
    assert_eq!(after.query_phases, captured.query_phases);
    assert_eq!(
        right
            .memory
            .as_ref()
            .unwrap()
            .response
            .as_ref()
            .unwrap()
            .references,
        left.memory
            .as_ref()
            .unwrap()
            .response
            .as_ref()
            .unwrap()
            .references
    );
}
