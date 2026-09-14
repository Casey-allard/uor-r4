//! A lossless mixture of the retained direct and dependent language corpora.
//!
//! This adapter authors no new grammar and consults no model output. Only raw
//! records and prompt bytes are runtime inputs; kind and expected paths are
//! reporting metadata, and final answer bytes are the training supervision.
use super::data as dependent;
use crate::native_geometric::relative_language::data as direct;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub split: String,
    pub kind: String,
    pub records: [Vec<u8>; 4],
    pub prompt: Vec<u8>,
    pub answer: Vec<u8>,
    pub expected_path: Vec<[usize; 2]>,
}

fn direct_row(row: direct::Example) -> Example {
    Example {
        id: format!("direct:{}", row.id),
        family: format!("direct:{}", row.family),
        split: row.split,
        kind: "direct".into(),
        records: row.records,
        prompt: row.question,
        answer: row.answer,
        expected_path: vec![[row.expected_source, row.expected_word]],
    }
}

fn dependent_row(row: dependent::Example) -> Example {
    Example {
        id: format!("dependent:{}", row.id),
        family: format!("dependent:{}", row.family),
        split: row.split,
        kind: "dependent".into(),
        records: row.records,
        prompt: row.prompt,
        answer: row.answer,
        expected_path: row
            .expected_sources
            .into_iter()
            .zip(row.expected_words)
            .map(|(source, word)| [source, word])
            .collect(),
    }
}

fn words(bytes: &[u8]) -> Vec<&[u8]> {
    bytes
        .split(|byte| !byte.is_ascii_alphabetic())
        .filter(|word| !word.is_empty())
        .collect()
}

fn check_rows(rows: &[Example], training: bool) -> Result<Value, String> {
    let expected = if training { 4096 } else { 1536 };
    let mut ids = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut families: BTreeMap<&str, usize> = BTreeMap::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        let path_len = match row.kind.as_str() {
            "direct" => 1,
            "dependent" => 2,
            _ => return Err(format!("{}: unknown corpus kind", row.id)),
        };
        let prefix = format!("{}:", row.kind);
        if !row.id.starts_with(&prefix)
            || !row.family.starts_with(&prefix)
            || !ids.insert(&row.id)
            || !inputs.insert((&row.records, &row.prompt))
            || row.expected_path.len() != path_len
            || row.prompt.is_empty()
            || row.prompt.len() > 256
            || row.prompt.last() != Some(&b'?')
            || row.prompt.iter().filter(|&&byte| byte == b'?').count() != path_len
            || row.answer.is_empty()
            || row.answer.len() > 16
            || !row.answer.iter().all(u8::is_ascii_lowercase)
        {
            return Err(format!(
                "{}: invalid mixed-row identity, path or raw prompt",
                row.id
            ));
        }
        if if training {
            row.split != "train"
        } else {
            !matches!(row.split.as_str(), "lexical" | "syntactic" | "joint")
        } {
            return Err(format!("{}: row is in the wrong split", row.id));
        }
        for bytes in row.records.iter().map(Vec::as_slice).chain(
            row.prompt
                .split_inclusive(|&byte| byte == b'?')
                .map(|bytes| bytes.strip_prefix(b" ").unwrap_or(bytes)),
        ) {
            let tokens = words(bytes);
            if bytes.len() > 128
                || tokens.is_empty()
                || tokens.len() > 16
                || tokens
                    .iter()
                    .any(|word| word.len() > 16 || !word.iter().all(u8::is_ascii_lowercase))
            {
                return Err(format!(
                    "{}: mixed-row lexical window exceeds its source bounds",
                    row.id
                ));
            }
        }
        for &[source, word] in &row.expected_path {
            if row
                .records
                .get(source)
                .and_then(|record| words(record).get(word).copied())
                .is_none()
            {
                return Err(format!(
                    "{}: expected path is outside its raw record",
                    row.id
                ));
            }
        }
        let &[source, word] = row.expected_path.last().ok_or("empty expected path")?;
        if words(&row.records[source])[word] != row.answer.as_slice() {
            return Err(format!(
                "{}: final expected occurrence differs from answer",
                row.id
            ));
        }
        *families.entry(&row.family).or_default() += 1;
        *counts
            .entry(format!("{}/{}", row.kind, row.split))
            .or_default() += 1;
    }
    if rows.len() != expected
        || families.len() != expected / 16
        || families.values().any(|&count| count != 16)
    {
        return Err("mixed corpus dropped, duplicated or regrouped source examples".into());
    }
    for kind in ["direct", "dependent"] {
        for split in if training {
            &["train"][..]
        } else {
            &["lexical", "syntactic", "joint"][..]
        } {
            if counts.get(&format!("{kind}/{split}")) != Some(&(if training { 2048 } else { 256 }))
            {
                return Err(format!("mixed corpus count differs for {kind}/{split}"));
            }
        }
    }
    Ok(
        json!({"examples":rows.len(),"families":families.len(),"counts":counts,
        "unique_ids":ids.len(),"unique_raw_inputs":inputs.len(),"all_source_examples_retained":true}),
    )
}

pub fn corpus() -> Result<(Vec<Example>, Vec<Example>), String> {
    // Both source constructors run their independent typed validation before
    // returning rows. Mapping does not inspect correctness or filter examples.
    let (direct_train, direct_dev) = direct::corpus()?;
    let (dependent_train, dependent_dev) = dependent::corpus()?;
    let train: Vec<_> = direct_train
        .into_iter()
        .map(direct_row)
        .chain(dependent_train.into_iter().map(dependent_row))
        .collect();
    let dev: Vec<_> = direct_dev
        .into_iter()
        .map(direct_row)
        .chain(dependent_dev.into_iter().map(dependent_row))
        .collect();
    check_rows(&train, true)?;
    check_rows(&dev, false)?;
    let mut ids = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    for row in train.iter().chain(&dev) {
        if !ids.insert(&row.id) || !inputs.insert((&row.records, &row.prompt)) {
            return Err(format!(
                "{}: duplicate identity or raw input across splits",
                row.id
            ));
        }
    }
    Ok((train, dev))
}

pub fn validate() -> Result<Value, String> {
    let source_direct = direct::validate()?;
    let source_dependent = dependent::validate()?;
    let (train, dev) = corpus()?;
    Ok(json!({
        "train":check_rows(&train,true)?,"development":check_rows(&dev,false)?,
        "source_validations":{"direct":source_direct,"dependent":source_dependent},
        "construction":"lossless concatenation of the existing relative-language and dependent-language corpora",
        "inference_inputs":["records","prompt"],"training_supervision":["answer"],
        "reporting_only":["id","family","split","kind","expected_path"],
        "new_grammar":false,"model_output_filtering":false,"final_held_out":"NOT_RUN"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_schedule_corpus_preserves_raw_inputs_and_one_or_two_read_paths() -> Result<(), String>
    {
        let (train, dev) = corpus()?;
        assert_eq!(train.len(), 4096);
        assert_eq!(dev.len(), 1536);
        let (direct_train, _) = direct::corpus()?;
        let (dependent_train, _) = dependent::corpus()?;
        assert_eq!(train[0], direct_row(direct_train[0].clone()));
        assert_eq!(train[2048], dependent_row(dependent_train[0].clone()));
        assert_eq!(train[0].expected_path.len(), 1);
        assert_eq!(train[2048].expected_path.len(), 2);
        assert_eq!(train[0].prompt, direct_train[0].question);
        assert_eq!(train[2048].prompt, dependent_train[0].prompt);
        let mut corrupted = train.clone();
        corrupted[2048].expected_path.pop();
        assert!(check_rows(&corrupted, true).is_err());
        Ok(())
    }
}
