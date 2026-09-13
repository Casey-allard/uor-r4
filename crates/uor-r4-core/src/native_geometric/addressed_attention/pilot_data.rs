//! Frozen, authored pilot data. This is offline preparation, never serving logic.
//! Each record is a distinct example/reset; strings do not create host turns.
use super::engine::EngineError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Example {
    pub id: String,
    pub family: String,
    pub prompt: Vec<u8>,
    /// Exact continuation bytes. EOS is a separate final scored symbol.
    pub answer: Vec<u8>,
    pub expected_scalar: Option<i64>,
}

fn memory(id: &str, family: &str, prompt: &str, owner: &str, value: &str) -> Example {
    // The pre-existing typed oracle authors the bare Current form exactly once.
    // Selection among oracle forms is frozen here; evaluation never invents an
    // alternate answer or derives an answer from a candidate's generated text.
    let answer: String =
        crate::answer_oracle::accepted(crate::answer_oracle::Intent::Current, true, owner, value)
            .into_iter()
            .take(1)
            .collect();
    Example {
        id: id.into(),
        family: family.into(),
        prompt: prompt.as_bytes().to_vec(),
        answer: answer.into_bytes(),
        expected_scalar: None,
    }
}

#[derive(Clone, Copy)]
enum Arithmetic {
    Add,
    Sub,
}
fn program(id: &str, family: &str, a: i8, b: i8, action: Arithmetic) -> Example {
    // Typed intent supplies the one exact scalar and target program offline.
    // i8 inputs make these i64 operations intrinsically within range.
    let (operator, scalar) = match action {
        Arithmetic::Add => ('+', i64::from(a) + i64::from(b)),
        Arithmetic::Sub => ('-', i64::from(a) - i64::from(b)),
    };
    let prompt = format!("Rust: print a{operator}b; a={a},b={b}.\n").into_bytes();
    let answer = format!("fn main(){{println!(\"{{}}\",{scalar});}}\n").into_bytes();
    Example {
        id: id.into(),
        family: family.into(),
        prompt,
        answer,
        expected_scalar: Some(scalar),
    }
}

/// Eight adjacent controlled pairs; their families are declared, not inferred.
pub fn training() -> Vec<Example> {
    vec![
        memory(
            "train/memory-a/0",
            "memory_same_query_changed_source",
            "Ada is in Lima. Ben is in Oslo.\nAda?\n",
            "Ada",
            "Lima",
        ),
        memory(
            "train/memory-a/1",
            "memory_same_query_changed_source",
            "Ada is in Oslo. Ben is in Lima.\nAda?\n",
            "Ada",
            "Oslo",
        ),
        memory(
            "train/memory-b/0",
            "memory_same_query_changed_source",
            "Ada is in Lima. Ben is in Oslo.\nBen?\n",
            "Ben",
            "Oslo",
        ),
        memory(
            "train/memory-b/1",
            "memory_same_query_changed_source",
            "Ada is in Oslo. Ben is in Lima.\nBen?\n",
            "Ben",
            "Lima",
        ),
        memory(
            "train/duplicate/0",
            "memory_duplicate_payload",
            "Ada is in Lima. Ben is in Lima.\nAda?\n",
            "Ada",
            "Lima",
        ),
        memory(
            "train/duplicate/1",
            "memory_duplicate_payload",
            "Ada is in Lima. Ben is in Lima.\nBen?\n",
            "Ben",
            "Lima",
        ),
        memory(
            "train/current/0",
            "memory_current_revision",
            "Ada is in Lima. Now Ada is in Oslo.\nAda?\n",
            "Ada",
            "Oslo",
        ),
        memory(
            "train/current/1",
            "memory_current_revision",
            "Ada is in Oslo. Now Ada is in Lima.\nAda?\n",
            "Ada",
            "Lima",
        ),
        program(
            "train/sub-9-2/0",
            "rust_ordered_subtraction",
            9,
            2,
            Arithmetic::Sub,
        ),
        program(
            "train/sub-9-2/1",
            "rust_ordered_subtraction",
            2,
            9,
            Arithmetic::Sub,
        ),
        program(
            "train/sub-6-4/0",
            "rust_ordered_subtraction",
            6,
            4,
            Arithmetic::Sub,
        ),
        program(
            "train/sub-6-4/1",
            "rust_ordered_subtraction",
            4,
            6,
            Arithmetic::Sub,
        ),
        program(
            "train/add-3-5/0",
            "rust_commutative_addition",
            3,
            5,
            Arithmetic::Add,
        ),
        program(
            "train/add-3-5/1",
            "rust_commutative_addition",
            5,
            3,
            Arithmetic::Add,
        ),
        program(
            "train/operator/0",
            "rust_same_operands_changed_operator",
            4,
            4,
            Arithmetic::Add,
        ),
        program(
            "train/operator/1",
            "rust_same_operands_changed_operator",
            4,
            4,
            Arithmetic::Sub,
        ),
    ]
}

/// Frozen new names/locations and arithmetic combinations. These are authored
/// family probes, not an independent natural-language corpus or final holdout.
pub fn development() -> Vec<Example> {
    vec![
        memory(
            "dev/memory/0",
            "memory_same_query_changed_source",
            "Eve is in Rome. Max is in Bath.\nEve?\n",
            "Eve",
            "Rome",
        ),
        memory(
            "dev/memory/1",
            "memory_same_query_changed_source",
            "Eve is in Bath. Max is in Rome.\nEve?\n",
            "Eve",
            "Bath",
        ),
        memory(
            "dev/duplicate/0",
            "memory_duplicate_payload",
            "Eve is in Bath. Max is in Bath.\nEve?\n",
            "Eve",
            "Bath",
        ),
        memory(
            "dev/current/0",
            "memory_current_revision",
            "Eve is in Bath. Now Eve is in Rome.\nEve?\n",
            "Eve",
            "Rome",
        ),
        program(
            "dev/sub-8-3/0",
            "rust_ordered_subtraction",
            8,
            3,
            Arithmetic::Sub,
        ),
        program(
            "dev/sub-8-3/1",
            "rust_ordered_subtraction",
            3,
            8,
            Arithmetic::Sub,
        ),
        program(
            "dev/operator/0",
            "rust_same_operands_changed_operator",
            6,
            6,
            Arithmetic::Add,
        ),
        program(
            "dev/operator/1",
            "rust_same_operands_changed_operator",
            6,
            6,
            Arithmetic::Sub,
        ),
    ]
}

/// Bind order, split-specific IDs, family, exact prompt/answer and scalar intent.
/// Fixed little-endian lengths make this independent of JSON pretty-printing.
pub fn corpus_digest(examples: &[Example]) -> [u8; 32] {
    fn field(hash: &mut blake3::Hasher, bytes: &[u8]) {
        hash.update(&(bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    let mut hash = blake3::Hasher::new();
    hash.update(b"uor-r4.addressed-attention-pilot-corpus/1\0");
    hash.update(&(examples.len() as u64).to_le_bytes());
    for example in examples {
        for bytes in [
            example.id.as_bytes(),
            example.family.as_bytes(),
            &example.prompt,
            &example.answer,
        ] {
            field(&mut hash, bytes);
        }
        match example.expected_scalar {
            None => {
                hash.update(&[0]);
            }
            Some(value) => {
                hash.update(&[1]);
                hash.update(&value.to_le_bytes());
            }
        }
    }
    *hash.finalize().as_bytes()
}

pub fn validate() -> std::result::Result<(), EngineError> {
    let train = training();
    let dev = development();
    if train.len() != 16 || dev.len() != 8 {
        return Err(EngineError::Invalid("pilot split size"));
    }
    let mut ids = BTreeSet::new();
    let mut prompts = BTreeSet::new();
    for (prefix, split) in [("train/", &train), ("dev/", &dev)] {
        for example in split {
            if !example.id.starts_with(prefix)
                || !ids.insert(example.id.as_str())
                || !prompts.insert(example.prompt.as_slice())
                || example.prompt.is_empty()
                || example.answer.is_empty()
                || !example.prompt.is_ascii()
                || !example.answer.is_ascii()
                || example.prompt.len() + example.answer.len() + 1 > 64
            {
                return Err(EngineError::Invalid(
                    "pilot identity/disjointness/64-position bound",
                ));
            }
            if let Some(scalar) = example.expected_scalar {
                if !example.family.starts_with("rust_")
                    || example.answer
                        != format!("fn main(){{println!(\"{{}}\",{scalar});}}\n").as_bytes()
                {
                    return Err(EngineError::Invalid("pilot typed program answer"));
                }
            } else if !example.family.starts_with("memory_") {
                return Err(EngineError::Invalid("pilot memory family"));
            }
        }
    }
    for pair in [0, 2, 6, 8, 10, 14] {
        if train[pair].answer == train[pair + 1].answer {
            return Err(EngineError::Invalid("pilot changed-answer pair"));
        }
    }
    for pair in [0, 4, 6] {
        if dev[pair].answer == dev[pair + 1].answer {
            return Err(EngineError::Invalid("pilot development pair"));
        }
    }
    if train[4].answer != train[5].answer || train[12].answer != train[13].answer {
        return Err(EngineError::Invalid("pilot equal-answer controls"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "pilot_data_tests.rs"]
mod tests;
