//! Corpus identity and causal-window metadata only. No scores/counts/ranks on host.
#![forbid(unsafe_code)]
use crate::{Error, Identity, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

pub const WINDOW_POLICY: &str =
    "response-masked-targets/within-document-predecessor/sliding-prefix/ascending-once/v1";
pub const IDENTITY_POLICY: &str = "corpus-SHA-bound/split-prefix2bits/document-ordinal62bits/absolute-token-occurrence/version0/v1";
pub const UORT_HEADER_BYTES: usize = 64;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn validate_sha256(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(invalid("content pins require lowercase64-hex SHA256"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilePin {
    pub path: PathBuf,
    pub sha256: String,
}
impl FilePin {
    pub fn read(&self) -> Result<Vec<u8>> {
        validate_sha256(&self.sha256)?;
        let bytes = fs::read(&self.path)
            .map_err(|e| invalid(format!("read {}: {e}", self.path.display())))?;
        if sha256(&bytes) != self.sha256 {
            return Err(invalid(format!(
                "content hash differs: {}",
                self.path.display()
            )));
        }
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Split {
    Train,
    Dev,
    Fresh,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BosIdentity {
    pub token: String,
    pub id: u32,
}
impl BosIdentity {
    /// Read tokenizer metadata; never infer a numeric BOS from corpus contents.
    pub fn validate_tokenizer(&self, bytes: &[u8], vocabulary: u32) -> Result<()> {
        if self.token.is_empty() || self.id >= vocabulary {
            return Err(invalid("invalid explicit BOS identity"));
        }
        let json: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| invalid(format!("tokenizer JSON: {e}")))?;
        let vocab = json.pointer("/model/vocab");
        let in_vocabulary = vocab
            .and_then(|v| v.get(&self.token))
            .and_then(|v| v.as_u64())
            == Some(u64::from(self.id));
        let in_added = json
            .get("added_tokens")
            .and_then(|v| v.as_array())
            .is_some_and(|tokens| {
                tokens.iter().any(|v| {
                    v.get("content").and_then(|v| v.as_str()) == Some(self.token.as_str())
                        && v.get("id").and_then(|v| v.as_u64()) == Some(u64::from(self.id))
                })
            });
        if !in_vocabulary && !in_added {
            return Err(invalid("explicit BOS text/id absent from pinned tokenizer"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSource {
    pub split: Split,
    pub tokens: FilePin,
    pub mask: FilePin,
    pub expected_payload_tokens: u64,
    pub expected_eligible_targets: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusBinding {
    pub source: CorpusSource,
    pub vocabulary: u32,
    pub bos: BosIdentity,
    pub payload_tokens: u64,
    pub mask_bytes: u64,
    pub mask_selected_positions: u64,
    pub eligible_targets: u64,
    pub documents: u64,
    pub window_policy: String,
    pub identity_policy: String,
}
#[derive(Debug)]
pub struct PreparedCorpus {
    binding: CorpusBinding,
    tokens: Vec<u32>,
    eligible: Vec<usize>,
    documents: Vec<usize>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusBatch {
    pub windows: Vec<Vec<Identity>>,
    pub targets: Vec<u32>,
    pub target_positions: Vec<u64>,
    pub next_target: u64,
}

impl PreparedCorpus {
    pub fn load(source: &CorpusSource, vocabulary: u32, bos: &BosIdentity) -> Result<Self> {
        Self::from_bytes(
            source.clone(),
            &source.tokens.read()?,
            &source.mask.read()?,
            vocabulary,
            bos.clone(),
        )
    }
    /// Pure metadata/fixture admission; checks the same content pins as file loading.
    pub fn from_bytes(
        source: CorpusSource,
        bytes: &[u8],
        mask: &[u8],
        vocabulary: u32,
        bos: BosIdentity,
    ) -> Result<Self> {
        validate_sha256(&source.tokens.sha256)?;
        validate_sha256(&source.mask.sha256)?;
        if sha256(bytes) != source.tokens.sha256 || sha256(mask) != source.mask.sha256 {
            return Err(invalid("corpus/mask bytes differ from declared pins"));
        }
        if bytes.len() < UORT_HEADER_BYTES || &bytes[..4] != b"UORT" {
            return Err(invalid(
                "requires canonical UORT64-byte header, no raw-u16 fallback",
            ));
        }
        let u32_at = |at: usize| {
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        };
        let mut count = [0u8; 8];
        count.copy_from_slice(&bytes[8..16]);
        let count = u64::from_le_bytes(count);
        if u32_at(4) != 1 || u32_at(16) != vocabulary || bytes[20..64].iter().any(|&b| b != 0) {
            return Err(invalid("UORT version/vocabulary/reserved-header mismatch"));
        }
        let physical = count
            .checked_mul(2)
            .and_then(|n| n.checked_add(64))
            .ok_or_else(|| invalid("UORT count overflow"))?;
        if physical != bytes.len() as u64 || count != source.expected_payload_tokens || count == 0 {
            return Err(invalid(
                "UORT payload length/count differs from pinned inventory",
            ));
        }
        if mask.len() as u64 != count || mask.iter().any(|&m| m > 1) {
            return Err(invalid(
                "mask must be binary and exactly UORT payload length",
            ));
        }
        let tokens = bytes[64..]
            .chunks_exact(2)
            .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
            .collect::<Vec<_>>();
        if bos.token.is_empty()
            || bos.id >= vocabulary
            || tokens.first() != Some(&bos.id)
            || tokens.iter().any(|&id| id >= vocabulary)
        {
            return Err(invalid(
                "corpus must begin at pinned BOS and contain only valid token IDs",
            ));
        }
        let documents = tokens
            .iter()
            .enumerate()
            .filter_map(|(p, &id)| (id == bos.id).then_some(p))
            .collect::<Vec<_>>();
        if documents.len() as u64 >= (1u64 << 62) {
            return Err(invalid(
                "document inventory exceeds split-prefixed identity namespace",
            ));
        }
        let eligible = tokens
            .iter()
            .zip(mask)
            .enumerate()
            .filter_map(|(p, (&id, &m))| (p > 0 && id != bos.id && m == 1).then_some(p))
            .collect::<Vec<_>>();
        if eligible.is_empty() || eligible.len() as u64 != source.expected_eligible_targets {
            return Err(invalid(
                "eligible response targets differ from positive expected inventory",
            ));
        }
        let binding = CorpusBinding {
            source,
            vocabulary,
            bos,
            payload_tokens: count,
            mask_bytes: mask.len() as u64,
            mask_selected_positions: mask.iter().filter(|&&m| m == 1).count() as u64,
            eligible_targets: eligible.len() as u64,
            documents: documents.len() as u64,
            window_policy: WINDOW_POLICY.into(),
            identity_policy: IDENTITY_POLICY.into(),
        };
        Ok(Self {
            binding,
            tokens,
            eligible,
            documents,
        })
    }
    pub fn binding(&self) -> &CorpusBinding {
        &self.binding
    }
    pub fn eligible_len(&self) -> usize {
        self.eligible.len()
    }
    /// Selection only; frequency/count ranking is performed by the CUDA evaluator.
    pub fn eligible_targets(&self) -> Vec<u32> {
        self.eligible.iter().map(|&p| self.tokens[p]).collect()
    }
    pub fn batch(&self, first_target: u64, context: u32, rows: u32) -> Result<CorpusBatch> {
        if !(1..=243).contains(&context) || !(1..=64).contains(&rows) {
            return Err(invalid(
                "corpus batches require context1..243 and rows1..64",
            ));
        }
        let first = usize::try_from(first_target)
            .map_err(|_| invalid("target cursor exceeds address space"))?;
        if first >= self.eligible.len() {
            return Err(invalid("target cursor outside eligible inventory"));
        }
        let end = first.saturating_add(rows as usize).min(self.eligible.len());
        let mut batch = CorpusBatch {
            windows: Vec::with_capacity(end - first),
            targets: Vec::with_capacity(end - first),
            target_positions: Vec::with_capacity(end - first),
            next_target: end as u64,
        };
        for &position in &self.eligible[first..end] {
            let doc = self
                .documents
                .partition_point(|&start| start <= position)
                .checked_sub(1)
                .ok_or_else(|| invalid("target has no document"))?;
            let start = position
                .saturating_sub(context as usize)
                .max(self.documents[doc]);
            if start >= position {
                return Err(invalid("target has no in-document predecessor"));
            }
            batch.windows.push(
                (start..position)
                    .map(|p| Identity {
                        token: self.tokens[p],
                        occurrence: p as u64,
                        // Split namespaces cannot alias within a driver. Across different
                        // corpora the bound source SHA remains part of exact identity scope.
                        record: (match self.binding.source.split {
                            Split::Train => 0,
                            Split::Dev => 1,
                            Split::Fresh => 2,
                        } << 62)
                            | doc as u64,
                        version: 0,
                    })
                    .collect(),
            );
            batch.targets.push(self.tokens[position]);
            batch.target_positions.push(position as u64);
        }
        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(tokens: &[u16], mask: &[u8], eligible: u64) -> (CorpusSource, Vec<u8>, BosIdentity) {
        let mut bytes = vec![0; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
        for t in tokens {
            bytes.extend_from_slice(&t.to_le_bytes());
        }
        (
            CorpusSource {
                split: Split::Train,
                tokens: FilePin {
                    path: "fixture.uort".into(),
                    sha256: sha256(&bytes),
                },
                mask: FilePin {
                    path: "fixture.mask".into(),
                    sha256: sha256(mask),
                },
                expected_payload_tokens: tokens.len() as u64,
                expected_eligible_targets: eligible,
            },
            bytes,
            BosIdentity {
                token: "<|bos|>".into(),
                id: 0,
            },
        )
    }
    #[test]
    fn corpus_header_masks_documents_and_tail_are_metadata() -> Result<()> {
        let mask = [1, 1, 1, 1, 1, 0, 1, 1];
        let (source, bytes, bos) = fixture(&[0, 1, 2, 3, 0, 4, 5, 6], &mask, 5);
        let corpus = PreparedCorpus::from_bytes(source, &bytes, &mask, 8, bos)?;
        assert_eq!(corpus.binding().payload_tokens, 8);
        assert_eq!(corpus.binding().mask_selected_positions, 7);
        let first = corpus.batch(0, 2, 4)?;
        assert_eq!(first.target_positions, [1, 2, 3, 6]);
        assert_eq!(
            first
                .windows
                .iter()
                .map(|w| w.iter().map(|i| i.token).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
            [vec![0], vec![0, 1], vec![1, 2], vec![0, 4]]
        );
        assert_eq!(first.windows[3][0].record, 1);
        let tail = corpus.batch(first.next_target, 2, 4)?;
        assert_eq!(tail.target_positions, [7]);
        assert_eq!(
            tail.windows[0].iter().map(|i| i.token).collect::<Vec<_>>(),
            [4, 5]
        );
        assert_eq!(tail.next_target, 5);
        Ok(())
    }
    #[test]
    fn corpus_refuses_hash_mask_header_and_missing_bos() {
        let mask = [0, 1];
        let (source, bytes, bos) = fixture(&[0, 1], &mask, 1);
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(PreparedCorpus::from_bytes(source.clone(), &bad, &mask, 8, bos.clone()).is_err());
        let invalid_mask = [0, 2];
        let mut invalid_source = source.clone();
        invalid_source.mask.sha256 = sha256(&invalid_mask);
        assert!(
            PreparedCorpus::from_bytes(invalid_source, &bytes, &invalid_mask, 8, bos.clone())
                .is_err()
        );
        let mut wrong = source.clone();
        wrong.expected_eligible_targets = 2;
        assert!(PreparedCorpus::from_bytes(wrong, &bytes, &mask, 8, bos.clone()).is_err());
        let (source, bytes, bos) = fixture(&[1, 2], &mask, 1);
        assert!(PreparedCorpus::from_bytes(source, &bytes, &mask, 8, bos).is_err());
    }
    #[test]
    fn tokenizer_bos_is_explicit_metadata() -> Result<()> {
        let bos = BosIdentity {
            token: "<|bos|>".into(),
            id: 0,
        };
        bos.validate_tokenizer(br#"{"model":{"vocab":{"<|bos|>":0}}}"#, 8)?;
        assert!(bos
            .validate_tokenizer(br#"{"model":{"vocab":{"<|bos|>":1}}}"#, 8)
            .is_err());
        Ok(())
    }
}
