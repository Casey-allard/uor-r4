//! Authored raw-language relation questions and matched source interventions.
//!
//! Only the sentence bytes and question bytes are inference inputs. The source,
//! word and answer fields are an offline authoring oracle. Construction neither
//! consults the model/geometry nor filters examples by geometric separability.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub records: [Vec<u8>; 4],
    pub question: Vec<u8>,
    pub answer: Vec<u8>,
    pub expected_source: usize,
    pub expected_word: usize,
}

const TRAIN_NAMES: [&str; 16] = [
    "mira", "nora", "lena", "omar", "ivan", "sara", "hugo", "ella", "jane", "owen", "iris", "noah",
    "tony", "paul", "peter", "grace",
];
const DEVELOPMENT_NAMES: [&str; 8] = [
    "ruby", "felix", "clara", "dylan", "alice", "bruno", "helen", "oscar",
];
const TRAIN_VERBS: [&str; 4] = ["help", "visit", "call", "follow"];
const DEVELOPMENT_VERBS: [&str; 2] = ["guide", "trust"];

fn sentence(words: [&str; 4]) -> Vec<u8> {
    format!("{}.", words.join(" ")).into_bytes()
}

fn make_split(development: bool) -> Vec<Example> {
    let prefix = if development { "development" } else { "train" };
    let names: &[&str] = if development {
        &DEVELOPMENT_NAMES
    } else {
        &TRAIN_NAMES
    };
    let family_count = if development { 8 } else { 32 };
    let mut examples = Vec::with_capacity(family_count * 16);
    for index in 0..family_count {
        let family = format!("{prefix}-{index:03}");
        let [a, b, c, d, e] = std::array::from_fn(|offset| names[(index + offset) % names.len()]);
        let (v, w) = if development {
            let first = index / 4;
            (
                DEVELOPMENT_VERBS[first],
                DEVELOPMENT_VERBS[(first + 1) % DEVELOPMENT_VERBS.len()],
            )
        } else {
            let first = index / 8;
            (
                TRAIN_VERBS[first],
                TRAIN_VERBS[(first + 1) % TRAIN_VERBS.len()],
            )
        };
        let facts = [
            [a, "did", v, b],
            [b, "did", v, c],
            [a, "did", w, d],
            [d, "did", w, c],
        ];
        let permutation = if index / 4 % 2 == 0 {
            [0, 1, 2, 3]
        } else {
            [0, 2, 3, 1]
        };
        let slots: [usize; 4] =
            std::array::from_fn(|logical| (permutation[logical] + index % 4) % 4);
        let mut records: [Vec<u8>; 4] = std::array::from_fn(|_| Vec::new());
        for logical in 0..4 {
            records[slots[logical]] = sentence(facts[logical]);
        }
        // Each fact is queried in both directions. V/W and A/B/D overlaps
        // require the question's order and relation word jointly.
        let questions = [
            (v, b, 0, 0),
            (b, v, 1, 3),
            (w, d, 2, 0),
            (d, w, 3, 3),
            (a, v, 0, 3),
            (a, w, 2, 3),
            (v, c, 1, 0),
            (w, c, 3, 0),
        ];
        for (question_index, (left, right, logical, word)) in questions.into_iter().enumerate() {
            let question = format!("who did {left} {right}?").into_bytes();
            for changed in [false, true] {
                let mut context = records.clone();
                let answer = if changed {
                    let mut replacement = facts[logical];
                    replacement[word] = e;
                    context[slots[logical]] = sentence(replacement);
                    e
                } else {
                    facts[logical][word]
                };
                examples.push(Example {
                    id: format!(
                        "{family}-q{question_index}{}",
                        if changed { "-changed" } else { "" }
                    ),
                    family: family.clone(),
                    records: context,
                    question: question.clone(),
                    answer: answer.as_bytes().to_vec(),
                    expected_source: slots[logical],
                    expected_word: word,
                });
            }
        }
    }
    examples
}

fn words(bytes: &[u8]) -> Vec<&[u8]> {
    bytes
        .split(|byte| !byte.is_ascii_alphabetic())
        .filter(|word| !word.is_empty())
        .collect()
}

/// Offline oracle only; the serving path must not call this authoring logic.
fn check_example(example: &Example) -> Result<(), String> {
    let question = words(&example.question);
    if question.len() != 4
        || question[0] != b"who"
        || question[1] != b"did"
        || example.question.last() != Some(&b'?')
    {
        return Err(format!("{}: malformed four-word question", example.id));
    }
    let mut matches = Vec::new();
    for (slot, record) in example.records.iter().enumerate() {
        let fact = words(record);
        if fact.len() != 4 || fact[1] != b"did" || record.last() != Some(&b'.') {
            return Err(format!("{}: malformed four-word fact", example.id));
        }
        if fact[0] == question[2] && fact[2] == question[3] {
            matches.push((slot, 3, fact[3]));
        }
        if fact[2] == question[2] && fact[3] == question[3] {
            matches.push((slot, 0, fact[0]));
        }
    }
    if matches.len() != 1
        || matches[0]
            != (
                example.expected_source,
                example.expected_word,
                example.answer.as_slice(),
            )
    {
        return Err(format!(
            "{}: ambiguous or mismatched authored answer",
            example.id
        ));
    }
    for word in example
        .records
        .iter()
        .flat_map(|record| words(record))
        .chain(question)
    {
        if word.len() > 16 || !word.iter().all(u8::is_ascii_lowercase) {
            return Err(format!("{}: invalid alphabetic word", example.id));
        }
    }
    Ok(())
}

fn check_split(examples: &[Example], family_count: usize) -> Result<Value, String> {
    let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
    let mut inputs = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut slots = [0usize; 4];
    let mut answer_words = [0usize; 4];
    let mut family_contexts = BTreeSet::new();
    for example in examples {
        check_example(example)?;
        if !inputs.insert((&example.records, &example.question)) || !ids.insert(&example.id) {
            return Err(format!("{}: duplicate raw input or ID", example.id));
        }
        slots[example.expected_source] += 1;
        answer_words[example.expected_word] += 1;
        families.entry(&example.family).or_default().push(example);
    }
    if families.len() != family_count || examples.len() != family_count * 16 {
        return Err("unexpected family or example count".into());
    }
    for (family, rows) in &families {
        if rows.len() != 16 || !family_contexts.insert(&rows[0].records) {
            return Err(format!(
                "{family}: duplicated base context or wrong variant count"
            ));
        }
        for pair in rows.chunks_exact(2) {
            let base = pair[0];
            let changed = pair[1];
            if base.records != rows[0].records
                || base.question != changed.question
                || base.expected_source != changed.expected_source
                || base.expected_word != changed.expected_word
                || base.answer == changed.answer
            {
                return Err(format!("{family}: malformed matched intervention"));
            }
            let mut expected = base.records.clone();
            let mut selected = words(&base.records[base.expected_source]);
            selected[base.expected_word] = &changed.answer;
            let mut replacement = Vec::new();
            for (index, word) in selected.iter().enumerate() {
                if index != 0 {
                    replacement.push(b' ');
                }
                replacement.extend_from_slice(word);
            }
            replacement.push(b'.');
            expected[base.expected_source] = replacement;
            if changed.records != expected {
                return Err(format!(
                    "{family}: intervention changed an unrelated word or fact"
                ));
            }
        }
    }
    if slots.iter().any(|&count| count != examples.len() / 4)
        || answer_words != [examples.len() / 2, 0, 0, examples.len() / 2]
    {
        return Err("source slots or subject/object answers are unbalanced".into());
    }
    Ok(
        json!({"families": families.len(), "examples": examples.len(),
        "matched_source_changes": examples.len() / 2, "source_slots": slots,
        "answer_word_indices": answer_words, "unique_raw_inputs": inputs.len(),
        "unique_base_contexts": family_contexts.len()}),
    )
}

fn lexical_content(examples: &[Example]) -> BTreeSet<Vec<u8>> {
    examples
        .iter()
        .flat_map(|example| example.records.iter())
        .flat_map(|record| words(record))
        .filter(|word| *word != b"did")
        .map(<[u8]>::to_vec)
        .collect()
}

pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    let train = make_split(false);
    let development = make_split(true);
    check_split(&train, 32)?;
    check_split(&development, 8)?;
    if !lexical_content(&train).is_disjoint(&lexical_content(&development)) {
        return Err("train/development entity or relation vocabulary overlaps".into());
    }
    Ok((train, development))
}

pub fn validate() -> Result<Value, String> {
    let (train, development) = corpus()?;
    Ok(json!({"train": check_split(&train, 32)?,
        "development": check_split(&development, 8)?,
        "entity_and_relation_vocabulary_disjoint": true,
        "inference_inputs": ["records", "question"],
        "geometry_or_model_filtering": false,
        "final_held_out": "NOT_RUN"}))
}
