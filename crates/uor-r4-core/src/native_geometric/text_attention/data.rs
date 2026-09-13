//! Offline grounded span-copy curriculum. Exact record bindings are supplied;
//! the English fragments are context text, not a claim of novel prose.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub key: [u8; 2],
    pub text: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub query: [u8; 2],
    pub records: [Record; 4],
    pub answer: Vec<u8>,
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

fn train_texts() -> Vec<Vec<u8>> {
    let adjectives = [
        "quick", "vexed", "jazzy", "bright", "calm", "small", "red", "blue", "green", "kind",
        "shy", "wise", "young", "brave", "happy", "quiet",
    ];
    let nouns = [
        "fox", "owl", "cat", "dog", "bird", "fish", "bear", "hare", "ant", "bee", "wolf", "elk",
        "mole", "newt", "otter", "yak",
    ];
    let verbs = ["runs", "rests", "jumps", "waits"];
    let mut texts: Vec<Vec<u8>> = (b'a'..=b'z').map(|byte| vec![byte]).collect();
    for index in 0..230 {
        texts.push(
            format!(
                "{} {} {}{}",
                adjectives[(index % 16 + index / 16) % 16],
                nouns[index / 16],
                verbs[(index / 4) % 4],
                [".", "!", "?"][index % 3]
            )
            .into_bytes(),
        );
    }
    texts
}

fn dev_texts() -> Vec<Vec<u8>> {
    // Novel combinations, including lengths beyond every training span. Words
    // and characters remain familiar; exact complete spans are held apart.
    [
        "?",
        "!",
        "ok",
        "go",
        "the fox runs.",
        "a bird waits!",
        "the owl rests?",
        "a cat jumps.",
        "the quick fox quietly runs.",
        "the vexed owl quietly rests.",
        "the jazzy cat quietly jumps!",
        "the bright dog quietly waits?",
        "the calm bird quietly runs.",
        "the small fish quietly rests.",
        "the red bear quietly jumps!",
        "the blue hare quietly waits?",
        "the green ant quietly runs.",
        "the kind bee quietly rests.",
        "the shy wolf quietly jumps!",
        "the wise elk quietly waits?",
        "the young mole quietly runs.",
        "the brave newt quietly rests.",
        "the happy otter quietly jumps!",
        "the quiet yak quietly waits?",
        "the bright otter quietly jumps.",
        "the happy young fox runs.",
        "the small blue bird waits!",
        "the brave green ant rests?",
        "the quiet young owl jumps.",
        "the bright otter quietly jumps!!",
        "the bright small dog runs!",
        "the jazzy happy cat rests.",
    ]
    .into_iter()
    .map(|text| text.as_bytes().to_vec())
    .collect()
}

pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    let alphabet: Vec<_> = (u8::MIN..=u8::MAX)
        .filter(|byte| !b"EFGH".contains(byte))
        .collect();
    let mut rng = Rng(0x625a_74cb_4f19_8173);
    let mut shuffled = alphabet.clone();
    rng.shuffle(&mut shuffled);
    // Every training query occupies all four source positions. Its bytes span
    // the broad training alphabet rather than an alphabetic instruction code.
    let queries: Vec<_> = (0..64)
        .map(|index| [shuffled[index], shuffled[index + 64]])
        .collect();
    let development_queries: Vec<_> = b"EFGH"
        .iter()
        .flat_map(|first| {
            b"EFGH"
                .iter()
                .filter(move |second| *second != first)
                .map(move |second| [*first, *second])
        })
        .collect();
    let train = make_split(
        "train",
        &queries,
        &alphabet,
        &train_texts(),
        256,
        0x39ab_4067_f851_827d,
    )?;
    let dev = make_split(
        "dev",
        &development_queries,
        b"EFGH",
        &dev_texts(),
        32,
        0x468c_15da_b8e2_7531,
    )?;
    validate(&train, &dev)?;
    Ok((train, dev))
}

fn make_split(
    name: &str,
    queries: &[[u8; 2]],
    alphabet: &[u8],
    texts: &[Vec<u8>],
    families: usize,
    seed: u64,
) -> Result<Vec<Example>, String> {
    if texts.len() != families || queries.is_empty() || alphabet.len() < 4 {
        return Err("invalid text curriculum configuration".into());
    }
    let mut rng = Rng(seed);
    let mut rows = Vec::with_capacity(families * 2);
    for family_index in 0..families {
        let target = family_index % 4;
        let query = if name == "train" {
            queries[family_index / 4]
        } else {
            queries[(family_index / 4 + target * 3) % queries.len()]
        };
        let reverse = [query[1], query[0]];
        let other = (target + 1 + rng.below(3)) % 4;
        let mut keys = BTreeSet::from([query, reverse]);
        let mut extra = Vec::new();
        while extra.len() < 2 {
            let key = [
                alphabet[rng.below(alphabet.len())],
                alphabet[rng.below(alphabet.len())],
            ];
            if keys.insert(key) {
                extra.push(key);
            }
        }
        let text_offset = (family_index / 4) * 4;
        let mut records: [Record; 4] = std::array::from_fn(|slot| Record {
            key: query,
            text: texts[text_offset + (slot + 4 - target) % 4].clone(),
        });
        records[target].key = query;
        records[other].key = reverse;
        let mut extra_index = 0;
        for (slot, record) in records.iter_mut().enumerate() {
            if slot != target && slot != other {
                record.key = extra[extra_index];
                extra_index += 1;
            }
        }
        // Fix the two target answers across all four positions in each group.
        // The two remaining texts are distractors, distinct from both answers.
        records[target].text = texts[text_offset].clone();
        records[other].text = texts[text_offset + 1].clone();
        let mut text_index = 2;
        for (slot, record) in records.iter_mut().enumerate() {
            if slot != target && slot != other {
                record.text = texts[text_offset + text_index].clone();
                text_index += 1;
            }
        }
        let family = format!("{name}-{family_index:04}");
        rows.push(Example {
            id: format!("{family}-a"),
            family: family.clone(),
            query,
            answer: records[target].text.clone(),
            records: records.clone(),
        });
        let replacement = records[other].text.clone();
        records[other].text = records[target].text.clone();
        records[target].text = replacement;
        rows.push(Example {
            id: format!("{family}-b"),
            family,
            query,
            answer: records[target].text.clone(),
            records,
        });
    }
    Ok(rows)
}

pub fn validate(train: &[Example], dev: &[Example]) -> Result<Value, String> {
    if train.len() != 512 || dev.len() != 64 {
        return Err("unexpected curriculum row count".into());
    }
    let mut ids = BTreeSet::new();
    let mut split_texts = Vec::new();
    let mut split_contexts = Vec::new();
    let mut split_characters = Vec::new();
    let mut split_families = Vec::new();
    let mut statistics = Vec::new();
    for (name, examples) in [("train", train), ("dev", dev)] {
        let mut texts = BTreeSet::new();
        let mut contexts = BTreeSet::new();
        let mut characters = BTreeSet::new();
        let mut lengths = BTreeMap::<usize, usize>::new();
        let mut families = BTreeMap::<&str, Vec<&Example>>::new();
        let mut slots = [0_usize; 4];
        let mut query_slots = BTreeMap::<[u8; 2], BTreeSet<usize>>::new();
        let mut answer_slots = BTreeMap::<Vec<u8>, [usize; 4]>::new();
        for example in examples {
            if !ids.insert(example.id.clone()) {
                return Err("duplicate row ID".into());
            }
            let keys: BTreeSet<_> = example.records.iter().map(|record| record.key).collect();
            let values: BTreeSet<_> = example
                .records
                .iter()
                .map(|record| record.text.clone())
                .collect();
            if keys.len() != 4
                || values.len() != 4
                || example.query[0] == example.query[1]
                || !keys.contains(&[example.query[1], example.query[0]])
            {
                return Err("missing unique query/reverse-key distractor records".into());
            }
            let target = example
                .records
                .iter()
                .position(|record| record.key == example.query)
                .ok_or("query has no record")?;
            if example.records[target].text != example.answer {
                return Err("answer differs from exact source oracle".into());
            }
            slots[target] += 1;
            query_slots.entry(example.query).or_default().insert(target);
            answer_slots.entry(example.answer.clone()).or_insert([0; 4])[target] += 1;
            for record in &example.records {
                if record.text.is_empty() || record.text.len() > 32 || !record.text.is_ascii() {
                    return Err("invalid ASCII span length".into());
                }
                texts.insert(record.text.clone());
                characters.extend(record.text.iter().copied());
            }
            *lengths.entry(example.answer.len()).or_default() += 1;
            let context = serde_json::to_vec(&(example.query, &example.records))
                .map_err(|error| error.to_string())?;
            if !contexts.insert(context) {
                return Err("duplicate exact context".into());
            }
            families.entry(&example.family).or_default().push(example);
        }
        for pair in families.values() {
            if pair.len() != 2 || pair[0].query != pair[1].query || pair[0].answer == pair[1].answer
            {
                return Err("invalid changed-source pair".into());
            }
            let a = pair[0];
            let b = pair[1];
            if a.records
                .iter()
                .zip(&b.records)
                .any(|(left, right)| left.key != right.key)
            {
                return Err("pair changed record keys".into());
            }
            let changed: Vec<_> = (0..4)
                .filter(|slot| a.records[*slot].text != b.records[*slot].text)
                .collect();
            if changed.len() != 2
                || a.records[changed[0]].text != b.records[changed[1]].text
                || a.records[changed[1]].text != b.records[changed[0]].text
            {
                return Err("pair is not exactly one payload swap".into());
            }
        }
        if slots.iter().any(|count| *count != examples.len() / 4)
            || answer_slots
                .values()
                .any(|counts| counts.iter().any(|count| *count != counts[0]))
        {
            return Err("answer/source positions are not balanced".into());
        }
        if query_slots.values().any(|positions| positions.len() < 2) {
            return Err("query predicts a unique source slot".into());
        }
        statistics.push(json!({"split":name,"rows":examples.len(),"paired_families":families.len(),"unique_spans":texts.len(),"answer_lengths":lengths,"source_slot_counts":slots,"query_count":query_slots.len(),"minimum_positions_per_query":query_slots.values().map(BTreeSet::len).min(),"characters":characters.iter().copied().collect::<Vec<_>>() }));
        split_families.push(
            families
                .keys()
                .map(|family| family.to_string())
                .collect::<BTreeSet<_>>(),
        );
        split_texts.push(texts);
        split_contexts.push(contexts);
        split_characters.push(characters);
    }
    if !split_texts[0].is_disjoint(&split_texts[1])
        || !split_contexts[0].is_disjoint(&split_contexts[1])
        || !split_families[0].is_disjoint(&split_families[1])
    {
        return Err("training/development overlap".into());
    }
    if !split_characters[1].is_subset(&split_characters[0]) {
        return Err("development contains untrained character".into());
    }
    let training_answer_characters: BTreeSet<_> = train
        .iter()
        .flat_map(|example| example.answer.iter().copied())
        .collect();
    if !split_characters[1].is_subset(&training_answer_characters) {
        return Err("development contains character absent from training answers".into());
    }
    if train
        .iter()
        .flat_map(|example| &example.records)
        .any(|record| record.text.len() > 24)
        || !dev.iter().any(|example| example.answer.len() > 24)
    {
        return Err("missing declared length-transfer boundary".into());
    }
    Ok(
        json!({"status":"PASS_TYPED_SPAN_ENVIRONMENT", "supplied_exact_record_bindings":true,"general_prose":false,"development_characters_seen_in_training":true,"all_spans_disjoint_across_splits":true,"statistics":statistics}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reproducible_balanced_causal_text_pairs() -> Result<(), String> {
        let (train, dev) = corpus()?;
        assert_eq!(corpus()?, (train.clone(), dev.clone()));
        validate(&train, &dev)?;
        assert_eq!(train.len(), 512);
        assert_eq!(dev.len(), 64);
        Ok(())
    }
    #[test]
    fn rejects_answer_leakage_and_broken_pair() -> Result<(), String> {
        let (mut train, dev) = corpus()?;
        train[0].answer = b"wrong".to_vec();
        assert!(validate(&train, &dev).is_err());
        let (train, mut dev) = corpus()?;
        dev[1].records.swap(0, 1);
        assert!(validate(&train, &dev).is_err());
        Ok(())
    }
    #[test]
    fn novel_longer_spans_reuse_training_character_repertoire() -> Result<(), String> {
        let (train, dev) = corpus()?;
        let chars: BTreeSet<_> = train
            .iter()
            .flat_map(|row| {
                row.records
                    .iter()
                    .flat_map(|record| record.text.iter().copied())
            })
            .collect();
        assert!(dev
            .iter()
            .flat_map(|row| &row.answer)
            .all(|byte| chars.contains(byte)));
        assert!(dev.iter().any(|row| row.answer.len() > 24));
        assert!(dev
            .iter()
            .flat_map(|row| &row.records)
            .any(|record| record.text.len() == 32));
        Ok(())
    }
}
