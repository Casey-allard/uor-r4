//! Checked packed-artifact source realization without offline training weights.
//! Shared integer execution preserves the allocating occurrence/action traces.
//! Trusted receipt loading is separate from offline float-source equivalence.
use crate::{
    geometric_context_q4::{ContextQ4Config, NativeContextQ4},
    geometric_cue_carrier::{CueAngularQ4, CueCarrierTrace, NativeCueCarrier},
    geometric_no_read::{NativeGeometricNoRead, NoReadConfig},
    geometric_occurrence_read::{
        NativeOccurrenceReader, OccurrenceBankSegment, OccurrenceComponents, OccurrenceRead,
        PreparedOccurrenceContext, QuerySnapshot, SelectedRecordFrame,
    },
    geometric_potential::{AddressLane, NativePotentialTables},
    geometric_potential_q4::{NativePotentialQ4, PotentialQ4Config},
    geometric_prefix_transport::{NativePrefixTransport, PrefixAngularQ4, PrefixTransportTrace},
    geometric_read_feedback::{
        FeedbackInputMode, FeedbackTrace, NativeReadFeedback, QuerySnapshotReport,
    },
    geometric_source_actions::{
        ActionHeadScores, ActionTrace, NativeSourceActions, SourceActionBinding,
    },
    geometric_source_emission_view::{SourceEmissionCompiler, SourceEmissionView},
    geometric_source_end_transport::{
        NativeSourceEndTransport, SourceEndAngularQ4, SourceEndTransportTrace,
    },
    h4_tables::{H4Code, HistoricalH4Tables, ROOT_COUNT},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    io::Read,
    path::Path,
};
pub const SCHEMA: &str = "uor-r4.geometric-source-realizer/1";
pub const POLICY:&str="parent-free-source-view-Copy-Period-Stop;current-packed-H4-context+potential+Stop+independent-Period;sum-head-logits-before-one-native-normalization;token-alias-mass-sum;no-authored-phase-or-length;128-sequence/1";
const REALIZER_SURROGATE:&str="native-joint-Q31-token-probability-forward;softmax-summed-head-Q24-score-adjoint;quarter-grid-coefficient-STE+frozen-coefficient-context-credit;subtraction-first-zero-forward;target-loss-only;prepared-source-immutable-until-drop/1";
const CONSUMER_SURROGATE:&str="native-Q31-normalized-forward;softmax-score-adjoint;hard-quarter-source-STE;coefficient-and-frozen-context-input-credit;whole-parent-NoRead-fallback;target-loss-only/1";
const ALGEBRA: &[u8] = include_bytes!("../fixtures/historical-h4-tables-v1.bin");
pub const CANONICAL_EXP_SHA256: &str =
    "79485d6e63cc28f5e01d98c5d73abe021db33fa7fef368142ae6592d06b4817f";
pub const CANONICAL_EXP_BYTES: usize = 32776;
#[derive(Debug)]
pub enum SourceRuntimeError {
    Invalid(String),
    Io(std::io::Error),
    Json(serde_json::Error),
}
impl fmt::Display for SourceRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native source runtime: {self:?}")
    }
}
impl std::error::Error for SourceRuntimeError {}
impl From<std::io::Error> for SourceRuntimeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for SourceRuntimeError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
pub type SourceRuntimeResult<T> = std::result::Result<T, SourceRuntimeError>;
use SourceRuntimeResult as Result;
pub(crate) fn invalid(message: impl Into<String>) -> SourceRuntimeError {
    SourceRuntimeError::Invalid(message.into())
}
pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccurrenceIdentity {
    pub record: u64,
    pub commit: u64,
    pub token_offset: u32,
    pub token_id: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadTrace {
    pub scores_q24: Vec<i64>,
    pub weights_q31: Vec<u64>,
    pub no_read_q24: i64,
    pub no_read_weight_q31: u64,
    pub total_weight_q31: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OccurrenceTrace {
    pub occurrences: Vec<OccurrenceIdentity>,
    pub heads: Vec<HeadTrace>,
    pub sequence_tokens: usize,
    pub context_coefficient_reads: usize,
    pub potential_table_reads: usize,
    pub no_read_table_reads: usize,
    pub geometry_relative_reads: usize,
    pub logical_owned_bytes: usize,
}

/// The inner token offsets belong to the explicitly derived emission view.
/// Original source bytes, token identities and byte-span provenance remain in
/// `emission_view`; they must not be confused with those view offsets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceEmissionTrace {
    pub record: u64,
    pub commit: u64,
    pub scope: Vec<u8>,
    pub entity: Vec<u32>,
    pub relation: u32,
    pub source_store_view: u32,
    pub emission_view: SourceEmissionView,
    pub view_kernel_trace: OccurrenceTrace,
}
pub fn source_view_trace(
    frame: SelectedRecordFrame<'_>,
    view: &SourceEmissionView,
    kernel: OccurrenceTrace,
) -> SourceEmissionTrace {
    SourceEmissionTrace {
        record: frame.identity.record,
        commit: frame.identity.commit,
        scope: frame.metadata.scope.to_vec(),
        entity: frame.metadata.entity.to_vec(),
        relation: frame.metadata.relation,
        source_store_view: frame.metadata.view,
        emission_view: view.clone(),
        view_kernel_trace: kernel,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ObservedCode {
    pub root: u8,
    pub radius_bin: u8,
    pub present: bool,
}
impl From<AddressLane> for ObservedCode {
    fn from(code: AddressLane) -> Self {
        Self {
            root: code.root(),
            radius_bin: code.radius_bin(),
            present: code.present(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SerializableContextReplay {
    pub tokens: Vec<u32>,
    pub heads: usize,
    pub lanes_per_head: usize,
    pub states: Vec<Vec<u8>>,
    pub actions: Vec<Vec<u8>>,
    pub raw_roots: Vec<u8>,
    pub categories: Vec<u8>,
    pub codes: Vec<ObservedCode>,
    pub coefficient_reads: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RealizerTrace {
    pub policy: &'static str,
    pub source: SourceEmissionTrace,
    /// Original causal replay shared by source scoring and controller observation.
    /// A dependent update is recorded separately, never disguised as token replay.
    pub period_context: SerializableContextReplay,
    pub period_q24: Vec<i64>,
    pub actions: ActionTrace,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DependentReadTrace {
    pub policy: &'static str,
    pub stage1: RealizerTrace,
    pub stage2: RealizerTrace,
    pub feedback: FeedbackTrace,
    /// Authoritative input for ALL stage2 Copy/Stop/Period scoring.
    pub stage2_controller_snapshot: QuerySnapshotReport,
    pub stage2_context_replay_policy: &'static str,
    pub logical_prepared_bytes: usize,
    pub original_context_replays: usize,
    pub scoring_stages: usize,
}

/// A declared finite intervention, never a target-selected occurrence or state.
/// H4Code construction rejects padded/noncanonical action IDs.
#[derive(Clone, Debug)]
pub struct NativeLaneActionRequest {
    pub lane: usize,
    pub actions: Vec<H4Code>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeActionTrace {
    pub action: u8,
    pub action_vector: Vec<u8>,
    /// Authoritative updated input to every final Copy/Period/Stop head.
    pub controller_snapshot: QuerySnapshotReport,
    pub final_actions: ActionTrace,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeLaneActionTraces {
    pub lane: usize,
    pub actions: Vec<NativeActionTrace>,
}

/// Exact native distributions under declared one-lane replacements. No answer
/// label, loss, utility or correctness decision enters this numerical API.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeActionCounterfactualTrace {
    pub policy: &'static str,
    pub factual: DependentReadTrace,
    pub lanes: Vec<NativeLaneActionTraces>,
    pub original_context_replays: usize,
    pub value_producer_evaluations: usize,
    pub scoring_stages: usize,
}
const MAX_ACTION_COUNTERFACTUALS: usize = 8 * ROOT_COUNT;

pub fn read_occurrence(
    components: OccurrenceComponents<'_>,
    frame: SelectedRecordFrame<'_>,
    query: &[u32],
    prefix: &[u32],
) -> Result<OccurrenceTrace> {
    let mut reader = NativeOccurrenceReader::new(components).map_err(|e| invalid(e.to_string()))?;
    let output = reader
        .read(frame, query, prefix)
        .map_err(|e| invalid(e.to_string()))?;
    trace_occurrence(output)
}
fn trace_occurrence(output: OccurrenceRead<'_>) -> Result<OccurrenceTrace> {
    let mut heads = Vec::with_capacity(output.heads.len());
    for h in 0..output.heads.len() {
        let head = output
            .head(h)
            .ok_or_else(|| invalid("consumer head absent"))?;
        heads.push(HeadTrace {
            scores_q24: head.potential_q24.to_vec(),
            weights_q31: head.occurrence_weights_q31.to_vec(),
            no_read_q24: head.no_read_q24,
            no_read_weight_q31: head.no_read_weight_q31,
            total_weight_q31: head.total_weight_q31,
        });
    }
    Ok(OccurrenceTrace {
        occurrences: output
            .occurrences
            .iter()
            .map(|x| OccurrenceIdentity {
                record: x.source.record,
                commit: x.source.commit,
                token_offset: x.token_offset,
                token_id: x.token_id,
            })
            .collect(),
        heads,
        sequence_tokens: output.stats.sequence_tokens,
        context_coefficient_reads: output.stats.context_coefficient_reads,
        potential_table_reads: output.stats.potential_table_reads,
        no_read_table_reads: output.stats.no_read_table_reads,
        geometry_relative_reads: output.stats.geometry_relative_reads,
        logical_owned_bytes: output.stats.logical_owned_bytes,
    })
}
/// Ordered caller-owned history. Source projection is segment-wise; Context
/// is not a source and role is provenance only in the current numeric model.
#[derive(Clone, Copy, Debug)]
pub enum SourceBankSegment<'a> {
    Source {
        frame: SelectedRecordFrame<'a>,
        view: &'a SourceEmissionView,
        event: u64,
    },
    Context {
        token_ids: &'a [u32],
        event: u64,
        role: u32,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum BankSegmentTrace {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: Vec<u8>,
        entity: Vec<u32>,
        relation: u32,
        store_view: u32,
        view: SourceEmissionView,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BankCandidateTrace {
    pub bank_index: usize,
    pub context_position: usize,
    pub event: u64,
    pub segment_index: usize,
    pub occurrence: OccurrenceIdentity,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BankRealizerTrace {
    pub policy: &'static str,
    /// Derived numerical replay digest, not the complete original lexical
    /// identity. `segments` retains original IDs/bytes, tokenizer and view policy.
    pub bank_binding_sha256: String,
    pub segments: Vec<BankSegmentTrace>,
    pub candidates: Vec<BankCandidateTrace>,
    pub context: SerializableContextReplay,
    pub heads: Vec<HeadTrace>,
    pub period_q24: Vec<i64>,
    pub actions: ActionTrace,
    pub logical_prepared_bytes: usize,
}

/// Added carrier evidence/cost is separate so a zero term preserves the entire
/// historical BankRealizerTrace, including its legacy replay accounting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CueBankRealizerTrace {
    pub bank: BankRealizerTrace,
    pub carrier: CueCarrierTrace,
}

/// Prefix evidence is separate from the unchanged cue/bank baseline traces.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PrefixBankRealizerTrace {
    pub cue_bank: CueBankRealizerTrace,
    pub prefix: PrefixTransportTrace,
    /// Extra combined cue+prefix score buffer; Vec descriptors/allocator metadata
    /// are separate from this logical payload count.
    pub combined_score_payload_bytes: usize,
    pub combined_score_vec_containers: usize,
}

/// Nested prefix bank remains the factual baseline. `actions` is the
/// authoritative terminal-adjusted global action/alias normalization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceEndBankRealizerTrace {
    pub prefix_bank: PrefixBankRealizerTrace,
    pub source_end: SourceEndTransportTrace,
    pub actions: ActionTrace,
}

pub struct RealizerExecution<'a> {
    pub context: &'a NativeContextQ4,
    pub potential_tables: &'a NativePotentialTables,
    pub no_read: &'a NativeGeometricNoRead,
    pub geometry: &'a HistoricalH4Tables,
    pub exp: &'a [u32],
    pub period: &'a NativeGeometricNoRead,
    pub binding: &'a SourceActionBinding,
}
impl<'a> RealizerExecution<'a> {
    /// One bounded causal replay and one bank-wide Copy/Period/Stop reduction.
    /// No answer-based admission or per-record probability normalization.
    pub fn read_bank(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
    ) -> Result<BankRealizerTrace> {
        Ok(self.read_bank_impl(segments, query, prefix, None, None)?.0)
    }
    pub fn read_bank_with_cue_carrier(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        parent: &NativeArtifactBinding,
        carrier: &NativeCueCarrier<'_>,
    ) -> Result<CueBankRealizerTrace> {
        carrier.validate_execution(parent, self.context, self.geometry)?;
        if parent.identity.tokenizer_sha256 != self.binding.tokenizer_sha256() {
            return Err(invalid("cue carrier tokenizer parent differs"));
        }
        let (bank, carrier, _) =
            self.read_bank_impl(segments, query, prefix, Some(carrier), None)?;
        Ok(CueBankRealizerTrace {
            bank,
            carrier: carrier.ok_or_else(|| invalid("cue carrier trace absent"))?,
        })
    }
    pub fn read_bank_with_prefix_transport(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        parent: &NativeArtifactBinding,
        cue: &NativeCueCarrier<'_>,
        transport: &NativePrefixTransport<'_>,
    ) -> Result<PrefixBankRealizerTrace> {
        cue.validate_execution(parent, self.context, self.geometry)?;
        transport.validate_execution(parent, self.context, self.geometry, cue.metadata())?;
        if parent.identity.tokenizer_sha256 != self.binding.tokenizer_sha256() {
            return Err(invalid("prefix transport tokenizer parent differs"));
        }
        let (bank, carrier, ordered) =
            self.read_bank_impl(segments, query, prefix, Some(cue), Some(transport))?;
        let combined_score_payload_bytes = bank.heads.len() * bank.candidates.len() * 8;
        let combined_score_vec_containers = 1 + bank.heads.len();
        Ok(PrefixBankRealizerTrace {
            cue_bank: CueBankRealizerTrace {
                bank,
                carrier: carrier.ok_or_else(|| invalid("cue trace absent"))?,
            },
            combined_score_payload_bytes,
            combined_score_vec_containers,
            prefix: ordered.ok_or_else(|| invalid("prefix trace absent"))?,
        })
    }
    pub fn read_bank_with_source_end_transport(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        parent: &NativeArtifactBinding,
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        end: &NativeSourceEndTransport<'_>,
    ) -> Result<SourceEndBankRealizerTrace> {
        end.validate_execution(
            parent,
            self.context,
            self.geometry,
            cue.metadata(),
            prefix.metadata(),
        )?;
        let prefix_bank = self.read_bank_with_prefix_transport(
            segments,
            query,
            actual_prefix,
            parent,
            cue,
            prefix,
        )?;
        let bank = &prefix_bank.cue_bank.bank;
        let source_end = end.prepare(&prefix_bank.prefix, &bank.heads)?;
        let mut periods = bank.period_q24.clone();
        let mut stops = bank.heads.iter().map(|h| h.no_read_q24).collect::<Vec<_>>();
        for h in 0..bank.heads.len() {
            periods[h] = periods[h]
                .checked_add(source_end.period_q24[h])
                .ok_or_else(|| invalid("source-end Period total overflow"))?;
            stops[h] = stops[h]
                .checked_add(source_end.stop_q24[h])
                .ok_or_else(|| invalid("source-end Stop total overflow"))?;
        }
        let scores = bank
            .heads
            .iter()
            .enumerate()
            .map(|(h, head)| ActionHeadScores {
                copy_q24: &head.scores_q24,
                period_q24: periods[h],
                stop_q24: stops[h],
            })
            .collect::<Vec<_>>();
        let ids = bank
            .candidates
            .iter()
            .map(|c| c.occurrence.token_id)
            .collect::<Vec<_>>();
        let actions = NativeSourceActions::new(self.binding.clone(), bank.heads.len(), self.exp)?
            .reduce(&ids, &scores)?;
        Ok(SourceEndBankRealizerTrace {
            prefix_bank,
            source_end,
            actions,
        })
    }

    fn read_bank_impl(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        carrier: Option<&NativeCueCarrier<'_>>,
        ordered: Option<&NativePrefixTransport<'_>>,
    ) -> Result<(
        BankRealizerTrace,
        Option<CueCarrierTrace>,
        Option<PrefixTransportTrace>,
    )> {
        if self.period.config() != self.no_read.config()
            || self.binding.vocab_size() != self.context.config().vocab_size
        {
            return Err(invalid("bank context/controller/vocabulary differs"));
        }
        self.binding.validate_tokens(query)?;
        self.binding.validate_tokens(prefix)?;
        if segments.len() > crate::geometric_occurrence_read::MAX_SEQUENCE {
            return Err(invalid("bank segment cap exceeds128"));
        }
        let mut projected = query.len().checked_add(prefix.len()).unwrap_or(usize::MAX);
        for segment in segments {
            let n = match segment {
                SourceBankSegment::Source { view, .. } => view.emitted_token_ids().len(),
                SourceBankSegment::Context { token_ids, .. } => token_ids.len(),
            };
            projected = projected.checked_add(n).unwrap_or(usize::MAX);
        }
        if projected > crate::geometric_occurrence_read::MAX_SEQUENCE {
            return Err(invalid(format!(
                "bank projected context {projected} exceeds128"
            )));
        }
        let mut derived = Vec::with_capacity(segments.len());
        let mut inventory = Vec::with_capacity(segments.len());
        let mut candidate_segments = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            match segment {
                SourceBankSegment::Source { frame, view, event } => {
                    if view.policy() != crate::geometric_source_emission_view::POLICY
                        || view.tokenizer_sha256() != self.binding.tokenizer_sha256()
                    {
                        return Err(invalid("bank source tokenizer/view policy differs"));
                    }
                    derived.push(OccurrenceBankSegment::Source {
                        frame: view.derived_frame(*frame)?,
                        event: *event,
                    });
                    candidate_segments
                        .extend(std::iter::repeat_n(index, view.emitted_token_ids().len()));
                    inventory.push(BankSegmentTrace::Source {
                        event: *event,
                        record: frame.identity.record,
                        commit: frame.identity.commit,
                        scope: frame.metadata.scope.to_vec(),
                        entity: frame.metadata.entity.to_vec(),
                        relation: frame.metadata.relation,
                        store_view: frame.metadata.view,
                        view: (*view).clone(),
                    });
                }
                SourceBankSegment::Context {
                    token_ids,
                    event,
                    role,
                } => {
                    derived.push(OccurrenceBankSegment::Context {
                        token_ids,
                        event: *event,
                        role: *role,
                    });
                    inventory.push(BankSegmentTrace::Context {
                        event: *event,
                        role: *role,
                        token_ids: token_ids.to_vec(),
                    });
                }
            }
        }
        let mut reader = NativeOccurrenceReader::new(OccurrenceComponents {
            context: self.context.native(),
            potential: self.potential_tables,
            no_read: self.no_read,
            geometry: self.geometry,
            exp_q31: self.exp,
        })
        .map_err(|e| invalid(e.to_string()))?;
        let prepared = reader
            .prepare_bank(&derived, query, prefix)
            .map_err(|e| invalid(e.to_string()))?;
        let snapshot = prepared
            .original_snapshot()
            .map_err(|e| invalid(e.to_string()))?;
        let c = self.context.config();
        let lanes = c.heads * c.lanes_per_head;
        let mut context = SerializableContextReplay {
            tokens: prepared.tokens().to_vec(),
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            states: Vec::new(),
            actions: Vec::new(),
            raw_roots: Vec::new(),
            categories: Vec::new(),
            codes: Vec::new(),
            coefficient_reads: 0,
        };
        for step in prepared.steps() {
            context
                .states
                .push(step.states[..lanes].iter().map(|s| s.index()).collect());
            context
                .actions
                .push(step.actions[..lanes].iter().map(|s| s.index()).collect());
            context
                .raw_roots
                .extend(step.readout_roots[..lanes].iter().map(|s| s.index()));
            context
                .categories
                .extend_from_slice(&step.categories[..lanes]);
            context
                .codes
                .extend(step.output[..lanes].iter().copied().map(ObservedCode::from));
            context.coefficient_reads += step.coefficient_reads;
        }
        let carrier = carrier
            .map(|carrier| carrier.prepare(&derived, query, &candidate_segments))
            .transpose()?;
        let ordered = ordered
            .map(|transport| transport.prepare(&derived, prefix, &candidate_segments))
            .transpose()?;
        let combined = if let Some(ordered) = &ordered {
            let cue = carrier
                .as_ref()
                .ok_or_else(|| invalid("prefix transport requires frozen cue"))?;
            if ordered.copy_q24.len() != cue.copy_q24.len() {
                return Err(invalid("prefix/cue head count differs"));
            }
            let mut scores = ordered.copy_q24.clone();
            for (head, cue_head) in scores.iter_mut().zip(&cue.copy_q24) {
                if head.len() != cue_head.len() {
                    return Err(invalid("prefix/cue candidate count differs"));
                }
                for (score, cue_score) in head.iter_mut().zip(cue_head) {
                    *score = score
                        .checked_add(*cue_score)
                        .ok_or_else(|| invalid("prefix/cue sum overflow"))?;
                }
            }
            Some(scores)
        } else {
            None
        };
        let output = match (&combined, &carrier) {
            (Some(scores), _) => {
                reader.score_bank_with_copy_adjustments(&prepared, &snapshot, scores)
            }
            (None, Some(carrier)) => {
                reader.score_bank_with_copy_adjustments(&prepared, &snapshot, &carrier.copy_q24)
            }
            (None, None) => reader.score_bank(&prepared, &snapshot),
        }
        .map_err(|e| invalid(e.to_string()))?;
        let mut heads = Vec::with_capacity(output.head_count());
        for h in 0..output.head_count() {
            let head = output.head(h).ok_or_else(|| invalid("bank head absent"))?;
            heads.push(HeadTrace {
                scores_q24: head.potential_q24.to_vec(),
                weights_q31: head.occurrence_weights_q31.to_vec(),
                no_read_q24: head.no_read_q24,
                no_read_weight_q31: head.no_read_weight_q31,
                total_weight_q31: head.total_weight_q31,
            });
        }
        let period = self
            .period
            .score(
                snapshot.last_token() as usize,
                snapshot.states(),
                snapshot.codes(),
                None,
            )
            .map_err(|e| invalid(e.to_string()))?;
        let head_scores = heads
            .iter()
            .enumerate()
            .map(|(h, s)| ActionHeadScores {
                copy_q24: &s.scores_q24,
                period_q24: period[h],
                stop_q24: s.no_read_q24,
            })
            .collect::<Vec<_>>();
        let ids = output
            .occurrences()
            .iter()
            .map(|o| o.token_id)
            .collect::<Vec<_>>();
        let actions = NativeSourceActions::new(self.binding.clone(), c.heads, self.exp)?
            .reduce(&ids, &head_scores)?;
        let candidates = output
            .occurrences()
            .iter()
            .enumerate()
            .map(|(i, o)| BankCandidateTrace {
                bank_index: i,
                context_position: usize::from(prepared.candidate_positions()[i]),
                event: prepared.candidate_events()[i],
                segment_index: candidate_segments[i],
                occurrence: OccurrenceIdentity {
                    record: o.source.record,
                    commit: o.source.commit,
                    token_offset: o.token_offset,
                    token_id: o.token_id,
                },
            })
            .collect();
        Ok((BankRealizerTrace {policy:"causal-bank-segment-emission-view;context/query/ownprefix-noncandidates;one-global-Copy-Period-Stop;roles-provenance-only;128-context/1",bank_binding_sha256:hex::encode(prepared.binding()),segments:inventory,candidates,context,heads,period_q24:period[..c.heads].to_vec(),actions,logical_prepared_bytes:prepared.logical_prepared_bytes()},carrier,ordered))
    }

    pub fn read_source_view(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<SourceEmissionTrace> {
        if view.policy() != crate::geometric_source_emission_view::POLICY
            || view.tokenizer_sha256() != self.binding.tokenizer_sha256()
        {
            return Err(invalid(
                "source emission view policy/tokenizer differs from artifact",
            ));
        }
        self.binding.validate_tokens(query)?;
        self.binding.validate_tokens(prefix)?;
        let kernel = read_occurrence(
            OccurrenceComponents {
                context: self.context.native(),
                potential: self.potential_tables,
                no_read: self.no_read,
                geometry: self.geometry,
                exp_q31: self.exp,
            },
            view.derived_frame(frame)?,
            query,
            prefix,
        )?;
        Ok(source_view_trace(frame, view, kernel))
    }
    fn prepare_view(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<(
        NativeOccurrenceReader<'a>,
        PreparedOccurrenceContext<'a>,
        SerializableContextReplay,
    )> {
        if self.period.config() != self.no_read.config()
            || self.binding.vocab_size() != self.context.config().vocab_size
            || view.policy() != crate::geometric_source_emission_view::POLICY
            || view.tokenizer_sha256() != self.binding.tokenizer_sha256()
        {
            return Err(invalid(
                "realizer context/controller/view/tokenizer differs",
            ));
        }
        self.binding.validate_tokens(query)?;
        self.binding.validate_tokens(prefix)?;
        let reader = NativeOccurrenceReader::new(OccurrenceComponents {
            context: self.context.native(),
            potential: self.potential_tables,
            no_read: self.no_read,
            geometry: self.geometry,
            exp_q31: self.exp,
        })
        .map_err(|e| invalid(e.to_string()))?;
        let prepared = reader
            .prepare(view.derived_frame(frame)?, query, prefix)
            .map_err(|e| invalid(e.to_string()))?;
        let c = self.context.config();
        let lanes = c.heads * c.lanes_per_head;
        let mut replay = SerializableContextReplay {
            tokens: prepared.tokens().to_vec(),
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            states: Vec::new(),
            actions: Vec::new(),
            raw_roots: Vec::new(),
            categories: Vec::new(),
            codes: Vec::new(),
            coefficient_reads: prepared.context_coefficient_reads(),
        };
        for step in prepared.steps() {
            replay
                .states
                .push(step.states[..lanes].iter().map(|r| r.index()).collect());
            replay
                .actions
                .push(step.actions[..lanes].iter().map(|r| r.index()).collect());
            replay
                .raw_roots
                .extend(step.readout_roots[..lanes].iter().map(|r| r.index()));
            replay
                .categories
                .extend_from_slice(&step.categories[..lanes]);
            replay
                .codes
                .extend(step.output[..lanes].iter().copied().map(ObservedCode::from));
        }
        Ok((reader, prepared, replay))
    }
    fn score_view(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        reader: &mut NativeOccurrenceReader<'a>,
        prepared: &PreparedOccurrenceContext<'a>,
        snapshot: &QuerySnapshot<'_, 'a>,
        replay: &SerializableContextReplay,
    ) -> Result<RealizerTrace> {
        let source = source_view_trace(
            frame,
            view,
            trace_occurrence(
                reader
                    .score(prepared, snapshot)
                    .map_err(|e| invalid(e.to_string()))?,
            )?,
        );
        let period = self
            .period
            .score(
                snapshot.last_token() as usize,
                snapshot.states(),
                snapshot.codes(),
                None,
            )
            .map_err(|e| invalid(e.to_string()))?;
        // Stop in source is scored from exactly this same checked snapshot.
        let heads = self.context.config().heads;
        let head_scores = source
            .view_kernel_trace
            .heads
            .iter()
            .enumerate()
            .map(|(head, s)| ActionHeadScores {
                copy_q24: &s.scores_q24,
                period_q24: period[head],
                stop_q24: s.no_read_q24,
            })
            .collect::<Vec<_>>();
        let actions = NativeSourceActions::new(self.binding.clone(), heads, self.exp)?
            .reduce(view.emitted_token_ids(), &head_scores)?;
        Ok(RealizerTrace {
            policy: POLICY,
            source,
            period_context: replay.clone(),
            period_q24: period[..heads].to_vec(),
            actions,
        })
    }
    pub fn read(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
    ) -> Result<RealizerTrace> {
        let (mut reader, prepared, replay) = self.prepare_view(frame, view, query, prefix)?;
        let snapshot = prepared
            .original_snapshot()
            .map_err(|e| invalid(e.to_string()))?;
        self.score_view(frame, view, &mut reader, &prepared, &snapshot, &replay)
    }
    pub fn read_dependent(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        parent: &NativeArtifactBinding,
        feedback: &NativeReadFeedback,
        mode: FeedbackInputMode,
    ) -> Result<DependentReadTrace> {
        Ok(self
            .read_dependent_action_traces_for(
                frame,
                view,
                query,
                prefix,
                parent,
                feedback,
                mode,
                &[],
            )?
            .factual)
    }

    /// All120 canonical actions for each configured lane, ordered0..119.
    /// Factual source replay, provisional occurrence and producer are evaluated
    /// once; other lane actions always remain the actual factual actions.
    pub fn read_dependent_action_traces(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        parent: &NativeArtifactBinding,
        feedback: &NativeReadFeedback,
        mode: FeedbackInputMode,
    ) -> Result<NativeActionCounterfactualTrace> {
        let lanes = self.context.config().heads * self.context.config().lanes_per_head;
        if lanes > 8 {
            return Err(invalid("native action lane bound exceeded"));
        }
        let actions = (0..ROOT_COUNT)
            .map(|action| H4Code::try_from(action as u8).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        let requests = (0..lanes)
            .map(|lane| NativeLaneActionRequest {
                lane,
                actions: actions.clone(),
            })
            .collect::<Vec<_>>();
        self.read_dependent_action_traces_for(
            frame, view, query, prefix, parent, feedback, mode, &requests,
        )
    }

    /// Declared subset of finite actions using the same prepared core. This
    /// exposes no public construction of an arbitrary QuerySnapshot.
    pub fn read_dependent_action_traces_for(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        parent: &NativeArtifactBinding,
        feedback: &NativeReadFeedback,
        mode: FeedbackInputMode,
        requests: &[NativeLaneActionRequest],
    ) -> Result<NativeActionCounterfactualTrace> {
        let lane_count = self.context.config().heads * self.context.config().lanes_per_head;
        if feedback.metadata().context != self.context.config()
            || &feedback.metadata().parent_artifact != parent
        {
            return Err(invalid("counterfactual feedback context/parent differs"));
        }
        let mut used_lanes = BTreeSet::new();
        let mut candidates = 0usize;
        for request in requests {
            if request.lane >= lane_count || !used_lanes.insert(request.lane) {
                return Err(invalid("counterfactual lane is invalid or duplicated"));
            }
            let unique = request
                .actions
                .iter()
                .map(|action| action.index())
                .collect::<BTreeSet<_>>();
            if request.actions.is_empty() || unique.len() != request.actions.len() {
                return Err(invalid("counterfactual actions empty or duplicated"));
            }
            candidates = candidates
                .checked_add(request.actions.len())
                .ok_or_else(|| invalid("counterfactual count overflow"))?;
        }
        if lane_count > 8 || candidates > MAX_ACTION_COUNTERFACTUALS {
            return Err(invalid("counterfactual native960 bound exceeded"));
        }
        let (mut reader, prepared, replay) = self.prepare_view(frame, view, query, prefix)?;
        let snapshot = prepared
            .original_snapshot()
            .map_err(|e| invalid(e.to_string()))?;
        let stage1 = self.score_view(frame, view, &mut reader, &prepared, &snapshot, &replay)?;
        let (updated, feedback_trace) =
            feedback.apply(parent, &prepared, &snapshot, &stage1.actions, mode)?;
        let stage2 = self.score_view(frame, view, &mut reader, &prepared, &updated, &replay)?;
        let factual=DependentReadTrace {
            policy: crate::geometric_read_feedback::POLICY,
            stage1,
            stage2,
            stage2_controller_snapshot: feedback_trace.after.clone(),
            stage2_context_replay_policy: "stage2.period_context is original token replay only; stage2_controller_snapshot is authoritative updated Copy/Stop/Period input",
            feedback: feedback_trace,
            logical_prepared_bytes: prepared.logical_prepared_bytes(),
            original_context_replays: 1,
            scoring_stages: 2,
        };
        let factual_actions = factual
            .feedback
            .actions
            .iter()
            .map(|action| H4Code::try_from(*action).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        if factual_actions.len() != lane_count {
            return Err(invalid("factual action-vector shape differs"));
        }
        let mut lanes = Vec::with_capacity(requests.len());
        for request in requests {
            let mut actions = Vec::with_capacity(request.actions.len());
            for action in &request.actions {
                let mut vector = factual_actions.clone();
                vector[request.lane] = *action;
                // Compose every declared action against the ORIGINAL state, not
                // the already-updated factual state; source keys remain immutable.
                let candidate = prepared
                    .apply_actions(&snapshot, &vector)
                    .map_err(|e| invalid(e.to_string()))?;
                let scored =
                    self.score_view(frame, view, &mut reader, &prepared, &candidate, &replay)?;
                actions.push(NativeActionTrace {
                    action: action.index(),
                    action_vector: vector.iter().map(|a| a.index()).collect(),
                    controller_snapshot: QuerySnapshotReport::from(&candidate),
                    final_actions: scored.actions,
                });
            }
            lanes.push(NativeLaneActionTraces {
                lane: request.lane,
                actions,
            });
        }
        Ok(NativeActionCounterfactualTrace{policy:"target-free-one-lane-H4-action-counterfactual;shared-factual-source-keys-original-query-producer;observation-only-right-composition;all-final-heads-and-alias-masses/1",factual,lanes,original_context_replays:1,value_producer_evaluations:1,scoring_stages:2+candidates})
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub tokenizer_sha256: String,
    pub parent_checkpoint_manifest_sha256: String,
    pub parent_model_sha256: String,
    pub parent_config_sha256: String,
}
impl ArtifactIdentity {
    pub(crate) fn validate(&self) -> Result<()> {
        for digest in [
            &self.tokenizer_sha256,
            &self.parent_checkpoint_manifest_sha256,
            &self.parent_model_sha256,
            &self.parent_config_sha256,
        ] {
            require_digest(digest)?;
        }
        Ok(())
    }
}
/// The caller obtains this receipt from trusted deployment/selection evidence,
/// never from the untrusted directory that this loader is about to admit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeArtifactBinding {
    pub metadata_sha256: String,
    pub identity: ArtifactIdentity,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterIdentity {
    shape: Vec<usize>,
    f32_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RealizerMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    source_view_policy: String,
    action_policy: String,
    identity: ArtifactIdentity,
    period: NoReadConfig,
    source_parameters: BTreeMap<String, ParameterIdentity>,
    files: BTreeMap<String, String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConsumerMetadata {
    schema: String,
    policy: String,
    surrogate: String,
    identity: ArtifactIdentity,
    context: ContextQ4Config,
    potential: PotentialQ4Config,
    no_read: NoReadConfig,
    algebra_sha256: String,
    files: BTreeMap<String, String>,
    source_view_policy: Option<String>,
}
const NATIVE_FILES: [&str; 7] = [
    "tokenizer.json",
    "period-q4.bin",
    "consumer/metadata.json",
    "consumer/context-q4.bin",
    "consumer/potential-q4.bin",
    "consumer/no-read-q4.bin",
    "consumer/exp-q31.bin",
];
const CONSUMER_FILES: [&str; 4] = [
    "context-q4.bin",
    "potential-q4.bin",
    "no-read-q4.bin",
    "exp-q31.bin",
];
const MAX_METADATA_BYTES: usize = 1 << 20;
const MAX_TOKENIZER_BYTES: usize = 8 << 20;
fn require_digest(digest: &str) -> Result<()> {
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("artifact digest format differs"));
    }
    Ok(())
}
fn read_bounded(path: &Path, maximum: usize) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > maximum as u64 {
        return Err(invalid("artifact file type or bounded byte length differs"));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(invalid("artifact grew beyond admitted byte bound"));
    }
    Ok(bytes)
}
fn read_exact(path: &Path, length: usize) -> Result<Vec<u8>> {
    let bytes = read_bounded(path, length)?;
    if bytes.len() != length {
        return Err(invalid("artifact exact byte length differs"));
    }
    Ok(bytes)
}
fn require_inventory(path: &Path) -> Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        return Err(invalid("native artifact root must be a real directory"));
    }
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| invalid("artifact path is not UTF8"))?;
        let ty = entry.file_type()?;
        if name == "consumer" && ty.is_dir() {
            for child in fs::read_dir(entry.path())? {
                let child = child?;
                let name = child
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("artifact path is not UTF8"))?;
                if !child.file_type()?.is_file() {
                    return Err(invalid("consumer artifact contains directory or symlink"));
                }
                actual.insert(format!("consumer/{name}"));
            }
        } else if ty.is_file() {
            actual.insert(name);
        } else {
            return Err(invalid("artifact contains unexpected directory or symlink"));
        }
    }
    let expected = NATIVE_FILES
        .into_iter()
        .chain(std::iter::once("metadata.json"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(invalid("native artifact exact inventory differs"));
    }
    Ok(())
}
fn require_hashes(files: &BTreeMap<String, String>, names: &[&str]) -> Result<()> {
    if files.keys().map(String::as_str).collect::<BTreeSet<_>>() != names.iter().copied().collect()
    {
        return Err(invalid("declared artifact inventory differs"));
    }
    for digest in files.values() {
        require_digest(digest)?;
    }
    Ok(())
}
fn check_file(files: &BTreeMap<String, String>, name: &str, bytes: &[u8]) -> Result<()> {
    if files.get(name) != Some(&sha256_bytes(bytes)) {
        return Err(invalid(format!("native artifact digest differs: {name}")));
    }
    Ok(())
}
pub(crate) fn canonical_exp(bytes: &[u8]) -> Result<Vec<u32>> {
    if bytes.len() != CANONICAL_EXP_BYTES || sha256_bytes(bytes) != CANONICAL_EXP_SHA256 {
        return Err(invalid(
            "native artifact canonical exponential bytes differ",
        ));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}
fn source_parameter_provenance(metadata: &RealizerMetadata, c: &ConsumerMetadata) -> Result<()> {
    let mut shapes = BTreeMap::new();
    for (name, shape) in c
        .context
        .coefficient_shapes()
        .map_err(|e| invalid(e.to_string()))?
    {
        shapes.insert(format!("consumer.context.{name}"), shape);
    }
    for (name, shape) in c
        .potential
        .coefficient_shapes()
        .map_err(|e| invalid(e.to_string()))?
    {
        shapes.insert(format!("consumer.potential.{name}"), shape);
    }
    shapes.insert(
        "consumer.no_read.coefficients".into(),
        vec![c.no_read.heads, c.no_read.coefficients_per_head()],
    );
    shapes.insert(
        "period.coefficients".into(),
        vec![
            metadata.period.heads,
            metadata.period.coefficients_per_head(),
        ],
    );
    if shapes.keys().ne(metadata.source_parameters.keys()) {
        return Err(invalid(
            "native artifact source-provenance parameter inventory differs",
        ));
    }
    for (name, shape) in shapes {
        let saved = &metadata.source_parameters[&name];
        require_digest(&saved.f32_sha256)?;
        if saved.shape != shape {
            return Err(invalid("native artifact source-provenance shape differs"));
        }
    }
    Ok(())
}
pub struct NativeSourceRealizer {
    context: NativeContextQ4,
    potential: NativePotentialQ4,
    potential_tables: NativePotentialTables,
    no_read: NativeGeometricNoRead,
    period: NativeGeometricNoRead,
    geometry: HistoricalH4Tables,
    exp: Vec<u32>,
    binding: SourceActionBinding,
    compiler: SourceEmissionCompiler,
    artifact_binding: NativeArtifactBinding,
}
impl NativeSourceRealizer {
    pub fn load_native(path: &Path, expected: &NativeArtifactBinding) -> Result<Self> {
        expected.identity.validate()?;
        require_digest(&expected.metadata_sha256)?;
        require_inventory(path)?;
        let outer = read_bounded(&path.join("metadata.json"), MAX_METADATA_BYTES)?;
        if sha256_bytes(&outer) != expected.metadata_sha256 {
            return Err(invalid(
                "native artifact trusted outer metadata digest differs",
            ));
        }
        let metadata: RealizerMetadata = serde_json::from_slice(&outer)?;
        require_hashes(&metadata.files, &NATIVE_FILES)?;
        let tokenizer = read_bounded(&path.join("tokenizer.json"), MAX_TOKENIZER_BYTES)?;
        let nested = read_bounded(&path.join("consumer/metadata.json"), MAX_METADATA_BYTES)?;
        check_file(&metadata.files, "tokenizer.json", &tokenizer)?;
        check_file(&metadata.files, "consumer/metadata.json", &nested)?;
        if metadata.schema != SCHEMA
            || metadata.policy != POLICY
            || metadata.surrogate != REALIZER_SURROGATE
            || metadata.action_policy != crate::geometric_source_actions::POLICY
            || metadata.source_view_policy != crate::geometric_source_emission_view::POLICY
            || metadata.identity != expected.identity
            || sha256_bytes(&tokenizer) != expected.identity.tokenizer_sha256
        {
            return Err(invalid(
                "native artifact realizer policy or trusted identity differs",
            ));
        }
        let consumer: ConsumerMetadata = serde_json::from_slice(&nested)?;
        require_hashes(&consumer.files, &CONSUMER_FILES)?;
        consumer
            .context
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        consumer
            .potential
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        consumer
            .no_read
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        metadata
            .period
            .validate()
            .map_err(|e| invalid(e.to_string()))?;
        let c = consumer.context;
        if consumer.schema != "uor-r4.geometric-occurrence-consumer/2"
            || consumer.policy != crate::geometric_occurrence_read::POLICY
            || consumer.surrogate != CONSUMER_SURROGATE
            || consumer.source_view_policy.as_deref()
                != Some(crate::geometric_source_emission_view::POLICY)
            || consumer.identity != expected.identity
            || consumer.algebra_sha256 != sha256_bytes(ALGEBRA)
            || c.heads != consumer.potential.heads
            || c.lanes_per_head != consumer.potential.lanes_per_head
            || c.heads != consumer.no_read.heads
            || c.lanes_per_head != consumer.no_read.latent_lanes_per_head
            || c.vocab_size != consumer.no_read.vocabulary
            || metadata.period != consumer.no_read
        {
            return Err(invalid(
                "native artifact nested policy, geometry, identity or component shapes differ",
            ));
        }
        let binding = SourceActionBinding::new(&tokenizer)?;
        if binding.vocab_size() != c.vocab_size {
            return Err(invalid(
                "native artifact tokenizer/component vocabulary differs",
            ));
        }
        source_parameter_provenance(&metadata, &consumer)?;
        let lengths = [
            c.coefficient_count()
                .map_err(|e| invalid(e.to_string()))?
                .div_ceil(2),
            consumer
                .potential
                .coefficient_count()
                .map_err(|e| invalid(e.to_string()))?
                .div_ceil(2),
            consumer.no_read.coefficient_count().div_ceil(2),
            CANONICAL_EXP_BYTES,
        ];
        let mut bytes = Vec::new();
        for (name, length) in CONSUMER_FILES.into_iter().zip(lengths) {
            let data = read_exact(&path.join("consumer").join(name), length)?;
            check_file(&consumer.files, name, &data)?;
            check_file(&metadata.files, &format!("consumer/{name}"), &data)?;
            bytes.push(data);
        }
        let period_bytes = read_exact(
            &path.join("period-q4.bin"),
            metadata.period.coefficient_count().div_ceil(2),
        )?;
        check_file(&metadata.files, "period-q4.bin", &period_bytes)?;
        let context = NativeContextQ4::new(c, &bytes[0]).map_err(|e| invalid(e.to_string()))?;
        let potential = NativePotentialQ4::new(consumer.potential, &bytes[1])
            .map_err(|e| invalid(e.to_string()))?;
        let potential_tables = NativePotentialQ4::new(consumer.potential, &bytes[1])
            .map_err(|e| invalid(e.to_string()))?
            .into_native()
            .map_err(|e| invalid(e.to_string()))?;
        let no_read = NativeGeometricNoRead::new(consumer.no_read, &bytes[2])
            .map_err(|e| invalid(e.to_string()))?;
        let period = NativeGeometricNoRead::new(metadata.period, &period_bytes)
            .map_err(|e| invalid(e.to_string()))?;
        let exp = canonical_exp(&bytes[3])?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        NativeOccurrenceReader::new(OccurrenceComponents {
            context: context.native(),
            potential: &potential_tables,
            no_read: &no_read,
            geometry: &geometry,
            exp_q31: &exp,
        })
        .map_err(|e| invalid(e.to_string()))?;
        let compiler = SourceEmissionCompiler::new(&tokenizer)?;
        Ok(Self {
            context,
            potential,
            potential_tables,
            no_read,
            period,
            geometry,
            exp,
            binding,
            compiler,
            artifact_binding: expected.clone(),
        })
    }
    pub fn artifact_binding(&self) -> &NativeArtifactBinding {
        &self.artifact_binding
    }
    pub fn context_config(&self) -> ContextQ4Config {
        self.context.config()
    }
    /// Read-only encoder view of this admitted artifact, preserving its exact
    /// token transitions and signed-H4 algebra. No independent mutable encoder.
    pub fn context_encoder_parts(
        &self,
    ) -> (
        &crate::geometric_context::NativeContextTables,
        &HistoricalH4Tables,
    ) {
        (self.context.native(), &self.geometry)
    }
    pub fn read_dependent(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        feedback: &NativeReadFeedback,
        mode: FeedbackInputMode,
    ) -> Result<DependentReadTrace> {
        if feedback.metadata().context != self.context.config()
            || feedback.metadata().parent_artifact != self.artifact_binding
        {
            return Err(invalid(
                "dependent feedback does not bind this native artifact",
            ));
        }
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_dependent(
            frame,
            view,
            query,
            prefix,
            &self.artifact_binding,
            feedback,
            mode,
        )
    }
    pub fn compile_source_end_transport(
        &self,
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        potential: SourceEndAngularQ4,
    ) -> Result<NativeSourceEndTransport<'_>> {
        let binding = self.artifact_binding.clone();
        cue.validate_execution(&binding, &self.context, &self.geometry)?;
        prefix.validate_execution(&binding, &self.context, &self.geometry, cue.metadata())?;
        NativeSourceEndTransport::compile(
            binding,
            &self.context,
            &self.geometry,
            cue.metadata().clone(),
            prefix.metadata().clone(),
            potential,
        )
    }
    pub fn read_bank_with_source_end_transport(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
        end: &NativeSourceEndTransport<'_>,
    ) -> Result<SourceEndBankRealizerTrace> {
        let binding = self.artifact_binding.clone();
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_bank_with_source_end_transport(
            segments,
            query,
            actual_prefix,
            &binding,
            cue,
            prefix,
            end,
        )
    }
    pub fn compile_prefix_transport(
        &self,
        cue: &NativeCueCarrier<'_>,
        potential: PrefixAngularQ4,
    ) -> Result<NativePrefixTransport<'_>> {
        cue.validate_execution(&self.artifact_binding, &self.context, &self.geometry)?;
        NativePrefixTransport::compile(
            self.artifact_binding.clone(),
            &self.context,
            &self.geometry,
            cue.metadata().clone(),
            potential,
        )
    }
    pub fn read_bank_with_prefix_transport(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        transport: &NativePrefixTransport<'_>,
    ) -> Result<PrefixBankRealizerTrace> {
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_bank_with_prefix_transport(
            segments,
            query,
            prefix,
            &self.artifact_binding,
            cue,
            transport,
        )
    }
    pub fn compile_cue_carrier(&self, potential: CueAngularQ4) -> Result<NativeCueCarrier<'_>> {
        NativeCueCarrier::compile(
            self.artifact_binding.clone(),
            &self.context,
            &self.geometry,
            potential,
        )
    }
    pub fn read_bank_with_cue_carrier(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
        carrier: &NativeCueCarrier<'_>,
    ) -> Result<CueBankRealizerTrace> {
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_bank_with_cue_carrier(
            segments,
            query,
            prefix,
            &self.artifact_binding,
            carrier,
        )
    }
    pub fn read_bank(
        &self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        prefix: &[u32],
    ) -> Result<BankRealizerTrace> {
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_bank(segments, query, prefix)
    }
    /// Bank-dependent feedback is intentionally not exposed yet: its legacy
    /// source-local offset lookup must not be used as a bank candidate ordinal.
    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn read_dependent_action_traces(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        feedback: &NativeReadFeedback,
        mode: FeedbackInputMode,
    ) -> Result<NativeActionCounterfactualTrace> {
        if feedback.metadata().context != self.context.config()
            || feedback.metadata().parent_artifact != self.artifact_binding
        {
            return Err(invalid(
                "counterfactual feedback does not bind this native artifact",
            ));
        }
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_dependent_action_traces(
            frame,
            view,
            query,
            prefix,
            &self.artifact_binding,
            feedback,
            mode,
        )
    }
    pub fn read_dependent_action_traces_for(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        prefix: &[u32],
        feedback: &NativeReadFeedback,
        mode: FeedbackInputMode,
        requests: &[NativeLaneActionRequest],
    ) -> Result<NativeActionCounterfactualTrace> {
        if feedback.metadata().context != self.context.config()
            || feedback.metadata().parent_artifact != self.artifact_binding
        {
            return Err(invalid(
                "counterfactual feedback does not bind this native artifact",
            ));
        }
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read_dependent_action_traces_for(
            frame,
            view,
            query,
            prefix,
            &self.artifact_binding,
            feedback,
            mode,
            requests,
        )
    }
    pub fn compile_view(&self, original_ids: &[u32]) -> Result<SourceEmissionView> {
        self.compiler.compile(original_ids)
    }
    pub fn read(
        &self,
        frame: SelectedRecordFrame<'_>,
        view: &SourceEmissionView,
        query: &[u32],
        own_prefix: &[u32],
    ) -> Result<RealizerTrace> {
        RealizerExecution {
            context: &self.context,
            potential_tables: &self.potential_tables,
            no_read: &self.no_read,
            geometry: &self.geometry,
            exp: &self.exp,
            period: &self.period,
            binding: &self.binding,
        }
        .read(frame, view, query, own_prefix)
    }
    pub fn stats(&self) -> serde_json::Value {
        serde_json::json!({"context":self.context.stats(),"potential":self.potential.stats(),"Stop":self.no_read.stats(),"Period":self.period.stats(),
            "exp_bytes":self.exp.len()*4,"algebra_bytes":ALGEBRA.len(),"additional_potential_table_copy_bytes":self.potential.table_bytes().len(),
            "extra_context_replays_per_read":0,"original_context_replays_per_read":1,"prepared_context_scratch_bytes":std::mem::size_of::<PreparedOccurrenceContext>(),"final_joint_reductions_per_read":1,
            "inherited_per_head_reductions_discarded":self.context.config().heads,
            "scope":"source-free packed integer components; loading/tokenization/trace wrapper allocate; no general chat or allocation-free qualification"})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_occurrence_read::{FrameMetadata, FrameStatus, SourceIdentity};
    use std::sync::atomic::{AtomicU64, Ordering};
    const TOKENIZER:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"\u0120":6},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"},{"id":9,"content":"<gap>"}]}"#;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn cue_potential(config: ContextQ4Config, nonzero: bool) -> Result<CueAngularQ4> {
        let config = crate::geometric_cue_carrier::CueAngularConfig {
            heads: config.heads,
            lanes_per_head: config.lanes_per_head,
            mode: crate::geometric_cue_carrier::CueScoreMode::DirectedRelative,
        };
        let mut coefficients = vec![if nonzero { 2 } else { 0 }; config.coefficient_count()?];
        if nonzero {
            for lane in coefficients.chunks_exact_mut(ROOT_COUNT) {
                lane[1] = 7;
            }
        }
        CueAngularQ4::new(
            config,
            &crate::geometric_potential_q4::pack_coefficients(&coefficients)
                .map_err(|e| invalid(e.to_string()))?,
        )
    }
    #[test]
    fn cue_carrier_zero_preserves_full_bank_and_fixed_lifetime_query_prefix() -> Result<()> {
        let f = ActionFixture::new()?;
        let ids = [4];
        let view = f.compiler.compile(&ids)?;
        let source = || SourceBankSegment::Source {
            frame: ActionFixture::frame(&ids),
            view: &view,
            event: 7,
        };
        let segments = [
            source(),
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 99,
                event: 20,
            },
            source(),
            source(),
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 1,
                event: 9,
            },
            SourceBankSegment::Context {
                token_ids: &[4, 5],
                role: 2,
                event: 1,
            },
            source(),
            SourceBankSegment::Context {
                token_ids: &[],
                role: 3,
                event: 0,
            },
            source(),
        ];
        let carrier = NativeCueCarrier::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue_potential(f.context.config(), false)?,
        )?;
        let old = f.execution().read_bank(&segments, &[5], &[])?;
        let added =
            f.execution()
                .read_bank_with_cue_carrier(&segments, &[5], &[], &f.parent, &carrier)?;
        assert_eq!(old, added.bank);
        assert_eq!(added.carrier.cues.len(), 2);
        assert_eq!(added.carrier.cues[0].source_segment_index, 2);
        assert_eq!(added.carrier.cues[0].context_segment_index, 1);
        assert_eq!(added.carrier.cues[0].state.token_ids, [5]);
        assert_eq!(added.carrier.cues[1].source_segment_index, 6);
        assert_eq!(added.carrier.cues[1].context_segment_index, 5);
        assert_eq!(added.carrier.cues[1].state.token_ids, [4, 5]);
        for candidate in &added.bank.candidates {
            let expected = match candidate.segment_index {
                2 => Some(0),
                6 => Some(1),
                _ => None,
            };
            assert_eq!(
                added.carrier.candidate_cue_indices[candidate.bank_index],
                expected
            );
        }
        let prefix = f.execution().read_bank_with_cue_carrier(
            &segments,
            &[5],
            &[3, 4],
            &f.parent,
            &carrier,
        )?;
        assert_eq!(prefix.carrier, added.carrier); // no prefix in carrier or local costs
        assert_ne!(prefix.bank.context.tokens, added.bank.context.tokens);
        assert!(added.carrier.copy_q24.iter().flatten().all(|x| *x == 0));
        assert_eq!(added.carrier.costs.extra_encoder_tokens, 4);
        let foreign = ActionFixture::new()?;
        assert!(foreign
            .execution()
            .read_bank_with_cue_carrier(&segments, &[5], &[], &f.parent, &carrier)
            .is_err());
        Ok(())
    }
    #[test]
    fn cue_carrier_nonzero_affects_every_copy_before_global_alias_terminal_reduce() -> Result<()> {
        let f = ActionFixture::new()?;
        let ids = [4, 4];
        let view = f.compiler.compile(&ids)?;
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 32,
                event: 999,
            },
            SourceBankSegment::Source {
                frame: ActionFixture::frame(&ids),
                view: &view,
                event: 7,
            },
        ];
        let carrier = NativeCueCarrier::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue_potential(f.context.config(), true)?,
        )?;
        let old = f.execution().read_bank(&segments, &[5], &[])?;
        let added =
            f.execution()
                .read_bank_with_cue_carrier(&segments, &[5], &[], &f.parent, &carrier)?;
        assert_eq!(old.context, added.bank.context);
        assert_eq!(old.candidates, added.bank.candidates);
        assert_eq!(old.segments, added.bank.segments);
        assert_eq!(old.period_q24, added.bank.period_q24);
        for (h, (before, after)) in old.heads.iter().zip(&added.bank.heads).enumerate() {
            assert_eq!(before.no_read_q24, after.no_read_q24);
            for j in 0..old.candidates.len() {
                assert!(added.carrier.copy_q24[h][j] > 0);
                assert_eq!(
                    after.scores_q24[j],
                    before.scores_q24[j] + added.carrier.copy_q24[h][j]
                );
            }
        }
        let ids = added
            .bank
            .candidates
            .iter()
            .map(|c| c.occurrence.token_id)
            .collect::<Vec<_>>();
        let scores = added
            .bank
            .heads
            .iter()
            .enumerate()
            .map(|(h, s)| ActionHeadScores {
                copy_q24: &s.scores_q24,
                period_q24: added.bank.period_q24[h],
                stop_q24: s.no_read_q24,
            })
            .collect::<Vec<_>>();
        let independent =
            NativeSourceActions::new(f.binding.clone(), f.context.config().heads, &f.exp)?
                .reduce(&ids, &scores)?;
        assert_eq!(independent, added.bank.actions);
        assert_eq!(
            added
                .bank
                .actions
                .token_masses
                .iter()
                .map(|x| x.weight_q31)
                .sum::<u64>(),
            added.bank.actions.total_weight_q31
        );
        Ok(())
    }
    #[test]
    fn cue_carrier_modes_share_capacity_codes_and_cost_but_use_declared_bins() -> Result<()> {
        use crate::geometric_cue_carrier::{CueAngularConfig, CueScoreMode};
        let f = ActionFixture::new()?;
        let ids = [4];
        let view = f.compiler.compile(&ids)?;
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 77,
                event: 999,
            },
            SourceBankSegment::Source {
                frame: ActionFixture::frame(&ids),
                view: &view,
                event: 7,
            },
        ];
        let directed = cue_potential(f.context.config(), true)?;
        let config = CueAngularConfig {
            mode: CueScoreMode::CueUnary,
            ..directed.config()
        };
        let control = CueAngularQ4::new(config, directed.packed_coefficients())?;
        let directed =
            NativeCueCarrier::compile(f.parent.clone(), &f.context, &f.geometry, directed)?;
        let control =
            NativeCueCarrier::compile(f.parent.clone(), &f.context, &f.geometry, control)?;
        let a =
            f.execution()
                .read_bank_with_cue_carrier(&segments, &[5], &[], &f.parent, &directed)?;
        let b =
            f.execution()
                .read_bank_with_cue_carrier(&segments, &[5], &[], &f.parent, &control)?;
        assert_eq!(a.carrier.query, b.carrier.query);
        assert_eq!(a.carrier.cues, b.carrier.cues);
        assert_eq!(a.carrier.costs, b.carrier.costs);
        assert_eq!(a.carrier.relative_roots, b.carrier.relative_roots);
        assert_eq!(
            directed.packed_coefficients(),
            control.packed_coefficients()
        );
        for lane in 0..a.carrier.angular_indices.len() {
            for (candidate, bin) in a.carrier.angular_indices[lane].iter().enumerate() {
                assert_eq!(*bin, a.carrier.relative_roots[lane][candidate]);
                assert_eq!(
                    b.carrier.angular_indices[lane][candidate],
                    Some(b.carrier.cues[0].state.codes[lane].root)
                );
            }
        }
        assert_ne!(a.carrier.angular_indices, b.carrier.angular_indices);
        assert_ne!(a.carrier.copy_q24, b.carrier.copy_q24);
        assert!(CueAngularQ4::new(
            CueAngularConfig {
                heads: 3,
                lanes_per_head: 1,
                mode: CueScoreMode::DirectedRelative
            },
            &[]
        )
        .is_err());
        assert!(CueAngularQ4::new(config, &vec![0x88; config.coefficient_count()? / 2]).is_err());
        Ok(())
    }
    fn prefix_potential(
        c: ContextQ4Config,
        mode: crate::geometric_prefix_transport::PrefixScoreMode,
        nonzero: bool,
    ) -> Result<PrefixAngularQ4> {
        let config = crate::geometric_prefix_transport::PrefixAngularConfig {
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            mode,
        };
        let mut q = vec![0; config.coefficient_count()?];
        if nonzero {
            for row in q.chunks_exact_mut(ROOT_COUNT) {
                row.fill(-2);
                row[usize::from(H4Code::IDENTITY.index())] = 7;
            }
        }
        PrefixAngularQ4::new(
            config,
            &crate::geometric_potential_q4::pack_coefficients(&q)
                .map_err(|e| invalid(e.to_string()))?,
        )
    }
    #[test]
    fn source_end_zero_full_parity_last_token_and_bank_source_binding() -> Result<()> {
        use crate::geometric_prefix_transport::PrefixScoreMode;
        use crate::geometric_source_end_transport::{SourceEndAngularConfig, SourceEndScoreMode};
        let f = ActionFixture::new()?;
        let ids = [4, 5];
        let view = f.compiler.compile(&ids)?;
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 1,
                event: 2,
            },
            SourceBankSegment::Source {
                frame: ActionFixture::frame(&ids),
                view: &view,
                event: 7,
            },
            SourceBankSegment::Source {
                frame: ActionFixture::frame(&ids),
                view: &view,
                event: 8,
            },
        ];
        let cue = NativeCueCarrier::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue_potential(f.context.config(), true)?,
        )?;
        let prefix = NativePrefixTransport::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue.metadata().clone(),
            prefix_potential(f.context.config(), PrefixScoreMode::DirectedRelative, true)?,
        )?;
        let c = f.context.config();
        let config = SourceEndAngularConfig {
            heads: c.heads,
            lanes_per_head: c.lanes_per_head,
            mode: SourceEndScoreMode::DirectedRelative,
        };
        let zero = vec![0; config.coefficient_count()? / 2];
        let end = NativeSourceEndTransport::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue.metadata().clone(),
            prefix.metadata().clone(),
            SourceEndAngularQ4::new(config, &zero, &zero)?,
        )?;
        let old = f.execution().read_bank_with_prefix_transport(
            &segments,
            &[5],
            &[4],
            &f.parent,
            &cue,
            &prefix,
        )?;
        let result = f.execution().read_bank_with_source_end_transport(
            &segments,
            &[5],
            &[4],
            &f.parent,
            &cue,
            &prefix,
            &end,
        )?;
        assert_eq!(result.prefix_bank, old);
        assert_eq!(result.actions, old.cue_bank.bank.actions);
        assert_eq!(result.source_end.response, old.prefix.response);
        assert_eq!(result.source_end.costs.extra_source_encoder_tokens, 2);
        let mut state =
            crate::geometric_context::NativeContextState::new(c.heads, c.lanes_per_head)
                .map_err(|e| invalid(e.to_string()))?;
        for &token in view.emitted_token_ids() {
            state
                .step(token as usize, f.context.native(), &f.geometry)
                .map_err(|e| invalid(e.to_string()))?;
        }
        assert_eq!(
            result.source_end.sources[0].states,
            state.states().iter().map(|s| s.index()).collect::<Vec<_>>()
        );
        assert_eq!(result.source_end.sources[0].source_segment_index, 1);
        let selected = result
            .source_end
            .selected_bank_index
            .ok_or_else(|| invalid("fixture Copy route absent"))?;
        let source = result
            .source_end
            .selected_source_index
            .ok_or_else(|| invalid("fixture Source route absent"))?;
        assert_eq!(
            old.cue_bank.bank.candidates[selected].segment_index,
            result.source_end.sources[source].source_segment_index
        );
        // Terminal coefficients cannot change the factual route, Copy scores,
        // or the latent response/end carriers; both tables independently add.
        let q =
            crate::geometric_potential_q4::pack_coefficients(&vec![1; config.coefficient_count()?])
                .map_err(|e| invalid(e.to_string()))?;
        let nonzero = NativeSourceEndTransport::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue.metadata().clone(),
            prefix.metadata().clone(),
            SourceEndAngularQ4::new(config, &q, &zero)?,
        )?;
        let changed = f.execution().read_bank_with_source_end_transport(
            &segments,
            &[5],
            &[4],
            &f.parent,
            &cue,
            &prefix,
            &nonzero,
        )?;
        assert_eq!(changed.prefix_bank, old);
        assert_eq!(
            changed.source_end.selected_bank_index,
            result.source_end.selected_bank_index
        );
        assert_eq!(changed.source_end.stop_q24, vec![0; c.heads]);
        assert_eq!(
            changed.source_end.period_q24,
            vec![(c.lanes_per_head as i64) << 22; c.heads]
        );
        let h = old
            .cue_bank
            .bank
            .heads
            .iter()
            .enumerate()
            .map(|(i, h)| ActionHeadScores {
                copy_q24: &h.scores_q24,
                period_q24: old.cue_bank.bank.period_q24[i] + changed.source_end.period_q24[i],
                stop_q24: h.no_read_q24,
            })
            .collect::<Vec<_>>();
        let tokens = old
            .cue_bank
            .bank
            .candidates
            .iter()
            .map(|c| c.occurrence.token_id)
            .collect::<Vec<_>>();
        assert_eq!(
            changed.actions,
            NativeSourceActions::new(f.binding.clone(), c.heads, &f.exp)?.reduce(&tokens, &h)?
        );
        let unary = NativeSourceEndTransport::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue.metadata().clone(),
            prefix.metadata().clone(),
            SourceEndAngularQ4::new(
                SourceEndAngularConfig {
                    mode: SourceEndScoreMode::SourceEndUnary,
                    ..config
                },
                &q,
                &zero,
            )?,
        )?;
        let control = f.execution().read_bank_with_source_end_transport(
            &segments,
            &[5],
            &[4],
            &f.parent,
            &cue,
            &prefix,
            &unary,
        )?;
        assert_eq!(control.source_end.sources, changed.source_end.sources);
        assert_eq!(
            control.source_end.relative_roots,
            changed.source_end.relative_roots
        );
        assert_eq!(control.source_end.costs, changed.source_end.costs);
        for (lane, &index) in control.source_end.angular_indices.iter().enumerate() {
            assert_eq!(index, control.source_end.sources[source].states[lane]);
        }
        let alternate = f.execution().read_bank_with_source_end_transport(
            &segments,
            &[5],
            &[5],
            &f.parent,
            &cue,
            &prefix,
            &end,
        )?;
        assert_eq!(alternate.source_end.sources, result.source_end.sources);
        assert_ne!(
            alternate.source_end.response.token_ids,
            result.source_end.response.token_ids
        );
        // A shorter source and repeated opaque events cannot let terminal
        // coefficients reroute the factual donor. Prefix/query changes are
        // independently recomputed rather than retaining a hidden cursor.
        let short_ids = [4];
        let short_view = f.compiler.compile(&short_ids)?;
        let mixed = [
            SourceBankSegment::Source {
                frame: ActionFixture::frame(&short_ids),
                view: &short_view,
                event: 7,
            },
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 1,
                event: 7,
            },
            SourceBankSegment::Source {
                frame: ActionFixture::frame(&ids),
                view: &view,
                event: 7,
            },
        ];
        for (query, actual) in [(&[4][..], &[4][..]), (&[5][..], &[4, 3][..])] {
            let base = f.execution().read_bank_with_source_end_transport(
                &mixed, query, actual, &f.parent, &cue, &prefix, &end,
            )?;
            let changed = f.execution().read_bank_with_source_end_transport(
                &mixed, query, actual, &f.parent, &cue, &prefix, &nonzero,
            )?;
            assert_eq!(changed.prefix_bank, base.prefix_bank);
            assert_eq!(
                changed.source_end.selected_bank_index,
                base.source_end.selected_bank_index
            );
            assert_eq!(
                changed.source_end.selected_source_index,
                base.source_end.selected_source_index
            );
            let index = base
                .source_end
                .selected_bank_index
                .ok_or_else(|| invalid("mixed route absent"))?;
            let source = base
                .source_end
                .selected_source_index
                .ok_or_else(|| invalid("mixed Source absent"))?;
            assert_eq!(
                base.prefix_bank.cue_bank.bank.candidates[index].segment_index,
                base.source_end.sources[source].source_segment_index
            );
        }
        let empty = [SourceBankSegment::Context {
            token_ids: &[5],
            role: 1,
            event: 2,
        }];
        // The retained occurrence reader requires a Source. This wrapper
        // preserves its EmptyBank error rather than inventing an absent-source
        // serving path. The standalone factual-route test covers zero atoms.
        let old_empty = f.execution().read_bank_with_prefix_transport(
            &empty,
            &[5],
            &[],
            &f.parent,
            &cue,
            &prefix,
        );
        let new_empty = f.execution().read_bank_with_source_end_transport(
            &empty,
            &[5],
            &[],
            &f.parent,
            &cue,
            &prefix,
            &nonzero,
        );
        assert!(old_empty.is_err());
        assert!(new_empty.is_err());
        let mut foreign = f.parent.clone();
        foreign.metadata_sha256 = "1".repeat(64);
        assert_ne!(foreign.metadata_sha256, f.parent.metadata_sha256);
        assert!(f
            .execution()
            .read_bank_with_source_end_transport(
                &segments,
                &[5],
                &[],
                &foreign,
                &cue,
                &prefix,
                &end
            )
            .is_err());
        Ok(())
    }

    #[test]
    fn ordered_prefix_zero_preserves_cue_bank_and_before_token_provenance() -> Result<()> {
        use crate::geometric_prefix_transport::PrefixScoreMode;
        let f = ActionFixture::new()?;
        let ids = [4, 5];
        let view = f.compiler.compile(&ids)?;
        let source = || SourceBankSegment::Source {
            frame: ActionFixture::frame(&ids),
            view: &view,
            event: 7,
        };
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[5],
                role: 1,
                event: 2,
            },
            source(),
            source(),
        ];
        let cue = NativeCueCarrier::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue_potential(f.context.config(), true)?,
        )?;
        let transport = NativePrefixTransport::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue.metadata().clone(),
            prefix_potential(f.context.config(), PrefixScoreMode::DirectedRelative, false)?,
        )?;
        let old =
            f.execution()
                .read_bank_with_cue_carrier(&segments, &[5], &[4], &f.parent, &cue)?;
        let added = f.execution().read_bank_with_prefix_transport(
            &segments,
            &[5],
            &[4],
            &f.parent,
            &cue,
            &transport,
        )?;
        assert_eq!(old, added.cue_bank);
        assert_eq!(added.prefix.sources.len(), 2);
        assert_eq!(added.prefix.sources[0].token_ids, view.emitted_token_ids());
        assert_eq!(
            added.prefix.sources[0].states_before,
            added.prefix.sources[1].states_before
        );
        let c = f.context.config();
        let width = c.heads * c.lanes_per_head;
        let mut state =
            crate::geometric_context::NativeContextState::new(c.heads, c.lanes_per_head)
                .map_err(|e| invalid(e.to_string()))?;
        for (j, &token) in view.emitted_token_ids().iter().enumerate() {
            assert_eq!(
                added.prefix.sources[0].states_before[j],
                state.states().iter().map(|x| x.index()).collect::<Vec<_>>()
            );
            state
                .step(token as usize, f.context.native(), &f.geometry)
                .map_err(|e| invalid(e.to_string()))?;
        }
        assert_eq!(
            added.prefix.sources[0].states_before[0],
            vec![H4Code::IDENTITY.index(); width]
        );
        assert_eq!(
            added.prefix.costs.extra_source_encoder_tokens,
            2 * (view.emitted_token_ids().len() - 1)
        );
        assert_eq!(added.prefix.costs.extra_response_encoder_tokens, 1);
        for candidate in &added.cue_bank.bank.candidates {
            let i = candidate.bank_index;
            let source = added.prefix.candidate_source_indices[i];
            assert_eq!(
                added.prefix.sources[source].source_segment_index,
                candidate.segment_index
            );
            assert_eq!(
                added.prefix.candidate_offsets[i],
                usize::try_from(candidate.occurrence.token_offset)
                    .map_err(|e| invalid(e.to_string()))?
            );
            assert_eq!(
                added.prefix.sources[source].token_ids[added.prefix.candidate_offsets[i]],
                candidate.occurrence.token_id
            );
        }
        let empty = f.execution().read_bank_with_prefix_transport(
            &segments,
            &[5],
            &[],
            &f.parent,
            &cue,
            &transport,
        )?;
        assert_eq!(
            empty.prefix.response.states,
            vec![H4Code::IDENTITY.index(); width]
        );
        assert_eq!(empty.prefix.sources, added.prefix.sources);
        let foreign = ActionFixture::new()?;
        assert!(foreign
            .execution()
            .read_bank_with_prefix_transport(&segments, &[5], &[], &f.parent, &cue, &transport)
            .is_err());
        let changed_cue = NativeCueCarrier::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue_potential(f.context.config(), false)?,
        )?;
        assert!(f
            .execution()
            .read_bank_with_prefix_transport(
                &segments,
                &[5],
                &[],
                &f.parent,
                &changed_cue,
                &transport
            )
            .is_err());
        Ok(())
    }
    #[test]
    fn ordered_prefix_nonzero_is_occurrence_and_actual_prefix_dependent_with_matched_control(
    ) -> Result<()> {
        use crate::geometric_prefix_transport::PrefixScoreMode;
        let f = ActionFixture::new()?;
        let ids = [4, 5];
        let view = f.compiler.compile(&ids)?;
        // No cue at all: ordered factor still scores every Source occurrence.
        let segments = [SourceBankSegment::Source {
            frame: ActionFixture::frame(&ids),
            view: &view,
            event: 7,
        }];
        let cue = NativeCueCarrier::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue_potential(f.context.config(), true)?,
        )?;
        let directed = NativePrefixTransport::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue.metadata().clone(),
            prefix_potential(f.context.config(), PrefixScoreMode::DirectedRelative, true)?,
        )?;
        let unary = NativePrefixTransport::compile(
            f.parent.clone(),
            &f.context,
            &f.geometry,
            cue.metadata().clone(),
            prefix_potential(f.context.config(), PrefixScoreMode::SourcePrefixUnary, true)?,
        )?;
        let read = |prefix: &[u32], transport: &NativePrefixTransport<'_>| {
            f.execution().read_bank_with_prefix_transport(
                &segments,
                &[5],
                prefix,
                &f.parent,
                &cue,
                transport,
            )
        };
        let a = read(&[], &directed)?;
        let b = read(&[4], &directed)?;
        let control = read(&[4], &unary)?;
        assert!(a.cue_bank.carrier.cues.is_empty());
        assert_ne!(a.prefix.copy_q24[0][0], a.prefix.copy_q24[0][1]);
        assert_ne!(a.prefix.angular_indices, b.prefix.angular_indices);
        assert_ne!(a.prefix.copy_q24, b.prefix.copy_q24);
        assert_eq!(b.prefix.sources, control.prefix.sources);
        assert_eq!(b.prefix.response, control.prefix.response);
        assert_eq!(b.prefix.relative_roots, control.prefix.relative_roots);
        assert_eq!(b.prefix.costs, control.prefix.costs);
        assert_ne!(b.prefix.angular_indices, control.prefix.angular_indices);
        for lane in 0..control.prefix.angular_indices.len() {
            for j in 0..control.prefix.candidate_offsets.len() {
                let source = control.prefix.candidate_source_indices[j];
                let offset = control.prefix.candidate_offsets[j];
                assert_eq!(
                    control.prefix.angular_indices[lane][j],
                    control.prefix.sources[source].states_before[offset][lane]
                );
                assert_eq!(
                    b.prefix.angular_indices[lane][j],
                    b.prefix.relative_roots[lane][j]
                );
            }
        }
        let baseline =
            f.execution()
                .read_bank_with_cue_carrier(&segments, &[5], &[], &f.parent, &cue)?;
        assert_eq!(a.cue_bank.bank.context, baseline.bank.context);
        assert_eq!(a.cue_bank.bank.candidates, baseline.bank.candidates);
        assert_eq!(a.cue_bank.bank.period_q24, baseline.bank.period_q24);
        for h in 0..baseline.bank.heads.len() {
            assert_eq!(
                a.cue_bank.bank.heads[h].no_read_q24,
                baseline.bank.heads[h].no_read_q24
            );
            for j in 0..baseline.bank.candidates.len() {
                assert_eq!(
                    a.cue_bank.bank.heads[h].scores_q24[j],
                    baseline.bank.heads[h].scores_q24[j] + a.prefix.copy_q24[h][j]
                );
            }
        }
        assert_eq!(directed.packed_coefficients(), unary.packed_coefficients());
        assert!(PrefixAngularQ4::new(
            directed.metadata().potential,
            &vec![0x88; directed.packed_coefficients().len()]
        )
        .is_err());
        Ok(())
    }
    struct ActionFixture {
        context: NativeContextQ4,
        potential: NativePotentialTables,
        stop: NativeGeometricNoRead,
        period: NativeGeometricNoRead,
        geometry: HistoricalH4Tables,
        exp: Vec<u32>,
        binding: SourceActionBinding,
        compiler: SourceEmissionCompiler,
        parent: NativeArtifactBinding,
        feedback: NativeReadFeedback,
    }
    impl ActionFixture {
        fn new() -> Result<Self> {
            let c = ContextQ4Config {
                vocab_size: 10,
                heads: 2,
                lanes_per_head: 1,
            };
            let mut context_q = Vec::new();
            for (name, shape) in c.coefficient_shapes().map_err(|e| invalid(e.to_string()))? {
                let mut q = vec![0; shape.iter().product::<usize>()];
                if name == "token_transition" {
                    for row in q.chunks_exact_mut(ROOT_COUNT) {
                        row[4] = 7;
                    }
                }
                if name == "self_root" {
                    for lane in q.chunks_exact_mut(ROOT_COUNT * 4) {
                        for (root, basis) in crate::geometric_context_q4::canonical_basis_q25()
                            .iter()
                            .enumerate()
                        {
                            for (coordinate, x) in basis.iter().enumerate() {
                                let numerator = i64::from(*x) * 7;
                                lane[root * 4 + coordinate] = if numerator >= 0 {
                                    ((numerator + (1 << 24)) >> 25) as i8
                                } else {
                                    -(((-numerator + (1 << 24)) >> 25) as i8)
                                };
                            }
                        }
                    }
                }
                if name == "token_category" {
                    for token in q.chunks_exact_mut(33) {
                        token[1] = 7;
                    }
                }
                context_q.extend(q);
            }
            let context = NativeContextQ4::new(
                c,
                &crate::geometric_context_q4::pack_coefficients(&context_q)
                    .map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
            let p = PotentialQ4Config {
                heads: 2,
                lanes_per_head: 1,
            };
            let mut potential_q = Vec::new();
            for (name, shape) in p.coefficient_shapes().map_err(|e| invalid(e.to_string()))? {
                let mut q = vec![0; shape.iter().product::<usize>()];
                if name == "context_unary" {
                    q[0] = 4;
                    q[4] = -4;
                }
                potential_q.extend(q);
            }
            let potential = NativePotentialQ4::new(
                p,
                &crate::geometric_potential_q4::pack_coefficients(&potential_q)
                    .map_err(|e| invalid(e.to_string()))?,
            )
            .map_err(|e| invalid(e.to_string()))?
            .into_native()
            .map_err(|e| invalid(e.to_string()))?;
            let n = NoReadConfig {
                vocabulary: 10,
                heads: 2,
                latent_lanes_per_head: 1,
            };
            let controller = |bias: i8| -> Result<NativeGeometricNoRead> {
                let mut q = vec![0; n.coefficient_count()];
                for row in q.chunks_exact_mut(n.coefficients_per_head()) {
                    row[0] = bias;
                    row[1 + n.vocabulary + 2] = 5;
                    row[1 + n.vocabulary + 4 + 2] = 3;
                }
                NativeGeometricNoRead::new(
                    n,
                    &crate::geometric_no_read::pack_coefficients(&q)
                        .map_err(|e| invalid(e.to_string()))?,
                )
                .map_err(|e| invalid(e.to_string()))
            };
            let binding = SourceActionBinding::new(TOKENIZER)?;
            let parent = NativeArtifactBinding {
                metadata_sha256: "0".repeat(64),
                identity: ArtifactIdentity {
                    tokenizer_sha256: binding.tokenizer_sha256().into(),
                    parent_checkpoint_manifest_sha256: "1".repeat(64),
                    parent_model_sha256: "2".repeat(64),
                    parent_config_sha256: "3".repeat(64),
                },
            };
            let value_config = NativeReadFeedback::value_config(c);
            let value = vec![
                0;
                value_config
                    .coefficient_count()
                    .map_err(|e| invalid(e.to_string()))?
                    .div_ceil(2)
            ];
            let mut bridge_q = vec![0; NativeReadFeedback::bridge_coefficient_count(c)?];
            bridge_q[0] = 1; // factual nonidentity action0 on lane0; lane1 identity tie.
            let feedback = NativeReadFeedback::compile(
                parent.clone(),
                c,
                &value,
                &crate::geometric_value_q4::pack_coefficients(&bridge_q)
                    .map_err(|e| invalid(e.to_string()))?,
            )?;
            let mut exp = vec![0; crate::geometric_read::EXP_TABLE_LEN];
            exp[0] = crate::geometric_read::WEIGHT_ONE as u32;
            Ok(Self {
                context,
                potential,
                stop: controller(7)?,
                period: controller(-7)?,
                geometry: HistoricalH4Tables::from_bytes(ALGEBRA)
                    .map_err(|e| invalid(e.to_string()))?,
                exp,
                binding,
                compiler: SourceEmissionCompiler::new(TOKENIZER)?,
                parent,
                feedback,
            })
        }
        fn execution(&self) -> RealizerExecution<'_> {
            RealizerExecution {
                context: &self.context,
                potential_tables: &self.potential,
                no_read: &self.stop,
                geometry: &self.geometry,
                exp: &self.exp,
                period: &self.period,
                binding: &self.binding,
            }
        }
        fn frame(tokens: &[u32]) -> SelectedRecordFrame<'_> {
            SelectedRecordFrame {
                identity: SourceIdentity {
                    record: 7,
                    commit: 9,
                },
                metadata: FrameMetadata {
                    scope: b"unit",
                    entity: &[],
                    relation: 3,
                    view: 0,
                    status: FrameStatus::Found,
                },
                token_ids: tokens,
            }
        }
    }
    #[test]
    fn native_bank_single_source_parent_exact_and_interleaved_global_reduction() -> Result<()> {
        let f = ActionFixture::new()?;
        let view = f.compiler.compile(&[4])?;
        let execution = f.execution();
        let frame = ActionFixture::frame(&[4]);
        for prefix in [&[][..], &[4][..]] {
            let old = execution.read(frame, &view, &[4], prefix)?;
            let single = [SourceBankSegment::Source {
                frame,
                view: &view,
                event: 1,
            }];
            let bank = execution.read_bank(&single, &[4], prefix)?;
            assert_eq!(bank.actions, old.actions);
            assert_eq!(bank.context, old.period_context);
            assert_eq!(bank.heads, old.source.view_kernel_trace.heads);
            assert_eq!(bank.period_q24, old.period_q24);
        }
        let segments = [
            SourceBankSegment::Context {
                token_ids: &[4],
                event: 1,
                role: 7,
            },
            SourceBankSegment::Source {
                frame,
                view: &view,
                event: 2,
            },
            SourceBankSegment::Context {
                token_ids: &[4],
                event: 3,
                role: 8,
            },
            SourceBankSegment::Source {
                frame,
                view: &view,
                event: 4,
            },
        ];
        let bank = execution.read_bank(&segments, &[4], &[4])?;
        let n = view.emitted_token_ids().len();
        assert_eq!(bank.candidates.len(), 2 * n);
        assert_eq!(bank.candidates[0].context_position, 1);
        assert_eq!(bank.candidates[n].context_position, n + 2);
        assert_eq!(bank.candidates[0].occurrence, bank.candidates[n].occurrence);
        assert_eq!(bank.candidates[0].segment_index, 1);
        assert_eq!(bank.candidates[n].segment_index, 3);
        let ids = bank
            .candidates
            .iter()
            .map(|c| c.occurrence.token_id)
            .collect::<Vec<_>>();
        let heads = bank
            .heads
            .iter()
            .enumerate()
            .map(|(h, s)| ActionHeadScores {
                copy_q24: &s.scores_q24,
                period_q24: bank.period_q24[h],
                stop_q24: s.no_read_q24,
            })
            .collect::<Vec<_>>();
        let direct = NativeSourceActions::new(f.binding.clone(), f.context.config().heads, &f.exp)?
            .reduce(&ids, &heads)?;
        assert_eq!(bank.actions, direct);
        assert_eq!(bank.actions.actions.len(), 2 * n + 2);
        Ok(())
    }
    #[test]
    fn native_action_counterfactual_factual_replay_and_full_action_identity() -> Result<()> {
        let f = ActionFixture::new()?;
        let view = f.compiler.compile(&[4])?;
        let execution = f.execution();
        let trace = execution.read_dependent_action_traces(
            ActionFixture::frame(&[4]),
            &view,
            &[4],
            &[],
            &f.parent,
            &f.feedback,
            FeedbackInputMode::JointCopy,
        )?;
        let factual = execution.read_dependent(
            ActionFixture::frame(&[4]),
            &view,
            &[4],
            &[],
            &f.parent,
            &f.feedback,
            FeedbackInputMode::JointCopy,
        )?;
        assert_eq!(trace.factual, factual);
        assert_eq!(trace.lanes.len(), 2);
        assert_eq!(trace.scoring_stages, 242);
        assert_eq!(trace.original_context_replays, 1);
        assert_eq!(trace.value_producer_evaluations, 1);
        for lane in &trace.lanes {
            assert_eq!(lane.actions.len(), 120);
            for (index, candidate) in lane.actions.iter().enumerate() {
                assert_eq!(candidate.action, index as u8);
                for (other, actual) in factual.feedback.actions.iter().enumerate() {
                    if other != lane.lane {
                        assert_eq!(candidate.action_vector[other], *actual);
                    }
                }
            }
            let actual = lane
                .actions
                .get(factual.feedback.actions[lane.lane] as usize)
                .ok_or_else(|| invalid("factual class missing"))?;
            assert_eq!(actual.final_actions, factual.stage2.actions);
            assert_eq!(
                actual.controller_snapshot,
                factual.stage2_controller_snapshot
            );
        }
        Ok(())
    }
    #[test]
    fn native_action_counterfactual_right_composes_original_and_scores_all_families() -> Result<()>
    {
        let f = ActionFixture::new()?;
        let view = f.compiler.compile(&[4])?;
        let execution = f.execution();
        let request = NativeLaneActionRequest {
            lane: 0,
            actions: vec![H4Code::try_from(2).map_err(|e| invalid(e.to_string()))?],
        };
        let trace = execution.read_dependent_action_traces_for(
            ActionFixture::frame(&[4]),
            &view,
            &[4],
            &[],
            &f.parent,
            &f.feedback,
            FeedbackInputMode::JointCopy,
            &[request],
        )?;
        let candidate = &trace.lanes[0].actions[0];
        let (mut reader, prepared, replay) =
            execution.prepare_view(ActionFixture::frame(&[4]), &view, &[4], &[])?;
        let original = prepared
            .original_snapshot()
            .map_err(|e| invalid(e.to_string()))?;
        let keys = prepared
            .source_occurrences()
            .iter()
            .map(|o| (o.source, o.token_offset, o.token_id))
            .collect::<Vec<_>>();
        let actions = candidate
            .action_vector
            .iter()
            .map(|a| H4Code::try_from(*a).map_err(|e| invalid(e.to_string())))
            .collect::<Result<Vec<_>>>()?;
        for (lane, (old, action)) in original.states().iter().zip(&actions).enumerate() {
            assert_eq!(
                candidate.controller_snapshot.states[lane],
                f.geometry.compose(*old, *action).index()
            );
        }
        assert_ne!(
            f.geometry.compose(original.states()[0], actions[0]),
            f.geometry.compose(actions[0], original.states()[0])
        );
        let snapshot = prepared
            .apply_actions(&original, &actions)
            .map_err(|e| invalid(e.to_string()))?;
        assert_eq!(
            candidate.controller_snapshot,
            QuerySnapshotReport::from(&snapshot)
        );
        let scored = execution.score_view(
            ActionFixture::frame(&[4]),
            &view,
            &mut reader,
            &prepared,
            &snapshot,
            &replay,
        )?;
        assert_eq!(candidate.final_actions, scored.actions);
        assert_eq!(
            keys,
            prepared
                .source_occurrences()
                .iter()
                .map(|o| (o.source, o.token_offset, o.token_id))
                .collect::<Vec<_>>()
        );
        assert_eq!(trace.factual.stage1.source.emission_view, view);
        assert_ne!(
            candidate.controller_snapshot.codes,
            trace.factual.stage2_controller_snapshot.codes
        );
        assert_ne!(
            candidate.final_actions.head_scores[0].copy_q24,
            trace.factual.stage2.actions.head_scores[0].copy_q24
        );
        assert_eq!(
            candidate.final_actions.head_scores[1].copy_q24,
            trace.factual.stage2.actions.head_scores[1].copy_q24
        );
        for head in 0..2 {
            assert_ne!(
                candidate.final_actions.head_scores[head].stop_q24,
                trace.factual.stage2.actions.head_scores[head].stop_q24
            );
            assert_ne!(
                candidate.final_actions.head_scores[head].period_q24,
                trace.factual.stage2.actions.head_scores[head].period_q24
            );
        }
        Ok(())
    }
    #[test]
    fn native_action_counterfactual_invalid_lane_duplicates_and_binding_are_errors() -> Result<()> {
        let f = ActionFixture::new()?;
        let view = f.compiler.compile(&[4])?;
        let execution = f.execution();
        for requests in [
            vec![NativeLaneActionRequest {
                lane: 2,
                actions: vec![H4Code::IDENTITY],
            }],
            vec![NativeLaneActionRequest {
                lane: 0,
                actions: vec![],
            }],
            vec![NativeLaneActionRequest {
                lane: 0,
                actions: vec![H4Code::IDENTITY, H4Code::IDENTITY],
            }],
            vec![
                NativeLaneActionRequest {
                    lane: 0,
                    actions: vec![H4Code::IDENTITY],
                },
                NativeLaneActionRequest {
                    lane: 0,
                    actions: vec![H4Code::IDENTITY],
                },
            ],
        ] {
            assert!(execution
                .read_dependent_action_traces_for(
                    ActionFixture::frame(&[4]),
                    &view,
                    &[4],
                    &[],
                    &f.parent,
                    &f.feedback,
                    FeedbackInputMode::JointCopy,
                    &requests
                )
                .is_err());
        }
        let mut foreign = f.parent.clone();
        foreign.metadata_sha256 = "f".repeat(64);
        assert!(execution
            .read_dependent_action_traces(
                ActionFixture::frame(&[4]),
                &view,
                &[4],
                &[],
                &foreign,
                &f.feedback,
                FeedbackInputMode::JointCopy
            )
            .is_err());
        Ok(())
    }
    struct Fixture {
        root: std::path::PathBuf,
        binding: NativeArtifactBinding,
    }
    impl Fixture {
        fn new() -> Result<Self> {
            let root = std::env::temp_dir().join(format!(
                "native-source-loader-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root)?;
            fs::create_dir(root.join("consumer"))?;
            let identity = ArtifactIdentity {
                tokenizer_sha256: sha256_bytes(TOKENIZER),
                parent_checkpoint_manifest_sha256: "a".repeat(64),
                parent_model_sha256: "b".repeat(64),
                parent_config_sha256: "c".repeat(64),
            };
            let c = ContextQ4Config {
                vocab_size: 10,
                heads: 1,
                lanes_per_head: 1,
            };
            let p = PotentialQ4Config {
                heads: 1,
                lanes_per_head: 1,
            };
            let n = NoReadConfig {
                vocabulary: 10,
                heads: 1,
                latent_lanes_per_head: 1,
            };
            let data = [
                vec![
                    0;
                    c.coefficient_count()
                        .map_err(|e| invalid(e.to_string()))?
                        .div_ceil(2)
                ],
                vec![
                    0;
                    p.coefficient_count()
                        .map_err(|e| invalid(e.to_string()))?
                        .div_ceil(2)
                ],
                vec![0; n.coefficient_count().div_ceil(2)],
                vec![0; CANONICAL_EXP_BYTES],
            ];
            let mut inner = BTreeMap::new();
            for (name, bytes) in CONSUMER_FILES.into_iter().zip(&data) {
                fs::write(root.join("consumer").join(name), bytes)?;
                inner.insert(name, sha256_bytes(bytes));
            }
            let nested = serde_json::json!({"schema":"uor-r4.geometric-occurrence-consumer/2","policy":crate::geometric_occurrence_read::POLICY,
                "surrogate":CONSUMER_SURROGATE,"identity":identity,"context":c,"potential":p,"no_read":n,
                "algebra_sha256":sha256_bytes(ALGEBRA),"files":inner,"source_view_policy":crate::geometric_source_emission_view::POLICY});
            fs::write(
                root.join("consumer/metadata.json"),
                serde_json::to_vec(&nested)?,
            )?;
            fs::write(root.join("tokenizer.json"), TOKENIZER)?;
            fs::write(root.join("period-q4.bin"), &data[2])?;
            let mut shapes = BTreeMap::new();
            for (name, shape) in c.coefficient_shapes().map_err(|e| invalid(e.to_string()))? {
                shapes.insert(format!("consumer.context.{name}"), shape);
            }
            for (name, shape) in p.coefficient_shapes().map_err(|e| invalid(e.to_string()))? {
                shapes.insert(format!("consumer.potential.{name}"), shape);
            }
            shapes.insert(
                "consumer.no_read.coefficients".into(),
                vec![1, n.coefficients_per_head()],
            );
            shapes.insert(
                "period.coefficients".into(),
                vec![1, n.coefficients_per_head()],
            );
            let provenance = shapes
                .into_iter()
                .map(|(name, shape)| {
                    (
                        name,
                        serde_json::json!({"shape":shape,"f32_sha256":"d".repeat(64)}),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let files = NATIVE_FILES
                .into_iter()
                .map(|name| Ok((name, sha256_bytes(&fs::read(root.join(name))?))))
                .collect::<Result<BTreeMap<_, _>>>()?;
            let outer = serde_json::to_vec(
                &serde_json::json!({"schema":SCHEMA,"policy":POLICY,"surrogate":REALIZER_SURROGATE,
                "source_view_policy":crate::geometric_source_emission_view::POLICY,"action_policy":crate::geometric_source_actions::POLICY,
                "identity":identity,"period":n,"source_parameters":provenance,"files":files}),
            )?;
            fs::write(root.join("metadata.json"), &outer)?;
            Ok(Self {
                root,
                binding: NativeArtifactBinding {
                    metadata_sha256: sha256_bytes(&outer),
                    identity,
                },
            })
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    fn error(f: &Fixture) -> String {
        match NativeSourceRealizer::load_native(&f.root, &f.binding) {
            Ok(_) => "accepted".into(),
            Err(e) => e.to_string(),
        }
    }
    #[test]
    fn source_free_loader_checks_trusted_receipt_inventory_and_payloads() -> Result<()> {
        let mut f = Fixture::new()?;
        // This deliberately noncanonical table reaches the exact exp trust
        // boundary; tests never generate floating exponentials as a fixture.
        assert!(error(&f).contains("canonical exponential bytes"));
        f.binding.metadata_sha256 = "e".repeat(64);
        assert!(error(&f).contains("trusted outer metadata"));
        let f = Fixture::new()?;
        fs::write(f.root.join("unexpected"), b"extra")?;
        assert!(error(&f).contains("exact inventory"));
        let f = Fixture::new()?;
        fs::write(f.root.join("consumer/no-read-q4.bin"), [1u8])?;
        assert!(error(&f).contains("exact byte length"));
        let f = Fixture::new()?;
        let mut bytes = fs::read(f.root.join("consumer/potential-q4.bin"))?;
        bytes[0] = 1;
        fs::write(f.root.join("consumer/potential-q4.bin"), bytes)?;
        assert!(error(&f).contains("digest differs"));
        let f = Fixture::new()?;
        fs::write(f.root.join("consumer/metadata.json"), b"{}")?;
        assert!(error(&f).contains("digest differs"));
        Ok(())
    }
    #[test]
    fn source_free_loader_rejects_consistently_rehashed_config_and_provenance_tampering(
    ) -> Result<()> {
        fn replace_outer(f: &mut Fixture, outer: &serde_json::Value) -> Result<()> {
            let bytes = serde_json::to_vec(outer)?;
            fs::write(f.root.join("metadata.json"), &bytes)?;
            f.binding.metadata_sha256 = sha256_bytes(&bytes);
            Ok(())
        }
        let mut f = Fixture::new()?;
        let mut nested: serde_json::Value =
            serde_json::from_slice(&fs::read(f.root.join("consumer/metadata.json"))?)?;
        nested["potential"]["heads"] = serde_json::json!(2);
        let bytes = serde_json::to_vec(&nested)?;
        fs::write(f.root.join("consumer/metadata.json"), &bytes)?;
        let mut outer: serde_json::Value =
            serde_json::from_slice(&fs::read(f.root.join("metadata.json"))?)?;
        outer["files"]["consumer/metadata.json"] = serde_json::json!(sha256_bytes(&bytes));
        replace_outer(&mut f, &outer)?;
        assert!(error(&f).contains("component shapes"));
        let mut f = Fixture::new()?;
        let mut outer: serde_json::Value =
            serde_json::from_slice(&fs::read(f.root.join("metadata.json"))?)?;
        outer["source_parameters"]["period.coefficients"]["shape"] = serde_json::json!([1, 999]);
        replace_outer(&mut f, &outer)?;
        assert!(error(&f).contains("provenance shape"));
        let mut f = Fixture::new()?;
        f.binding.identity.tokenizer_sha256 = "e".repeat(64);
        assert!(error(&f).contains("trusted identity"));
        Ok(())
    }
    #[cfg(unix)]
    #[test]
    fn source_free_loader_rejects_payload_symlinks_and_bounded_reads() -> Result<()> {
        let f = Fixture::new()?;
        fs::remove_file(f.root.join("consumer/context-q4.bin"))?;
        std::os::unix::fs::symlink("potential-q4.bin", f.root.join("consumer/context-q4.bin"))?;
        assert!(error(&f).contains("symlink"));
        let f = Fixture::new()?;
        assert!(read_bounded(&f.root.join("tokenizer.json"), 1).is_err());
        Ok(())
    }
    #[test]
    fn canonical_exp_rejects_partial_words_and_numerically_valid_substitution() -> Result<()> {
        let mut bytes = vec![0; CANONICAL_EXP_BYTES];
        bytes[..4].copy_from_slice(&(crate::geometric_read::WEIGHT_ONE as u32).to_le_bytes());
        let values = bytes
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        assert!(crate::geometric_read::NativeGeometricRead::new(1, 1, &values).is_ok());
        assert!(canonical_exp(&bytes).is_err());
        assert!(canonical_exp(&bytes[..bytes.len() - 1]).is_err());
        Ok(())
    }
    #[test]
    fn checked_shared_runtime_rejects_sparse_query_wrong_frame_and_window() -> Result<()> {
        let binding = SourceActionBinding::new(TOKENIZER)?;
        let compiler = SourceEmissionCompiler::new(TOKENIZER)?;
        let view = compiler.compile(&[4])?;
        let c = ContextQ4Config {
            vocab_size: 10,
            heads: 1,
            lanes_per_head: 1,
        };
        let p = PotentialQ4Config {
            heads: 1,
            lanes_per_head: 1,
        };
        let n = NoReadConfig {
            vocabulary: 10,
            heads: 1,
            latent_lanes_per_head: 1,
        };
        let context = NativeContextQ4::new(
            c,
            &vec![
                0;
                c.coefficient_count()
                    .map_err(|e| invalid(e.to_string()))?
                    .div_ceil(2)
            ],
        )
        .map_err(|e| invalid(e.to_string()))?;
        let potential = NativePotentialQ4::new(
            p,
            &vec![
                0;
                p.coefficient_count()
                    .map_err(|e| invalid(e.to_string()))?
                    .div_ceil(2)
            ],
        )
        .map_err(|e| invalid(e.to_string()))?
        .into_native()
        .map_err(|e| invalid(e.to_string()))?;
        let no_read = NativeGeometricNoRead::new(n, &vec![0; n.coefficient_count().div_ceil(2)])
            .map_err(|e| invalid(e.to_string()))?;
        let geometry =
            HistoricalH4Tables::from_bytes(ALGEBRA).map_err(|e| invalid(e.to_string()))?;
        let mut exp = vec![0; crate::geometric_read::EXP_TABLE_LEN];
        exp[0] = crate::geometric_read::WEIGHT_ONE as u32;
        let executor = RealizerExecution {
            context: &context,
            potential_tables: &potential,
            no_read: &no_read,
            geometry: &geometry,
            exp: &exp,
            period: &no_read,
            binding: &binding,
        };
        fn frame(tokens: &[u32]) -> SelectedRecordFrame<'_> {
            SelectedRecordFrame {
                identity: SourceIdentity {
                    record: 7,
                    commit: 9,
                },
                metadata: FrameMetadata {
                    scope: b"unit",
                    entity: &[],
                    relation: 3,
                    view: 0,
                    status: FrameStatus::Found,
                },
                token_ids: tokens,
            }
        }

        assert!(executor.read(frame(&[4]), &view, &[7], &[]).is_err());
        assert!(executor.read(frame(&[5]), &view, &[4], &[]).is_err());
        assert!(executor.read(frame(&[4]), &view, &[], &[]).is_err());
        assert!(executor
            .read(frame(&[4]), &view, &[4], &vec![4; 128])
            .is_err());
        let valid = executor.read(frame(&[4]), &view, &[4], &[])?;
        assert_eq!(
            valid.period_context.tokens,
            view.emitted_token_ids()
                .iter()
                .copied()
                .chain([4])
                .collect::<Vec<_>>()
        );
        assert!(valid
            .actions
            .token_masses
            .iter()
            .any(|token| token.token_id == 4));
        Ok(())
    }
}
