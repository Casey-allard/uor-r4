//! TRAIN-only anchor metadata. This module records token identities and original
//! corpus windows; it performs no model evaluation or admission calculation.
use crate::controller::attention_learning::evidence::is_copy_successor;
use crate::corpus::{sha256, BosIdentity, CorpusBinding, CorpusSource, PreparedCorpus, Split};
use crate::{Error, Identity, Result};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

pub const ANCHOR_SCHEMA: &str = "uor-r4.train-copy-successor-anchor/v1";
pub const SELECTION_POLICY: &str = "first-copy-successor-at-or-after-original-eligible-index/v1";
pub const ROW_CSV_POLICY: &str =
    "original-eligible-index,target-absolute-position,target-token,1/headerless-decimal-lf/v1";
pub const WINDOW_HASH_POLICY: &str =
    "uor-r4.original-corpus-window/identities-u32-u64-u64-u64-le/v1";

/// Indexes refer to the original response-masked eligible target stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorPolicy {
    pub start_eligible_index: u64,
    pub count: u64,
    pub context: u32,
}

/// The contiguous anchor ordinal is deliberately distinct from the source
/// eligible index. Neither is an absolute token occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorRow {
    pub anchor_ordinal: u64,
    pub eligible_index: u64,
    pub target_position: u64,
    pub target_token: u32,
    pub target_identity: Identity,
    pub original_window: OriginalWindowPin,
}

/// A commitment to ordered original corpus identities, not model outputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OriginalWindowPin {
    pub encoding: String,
    pub sha256: String,
}

impl OriginalWindowPin {
    fn from_window(window: &[Identity]) -> Result<Self> {
        // Exact bytes: policy UTF-8 + NUL, count u64 LE, then each Identity's
        // token u32 LE, occurrence u64 LE, record u64 LE, version u64 LE.
        let mut bytes = WINDOW_HASH_POLICY.as_bytes().to_vec();
        bytes.push(0);
        let count = u64::try_from(window.len())
            .map_err(|_| invalid("original window inventory exceeds u64"))?;
        bytes.extend_from_slice(&count.to_le_bytes());
        for identity in window {
            bytes.extend_from_slice(&identity.token.to_le_bytes());
            bytes.extend_from_slice(&identity.occurrence.to_le_bytes());
            bytes.extend_from_slice(&identity.record.to_le_bytes());
            bytes.extend_from_slice(&identity.version.to_le_bytes());
        }
        Ok(Self {
            encoding: WINDOW_HASH_POLICY.into(),
            sha256: sha256(&bytes),
        })
    }
}

/// Deserialized metadata is untrusted until `validate` succeeds against the
/// caller's authenticated TRAIN corpus and independently expected policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainAnchorManifest {
    pub schema: String,
    pub selection_policy: String,
    pub row_csv_policy: String,
    pub corpus: CorpusBinding,
    pub policy: AnchorPolicy,
    pub rows: Vec<AnchorRow>,
    pub row_csv_sha256: String,
    pub manifest_sha256: String,
}

/// Stable hash payload excludes the manifest's own digest. Field order and
/// compact serde JSON encoding are part of the versioned manifest hash domain.
#[derive(Serialize)]
struct ManifestPayload<'a> {
    schema: &'a str,
    selection_policy: &'a str,
    row_csv_policy: &'a str,
    corpus: &'a CorpusBinding,
    policy: &'a AnchorPolicy,
    rows: &'a [AnchorRow],
    row_csv_sha256: &'a str,
}

fn invalid(message: &str) -> Error {
    Error::Invalid(format!("TRAIN anchor: {message}"))
}

impl TrainAnchorManifest {
    /// Only the explicit TRAIN token/mask paths are read. There is no DEV or
    /// fresh input, tokenizer execution, or model/device dependency.
    pub fn load(
        source: &CorpusSource,
        vocabulary: u32,
        bos: &BosIdentity,
        policy: AnchorPolicy,
    ) -> Result<Self> {
        if source.split != Split::Train {
            return Err(invalid("source must be TRAIN"));
        }
        Self::build(&PreparedCorpus::load(source, vocabulary, bos)?, policy)
    }

    /// Build from a corpus already authenticated by `PreparedCorpus`.
    pub fn build(corpus: &PreparedCorpus, policy: AnchorPolicy) -> Result<Self> {
        if corpus.binding().source.split != Split::Train {
            return Err(invalid("source must be TRAIN"));
        }
        if !(1..=243).contains(&policy.context) || policy.count == 0 {
            return Err(invalid(
                "context must be 1..=243 and count must be positive",
            ));
        }
        let inventory = u64::try_from(corpus.eligible_len())
            .map_err(|_| invalid("eligible inventory exceeds u64"))?;
        let minimum_end = policy
            .start_eligible_index
            .checked_add(policy.count)
            .ok_or_else(|| invalid("selection range overflow"))?;
        if policy.start_eligible_index >= inventory || minimum_end > inventory {
            return Err(invalid("selection exceeds eligible inventory"));
        }
        let mut rows = Vec::new();
        for eligible_index in policy.start_eligible_index..inventory {
            let batch = corpus.batch(eligible_index, policy.context, 1)?;
            let target_token = batch.targets[0];
            if is_copy_successor(&batch.windows[0], target_token) {
                let anchor_ordinal = u64::try_from(rows.len())
                    .map_err(|_| invalid("anchor inventory exceeds u64"))?;
                // PreparedCorpus guarantees that target and nonempty
                // predecessor window share the original document. Reuse that
                // record/version namespace instead of deriving split/doc IDs.
                let predecessor = batch.windows[0]
                    .last()
                    .copied()
                    .ok_or_else(|| invalid("target has no original predecessor identity"))?;
                rows.push(AnchorRow {
                    anchor_ordinal,
                    eligible_index,
                    target_position: batch.target_positions[0],
                    target_token,
                    target_identity: Identity {
                        token: target_token,
                        occurrence: batch.target_positions[0],
                        ..predecessor
                    },
                    original_window: OriginalWindowPin::from_window(&batch.windows[0])?,
                });
                if anchor_ordinal + 1 == policy.count {
                    break;
                }
            }
        }
        if u64::try_from(rows.len()).ok() != Some(policy.count) {
            return Err(invalid(
                "copy-successor selection exhausted before requested count",
            ));
        }
        let mut manifest = Self {
            schema: ANCHOR_SCHEMA.into(),
            selection_policy: SELECTION_POLICY.into(),
            row_csv_policy: ROW_CSV_POLICY.into(),
            corpus: corpus.binding().clone(),
            policy,
            rows,
            row_csv_sha256: String::new(),
            manifest_sha256: String::new(),
        };
        manifest.row_csv_sha256 = sha256(&manifest.canonical_rows_csv()?);
        manifest.manifest_sha256 = manifest.payload_sha256()?;
        Ok(manifest)
    }

    /// Headerless decimal CSV uses original eligible indices, never the
    /// contiguous anchor ordinal. Structural refusal prevents duplicate or
    /// reordered rows from being silently canonicalized. This encoding helper
    /// does not authenticate row membership; use `validate` before accepting
    /// deserialized metadata.
    pub fn canonical_rows_csv(&self) -> Result<Vec<u8>> {
        if self.rows.is_empty() || u64::try_from(self.rows.len()).ok() != Some(self.policy.count) {
            return Err(invalid("row count mismatch"));
        }
        let mut csv = String::new();
        let mut previous: Option<&AnchorRow> = None;
        for (ordinal, row) in self.rows.iter().enumerate() {
            if u64::try_from(ordinal).ok() != Some(row.anchor_ordinal)
                || row.eligible_index < self.policy.start_eligible_index
                || row.target_token >= self.corpus.vocabulary
                || row.target_token == self.corpus.bos.id
                || previous.is_some_and(|p| {
                    p.eligible_index >= row.eligible_index
                        || p.target_position >= row.target_position
                })
            {
                return Err(invalid("invalid row identity or ordering"));
            }
            // Writing into String cannot fail.
            writeln!(
                &mut csv,
                "{},{},{},1",
                row.eligible_index, row.target_position, row.target_token
            )
            .map_err(|_| invalid("CSV encoding failed"))?;
            previous = Some(row);
        }
        Ok(csv.into_bytes())
    }

    fn payload_sha256(&self) -> Result<String> {
        let payload = ManifestPayload {
            schema: &self.schema,
            selection_policy: &self.selection_policy,
            row_csv_policy: &self.row_csv_policy,
            corpus: &self.corpus,
            policy: &self.policy,
            rows: &self.rows,
            row_csv_sha256: &self.row_csv_sha256,
        };
        let bytes =
            serde_json::to_vec(&payload).map_err(|_| invalid("manifest encoding failed"))?;
        Ok(sha256(&bytes))
    }

    /// Bind the caller's prospective policy independently of the serialized
    /// manifest, then recompute selection, contexts and hashes from the original
    /// authenticated corpus. Self-consistent forged metadata is not sufficient.
    pub fn validate(&self, corpus: &PreparedCorpus, expected_policy: &AnchorPolicy) -> Result<()> {
        if &self.policy != expected_policy {
            return Err(invalid("policy differs from expected registration"));
        }
        let expected = Self::build(corpus, expected_policy.clone())?;
        if self != &expected {
            return Err(invalid(
                "manifest differs from original TRAIN corpus selection",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{sha256, BosIdentity, CorpusSource, FilePin, PreparedCorpus, Split};

    fn corpus(tokens: &[u16], mask: &[u8]) -> PreparedCorpus {
        let mut bytes = vec![0; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
        for token in tokens {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
        let eligible = tokens
            .iter()
            .zip(mask)
            .enumerate()
            .filter(|(p, (t, m))| *p > 0 && **t != 0 && **m == 1)
            .count() as u64;
        let source = CorpusSource {
            split: Split::Train,
            tokens: FilePin {
                path: "unreadable-train.uort".into(),
                sha256: sha256(&bytes),
            },
            mask: FilePin {
                path: "unreadable-train.mask".into(),
                sha256: sha256(mask),
            },
            expected_payload_tokens: tokens.len() as u64,
            expected_eligible_targets: eligible,
        };
        PreparedCorpus::from_bytes(
            source,
            &bytes,
            mask,
            8,
            BosIdentity {
                token: "<|bos|>".into(),
                id: 0,
            },
        )
        .unwrap()
    }

    fn fixture() -> PreparedCorpus {
        // Two documents; masked gaps still belong to the original context.
        corpus(
            &[0, 1, 2, 1, 2, 3, 0, 4, 5, 4, 5, 4, 5],
            &[0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1],
        )
    }

    #[test]
    fn original_windows_filter_and_distinct_ordinal_domains() {
        let corpus = fixture();
        let policy = AnchorPolicy {
            start_eligible_index: 0,
            count: 3,
            context: 4,
        };
        let manifest = TrainAnchorManifest::build(&corpus, policy.clone()).unwrap();
        assert_eq!(
            manifest
                .rows
                .iter()
                .map(|r| (
                    r.anchor_ordinal,
                    r.eligible_index,
                    r.target_position,
                    r.target_token
                ))
                .collect::<Vec<_>>(),
            vec![(0, 2, 4, 2), (1, 7, 10, 5), (2, 8, 11, 4)]
        );
        let row = &manifest.rows[1];
        let window = corpus.batch(row.eligible_index, 4, 1).unwrap();
        assert_eq!(
            window.windows[0]
                .iter()
                .map(|i| i.occurrence)
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9]
        );
        assert!(window.windows[0].iter().all(|i| i.record == 1));
        manifest.validate(&corpus, &policy).unwrap();
    }

    #[test]
    fn prefix_tail_exhaustion_bounds_and_overflow() {
        let corpus = fixture();
        let policy = AnchorPolicy {
            start_eligible_index: 8,
            count: 2,
            context: 4,
        };
        let manifest = TrainAnchorManifest::build(&corpus, policy).unwrap();
        assert_eq!(
            manifest
                .rows
                .iter()
                .map(|r| r.target_position)
                .collect::<Vec<_>>(),
            vec![11, 12]
        );
        for policy in [
            AnchorPolicy {
                start_eligible_index: 8,
                count: 3,
                context: 4,
            },
            AnchorPolicy {
                start_eligible_index: 10,
                count: 1,
                context: 4,
            },
            AnchorPolicy {
                start_eligible_index: 0,
                count: 0,
                context: 4,
            },
            AnchorPolicy {
                start_eligible_index: u64::MAX,
                count: 2,
                context: 4,
            },
            AnchorPolicy {
                start_eligible_index: 0,
                count: 1,
                context: 0,
            },
            AnchorPolicy {
                start_eligible_index: 0,
                count: 1,
                context: 244,
            },
            AnchorPolicy {
                start_eligible_index: 0,
                count: 4,
                context: 2,
            },
        ] {
            assert!(TrainAnchorManifest::build(&corpus, policy).is_err());
        }
    }

    #[test]
    fn exact_headerless_csv_hash_and_serialization() {
        let corpus = fixture();
        let manifest = TrainAnchorManifest::build(
            &corpus,
            AnchorPolicy {
                start_eligible_index: 0,
                count: 3,
                context: 4,
            },
        )
        .unwrap();
        let expected = b"2,4,2,1\n7,10,5,1\n8,11,4,1\n";
        assert_eq!(manifest.canonical_rows_csv().unwrap(), expected);
        assert_eq!(manifest.row_csv_sha256, sha256(expected));
        let mut json = serde_json::to_value(&manifest).unwrap();
        let decoded: TrainAnchorManifest = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(decoded, manifest);
        json["unexpected"] = true.into();
        assert!(serde_json::from_value::<TrainAnchorManifest>(json).is_err());
        for (section, key) in [("policy", "arbitrary"), ("corpus", "arbitrary")] {
            let mut json = serde_json::to_value(&manifest).unwrap();
            json[section][key] = true.into();
            assert!(serde_json::from_value::<TrainAnchorManifest>(json).is_err());
        }
        let mut json = serde_json::to_value(&manifest).unwrap();
        json["corpus"]["source"]["split"] = "unknown".into();
        assert!(serde_json::from_value::<TrainAnchorManifest>(json).is_err());
    }

    #[test]
    fn tampering_is_rejected_against_expected_policy_and_original_corpus() {
        let corpus = fixture();
        let policy = AnchorPolicy {
            start_eligible_index: 0,
            count: 3,
            context: 4,
        };
        let manifest = TrainAnchorManifest::build(&corpus, policy.clone()).unwrap();
        let mut mutations = Vec::new();
        let mut m = manifest.clone();
        m.rows[0].target_token = 7;
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows[1] = m.rows[0].clone();
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows.swap(0, 1);
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows[0].eligible_index = 3;
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows[0].anchor_ordinal = 2;
        mutations.push(m);
        let mut m = manifest.clone();
        m.row_csv_sha256 = "0".repeat(64);
        mutations.push(m);
        let mut m = manifest.clone();
        m.manifest_sha256 = "0".repeat(64);
        mutations.push(m);
        let mut m = manifest.clone();
        m.corpus.source.split = Split::Dev;
        mutations.push(m);
        let mut m = manifest.clone();
        m.corpus.source.tokens.path = "elsewhere".into();
        mutations.push(m);
        let mut m = manifest.clone();
        m.corpus.source.mask.sha256 = "0".repeat(64);
        mutations.push(m);
        let mut m = manifest.clone();
        m.corpus.identity_policy.push('x');
        mutations.push(m);
        let mut m = manifest.clone();
        m.policy.context = 3;
        mutations.push(m);
        for m in mutations {
            assert!(m.validate(&corpus, &policy).is_err());
        }
        // A correctly re-hashed but misclassified row is still rejected by
        // recomputation of the shared predicate on its original corpus window.
        let mut forged = manifest;
        forged.rows[0].eligible_index = 3;
        forged.rows[0].target_position = 5;
        forged.rows[0].target_token = 3;
        forged.row_csv_sha256 = sha256(&forged.canonical_rows_csv().unwrap());
        forged.manifest_sha256 = forged.payload_sha256().unwrap();
        assert!(forged.validate(&corpus, &policy).is_err());
    }

    #[test]
    fn context_clipping_and_nearest_query_are_source_defined() {
        // Previous query has successor 2, but the nearer query has successor 3.
        let corpus = corpus(
            &[0, 1, 2, 1, 3, 1, 2, 0, 1, 2],
            &[0, 1, 1, 1, 1, 1, 1, 0, 1, 1],
        );
        let policy = AnchorPolicy {
            start_eligible_index: 0,
            count: 1,
            context: 6,
        };
        // p4 is not Psucc (target3 vs previous successor2); p6 must not
        // incorrectly use the older query at p1 instead of nearest p3.
        assert!(TrainAnchorManifest::build(&corpus, policy).is_err());
        // A four-token prefix admits p4 in the first fixture; a two-token
        // prefix removes the repeated query rather than inventing context.
        let corpus = fixture();
        assert!(TrainAnchorManifest::build(
            &corpus,
            AnchorPolicy {
                start_eligible_index: 0,
                count: 1,
                context: 2
            }
        )
        .is_err());
    }

    #[test]
    fn corpus_authentication_refuses_bad_pins_counts_bos_and_vocabulary() {
        let corpus = fixture();
        let mask = [0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1];
        let mut bytes = vec![0; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(corpus.tokens().len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
        for token in corpus.tokens() {
            bytes.extend_from_slice(&(*token as u16).to_le_bytes());
        }
        let source = corpus.binding().source.clone();
        let bos = corpus.binding().bos.clone();
        assert!(PreparedCorpus::from_bytes(source.clone(), &bytes, &mask, 8, bos.clone()).is_ok());
        let mut wrong = source.clone();
        wrong.tokens.sha256 = "0".repeat(64);
        assert!(PreparedCorpus::from_bytes(wrong, &bytes, &mask, 8, bos.clone()).is_err());
        let mut wrong = source.clone();
        wrong.mask.sha256 = "not-a-hash".into();
        assert!(PreparedCorpus::from_bytes(wrong, &bytes, &mask, 8, bos.clone()).is_err());
        let mut wrong = source.clone();
        wrong.expected_eligible_targets += 1;
        assert!(PreparedCorpus::from_bytes(wrong, &bytes, &mask, 8, bos.clone()).is_err());
        let mut wrong = source.clone();
        wrong.expected_payload_tokens += 1;
        assert!(PreparedCorpus::from_bytes(wrong, &bytes, &mask, 8, bos.clone()).is_err());
        assert!(PreparedCorpus::from_bytes(source.clone(), &bytes, &mask, 7, bos).is_err());
        assert!(PreparedCorpus::from_bytes(
            source,
            &bytes,
            &mask,
            8,
            BosIdentity {
                token: "other".into(),
                id: 1
            }
        )
        .is_err());
    }

    #[test]
    fn load_rejects_non_train_before_touching_any_paths() {
        let corpus = fixture();
        let mut source = corpus.binding().source.clone();
        source.split = Split::Dev;
        let result = TrainAnchorManifest::load(
            &source,
            8,
            &corpus.binding().bos,
            AnchorPolicy {
                start_eligible_index: 0,
                count: 1,
                context: 4,
            },
        );
        assert!(result.unwrap_err().to_string().contains("TRAIN"));
        // The in-memory TRAIN fixture needs neither its missing file paths nor any DEV source.
        TrainAnchorManifest::build(
            &corpus,
            AnchorPolicy {
                start_eligible_index: 0,
                count: 1,
                context: 4,
            },
        )
        .unwrap();
    }

    #[test]
    fn target_identity_and_versioned_original_window_are_explicit() {
        let corpus = fixture();
        let policy = AnchorPolicy {
            start_eligible_index: 0,
            count: 3,
            context: 4,
        };
        let manifest = TrainAnchorManifest::build(&corpus, policy.clone()).unwrap();
        assert_eq!(
            manifest.rows[0].target_identity,
            crate::Identity {
                token: 2,
                occurrence: 4,
                record: 0,
                version: 0
            }
        );
        assert_eq!(
            manifest.rows[1].target_identity,
            crate::Identity {
                token: 5,
                occurrence: 10,
                record: 1,
                version: 0
            }
        );
        assert_eq!(
            manifest.rows[1].original_window.encoding,
            WINDOW_HASH_POLICY
        );
        // Exact domain: encoding bytes plus NUL, u64 LE length, then ordered
        // token u32/occurrence u64/record u64/version u64, all little endian.
        let mut expected = WINDOW_HASH_POLICY.as_bytes().to_vec();
        expected.push(0);
        expected.extend_from_slice(&4u64.to_le_bytes());
        for (token, occurrence) in [(0u32, 6u64), (4, 7), (5, 8), (4, 9)] {
            expected.extend_from_slice(&token.to_le_bytes());
            expected.extend_from_slice(&occurrence.to_le_bytes());
            expected.extend_from_slice(&1u64.to_le_bytes());
            expected.extend_from_slice(&0u64.to_le_bytes());
        }
        assert_eq!(manifest.rows[1].original_window.sha256, sha256(&expected));
        assert_eq!(
            manifest.canonical_rows_csv().unwrap(),
            b"2,4,2,1\n7,10,5,1\n8,11,4,1\n"
        );
        let mut mutations = Vec::new();
        let mut m = manifest.clone();
        m.rows[1].target_identity.record = 0;
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows[1].target_identity.version = 1;
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows[1].target_identity.occurrence = 9;
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows[1].target_identity.token = 4;
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows[1].original_window.sha256 = "0".repeat(64);
        mutations.push(m);
        let mut m = manifest.clone();
        m.rows[1].original_window.encoding.push('x');
        mutations.push(m);
        for mut m in mutations {
            m.manifest_sha256 = m.payload_sha256().unwrap();
            assert!(m.validate(&corpus, &policy).is_err());
        }
    }
}
