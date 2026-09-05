//! Focused typed-state checks. Deliberately set rows exercise causal mechanics;
//! the separately named fitting check uses only raw prompt/response examples.
use super::numeral::NUMERAL_CODEC;
use super::value_types::*;
use super::*;

fn baseline() -> Model {
    let catalog = [Document {
        id: "typed-value-test-catalog".into(),
        text: "left right total copy none 13 4 17 18 answer unknown result".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 32,
            candidate_limit: 16,
            max_lexical_pieces: 128,
            ..Config::default()
        },
        &catalog,
    )
    .unwrap();
    trainer.train_documents(&catalog).unwrap();
    trainer.compile().unwrap()
}

fn mechanical_model(action: ValueAction) -> Model {
    let mut model = baseline();
    model.values = Some(ValueModel {
        schema: VALUE_SCHEMA.into(),
        codec: NUMERAL_CODEC.into(),
        capacity: VALUES,
        rows: [ValueAction::Copy, ValueAction::Add]
            .into_iter()
            .enumerate()
            .map(|(index, candidate)| ValueRow {
                feature: ValueFeature {
                    kind: 0,
                    a: index as u64,
                    b: 0,
                },
                weight: if candidate == action { 4096 } else { -4096 },
            })
            .collect(),
        continuation_score: 4096,
        fit_config: [0; 4],
        training: Vec::new(),
    });
    model.refresh_identity().unwrap();
    model
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

#[test]
fn native_value_prediction_is_causal_and_mismatch_never_selects_a_target_source() {
    let model = mechanical_model(ValueAction::Add);
    let mut session = prefix(&model, "left = 13; right = 4; total:", Control::Full);
    let before = session.state();
    let first = session.predict(&model).unwrap();
    let decision = session.value_decision().unwrap();
    assert_eq!(decision.action, ValueAction::Add);
    assert_eq!(decision.value, 17);
    assert_ne!(decision.operands[0].id, decision.operands[1].id);
    assert_eq!(session.state(), before);
    assert_eq!(session.predict(&model).unwrap(), first);
    assert_eq!(session.value_decision(), Some(decision));
    assert_eq!(session.work.values.derived_writes, 0);
    assert_eq!(session.work.values.emission_commits, 0);

    let wrong = u32::from(b'9') + 2;
    session.observe(&model, wrong).unwrap();
    assert_eq!(session.work.values.derived_writes, 0);
    assert_eq!(session.work.values.emission_mismatches, 1);
    assert_eq!(session.value_decision(), None);
    assert!(session.values.as_ref().unwrap().emission.is_none());
    // Wrong observed bytes did not create a numeral or identify another pair.
    session.predict(&model).unwrap();
    let retry = session.value_decision().unwrap();
    assert_eq!(retry.value, decision.value);
    assert_eq!(retry.operands, decision.operands);
    assert_eq!(retry.write_id, decision.write_id);
    session.observe(&model, retry.token).unwrap();
    assert_eq!(session.work.values.derived_writes, 1);
    assert_eq!(session.work.values.emission_commits, 1);
    assert_eq!(session.state().values.unwrap().emission_cursor, Some(1));

    session.predict(&model).unwrap();
    assert_eq!(session.value_decision().unwrap().token, u32::from(b'7') + 2);
    session.observe(&model, wrong).unwrap();
    assert!(session.values.as_ref().unwrap().emission.is_none());
    assert_eq!(session.work.values.emission_commits, 1);
    assert_eq!(session.work.values.emission_mismatches, 2);
    session.predict(&model).unwrap();
    assert!(session.value_decision().is_none());
    let records = &session.values.as_ref().unwrap().records;
    assert_eq!(records.iter().filter(|record| record.derived).count(), 1);
    assert_eq!(records.last().unwrap().value, 17);
}

#[test]
fn native_value_equal_first_bytes_preserve_complete_derivation_identity() {
    let model = mechanical_model(ValueAction::Add);
    let mut first = prefix(&model, "left = 13; right = 4; total:", Control::Full);
    let mut second = prefix(&model, "left = 13; right = 5; total:", Control::Full);
    assert_eq!(
        first.predict(&model).unwrap().token,
        second.predict(&model).unwrap().token
    );
    let a = first.value_decision().unwrap();
    let b = second.value_decision().unwrap();
    assert_eq!(a.token, u32::from(b'1') + 2);
    assert_eq!((a.value, b.value), (17, 18));
    assert_ne!(a.operands, b.operands);
    first.observe(&model, a.token).unwrap();
    second.observe(&model, b.token).unwrap();
    first.predict(&model).unwrap();
    second.predict(&model).unwrap();
    assert_eq!(first.value_decision().unwrap().token, u32::from(b'7') + 2);
    assert_eq!(second.value_decision().unwrap().token, u32::from(b'8') + 2);
    assert_eq!(
        first.values.as_ref().unwrap().records.last().unwrap().value,
        17
    );
    assert_eq!(
        second
            .values
            .as_ref()
            .unwrap()
            .records
            .last()
            .unwrap()
            .value,
        18
    );
}

#[test]
fn native_value_signed_extrema_use_ordinary_tokens_and_complete_once() {
    let model = mechanical_model(ValueAction::Copy);
    for value in [i64::MIN, i64::MAX, -17, 0, 17] {
        let expected = value.to_string();
        let mut session = prefix(&model, &format!("copy {value}"), Control::Full);
        let mut tokens = Vec::new();
        for _ in expected.bytes() {
            let token = session.predict(&model).unwrap().token;
            let decision = session.value_decision().unwrap();
            assert_eq!(decision.value, value);
            assert_eq!(decision.action, ValueAction::Copy);
            assert_eq!(session.candidates().first().unwrap().token, token);
            tokens.push(token);
            session.observe(&model, token).unwrap();
        }
        assert_eq!(model.decode(&tokens).unwrap(), expected.as_bytes());
        assert_eq!(session.work.values.derived_writes, 1);
        assert_eq!(session.work.values.emission_commits, expected.len() as u64);
        assert!(session.values.as_ref().unwrap().emission.is_none());
        session.predict(&model).unwrap();
        assert!(session.value_decision().is_none());
    }
    // The lexical codec and byte emission may have different token counts,
    // while their decoded bytes remain the same actual output interface.
    let lexical = model.encode(" 17").unwrap();
    assert_eq!(lexical.len(), 1);
    assert_eq!(
        model.decode(&lexical).unwrap(),
        model.decode(&[34, 51, 57]).unwrap()
    );
}

#[test]
fn native_value_overflow_and_disabled_control_offer_no_derived_candidate() {
    let model = mechanical_model(ValueAction::Add);
    let mut overflow = prefix(
        &model,
        "left = 9223372036854775807; right = 1; total:",
        Control::Full,
    );
    overflow.predict(&model).unwrap();
    assert_eq!(overflow.value_decision(), None);
    assert_eq!(overflow.work.values.overflow_rejections, 2);
    assert_eq!(overflow.work.values.derived_writes, 0);
    for control in [Control::ValuesDisabled, Control::MemoryDisabled] {
        let mut disabled = prefix(&model, "left = 13; right = 4; total:", control);
        disabled.predict(&model).unwrap();
        assert_eq!(disabled.value_decision(), None);
        assert_eq!(disabled.work.values.proposals, 0);
        assert_eq!(disabled.work.values.derived_writes, 0);
    }
}

#[test]
fn native_value_absence_preserves_legacy_serialized_fields_and_roundtrip() {
    let model = baseline();
    let bytes = model.to_bytes().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(wire.get("values").is_none());
    assert_eq!(
        Model::from_bytes(&bytes).unwrap().to_bytes().unwrap(),
        bytes
    );
    let state = prefix(&model, "left = 13; right = 4; total:", Control::Full);
    assert!(state.values.is_none());
    assert!(serde_json::to_value(state.state())
        .unwrap()
        .get("values")
        .is_none());
    assert!(serde_json::to_value(state.work)
        .unwrap()
        .get("values")
        .is_none());
    let saved = state.checkpoint().unwrap();
    let checkpoint: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    assert_eq!(checkpoint["schema"], "uor-r4.native-geometric-session/1");
    assert_eq!(
        model.restore_session(&saved).unwrap().checkpoint().unwrap(),
        saved
    );
}

#[test]
fn native_value_fit_uses_raw_responses_and_emits_an_absent_sum() {
    let baseline = baseline();
    let source = (0..12)
        .flat_map(|index| {
            let a = 11 + index;
            let b = 3 + index;
            [
                ValueExample {
                    id: format!("typed-sum-{index}"),
                    prompt: format!("left = {a}; right = {b}; total:"),
                    response: (a + b).to_string(),
                },
                ValueExample {
                    id: format!("typed-no-write-{index}"),
                    prompt: format!("left = {a}; right = {b}; none:"),
                    response: "unknown".into(),
                },
            ]
        })
        .collect::<Vec<_>>();
    let (learned, report) = baseline
        .fit_values(
            &source,
            ValueFitConfig {
                epochs: 96,
                learning_rate: 0.25,
                max_features: 4096,
            },
        )
        .unwrap();
    assert_eq!(report.numeric_targets, 12);
    assert_eq!(report.reachable_numeric_targets, 12);
    assert_eq!(report.no_write_targets, 12);
    assert!(report.learned_features > 0);
    assert!(report.continuation_positions > 0);
    assert!(baseline.values.is_none());
    let learned = Model::from_bytes(&learned.to_bytes().unwrap()).unwrap();
    let mut correct_sums = 0;
    let mut correct_no_writes = 0;
    for example in &source {
        let mut session = prefix(&learned, &example.prompt, Control::Full);
        session.predict(&learned).unwrap();
        if example.response == "unknown" {
            correct_no_writes += usize::from(session.value_decision().is_none());
            continue;
        }
        let mut output = Vec::new();
        let mut decisions = Vec::new();
        for _ in example.response.bytes() {
            let prediction = session.predict(&learned).unwrap();
            let Some(decision) = session.value_decision() else {
                break;
            };
            decisions.push(decision);
            output.push(prediction.token);
            session.observe(&learned, prediction.token).unwrap();
        }
        if learned.decode(&output).unwrap() == example.response.as_bytes() {
            correct_sums += 1;
            assert!(decisions
                .iter()
                .all(|decision| decision.action == ValueAction::Add));
            assert_eq!(session.work.values.derived_writes, 1);
        }
    }
    // This is a finite construction-fitting integration smoke, not held-out
    // capability evidence or a claim of a geometric contribution.
    assert_eq!(correct_sums, 12);
    assert_eq!(correct_no_writes, 12);
}

#[test]
fn native_value_fit_preserves_canonical_spelling_and_terminal_punctuation() {
    let baseline = baseline();
    let examples = [
        ("canonical", "17"),
        ("plus", "+17"),
        ("zero", "017"),
        ("period", "17."),
    ]
    .into_iter()
    .map(|(id, response)| ValueExample {
        id: format!("typed-spelling-{id}"),
        prompt: format!("{id}: left = 13; right = 4; total:"),
        response: response.into(),
    })
    .collect::<Vec<_>>();
    let (_, report) = baseline
        .fit_values(
            &examples,
            ValueFitConfig {
                epochs: 4,
                learning_rate: 0.25,
                max_features: 4096,
            },
        )
        .unwrap();
    assert_eq!(report.numeric_targets, 4);
    // 17 is the generated numeral prefix; the terminal period remains an
    // ordinary-token responsibility. +17 and 017 are different output bytes.
    assert_eq!(report.reachable_numeric_targets, 2);
    assert_eq!(report.no_write_targets, 0);
}

#[test]
fn native_value_document_boundaries_do_not_join_numerals() {
    let model = mechanical_model(ValueAction::Copy);
    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, BOS).unwrap();
    session.observe(&model, u32::from(b'1') + 2).unwrap();
    session.observe(&model, EOS).unwrap();
    session.observe(&model, u32::from(b'2') + 2).unwrap();
    session.begin_response(&model).unwrap();
    let values: Vec<_> = session
        .values
        .as_ref()
        .unwrap()
        .records
        .iter()
        .map(|r| r.value)
        .collect();
    assert_eq!(values, vec![1, 2]);
}
