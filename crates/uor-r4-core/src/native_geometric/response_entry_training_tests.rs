//! Real fitting tests distinguish learned Enter from equal-token Base output.
use super::completion_types::{CompletionModel, COMPLETION_SCHEMA};
use super::numeral::NUMERAL_CODEC;
use super::response_entry_types::*;
use super::value_types::{ValueModel, LEXEME_VALUE_SCHEMA, VALUES};
use super::*;
use std::sync::OnceLock;

fn no_write_baseline() -> Model {
    let catalog = [Document {
        id: "entry-fit-catalog".into(),
        text: "source value query reply total done unknown 13 4 17 18 ; : .\n".into(),
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
    let mut baseline = trainer.compile().unwrap();
    baseline.values = Some(ValueModel {
        schema: LEXEME_VALUE_SCHEMA.into(),
        codec: NUMERAL_CODEC.into(),
        capacity: VALUES,
        rows: Vec::new(),
        continuation_score: 0,
        fit_config: [0; 4],
        training: Vec::new(),
    });
    baseline.refresh_identity().unwrap();
    baseline.completion = Some(CompletionModel {
        schema: COMPLETION_SCHEMA.into(),
        baseline_artifact: baseline.artifact_cid.clone(),
        rows: Vec::new(),
        global_postings: Vec::new(),
        fit_config: [0; 4],
        fit_positions: 0,
        training: Vec::new(),
    });
    baseline.refresh_identity().unwrap();
    baseline.validate().unwrap();
    baseline
}

fn example(id: &str, prompt: &str, response: &str) -> ValueExample {
    ValueExample {
        id: id.into(),
        prompt: prompt.into(),
        response: response.into(),
    }
}

#[test]
fn native_response_entry_fitting_bootstraps_actual_selected_entry_and_freezes_numeric_baseline() {
    let baseline = no_write_baseline();
    let examples = [example("entry-fit-short", "source 17; query:", " done.\n")];
    let (model, report) = baseline
        .fit_response_entry(&examples, ResponseEntryFitConfig::default())
        .unwrap();
    assert_eq!(report.entry_positions, 1);
    assert_eq!(report.entry_fit_correct, 1);
    assert_eq!(report.entered_rollouts, 1);
    assert_eq!(report.entry_rollout_failures, 0);
    assert_eq!(report.final_entry_correct, 1);
    assert!(report.continuation_positions > 0);
    assert_eq!(report.final_exact_responses, 1);
    let mut session = model.session(Control::Full).unwrap();
    session.observe(&model, BOS).unwrap();
    for token in model.encode(&examples[0].prompt).unwrap() {
        session.observe(&model, token).unwrap();
    }
    session.begin_response(&model).unwrap();
    assert!(!session.response_entry.as_ref().unwrap().active);
    let first = session.predict(&model).unwrap();
    assert_eq!(
        session.response_entry_decision().unwrap().action,
        ResponseEntryAction::Enter
    );
    assert!(!session.response_entry.as_ref().unwrap().active);
    session.observe(&model, first.token).unwrap();
    assert!(session.response_entry.as_ref().unwrap().active);
    assert_eq!(session.work.values.derived_writes, 0);
    let mut restored_baseline = model.clone();
    restored_baseline.response_entry = None;
    restored_baseline.refresh_identity().unwrap();
    assert_eq!(
        restored_baseline.to_bytes().unwrap(),
        baseline.to_bytes().unwrap()
    );
}

#[test]
fn native_response_entry_fitting_skips_overlong_and_overbudget_responses_whole() {
    let baseline = no_write_baseline();
    let first = example("entry-bound-short", "source 17; query:", " done.\n");
    let positions = baseline.encode(&first.response).unwrap().len() + 1;
    let examples = [
        first,
        example("entry-bound-long", "source 18; query:", &"🚀".repeat(20)),
        example("entry-bound-budget", "source 19; query:", " unknown.\n"),
    ];
    let (_, report) = baseline
        .fit_response_entry(
            &examples,
            ResponseEntryFitConfig {
                max_positions: positions,
                ..ResponseEntryFitConfig::default()
            },
        )
        .unwrap();
    assert_eq!(report.examples, 3);
    assert_eq!(report.overlong_responses, 1);
    assert_eq!(report.position_limit_skips, 1);
    assert_eq!(report.entry_positions, 1);
    assert_eq!(
        report.entry_positions + report.continuation_positions,
        positions
    );
}

#[test]
fn native_response_entry_artifact_rejects_foreign_baseline_and_malformed_rows() {
    let (model, _) = no_write_baseline()
        .fit_response_entry(
            &[example("entry-shape", "source 17; query:", " done.\n")],
            ResponseEntryFitConfig::default(),
        )
        .unwrap();
    let mut bad = model.clone();
    bad.response_entry.as_mut().unwrap().baseline_artifact = "blake3:foreign".into();
    bad.refresh_identity().unwrap();
    assert!(bad.validate().is_err());
    let mut bad = model.clone();
    bad.response_entry.as_mut().unwrap().rows[0].feature.kind = 32;
    bad.refresh_identity().unwrap();
    assert!(bad.validate().is_err());
    let mut bad = model.clone();
    bad.response_entry
        .as_mut()
        .unwrap()
        .global_postings
        .push(BOS);
    bad.refresh_identity().unwrap();
    assert!(bad.validate().is_err());
    let mut bad = model.clone();
    bad.response_entry.as_mut().unwrap().fit_config[4] = 65;
    bad.refresh_identity().unwrap();
    assert!(bad.validate().is_err());
    assert!(Model::from_bytes(&model.to_bytes().unwrap()).is_ok());
}

fn numeric_fixture() -> &'static (Model, Model) {
    static MODELS: OnceLock<(Model, Model)> = OnceLock::new();
    MODELS.get_or_init(|| {
        let catalog = [Document {
            id: "entry-numeric-catalog".into(),
            text: "left = 13; right = 4; total: 17.\nreply: Unknown.\n Unknown.\n".into(),
        }];
        let mut trainer = Trainer::new(
            Config {
                context_tokens: 32,
                candidate_limit: 8,
                max_lexical_pieces: 128,
                ..Config::default()
            },
            &catalog,
        )
        .unwrap();
        trainer.train_documents(&catalog).unwrap();
        let count = trainer.compile().unwrap();
        let mut examples = Vec::new();
        for index in 0..4 {
            examples.push(example(
                &format!("entry-number-{index}"),
                &format!("left = {}; right = 4; total:", 13 + index),
                &format!("{}.\n", 17 + index),
            ));
            examples.push(example(
                &format!("entry-unknown-{index}"),
                &format!("left = {}; right = 4; reply:", 13 + index),
                " Unknown.\n",
            ));
        }
        let (typed, _) = count
            .fit_values_with_lexeme_cues(
                &examples,
                ValueFitConfig {
                    epochs: 64,
                    learning_rate: 0.25,
                    max_features: 4096,
                },
            )
            .unwrap();
        let (completion, _) = typed
            .fit_value_completion(&examples, ValueCompletionFitConfig::default())
            .unwrap();
        let (entry, report) = completion
            .fit_response_entry(&examples, ResponseEntryFitConfig::default())
            .unwrap();
        assert_eq!(report.numeric_examples, 4);
        assert_eq!(report.matched_numeric, 4);
        assert_eq!(report.upstream_failures, 0);
        assert_eq!(report.final_entry_correct, 4);
        (completion, entry)
    })
}

#[test]
fn native_response_entry_keeps_complete_numeric_output_and_both_prior_heads() {
    let (completion, entry) = numeric_fixture();
    for index in 0..4 {
        let prompt = format!("left = {}; right = 4; total:", 13 + index);
        let before = completion.generate(&prompt, 16, Control::Full).unwrap();
        let after = entry.generate(&prompt, 16, Control::Full).unwrap();
        assert_eq!(before.bytes, after.bytes);
        assert_eq!(before.token_ids, after.token_ids);
        assert_eq!(before.stop, after.stop);
        assert_eq!(
            serde_json::to_value(before.value_trace).unwrap(),
            serde_json::to_value(after.value_trace).unwrap()
        );
        assert_eq!(
            serde_json::to_value(before.completion_trace).unwrap(),
            serde_json::to_value(after.completion_trace).unwrap()
        );
    }
    let mut stripped = entry.clone();
    stripped.response_entry = None;
    stripped.refresh_identity().unwrap();
    assert_eq!(stripped.to_bytes().unwrap(), completion.to_bytes().unwrap());
}
