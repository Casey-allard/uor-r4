//! Strict masked-corpus ingestion and deterministic document-level splits.
//!
//! This module prepares causal rank-margin samples from little-endian `u16`
//! token streams and byte response masks. It does not train or evaluate by
//! itself; those operations remain explicit caller actions.

use std::collections::BTreeSet;
use std::ops::Range;

use serde_json::Value;

use crate::train::RankMarginSample;
use crate::{invalid, CpuTrainError};

type Result<T> = std::result::Result<T, CpuTrainError>;

const UORT_HEADER_BYTES: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuCorpusIdentity {
    pub schema: String,
    pub split: String,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub tokenizer_sha256: String,
    pub tokens: usize,
    pub token_bytes: usize,
    pub mask_bytes: usize,
    pub response_targets: usize,
    pub max_tokens: usize,
    pub vocab: usize,
    pub bos: u32,
}

impl CpuCorpusIdentity {
    pub fn validate_context(&self, context: usize) -> Result<()> {
        if context == 0 || self.max_tokens < context {
            return Err(invalid(
                "corpus manifest maximum horizon is below the model context",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuMaskedCorpus {
    tokens: Vec<u32>,
    mask: Vec<u8>,
    documents: Vec<Range<usize>>,
    context: usize,
    bos: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuCorpusSplit {
    pub training: Vec<RankMarginSample>,
    pub evaluation: Vec<RankMarginSample>,
    pub training_documents: usize,
    pub evaluation_documents: usize,
    pub training_targets: usize,
    pub evaluation_targets: usize,
    pub evaluation_stride: usize,
}

impl CpuMaskedCorpus {
    pub fn from_bytes(
        token_bytes: &[u8],
        mask: &[u8],
        vocab: usize,
        context: usize,
        bos: u32,
    ) -> Result<Self> {
        if vocab == 0 || vocab > u16::MAX as usize + 1 {
            return Err(invalid("corpus vocabulary must be in 1..=65536"));
        }
        if context < 2 {
            return Err(invalid("corpus context must be at least 2"));
        }
        if token_bytes.len() < UORT_HEADER_BYTES || &token_bytes[..4] != b"UORT" {
            return Err(invalid("corpus requires a canonical UORT header"));
        }
        let u32_at = |at: usize| {
            u32::from_le_bytes([
                token_bytes[at],
                token_bytes[at + 1],
                token_bytes[at + 2],
                token_bytes[at + 3],
            ])
        };
        let count = u64::from_le_bytes(
            token_bytes[8..16]
                .try_into()
                .map_err(|_| invalid("corpus UORT count differs"))?,
        );
        let physical = count
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(UORT_HEADER_BYTES as u64))
            .ok_or_else(|| invalid("corpus UORT length overflows"))?;
        if u32_at(4) != 1
            || usize::try_from(u32_at(16)).ok() != Some(vocab)
            || token_bytes[20..64].iter().any(|&byte| byte != 0)
            || physical != token_bytes.len() as u64
            || usize::try_from(count).ok() != Some(mask.len())
            || count == 0
        {
            return Err(invalid("corpus UORT header or payload length differs"));
        }
        if mask.iter().any(|&value| value > 1) {
            return Err(invalid(
                "corpus response mask holds a byte other than 0 or 1",
            ));
        }
        let tokens = token_bytes[UORT_HEADER_BYTES..]
            .chunks_exact(2)
            .map(|bytes| u32::from(u16::from_le_bytes([bytes[0], bytes[1]])))
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            return Err(invalid("corpus token stream is empty"));
        }
        if tokens.iter().any(|&token| token as usize >= vocab) {
            return Err(invalid("corpus token is outside the artifact vocabulary"));
        }
        if tokens[0] != bos {
            return Err(invalid("corpus must begin with BOS"));
        }
        let starts = tokens
            .iter()
            .enumerate()
            .filter_map(|(index, &token)| (token == bos).then_some(index))
            .collect::<Vec<_>>();
        let documents = starts
            .iter()
            .enumerate()
            .map(|(index, &start)| start..starts.get(index + 1).copied().unwrap_or(tokens.len()))
            .collect::<Vec<_>>();
        if documents.is_empty() {
            return Err(invalid("corpus has no BOS-delimited documents"));
        }
        Ok(Self {
            tokens,
            mask: mask.to_vec(),
            documents,
            context,
            bos,
        })
    }

    pub fn tokens(&self) -> &[u32] {
        &self.tokens
    }

    pub fn mask(&self) -> &[u8] {
        &self.mask
    }

    pub fn context(&self) -> usize {
        self.context
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn split(&self, evaluation_stride: usize, max_evaluation: usize) -> Result<CpuCorpusSplit> {
        if evaluation_stride < 2 {
            return Err(invalid("evaluation stride must be at least 2"));
        }
        if max_evaluation == 0 {
            return Err(invalid("evaluation sample cap must be positive"));
        }
        let mut training = Vec::new();
        let mut evaluation = Vec::new();
        let mut training_documents = 0usize;
        for (ordinal, document) in self.documents.iter().enumerate() {
            let samples = self.document_samples(document)?;
            if ordinal % evaluation_stride == evaluation_stride - 1 {
                evaluation.extend(samples.into_iter().map(|sample| (ordinal, sample)));
            } else {
                if !samples.is_empty() {
                    training_documents += 1;
                }
                training.extend(samples);
            }
        }
        if training.is_empty() {
            return Err(invalid("corpus split has no training targets"));
        }
        if evaluation.is_empty() {
            return Err(invalid("corpus split has no evaluation targets"));
        }
        evaluation = evenly_cap(evaluation, max_evaluation);
        let evaluation_documents = evaluation
            .iter()
            .map(|(ordinal, _)| *ordinal)
            .collect::<BTreeSet<_>>()
            .len();
        let evaluation = evaluation
            .into_iter()
            .map(|(_, sample)| sample)
            .collect::<Vec<RankMarginSample>>();
        Ok(CpuCorpusSplit {
            training_targets: training.len(),
            evaluation_targets: evaluation.len(),
            training,
            evaluation,
            training_documents,
            evaluation_documents,
            evaluation_stride,
        })
    }

    fn document_samples(&self, document: &Range<usize>) -> Result<Vec<RankMarginSample>> {
        let mut samples = Vec::new();
        for target_position in document.start + 1..document.end {
            if self.mask[target_position] != 1 || self.tokens[target_position] == self.bos {
                continue;
            }
            let prefix_start = target_position
                .saturating_sub(self.context)
                .max(document.start);
            let input = self.tokens[prefix_start..target_position].to_vec();
            if input.is_empty() || input.len() > self.context {
                return Err(invalid(
                    "corpus causal prefix length is outside the model context",
                ));
            }
            samples.push(RankMarginSample {
                input,
                target: self.tokens[target_position],
            });
        }
        Ok(samples)
    }
}

pub fn validate_corpus_manifest(
    manifest_bytes: &[u8],
    token_bytes: &[u8],
    mask: &[u8],
) -> Result<CpuCorpusIdentity> {
    let manifest: Value =
        serde_json::from_slice(manifest_bytes).map_err(|error| invalid(error.to_string()))?;
    let text = |name: &str| {
        manifest[name]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| invalid(format!("corpus manifest {name} differs")))
    };
    let integer = |name: &str| {
        manifest[name]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| invalid(format!("corpus manifest {name} differs")))
    };
    let schema = text("schema")?;
    if schema != "uor-r4-chat-corpus/v1" || text("mask_schema")? != "uor-r4-response-mask/u8/v1" {
        return Err(invalid("corpus manifest schema differs"));
    }
    let tokenizer = manifest["tokenizer"]
        .as_object()
        .ok_or_else(|| invalid("corpus manifest tokenizer differs"))?;
    let vocab = tokenizer
        .get("vocab_size")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid("corpus manifest vocabulary differs"))?;
    let bos = tokenizer
        .get("bos_id")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| invalid("corpus manifest BOS differs"))?;
    let tokenizer_sha256 = tokenizer
        .get("sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("corpus manifest tokenizer sha256 differs"))?
        .to_owned();
    uor_r4_discrete::corpus::validate_sha256(&tokenizer_sha256)
        .map_err(|error| invalid(error.to_string()))?;
    let tokens = integer("tokens")?;
    let token_bytes_declared = integer("tokens_bytes")?;
    let mask_bytes = integer("mask_bytes")?;
    let response_targets = integer("response_tokens")?;
    let max_tokens = integer("max_tokens")?;
    let selected_positions = mask.iter().filter(|&&value| value == 1).count();
    if token_bytes.len() < UORT_HEADER_BYTES
        || &token_bytes[..4] != b"UORT"
        || u64::from_le_bytes(
            token_bytes[8..16]
                .try_into()
                .map_err(|_| invalid("corpus UORT count differs"))?,
        ) != tokens as u64
        || token_bytes_declared != token_bytes.len()
        || mask_bytes != mask.len()
        || selected_positions != response_targets
        || max_tokens == 0
    {
        return Err(invalid("corpus manifest inventory differs"));
    }
    let tokens_sha256 = text("tokens_sha256")?;
    let mask_sha256 = text("mask_sha256")?;
    for hash in [&tokens_sha256, &mask_sha256] {
        uor_r4_discrete::corpus::validate_sha256(hash)
            .map_err(|error| invalid(error.to_string()))?;
    }
    if uor_r4_discrete::corpus::sha256(token_bytes) != tokens_sha256
        || uor_r4_discrete::corpus::sha256(mask) != mask_sha256
    {
        return Err(invalid("corpus manifest content hash differs"));
    }
    Ok(CpuCorpusIdentity {
        schema,
        split: text("split")?,
        tokens_sha256,
        mask_sha256,
        tokenizer_sha256,
        tokens,
        token_bytes: token_bytes_declared,
        mask_bytes,
        response_targets,
        max_tokens,
        vocab,
        bos,
    })
}

fn evenly_cap<T: Clone>(values: Vec<T>, maximum: usize) -> Vec<T> {
    if values.len() <= maximum {
        return values;
    }
    (0..maximum)
        .map(|index| values[index * values.len() / maximum].clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus() -> CpuMaskedCorpus {
        let tokens = [
            0u16, 10, 11, 12, 0, 20, 21, 22, 0, 30, 31, 32, 0, 40, 41, 42,
        ];
        let bytes = uort(&tokens, 64);
        let mask = [0, 0, 1, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1];
        CpuMaskedCorpus::from_bytes(&bytes, &mask, 64, 3, 0).unwrap()
    }

    fn uort(tokens: &[u16], vocab: u32) -> Vec<u8> {
        let mut bytes = vec![0u8; UORT_HEADER_BYTES];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&vocab.to_le_bytes());
        for token in tokens {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn causal_samples_stop_at_document_boundaries_and_context() {
        let split = corpus().split(2, 16).unwrap();
        assert_eq!(split.training_documents, 2);
        assert_eq!(split.evaluation_documents, 2);
        assert_eq!(
            split.training[0],
            RankMarginSample {
                input: vec![0, 10],
                target: 11,
            }
        );
        assert!(split
            .training
            .iter()
            .chain(&split.evaluation)
            .all(|sample| sample.input.len() <= 3));
        assert!(split
            .training
            .iter()
            .chain(&split.evaluation)
            .any(|sample| sample.input.len() == 3));
        assert!(split.training.iter().all(|sample| sample
            .input
            .iter()
            .chain([sample.target].iter())
            .all(|token| ![20, 21, 22, 40, 41, 42].contains(token))));
        assert!(split.evaluation.iter().all(|sample| sample
            .input
            .iter()
            .chain([sample.target].iter())
            .all(|token| *token == 0 || [20, 21, 22, 40, 41, 42].contains(token))));
    }

    #[test]
    fn document_split_never_shares_targets_or_documents() {
        let split = corpus().split(2, 16).unwrap();
        let left = split
            .training
            .iter()
            .flat_map(|sample| sample.input.iter().copied().chain([sample.target]))
            .collect::<Vec<_>>();
        let right = split
            .evaluation
            .iter()
            .flat_map(|sample| sample.input.iter().copied().chain([sample.target]))
            .collect::<Vec<_>>();
        assert!(left
            .iter()
            .all(|token| *token == 0 || [10, 11, 12, 30, 31, 32].contains(token)));
        assert!(right
            .iter()
            .all(|token| *token == 0 || [20, 21, 22, 40, 41, 42].contains(token)));
    }

    #[test]
    fn evaluation_cap_is_deterministic() {
        let first = corpus().split(2, 1).unwrap();
        let second = corpus().split(2, 1).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.evaluation.len(), 1);
    }

    #[test]
    fn malformed_corpus_inputs_are_refused() {
        let token_bytes = uort(&[0, 1], 64);
        assert!(CpuMaskedCorpus::from_bytes(&token_bytes[..3], &[0, 1], 64, 3, 0).is_err());
        assert!(CpuMaskedCorpus::from_bytes(&token_bytes, &[0], 64, 3, 0).is_err());
        assert!(CpuMaskedCorpus::from_bytes(&token_bytes, &[0, 2], 64, 3, 0).is_err());
        assert!(CpuMaskedCorpus::from_bytes(&token_bytes, &[0, 1], 1, 3, 0).is_err());
        assert!(CpuMaskedCorpus::from_bytes(&token_bytes, &[0, 1], 64, 3, 2).is_err());
        assert!(CpuMaskedCorpus::from_bytes(&token_bytes, &[0, 1], 64, 1, 0).is_err());
        let mut bad_count = token_bytes.clone();
        bad_count[8..16].copy_from_slice(&3u64.to_le_bytes());
        assert!(CpuMaskedCorpus::from_bytes(&bad_count, &[0, 1], 64, 3, 0).is_err());
        let mut bad_version = token_bytes.clone();
        bad_version[4..8].copy_from_slice(&2u32.to_le_bytes());
        assert!(CpuMaskedCorpus::from_bytes(&bad_version, &[0, 1], 64, 3, 0).is_err());
        let mut bad_vocab = token_bytes.clone();
        bad_vocab[16..20].copy_from_slice(&65u32.to_le_bytes());
        assert!(CpuMaskedCorpus::from_bytes(&bad_vocab, &[0, 1], 64, 3, 0).is_err());
        let mut bad_reserved = token_bytes;
        bad_reserved[20] = 1;
        assert!(CpuMaskedCorpus::from_bytes(&bad_reserved, &[0, 1], 64, 3, 0).is_err());
    }

    #[test]
    fn manifest_binds_exact_token_and_mask_content() {
        let token_bytes = uort(&[0, 1, 2], 4096);
        let mask = [0u8, 1, 1];
        let manifest = serde_json::json!({
            "schema": "uor-r4-chat-corpus/v1",
            "mask_schema": "uor-r4-response-mask/u8/v1",
            "split": "train",
            "tokens": 3,
            "tokens_bytes": 70,
            "tokens_sha256": uor_r4_discrete::corpus::sha256(&token_bytes),
            "mask_bytes": 3,
            "mask_sha256": uor_r4_discrete::corpus::sha256(&mask),
            "response_tokens": 2,
            "max_tokens": 192,
            "tokenizer": {
                "vocab_size": 4096,
                "bos_id": 0,
                "sha256": "00".repeat(32)
            }
        });
        let bytes = serde_json::to_vec(&manifest).unwrap();
        let identity = validate_corpus_manifest(&bytes, &token_bytes, &mask).unwrap();
        assert_eq!(identity.split, "train");
        assert_eq!(identity.vocab, 4096);
        assert_eq!(identity.response_targets, 2);
        assert_eq!(identity.max_tokens, 192);
        identity.validate_context(192).unwrap();
        assert!(identity.validate_context(193).is_err());
        assert!(validate_corpus_manifest(&bytes, &token_bytes[..4], &mask).is_err());
        assert!(validate_corpus_manifest(&bytes, &token_bytes, &[0, 1, 2]).is_err());
        let mut tampered: Value = serde_json::from_slice(&bytes).unwrap();
        tampered["response_tokens"] = serde_json::json!(1);
        assert!(validate_corpus_manifest(
            &serde_json::to_vec(&tampered).unwrap(),
            &token_bytes,
            &mask
        )
        .is_err());
    }

    #[test]
    fn two_checkpoint_resumes_reproduce_the_complete_corpus_continuation() {
        use crate::{CpuPanelEvaluator, CpuRankMarginTrainer};
        use uor_r4_integer::stack::export_fbs_trunk_uorlut01;
        use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};

        let snapshot = synthetic_fbs_trunk_with(
            307,
            FbsSynth {
                vocab: 4096,
                d: 64,
                mlp: 64,
                layers: 1,
                context: 8,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let mut tokens_words = Vec::new();
        let mut mask = Vec::new();
        for document in 0u16..4 {
            tokens_words.push(0);
            mask.push(0);
            for offset in 1u16..12 {
                tokens_words.push(10 + document * 20 + offset);
                mask.push(1);
            }
        }
        let token_bytes = uort(&tokens_words, 4096);
        let split = CpuMaskedCorpus::from_bytes(&token_bytes, &mask, 4096, 8, 0)
            .unwrap()
            .split(2, 16)
            .unwrap();
        let mut left = CpuRankMarginTrainer::from_artifact(
            &artifact,
            split.training.clone(),
            crate::RankMarginConfig::default(),
            41,
        )
        .unwrap();
        left.train_steps(3).unwrap();
        let checkpoint = left.checkpoint().unwrap();
        let mut right_a =
            CpuRankMarginTrainer::resume(&checkpoint, split.training.clone()).unwrap();
        let mut right_b =
            CpuRankMarginTrainer::resume(&checkpoint, split.training.clone()).unwrap();
        assert!(split
            .training
            .iter()
            .chain(&split.evaluation)
            .any(|sample| sample.input.len() == 8));
        let continuation = 2;
        let continued = left.train_steps(continuation).unwrap();
        assert_eq!(continued, right_a.train_steps(continuation).unwrap());
        assert_eq!(continued, right_b.train_steps(continuation).unwrap());
        assert_eq!(left.artifact(), right_a.artifact());
        assert_eq!(right_a.artifact(), right_b.artifact());
        assert_eq!(left.checkpoint().unwrap(), right_a.checkpoint().unwrap());
        assert_eq!(right_a.checkpoint().unwrap(), right_b.checkpoint().unwrap());
        let mut left_panel =
            CpuPanelEvaluator::from_artifact(left.artifact(), split.evaluation.clone(), 1).unwrap();
        let mut right_panel =
            CpuPanelEvaluator::from_artifact(right_b.artifact(), split.evaluation, 1).unwrap();
        assert_eq!(
            left_panel.evaluate().unwrap(),
            right_panel.evaluate().unwrap()
        );
    }
}
