//! Offline typed dependent-read curriculum. Only query and records are inputs.
//!
//! `next_query` and `expected_query` are validation oracles, never direct learner
//! targets. The learner must derive update supervision from actual suffix answer
//! utility. Pair members differ only in the first addressed record's payload.

pub use crate::native_geometric::relational_attention::data::Record;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const VALUES: &[u8; 8] = b"abcdefgh";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub query: [u8; 2],
    pub records: [Record; 4],
    pub answer: u8,
    pub next_query: [u8; 2],
}

/// Offline task definition. Inference and learner targets must not call this.
pub fn expected_query(query: [u8; 2], payload: u8) -> [u8; 2] {
    [query[0] ^ payload, query[1] ^ payload]
}

/// The retained byte codec maps a first generated symbol to `value & 0x6f`.
/// Excluding answer-like payloads prevents a first-record read from receiving
/// positive suffix utility accidentally. This is a codec-domain curriculum
/// condition, not a claim that the retained codec handles arbitrary bytes.
fn eligible_payload(value: u8) -> bool {
    value != 0 && !VALUES.contains(&(value & 0x6f))
}

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

/// Fixed open-development data, not final independent qualification.
pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    let alphabet: Vec<_> = (u8::MIN..=u8::MAX)
        .filter(|value| !b"EFGH".contains(value))
        .collect();
    let train = make_split("train", &alphabet, 1024, 0x87c2_5a19_4fb3_601d)?;
    let dev = make_split("dev", b"EFGH", 64, 0xa215_76cd_890e_43bf)?;
    validate(&train, &dev)?;
    Ok((train, dev))
}

fn make_split(
    name: &str,
    alphabet: &[u8],
    groups: usize,
    seed: u64,
) -> Result<Vec<Example>, String> {
    let mut rng = Rng(seed);
    let mut keys: Vec<_> = if alphabet.len() > 4 {
        alphabet
            .iter()
            .enumerate()
            .flat_map(|(index, first)| {
                [1, 7, 31, 73]
                    .into_iter()
                    .map(move |offset| [*first, alphabet[(index + offset) % alphabet.len()]])
            })
            .collect()
    } else {
        alphabet
            .iter()
            .flat_map(|first| {
                alphabet
                    .iter()
                    .filter(move |second| *second != first)
                    .map(move |second| [*first, *second])
            })
            .collect()
    };
    rng.shuffle(&mut keys);
    let payloads: Vec<_> = (u8::MIN..=u8::MAX)
        .filter(|value| eligible_payload(*value))
        .collect();
    let mut contexts = BTreeSet::new();
    let mut result = Vec::with_capacity(groups * 2);
    for group in 0..groups {
        // Rotate every semantic role through every slot in each four-family
        // block. Alternate candidate offsets to cover every distinct first /
        // second slot pair while preserving exact answer-by-slot balance.
        let first_slot = group % 4;
        let offsets = if (group / 32) % 2 == 0 {
            [1, 2, 3]
        } else {
            [2, 3, 1]
        };
        let final_slots = [(first_slot + offsets[0]) % 4, (first_slot + offsets[1]) % 4];
        let distractor_slot = (first_slot + offsets[2]) % 4;
        let query = keys[(group + group / keys.len()) % keys.len()];
        let answers = [VALUES[(group / 4) % 8], VALUES[((group / 4) + 4) % 8]];
        let mut accepted = None;
        for _ in 0..256 {
            let first_payload = payloads[rng.below(payloads.len())];
            let second_payload = payloads[rng.below(payloads.len())];
            if first_payload == second_payload {
                continue;
            }
            let next_keys = [
                expected_query(query, first_payload),
                expected_query(query, second_payload),
            ];
            let distractor_key = [rng.below(256) as u8, rng.below(256) as u8];
            if [query, next_keys[0], next_keys[1]].contains(&distractor_key) {
                continue;
            }
            let distractor_values: Vec<_> = VALUES
                .iter()
                .copied()
                .filter(|value| !answers.contains(value))
                .collect();
            let mut records = [Record {
                key: query,
                value: first_payload,
            }; 4];
            records[final_slots[0]] = Record {
                key: next_keys[0],
                value: answers[0],
            };
            records[final_slots[1]] = Record {
                key: next_keys[1],
                value: answers[1],
            };
            records[distractor_slot] = Record {
                key: distractor_key,
                value: distractor_values[rng.below(distractor_values.len())],
            };
            let mut changed = records;
            changed[first_slot].value = second_payload;
            if contexts.contains(&(query, records)) || contexts.contains(&(query, changed)) {
                continue;
            }
            contexts.insert((query, records));
            contexts.insert((query, changed));
            accepted = Some((records, changed, next_keys));
            break;
        }
        let Some((records, changed, next_keys)) = accepted else {
            return Err(format!(
                "could not construct {name} dependent family {group}"
            ));
        };
        let family = format!("dependent-{name}-family-{group:04}");
        for (variant, records) in [records, changed].into_iter().enumerate() {
            result.push(Example {
                id: format!("{family}-variant-{variant}"),
                family: family.clone(),
                query,
                records,
                answer: answers[variant],
                next_query: next_keys[variant],
            });
        }
    }
    rng.shuffle(&mut result);
    Ok(result)
}

pub fn validate(train: &[Example], dev: &[Example]) -> Result<Value, String> {
    let train_report = validate_split(train)?;
    let dev_report = validate_split(dev)?;
    let contexts: BTreeSet<_> = train.iter().map(|e| (e.query, e.records)).collect();
    let families: BTreeSet<_> = train.iter().map(|e| &e.family).collect();
    let ids: BTreeSet<_> = train.iter().map(|e| &e.id).collect();
    if dev.iter().any(|e| contexts.contains(&(e.query, e.records)))
        || dev.iter().any(|e| families.contains(&e.family))
        || dev.iter().any(|e| ids.contains(&e.id))
    {
        return Err("dependent context, family, or id crosses split".into());
    }
    if train
        .iter()
        .any(|e| e.query.iter().any(|v| b"EFGH".contains(v)))
        || dev
            .iter()
            .any(|e| e.query.iter().any(|v| !b"EFGH".contains(v)))
    {
        return Err("initial-query alphabet holdout violated".into());
    }
    Ok(json!({
        "train": train_report,
        "dev": dev_report,
        "disjoint_contexts": true,
        "disjoint_families": true,
        "disjoint_initial_query_alphabets": true,
        "global_key_alphabet_disjointness_claimed": false,
        "inference_fields": ["query", "records"],
        "offline_oracle_fields": ["next_query", "answer", "id", "family"],
        "suffix_utility_uniqueness": "retained codec first symbol value & 0x6f; actual suffix outputs require separate prefit verification",
        "final_independent_qualification": "NOT_RUN"
    }))
}

fn validate_split(examples: &[Example]) -> Result<Value, String> {
    if examples.is_empty() || examples.len() % 64 != 0 {
        return Err("dependent split must contain nonzero complete balanced blocks".into());
    }
    let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
    let mut contexts = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut answer_counts = [0usize; 8];
    let mut first_slot_counts = [0usize; 4];
    let mut final_slot_counts = [0usize; 4];
    let mut answer_by_first_slot = [[0usize; 8]; 4];
    let mut answer_by_final_slot = [[0usize; 8]; 4];
    let mut first_final_counts = [[0usize; 4]; 4];
    let mut query_bytes = [BTreeSet::new(), BTreeSet::new()];
    let mut payload_values = BTreeSet::new();
    for example in examples {
        if !ids.insert(&example.id) || !contexts.insert((example.query, example.records)) {
            return Err("duplicate dependent id or context".into());
        }
        let keys: BTreeSet<_> = example.records.iter().map(|r| r.key).collect();
        let values: BTreeSet<_> = example.records.iter().map(|r| r.value).collect();
        if keys.len() != 4 || values.len() != 4 || example.query[0] == example.query[1] {
            return Err(
                "dependent record keys/values must be unique and query coordinates distinct".into(),
            );
        }
        let Some(first) = example.records.iter().position(|r| r.key == example.query) else {
            return Err("initial query has no source".into());
        };
        let payload = example.records[first].value;
        if !eligible_payload(payload)
            || expected_query(example.query, payload) != example.next_query
        {
            return Err(
                "invalid dependent update oracle or ambiguous first-read codec value".into(),
            );
        }
        let Some(last) = example
            .records
            .iter()
            .position(|r| r.key == example.next_query)
        else {
            return Err("dependent query has no source".into());
        };
        let Some(answer_index) = VALUES.iter().position(|value| *value == example.answer) else {
            return Err("dependent answer outside codec repertoire".into());
        };
        if first == last
            || example.records[last].value != example.answer
            || example
                .records
                .iter()
                .filter(|r| (r.value & 0x6f) == example.answer)
                .count()
                != 1
        {
            return Err("dependent final answer or unique forced-suffix utility violated".into());
        }
        answer_counts[answer_index] += 1;
        first_slot_counts[first] += 1;
        final_slot_counts[last] += 1;
        answer_by_first_slot[first][answer_index] += 1;
        answer_by_final_slot[last][answer_index] += 1;
        first_final_counts[first][last] += 1;
        query_bytes[0].insert(example.query[0]);
        query_bytes[1].insert(example.query[1]);
        payload_values.insert(payload);
        families.entry(&example.family).or_default().push(example);
    }
    for members in families.values() {
        if members.len() != 2 {
            return Err("dependent family must have exactly two interventions".into());
        }
        let a = members[0];
        let b = members[1];
        let changed: Vec<_> = (0..4)
            .filter(|slot| a.records[*slot] != b.records[*slot])
            .collect();
        if a.query != b.query
            || a.answer == b.answer
            || a.next_query == b.next_query
            || changed.len() != 1
            || a.records[changed[0]].key != a.query
            || b.records[changed[0]].key != b.query
        {
            return Err("paired intervention must change only the initial-source payload".into());
        }
    }
    let n = examples.len();
    if answer_counts != [n / 8; 8]
        || first_slot_counts != [n / 4; 4]
        || final_slot_counts != [n / 4; 4]
        || answer_by_first_slot != [[n / 32; 8]; 4]
        || answer_by_final_slot != [[n / 32; 8]; 4]
        || (0..4)
            .any(|first| (0..4).any(|last| first != last && first_final_counts[first][last] == 0))
    {
        return Err(
            "dependent answer/slot balance or first-to-final slot coverage violated".into(),
        );
    }
    Ok(json!({
        "examples": n,
        "families": families.len(),
        "answer_counts": answer_counts,
        "initial_slot_counts": first_slot_counts,
        "final_slot_counts": final_slot_counts,
        "answer_by_initial_slot": answer_by_first_slot,
        "answer_by_final_slot": answer_by_final_slot,
        "initial_to_final_slot_counts": first_final_counts,
        "initial_query_coordinate_alphabets": query_bytes,
        "first_payload_values": payload_values,
        "all_four_keys_and_raw_values_unique": true,
        "exactly_one_first_source_payload_changed_per_pair": true,
        "unique_forced_suffix_utility_under_retained_codec_map": true
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paired_dependent_corpus_is_balanced_and_causal() -> Result<(), String> {
        let (train, dev) = corpus()?;
        assert_eq!(train.len(), 2048);
        assert_eq!(dev.len(), 128);
        assert_eq!(corpus()?, (train.clone(), dev.clone()));
        let report = validate(&train, &dev)?;
        assert_eq!(report["train"]["families"], 1024);
        assert_eq!(report["dev"]["families"], 64);
        Ok(())
    }

    #[test]
    fn validation_rejects_changed_oracle_and_pair_pollution() -> Result<(), String> {
        let (train, mut dev) = corpus()?;
        dev[0].next_query[0] ^= 1;
        assert!(validate(&train, &dev).is_err());
        let (_, mut dev) = corpus()?;
        let initial = dev[0]
            .records
            .iter()
            .position(|r| r.key == dev[0].query)
            .ok_or("missing initial source")?;
        let other = (initial + 1) % 4;
        dev[0].records[other].key[0] ^= 1;
        assert!(validate(&train, &dev).is_err());
        Ok(())
    }

    #[test]
    fn update_requires_both_query_coordinates_and_payload() {
        let query = [17, 231];
        let payload = 58;
        let baseline = expected_query(query, payload);
        for bit in 0..8 {
            assert_ne!(
                expected_query([query[0] ^ (1 << bit), query[1]], payload),
                baseline
            );
            assert_ne!(
                expected_query([query[0], query[1] ^ (1 << bit)], payload),
                baseline
            );
            let changed = expected_query(query, payload ^ (1 << bit));
            assert_ne!(changed[0], baseline[0]);
            assert_ne!(changed[1], baseline[1]);
        }
    }
}
