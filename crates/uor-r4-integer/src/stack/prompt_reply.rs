//! Artifact-bound complete-reply retrieval for stack serving.
//!
//! A sidecar stores prompt/reply token pairs from a training corpus. At load
//! time each prompt becomes a vocabulary-dimensional binary vector. At a chat
//! boundary, bitwise AND/OR popcounts recover exact set Jaccard overlap and the
//! selector returns one complete reply. The model artifact and dialogue bytes
//! are unchanged.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::Digest;

use super::StackError;

pub const STACK_PROMPT_REPLY_SCHEMA: &str = "uor-r4.stack-prompt-reply-readout/1";
pub const MAX_PROMPT_REPLY_SIDECAR_BYTES: u64 = 64 << 20;
pub const MAX_PROMPT_REPLY_ENTRIES: usize = 100_000;
pub const MAX_PROMPT_REPLY_PROMPT_TOKENS: usize = 65_536;
pub const MAX_PROMPT_REPLY_REPLY_TOKENS: usize = 4_096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptReplyExample {
    pub prompt: Vec<u32>,
    pub reply: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PromptReplyIndex {
    prompt_tokens: Vec<u32>,
    prompt_signature: Vec<u64>,
    reply: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptReplyReadout {
    artifact_sha256: String,
    tokenizer_sha256: String,
    protocol_version: u8,
    ignore_token_ids: Vec<u32>,
    entries: Vec<PromptReplyIndex>,
    signature_words: usize,
    sidecar_sha256: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PromptReplySidecar {
    schema: String,
    artifact_sha256: String,
    tokenizer_sha256: String,
    protocol_version: u8,
    ignore_token_ids: Vec<u32>,
    entries: Vec<PromptReplyExample>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptReplySelection {
    pub prompt_ordinal: usize,
    pub similarity_numerator: u64,
    pub similarity_denominator: u64,
    pub reply: Vec<u32>,
}

impl PromptReplyReadout {
    pub fn from_entries(
        artifact_sha256: impl Into<String>,
        tokenizer_sha256: impl Into<String>,
        protocol_version: u8,
        ignore_token_ids: Vec<u32>,
        entries: Vec<PromptReplyExample>,
        vocab: usize,
    ) -> Result<Self, StackError> {
        let artifact_sha256 = artifact_sha256.into();
        let tokenizer_sha256 = tokenizer_sha256.into();
        validate_hash(&artifact_sha256, "artifact_sha256")?;
        validate_hash(&tokenizer_sha256, "tokenizer_sha256")?;
        if !(1..=2).contains(&protocol_version) {
            return Err(prompt_error("protocol_version must be 1 or 2"));
        }
        validate_ignore_ids(&ignore_token_ids)?;
        let signature_words = vocab.div_ceil(64);
        let indexes = validate_entries(entries, &ignore_token_ids, vocab, signature_words)?;
        Ok(Self {
            artifact_sha256,
            tokenizer_sha256,
            protocol_version,
            ignore_token_ids: ignore_token_ids.into_iter().collect(),
            entries: indexes,
            signature_words,
            sidecar_sha256: None,
        })
    }

    pub fn from_sidecar(
        bytes: &[u8],
        artifact_sha256: &str,
        tokenizer_sha256: &str,
        expected_protocol: u8,
        expected_ignore_token_ids: &[u32],
        vocab: usize,
    ) -> Result<Self, StackError> {
        if bytes.len() as u64 > MAX_PROMPT_REPLY_SIDECAR_BYTES {
            return Err(prompt_error(format!(
                "sidecar exceeds {MAX_PROMPT_REPLY_SIDECAR_BYTES} bytes"
            )));
        }
        let sidecar: PromptReplySidecar = serde_json::from_slice(bytes)
            .map_err(|error| prompt_error(format!("invalid sidecar JSON: {error}")))?;
        if sidecar.schema != STACK_PROMPT_REPLY_SCHEMA {
            return Err(prompt_error(format!(
                "schema {:?} is not {STACK_PROMPT_REPLY_SCHEMA:?}",
                sidecar.schema
            )));
        }
        validate_hash(&sidecar.artifact_sha256, "artifact_sha256")?;
        validate_hash(&sidecar.tokenizer_sha256, "tokenizer_sha256")?;
        if sidecar.artifact_sha256 != artifact_sha256 {
            return Err(prompt_error(format!(
                "sidecar binds artifact {}, loaded artifact is {artifact_sha256}",
                sidecar.artifact_sha256
            )));
        }
        if sidecar.tokenizer_sha256 != tokenizer_sha256 {
            return Err(prompt_error(format!(
                "sidecar binds tokenizer {}, loaded tokenizer is {tokenizer_sha256}",
                sidecar.tokenizer_sha256
            )));
        }
        if sidecar.protocol_version != expected_protocol {
            return Err(prompt_error(format!(
                "sidecar protocol is {}, serving protocol is {expected_protocol}",
                sidecar.protocol_version
            )));
        }
        if sidecar.ignore_token_ids != expected_ignore_token_ids {
            return Err(prompt_error(format!(
                "sidecar ignores {:?}, protocol requires {expected_ignore_token_ids:?}",
                sidecar.ignore_token_ids
            )));
        }
        let mut readout = Self::from_entries(
            sidecar.artifact_sha256,
            sidecar.tokenizer_sha256,
            sidecar.protocol_version,
            sidecar.ignore_token_ids,
            sidecar.entries,
            vocab,
        )?;
        readout.sidecar_sha256 = Some(hex::encode(sha2::Sha256::digest(bytes)));
        Ok(readout)
    }

    pub fn load(
        path: &Path,
        artifact_sha256: &str,
        tokenizer_sha256: &str,
        expected_protocol: u8,
        expected_ignore_token_ids: &[u32],
        vocab: usize,
    ) -> Result<Self, StackError> {
        Self::read_sidecar(
            path,
            artifact_sha256,
            tokenizer_sha256,
            expected_protocol,
            expected_ignore_token_ids,
            vocab,
        )
        .map(|(_, readout)| readout)
    }

    pub fn read_sidecar(
        path: &Path,
        artifact_sha256: &str,
        tokenizer_sha256: &str,
        expected_protocol: u8,
        expected_ignore_token_ids: &[u32],
        vocab: usize,
    ) -> Result<(Vec<u8>, Self), StackError> {
        let file = std::fs::File::open(path).map_err(StackError::Io)?;
        let mut bytes = Vec::new();
        file.take(MAX_PROMPT_REPLY_SIDECAR_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(StackError::Io)?;
        let readout = Self::from_sidecar(
            &bytes,
            artifact_sha256,
            tokenizer_sha256,
            expected_protocol,
            expected_ignore_token_ids,
            vocab,
        )?;
        Ok((bytes, readout))
    }

    pub fn to_sidecar_bytes(&self) -> Result<Vec<u8>, StackError> {
        let sidecar = PromptReplySidecar {
            schema: STACK_PROMPT_REPLY_SCHEMA.to_owned(),
            artifact_sha256: self.artifact_sha256.clone(),
            tokenizer_sha256: self.tokenizer_sha256.clone(),
            protocol_version: self.protocol_version,
            ignore_token_ids: self.ignore_token_ids.clone(),
            entries: self
                .entries
                .iter()
                .map(|entry| PromptReplyExample {
                    prompt: entry.prompt_tokens.clone(),
                    reply: entry.reply.clone(),
                })
                .collect(),
        };
        serde_json::to_vec(&sidecar)
            .map_err(|error| prompt_error(format!("cannot serialize sidecar: {error}")))
    }

    pub fn entries(&self) -> usize {
        self.entries.len()
    }

    pub fn signature_words(&self) -> usize {
        self.signature_words
    }

    pub fn artifact_sha256(&self) -> &str {
        &self.artifact_sha256
    }

    pub fn tokenizer_sha256(&self) -> &str {
        &self.tokenizer_sha256
    }

    pub fn protocol_version(&self) -> u8 {
        self.protocol_version
    }

    pub fn ignore_token_ids(&self) -> &[u32] {
        &self.ignore_token_ids
    }

    pub fn sidecar_sha256(&self) -> Option<&str> {
        self.sidecar_sha256.as_deref()
    }

    pub fn select(&self, history: &[u32]) -> Option<PromptReplySelection> {
        let ignored = self
            .ignore_token_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut query = vec![0u64; self.signature_words];
        for &token in history {
            if !ignored.contains(&token) {
                let token = token as usize;
                query[token / 64] |= 1u64 << (token % 64);
            }
        }
        if query.iter().all(|&word| word == 0) {
            return None;
        }
        let mut best: Option<(u64, u64, usize)> = None;
        for (ordinal, entry) in self.entries.iter().enumerate() {
            let (numerator, denominator) = signature_overlap(&query, &entry.prompt_signature);
            let replace = match best {
                None => true,
                Some((best_num, best_den, best_ordinal)) => {
                    match compare_ratio(numerator, denominator, best_num, best_den) {
                        std::cmp::Ordering::Greater => true,
                        std::cmp::Ordering::Equal => ordinal < best_ordinal,
                        std::cmp::Ordering::Less => false,
                    }
                }
            };
            if replace {
                best = Some((numerator, denominator, ordinal));
            }
        }
        let (similarity_numerator, similarity_denominator, prompt_ordinal) = best?;
        Some(PromptReplySelection {
            prompt_ordinal,
            similarity_numerator,
            similarity_denominator,
            reply: self.entries[prompt_ordinal].reply.clone(),
        })
    }
}

fn validate_entries(
    entries: Vec<PromptReplyExample>,
    ignore_token_ids: &[u32],
    vocab: usize,
    signature_words: usize,
) -> Result<Vec<PromptReplyIndex>, StackError> {
    if entries.is_empty() || entries.len() > MAX_PROMPT_REPLY_ENTRIES {
        return Err(prompt_error(format!(
            "entry count {} is outside 1..={MAX_PROMPT_REPLY_ENTRIES}",
            entries.len()
        )));
    }
    if vocab == 0 {
        return Err(prompt_error("vocabulary is empty"));
    }
    let ignored = ignore_token_ids.iter().copied().collect::<BTreeSet<_>>();
    entries
        .into_iter()
        .enumerate()
        .map(|(ordinal, entry)| {
            if entry.prompt.is_empty() || entry.prompt.len() > MAX_PROMPT_REPLY_PROMPT_TOKENS {
                return Err(prompt_error(format!(
                    "entry {ordinal} prompt length {} is outside 1..={MAX_PROMPT_REPLY_PROMPT_TOKENS}",
                    entry.prompt.len()
                )));
            }
            if entry.reply.is_empty() || entry.reply.len() > MAX_PROMPT_REPLY_REPLY_TOKENS {
                return Err(prompt_error(format!(
                    "entry {ordinal} reply length {} is outside 1..={MAX_PROMPT_REPLY_REPLY_TOKENS}",
                    entry.reply.len()
                )));
            }
            for &token in entry.prompt.iter().chain(&entry.reply) {
                if token as usize >= vocab {
                    return Err(prompt_error(format!(
                        "entry {ordinal} token {token} is outside vocabulary {vocab}"
                    )));
                }
                if ignored.contains(&token) {
                    return Err(prompt_error(format!(
                        "entry {ordinal} contains ignored control token {token}"
                    )));
                }
            }
            let mut prompt_signature = vec![0u64; signature_words];
            for &token in &entry.prompt {
                let token = token as usize;
                prompt_signature[token / 64] |= 1u64 << (token % 64);
            }
            Ok(PromptReplyIndex {
                prompt_tokens: entry.prompt,
                prompt_signature,
                reply: entry.reply,
            })
        })
        .collect()
}

fn signature_overlap(left: &[u64], right: &[u64]) -> (u64, u64) {
    left.iter()
        .zip(right)
        .fold((0u64, 0u64), |(intersection, union), (&left, &right)| {
            (
                intersection + u64::from((left & right).count_ones()),
                union + u64::from((left | right).count_ones()),
            )
        })
}

fn validate_ignore_ids(ids: &[u32]) -> Result<(), StackError> {
    if ids.is_empty() || ids.len() > 8 || ids.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(prompt_error(
            "ignore_token_ids must be 1..=8 strictly increasing ids",
        ));
    }
    Ok(())
}

fn validate_hash(value: &str, name: &str) -> Result<(), StackError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(prompt_error(format!(
            "{name} is not 64 lowercase hex characters"
        )));
    }
    Ok(())
}

fn compare_ratio(
    left_num: u64,
    left_den: u64,
    right_num: u64,
    right_den: u64,
) -> std::cmp::Ordering {
    (u128::from(left_num) * u128::from(right_den))
        .cmp(&(u128::from(right_num) * u128::from(left_den)))
}

fn prompt_error(reason: impl Into<String>) -> StackError {
    StackError::PromptReply(reason.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(prompt: &[u32], reply: &[u32]) -> PromptReplyExample {
        PromptReplyExample {
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
        }
    }

    fn readout(entries: Vec<PromptReplyExample>) -> PromptReplyReadout {
        PromptReplyReadout::from_entries(
            "a".repeat(64),
            "b".repeat(64),
            2,
            vec![0, 1, 2],
            entries,
            64,
        )
        .unwrap()
    }

    #[test]
    fn selects_complete_reply_by_exact_set_overlap_and_lower_tie() {
        let readout = readout(vec![
            example(&[3, 4, 5], &[10]),
            example(&[3, 4, 6], &[11]),
            example(&[7, 8, 9], &[12]),
        ]);
        let selection = readout.select(&[0, 3, 4, 13, 1]).unwrap();
        assert_eq!(selection.prompt_ordinal, 0);
        assert_eq!(selection.reply, [10]);
        assert_eq!(selection.similarity_numerator, 2);
        assert_eq!(selection.similarity_denominator, 4);
    }

    #[test]
    fn binary_signature_preserves_set_counts_and_hamming_symmetric_difference() {
        let readout = readout(vec![example(&[3, 4], &[10])]);
        let query = [3, 5];
        let mut query_signature = vec![0u64; readout.signature_words()];
        for token in query {
            query_signature[token as usize / 64] |= 1u64 << (token as usize % 64);
        }
        let (intersection, union) =
            signature_overlap(&query_signature, &readout.entries[0].prompt_signature);
        let hamming: u32 = query_signature
            .iter()
            .zip(&readout.entries[0].prompt_signature)
            .map(|(&left, &right)| (left ^ right).count_ones())
            .sum();
        assert_eq!((intersection, union, hamming), (1, 3, 2));
        assert_eq!(hamming, union as u32 - intersection as u32);
    }

    #[test]
    fn ignores_protocol_controls_and_refuses_empty_query() {
        let readout = readout(vec![example(&[3, 4], &[10])]);
        assert!(readout.select(&[0, 1, 2]).is_none());
        assert_eq!(readout.select(&[0, 3, 4, 1]).unwrap().reply, [10]);
    }

    #[test]
    fn sidecar_round_trips_and_binds_artifact_tokenizer_and_protocol() {
        let readout = readout(vec![example(&[3, 4], &[10, 11])]);
        let bytes = readout.to_sidecar_bytes().unwrap();
        let loaded = PromptReplyReadout::from_sidecar(
            &bytes,
            &"a".repeat(64),
            &"b".repeat(64),
            2,
            &[0, 1, 2],
            64,
        )
        .unwrap();
        assert_eq!(loaded.entries(), 1);
        assert!(loaded.sidecar_sha256().is_some());
        assert!(PromptReplyReadout::from_sidecar(
            &bytes,
            &"c".repeat(64),
            &"b".repeat(64),
            2,
            &[0, 1, 2],
            64
        )
        .is_err());
        assert!(PromptReplyReadout::from_sidecar(
            &bytes,
            &"a".repeat(64),
            &"c".repeat(64),
            2,
            &[0, 1, 2],
            64
        )
        .is_err());
        assert!(PromptReplyReadout::from_sidecar(
            &bytes,
            &"a".repeat(64),
            &"b".repeat(64),
            1,
            &[0, 1, 2],
            64
        )
        .is_err());
        assert!(PromptReplyReadout::from_sidecar(
            &bytes,
            &"a".repeat(64),
            &"b".repeat(64),
            2,
            &[0, 1, 3],
            64
        )
        .is_err());
    }

    #[test]
    fn parser_refuses_unknown_fields_out_of_range_and_control_payloads() {
        let mut value = serde_json::to_value(PromptReplySidecar {
            schema: STACK_PROMPT_REPLY_SCHEMA.to_owned(),
            artifact_sha256: "a".repeat(64),
            tokenizer_sha256: "b".repeat(64),
            protocol_version: 2,
            ignore_token_ids: vec![0, 1, 2],
            entries: vec![example(&[3], &[4])],
        })
        .unwrap();
        value["extra"] = serde_json::json!(true);
        assert!(PromptReplyReadout::from_sidecar(
            &serde_json::to_vec(&value).unwrap(),
            &"a".repeat(64),
            &"b".repeat(64),
            2,
            &[0, 1, 2],
            64
        )
        .is_err());
        assert!(PromptReplyReadout::from_entries(
            "a".repeat(64),
            "b".repeat(64),
            2,
            vec![0, 1, 2],
            vec![example(&[0, 3], &[4])],
            64
        )
        .is_err());
        assert!(PromptReplyReadout::from_entries(
            "a".repeat(64),
            "b".repeat(64),
            2,
            vec![0, 1, 2],
            vec![example(&[3], &[64])],
            64
        )
        .is_err());
    }
}
