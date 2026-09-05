//! Mechanical state preservation and malformed-input checks. Synthetic
//! decisions establish causal persistence, not learned operator quality.
use super::*;
use crate::native_geometric::memory_types::{
    MemoryModel, MemoryReadFitConfig, OCCURRENCE_MEMORY_SCHEMA,
};
use crate::native_geometric::numeral::NUMERAL_CODEC;
use crate::native_geometric::value_types::{
    ValueAction, ValueDecision, ValueModel, LEXEME_VALUE_SCHEMA, VALUES, VALUE_SCHEMA,
};

fn fixture(response: bool) -> Model {
    let documents = [Document {
        id: "value-snapshot-fixture".into(),
        text: "x = 13; y = 4; value query answer".into(),
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
    model.memory_read = Some(MemoryModel {
        schema: if response {
            RESPONSE_MEMORY_SCHEMA
        } else {
            OCCURRENCE_MEMORY_SCHEMA
        }
        .into(),
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
    model.values = Some(ValueModel {
        schema: VALUE_SCHEMA.into(),
        codec: NUMERAL_CODEC.into(),
        capacity: VALUES,
        rows: Vec::new(),
        continuation_score: 1000,
        fit_config: [1, 0.1f64.to_bits(), 65536, 1],
        training: model.construction.clone(),
    });
    model.refresh_identity().unwrap();
    model
}

fn input(session: &mut Session, model: &Model, source: &[u8]) {
    for &byte in source {
        session.observe(model, u32::from(byte) + 2).unwrap();
    }
}

fn emitting(model: &Model) -> Session {
    let mut session = model.session(Control::Full).unwrap();
    session.observe(model, BOS).unwrap();
    input(&mut session, model, b"x = 13; y = 4; ");
    session.begin_response(model).unwrap();
    let values = session.values.as_mut().unwrap();
    assert_eq!(values.sources.len(), 2);
    values.pending = Some(ValueDecision {
        action: ValueAction::Add,
        operands: [values.sources[0], values.sources[1]],
        value: 17,
        write_id: values.next_id,
        token: 51,
        cursor: 0,
        score: 1,
        at_seen: values.seen,
    });
    session.observe(model, 51).unwrap();
    assert_eq!(session.values.as_ref().unwrap().emission.unwrap().cursor, 1);
    session
}

#[test]
fn native_value_checkpoint_preserves_committed_emission_and_optional_response_state() {
    for response in [false, true] {
        let model = fixture(response);
        let mut original = emitting(&model);
        original.predict(&model).unwrap();
        let bytes = original.checkpoint().unwrap();
        let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(wire["schema"], VALUE_SESSION_SCHEMA);
        assert_eq!(wire.get("response_memory").is_some(), response);
        assert!(wire["values"].get("pending").is_none());
        let mut restored = model.restore_session(&bytes).unwrap();
        assert!(restored.values.as_ref().unwrap().pending.is_none());
        assert_eq!(restored.checkpoint().unwrap(), bytes);
        let expected = original.predict(&model).unwrap();
        let actual = restored.predict(&model).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.token, 57);
        original.observe(&model, expected.token).unwrap();
        restored.observe(&model, actual.token).unwrap();
        assert_eq!(
            serde_json::to_value(&restored.values).unwrap(),
            serde_json::to_value(&original.values).unwrap()
        );
        assert!(restored.values.as_ref().unwrap().emission.is_none());
    }
}

#[test]
fn native_value_checkpoint_retains_numeric_records_and_open_scanner_after_token_eviction() {
    let model = fixture(false);
    let mut original = model.session(Control::Full).unwrap();
    original.observe(&model, BOS).unwrap();
    input(&mut original, &model, b"x = 13; y = 4; ");
    input(&mut original, &model, &b"z ".repeat(40));
    input(&mut original, &model, b"-922337203685477580");
    let bytes = original.checkpoint().unwrap();
    let mut restored = model.restore_session(&bytes).unwrap();
    assert_eq!(restored.checkpoint().unwrap(), bytes);
    assert!(restored.values.as_ref().unwrap().records.capacity() >= VALUES);
    assert!(restored.values.as_ref().unwrap().sources.capacity() >= VALUES);
    input(&mut original, &model, b"8; ");
    input(&mut restored, &model, b"8; ");
    assert_eq!(
        restored
            .values
            .as_ref()
            .unwrap()
            .records
            .last()
            .unwrap()
            .value,
        i64::MIN
    );
    assert_eq!(
        serde_json::to_value(&restored.values).unwrap(),
        serde_json::to_value(&original.values).unwrap()
    );
    original.begin_response(&model).unwrap();
    restored.begin_response(&model).unwrap();
    assert_eq!(restored.values.as_ref().unwrap().sources.len(), 3);
    assert_eq!(
        restored.checkpoint().unwrap(),
        original.checkpoint().unwrap()
    );
}

#[test]
fn native_value_checkpoint_rejects_malformed_scanner_records_queries_and_derivations() {
    let model = fixture(false);
    let original = emitting(&model);
    let value: serde_json::Value = serde_json::from_slice(&original.checkpoint().unwrap()).unwrap();
    let mutations: [fn(&mut serde_json::Value); 12] = [
        |v| v["values"]["seen"] = 999.into(),
        |v| v["values"]["recent_cursor"] = 32.into(),
        |v| v["values"]["records"][0]["value"] = 14.into(),
        |v| v["values"]["records"][0]["end"] = 999.into(),
        |v| v["values"]["queries"][0]["pose"] = 120.into(),
        |v| v["values"]["queries"][0]["token"] = u32::MAX.into(),
        |v| v["values"]["emission"]["decision"]["value"] = 18.into(),
        |v| v["values"]["emission"]["decision"]["operands"][0]["value"] = 14.into(),
        |v| v["values"]["emission"]["numeral"]["tokens"][0] = 52.into(),
        |v| v["values"]["emission"]["cursor"] = 20.into(),
        |v| v["values"]["scanner"]["digits"] = 255.into(),
        |v| v["values"]["pending"] = serde_json::Value::Null,
    ];
    for mutate in mutations {
        let mut invalid = value.clone();
        mutate(&mut invalid);
        assert!(model
            .restore_session(&serde_json::to_vec(&invalid).unwrap())
            .is_err());
    }
    let mut open = model.session(Control::Full).unwrap();
    input(&mut open, &model, b"-17");
    let mut invalid: serde_json::Value =
        serde_json::from_slice(&open.checkpoint().unwrap()).unwrap();
    invalid["values"]["scanner"]["accumulated"] = (-18).into();
    assert!(model
        .restore_session(&serde_json::to_vec(&invalid).unwrap())
        .is_err());
}

#[test]
fn native_value_checkpoint_preserves_old_schema_laws_and_rejects_cross_schema_values() {
    for response in [false, true] {
        let mut model = fixture(response);
        model.values = None;
        model.refresh_identity().unwrap();
        let mut session = model.session(Control::Full).unwrap();
        session.observe(&model, BOS).unwrap();
        let bytes = session.checkpoint().unwrap();
        let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            wire["schema"],
            if response {
                RESPONSE_SESSION_SCHEMA
            } else {
                LEGACY_SESSION_SCHEMA
            }
        );
        assert!(wire.get("values").is_none());
        assert_eq!(
            model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
            bytes
        );
        wire["values"] = serde_json::Value::Null;
        assert!(model
            .restore_session(&serde_json::to_vec(&wire).unwrap())
            .is_err());
    }
    let model = fixture(false);
    let session = emitting(&model);
    let mut wire: serde_json::Value =
        serde_json::from_slice(&session.checkpoint().unwrap()).unwrap();
    wire["schema"] = LEGACY_SESSION_SCHEMA.into();
    wire.as_object_mut().unwrap().remove("values");
    assert!(model
        .restore_session(&serde_json::to_vec(&wire).unwrap())
        .is_err());
}

#[test]
fn native_value_checkpoint_validates_old_derivation_after_operand_and_token_eviction() {
    let model = fixture(false);
    let mut session = emitting(&model);
    let next = session.predict(&model).unwrap();
    session.observe(&model, next.token).unwrap();
    session.end_response(&model).unwrap();
    // Retain result write 2 as the oldest of sixteen numeric records while
    // evicting both operand records 0 and 1 and all their raw token metadata.
    for value in 0..15 {
        input(&mut session, &model, format!(" {value};").as_bytes());
    }
    input(&mut session, &model, &b"z ".repeat(40));
    let bytes = session.checkpoint().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(wire["values"]["records"][0]["id"], 2);
    assert_eq!(
        wire["values"]["records"][0]["derivation"]["operand_ids"],
        serde_json::json!([0, 1])
    );
    assert_eq!(
        wire["values"]["records"][0]["derivation"]["operand_values"],
        serde_json::json!([13, 4])
    );
    assert_eq!(
        model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
        bytes
    );
    let mutations: [fn(&mut serde_json::Value); 6] = [
        |v| v["values"]["records"][0]["derivation"] = serde_json::Value::Null,
        |v| v["values"]["records"][0]["derived"] = false.into(),
        |v| v["values"]["records"][0]["derivation"]["operand_ids"][0] = 2.into(),
        |v| v["values"]["records"][0]["derivation"]["operand_values"][1] = 5.into(),
        |v| v["values"]["records"][0]["derivation"]["action"] = "copy".into(),
        |v| {
            v["values"]["records"][0]["derivation"]["operand_values"] =
                serde_json::json!([i64::MAX, 1])
        },
    ];
    for mutate in mutations {
        let mut invalid = wire.clone();
        mutate(&mut invalid);
        assert!(model
            .restore_session(&serde_json::to_vec(&invalid).unwrap())
            .is_err());
    }
}

#[test]
fn native_value_word_checkpoint_preserves_split_word_and_frozen_query_emission() {
    let mut model = fixture(false);
    model.values.as_mut().unwrap().schema = LEXEME_VALUE_SCHEMA.into();
    model.refresh_identity().unwrap();
    // Response boundaries separate words without inserting a source byte.
    let mut boundary = model.session(Control::Full).unwrap();
    input(&mut boundary, &model, b"suri");
    boundary.begin_response(&model).unwrap();
    boundary.end_response(&model).unwrap();
    input(&mut boundary, &model, b"tavi");
    let bytes = boundary.checkpoint().unwrap();
    assert_eq!(
        model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
        bytes
    );
    boundary.begin_response(&model).unwrap();
    let bytes = boundary.checkpoint().unwrap();
    assert_eq!(
        model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
        bytes
    );
    let mut original = model.session(Control::Full).unwrap();
    original.observe(&model, BOS).unwrap();
    input(&mut original, &model, b"left = 13; righ");
    let bytes = original.checkpoint().unwrap();
    let mut restored = model.restore_session(&bytes).unwrap();
    assert_eq!(restored.checkpoint().unwrap(), bytes);
    for session in [&mut original, &mut restored] {
        input(session, &model, b"t = 4; total");
        session.begin_response(&model).unwrap();
        let values = session.values.as_mut().unwrap();
        let words = values.lexemes.as_ref().unwrap();
        assert_eq!(words.query_len, 3);
        assert_eq!(&words.queries[0].bytes[..5], b"total");
        assert_eq!(&values.sources[1].lexical.unwrap()[0].bytes[..5], b"right");
        values.pending = Some(ValueDecision {
            action: ValueAction::Add,
            operands: [values.sources[0], values.sources[1]],
            value: 17,
            write_id: values.next_id,
            token: 51,
            cursor: 0,
            score: 1,
            at_seen: values.seen,
        });
        session.observe(&model, 51).unwrap();
    }
    assert_eq!(
        original.checkpoint().unwrap(),
        restored.checkpoint().unwrap()
    );
    let bytes = original.checkpoint().unwrap();
    let mut restored = model.restore_session(&bytes).unwrap();
    assert_eq!(restored.checkpoint().unwrap(), bytes);
    let expected = original.predict(&model).unwrap();
    let actual = restored.predict(&model).unwrap();
    assert_eq!(actual, expected);
    original.observe(&model, expected.token).unwrap();
    restored.observe(&model, actual.token).unwrap();
    assert_eq!(
        original.values.as_ref().unwrap().lexemes,
        restored.values.as_ref().unwrap().lexemes
    );
    for session in [&mut original, &mut restored] {
        session.end_response(&model).unwrap();
        input(session, &model, &b"padding ".repeat(40));
    }
    let bytes = original.checkpoint().unwrap();
    assert_eq!(
        model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
        bytes
    );
    // The inherited /4 snapshot rebuilds postings from retained tokens and
    // omits obsolete posting rows. A later prediction can therefore record
    // fewer stale rejections than uninterrupted execution. Preserve every
    // other checkpoint field and cost counter, including the full typed
    // state; only this physical-index diagnostic has a different replay law.
    let mut original_wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mut restored_wire: serde_json::Value =
        serde_json::from_slice(&restored.checkpoint().unwrap()).unwrap();
    let original_stale = original_wire["work"]
        .as_object_mut()
        .unwrap()
        .remove("memory_stale_rejections")
        .unwrap();
    let restored_stale = restored_wire["work"]
        .as_object_mut()
        .unwrap()
        .remove("memory_stale_rejections")
        .unwrap();
    assert!(restored_stale.as_u64().unwrap() <= original_stale.as_u64().unwrap());
    assert_eq!(restored_wire, original_wire);
    original.begin_response(&model).unwrap();
    restored.begin_response(&model).unwrap();
    assert_eq!(
        restored.predict(&model).unwrap(),
        original.predict(&model).unwrap()
    );
    assert_eq!(restored.value_decision(), original.value_decision());
}

#[test]
fn native_value_word_checkpoint_rejects_missing_fields_and_malformed_atoms_without_changing_v1() {
    let legacy = fixture(false);
    let legacy_session = emitting(&legacy);
    let legacy_bytes = legacy_session.checkpoint().unwrap();
    let mut legacy_wire: serde_json::Value = serde_json::from_slice(&legacy_bytes).unwrap();
    assert!(legacy_wire["values"].get("lexemes").is_none());
    assert!(legacy_wire["values"]["records"][0].get("lexical").is_none());
    assert_eq!(
        legacy
            .restore_session(&legacy_bytes)
            .unwrap()
            .checkpoint()
            .unwrap(),
        legacy_bytes
    );
    legacy_wire["values"]["lexemes"] = serde_json::Value::Null;
    assert!(legacy
        .restore_session(&serde_json::to_vec(&legacy_wire).unwrap())
        .is_err());
    let mut model = fixture(false);
    model.values.as_mut().unwrap().schema = LEXEME_VALUE_SCHEMA.into();
    model.refresh_identity().unwrap();
    let original = emitting(&model);
    let wire: serde_json::Value = serde_json::from_slice(&original.checkpoint().unwrap()).unwrap();
    assert_eq!(
        model
            .restore_session(&serde_json::to_vec(&wire).unwrap())
            .unwrap()
            .checkpoint()
            .unwrap(),
        original.checkpoint().unwrap()
    );
    let mutations: [fn(&mut serde_json::Value); 9] = [
        |v| {
            v["values"].as_object_mut().unwrap().remove("lexemes");
        },
        |v| {
            v["values"]["records"][0]
                .as_object_mut()
                .unwrap()
                .remove("lexical");
        },
        |v| v["values"]["sources"][0]["lexical"] = serde_json::Value::Null,
        |v| v["values"]["lexemes"]["source_bytes_seen"] = 999.into(),
        |v| v["values"]["lexemes"]["recent"][0]["len"] = 33.into(),
        |v| v["values"]["lexemes"]["recent"][0]["bytes"][31] = 1.into(),
        |v| v["values"]["lexemes"]["queries"][0]["end"] = 999.into(),
        |v| v["values"]["records"][0]["lexical"][0]["bytes"][0] = b'z'.into(),
        |v| v["values"]["emission"]["decision"]["operands"][0]["lexical"] = serde_json::Value::Null,
    ];
    for mutate in mutations {
        let mut invalid = wire.clone();
        mutate(&mut invalid);
        assert!(model
            .restore_session(&serde_json::to_vec(&invalid).unwrap())
            .is_err());
    }
    let mut pending = model.session(Control::Full).unwrap();
    input(&mut pending, &model, b"left = 13; righ");
    let mut invalid: serde_json::Value =
        serde_json::from_slice(&pending.checkpoint().unwrap()).unwrap();
    invalid["values"]["lexemes"]["scanner"]["pending"]["bytes"][0] = b'z'.into();
    assert!(model
        .restore_session(&serde_json::to_vec(&invalid).unwrap())
        .is_err());
}

#[test]
fn native_value_word_checkpoint_checks_early_whole_lexical_token_bytes() {
    let mut model = fixture(false);
    model.values.as_mut().unwrap().schema = LEXEME_VALUE_SCHEMA.into();
    model.refresh_identity().unwrap();
    let word_tokens = model.encode(" value").unwrap();
    assert_eq!(word_tokens.len(), 1);
    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, BOS).unwrap();
    session.observe(&model, word_tokens[0]).unwrap();
    for token in model.encode(" 13").unwrap() {
        session.observe(&model, token).unwrap();
    }
    session.begin_response(&model).unwrap();
    let values = session.values.as_ref().unwrap();
    let atom = values.lexemes.as_ref().unwrap().recent[0];
    assert_eq!(atom.end, 1);
    assert_eq!(atom.len, 5);
    assert_eq!(values.recent_len as u64, values.seen);
    let bytes = session.checkpoint().unwrap();
    assert_eq!(
        model.restore_session(&bytes).unwrap().checkpoint().unwrap(),
        bytes
    );

    // Change all copies together so shape, frozen-state and source-cue
    // equality still agree. The retained original token must reject them.
    let mut invalid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    invalid["values"]["lexemes"]["recent"][0]["bytes"][0] = b'w'.into();
    invalid["values"]["lexemes"]["queries"][0]["bytes"][0] = b'w'.into();
    invalid["values"]["records"][0]["lexical"][0]["bytes"][0] = b'w'.into();
    invalid["values"]["sources"][0]["lexical"][0]["bytes"][0] = b'w'.into();
    let error = model
        .restore_session(&serde_json::to_vec(&invalid).unwrap())
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("word payload differs from retained source bytes"));
}
