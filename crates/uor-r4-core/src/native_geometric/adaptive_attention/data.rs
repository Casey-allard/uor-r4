//! Offline mixed-depth curriculum with supplied exact records. Terminal bytes
//! and link bytes occupy declared disjoint content domains; this is not prose.
//! `depth` is an evaluation oracle, never an input or direct learner target.

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
    pub depth: usize,
}

fn eligible_link(value: u8) -> bool {
    value != 0 && !VALUES.contains(&(value & 0x6f))
}

/// Offline XOR-composition oracle; production rollouts use the retained learner.
fn next_query(query: [u8; 2], payload: u8) -> [u8; 2] {
    [query[0] ^ payload, query[1] ^ payload]
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
        for index in (1..values.len()).rev() {
            values.swap(index, self.below(index + 1));
        }
    }
}

pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    let alphabet: Vec<_> = (u8::MIN..=u8::MAX)
        .filter(|byte| !b"EFGH".contains(byte))
        .collect();
    let train = make_split("train", &alphabet, 1024, false, 0x382e_731a_642d_c091)?;
    let dev = make_split("dev", b"EFGH", 96, true, 0x513d_42ba_91ef_8027)?;
    validate(&train, &dev)?;
    Ok((train, dev))
}

fn make_split(
    name: &str,
    alphabet: &[u8],
    groups: usize,
    include_four: bool,
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
    let links: Vec<_> = (u8::MIN..=u8::MAX)
        .filter(|value| eligible_link(*value))
        .collect();
    let mut contexts = BTreeSet::new();
    let mut result = Vec::with_capacity(groups * 2);
    let mut offsets = [1, 2, 3];
    for group in 0..groups {
        if group % 4 == 0 {
            rng.shuffle(&mut offsets);
        }
        let initial_slot = group % 4;
        let slots = [
            initial_slot,
            (initial_slot + offsets[0]) % 4,
            (initial_slot + offsets[1]) % 4,
            (initial_slot + offsets[2]) % 4,
        ];
        let depth = if include_four {
            2 + group % 3
        } else {
            2 + (group / 32) % 2
        };
        let query = keys[group % keys.len()];
        let answers = [VALUES[(group / 4) % 8], VALUES[((group / 4) + 4) % 8]];
        // Exhaustive first-link cycling guarantees coverage of the entire
        // permitted action-content domain in training, independently of retries.
        let first_link = links[group % links.len()];
        let mut accepted = None;
        for _ in 0..512 {
            let mut records = [Record {
                key: query,
                value: first_link,
            }; 4];
            let mut visited = BTreeSet::from([query]);
            let mut used_values = BTreeSet::from([first_link]);
            let mut current = query;
            let mut payload = first_link;
            let mut valid = true;
            for hop in 1..depth {
                current = next_query(current, payload);
                if !visited.insert(current) {
                    valid = false;
                    break;
                }
                payload = if hop + 1 == depth {
                    answers[1]
                } else {
                    links[rng.below(links.len())]
                };
                if !used_values.insert(payload) {
                    valid = false;
                    break;
                }
                records[slots[hop]] = Record {
                    key: current,
                    value: payload,
                };
            }
            if !valid {
                continue;
            }
            for slot in slots.iter().skip(depth) {
                let key = [rng.below(256) as u8, rng.below(256) as u8];
                if !visited.insert(key) {
                    valid = false;
                    break;
                }
                let available: Vec<_> = VALUES
                    .iter()
                    .copied()
                    .filter(|value| *value != answers[0] && !used_values.contains(value))
                    .collect();
                let value = available[rng.below(available.len())];
                used_values.insert(value);
                records[*slot] = Record { key, value };
            }
            if !valid {
                continue;
            }
            let mut short = records;
            short[initial_slot].value = answers[0];
            if contexts.contains(&(query, short)) || contexts.contains(&(query, records)) {
                continue;
            }
            contexts.insert((query, short));
            contexts.insert((query, records));
            accepted = Some((short, records));
            break;
        }
        let Some((short, long)) = accepted else {
            return Err(format!(
                "could not construct {name} adaptive family {group}"
            ));
        };
        let family = format!("adaptive-{name}-family-{group:04}");
        for (variant, records) in [short, long].into_iter().enumerate() {
            result.push(Example {
                id: format!("{family}-variant-{variant}"),
                family: family.clone(),
                query,
                records,
                answer: answers[variant],
                depth: if variant == 0 { 1 } else { depth },
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
    if dev.iter().any(|e| {
        contexts.contains(&(e.query, e.records))
            || families.contains(&e.family)
            || ids.contains(&e.id)
    }) {
        return Err("adaptive context, family, or id crosses split".into());
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
    let mut seen_links = BTreeSet::new();
    for example in train {
        let mut query = example.query;
        for _ in 1..example.depth {
            let record = example
                .records
                .iter()
                .find(|record| record.key == query)
                .ok_or("missing training link")?;
            seen_links.insert(record.value);
            query = next_query(query, record.value);
        }
    }
    let eligible: BTreeSet<_> = (u8::MIN..=u8::MAX)
        .filter(|value| eligible_link(*value))
        .collect();
    if seen_links != eligible {
        return Err("training does not cover every eligible link payload".into());
    }
    let train_depths: BTreeMap<usize, usize> =
        train.iter().fold(BTreeMap::new(), |mut counts, e| {
            *counts.entry(e.depth).or_default() += 1;
            counts
        });
    let dev_depths: BTreeMap<usize, usize> = dev.iter().fold(BTreeMap::new(), |mut counts, e| {
        *counts.entry(e.depth).or_default() += 1;
        counts
    });
    if train_depths != BTreeMap::from([(1, 1024), (2, 512), (3, 512)])
        || dev_depths != BTreeMap::from([(1, 96), (2, 32), (3, 32), (4, 32)])
    {
        return Err("mixed-depth corpus size or depth distribution changed".into());
    }
    Ok(json!({ "train": train_report, "dev": dev_report,
        "disjoint_contexts": true, "disjoint_families": true,
        "disjoint_initial_query_alphabets": true, "global_key_alphabet_disjointness_claimed": false,
        "training_eligible_link_payloads": seen_links.len(), "all_eligible_link_payloads_seen": true,
        "inference_fields": ["query", "records"], "offline_oracle_fields": ["depth", "answer", "id", "family"],
        "curriculum": "Terminal a..h versus nonzero link bytes whose retained codec value & 0x6f is outside a..h; supplied exact record boundaries and keys",
        "four_read_depth_training_examples": 0, "final_independent_qualification": "NOT_RUN"
    }))
}

fn validate_split(examples: &[Example]) -> Result<Value, String> {
    if examples.is_empty() {
        return Err("empty adaptive split".into());
    }
    let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    let mut depths = BTreeMap::<usize, usize>::new();
    let mut answers = BTreeMap::<u8, usize>::new();
    let mut initial_slots = [0usize; 4];
    let mut answers_by_depth = BTreeMap::<usize, [usize; 8]>::new();
    let mut initial_slots_by_depth = BTreeMap::<usize, [usize; 4]>::new();
    let mut terminal_slots_by_depth = BTreeMap::<usize, [usize; 4]>::new();
    for example in examples {
        if !ids.insert(&example.id) || !contexts.insert((example.query, example.records)) {
            return Err("duplicate adaptive id or context".into());
        }
        if !(1..=4).contains(&example.depth) || !VALUES.contains(&example.answer) {
            return Err("invalid depth or answer domain".into());
        }
        let keys: BTreeSet<_> = example.records.iter().map(|record| record.key).collect();
        let values: BTreeSet<_> = example.records.iter().map(|record| record.value).collect();
        if keys.len() != 4
            || values.len() != 4
            || example
                .records
                .iter()
                .filter(|r| r.value == example.answer)
                .count()
                != 1
        {
            return Err("record keys or values are not distinct, or answer is ambiguous".into());
        }
        let slot = example
            .records
            .iter()
            .position(|record| record.key == example.query)
            .ok_or("initial key missing")?;
        initial_slots[slot] += 1;
        initial_slots_by_depth.entry(example.depth).or_default()[slot] += 1;
        answers_by_depth.entry(example.depth).or_default()[(example.answer - b'a') as usize] += 1;
        let terminal_slot = example
            .records
            .iter()
            .position(|record| record.value == example.answer)
            .ok_or("answer slot missing")?;
        terminal_slots_by_depth.entry(example.depth).or_default()[terminal_slot] += 1;
        let mut query = example.query;
        let mut visited = BTreeSet::new();
        for hop in 0..example.depth {
            if !visited.insert(query) {
                return Err("trajectory repeats a key".into());
            }
            let record = example
                .records
                .iter()
                .find(|record| record.key == query)
                .ok_or("trajectory leaves memory")?;
            if hop + 1 == example.depth {
                if record.value != example.answer {
                    return Err("declared answer/depth disagrees with exact trajectory".into());
                }
            } else {
                if !eligible_link(record.value) {
                    return Err("intermediate payload enters terminal codec domain".into());
                }
                query = next_query(query, record.value);
            }
        }
        families.entry(&example.family).or_default().push(example);
        *depths.entry(example.depth).or_default() += 1;
        *answers.entry(example.answer).or_default() += 1;
    }
    for family in families.values() {
        if family.len() != 2 {
            return Err("family is not a pair".into());
        }
        let (first, second) = (family[0], family[1]);
        if first.query != second.query
            || first.answer == second.answer
            || (first.depth == 1) == (second.depth == 1)
        {
            return Err("pair does not contrast one read with dependent reads".into());
        }
        let differences: Vec<_> = first
            .records
            .iter()
            .zip(&second.records)
            .filter(|(a, b)| a != b)
            .collect();
        if differences.len() != 1
            || differences[0].0.key != first.query
            || differences[0].1.key != first.query
        {
            return Err("pair changes more than initial payload".into());
        }
    }
    if answers.len() != 8
        || answers.values().any(|count| *count != examples.len() / 8)
        || initial_slots
            .iter()
            .any(|count| *count != examples.len() / 4)
    {
        return Err("answer or initial slot balance violated".into());
    }
    for (depth, count) in &depths {
        if answers_by_depth[depth]
            .iter()
            .any(|value| *value != count / 8)
            || initial_slots_by_depth[depth]
                .iter()
                .any(|value| *value != count / 4)
        {
            return Err("answer or initial slot predicts read depth".into());
        }
    }
    Ok(
        json!({ "examples": examples.len(), "families": families.len(), "depth_counts": depths,
        "answer_counts": answers, "initial_slot_counts": initial_slots,
        "answer_counts_by_depth": answers_by_depth, "initial_slot_counts_by_depth": initial_slots_by_depth,
        "terminal_slot_counts_by_depth": terminal_slots_by_depth,
        "unique_record_keys_and_values": true, "paired_only_initial_payload_changes": true,
        "exact_xor_trajectories_valid": true, "no_repeated_trajectory_keys": true }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_depth_pairs_are_balanced_and_cover_link_domain() -> Result<(), String> {
        let (train, dev) = corpus()?;
        assert_eq!((train.len(), dev.len()), (2048, 192));
        assert!(train.iter().all(|example| example.depth < 4));
        assert_eq!(dev.iter().filter(|example| example.depth == 4).count(), 32);
        assert_eq!(
            validate(&train, &dev)?["all_eligible_link_payloads_seen"],
            true
        );
        Ok(())
    }

    #[test]
    fn validator_rejects_wrong_depth_and_changed_distractor() -> Result<(), String> {
        let (train, dev) = corpus()?;
        let mut wrong_depth = dev.clone();
        wrong_depth[0].depth = if wrong_depth[0].depth == 1 { 2 } else { 1 };
        assert!(validate(&train, &wrong_depth).is_err());
        let mut changed = dev;
        let slot = changed[0]
            .records
            .iter()
            .position(|record| record.key != changed[0].query)
            .ok_or("no distractor")?;
        changed[0].records[slot].value ^= 128;
        assert!(validate(&train, &changed).is_err());
        Ok(())
    }
}
