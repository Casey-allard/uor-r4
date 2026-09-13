//! Offline matched source interventions for byte-feedback span composition.
//! Record bindings are supplied. The keyed trace is an answer oracle, never a
//! runtime input or a claim that XOR addresses encode language semantics.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub type Record = super::super::text_attention::data::Record;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub records: [Record; 4],
    pub query: [u8; 2],
    pub answer: Vec<u8>,
}

fn fold(text: &[u8]) -> u8 {
    text.iter().fold(0, |value, byte| value ^ byte)
}

fn next(key: [u8; 2], text: &[u8]) -> [u8; 2] {
    let delta = fold(text);
    [key[0] ^ delta, key[1] ^ delta]
}

/// Offline expected source path. No path, depth, or terminal label is exposed
/// to inference: terminal means the byte-updated address is absent.
pub fn trace(example: &Example) -> Result<Vec<usize>, String> {
    let mut key = example.query;
    let mut path = Vec::new();
    loop {
        let matches: Vec<_> = example
            .records
            .iter()
            .enumerate()
            .filter(|(_, record)| record.key == key)
            .map(|(slot, _)| slot)
            .collect();
        if matches.len() > 1 {
            return Err("ambiguous duplicate record address".into());
        }
        let Some(&slot) = matches.first() else {
            break;
        };
        if path.contains(&slot) || path.len() >= 4 {
            return Err("cyclic or overlong source path".into());
        }
        path.push(slot);
        key = next(key, &example.records[slot].text);
    }
    if path.is_empty() {
        return Err("initial source is absent".into());
    }
    let output: Vec<_> = path
        .iter()
        .flat_map(|slot| example.records[*slot].text.iter().copied())
        .collect();
    if output != example.answer {
        return Err("answer does not equal the complete source concatenation".into());
    }
    Ok(path)
}

const ADJECTIVES: [&str; 16] = [
    "quick", "vexed", "jazzy", "bright", "calm", "small", "red", "blue", "green", "kind", "shy",
    "wise", "young", "brave", "happy", "quiet",
];
const NOUNS: [&str; 16] = [
    "fox", "owl", "cat", "dog", "bird", "fish", "bear", "hare", "ant", "bee", "wolf", "elk",
    "mole", "newt", "otter", "yak",
];
const VERBS: [&str; 8] = [
    "runs", "rests", "jumps", "waits", "sits", "walks", "hides", "plays",
];
const ADVERBS: [&str; 8] = [
    "quietly", "softly", "calmly", "happily", "slowly", "bravely", "nearby", "outside",
];

fn phrases(
    index: usize,
    attempt: usize,
    depth: usize,
    development: bool,
) -> (Vec<Vec<u8>>, Vec<u8>) {
    let n = index + attempt * 263;
    let article = if development { "the" } else { "a" };
    let adjective = ADJECTIVES[n % ADJECTIVES.len()];
    let noun = NOUNS[(n / 16 + n / 5) % NOUNS.len()];
    let verb = VERBS[(n / 7 + attempt) % VERBS.len()];
    let adverb = ADVERBS[(n / 11 + attempt / 3) % ADVERBS.len()];
    let punctuation = [".", "!", "?"][(index + attempt) % 3];
    let subject = format!("{article} {adjective} {noun} ");
    let spans = match depth {
        2 => vec![subject, format!("{verb} {adverb}{punctuation}")],
        3 => vec![
            subject,
            format!("{verb} "),
            format!("{adverb}{punctuation}"),
        ],
        _ => vec![
            format!("{article} {adjective} "),
            format!("{noun} "),
            format!("{verb} "),
            format!("{adverb}{punctuation}"),
        ],
    };
    let short = format!("{article} {adjective} {noun} {verb}{punctuation}").into_bytes();
    (spans.into_iter().map(String::into_bytes).collect(), short)
}

fn make_split(development: bool, families: usize) -> Result<Vec<Example>, String> {
    let alphabet: Vec<_> = (0..=255u8).filter(|byte| !b"EFGH".contains(byte)).collect();
    let prefix = if development { "dev" } else { "train" };
    let mut answers = BTreeSet::new();
    let mut rows = Vec::with_capacity(families * 2);
    for index in 0..families {
        let slot = index % 4;
        let depth = if development {
            2 + index / 4 % 3
        } else {
            2 + index / 4 % 2
        };
        let query = if development {
            [
                b"EFGH"[index % 4],
                b"EFGH"[(index % 4 + 1 + index / 4 % 3) % 4],
            ]
        } else {
            [
                alphabet[(index * 37) % alphabet.len()],
                alphabet[(index * 37 + 113) % alphabet.len()],
            ]
        };
        let mut accepted = None;
        // Bounded deterministic search only resolves address collisions. It
        // does not consult a model, its predictions, or development scores.
        for attempt in 0..4096 {
            let (spans, short) = phrases(index, attempt, depth, development);
            if short.len() > 32
                || fold(&short) == 0
                || spans.iter().any(|s| s.len() > 32 || fold(s) == 0)
            {
                continue;
            }
            let answer: Vec<_> = spans.iter().flatten().copied().collect();
            if answers.contains(&short) || answers.contains(&answer) {
                continue;
            }
            let mut chain = vec![query];
            for span in &spans {
                chain.push(next(*chain.last().ok_or("empty construction chain")?, span));
            }
            if chain.iter().copied().collect::<BTreeSet<_>>().len() != chain.len() {
                continue;
            }
            if !development
                && chain[..depth]
                    .iter()
                    .flatten()
                    .any(|byte| b"EFGH".contains(byte))
            {
                continue;
            }
            let short_end = next(query, &short);
            if chain[..depth].contains(&short_end) || chain[..depth].contains(&[query[1], query[0]])
            {
                continue;
            }
            let mut records: [Record; 4] = std::array::from_fn(|_| Record {
                key: query,
                text: b"quiet fox rests.".to_vec(),
            });
            for step in 0..depth {
                records[(slot + step) % 4] = Record {
                    key: chain[step],
                    text: spans[step].clone(),
                };
            }
            let mut used: BTreeSet<_> = chain.iter().copied().collect();
            used.insert(short_end);
            // Distractor addresses need not share the reached XOR orbit.
            let mut extra = 0usize;
            for step in depth..4 {
                loop {
                    if extra >= 65536 {
                        return Err("distractor address search exhausted".into());
                    }
                    let key = [(extra / 256) as u8, (extra % 256) as u8];
                    extra += 1;
                    if used.insert(key) {
                        records[(slot + step) % 4].key = key;
                        break;
                    }
                }
            }
            let family = format!("{prefix}-{index:04}");
            let long = Example {
                id: format!("{family}-long"),
                family: family.clone(),
                records: records.clone(),
                query,
                answer: answer.clone(),
            };
            records[slot].text = short.clone();
            let one = Example {
                id: format!("{family}-short"),
                family,
                records,
                query,
                answer: short.clone(),
            };
            if trace(&one)?.len() != 1 || trace(&long)?.len() != depth {
                return Err("constructed path depth mismatch".into());
            }
            answers.insert(short);
            answers.insert(answer);
            accepted = Some([one, long]);
            break;
        }
        rows.extend(
            accepted
                .ok_or_else(|| format!("bounded phrase search exhausted for {prefix}-{index}"))?,
        );
    }
    Ok(rows)
}

pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    let train = make_split(false, 256)?;
    let development = make_split(true, 32)?;
    validate(&train, &development)?;
    Ok((train, development))
}

pub fn validate(train: &[Example], development: &[Example]) -> Result<Value, String> {
    if train.len() != 512 || development.len() != 64 {
        return Err("expected 512 training and 64 development rows".into());
    }
    let mut all_ids = BTreeSet::new();
    let mut split_answers = Vec::new();
    let mut split_contexts = Vec::new();
    let mut split_chars = Vec::new();
    let mut split_families = Vec::new();
    let mut summaries = Vec::new();
    for (split, rows) in [train, development].into_iter().enumerate() {
        let mut families: BTreeMap<&str, Vec<&Example>> = BTreeMap::new();
        let mut depths: BTreeMap<usize, usize> = BTreeMap::new();
        let mut slots = [0usize; 4];
        let mut chars = BTreeSet::new();
        let mut answers = BTreeSet::new();
        let mut contexts = BTreeSet::new();
        let mut initial_symbols = BTreeSet::new();
        let mut reached_symbols = BTreeSet::new();
        let mut max_answer = 0usize;
        for row in rows {
            if !all_ids.insert(row.id.clone()) || !answers.insert(row.answer.clone()) {
                return Err("duplicate example identity or complete answer within split".into());
            }
            if row.answer.is_empty()
                || row.answer.len() > 128
                || row
                    .records
                    .iter()
                    .any(|r| r.text.is_empty() || r.text.len() > 32 || fold(&r.text) == 0)
            {
                return Err("invalid empty, oversized, or zero-fold text".into());
            }
            if row
                .records
                .iter()
                .map(|r| r.key)
                .collect::<BTreeSet<_>>()
                .len()
                != 4
            {
                return Err("duplicate supplied record key".into());
            }
            if row
                .records
                .iter()
                .flat_map(|r| &r.text)
                .any(|b| !b.is_ascii_lowercase() && !b" !.?".contains(b))
            {
                return Err("text is outside the retained writer repertoire".into());
            }
            if row
                .query
                .iter()
                .any(|b| b"EFGH".contains(b) != (split == 1))
            {
                return Err("initial query violates train/development domain separation".into());
            }
            let path = trace(row)?;
            *depths.entry(path.len()).or_default() += 1;
            slots[path[0]] += 1;
            for slot in path {
                reached_symbols.extend(row.records[slot].key);
            }
            initial_symbols.extend(row.query);
            chars.extend(row.answer.iter().copied());
            max_answer = max_answer.max(row.answer.len());
            contexts
                .insert(serde_json::to_vec(&(row.query, &row.records)).map_err(|e| e.to_string())?);
            families.entry(&row.family).or_default().push(row);
        }
        for pair in families.values() {
            if pair.len() != 2 {
                return Err("family must contain exactly one causal pair".into());
            }
            let (a, b) = (pair[0], pair[1]);
            let pa = trace(a)?;
            let pb = trace(b)?;
            let source = pa[0];
            if a.query != b.query
                || pa.len() != 1
                || pb.len() < 2
                || pb[0] != source
                || a.answer == b.answer
                || a.records[source].text == b.records[source].text
            {
                return Err("invalid short/long source intervention".into());
            }
            for slot in 0..4 {
                if a.records[slot].key != b.records[slot].key
                    || (slot != source && a.records[slot] != b.records[slot])
                {
                    return Err("causal pair changed a key or a noninitial record".into());
                }
            }
        }
        if slots.iter().any(|count| *count != rows.len() / 4)
            || (split == 0 && depths.contains_key(&4))
            || (split == 1 && depths.get(&4).copied().unwrap_or(0) < 8)
        {
            return Err("position balance or depth transfer contract failed".into());
        }
        summaries.push(json!({"rows": rows.len(), "pairs": families.len(), "depth_counts": depths, "initial_source_slot_counts": slots, "initial_query_symbols": initial_symbols, "reached_query_symbols": reached_symbols, "max_answer_bytes": max_answer}));
        split_families.push(
            families
                .keys()
                .map(|name| name.to_string())
                .collect::<BTreeSet<_>>(),
        );
        split_answers.push(answers);
        split_contexts.push(contexts);
        split_chars.push(chars);
    }
    if !split_answers[0].is_disjoint(&split_answers[1])
        || !split_contexts[0].is_disjoint(&split_contexts[1])
        || !split_families[0].is_disjoint(&split_families[1])
        || !split_chars[1].is_subset(&split_chars[0])
    {
        return Err("split leakage or unseen development output character".into());
    }
    Ok(
        json!({"schema": 1, "training": summaries[0], "development": summaries[1], "answer_context_family_disjoint": true, "development_characters_in_training": true, "scope": "supplied exact records; grounded span composition; initial EFGH query holdout, XOR-derived later queries unrestricted; no novel prose or semantic metric claim"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recurrent_text_corpus_has_causal_depth_transfer() -> Result<(), String> {
        let (train, dev) = corpus()?;
        let report = validate(&train, &dev)?;
        assert_eq!(report["development"]["depth_counts"]["4"], 8);
        assert_eq!(report["training"]["depth_counts"]["1"], 256);
        Ok(())
    }

    #[test]
    fn recurrent_text_validation_rejects_noninitial_intervention() -> Result<(), String> {
        let (mut train, dev) = corpus()?;
        let initial = trace(&train[0])?[0];
        train[0].records[(initial + 1) % 4].text = b"a fox waits!".to_vec();
        assert!(validate(&train, &dev).is_err());
        Ok(())
    }

    #[test]
    fn recurrent_text_trace_rejects_cycles_and_wrong_answers() -> Result<(), String> {
        let (train, _) = corpus()?;
        let mut row = train[0].clone();
        row.answer.push(b'!');
        assert!(trace(&row).is_err());
        let initial = trace(&train[0])?[0];
        row = train[0].clone();
        row.records[initial].text = b"aa".to_vec();
        assert!(trace(&row).is_err());
        Ok(())
    }
}
