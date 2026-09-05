//! Mechanical causal checks and separately learned raw-byte completion tests.
use super::completion_runtime;
use super::completion_types::*;
use super::numeral::NUMERAL_CODEC;
use super::value_types::{ValueFeature, ValueModel, ValueRow, LEXEME_VALUE_SCHEMA, VALUES};
use super::*;

fn baseline() -> Model {
    let documents = [Document {
        id: "completion-unit-count-construction".into(),
        text: "source value query without a number 9 17 101 176 ; : , . ( ) { }\n".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens: 64,
            candidate_limit: 16,
            max_lexical_pieces: 128,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let mut model = trainer.compile().unwrap();
    // A fixed Copy gate establishes typed mechanics for these unit fixtures;
    // the test below learns completion weights from raw responses separately.
    model.values = Some(ValueModel {
        schema: LEXEME_VALUE_SCHEMA.into(),
        codec: NUMERAL_CODEC.into(),
        capacity: VALUES,
        rows: vec![ValueRow {
            feature: ValueFeature {
                kind: 0,
                a: 0,
                b: 0,
            },
            weight: 4096,
        }],
        continuation_score: 4096,
        fit_config: [0; 4],
        training: Vec::new(),
    });
    model.refresh_identity().unwrap();
    model
}

fn mechanical(token: u32, score: i32) -> Model {
    let mut model = baseline();
    let baseline_artifact = model.artifact_cid.clone();
    model.completion = Some(CompletionModel {
        schema: COMPLETION_SCHEMA.into(),
        baseline_artifact,
        rows: vec![ScoreRow {
            feature: Feature { kind: 0, value: 0 },
            default_score: 0,
            scores: vec![TokenScore { token, score }],
            postings: vec![token],
        }],
        global_postings: vec![token],
        fit_config: [0; 4],
        fit_positions: 0,
        training: Vec::new(),
    });
    model.refresh_identity().unwrap();
    model
}

fn prefix(model: &Model, source: &str, control: Control) -> Session {
    let mut session = model.session(control).unwrap();
    session.observe(model, BOS).unwrap();
    for token in model.encode(source).unwrap() {
        session.observe(model, token).unwrap();
    }
    session.begin_response(model).unwrap();
    session
}

#[test]
fn native_completion_requires_the_observed_final_digit_and_uses_actual_suffix_history() {
    let model = mechanical(u32::from(b'.') + 2, 1000);
    let mut session = prefix(&model, "source 17; query:", Control::Full);
    let first = session.predict(&model).unwrap();
    assert_eq!(first.token, u32::from(b'1') + 2);
    assert!(!session.completion.as_ref().unwrap().active);
    assert_eq!(session.predict(&model).unwrap(), first);
    session.observe(&model, first.token).unwrap();
    assert!(!session.completion.as_ref().unwrap().active);
    let final_digit = session.predict(&model).unwrap();
    assert_eq!(final_digit.token, u32::from(b'7') + 2);
    session.observe(&model, final_digit.token).unwrap();
    let anchor = session.completion.as_ref().unwrap().anchor.unwrap();
    let values = session.values.as_ref().unwrap();
    assert_eq!(anchor.at_seen, values.seen);
    assert_eq!(anchor.pose, values.pose);
    assert_eq!(anchor.phases, values.phases);
    let writes = session.work.values.derived_writes;
    let state = *session.completion.as_ref().unwrap();
    let next = session.predict(&model).unwrap();
    assert_eq!(next.token, u32::from(b'.') + 2);
    assert_eq!(session.completion.as_ref().unwrap().steps, state.steps);
    assert_eq!(session.predict(&model).unwrap(), next);
    let wrong = u32::from(b'?') + 2;
    session.observe(&model, wrong).unwrap();
    let completion = session.completion.as_ref().unwrap();
    assert_eq!(completion.steps, 1);
    assert_eq!(completion.last, wrong);
    assert_eq!(completion.last_action, CompletionAction::Base);
    assert_eq!(session.work.completion.mismatches, 1);
    assert_eq!(session.work.values.derived_writes, writes);
    let (features, len) = completion.features(
        &model,
        session.values.as_ref().unwrap(),
        Control::Full,
        &mut CompletionWork::default(),
    );
    assert!(features[..len].contains(&Feature {
        kind: 1,
        value: u64::from(model.geometry.tokens[wrong as usize].prime)
    }));
    session.observe(&model, EOS).unwrap();
    assert!(!session.completion.as_ref().unwrap().active);
    assert!(session.completion.as_ref().unwrap().anchor.is_none());
    session.observe(&model, u32::from(b'x') + 2).unwrap();
    assert_eq!(
        session.completion.as_ref().unwrap().last_action,
        CompletionAction::Base
    );

    let mut interrupted = prefix(&model, "source 17; query:", Control::Full);
    let first = interrupted.predict(&model).unwrap();
    interrupted.observe(&model, first.token).unwrap();
    interrupted.predict(&model).unwrap();
    interrupted.observe(&model, wrong).unwrap();
    assert!(!interrupted.completion.as_ref().unwrap().active);
    assert_eq!(interrupted.work.completion.anchors, 0);
}

#[test]
fn native_completion_single_digit_stop_disabled_controls_and_step_cap_are_causal() {
    let model = mechanical(EOS, 1000);
    let mut session = prefix(&model, "source 9; query:", Control::Full);
    let digit = session.predict(&model).unwrap();
    session.observe(&model, digit.token).unwrap();
    assert!(session.completion.as_ref().unwrap().active);
    assert_eq!(session.work.completion.anchors, 1);
    let stop = session.predict(&model).unwrap();
    assert_eq!(stop.token, EOS);
    assert_eq!(
        session.completion.as_ref().unwrap().pending.unwrap().action,
        CompletionAction::Stop
    );
    session.observe(&model, EOS).unwrap();
    assert_eq!(session.work.completion.commits, 1);
    assert_eq!(session.work.completion.stops, 1);
    assert!(!session.completion.as_ref().unwrap().active);

    let mut disabled = prefix(&model, "source 9; query:", Control::ValueCompletionDisabled);
    let digit = disabled.predict(&model).unwrap();
    disabled.observe(&model, digit.token).unwrap();
    assert!(disabled.completion.as_ref().unwrap().active);
    disabled.predict(&model).unwrap();
    assert!(disabled.completion.as_ref().unwrap().pending.is_none());
    assert_eq!(disabled.work.completion.feature_queries, 0);
    let mut no_values = prefix(&model, "source 9; query:", Control::ValuesDisabled);
    no_values.predict(&model).unwrap();
    no_values.observe(&model, u32::from(b'9') + 2).unwrap();
    assert!(!no_values.completion.as_ref().unwrap().active);

    let model = mechanical(u32::from(b'.') + 2, 1000);
    let mut bounded = prefix(&model, "source 9; query:", Control::Full);
    let digit = bounded.predict(&model).unwrap();
    bounded.observe(&model, digit.token).unwrap();
    for _ in 0..COMPLETION_STEPS {
        let next = bounded.predict(&model).unwrap();
        assert_ne!(next.token, EOS);
        bounded.observe(&model, next.token).unwrap();
    }
    assert!(!bounded.completion.as_ref().unwrap().active);
    assert!(bounded.completion.as_ref().unwrap().anchor.is_none());
    assert_eq!(bounded.work.completion.step_limits, 1);
    assert_eq!(bounded.work.completion.commits, u64::from(COMPLETION_STEPS));
}

#[test]
fn native_completion_postings_and_sparse_score_work_have_fixed_bounds() {
    let features: [Feature; COMPLETION_FEATURES] = std::array::from_fn(|kind| Feature {
        kind: kind as u8,
        value: 0,
    });
    let head = CompletionModel {
        schema: COMPLETION_SCHEMA.into(),
        baseline_artifact: String::new(),
        fit_config: [0; 4],
        fit_positions: 0,
        training: Vec::new(),
        global_postings: (2..18).collect(),
        rows: features
            .iter()
            .enumerate()
            .map(|(index, feature)| {
                let postings: Vec<_> = (0..4)
                    .map(|offset| 2 + (index * 4 + offset) as u32)
                    .collect();
                ScoreRow {
                    feature: *feature,
                    default_score: 0,
                    scores: postings
                        .iter()
                        .map(|&token| TokenScore { token, score: 1 })
                        .collect(),
                    postings,
                }
            })
            .collect(),
    };
    let mut work = CompletionWork::default();
    let (tokens, len, rows, row_count) =
        completion_runtime::candidates(&head, &features, &mut work);
    assert_eq!(len, COMPLETION_CANDIDATES);
    assert_eq!(row_count, COMPLETION_FEATURES);
    assert_eq!(work.posting_offers, 80);
    assert_eq!(work.candidate_drops, 48);
    assert!(work.candidate_comparisons <= 1280);
    for &token in &tokens[..len] {
        completion_runtime::score_candidate(&head, token, &rows[..row_count], &mut work);
    }
    assert_eq!(work.score_lookups, 256);
    assert_eq!(work.candidate_evaluations, 16);
}

#[test]
fn native_completion_raw_response_fit_learns_suffix_and_stop_without_repairing_wrong_numbers() {
    let model = baseline();
    let mut examples = Vec::new();
    for index in 0..12 {
        let number = 101 + index;
        for (family, query, suffix) in [("prose", ':', ".\n"), ("rust", ',', ");\n}\n")] {
            examples.push(ValueExample {
                id: format!("completion-fit/{family}/{index}"),
                prompt: format!("source {number}; query{query}"),
                response: format!("{number}{suffix}"),
            });
        }
    }
    examples.push(ValueExample {
        id: "completion-fit/no-write".into(),
        prompt: "query without a number:".into(),
        response: "unknown\n".into(),
    });
    for (name, response) in [("short", "176.\n"), ("plus", "+17.\n"), ("zero", "017.\n")] {
        examples.push(ValueExample {
            id: format!("completion-fit/upstream-{name}"),
            prompt: format!("source 17; {name} query:"),
            response: response.into(),
        });
    }
    let original_values = serde_json::to_value(&model.values).unwrap();
    let original_bytes = model.to_bytes().unwrap();
    let (learned, report) = model
        .fit_value_completion(
            &examples,
            ValueCompletionFitConfig {
                epochs: 24,
                learning_rate: 0.1,
                max_positions: 4096,
            },
        )
        .unwrap();
    assert_eq!(report.matched_numeric, 24);
    assert_eq!(report.skipped_no_write, 1);
    assert_eq!(report.upstream_failures, 3);
    assert_eq!(report.positions, 108);
    assert_eq!(report.target_in_candidates, report.positions);
    assert_eq!(report.fit_correct, report.positions);
    assert_eq!(
        serde_json::to_value(&learned.values).unwrap(),
        original_values
    );
    assert_eq!(model.to_bytes().unwrap(), original_bytes);
    for example in &examples[..24] {
        let generated = learned
            .generate(&example.prompt, 32, Control::Full)
            .unwrap();
        assert_eq!(
            generated.bytes,
            example.response.as_bytes(),
            "{}",
            example.id
        );
        assert_eq!(generated.stop, "end_of_document");
        assert_eq!(generated.work.completion.stops, 1);
    }
}
