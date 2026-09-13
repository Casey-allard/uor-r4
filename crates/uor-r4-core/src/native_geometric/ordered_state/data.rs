//! Offline ordered-occurrence and selected-payload interventions.
//!
//! Keys are supplied typed byte-token sequences, not parsed language relations.
//! This authoring oracle follows exact prefixes. Inference receives neither the
//! expected source path nor answers. No geometric state or model is consulted
//! when constructing or retaining examples.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub key: Vec<u8>,
    pub text: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub records: [Record; 4],
    pub query: Vec<u8>,
    pub answer: Vec<u8>,
    pub expected_sources: Vec<usize>,
}

const SUBJECTS: [&str; 16] = [
    "bird", "fish", "cat", "dog", "fox", "owl", "bear", "hare", "ant", "bee", "wolf", "elk",
    "mole", "newt", "otter", "yak",
];
const ENDINGS: [&str; 8] = [
    "waits.", "rests.", "walks.", "hides.", "plays.", "moves.", "stays.", "sleeps.",
];
const CHANGED_ENDINGS: [&str; 8] = [
    "waits nearby.",
    "rests quietly.",
    "walks slowly.",
    "hides outside.",
    "plays happily.",
    "moves softly.",
    "stays here.",
    "sleeps soundly.",
];

fn queries(index: usize) -> (Vec<u8>, Vec<u8>) {
    // A unique opaque family token separates raw train/development prefixes.
    // No byte is a target label, depth annotation, or source-slot selector.
    let interior = [
        17 + (index % 7) as u8,
        48 + (index % 11) as u8,
        80 + (index % 13) as u8,
        110 + (index % 5) as u8,
    ];
    let mut query = vec![128 + index as u8];
    query.extend_from_slice(&interior[..2 + index % 3]);
    query.push(240 + (index % 8) as u8);
    let mut reordered = query.clone();
    reordered.swap(1, 2);
    (query, reordered)
}

fn make_split(development: bool, families: usize) -> Vec<Example> {
    let prefix = if development { "development" } else { "train" };
    let offset = if development { 64 } else { 0 };
    let mut examples = Vec::with_capacity(families * 4);
    for family_index in 0..families {
        let index = offset + family_index;
        let family = format!("{prefix}-{family_index:03}");
        let (query, reordered) = queries(index);
        let first_a = format!("the {} ", SUBJECTS[(index * 3) % SUBJECTS.len()]).into_bytes();
        let first_b = format!("the {} ", SUBJECTS[(index * 3 + 5) % SUBJECTS.len()]).into_bytes();
        let ending_a = index % ENDINGS.len();
        let ending_b = (index + 3) % ENDINGS.len();
        let mut tail_a = query.clone();
        tail_a.extend_from_slice(&first_a);
        let mut tail_b = reordered.clone();
        tail_b.extend_from_slice(&first_b);
        let logical = [
            Record {
                key: query.clone(),
                text: first_a,
            },
            Record {
                key: tail_a,
                text: ENDINGS[ending_a].as_bytes().to_vec(),
            },
            Record {
                key: reordered.clone(),
                text: first_b,
            },
            Record {
                key: tail_b,
                text: ENDINGS[ending_b].as_bytes().to_vec(),
            },
        ];
        // Every block of four families balances each logical record across all
        // physical slots. Alternate permutations avoid one fixed adjacency.
        let permutation = if family_index / 4 % 2 == 0 {
            [0, 1, 2, 3]
        } else {
            [0, 2, 3, 1]
        };
        let slots: [usize; 4] =
            std::array::from_fn(|logical| (permutation[logical] + family_index % 4) % 4);
        let mut records = logical.clone();
        for logical_slot in 0..4 {
            records[slots[logical_slot]] = logical[logical_slot].clone();
        }
        for variant in 0..4 {
            let order_b = variant % 2 == 1;
            let changed = variant >= 2;
            let first = if order_b { 2 } else { 0 };
            let tail = first + 1;
            let mut context = records.clone();
            if changed {
                let ending = if order_b { ending_b } else { ending_a };
                context[slots[tail]].text = CHANGED_ENDINGS[ending].as_bytes().to_vec();
            }
            let answer = context[slots[first]]
                .text
                .iter()
                .chain(&context[slots[tail]].text)
                .copied()
                .collect();
            examples.push(Example {
                id: format!(
                    "{family}-{}{}",
                    if order_b { "ba" } else { "ab" },
                    if changed { "-changed" } else { "" }
                ),
                family: family.clone(),
                records: context,
                query: if order_b {
                    reordered.clone()
                } else {
                    query.clone()
                },
                answer,
                expected_sources: vec![slots[first], slots[tail]],
            });
        }
    }
    examples
}

/// Exact raw-prefix answer oracle for offline validation only.
pub fn trace(example: &Example) -> Result<Vec<usize>, String> {
    let mut prefix = example.query.clone();
    let mut path = Vec::new();
    let mut answer = Vec::new();
    loop {
        let matching: Vec<_> = example
            .records
            .iter()
            .enumerate()
            .filter(|(_, record)| record.key == prefix)
            .map(|(slot, _)| slot)
            .collect();
        if matching.len() > 1 {
            return Err(format!("{}: duplicate exact occurrence key", example.id));
        }
        let Some(&slot) = matching.first() else {
            break;
        };
        if path.len() >= 4 || path.contains(&slot) {
            return Err(format!("{}: cyclic or overlong exact path", example.id));
        }
        path.push(slot);
        prefix.extend_from_slice(&example.records[slot].text);
        answer.extend_from_slice(&example.records[slot].text);
    }
    if path != example.expected_sources || answer != example.answer || path.len() != 2 {
        return Err(format!("{}: source or answer oracle mismatch", example.id));
    }
    Ok(path)
}

fn split_validation(examples: &[Example], expected_families: usize) -> Result<Value, String> {
    let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut first_slots = [0usize; 4];
    let mut tail_slots = [0usize; 4];
    let mut lengths = BTreeMap::new();
    for example in examples {
        if !ids.insert(&example.id) {
            return Err(format!("duplicate example {}", example.id));
        }
        if !(4..=6).contains(&example.query.len())
            || example.records.iter().any(|record| {
                record.key.is_empty()
                    || record.key.len() > 128
                    || record.text.is_empty()
                    || record.text.len() > 32
                    || !record.text.is_ascii()
            })
            || example
                .records
                .iter()
                .map(|record| &record.key)
                .collect::<BTreeSet<_>>()
                .len()
                != 4
        {
            return Err(format!(
                "{}: invalid sequence, span, or key bounds",
                example.id
            ));
        }
        let path = trace(example)?;
        first_slots[path[0]] += 1;
        tail_slots[path[1]] += 1;
        *lengths.entry(example.query.len()).or_insert(0usize) += 1;
        families.entry(&example.family).or_default().push(example);
    }
    if families.len() != expected_families || examples.len() != expected_families * 4 {
        return Err("incorrect family or row count".into());
    }
    for (family, rows) in &families {
        if rows.len() != 4 {
            return Err(format!("{family}: incomplete intervention family"));
        }
        let (a, b, changed_a, changed_b) = (rows[0], rows[1], rows[2], rows[3]);
        let mut sorted_a = a.query.clone();
        let mut sorted_b = b.query.clone();
        sorted_a.sort_unstable();
        sorted_b.sort_unstable();
        if a.query == b.query
            || sorted_a != sorted_b
            || a.query.first() != b.query.first()
            || a.query.last() != b.query.last()
            || a.records != b.records
            || a.answer == b.answer
            || a.expected_sources == b.expected_sources
        {
            return Err(format!("{family}: invalid interior-order intervention"));
        }
        for (base, changed) in [(a, changed_a), (b, changed_b)] {
            let differences: Vec<_> = base
                .records
                .iter()
                .zip(&changed.records)
                .enumerate()
                .filter(|(_, (before, after))| before != after)
                .map(|(slot, _)| slot)
                .collect();
            if base.query != changed.query
                || base.expected_sources != changed.expected_sources
                || base.answer == changed.answer
                || differences != vec![base.expected_sources[1]]
                || base
                    .records
                    .iter()
                    .zip(&changed.records)
                    .any(|(before, after)| before.key != after.key)
            {
                return Err(format!("{family}: invalid selected-tail intervention"));
            }
        }
    }
    if first_slots != [examples.len() / 4; 4] || tail_slots != [examples.len() / 4; 4] {
        return Err("unbalanced selected source slots".into());
    }
    Ok(json!({
        "rows": examples.len(), "families": families.len(),
        "query_order_pairs": families.len(), "changed_source_pairs": families.len() * 2,
        "first_source_slots": first_slots, "tail_source_slots": tail_slots,
        "query_length_counts": lengths, "exact_trace_rows": examples.len(),
        "max_key_tokens": examples.iter().flat_map(|e| &e.records).map(|r| r.key.len()).max(),
        "max_span_bytes": examples.iter().flat_map(|e| &e.records).map(|r| r.text.len()).max(),
        "max_answer_bytes": examples.iter().map(|e| e.answer.len()).max(),
    }))
}

pub fn validate(train: &[Example], development: &[Example]) -> Result<Value, String> {
    let training = split_validation(train, 64)?;
    let development_report = split_validation(development, 16)?;
    let train_keys: BTreeSet<_> = train
        .iter()
        .flat_map(|e| &e.records)
        .map(|r| &r.key)
        .collect();
    if development
        .iter()
        .flat_map(|e| &e.records)
        .any(|r| train_keys.contains(&r.key))
    {
        return Err("training and development exact keys overlap".into());
    }
    Ok(json!({
        "training": training, "development": development_report,
        "exact_key_split_overlap": 0, "geometric_or_model_filtering": false,
        "query_semantics": "supplied opaque ordered byte-token occurrences",
        "answer_oracle": "concatenate payloads along exact raw-prefix path, then EOS",
        "final_held_out": "NOT_RUN",
    }))
}

pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    let train = make_split(false, 64);
    let development = make_split(true, 16);
    validate(&train, &development)?;
    Ok((train, development))
}
