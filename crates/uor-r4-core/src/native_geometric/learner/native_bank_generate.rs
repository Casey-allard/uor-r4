//! Artifact-only native bank generation over an externally pinned snapshot.
//! The caller admits the store view and supplies actual chronological Context
//! tokens. This module neither parses relations nor selects an answer record.
//! Integer operators and the full Generate/Copy alias pool are reused unchanged.
//! Bank admission and replay allocate; this is not an allocation-free kernel or
//! a qualification of general chat, store completeness or complete-path D11 cost.
#![forbid(unsafe_code)]
use super::geometric_generate::{GenerateReadCounts, NativeGeometricGenerate};
use super::geometric_read_state_bridge::{BridgeReadCounts, NativeGeometricReadStateBridge};
use sha2::{Digest, Sha256};
use std::path::Path;
use uor_r4_integer::{
    geometric_context::NativeContextState,
    geometric_cue_carrier::{CueAngularQ4, CueCarrierMetadata, CueJointQ4},
    geometric_occurrence_read::{
        FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity, MAX_SEQUENCE,
    },
    geometric_prefix_transport::{PrefixAngularQ4, PrefixTransportMetadata},
    geometric_source_actions::SourceActionBinding,
    geometric_source_emission_view::SourceEmissionView,
    geometric_source_realizer::{
        BankCandidateTrace, NativeArtifactBinding, NativeSourceRealizer, PrefixBankRealizerTrace,
        SourceBankSegment,
    },
    geometric_vocabulary_actions::{NativeVocabularyActions, VocabularyActionTrace},
    h4_tables::H4Code,
};
pub type Result<T> = std::result::Result<T, NativeBankGenerateError>;
#[derive(Debug)]
pub enum NativeBankGenerateError {
    Artifact(String),
    Input(String),
    SourceStatus {
        record: u64,
        status: SnapshotSourceStatus,
    },
    ContextBudget {
        actual: usize,
        maximum: usize,
    },
    Arithmetic,
    Execution(String),
}
impl std::fmt::Display for NativeBankGenerateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for NativeBankGenerateError {}
fn execution(e: impl std::fmt::Display) -> NativeBankGenerateError {
    NativeBankGenerateError::Execution(e.to_string())
}
fn artifact(e: impl std::fmt::Display) -> NativeBankGenerateError {
    NativeBankGenerateError::Artifact(e.to_string())
}
fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
fn verify_bytes(bytes: &[u8], expected: &str) -> Result<()> {
    if hash(bytes) != expected {
        return Err(artifact("expected artifact digest differs"));
    }
    Ok(())
}
/// Expected digests must come from a separately trusted manifest/receipt.
pub struct BoundNativeBytes<'a> {
    pub bytes: &'a [u8],
    pub sha256: &'a str,
}
/// Sidecar metadata binds packed payloads and their source artifact. Unreceipted
/// joint cue payloads are rejected, rather than silently installed or ignored.
pub struct NativeBankArtifacts<'a> {
    pub native_directory: &'a Path,
    pub source_binding: &'a NativeArtifactBinding,
    pub generate: BoundNativeBytes<'a>,
    pub bridge: Option<BoundNativeBytes<'a>>,
    pub cue_packed: &'a [u8],
    pub cue_joint_packed: Option<&'a [u8]>,
    pub cue_metadata: &'a CueCarrierMetadata,
    pub prefix_packed: &'a [u8],
    pub prefix_metadata: &'a PrefixTransportMetadata,
    pub exp: BoundNativeBytes<'a>,
}
/// This pin is provenance supplied by the store adapter, not a learned feature.
/// Store enumeration/completeness and lineage authenticity remain caller-owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BankPin {
    pub lineage: u64,
    pub commit: u64,
    pub scope: Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotSourceStatus {
    Found,
    Conflict,
    Unresolved,
    Absent,
    Evicted,
    NoHistory,
}
#[derive(Clone, Debug)]
pub struct OwnedBankSource {
    pub event: u64,
    pub record: u64,
    pub commit: u64,
    pub scope: Vec<u8>,
    pub entity: Vec<u32>,
    pub relation: u32,
    pub view: u32,
    pub status: SnapshotSourceStatus,
    pub original_token_ids: Vec<u32>,
}
#[derive(Clone, Debug)]
pub enum OwnedBankSegment {
    Source(OwnedBankSource),
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
pub struct PinnedBankSnapshot {
    pub pin: BankPin,
    pub segments: Vec<OwnedBankSegment>,
    pub query_ids: Vec<u32>,
}
/// Immutable admitted inputs; callers cannot change records beneath their views.
pub struct AdmittedBank {
    pin: BankPin,
    segments: Vec<OwnedBankSegment>,
    views: Vec<Option<SourceEmissionView>>,
    query_ids: Vec<u32>,
    base_tokens: usize,
    source_metadata_sha256: String,
}
impl AdmittedBank {
    pub fn pin(&self) -> &BankPin {
        &self.pin
    }
    pub fn base_tokens(&self) -> usize {
        self.base_tokens
    }
    fn has_source(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, OwnedBankSegment::Source(_)))
    }
    fn borrowed_segments(&self) -> Result<Vec<SourceBankSegment<'_>>> {
        self.segments
            .iter()
            .zip(&self.views)
            .map(|(s, view)| {
                Ok(match s {
                    OwnedBankSegment::Source(s) => SourceBankSegment::Source {
                        event: s.event,
                        frame: SelectedRecordFrame {
                            identity: SourceIdentity {
                                record: s.record,
                                commit: s.commit,
                            },
                            metadata: FrameMetadata {
                                scope: &s.scope,
                                entity: &s.entity,
                                relation: s.relation,
                                view: s.view,
                                status: FrameStatus::Found,
                            },
                            token_ids: &s.original_token_ids,
                        },
                        view: view
                            .as_ref()
                            .ok_or_else(|| execution("source lexical view missing"))?,
                    },
                    OwnedBankSegment::Context {
                        event,
                        role,
                        token_ids,
                    } => SourceBankSegment::Context {
                        event: *event,
                        role: *role,
                        token_ids,
                    },
                })
            })
            .collect()
    }
    fn no_source_ids(&self, actual: &[u32]) -> Result<Vec<u32>> {
        let mut ids = Vec::with_capacity(projected(self.base_tokens, actual.len())?);
        for s in &self.segments {
            match s {
                OwnedBankSegment::Context { token_ids, .. } => ids.extend_from_slice(token_ids),
                _ => return Err(execution("no-source replay received Source")),
            }
        }
        ids.extend_from_slice(&self.query_ids);
        ids.extend_from_slice(actual);
        Ok(ids)
    }
}
/// Hosts the already admitted native operators, not training state or answers.
pub struct NativeBankGenerator {
    model: NativeSourceRealizer,
    generate: NativeGeometricGenerate,
    bridge: Option<NativeGeometricReadStateBridge>,
    cue: CueAngularQ4,
    prefix: PrefixAngularQ4,
    pool: NativeVocabularyActions,
    generate_sha256: String,
    bridge_sha256: Option<String>,
}
pub struct NativeBridgeWitness {
    pub selected_ordinal: usize,
    pub selected_candidate: BankCandidateTrace,
    pub query_state: Vec<H4Code>,
    pub source_state: Vec<H4Code>,
    pub action_codes: Vec<H4Code>,
    pub action_scores_q24: Vec<i64>,
    pub counts: BridgeReadCounts,
}
pub struct NativeBankGenerateStep {
    pub actual_prefix_ids: Vec<u32>,
    pub post_state: Vec<H4Code>,
    pub copy_token_ids: Vec<u32>,
    pub copy_raw_scores_q24: Vec<i64>,
    pub generate_raw_scores_q24: Vec<i64>,
    pub actions: VocabularyActionTrace,
    pub bank_trace: Option<PrefixBankRealizerTrace>,
    pub bridge: Option<NativeBridgeWitness>,
    pub generate_counts: GenerateReadCounts,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationStop {
    Eos,
    OutputLimit,
    ContextLimit,
}
/// Kept steps allocate their full traces. Limit them separately from tokens.
#[derive(Clone, Copy, Debug)]
pub struct GenerationLimits {
    pub maximum_tokens: usize,
    pub maximum_retained_steps: usize,
}
pub struct NativeBankGeneration {
    pub pin: BankPin,
    pub generated_ids: Vec<u32>,
    pub stop: GenerationStop,
    pub steps: Vec<NativeBankGenerateStep>,
    pub executed_steps: usize,
    pub omitted_step_traces: usize,
}
fn projected(base: usize, prefix: usize) -> Result<usize> {
    let actual = base
        .checked_add(prefix)
        .ok_or(NativeBankGenerateError::Arithmetic)?;
    if actual > MAX_SEQUENCE {
        return Err(NativeBankGenerateError::ContextBudget {
            actual,
            maximum: MAX_SEQUENCE,
        });
    }
    Ok(actual)
}
fn valid_ids(binding: &SourceActionBinding, ids: &[u32]) -> Result<()> {
    if ids.iter().any(|&id| !binding.admits_token(id)) {
        return Err(NativeBankGenerateError::Input(
            "unknown or sparse-hole token".into(),
        ));
    }
    Ok(())
}
fn source_pin_admit(
    pin: &BankPin,
    source: &OwnedBankSource,
    binding: &SourceActionBinding,
) -> Result<()> {
    if source.status != SnapshotSourceStatus::Found {
        return Err(NativeBankGenerateError::SourceStatus {
            record: source.record,
            status: source.status,
        });
    }
    if source.scope != pin.scope || source.commit > pin.commit || source.entity.is_empty() {
        return Err(NativeBankGenerateError::Input(
            "source scope/commit/entity differs from pin".into(),
        ));
    }
    valid_ids(binding, &source.entity)?;
    valid_ids(binding, &source.original_token_ids)?;
    Ok(())
}
fn earliest_raw_max(scores: &[i64]) -> Result<usize> {
    let mut winner = 0;
    let mut best = *scores
        .first()
        .ok_or_else(|| execution("bridge Copy candidates empty"))?;
    for (i, &score) in scores.iter().enumerate().skip(1) {
        if score > best {
            winner = i;
            best = score;
        }
    }
    Ok(winner)
}
fn summed_copy_scores(
    heads: &[uor_r4_integer::geometric_source_realizer::HeadTrace],
    count: usize,
) -> Result<Vec<i64>> {
    let mut scores = vec![0i64; count];
    for h in heads {
        if h.scores_q24.len() != count {
            return Err(execution("Copy head candidate count differs"));
        }
        for (i, &s) in h.scores_q24.iter().enumerate() {
            scores[i] = scores[i]
                .checked_add(s)
                .ok_or(NativeBankGenerateError::Arithmetic)?;
        }
    }
    Ok(scores)
}
impl NativeBankGenerator {
    pub fn load(a: NativeBankArtifacts<'_>) -> Result<Self> {
        verify_bytes(a.generate.bytes, a.generate.sha256)?;
        verify_bytes(a.exp.bytes, a.exp.sha256)?;
        let model = NativeSourceRealizer::load_native(a.native_directory, a.source_binding)
            .map_err(artifact)?;
        let generate = NativeGeometricGenerate::from_bytes(a.generate.bytes, model.binding())
            .map_err(artifact)?;
        let lanes = model.context_config().heads * model.context_config().lanes_per_head;
        if generate.lanes() != lanes {
            return Err(artifact("Generate/source lanes differ"));
        }
        let bridge_sha256 = a.bridge.as_ref().map(|b| b.sha256.to_string());
        let bridge = a
            .bridge
            .map(|b| {
                verify_bytes(b.bytes, b.sha256)?;
                let bridge = NativeGeometricReadStateBridge::from_bytes(b.bytes, model.binding())
                    .map_err(artifact)?;
                if bridge.lanes() != lanes {
                    return Err(artifact("bridge/source lanes differ"));
                }
                Ok(bridge)
            })
            .transpose()?;
        let mut cue =
            CueAngularQ4::new(a.cue_metadata.potential, a.cue_packed).map_err(artifact)?;
        match (&a.cue_metadata.joint, a.cue_joint_packed) {
            (Some(m), Some(p)) => {
                let j = CueJointQ4::new(m.config, p.to_vec()).map_err(artifact)?;
                if j.metadata() != *m {
                    return Err(artifact("joint cue payload receipt differs"));
                }
                cue = cue.with_joint(j).map_err(artifact)?;
            }
            (None, None) => {}
            _ => return Err(artifact("joint cue payload/receipt presence differs")),
        }
        let prefix =
            PrefixAngularQ4::new(a.prefix_metadata.potential, a.prefix_packed).map_err(artifact)?;
        let cc = model.compile_cue_carrier(cue.clone()).map_err(artifact)?;
        let pp = model
            .compile_prefix_transport(
                &cc,
                PrefixAngularQ4::new(prefix.config(), prefix.packed_coefficients())
                    .map_err(artifact)?,
            )
            .map_err(artifact)?;
        if cc.metadata() != a.cue_metadata || pp.metadata() != a.prefix_metadata {
            return Err(artifact("cue/prefix exact native metadata differs"));
        }
        let pool =
            NativeVocabularyActions::new(model.binding().clone(), a.exp.bytes).map_err(artifact)?;
        Ok(Self {
            model,
            generate,
            bridge,
            cue,
            prefix,
            pool,
            generate_sha256: a.generate.sha256.into(),
            bridge_sha256,
        })
    }
    pub fn source_binding(&self) -> &NativeArtifactBinding {
        self.model.artifact_binding()
    }
    pub fn generate_sha256(&self) -> &str {
        &self.generate_sha256
    }
    pub fn bridge_sha256(&self) -> Option<&str> {
        self.bridge_sha256.as_deref()
    }
    pub fn admit_bank(&self, s: PinnedBankSnapshot) -> Result<AdmittedBank> {
        if s.pin.scope.is_empty() || s.query_ids.is_empty() || s.segments.len() > MAX_SEQUENCE {
            return Err(NativeBankGenerateError::Input(
                "empty scope/query or segment cap exceeded".into(),
            ));
        }
        valid_ids(self.model.binding(), &s.query_ids)?;
        let mut base = s.query_ids.len();
        projected(base, 0)?;
        let mut views = Vec::with_capacity(s.segments.len());
        for (index, segment) in s.segments.iter().enumerate() {
            let view = match segment {
                OwnedBankSegment::Source(source) => {
                    source_pin_admit(&s.pin, source, self.model.binding())?;
                    if index == 0
                        || !matches!(&s.segments[index - 1],
                        OwnedBankSegment::Context { token_ids, .. } if !token_ids.is_empty())
                    {
                        return Err(NativeBankGenerateError::Input(
                            "Source requires its preceding actual nonempty Context".into(),
                        ));
                    }
                    Some(
                        self.model
                            .compile_view(&source.original_token_ids)
                            .map_err(execution)?,
                    )
                }
                OwnedBankSegment::Context { token_ids, .. } => {
                    valid_ids(self.model.binding(), token_ids)?;
                    None
                }
            };
            let n = match (segment, &view) {
                (OwnedBankSegment::Source(_), Some(v)) => v.emitted_token_ids().len(),
                (OwnedBankSegment::Context { token_ids, .. }, _) => token_ids.len(),
                _ => return Err(execution("admitted source view missing")),
            };
            base = base
                .checked_add(n)
                .ok_or(NativeBankGenerateError::Arithmetic)?;
            projected(base, 0)?;
            views.push(view);
        }
        projected(base, 0)?;
        Ok(AdmittedBank {
            pin: s.pin,
            segments: s.segments,
            views,
            query_ids: s.query_ids,
            base_tokens: base,
            source_metadata_sha256: self.model.artifact_binding().metadata_sha256.clone(),
        })
    }
    pub fn step(
        &mut self,
        bank: &AdmittedBank,
        actual_prefix: &[u32],
    ) -> Result<NativeBankGenerateStep> {
        if bank.source_metadata_sha256 != self.model.artifact_binding().metadata_sha256 {
            return Err(artifact(
                "admitted bank belongs to a different source artifact",
            ));
        }
        projected(bank.base_tokens, actual_prefix.len())?;
        valid_ids(self.model.binding(), actual_prefix)?;
        let (mut codes, copy_ids, scores, bank_trace) = if bank.has_source() {
            let cue = self
                .model
                .compile_cue_carrier(self.cue.clone())
                .map_err(execution)?;
            let prefix = self
                .model
                .compile_prefix_transport(
                    &cue,
                    PrefixAngularQ4::new(self.prefix.config(), self.prefix.packed_coefficients())
                        .map_err(execution)?,
                )
                .map_err(execution)?;
            let trace = self
                .model
                .read_bank_with_prefix_transport(
                    &bank.borrowed_segments()?,
                    &bank.query_ids,
                    actual_prefix,
                    &cue,
                    &prefix,
                )
                .map_err(execution)?;
            let b = &trace.cue_bank.bank;
            let codes = b
                .context
                .states
                .last()
                .ok_or_else(|| execution("bank final state absent"))?
                .iter()
                .copied()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(execution)?;
            let ids = b
                .candidates
                .iter()
                .map(|c| c.occurrence.token_id)
                .collect::<Vec<_>>();
            let scores = summed_copy_scores(&b.heads, ids.len())?;
            (codes, ids, scores, Some(trace))
        } else {
            let cfg = self.model.context_config();
            let (tables, geometry) = self.model.context_encoder_parts();
            let mut state =
                NativeContextState::new(cfg.heads, cfg.lanes_per_head).map_err(execution)?;
            for id in bank.no_source_ids(actual_prefix)? {
                state
                    .step(id as usize, tables, geometry)
                    .map_err(execution)?;
            }
            (state.states().to_vec(), Vec::new(), Vec::new(), None)
        };
        let bridge_witness = if let (Some(bridge), Some(trace)) = (&self.bridge, &bank_trace) {
            let selected = earliest_raw_max(&scores)?;
            let b = &trace.cue_bank.bank;
            let candidate = b
                .candidates
                .get(selected)
                .ok_or_else(|| execution("selected occurrence missing"))?;
            let source = b
                .context
                .states
                .get(candidate.context_position)
                .ok_or_else(|| execution("selected cumulative source state missing"))?
                .iter()
                .copied()
                .map(H4Code::try_from)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(execution)?;
            let query = codes.clone();
            let mut actions = vec![H4Code::IDENTITY; codes.len()];
            let mut action_scores = vec![0; codes.len() * 120];
            let mut counts = BridgeReadCounts::default();
            bridge
                .apply_into(
                    &query,
                    &source,
                    &mut codes,
                    &mut actions,
                    &mut action_scores,
                    &mut counts,
                )
                .map_err(execution)?;
            Some(NativeBridgeWitness {
                selected_ordinal: selected,
                selected_candidate: candidate.clone(),
                query_state: query,
                source_state: source,
                action_codes: actions,
                action_scores_q24: action_scores,
                counts,
            })
        } else {
            None
        };
        let mut gen_scores = vec![0; self.generate.vocab_size()];
        let mut counts = GenerateReadCounts::default();
        self.generate
            .score_into(&codes, &mut gen_scores, &mut counts)
            .map_err(execution)?;
        let actions = self
            .pool
            .reduce_trace(&gen_scores, &copy_ids, &scores)
            .map_err(execution)?;
        Ok(NativeBankGenerateStep {
            actual_prefix_ids: actual_prefix.to_vec(),
            post_state: codes,
            copy_token_ids: copy_ids,
            copy_raw_scores_q24: scores,
            generate_raw_scores_q24: gen_scores,
            actions,
            bank_trace,
            bridge: bridge_witness,
            generate_counts: counts,
        })
    }
    pub fn generate(
        &mut self,
        bank: &AdmittedBank,
        limits: GenerationLimits,
    ) -> Result<NativeBankGeneration> {
        let eos = self.model.binding().eos_token_id();
        let mut steps = Vec::new();
        let (ids, stop) =
            own_prefix_loop(bank.base_tokens, limits.maximum_tokens, eos, |actual| {
                let step = self.step(bank, actual)?;
                let id = step.actions.summary.chosen_token_id;
                if steps.len() < limits.maximum_retained_steps {
                    steps.push(step);
                }
                Ok(id)
            })?;
        let executed_steps = ids.len();
        Ok(NativeBankGeneration {
            pin: bank.pin.clone(),
            generated_ids: ids,
            stop,
            omitted_step_traces: executed_steps - steps.len(),
            executed_steps,
            steps,
        })
    }
}
fn own_prefix_loop(
    mut_base: usize,
    maximum: usize,
    eos: u32,
    mut step: impl FnMut(&[u32]) -> Result<u32>,
) -> Result<(Vec<u32>, GenerationStop)> {
    projected(mut_base, 0)?;
    if maximum > MAX_SEQUENCE {
        return Err(NativeBankGenerateError::Input(
            "output limit exceeds128".into(),
        ));
    }
    let mut ids = Vec::new();
    while ids.len() < maximum {
        if mut_base + ids.len() >= MAX_SEQUENCE {
            return Ok((ids, GenerationStop::ContextLimit));
        }
        let id = step(&ids)?;
        ids.push(id);
        if id == eos {
            return Ok((ids, GenerationStop::Eos));
        }
    }
    Ok((ids, GenerationStop::OutputLimit))
}

#[cfg(test)]
mod tests {
    use super::*;
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"Ġ":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"},{"id":7,"content":"z"}]}"#;
    #[test]
    fn bank_generation_actual_feedback_eos_and_distinct_caps() -> Result<()> {
        let (ids, stop) = own_prefix_loop(10, 8, 1, |actual| match actual {
            [] => Ok(4),
            [4] => Ok(7),
            [4, 7] => Ok(1),
            _ => Err(execution("feedback is not actual chosen prefix")),
        })?;
        assert_eq!(ids, [4, 7, 1]);
        assert_eq!(stop, GenerationStop::Eos);
        let (ids, stop) = own_prefix_loop(126, 8, 1, |_| Ok(4))?;
        assert_eq!(ids, [4, 4]);
        assert_eq!(stop, GenerationStop::ContextLimit);
        let (ids, stop) = own_prefix_loop(126, 1, 1, |_| Ok(4))?;
        assert_eq!(ids, [4]);
        assert_eq!(stop, GenerationStop::OutputLimit);
        let (ids, stop) = own_prefix_loop(128, 8, 1, |_| Err(execution("must not execute")))?;
        assert!(ids.is_empty());
        assert_eq!(stop, GenerationStop::ContextLimit);
        assert!(own_prefix_loop(129, 1, 1, |_| Ok(4)).is_err());
        assert!(own_prefix_loop(1, 129, 1, |_| Ok(4)).is_err());
        Ok(())
    }
    #[test]
    fn bank_generation_pin_status_and_sparse_holes_are_not_absence() -> Result<()> {
        let binding = SourceActionBinding::new(TOK.as_bytes()).map_err(artifact)?;
        assert!(valid_ids(&binding, &[6]).is_err());
        valid_ids(&binding, &[7])?;
        let pin = BankPin {
            lineage: 9,
            commit: 20,
            scope: b"owner".to_vec(),
        };
        let mut source = OwnedBankSource {
            event: 99,
            record: 10,
            commit: 20,
            scope: pin.scope.clone(),
            entity: vec![4],
            relation: 7,
            view: 0,
            status: SnapshotSourceStatus::Found,
            original_token_ids: vec![7],
        };
        source_pin_admit(&pin, &source, &binding)?;
        for status in [
            SnapshotSourceStatus::Conflict,
            SnapshotSourceStatus::Unresolved,
            SnapshotSourceStatus::Absent,
            SnapshotSourceStatus::Evicted,
            SnapshotSourceStatus::NoHistory,
        ] {
            source.status = status;
            assert!(matches!(source_pin_admit(&pin, &source, &binding),
                Err(NativeBankGenerateError::SourceStatus { record: 10, status: s }) if s == status));
        }
        source.status = SnapshotSourceStatus::Found;
        source.commit = 21;
        assert!(source_pin_admit(&pin, &source, &binding).is_err());
        source.commit = 20;
        source.scope = b"other".to_vec();
        assert!(source_pin_admit(&pin, &source, &binding).is_err());
        source.scope = pin.scope.clone();
        source.original_token_ids = vec![6];
        assert!(source_pin_admit(&pin, &source, &binding).is_err());
        Ok(())
    }
    #[test]
    fn bank_generation_chronology_and_source_identity_preserved() -> Result<()> {
        let source = OwnedBankSource {
            event: 3,
            record: 42,
            commit: 7,
            scope: vec![9],
            entity: vec![4],
            relation: 8,
            view: 2,
            status: SnapshotSourceStatus::Found,
            original_token_ids: vec![4],
        };
        let bank = AdmittedBank {
            pin: BankPin {
                lineage: 1,
                commit: 7,
                scope: vec![9],
            },
            segments: vec![
                OwnedBankSegment::Context {
                    event: 99,
                    role: 2,
                    token_ids: vec![7, 4],
                },
                OwnedBankSegment::Context {
                    event: 3,
                    role: 5,
                    token_ids: vec![3],
                },
            ],
            views: vec![None, None],
            query_ids: vec![4, 7],
            base_tokens: 5,
            source_metadata_sha256: String::new(),
        };
        assert_eq!(bank.no_source_ids(&[7, 3])?, [7, 4, 3, 4, 7, 7, 3]);
        let borrowed = bank.borrowed_segments()?;
        assert!(
            matches!(&borrowed[0], SourceBankSegment::Context { event:99, role:2, token_ids } if *token_ids == [7,4])
        );
        assert!(
            matches!(&borrowed[1], SourceBankSegment::Context { event:3, role:5, token_ids } if *token_ids == [3])
        );
        let mut invalid = bank;
        invalid.segments.push(OwnedBankSegment::Source(source));
        invalid.views.push(None);
        assert!(invalid.no_source_ids(&[]).is_err());
        assert!(invalid.borrowed_segments().is_err());
        let compiler = uor_r4_integer::geometric_source_emission_view::SourceEmissionCompiler::new(
            TOK.as_bytes(),
        )
        .map_err(artifact)?;
        invalid.views[2] = Some(compiler.compile(&[4]).map_err(execution)?);
        let segments = invalid.borrowed_segments()?;
        match &segments[2] {
            SourceBankSegment::Source { frame, view, event } => {
                assert_eq!(
                    (*event, frame.identity.record, frame.identity.commit),
                    (3, 42, 7)
                );
                assert_eq!(frame.metadata.scope, &[9]);
                assert_eq!(frame.token_ids, &[4]);
                assert_eq!(view.original_token_ids(), &[4]);
            }
            _ => return Err(execution("source replaced by Context")),
        }
        Ok(())
    }
    #[test]
    fn bank_generation_copy_alias_order_checked_sums_and_strict_ties() -> Result<()> {
        use uor_r4_integer::geometric_source_realizer::HeadTrace;
        let head = |scores| HeadTrace {
            scores_q24: scores,
            weights_q31: Vec::new(),
            no_read_q24: i64::MAX,
            no_read_weight_q31: 0,
            total_weight_q31: 0,
        };
        let scores = summed_copy_scores(&[head(vec![5, 7, 5]), head(vec![2, 0, 2])], 3)?;
        assert_eq!(scores, [7, 7, 7]); // aliases/occurrences remain separate and in original order
        assert_eq!(earliest_raw_max(&scores)?, 0);
        assert_eq!(earliest_raw_max(&[-4, 3, 3, 2])?, 1);
        assert!(earliest_raw_max(&[]).is_err());
        assert!(summed_copy_scores(&[head(vec![1])], 2).is_err());
        assert!(summed_copy_scores(&[head(vec![i64::MAX]), head(vec![1])], 1).is_err());
        Ok(())
    }
    #[test]
    fn bank_generation_artifact_digest_and_projected_budget() -> Result<()> {
        verify_bytes(b"bound bytes", &hash(b"bound bytes"))?;
        assert!(verify_bytes(b"changed", &hash(b"bound bytes")).is_err());
        assert_eq!(projected(127, 1)?, 128);
        assert!(matches!(
            projected(127, 2),
            Err(NativeBankGenerateError::ContextBudget {
                actual: 129,
                maximum: 128
            })
        ));
        assert!(matches!(
            projected(usize::MAX, 1),
            Err(NativeBankGenerateError::Arithmetic)
        ));
        Ok(())
    }
}
