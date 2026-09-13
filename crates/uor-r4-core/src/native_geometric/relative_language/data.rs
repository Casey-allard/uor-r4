//! Authored variable-length relation questions and matched source interventions.
//!
//! Records and question bytes are the only model inputs. Construction, split,
//! source, word and answer fields belong to the offline authoring oracle. No
//! model, geometry, feature learner or successful-output filter is consulted.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub split: String,
    pub construction: usize,
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

struct Construction {
    auxiliary: &'static str,
    adverb: &'static str,
    before: &'static str,
    after: &'static str,
    question_prefix: &'static str,
}

// The development constructions recombine familiar grammatical components;
// their complete source/question construction pairs never occur in training.
const CONSTRUCTIONS: [Construction; 12] = [
    Construction {
        auxiliary: "did",
        adverb: "",
        before: "",
        after: "",
        question_prefix: "who",
    },
    Construction {
        auxiliary: "can",
        adverb: "quietly",
        before: "today",
        after: "",
        question_prefix: "which person",
    },
    Construction {
        auxiliary: "will",
        adverb: "",
        before: "",
        after: "tomorrow",
        question_prefix: "please tell me who",
    },
    Construction {
        auxiliary: "did",
        adverb: "really",
        before: "yesterday",
        after: "outside",
        question_prefix: "who",
    },
    Construction {
        auxiliary: "can",
        adverb: "often",
        before: "",
        after: "outside",
        question_prefix: "please tell me who",
    },
    Construction {
        auxiliary: "will",
        adverb: "quietly",
        before: "tomorrow",
        after: "",
        question_prefix: "who",
    },
    Construction {
        auxiliary: "did",
        adverb: "often",
        before: "",
        after: "",
        question_prefix: "which person",
    },
    Construction {
        auxiliary: "will",
        adverb: "really",
        before: "",
        after: "outside",
        question_prefix: "which person",
    },
    Construction {
        auxiliary: "did",
        adverb: "quietly",
        before: "yesterday",
        after: "outside",
        question_prefix: "please tell me who",
    },
    Construction {
        auxiliary: "can",
        adverb: "really",
        before: "today",
        after: "outside",
        question_prefix: "who",
    },
    Construction {
        auxiliary: "will",
        adverb: "often",
        before: "tomorrow",
        after: "outside",
        question_prefix: "which person",
    },
    Construction {
        auxiliary: "did",
        adverb: "really",
        before: "yesterday",
        after: "",
        question_prefix: "which person",
    },
];

fn render(parts: &[&str], punctuation: char) -> Vec<u8> {
    let text = parts
        .iter()
        .filter(|part| !part.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    format!("{text}{punctuation}").into_bytes()
}

fn sentence(c: &Construction, subject: &str, verb: &str, object: &str) -> Vec<u8> {
    render(
        &[
            c.before,
            subject,
            c.auxiliary,
            c.adverb,
            verb,
            object,
            c.after,
        ],
        '.',
    )
}

fn make_split(split: &str) -> Vec<Example> {
    let training = split == "train";
    let new_lexicon = matches!(split, "lexical" | "joint");
    let new_construction = matches!(split, "syntactic" | "joint");
    let names: &[&str] = if new_lexicon {
        &DEVELOPMENT_NAMES
    } else {
        &TRAIN_NAMES
    };
    let verbs: &[&str] = if new_lexicon {
        &DEVELOPMENT_VERBS
    } else {
        &TRAIN_VERBS
    };
    let count = if training { 128 } else { 16 };
    let mut examples = Vec::with_capacity(count * 16);
    for index in 0..count {
        let construction = if new_construction {
            8 + index % 4
        } else {
            index % 8
        };
        let c = &CONSTRUCTIONS[construction];
        let family = format!("{split}-{index:03}");
        let name_offset = if training {
            index / 8
        } else if new_construction {
            index / 4
        } else {
            index % 8
        };
        let [a, b, cc, d, e] =
            std::array::from_fn(|offset| names[(name_offset + offset) % names.len()]);
        let first_verb = if training { index / 32 } else { index / 8 };
        let v = verbs[first_verb % verbs.len()];
        let w = verbs[(first_verb + 1) % verbs.len()];
        let facts = [(a, v, b), (b, v, cc), (a, w, d), (d, w, cc)];
        let permutation = if index / 4 % 2 == 0 {
            [0, 1, 2, 3]
        } else {
            [0, 2, 3, 1]
        };
        let slots: [usize; 4] =
            std::array::from_fn(|logical| (permutation[logical] + index % 4) % 4);
        let mut records: [Vec<u8>; 4] = std::array::from_fn(|_| Vec::new());
        for (logical, &(subject, verb, object)) in facts.iter().enumerate() {
            records[slots[logical]] = sentence(c, subject, verb, object);
        }
        // Both directions of all four facts require role and verb alignment.
        let queries = [
            (0, true),
            (1, false),
            (2, true),
            (3, false),
            (0, false),
            (2, false),
            (1, true),
            (3, true),
        ];
        for (question_index, (logical, subject_answer)) in queries.into_iter().enumerate() {
            let (subject, verb, object) = facts[logical];
            let question = if subject_answer {
                render(
                    &[c.question_prefix, c.auxiliary, c.adverb, verb, object],
                    '?',
                )
            } else {
                render(
                    &[c.question_prefix, c.auxiliary, subject, c.adverb, verb],
                    '?',
                )
            };
            let expected_word = usize::from(!c.before.is_empty())
                + if subject_answer {
                    0
                } else {
                    3 + usize::from(!c.adverb.is_empty())
                };
            for changed in [false, true] {
                let answer = if changed {
                    e
                } else if subject_answer {
                    subject
                } else {
                    object
                };
                let mut context = records.clone();
                if changed {
                    context[slots[logical]] = if subject_answer {
                        sentence(c, e, verb, object)
                    } else {
                        sentence(c, subject, verb, e)
                    };
                }
                examples.push(Example {
                    id: format!(
                        "{family}-q{question_index}{}",
                        if changed { "-changed" } else { "" }
                    ),
                    family: family.clone(),
                    split: split.into(),
                    construction,
                    records: context,
                    question: question.clone(),
                    answer: answer.as_bytes().to_vec(),
                    expected_source: slots[logical],
                    expected_word,
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

/// Independent typed construction oracle. It does not implement or consult the
/// proposed relative-position features, Hamming distance or learned rule masks.
fn check_example(example: &Example) -> Result<bool, String> {
    let c = CONSTRUCTIONS
        .get(example.construction)
        .ok_or("unknown construction")?;
    let q = words(&example.question);
    let prefix: Vec<_> = c
        .question_prefix
        .split_whitespace()
        .map(str::as_bytes)
        .collect();
    let lead = prefix.len() + 1;
    if q.len() != lead + 2 + usize::from(!c.adverb.is_empty())
        || q[..prefix.len()] != prefix
        || q[prefix.len()] != c.auxiliary.as_bytes()
        || example.question.last() != Some(&b'?')
    {
        return Err(format!("{}: malformed authored question", example.id));
    }
    let subject_word = usize::from(!c.before.is_empty());
    let verb_word = subject_word + 2 + usize::from(!c.adverb.is_empty());
    let object_word = verb_word + 1;
    let mut matches = Vec::new();
    for (slot, record) in example.records.iter().enumerate() {
        let fact = words(record);
        let expected_len = object_word + 1 + usize::from(!c.after.is_empty());
        if fact.len() != expected_len
            || fact[subject_word + 1] != c.auxiliary.as_bytes()
            || (!c.before.is_empty() && fact[0] != c.before.as_bytes())
            || (!c.adverb.is_empty() && fact[subject_word + 2] != c.adverb.as_bytes())
            || (!c.after.is_empty() && fact[object_word + 1] != c.after.as_bytes())
            || record.last() != Some(&b'.')
        {
            return Err(format!("{}: malformed authored fact", example.id));
        }
        let subject_query_start = lead + usize::from(!c.adverb.is_empty());
        let object_query_verb = lead + 1 + usize::from(!c.adverb.is_empty());
        let subject_adverb = c.adverb.is_empty() || q[lead] == c.adverb.as_bytes();
        let object_adverb = c.adverb.is_empty() || q[lead + 1] == c.adverb.as_bytes();
        if subject_adverb
            && fact[verb_word] == q[subject_query_start]
            && fact[object_word] == q[subject_query_start + 1]
        {
            matches.push((slot, subject_word, fact[subject_word]));
        }
        if object_adverb && fact[subject_word] == q[lead] && fact[verb_word] == q[object_query_verb]
        {
            matches.push((slot, object_word, fact[object_word]));
        }
    }
    if matches.as_slice()
        != [(
            example.expected_source,
            example.expected_word,
            example.answer.as_slice(),
        )]
    {
        return Err(format!(
            "{}: ambiguous or mismatched authored answer",
            example.id
        ));
    }
    for bytes in example
        .records
        .iter()
        .chain(std::iter::once(&example.question))
    {
        let tokens = words(bytes);
        if bytes.len() > 128
            || tokens.len() > 16
            || tokens
                .iter()
                .any(|word| word.len() > 16 || !word.iter().all(u8::is_ascii_lowercase))
        {
            return Err(format!("{}: unsupported lexical window", example.id));
        }
    }
    Ok(example.expected_word == subject_word)
}

fn check_split(examples: &[Example], split: &str, count: usize) -> Result<Value, String> {
    let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
    let mut inputs = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    let mut slots = [0usize; 4];
    let mut subjects = 0;
    let mut constructions = BTreeSet::new();
    let mut record_lengths = BTreeSet::new();
    let mut question_lengths = BTreeSet::new();
    let mut answer_positions: BTreeMap<usize, usize> = BTreeMap::new();
    for row in examples {
        subjects += usize::from(check_example(row)?);
        if row.split != split
            || !inputs.insert((&row.records, &row.question))
            || !ids.insert(&row.id)
        {
            return Err(format!(
                "{}: wrong split or duplicated raw input/ID",
                row.id
            ));
        }
        constructions.insert(row.construction);
        slots[row.expected_source] += 1;
        *answer_positions.entry(row.expected_word).or_default() += 1;
        record_lengths.extend(row.records.iter().map(|record| words(record).len()));
        question_lengths.insert(words(&row.question).len());
        families.entry(&row.family).or_default().push(row);
    }
    if families.len() != count
        || examples.len() != count * 16
        || subjects != examples.len() / 2
        || slots.iter().any(|&n| n != examples.len() / 4)
    {
        return Err(format!(
            "{split}: incorrect counts or unbalanced answer paths"
        ));
    }
    for (family, rows) in &families {
        if rows.len() != 16 || !contexts.insert(&rows[0].records) {
            return Err(format!(
                "{family}: duplicate base context or incomplete family"
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
                || base.construction != changed.construction
            {
                return Err(format!("{family}: malformed intervention pair"));
            }
            let mut expected = base.records.clone();
            let mut selected = words(&base.records[base.expected_source]);
            selected[base.expected_word] = &changed.answer;
            expected[base.expected_source] = selected.join(&b' ');
            expected[base.expected_source].push(b'.');
            if changed.records != expected {
                return Err(format!(
                    "{family}: intervention modified an unrelated word or source"
                ));
            }
        }
    }
    Ok(
        json!({"families":families.len(),"examples":examples.len(),"matched_source_changes":examples.len()/2,
        "source_slots":slots,"subject_answers":subjects,"object_answers":examples.len()-subjects,
        "answer_word_indices":answer_positions,"constructions":constructions,"record_word_lengths":record_lengths,
        "question_word_lengths":question_lengths,"unique_raw_inputs":inputs.len(),"unique_base_contexts":contexts.len()}),
    )
}

fn content_vocabulary(examples: &[Example]) -> BTreeSet<Vec<u8>> {
    let mut result = BTreeSet::new();
    for row in examples {
        let c = &CONSTRUCTIONS[row.construction];
        let subject = usize::from(!c.before.is_empty());
        let verb = subject + 2 + usize::from(!c.adverb.is_empty());
        for record in &row.records {
            let tokens = words(record);
            result.extend([
                tokens[subject].to_vec(),
                tokens[verb].to_vec(),
                tokens[verb + 1].to_vec(),
            ]);
        }
    }
    result
}

pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    let train = make_split("train");
    check_split(&train, "train", 128)?;
    let vocabulary = content_vocabulary(&train);
    let answers: BTreeSet<_> = train.iter().map(|row| row.answer.clone()).collect();
    let train_constructions: BTreeSet<_> = train.iter().map(|row| row.construction).collect();
    let mut all_inputs: BTreeSet<_> = train
        .iter()
        .map(|row| (row.records.clone(), row.question.clone()))
        .collect();
    let mut development = Vec::with_capacity(768);
    for split in ["lexical", "syntactic", "joint"] {
        let rows = make_split(split);
        check_split(&rows, split, 16)?;
        let content = content_vocabulary(&rows);
        let constructions: BTreeSet<_> = rows.iter().map(|row| row.construction).collect();
        if matches!(split, "lexical" | "joint") {
            if !vocabulary.is_disjoint(&content)
                || rows.iter().any(|row| answers.contains(&row.answer))
            {
                return Err(format!(
                    "{split}: entity, verb or answer leaks from training"
                ));
            }
        } else if !content.is_subset(&vocabulary) {
            return Err("syntactic split unexpectedly changes content vocabulary".into());
        }
        if matches!(split, "syntactic" | "joint") {
            if !train_constructions.is_disjoint(&constructions) {
                return Err(format!("{split}: construction leaks from training"));
            }
        } else if !constructions.is_subset(&train_constructions) {
            return Err("lexical split unexpectedly changes constructions".into());
        }
        for row in &rows {
            if !all_inputs.insert((row.records.clone(), row.question.clone())) {
                return Err(format!("{}: duplicated raw input across splits", row.id));
            }
        }
        development.extend(rows);
    }
    Ok((train, development))
}

pub fn validate() -> Result<Value, String> {
    let (train, development) = corpus()?;
    let mut splits = serde_json::Map::new();
    for split in ["lexical", "syntactic", "joint"] {
        let rows: Vec<_> = development
            .iter()
            .filter(|row| row.split == split)
            .cloned()
            .collect();
        splits.insert(split.into(), check_split(&rows, split, 16)?);
    }
    Ok(
        json!({"train":check_split(&train,"train",128)?,"development":{"examples":development.len(),"families":48,"splits":splits},
        "lexical_and_joint_entity_relation_and_answer_vocabulary_disjoint":true,
        "syntactic_content_vocabulary_seen_in_training":true,"syntactic_and_joint_constructions_disjoint":true,
        "syntax_scope":"unseen combinations of familiar active auxiliary/adverb/question-prefix/source-adjunct components",
        "inference_inputs":["records","question"],"geometry_or_model_filtering":false,"final_held_out":"NOT_RUN"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_corpus_separates_lexical_and_construction_transfer_and_rejects_unrelated_edits(
    ) -> Result<(), String> {
        let (train, development) = corpus()?;
        assert_eq!(train.len(), 2048);
        assert_eq!(development.len(), 768);
        assert_eq!(
            validate()?["development"]["splits"]["joint"]["examples"],
            256
        );
        let mut corrupted = train.clone();
        let slot = (corrupted[1].expected_source + 1) % 4;
        let selected = corrupted[1].expected_source;
        let replacement = corrupted[1].records[selected].clone();
        corrupted[1].records[slot] = replacement;
        assert!(check_split(&corrupted, "train", 128).is_err());
        let mut corrupted = development[0].clone();
        corrupted.expected_word += 1;
        assert!(check_example(&corrupted).is_err());
        Ok(())
    }
}
