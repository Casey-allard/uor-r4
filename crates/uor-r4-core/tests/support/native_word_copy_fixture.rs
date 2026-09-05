// Shared real-fit fixture for core, allocation and HTTP continuity checks.
// The including module supplies the native module alias. These construction
// examples establish executable state laws, not development transfer quality.
use native::{
    Config, Document, Model, ResponseEntryFitConfig, Trainer,
    ValueCompletionFitConfig, ValueExample, ValueFitConfig,
};
use std::sync::OnceLock;

pub const COPY_PROMPT: &str =
    "left = 13; right = 4; fn identity(alpha: i32) -> i32 {\n    ";
pub const DUPLICATE_PROMPT: &str =
    "left = 13; right = 4; alpha ignored; fn identity(alpha: i32) -> i32 {\n    ";
pub const COPY_RESPONSE: &str = "alpha\n}\n";
pub const NUMERIC_PROMPT: &str = "left = 13; right = 4; total:";
static CONSTRUCTION: OnceLock<Vec<ValueExample>> = OnceLock::new();

pub fn fitted() -> &'static (Model, Model) {
    static MODELS: OnceLock<(Model, Model)> = OnceLock::new();
    MODELS.get_or_init(|| {
        let catalog = [Document {
            id: "copy-contract-catalog".into(),
            text: "left = 13; right = 4; total: 17.\nreply: Unknown.\n Unknown.\nfn identity(value: i32) -> i32 {\n    value\n}\n".into(),
        }];
        let mut trainer = Trainer::new(
            Config {
                context_tokens: 128,
                candidate_limit: 16,
                max_lexical_pieces: 128,
                ..Config::default()
            },
            &catalog,
        )
        .unwrap();
        trainer.train_documents(&catalog).unwrap();
        let mut examples = Vec::new();
        for index in 0..4 {
            for (label, tail, response) in [
                ("numeric", "total:", format!("{}.\n", 17 + index)),
                ("unknown", "reply:", " Unknown.\n".into()),
                (
                    "identity",
                    "fn identity(value: i32) -> i32 {\n    ",
                    "value\n}\n".into(),
                ),
            ] {
                examples.push(ValueExample {
                    id: format!("copy-contract-parent-{label}-{index}"),
                    prompt: format!("left = {}; right = 4; {tail}", 13 + index),
                    response,
                });
            }
        }
        let (typed, _) = trainer
            .compile()
            .unwrap()
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
        let (entry, _) = completion
            .fit_response_entry(&examples, ResponseEntryFitConfig::default())
            .unwrap();
        for (index, name) in ["alpha", "bravo", "cedar", "delta"].into_iter().enumerate() {
            examples.push(ValueExample {
                id: format!("copy-contract-name-{index}"),
                prompt: format!("left = 13; right = 4; fn identity({name}: i32) -> i32 {{\n    "),
                response: format!("{name}\n}}\n"),
            });
        }
        examples.push(ValueExample {
            id: "copy-contract-duplicate".into(),
            prompt: DUPLICATE_PROMPT.into(),
            response: COPY_RESPONSE.into(),
        });
        let (copy, _) = entry
            .fit_response_entry_copy(&examples, ResponseEntryFitConfig::default())
            .unwrap();
        let copy = Model::from_bytes(&copy.to_bytes().unwrap()).unwrap();
        CONSTRUCTION.set(examples).unwrap();
        (entry, copy)
    })
}

// The original fixture stays byte-for-byte unchanged; only these new focused
// tests request the separate completed-word suffix artifact.
#[allow(dead_code)]
pub fn fitted_completed_word() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        let (entry, _) = fitted();
        let mut examples = CONSTRUCTION.get().unwrap().clone();
        for name in ["reed", "payload"] {
            examples.push(ValueExample {
                id: format!("copy-completed-word-{name}"),
                prompt: format!("left = 13; right = 4; fn identity({name}: i32) -> i32 {{\n    "),
                response: format!("{name}\n}}\n"),
            });
        }
        let (model, _) = entry
            .fit_response_entry_copy_completed_word(&examples, ResponseEntryFitConfig::default())
            .unwrap();
        Model::from_bytes(&model.to_bytes().unwrap()).unwrap()
    })
}
