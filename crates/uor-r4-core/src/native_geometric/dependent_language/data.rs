//! Authored two-query relation chains and matched source interventions.
//!
//! Only records and raw prompt bytes are inference inputs. Typed authoring
//! metadata validates the chain independently of geometry or model outputs.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub split: String,
    pub construction: usize,
    pub variant: String,
    pub records: [Vec<u8>; 4],
    pub prompt: Vec<u8>,
    pub answer: Vec<u8>,
    pub expected_sources: [usize; 2],
    pub expected_words: [usize; 2],
    pub intermediate: Vec<u8>,
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

fn question(c: &Construction, subject_answer: bool, known: &str, verb: &str) -> Vec<u8> {
    if subject_answer {
        render(
            &[c.question_prefix, c.auxiliary, c.adverb, verb, known],
            '?',
        )
    } else {
        render(
            &[c.question_prefix, c.auxiliary, known, c.adverb, verb],
            '?',
        )
    }
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
        let offset = if training {
            index / 8
        } else if new_construction {
            index / 4
        } else {
            index % 8
        };
        let [a, b, cc, d, e, f] = std::array::from_fn(|i| names[(offset + i) % names.len()]);
        let vi = if training { index / 32 } else { index / 8 };
        let v = verbs[vi % verbs.len()];
        let w = verbs[(vi + 1) % verbs.len()];
        let permutation = if index / 4 % 2 == 0 {
            [0, 1, 2, 3]
        } else {
            [0, 2, 3, 1]
        };
        let slots: [usize; 4] = std::array::from_fn(|i| (permutation[i] + index % 4) % 4);
        for first_subject in [false, true] {
            for second_subject in [false, true] {
                let mut prompt = question(c, first_subject, a, v);
                prompt.push(b' ');
                prompt.extend(question(
                    c,
                    second_subject,
                    if second_subject { "them" } else { "they" },
                    w,
                ));
                for variant in ["base", "first-source", "active-final", "inactive-final"] {
                    let intermediate = if variant == "first-source" { d } else { b };
                    let active_answer = if variant == "active-final" { f } else { cc };
                    let inactive_answer = if variant == "inactive-final" { f } else { e };
                    let facts = [
                        if first_subject {
                            (intermediate, v, a)
                        } else {
                            (a, v, intermediate)
                        },
                        if second_subject {
                            (active_answer, w, b)
                        } else {
                            (b, w, active_answer)
                        },
                        if second_subject {
                            (inactive_answer, w, d)
                        } else {
                            (d, w, inactive_answer)
                        },
                        if second_subject { (f, w, a) } else { (a, w, f) },
                    ];
                    let mut records: [Vec<u8>; 4] = std::array::from_fn(|_| Vec::new());
                    for (logical, &(s, verb, o)) in facts.iter().enumerate() {
                        records[slots[logical]] = sentence(c, s, verb, o);
                    }
                    let subject_word = usize::from(!c.before.is_empty());
                    let object_word = subject_word + 3 + usize::from(!c.adverb.is_empty());
                    examples.push(Example {
                        id: format!(
                            "{family}-{}{}-{variant}",
                            usize::from(first_subject),
                            usize::from(second_subject)
                        ),
                        family: family.clone(),
                        split: split.into(),
                        construction,
                        variant: variant.into(),
                        records,
                        prompt: prompt.clone(),
                        answer: if variant == "first-source" {
                            e
                        } else {
                            active_answer
                        }
                        .as_bytes()
                        .to_vec(),
                        expected_sources: [
                            slots[0],
                            slots[if variant == "first-source" { 2 } else { 1 }],
                        ],
                        expected_words: [
                            if first_subject {
                                subject_word
                            } else {
                                object_word
                            },
                            if second_subject {
                                subject_word
                            } else {
                                object_word
                            },
                        ],
                        intermediate: intermediate.as_bytes().to_vec(),
                    });
                }
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

type Answer = (usize, usize, Vec<u8>, bool);

/// Typed grammar oracle, deliberately independent of the runtime's geometry,
/// candidate descriptors, learned substitutions and learned selection rules.
fn resolve(
    c: &Construction,
    records: &[Vec<u8>; 4],
    question: &[u8],
    reference: Option<&[u8]>,
) -> Result<Answer, String> {
    let mut q = words(question);
    let prefix: Vec<_> = c
        .question_prefix
        .split_whitespace()
        .map(str::as_bytes)
        .collect();
    let lead = prefix.len() + 1;
    if q.len() != lead + 2 + usize::from(!c.adverb.is_empty())
        || q[..prefix.len()] != prefix
        || q[prefix.len()] != c.auxiliary.as_bytes()
        || question.last() != Some(&b'?')
    {
        return Err("malformed authored question".into());
    }
    let references: Vec<_> = q
        .iter()
        .enumerate()
        .filter(|(_, token)| **token == b"they" || **token == b"them")
        .map(|(i, _)| i)
        .collect();
    match reference {
        Some(value) if references.len() == 1 => q[references[0]] = value,
        None if references.is_empty() => (),
        _ => return Err("reference token missing, repeated or supplied to first query".into()),
    }
    let subject_word = usize::from(!c.before.is_empty());
    let verb_word = subject_word + 2 + usize::from(!c.adverb.is_empty());
    let object_word = verb_word + 1;
    let mut matches = Vec::new();
    for (slot, record) in records.iter().enumerate() {
        let fact = words(record);
        if fact.len() != object_word + 1 + usize::from(!c.after.is_empty())
            || fact[subject_word + 1] != c.auxiliary.as_bytes()
            || (!c.before.is_empty() && fact[0] != c.before.as_bytes())
            || (!c.adverb.is_empty() && fact[subject_word + 2] != c.adverb.as_bytes())
            || (!c.after.is_empty() && fact[object_word + 1] != c.after.as_bytes())
            || record.last() != Some(&b'.')
        {
            return Err("malformed authored fact".into());
        }
        let si = lead + usize::from(!c.adverb.is_empty());
        let oi = lead + 1 + usize::from(!c.adverb.is_empty());
        if (c.adverb.is_empty() || q[lead] == c.adverb.as_bytes())
            && fact[verb_word] == q[si]
            && fact[object_word] == q[si + 1]
        {
            matches.push((slot, subject_word, fact[subject_word].to_vec(), true));
        }
        if (c.adverb.is_empty() || q[lead + 1] == c.adverb.as_bytes())
            && fact[subject_word] == q[lead]
            && fact[verb_word] == q[oi]
        {
            matches.push((slot, object_word, fact[object_word].to_vec(), false));
        }
    }
    if matches.len() != 1 {
        return Err("authored query is absent or ambiguous".into());
    }
    matches
        .pop()
        .ok_or_else(|| "authored answer disappeared".into())
}

fn check_example(row: &Example) -> Result<[bool; 2], String> {
    let c = CONSTRUCTIONS
        .get(row.construction)
        .ok_or("unknown construction")?;
    if row.prompt.len() > 256
        || !row.prompt.ends_with(b"?")
        || row.prompt.iter().filter(|&&b| b == b'?').count() != 2
    {
        return Err(format!(
            "{}: expected exactly two bounded question sentences",
            row.id
        ));
    }
    let boundary = row
        .prompt
        .iter()
        .position(|&b| b == b'?')
        .ok_or("missing question boundary")?;
    let first_query = &row.prompt[..=boundary];
    let second_query = &row.prompt[boundary + 1..];
    for bytes in row
        .records
        .iter()
        .map(Vec::as_slice)
        .chain([first_query, second_query])
    {
        let tokens = words(bytes);
        if bytes.len() > 128
            || tokens.len() > 16
            || tokens
                .iter()
                .any(|word| word.len() > 16 || !word.iter().all(u8::is_ascii_lowercase))
        {
            return Err(format!("{}: unsupported lexical window", row.id));
        }
    }
    let first = resolve(c, &row.records, first_query, None)?;
    let second = resolve(c, &row.records, second_query, Some(&first.2))?;
    if row.expected_sources != [first.0, second.0]
        || row.expected_words != [first.1, second.1]
        || row.intermediate != first.2
        || row.answer != second.2
        || first.0 == second.0
    {
        return Err(format!(
            "{}: typed two-read path, intermediate or answer mismatch",
            row.id
        ));
    }
    if words(&row.prompt)
        .iter()
        .any(|&token| token == row.intermediate || token == row.answer)
    {
        return Err(format!(
            "{}: intermediate or answer supplied in prompt",
            row.id
        ));
    }
    Ok([first.3, second.3])
}

fn replace_word(record: &[u8], word: usize, replacement: &[u8]) -> Result<Vec<u8>, String> {
    let mut tokens = words(record);
    *tokens
        .get_mut(word)
        .ok_or("intervention word outside record")? = replacement;
    let mut result = tokens.join(&b' ');
    result.push(b'.');
    Ok(result)
}

fn check_split(examples: &[Example], split: &str, count: usize) -> Result<Value, String> {
    let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
    let mut inputs = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut slots = [[0usize; 4]; 2];
    let mut roles = [[0usize; 2]; 2];
    let mut combinations = [0usize; 4];
    let mut constructions = BTreeSet::new();
    let mut record_lengths = BTreeSet::new();
    let mut question_lengths = BTreeSet::new();
    for row in examples {
        let subject = check_example(row)?;
        if row.split != split || !inputs.insert((&row.records, &row.prompt)) || !ids.insert(&row.id)
        {
            return Err(format!("{}: wrong split or duplicate raw input/ID", row.id));
        }
        for i in 0..2 {
            slots[i][row.expected_sources[i]] += 1;
            roles[i][usize::from(subject[i])] += 1;
        }
        combinations[usize::from(subject[0]) * 2 + usize::from(subject[1])] += 1;
        constructions.insert(row.construction);
        record_lengths.extend(row.records.iter().map(|record| words(record).len()));
        question_lengths.extend(
            row.prompt
                .split(|&byte| byte == b'?')
                .filter(|bytes| !words(bytes).is_empty())
                .map(|q| words(q).len()),
        );
        families.entry(&row.family).or_default().push(row);
    }
    if examples.len() != count * 16
        || families.len() != count
        || slots.iter().flatten().any(|&n| n != examples.len() / 4)
        || roles.iter().flatten().any(|&n| n != examples.len() / 2)
        || combinations.iter().any(|&n| n != examples.len() / 4)
    {
        return Err(format!(
            "{split}: incorrect counts or unbalanced role/source paths"
        ));
    }
    for (family, rows) in &families {
        if rows.len() != 16 {
            return Err(format!("{family}: incomplete family"));
        }
        for group in rows.chunks_exact(4) {
            let [base, first, active, inactive] = [group[0], group[1], group[2], group[3]];
            if [
                base.variant.as_str(),
                first.variant.as_str(),
                active.variant.as_str(),
                inactive.variant.as_str(),
            ] != ["base", "first-source", "active-final", "inactive-final"]
                || group.iter().any(|row| {
                    row.prompt != base.prompt
                        || row.construction != base.construction
                        || row.expected_words != base.expected_words
                })
                || first.expected_sources[0] != base.expected_sources[0]
                || first.expected_sources[1] == base.expected_sources[1]
                || active.expected_sources != base.expected_sources
                || inactive.expected_sources != base.expected_sources
                || first.intermediate == base.intermediate
                || first.answer == base.answer
                || active.intermediate != base.intermediate
                || inactive.intermediate != base.intermediate
                || active.answer == base.answer
                || inactive.answer != base.answer
            {
                return Err(format!("{family}: malformed causal intervention group"));
            }
            let mut expected = base.records.clone();
            expected[base.expected_sources[0]] = replace_word(
                &expected[base.expected_sources[0]],
                base.expected_words[0],
                &first.intermediate,
            )?;
            if first.records != expected {
                return Err(format!("{family}: first-source changes unrelated bytes"));
            }
            expected = base.records.clone();
            expected[base.expected_sources[1]] = replace_word(
                &expected[base.expected_sources[1]],
                base.expected_words[1],
                &active.answer,
            )?;
            if active.records != expected {
                return Err(format!("{family}: active-final changes unrelated bytes"));
            }
            expected = base.records.clone();
            expected[first.expected_sources[1]] = replace_word(
                &expected[first.expected_sources[1]],
                first.expected_words[1],
                &active.answer,
            )?;
            if inactive.records != expected {
                return Err(format!("{family}: inactive-final changes unrelated bytes"));
            }
        }
    }
    Ok(
        json!({"families":families.len(),"examples":examples.len(),"source_slots_by_read":slots,
        "subject_object_counts_by_read":roles,"role_combinations":combinations,"constructions":constructions,
        "record_word_lengths":record_lengths,"question_word_lengths":question_lengths,"unique_raw_inputs":inputs.len(),
        "matched_first_source_switches":examples.len()/4,"matched_active_final_changes":examples.len()/4,
        "matched_inactive_final_controls":examples.len()/4}),
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
        .map(|row| (row.records.clone(), row.prompt.clone()))
        .collect();
    let mut development = Vec::with_capacity(768);
    for split in ["lexical", "syntactic", "joint"] {
        let rows = make_split(split);
        check_split(&rows, split, 16)?;
        let content = content_vocabulary(&rows);
        let constructions: BTreeSet<_> = rows.iter().map(|row| row.construction).collect();
        if matches!(split, "lexical" | "joint") {
            if !vocabulary.is_disjoint(&content)
                || rows.iter().any(|row| {
                    answers.contains(&row.answer) || vocabulary.contains(&row.intermediate)
                })
            {
                return Err(format!(
                    "{split}: entity, verb, intermediate or answer leaks from training"
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
            if !all_inputs.insert((row.records.clone(), row.prompt.clone())) {
                return Err(format!("{}: duplicate raw input across splits", row.id));
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
        "lexical_and_joint_content_and_answers_disjoint":true,"syntactic_content_seen_in_training":true,
        "syntactic_and_joint_constructions_disjoint":true,"references":{"known_subject":"they","known_object":"them"},
        "syntax_scope":"two-query chains over unseen combinations of familiar active auxiliary/adverb/question-prefix/source-adjunct components",
        "inference_inputs":["records","prompt"],"training_supervision":["final answer"],
        "evaluation_only":["expected_sources","expected_words","intermediate"],"geometry_or_model_filtering":false,"final_held_out":"NOT_RUN"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependent_corpus_checks_two_read_causality_and_rejects_path_or_unrelated_edits(
    ) -> Result<(), String> {
        let (train, development) = corpus()?;
        assert_eq!(train.len(), 2048);
        assert_eq!(development.len(), 768);
        assert_eq!(
            validate()?["development"]["splits"]["joint"]["examples"],
            256
        );
        let mut corrupted = train.clone();
        let slot = (corrupted[1].expected_sources[0] + 1) % 4;
        let source = corrupted[1].expected_sources[0];
        corrupted[1].records[slot] = corrupted[1].records[source].clone();
        assert!(check_split(&corrupted, "train", 128).is_err());
        let mut corrupted = development[0].clone();
        corrupted.expected_sources.swap(0, 1);
        assert!(check_example(&corrupted).is_err());
        let mut corrupted = development[0].clone();
        corrupted.intermediate = b"wrong".to_vec();
        assert!(check_example(&corrupted).is_err());
        Ok(())
    }
}
