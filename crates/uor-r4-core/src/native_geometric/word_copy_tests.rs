//! Actual fitted-copy state laws, provenance and persistence boundaries.
use super::response_entry_types::{RESPONSE_ENTRY_SCHEMA, RESPONSE_ENTRY_STEPS};
use super::word_copy_types::{WordCopyAction, WordCopyProgress};
use super::*;
use serde_json::json;

mod fixture {
    use crate::native_geometric as native;
    include!("../../tests/support/native_word_copy_fixture.rs");
}

fn prefix(model: &Model, prompt: &str, control: Control) -> Session {
    let mut session = model.session(control).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(prompt).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    session
}

fn begin_copy(model: &Model, prompt: &str) -> Session {
    let mut session = prefix(model, prompt, Control::Full);
    let first = session.predict(model).unwrap();
    let selected = session
        .word_copy_decision()
        .expect("fitted occurrence selected");
    assert_eq!(selected.action, WordCopyAction::Start);
    assert_eq!(first.token, u32::from(b'a') + 2);
    session.observe(model, first.token).unwrap();
    session
}

fn complete_word(model: &Model, name: &str) -> Session {
    let prompt = format!("left = 13; right = 4; fn identity({name}: i32) -> i32 {{\n    ");
    let mut session = prefix(model, &prompt, Control::Full);
    for (index, byte) in name.bytes().enumerate() {
        let next = session.predict(model).unwrap();
        assert_eq!(next.token, u32::from(byte) + 2);
        assert_eq!(
            session.word_copy_decision().unwrap().action,
            if index == 0 {
                WordCopyAction::Start
            } else {
                WordCopyAction::Byte
            }
        );
        session.observe(model, next.token).unwrap();
    }
    assert_eq!(
        session.word_copy.as_ref().unwrap().progress,
        WordCopyProgress::Complete
    );
    session
}

fn suffix_features(model: &Model, session: &Session) -> Vec<Feature> {
    let (features, len) = session.word_copy.as_ref().unwrap().continuation_features(
        model,
        session.response_entry.as_ref().unwrap(),
        session.values.as_ref().unwrap(),
        Control::Full,
        &mut WordCopyWork::default(),
    );
    features[..len].to_vec()
}

#[test]
fn native_word_copy_completed_suffix_retains_observed_geometry_without_word_shape() {
    let model = fixture::fitted_completed_word();
    let mut short = complete_word(model, "reed");
    let mut long = complete_word(model, "payload");
    let short_state = short.state();
    let long_state = long.state();
    let initial = suffix_features(model, &short);
    assert!(!initial.is_empty());
    assert_eq!(initial, suffix_features(model, &long));
    assert_eq!(short.state(), short_state);
    assert_eq!(long.state(), long_state);
    assert_ne!(
        short.response_entry.as_ref().unwrap().steps,
        long.response_entry.as_ref().unwrap().steps
    );
    let short_value = short.values.as_ref().unwrap();
    let long_value = long.values.as_ref().unwrap();
    assert_ne!(
        (short_value.pose, short_value.phases),
        (long_value.pose, long_value.phases)
    );
    let next = short.predict(model).unwrap();
    assert_eq!(long.predict(model).unwrap().token, next.token);
    assert_ne!(next.token, EOS);
    assert_eq!(initial, suffix_features(model, &short));
    assert_eq!(initial, suffix_features(model, &long));
    short.observe(model, next.token).unwrap();
    long.observe(model, next.token).unwrap();
    let progressed = suffix_features(model, &short);
    assert_eq!(progressed, suffix_features(model, &long));
    let geometry = |features: &[Feature]| {
        features
            .iter()
            .filter(|feature| feature.kind >= 22)
            .copied()
            .collect::<Vec<_>>()
    };
    assert_ne!(geometry(&initial), geometry(&progressed));
    assert_eq!(
        short.word_copy.as_ref().unwrap().progress,
        WordCopyProgress::Complete
    );
    assert_eq!(
        long.word_copy.as_ref().unwrap().progress,
        WordCopyProgress::Complete
    );
    assert_eq!(
        short.state().word_copy.unwrap().storage_bytes,
        short_state.word_copy.unwrap().storage_bytes
    );
}

#[test]
fn native_word_copy_completed_suffix_restores_at_the_actual_word_end() {
    let model = fixture::fitted_completed_word();
    let mut original = complete_word(model, "alpha");
    let snapshot = original.checkpoint().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
    assert_eq!(wire["schema"], "uor-r4.native-geometric-session/6");
    assert_eq!(wire["word_copy"]["progress"], "complete");
    let mut restored = model.restore_session(&snapshot).unwrap();
    assert_eq!(original.state(), restored.state());
    assert_eq!(
        suffix_features(model, &original),
        suffix_features(model, &restored)
    );
    let mut suffix = Vec::new();
    for _ in 0..32 {
        let next = original.predict(model).unwrap();
        assert_eq!(restored.predict(model).unwrap(), next);
        assert_eq!(original.word_copy_decision(), restored.word_copy_decision());
        original.observe(model, next.token).unwrap();
        restored.observe(model, next.token).unwrap();
        assert_eq!(original.state(), restored.state());
        if next.token == EOS {
            assert_eq!(model.decode(&suffix).unwrap(), b"\n}\n");
            assert!(original.word_copy.as_ref().unwrap().origin.is_none());
            return;
        }
        suffix.push(next.token);
    }
    panic!("completed-word suffix did not reach EOS within its bound");
}

#[test]
fn native_word_copy_completed_suffix_preserves_default_artifact_identity() {
    let (_, old) = fixture::fitted();
    let bytes = old.to_bytes().unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire["response_entry"]["copy"]
        .get("completed_word_suffix")
        .is_none());
    assert!(
        !old.response_entry
            .as_ref()
            .unwrap()
            .copy
            .as_ref()
            .unwrap()
            .completed_word_suffix
    );
    wire["response_entry"]["copy"]["completed_word_suffix"] = json!(false);
    let explicit_default = Model::from_bytes(&serde_json::to_vec(&wire).unwrap()).unwrap();
    explicit_default.validate().unwrap();
    assert_eq!(explicit_default.to_bytes().unwrap(), bytes);
    let session = complete_word(old, "alpha");
    let (inherited, len) = session.response_entry.as_ref().unwrap().features(
        old,
        session.values.as_ref().unwrap(),
        Control::Full,
        &mut CompletionWork::default(),
    );
    assert_eq!(suffix_features(old, &session), inherited[..len]);
    let completed = fixture::fitted_completed_word();
    assert!(
        completed
            .response_entry
            .as_ref()
            .unwrap()
            .copy
            .as_ref()
            .unwrap()
            .completed_word_suffix
    );
    assert_ne!(completed.artifact_cid, old.artifact_cid);
    assert_eq!(old.to_bytes().unwrap(), bytes);
    assert_eq!(
        Model::from_bytes(&completed.to_bytes().unwrap())
            .unwrap()
            .to_bytes()
            .unwrap(),
        completed.to_bytes().unwrap()
    );
}

#[test]
fn native_word_copy_fitted_selection_commits_observations_without_rescoring_words() {
    let (_, model) = fixture::fitted();
    let mut session = prefix(model, fixture::COPY_PROMPT, Control::Full);
    let frozen = session.values.as_ref().unwrap().lexemes;
    let before = session.word_copy.as_ref().unwrap().progress;
    let first = session.predict(model).unwrap();
    let selected = session.word_copy_decision().unwrap();
    assert_eq!(selected.action, WordCopyAction::Start);
    assert_eq!(selected.cursor, 0);
    assert_eq!(first.token, u32::from(b'a') + 2);
    assert_eq!(session.predict(model).unwrap(), first);
    assert_eq!(session.word_copy_decision().unwrap(), selected);
    assert_eq!(session.word_copy.as_ref().unwrap().progress, before);
    assert!(session.word_copy.as_ref().unwrap().origin.is_none());
    assert!(!session.response_entry.as_ref().unwrap().active);
    session.observe(model, first.token).unwrap();
    assert_eq!(
        session.word_copy.as_ref().unwrap().origin,
        Some(selected.word_index)
    );
    assert_eq!(
        session.word_copy.as_ref().unwrap().progress,
        WordCopyProgress::Emitting { cursor: 1 }
    );
    let candidates = session.work.word_copy.word_candidates;
    let typed = session.work.values;
    for (offset, byte) in b"lpha".iter().copied().enumerate() {
        let next = session.predict(model).unwrap();
        assert_eq!(next.token, u32::from(byte) + 2);
        let decision = session.word_copy_decision().unwrap();
        assert_eq!(decision.action, WordCopyAction::Byte);
        assert_eq!(decision.cursor as usize, offset + 1);
        assert_eq!(decision.word_index, selected.word_index);
        assert_eq!(decision.source_end, selected.source_end);
        assert_eq!(decision.source_byte_end, selected.source_byte_end);
        assert_eq!(session.predict(model).unwrap(), next);
        session.observe(model, next.token).unwrap();
    }
    assert_eq!(
        session.word_copy.as_ref().unwrap().progress,
        WordCopyProgress::Complete
    );
    assert_eq!(session.work.word_copy.word_candidates, candidates);
    assert_eq!(session.work.values.proposals, typed.proposals);
    assert_eq!(session.work.values.additions, typed.additions);
    assert_eq!(session.values.as_ref().unwrap().lexemes, frozen);
    assert_eq!(session.work.values.derived_writes, 0);
    let output = model
        .generate(fixture::COPY_PROMPT, 32, Control::Full)
        .unwrap();
    assert_eq!(output.bytes, fixture::COPY_RESPONSE.as_bytes());
}

#[test]
fn native_word_copy_mismatch_preserves_origin_and_disables_copy_suffix() {
    let (_, model) = fixture::fitted();
    for predicted in [false, true] {
        let mut session = prefix(model, fixture::COPY_PROMPT, Control::Full);
        if predicted {
            session.predict(model).unwrap();
        }
        session.observe(model, u32::from(b'?') + 2).unwrap();
        assert!(session.word_copy.as_ref().unwrap().origin.is_none());
        assert!(!session.response_entry.as_ref().unwrap().active);
        model
            .restore_session(&session.checkpoint().unwrap())
            .unwrap();
    }

    let mut session = begin_copy(model, fixture::COPY_PROMPT);
    let origin = session.word_copy.as_ref().unwrap().origin;
    let frozen = session.values.as_ref().unwrap().lexemes;
    session.predict(model).unwrap();
    session.observe(model, u32::from(b'?') + 2).unwrap();
    assert_eq!(session.word_copy.as_ref().unwrap().origin, origin);
    assert_eq!(
        session.word_copy.as_ref().unwrap().progress,
        WordCopyProgress::Aborted
    );
    session = model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
    for byte in b"lpha" {
        session.predict(model).unwrap();
        assert!(session.word_copy_decision().is_none());
        session.observe(model, u32::from(*byte) + 2).unwrap();
        assert_eq!(session.word_copy.as_ref().unwrap().origin, origin);
        assert_eq!(
            session.word_copy.as_ref().unwrap().progress,
            WordCopyProgress::Aborted
        );
    }
    assert_eq!(session.values.as_ref().unwrap().lexemes, frozen);
    model
        .restore_session(&session.checkpoint().unwrap())
        .unwrap();
}

#[test]
fn native_word_copy_snapshot_restores_mid_copy_and_rejects_changed_cursor_or_origin() {
    let (_, model) = fixture::fitted();
    let mut original = begin_copy(model, fixture::DUPLICATE_PROMPT);
    let selected = original.word_copy.as_ref().unwrap().origin.unwrap();
    let lexemes = original.values.as_ref().unwrap().lexemes.as_ref().unwrap();
    let duplicates = lexemes.queries[..lexemes.query_len]
        .iter()
        .enumerate()
        .filter(|(_, word)| &word.bytes[..usize::from(word.len)] == b"alpha")
        .map(|(index, word)| (index as u8, word.end, word.byte_end))
        .collect::<Vec<_>>();
    assert_eq!(duplicates.len(), 2);
    assert_ne!(duplicates[0].2, duplicates[1].2);
    let other = duplicates.iter().find(|word| word.0 != selected).unwrap().0;
    let snapshot = original.checkpoint().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
    assert_eq!(wire["schema"], "uor-r4.native-geometric-session/6");
    assert!(wire["word_copy"].get("pending").is_none());
    for (field, value) in [
        ("origin", json!(other)),
        ("origin", json!(255)),
        ("progress", json!({"emitting": {"cursor": 0}})),
        ("progress", json!({"emitting": {"cursor": 32}})),
        ("progress", json!("complete")),
        ("pending", serde_json::Value::Null),
    ] {
        let mut bad = wire.clone();
        bad["word_copy"][field] = value;
        assert!(
            model
                .restore_session(&serde_json::to_vec(&bad).unwrap())
                .is_err(),
            "{field}"
        );
    }
    let mut bad = wire.clone();
    bad.as_object_mut().unwrap().remove("word_copy");
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());
    let mut bad = wire;
    bad["response_entry"]["last"] = json!(u32::from(b'?') + 2);
    assert!(model
        .restore_session(&serde_json::to_vec(&bad).unwrap())
        .is_err());

    let mut restored = model.restore_session(&snapshot).unwrap();
    for _ in 0..32 {
        let next = original.predict(model).unwrap();
        assert_eq!(restored.predict(model).unwrap(), next);
        assert_eq!(restored.word_copy_decision(), original.word_copy_decision());
        original.observe(model, next.token).unwrap();
        restored.observe(model, next.token).unwrap();
        assert_eq!(original.state(), restored.state());
        if next.token == EOS {
            assert!(original.word_copy.as_ref().unwrap().origin.is_none());
            return;
        }
    }
    panic!("fitted continuation did not reach EOS within its bound");
}

#[test]
fn native_word_copy_snapshot_preserves_an_active_inherited_lexical_entry() {
    let (_, model) = fixture::fitted();
    let mut original = prefix(model, "left = 13; right = 4; reply:", Control::Full);
    let first = original.predict(model).unwrap();
    assert_eq!(
        original.response_entry_decision().unwrap().action,
        ResponseEntryAction::Enter
    );
    assert!(original.word_copy_decision().is_none());
    original.observe(model, first.token).unwrap();
    assert!(original.response_entry.as_ref().unwrap().active);
    assert_eq!(
        original.word_copy.as_ref().unwrap().progress,
        WordCopyProgress::Idle
    );
    assert!(original.word_copy.as_ref().unwrap().origin.is_none());
    let snapshot = original.checkpoint().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
    assert_eq!(wire["schema"], "uor-r4.native-geometric-session/6");
    let mut restored = model.restore_session(&snapshot).unwrap();
    for _ in 0..32 {
        let next = original.predict(model).unwrap();
        assert_eq!(restored.predict(model).unwrap(), next);
        assert!(original.word_copy_decision().is_none());
        assert!(restored.word_copy_decision().is_none());
        original.observe(model, next.token).unwrap();
        restored.observe(model, next.token).unwrap();
        assert_eq!(original.state(), restored.state());
        if next.token == EOS {
            return;
        }
    }
    panic!("inherited lexical continuation did not reach EOS within its bound");
}

#[test]
fn native_word_copy_observed_eos_boundary_and_cap_clear_the_original_occurrence() {
    let (_, model) = fixture::fitted();
    let mut eos = begin_copy(model, fixture::COPY_PROMPT);
    eos.observe(model, EOS).unwrap();
    assert!(eos.word_copy.as_ref().unwrap().origin.is_none());
    assert!(!eos.response_entry.as_ref().unwrap().active);
    model.restore_session(&eos.checkpoint().unwrap()).unwrap();

    let mut boundary = begin_copy(model, fixture::COPY_PROMPT);
    boundary.end_response(model).unwrap();
    assert!(boundary.word_copy.as_ref().unwrap().origin.is_none());
    assert!(boundary.word_copy_decision().is_none());
    model
        .restore_session(&boundary.checkpoint().unwrap())
        .unwrap();

    let mut capped = begin_copy(model, fixture::COPY_PROMPT);
    for _ in 1..RESPONSE_ENTRY_STEPS {
        capped.predict(model).unwrap();
        capped.observe(model, u32::from(b'?') + 2).unwrap();
    }
    assert!(!capped.response_entry.as_ref().unwrap().active);
    assert!(capped.word_copy.as_ref().unwrap().origin.is_none());
    assert_eq!(capped.work.response_entry.step_limits, 1);
    assert_eq!(capped.work.response_entry.stops, 0);
    model
        .restore_session(&capped.checkpoint().unwrap())
        .unwrap();
}

#[test]
fn native_word_copy_requires_the_parent_to_retain_lexical_words() {
    let (lexical_parent, copied) = fixture::fitted();
    // The older typed /1 model is valid without completion and retains no
    // words. Completion already requires /2, so an entry-capable /1 shape
    // cannot be a valid model. Exercise each new copy boundary explicitly.
    let mut parent = lexical_parent.clone();
    let mut entry = parent.response_entry.take().unwrap();
    let mut completion = parent.completion.take().unwrap();
    let values = parent.values.as_mut().unwrap();
    values.schema = super::value_types::VALUE_SCHEMA.into();
    values.rows.retain(|row| row.feature.kind < 16);
    parent.refresh_identity().unwrap();
    parent.validate().unwrap();
    assert!(parent
        .session(Control::Full)
        .unwrap()
        .values
        .unwrap()
        .lexemes
        .is_none());
    // Correct lineage identities ensure that the direct copy validator's
    // rejection concerns this missing dependency, not a changed parent hash.
    completion.baseline_artifact = parent.artifact_cid.clone();
    parent.completion = Some(completion);
    parent.refresh_identity().unwrap();
    entry.baseline_artifact = parent.artifact_cid.clone();
    parent.response_entry = Some(entry);
    parent.refresh_identity().unwrap();
    assert!(parent
        .completion
        .as_ref()
        .unwrap()
        .validate(&parent)
        .is_err());
    let source = [ValueExample {
        id: "copy-missing-lexeme-parent".into(),
        prompt: fixture::COPY_PROMPT.into(),
        response: fixture::COPY_RESPONSE.into(),
    }];
    assert!(parent
        .fit_response_entry_copy(&source, ResponseEntryFitConfig::default())
        .is_err());

    let mut copy = copied
        .response_entry
        .as_ref()
        .unwrap()
        .copy
        .clone()
        .unwrap();
    copy.baseline_artifact = parent.artifact_cid.clone();
    let entry = parent.response_entry.as_mut().unwrap();
    entry.schema = super::word_copy_types::RESPONSE_COPY_SCHEMA.into();
    entry.copy = Some(copy);
    parent.refresh_identity().unwrap();
    assert!(parent
        .response_entry
        .as_ref()
        .unwrap()
        .copy
        .as_ref()
        .unwrap()
        .validate(&parent)
        .is_err());
    assert!(parent.validate().is_err());
}

#[test]
fn native_word_copy_preserves_parent_and_respects_typed_precedence_and_controls() {
    let (parent, model) = fixture::fitted();
    let mut stripped = model.clone();
    let entry = stripped.response_entry.as_mut().unwrap();
    entry.copy = None;
    entry.schema = RESPONSE_ENTRY_SCHEMA.into();
    stripped.refresh_identity().unwrap();
    assert_eq!(stripped.to_bytes().unwrap(), parent.to_bytes().unwrap());
    let legacy = prefix(parent, fixture::COPY_PROMPT, Control::Full);
    let legacy_wire: serde_json::Value =
        serde_json::from_slice(&legacy.checkpoint().unwrap()).unwrap();
    assert_eq!(legacy_wire["schema"], "uor-r4.native-geometric-session/5");
    assert!(legacy_wire.get("word_copy").is_none());
    let numeric_parent = parent
        .generate(fixture::NUMERIC_PROMPT, 32, Control::Full)
        .unwrap();
    let numeric_copy = model
        .generate(fixture::NUMERIC_PROMPT, 32, Control::Full)
        .unwrap();
    assert_eq!(numeric_copy.bytes, numeric_parent.bytes);
    let mut numeric = prefix(model, fixture::NUMERIC_PROMPT, Control::Full);
    numeric.predict(model).unwrap();
    assert!(numeric.value_decision().is_some());
    assert!(numeric.word_copy_decision().is_none());
    for control in [
        Control::WordCopyDisabled,
        Control::ResponseEntryDisabled,
        Control::ValuesDisabled,
    ] {
        let mut session = prefix(model, fixture::COPY_PROMPT, control);
        session.predict(model).unwrap();
        assert!(session.word_copy_decision().is_none());
    }
    let mut full = prefix(model, fixture::COPY_PROMPT, Control::Full);
    let mut no_geometry = prefix(
        model,
        fixture::COPY_PROMPT,
        Control::WordCopyGeometryDisabled,
    );
    full.predict(model).unwrap();
    no_geometry.predict(model).unwrap();
    assert_eq!(
        full.work.word_copy.word_candidates,
        no_geometry.work.word_copy.word_candidates
    );
    assert_eq!(
        full.work.word_copy.bound_rejections,
        no_geometry.work.word_copy.bound_rejections
    );
    assert_eq!(no_geometry.work.word_copy.selector.h4_reads, 0);
    assert_eq!(no_geometry.work.word_copy.selector.orientation_reads, 0);
    assert_eq!(no_geometry.work.word_copy.selector.phase_subtractions, 0);
    let disabled = model
        .generate(fixture::COPY_PROMPT, 32, Control::WordCopyDisabled)
        .unwrap();
    let original = parent
        .generate(fixture::COPY_PROMPT, 32, Control::Full)
        .unwrap();
    assert_eq!(disabled.bytes, original.bytes);
    let long_name = "a".repeat(32);
    let long_prompt =
        format!("left = 13; right = 4; fn identity({long_name}: i32) -> i32 {{\n    ");
    let mut bounded = prefix(model, &long_prompt, Control::Full);
    bounded.predict(model).unwrap();
    assert!(bounded.work.word_copy.bound_rejections > 0);
    if let Some(selected) = bounded.word_copy_decision() {
        let word = bounded
            .values
            .as_ref()
            .unwrap()
            .lexemes
            .as_ref()
            .unwrap()
            .queries[usize::from(selected.word_index)];
        assert!(word.len < RESPONSE_ENTRY_STEPS);
    }
}
