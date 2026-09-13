//! A typed contextual-selection environment, not a prose or binding benchmark.
//!
//! Only `records` and `query` cross the learner's inference boundary. `answer`,
//! `id` and `family` belong to offline supervision/scoring. The two episodes in
//! each family change the queried value without changing keys or query.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VALUES: &[u8; 8] = b"abcdefgh";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Record {
    pub key: [u8; 2],
    pub value: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub records: [Record; 4],
    pub query: [u8; 2],
    pub answer: u8,
    pub family: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryPositionCounts {
    pub query: [u8; 2],
    pub target_slot_counts: [usize; 4],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitValidation {
    pub examples: usize,
    pub families: usize,
    pub answer_counts: [usize; 8],
    pub target_slot_counts: [usize; 4],
    pub answer_counts_by_target_slot: [[usize; 8]; 4],
    pub key_alphabet: Vec<u8>,
    pub query_positions: Vec<QueryPositionCounts>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorpusValidation {
    pub train: SplitValidation,
    pub dev: SplitValidation,
    pub disjoint_contexts: bool,
    pub disjoint_families: bool,
    pub disjoint_key_alphabets: bool,
}

/// Fixed construction/development data; neither split is a final held-out gate.
pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    // Cover canonical byte-key identities without claiming prose competence.
    // Four cyclic partners per symbol avoid a full Cartesian training pool.
    // Development retains its original Cartesian pool, seed and construction.
    let train_alphabet: Vec<u8> = (u8::MIN..=u8::MAX)
        .filter(|symbol| !b"EFGH".contains(symbol))
        .collect();
    let train = make_split(
        "train",
        &train_alphabet,
        Some(&[1, 7, 31, 73]),
        4096,
        0x41a9_39c7,
    )?;
    let dev = make_split("dev", b"EFGH", None, 32, 0x734e_129d)?;
    validate(&train, &dev)?;
    Ok((train, dev))
}

// Reproducible offline shuffling, unrelated to model state or serving arithmetic.
struct Rng(u64);

impl Rng {
    fn below(&mut self, upper: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % upper as u64) as usize
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for i in (1..values.len()).rev() {
            let j = self.below(i + 1);
            values.swap(i, j);
        }
    }
}

fn make_split(
    name: &str,
    alphabet: &[u8],
    partner_offsets: Option<&[usize]>,
    groups: usize,
    seed: u64,
) -> Result<Vec<Example>, String> {
    let keys: Vec<[u8; 2]> = match partner_offsets {
        Some(offsets) => alphabet
            .iter()
            .enumerate()
            .flat_map(|(index, symbol)| {
                offsets
                    .iter()
                    .map(move |offset| [*symbol, alphabet[(index + offset) % alphabet.len()]])
            })
            .collect(),
        None => alphabet
            .iter()
            .flat_map(|a| {
                alphabet
                    .iter()
                    .filter(move |b| *b != a)
                    .map(move |b| [*a, *b])
            })
            .collect(),
    };
    let mut rng = Rng(seed);
    let mut result = Vec::with_capacity(groups * 2);
    let mut contexts = BTreeSet::new();
    for group in 0..groups {
        // Advance the query-to-position phase after each ordered-key cycle.
        // A fixed query must not reveal its target slot: all training queries
        // appear in four slots, and every development query in at least two.
        let query = keys[(group + group / keys.len()) % keys.len()];
        let reverse = [query[1], query[0]];
        let target_slot = group % 4;
        // Each slot encounters all eight answers in a 32-family cycle.
        let answer = VALUES[(group / 4) % VALUES.len()];
        let changed_answer = VALUES[((group / 4) + 4) % VALUES.len()];
        let mut accepted = None;
        for _ in 0..256 {
            let mut other_keys: Vec<_> = keys
                .iter()
                .copied()
                .filter(|key| *key != query && *key != reverse)
                .collect();
            rng.shuffle(&mut other_keys);
            let mut distractor_keys = [reverse, other_keys[0], other_keys[1]];
            rng.shuffle(&mut distractor_keys);
            let mut remaining_values: Vec<_> = VALUES
                .iter()
                .copied()
                .filter(|value| *value != answer && *value != changed_answer)
                .collect();
            rng.shuffle(&mut remaining_values);
            let mut distractor_values = [changed_answer, remaining_values[0], remaining_values[1]];
            rng.shuffle(&mut distractor_values);
            let mut records = [Record {
                key: query,
                value: answer,
            }; 4];
            let mut offset = 0;
            for (slot, record) in records.iter_mut().enumerate() {
                if slot != target_slot {
                    *record = Record {
                        key: distractor_keys[offset],
                        value: distractor_values[offset],
                    };
                    offset += 1;
                }
            }
            let Some(swap_slot) = records
                .iter()
                .position(|record| record.value == changed_answer)
            else {
                return Err("changed-source value missing during construction".into());
            };
            let mut changed = records;
            changed[target_slot].value = changed_answer;
            changed[swap_slot].value = answer;
            if !contexts.contains(&(query, records)) && !contexts.contains(&(query, changed)) {
                contexts.insert((query, records));
                contexts.insert((query, changed));
                accepted = Some((records, changed));
                break;
            }
        }
        let Some((records, changed)) = accepted else {
            return Err(format!("could not construct unique {name} family {group}"));
        };
        let family = format!("{name}-family-{group:03}");
        for (variant, records, answer) in [(0, records, answer), (1, changed, changed_answer)] {
            result.push(Example {
                id: format!("{family}-variant-{variant}"),
                records,
                query,
                answer,
                family: family.clone(),
            });
        }
    }
    // Avoid presenting paired variants or cycling labels as an input schedule.
    rng.shuffle(&mut result);
    Ok(result)
}

/// Reject invalid labels, split leakage, changed-source pairing and shortcuts
/// from an unbalanced answer/position distribution. No target index is stored.
pub fn validate(train: &[Example], dev: &[Example]) -> Result<CorpusValidation, String> {
    let train_report = validate_split(train)?;
    let dev_report = validate_split(dev)?;
    let contexts: BTreeSet<_> = train.iter().map(|e| (e.query, e.records)).collect();
    let families: BTreeSet<_> = train.iter().map(|e| e.family.as_str()).collect();
    let ids: BTreeSet<_> = train.iter().map(|e| e.id.as_str()).collect();
    if dev.iter().any(|e| contexts.contains(&(e.query, e.records))) {
        return Err("full context crosses the train/dev split".into());
    }
    if dev.iter().any(|e| families.contains(e.family.as_str())) {
        return Err("paired family crosses the train/dev split".into());
    }
    if dev.iter().any(|e| ids.contains(e.id.as_str())) {
        return Err("example id crosses the train/dev split".into());
    }
    if train_report
        .key_alphabet
        .iter()
        .any(|key| dev_report.key_alphabet.contains(key))
    {
        return Err("development key identities overlap training".into());
    }
    Ok(CorpusValidation {
        train: train_report,
        dev: dev_report,
        disjoint_contexts: true,
        disjoint_families: true,
        disjoint_key_alphabets: true,
    })
}

fn validate_split(examples: &[Example]) -> Result<SplitValidation, String> {
    if examples.is_empty() {
        return Err("empty attention split".into());
    }
    let mut report = SplitValidation {
        examples: examples.len(),
        families: 0,
        answer_counts: [0; 8],
        target_slot_counts: [0; 4],
        answer_counts_by_target_slot: [[0; 8]; 4],
        key_alphabet: Vec::new(),
        query_positions: Vec::new(),
    };
    let mut ids = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    let mut alphabet = BTreeSet::new();
    let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
    let mut query_positions: BTreeMap<[u8; 2], [usize; 4]> = BTreeMap::new();
    for example in examples {
        if !ids.insert(&example.id) || !contexts.insert((example.query, example.records)) {
            return Err(format!("duplicate example/context: {}", example.id));
        }
        let keys: BTreeSet<_> = example.records.iter().map(|r| r.key).collect();
        let values: BTreeSet<_> = example.records.iter().map(|r| r.value).collect();
        if keys.len() != 4 || values.len() != 4 {
            return Err(format!("non-distinct keys/values: {}", example.id));
        }
        for record in example.records {
            if record.key[0] == record.key[1] || !VALUES.contains(&record.value) {
                return Err(format!("invalid key/value: {}", example.id));
            }
            alphabet.extend(record.key);
        }
        if !keys.contains(&[example.query[1], example.query[0]]) {
            return Err(format!("missing order-reversal distractor: {}", example.id));
        }
        let matching: Vec<_> = example
            .records
            .iter()
            .enumerate()
            .filter(|(_, record)| record.key == example.query)
            .collect();
        if matching.len() != 1 || matching[0].1.value != example.answer {
            return Err(format!("incorrect typed answer: {}", example.id));
        }
        let Some(answer_index) = VALUES.iter().position(|v| *v == example.answer) else {
            return Err(format!("answer outside payload alphabet: {}", example.id));
        };
        let slot = matching[0].0;
        report.answer_counts[answer_index] += 1;
        report.target_slot_counts[slot] += 1;
        report.answer_counts_by_target_slot[slot][answer_index] += 1;
        query_positions.entry(example.query).or_default()[slot] += 1;
        families.entry(&example.family).or_default().push(example);
    }
    for (family, pair) in &families {
        if pair.len() != 2 {
            return Err(format!("family must contain two variants: {family}"));
        }
        let (first, second) = (pair[0], pair[1]);
        if first.query != second.query || first.answer == second.answer {
            return Err(format!("invalid changed-source query/answer: {family}"));
        }
        let changed: Vec<_> = first
            .records
            .iter()
            .zip(&second.records)
            .filter(|(a, b)| a.value != b.value)
            .collect();
        if first
            .records
            .iter()
            .zip(&second.records)
            .any(|(a, b)| a.key != b.key)
            || changed.len() != 2
            || changed[0].0.value != changed[1].1.value
            || changed[1].0.value != changed[0].1.value
        {
            return Err(format!("family is not one value swap: {family}"));
        }
    }
    for (query, target_slot_counts) in query_positions {
        if target_slot_counts
            .iter()
            .filter(|count| **count > 0)
            .count()
            < 2
        {
            return Err(format!("query has a fixed target-slot shortcut: {query:?}"));
        }
        report.query_positions.push(QueryPositionCounts {
            query,
            target_slot_counts,
        });
    }
    if report
        .answer_counts
        .iter()
        .any(|n| *n != examples.len() / 8)
        || report
            .target_slot_counts
            .iter()
            .any(|n| *n != examples.len() / 4)
        || report
            .answer_counts_by_target_slot
            .iter()
            .flatten()
            .any(|n| *n != examples.len() / 32)
        || examples.len() % 32 != 0
    {
        return Err("answers and target slots must be jointly balanced".into());
    }
    report.families = families.len();
    report.key_alphabet = alphabet.into_iter().collect();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_attention_corpus_balances_slots_values_and_unseen_keys() -> Result<(), String> {
        let (train, dev) = corpus()?;
        let report = validate(&train, &dev)?;
        assert_eq!((train.len(), dev.len()), (8192, 64));
        assert_eq!((report.train.families, report.dev.families), (4096, 32));
        assert_eq!(report.train.answer_counts, [1024; 8]);
        assert_eq!(report.dev.answer_counts, [8; 8]);
        assert_eq!(report.train.answer_counts_by_target_slot, [[256; 8]; 4]);
        assert_eq!(report.dev.answer_counts_by_target_slot, [[2; 8]; 4]);
        assert_eq!(
            report.train.key_alphabet,
            (u8::MIN..=u8::MAX)
                .filter(|symbol| !b"EFGH".contains(symbol))
                .collect::<Vec<_>>()
        );
        assert_eq!(report.train.key_alphabet.len(), 252);
        assert_eq!(report.dev.key_alphabet, b"EFGH");
        assert_eq!(report.train.query_positions.len(), 1008);
        assert_eq!(report.dev.query_positions.len(), 12);
        for symbol in &report.train.key_alphabet {
            for coordinate in 0..2 {
                assert_eq!(
                    report
                        .train
                        .query_positions
                        .iter()
                        .filter(|row| row.query[coordinate] == *symbol)
                        .count(),
                    4
                );
            }
        }
        assert!(report
            .train
            .query_positions
            .iter()
            .all(|row| row.target_slot_counts.iter().all(|n| *n > 0)));
        assert!(report.dev.query_positions.iter().all(|row| row
            .target_slot_counts
            .iter()
            .filter(|n| **n > 0)
            .count()
            >= 2));
        assert_eq!(corpus()?, (train, dev));
        Ok(())
    }

    #[test]
    fn changed_source_changes_answer_without_changing_query_or_keys() -> Result<(), String> {
        let (train, _) = corpus()?;
        let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
        for example in &train {
            families.entry(&example.family).or_default().push(example);
        }
        for first in &train {
            let second = families
                .get(first.family.as_str())
                .and_then(|pair| pair.iter().find(|e| e.id != first.id))
                .ok_or("missing paired intervention")?;
            assert_eq!(first.query, second.query);
            assert_ne!(first.answer, second.answer);
            assert_eq!(first.records.map(|r| r.key), second.records.map(|r| r.key));
            let mut values_a = first.records.map(|r| r.value);
            let mut values_b = second.records.map(|r| r.value);
            values_a.sort();
            values_b.sort();
            assert_eq!(values_a, values_b);
        }
        Ok(())
    }

    #[test]
    fn validation_rejects_corrupt_answers_and_cross_split_families() -> Result<(), String> {
        let (mut train, mut dev) = corpus()?;
        let original_answer = train[0].answer;
        train[0].answer = b'?';
        assert!(validate(&train, &dev).is_err());
        train[0].answer = original_answer;
        let family = dev[0].family.clone();
        for example in dev.iter_mut().filter(|e| e.family == family) {
            example.family = train[0].family.clone();
        }
        assert!(validate(&train, &dev).is_err());
        Ok(())
    }

    #[test]
    fn validation_rejects_query_identity_as_a_position_shortcut() -> Result<(), String> {
        let (mut train, dev) = corpus()?;
        for example in &mut train {
            let slot = example
                .records
                .iter()
                .position(|record| record.key == example.query)
                .ok_or("missing query record")?;
            example.records.swap(slot, 0);
        }
        let result = validate(&train, &dev);
        assert!(matches!(result, Err(reason) if reason.contains("fixed target-slot shortcut")));
        Ok(())
    }
}
