//! Target-free composition of actual full-bank Copy and native Generate.
//!
//! The factual branches consume the same authentic causal ContextQ4Output. An
//! optional continuation factor separately replays query || actual prefix from
//! identity and adds signed-H4 scores for every Generate ID before the common
//! pool. The default field is absent; there is no first-token or phase gate. Generate
//! reads its final retained H4 state and explicit full120 state-utility channel, never observed
//! address roots. All source candidates retain occurrence order/provenance.
//! Only the new Copy+Generate pool supplies emission mass; old Period/Stop
//! reductions retained in the Copy preparation trace are discarded diagnostics
//! and real computational overhead, not extra action mass or a hidden bonus.
//!
//! Frozen Copy operators score every source occurrence. Their legacy local
//! state-logit/earlier four-coordinate credit remains; Generate instead uses
//! full120 temporal utility. This API establishes integration, not learned language,
//! geometric advantage, energy savings or durable memory consumption.

use candle_core::{DType, Device, Tensor};
use sha2::Digest;
use uor_r4_core::native_geometric::learner::{
    geometric_continuation_field::NativeContinuationField,
    native_bank_generate::{AdmittedBank, BankPin, NativeBankGenerator, PinnedBankSnapshot},
};
use uor_r4_integer::{
    geometric_cue_carrier::NativeCueCarrier,
    geometric_prefix_transport::NativePrefixTransport,
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::SourceBankSegment,
    geometric_vocabulary_actions::{NativeVocabularyActions, VocabularyActionTrace},
    h4_tables::{H4Code, ROOT_COUNT},
};

use crate::{
    geometric_context::ContextQ4Output,
    geometric_continuation_learning::{
        ContinuationDeviceLearningOutput, ContinuationLearningOutput, ContinuationLearningWeights,
        PreparedContinuationLearning,
    },
    geometric_generate_learning::{
        preclip_entry_margin_diagnostic, vocabulary_marginal_loss,
        vocabulary_marginal_loss_with_credit, GenerateLearningOutput, GenerateLearningWeights,
        PreclipEntryMarginDiagnostic, PreparedGenerateLearning, VocabularyScoreAdjoint,
    },
    geometric_occurrence_consumer::source_realizer::{
        ComposedCopyBankOutput, PreparedSourceRealizer,
    },
    geometric_read_state_bridge::{
        BridgeLearningOutput, BridgeLearningWeights, PreparedBridgeLearning,
        PreparedCategoricalBridge,
    },
    invalid, Result,
};

pub const CREDIT_SCOPE:&str="same-actual-fullbank-context;retained-H4-onehot120-Generate;full120-temporal-utility-factual-action-carry-and-one-choice-pullback;frozen-native-allsource-Copy-context/cue/prefix-credit-legacy-ambient4;one-common-clipped-fullvocab-token-alias-marginal;no-old-terminals-or-bonus;all-Copy-occurrences-scored-without-selected-winner;local-finite-choice-surrogate-not-global-posterior/3";
pub const NO_SOURCE_CREDIT_SCOPE:&str="actual-causal-history-context;retained-H4-onehot120-Generate;full120-temporal-utility-factual-action-carry-and-one-choice-pullback;full-legal-vocabulary;zero-Copy-occurrences;no-fabricated-source-or-initial-token;local-finite-choice-surrogate-not-global-posterior/2";
pub const PREFIX_TEMPORAL_CREDIT_SCOPE: &str = "same-actual-fullbank-context;Generate-and-exact-prefix-Copy-table-full120-temporal-utility;one-context-choice-pullback;contextual-readout/cue-Copy-credit-legacy-ambient4;all-source-occurrences-no-selected-winner;local-conditional-surrogate-not-global-posterior/4";

/// Shared frozen upstream operators for fixed teacher-prefix continuation fits.
/// Ownership prevents replacing Context/Potential/Generate/bridge/cue/prefix
/// after positions are prepared. This is not an own-prefix sampling cache.
pub struct PreparedFixedContinuationBank {
    native: std::cell::RefCell<NativeBankGenerator>,
    pool: std::cell::RefCell<NativeVocabularyActions>,
    binding: SourceActionBinding,
    device: Device,
    shared_action: bool,
}

/// A privately admitted packet tied to the exact shared upstream owner.
/// Equal Source metadata does not authorize reuse under different sidecars.
pub struct AdmittedFixedContinuationBank {
    owner: std::rc::Rc<PreparedFixedContinuationBank>,
    bank: AdmittedBank,
}

/// A target-free physical bank position prepared by the production native step.
/// The admitted packet remains privately owned and immutable; prefixes are
/// copied, not borrowed from a mutable teacher buffer. No labels enter here.
pub struct FixedContinuationPosition {
    owner: std::rc::Rc<PreparedFixedContinuationBank>,
    bank: std::rc::Rc<AdmittedFixedContinuationBank>,
    actual_prefix_ids: Vec<u32>,
    post_state: Vec<H4Code>,
    local_state: Vec<H4Code>,
    base_generate_q24: Tensor,
    copy_token_ids: Vec<u32>,
    copy_scores_q24: Vec<i64>,
    copy_raw: Option<Tensor>,
    factual_bank_trace_sha256: Option<String>,
    factual_bank_binding_sha256: Option<String>,
    physical_candidates: Vec<uor_r4_integer::geometric_source_realizer::BankCandidateTrace>,
    native_generate_counts:
        uor_r4_core::native_geometric::learner::geometric_generate::GenerateReadCounts,
    encoding_coefficient_reads: u64,
}

pub struct FixedContinuationOutput {
    pub actions: VocabularyActionTrace,
    pub generate_scores_q24: Vec<i64>,
    pub generate_raw_scores: Tensor,
    pub copy_raw_scores: Option<Tensor>,
    pub copy_scores_q24: Vec<i64>,
    pub continuation: ContinuationDeviceLearningOutput,
    /// Compatibility CPU reducer: CUDA downloads the full combined I64 scores.
    /// This is separate from U scorer's zero-download device-output cost.
    pub common_pool_score_download_bytes: usize,
    pub common_pool_anchor_upload_bytes: usize,
    pub common_pool_copy_index_upload_bytes: usize,
    pub common_pool_backend: &'static str,
}

impl PreparedFixedContinuationBank {
    /// The generator must have the authentic all-zero continuation artifact
    /// installed. canonical_exp_bytes are independently authenticated against
    /// the same pinned full table by NativeVocabularyActions::new; arbitrary
    /// caller score vectors or precomputed probability pools are never admitted.
    pub fn new(
        generator: NativeBankGenerator,
        weights: &ContinuationLearningWeights,
        canonical_exp_bytes: &[u8],
    ) -> Result<std::rc::Rc<Self>> {
        if !weights.device().is_cpu() && !weights.device().is_cuda() {
            return Err(invalid("fixed continuation cache supports CPU/CUDA only"));
        }
        let generate = generator.generate_model();
        if weights.native_binding() != generator.source_binding()
            || generate.metadata().tokenizer_sha256 != weights.binding().tokenizer_sha256()
            || generate.metadata().dialogue_protocol != *weights.binding().protocol()
            || generate.vocab_size() != weights.vocab_size()
            || generate.lanes() != weights.lanes()
        {
            return Err(invalid(
                "fixed continuation upstream parent/token binding differs",
            ));
        }
        let zero = if weights.applies_to_copy() {
            NativeContinuationField::compile_shared_action(
                generator.source_binding(),
                generate,
                &vec![0; generate.lanes() * 60],
            )
        } else {
            NativeContinuationField::zeroed(generator.source_binding(), generate)
        }
        .map_err(|e| invalid(e.to_string()))?;
        let zero_bytes = zero.to_bytes().map_err(|e| invalid(e.to_string()))?;
        let zero_sha = format!("{:x}", sha2::Sha256::digest(&zero_bytes));
        if generator.continuation_sha256() != Some(zero_sha.as_str()) {
            return Err(invalid(
                "fixed continuation cache requires authentic zero field",
            ));
        }
        let pool = NativeVocabularyActions::new(weights.binding().clone(), canonical_exp_bytes)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(std::rc::Rc::new(Self {
            native: std::cell::RefCell::new(generator),
            pool: std::cell::RefCell::new(pool),
            binding: weights.binding().clone(),
            device: weights.device().clone(),
            shared_action: weights.applies_to_copy(),
        }))
    }

    /// Admit a packet against the same privately owned frozen parent.
    pub fn admit_bank(
        self: &std::rc::Rc<Self>,
        snapshot: PinnedBankSnapshot,
    ) -> Result<std::rc::Rc<AdmittedFixedContinuationBank>> {
        let native = self
            .native
            .try_borrow()
            .map_err(|_| invalid("fixed continuation upstream already borrowed"))?;
        let bank = native
            .admit_bank(snapshot)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(std::rc::Rc::new(AdmittedFixedContinuationBank {
            owner: self.clone(),
            bank,
        }))
    }

    /// Refresh the960 field snapshot after each optimizer update, using the
    /// actual frozen Generate and parent rather than a caller replacement.
    pub fn prepare_field(
        &self,
        weights: &ContinuationLearningWeights,
    ) -> Result<PreparedContinuationLearning> {
        let native = self
            .native
            .try_borrow()
            .map_err(|_| invalid("fixed continuation upstream already borrowed"))?;
        if !same_binding(weights.binding(), &self.binding)
            || weights.applies_to_copy() != self.shared_action
            || !weights.device().same_device(&self.device)
        {
            return Err(invalid("fixed continuation field binding/device differs"));
        }
        weights.prepare_native(native.source_binding(), native.generate_model())
    }

    /// Prepare all raw scores and local state via the real target-free native
    /// step. The bank was independently admitted by the same native parent.
    pub fn prepare_position(
        self: &std::rc::Rc<Self>,
        bank: std::rc::Rc<AdmittedFixedContinuationBank>,
        actual_prefix_ids: &[u32],
    ) -> Result<FixedContinuationPosition> {
        admit_fixed_owner(self, &bank.owner)?;
        let mut generator = self
            .native
            .try_borrow_mut()
            .map_err(|_| invalid("fixed continuation preparation already borrowed"))?;
        let step = generator
            .step(&bank.bank, actual_prefix_ids)
            .map_err(|e| invalid(e.to_string()))?;
        let witness = step
            .continuation
            .ok_or_else(|| invalid("fixed continuation local witness absent"))?;
        if witness.actual_prefix_tokens != actual_prefix_ids.len()
            || witness.query_tokens == 0
            || witness.state_codes.len() != generator.generate_model().lanes()
            || witness.delta_scores_q24.len() != self.binding.vocab_size()
            || witness.delta_scores_q24.iter().any(|&score| score != 0)
            || step.actual_prefix_ids != actual_prefix_ids
        {
            return Err(invalid(
                "fixed continuation zero/prefix/local witness differs",
            ));
        }
        let mut pool = self
            .pool
            .try_borrow_mut()
            .map_err(|_| invalid("fixed continuation pool already borrowed"))?;
        let replay = pool
            .reduce_trace(
                &step.generate_raw_scores_q24,
                &step.copy_token_ids,
                &step.copy_raw_scores_q24,
            )
            .map_err(|e| invalid(e.to_string()))?;
        if replay != step.actions {
            return Err(invalid(
                "fixed continuation complete zero pool parity differs",
            ));
        }
        let vocab = self.binding.vocab_size();
        let base_generate_q24 =
            Tensor::from_vec(step.generate_raw_scores_q24, vocab, &self.device)?;
        let copy_raw = if step.copy_token_ids.is_empty() {
            None
        } else {
            Some(
                Tensor::from_vec(
                    step.copy_raw_scores_q24
                        .iter()
                        .map(|&v| (v as f64 / 16_777_216.) as f32)
                        .collect::<Vec<_>>(),
                    step.copy_token_ids.len(),
                    &self.device,
                )?
                .detach(),
            )
        };
        // Hash and discard the verbose native replay. Keep only physical
        // occurrence identities and compact provenance/counts in every cache.
        let (factual_bank_trace_sha256, factual_bank_binding_sha256, physical_candidates) =
            if let Some(trace) = step.bank_trace {
                let bytes = serde_json::to_vec(&trace).map_err(|e| invalid(e.to_string()))?;
                let digest = format!("{:x}", sha2::Sha256::digest(&bytes));
                let bank = trace.cue_bank.bank;
                (
                    Some(digest),
                    Some(bank.bank_binding_sha256),
                    bank.candidates,
                )
            } else {
                (None, None, Vec::new())
            };
        Ok(FixedContinuationPosition {
            owner: self.clone(),
            bank,
            actual_prefix_ids: actual_prefix_ids.to_vec(),
            post_state: step.post_state,
            local_state: witness.state_codes,
            base_generate_q24,
            copy_token_ids: step.copy_token_ids,
            copy_scores_q24: step.copy_raw_scores_q24,
            copy_raw,
            factual_bank_trace_sha256,
            factual_bank_binding_sha256,
            physical_candidates,
            native_generate_counts: step.generate_counts,
            encoding_coefficient_reads: witness.encoding_coefficient_reads,
        })
    }
}

impl FixedContinuationPosition {
    pub fn bank_pin(&self) -> &BankPin {
        self.bank.bank.pin()
    }
    pub fn actual_prefix_ids(&self) -> &[u32] {
        &self.actual_prefix_ids
    }
    pub fn post_state(&self) -> &[H4Code] {
        &self.post_state
    }
    pub fn local_state(&self) -> &[H4Code] {
        &self.local_state
    }
    pub fn base_generate_q24(&self) -> &Tensor {
        &self.base_generate_q24
    }
    pub fn copy_token_ids(&self) -> &[u32] {
        &self.copy_token_ids
    }
    pub fn copy_scores_q24(&self) -> &[i64] {
        &self.copy_scores_q24
    }
    pub fn factual_bank_trace_sha256(&self) -> Option<&str> {
        self.factual_bank_trace_sha256.as_deref()
    }
    pub fn factual_bank_binding_sha256(&self) -> Option<&str> {
        self.factual_bank_binding_sha256.as_deref()
    }
    pub fn physical_candidates(
        &self,
    ) -> &[uor_r4_integer::geometric_source_realizer::BankCandidateTrace] {
        &self.physical_candidates
    }
    pub fn native_generate_counts(
        &self,
    ) -> &uor_r4_core::native_geometric::learner::geometric_generate::GenerateReadCounts {
        &self.native_generate_counts
    }
    pub fn encoding_coefficient_reads(&self) -> u64 {
        self.encoding_coefficient_reads
    }
    pub fn source_binding(
        &self,
    ) -> Result<uor_r4_integer::geometric_source_realizer::NativeArtifactBinding> {
        let generator = self
            .owner
            .native
            .try_borrow()
            .map_err(|_| invalid("fixed upstream already borrowed"))?;
        Ok(generator.source_binding().clone())
    }

    /// Recompute U and the complete single pool; cached probabilities, target
    /// masks and winner gates are deliberately absent. Only U masters vary.
    pub fn forward_coefficients_only(
        &self,
        weights: &ContinuationLearningWeights,
        prepared: &PreparedContinuationLearning,
    ) -> Result<FixedContinuationOutput> {
        let generator = self
            .owner
            .native
            .try_borrow()
            .map_err(|_| invalid("fixed continuation upstream already borrowed"))?;
        if weights.applies_to_copy() != self.owner.shared_action {
            return Err(invalid("fixed continuation action policy changed"));
        }
        admit_fixed_continuation_refresh(
            weights,
            prepared,
            &self.owner.binding,
            generator.source_binding(),
            generator.generate_sha256(),
            generator.generate_model().metadata(),
            &self.owner.device,
        )?;
        let delta =
            weights.forward_prepared_coefficients_only_on_device(prepared, &self.local_state)?;
        let (scores, raw) = join_fixed_continuation_scores(&self.base_generate_q24, &delta)?;
        let (copy_scores, copy_raw) = if prepared.native.applies_to_copy() {
            join_shared_continuation_copy(
                &self.copy_scores_q24,
                &self.copy_token_ids,
                self.copy_raw.as_ref(),
                &delta.delta_scores_q24,
                &delta.delta_raw_scores,
            )?
        } else {
            (self.copy_scores_q24.clone(), self.copy_raw.clone())
        };
        let actions = self
            .owner
            .pool
            .try_borrow_mut()
            .map_err(|_| invalid("fixed continuation pool already borrowed"))?
            .reduce_trace(&scores, &self.copy_token_ids, &copy_scores)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(FixedContinuationOutput {
            actions,
            generate_scores_q24: scores,
            generate_raw_scores: raw,
            copy_raw_scores: copy_raw,
            copy_scores_q24: copy_scores,
            continuation: delta,
            common_pool_score_download_bytes: if self.owner.device.is_cuda() {
                8 * (self.owner.binding.vocab_size()
                    + usize::from(prepared.native.applies_to_copy()) * self.copy_token_ids.len())
            } else {
                0
            },
            common_pool_anchor_upload_bytes: if self.owner.device.is_cuda() {
                4 * (self.owner.binding.vocab_size()
                    + usize::from(prepared.native.applies_to_copy()) * self.copy_token_ids.len())
            } else {
                0
            },
            common_pool_copy_index_upload_bytes: if self.owner.device.is_cuda()
                && prepared.native.applies_to_copy()
            {
                4 * self.copy_token_ids.len()
            } else {
                0
            },
            common_pool_backend: "cpu-authenticated-native-alias-reducer",
        })
    }
}
impl FixedContinuationOutput {
    /// Labels enter only after all native actions and their common denominator.
    pub fn loss_with_credit(&self, target: u32, credit: VocabularyScoreAdjoint) -> Result<Tensor> {
        vocabulary_marginal_loss_with_credit(
            &self.actions,
            &self.generate_raw_scores,
            self.copy_raw_scores.as_ref(),
            target,
            credit,
        )
    }
}

fn admit_fixed_owner<T>(expected: &std::rc::Rc<T>, actual: &std::rc::Rc<T>) -> Result<()> {
    if !std::rc::Rc::ptr_eq(expected, actual) {
        return Err(invalid(
            "fixed continuation bank belongs to different upstream owner",
        ));
    }
    Ok(())
}

fn admit_fixed_continuation_refresh(
    weights: &ContinuationLearningWeights,
    prepared: &PreparedContinuationLearning,
    binding: &SourceActionBinding,
    parent: &uor_r4_integer::geometric_source_realizer::NativeArtifactBinding,
    generate_sha: &str,
    generate_metadata: &uor_r4_core::native_geometric::learner::geometric_generate::GenerateMetadata,
    device: &Device,
) -> Result<()> {
    prepared.validate_binding(parent)?;
    if !same_binding(weights.binding(), binding)
        || !weights.device().same_device(device)
        || prepared.field_generate_sha() != generate_sha
        || prepared.generate.metadata() != generate_metadata
    {
        return Err(invalid(
            "fixed continuation refreshed field/upstream identity differs",
        ));
    }
    Ok(())
}

fn join_fixed_continuation_scores(
    base: &Tensor,
    delta: &ContinuationDeviceLearningOutput,
) -> Result<(Vec<i64>, Tensor)> {
    if base.dtype() != DType::I64
        || base.dims() != delta.delta_scores_q24.dims()
        || !base.device().same_device(delta.delta_scores_q24.device())
    {
        return Err(invalid(
            "fixed continuation score join dimensions/device differ",
        ));
    }
    let combined = (base + &delta.delta_scores_q24)?;
    let scores = combined.to_vec1::<i64>()?;
    // Exact legacy F64->F32 native anchor; only the U zero-valued correction
    // supplies derivatives. This adapter still stages4V bytes on CUDA.
    let anchor = Tensor::from_vec(
        scores
            .iter()
            .map(|&v| (v as f64 / 16_777_216.) as f32)
            .collect::<Vec<_>>(),
        scores.len(),
        base.device(),
    )?
    .detach();
    let raw = (&anchor + (&delta.delta_raw_scores - delta.delta_raw_scores.detach())?)?;
    Ok((scores, raw))
}

/// Shared token field is gathered once for EACH physical Copy occurrence.
/// Preserve old upstream Copy credit plus U credit under the exact combined
/// native anchor, rather than rounding an independently added F32 delta.
fn join_shared_continuation_copy(
    base: &[i64],
    ids: &[u32],
    base_raw: Option<&Tensor>,
    delta_q24: &Tensor,
    delta_raw: &Tensor,
) -> Result<(Vec<i64>, Option<Tensor>)> {
    if base.len() != ids.len()
        || delta_q24.dtype() != DType::I64
        || delta_q24.rank() != 1
        || delta_raw.dims() != delta_q24.dims()
        || !delta_raw.device().same_device(delta_q24.device())
    {
        return Err(invalid(
            "shared continuation Copy delta shape/device differs",
        ));
    }
    if ids.is_empty() {
        if base_raw.is_some() {
            return Err(invalid("empty Copy unexpectedly has graph"));
        }
        return Ok((Vec::new(), None));
    }
    let old = base_raw.ok_or_else(|| invalid("shared continuation Copy graph absent"))?;
    if old.dims() != [ids.len()]
        || old.dtype() != DType::F32
        || !old.device().same_device(delta_raw.device())
        || ids.iter().any(|&id| id as usize >= delta_q24.elem_count())
    {
        return Err(invalid("shared continuation Copy IDs/graph differ"));
    }
    let indices = Tensor::from_vec(ids.to_vec(), ids.len(), delta_raw.device())?;
    let correction = delta_q24.index_select(&indices, 0)?.to_vec1::<i64>()?;
    let scores = base
        .iter()
        .zip(correction)
        .map(|(&a, b)| {
            a.checked_add(b)
                .ok_or_else(|| invalid("shared continuation Copy score overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let anchor = Tensor::from_vec(
        scores
            .iter()
            .map(|&v| (v as f64 / 16_777_216.) as f32)
            .collect::<Vec<_>>(),
        scores.len(),
        old.device(),
    )?
    .detach();
    let gathered = delta_raw.index_select(&indices, 0)?;
    let graph = ((&anchor + (old - old.detach())?)? + (&gathered - gathered.detach())?)?;
    Ok((scores, Some(graph)))
}

pub struct PreparedBankGenerate<'a, 'source> {
    realizer: &'a PreparedSourceRealizer<'source>,
    generate: &'a GenerateLearningWeights,
    prepared_generate: &'a PreparedGenerateLearning,
    pool: NativeVocabularyActions,
    prefix_temporal_utility: bool,
    read_state_bridge: Option<ReadStateBridge<'a>>,
    read_selector_credit: bool,
    continuation: Option<ContinuationBranch<'a>>,
}

#[derive(Clone, Copy)]
struct ContinuationBranch<'a> {
    weights: &'a ContinuationLearningWeights,
    prepared: &'a PreparedContinuationLearning,
    context_credit: bool,
}

/// Separate local replay and its cost/credit witness, never a supplied answer record.
pub struct BankContinuationOutput {
    pub token_ids: Vec<u32>,
    pub local_context: ContextQ4Output,
    pub final_state_codes: Vec<H4Code>,
    pub field: ContinuationLearningOutput,
}

/// One explicitly selected offline pullback for the same native bridge operation.
#[derive(Clone, Copy)]
enum ReadStateBridge<'a> {
    Legacy(&'a BridgeLearningWeights, &'a PreparedBridgeLearning),
    Categorical(&'a PreparedCategoricalBridge),
}
impl ReadStateBridge<'_> {
    fn categorical(&self) -> bool {
        matches!(self, Self::Categorical(_))
    }
    fn native(
        &self,
    ) -> &uor_r4_core::native_geometric::learner::geometric_read_state_bridge::NativeGeometricReadStateBridge{
        match self {
            Self::Legacy(_, prepared) => &prepared.native,
            Self::Categorical(prepared) => prepared.native(),
        }
    }
    fn forward(
        &self,
        query: &[H4Code],
        source: &[H4Code],
        query_choices: &Tensor,
        source_choices: &Tensor,
    ) -> Result<BridgeLearningOutput> {
        match self {
            Self::Legacy(weights, prepared) => {
                weights.forward_prepared(prepared, query, source, query_choices, source_choices)
            }
            Self::Categorical(prepared) => {
                prepared.forward(query, source, query_choices, source_choices)
            }
        }
    }
}
fn configure_bridge<'a>(
    slot: &mut Option<ReadStateBridge<'a>>,
    bridge: ReadStateBridge<'a>,
) -> Result<()> {
    if slot
        .as_ref()
        .is_some_and(|old| old.categorical() != bridge.categorical())
    {
        return Err(invalid(
            "legacy and categorical bank read-state bridges are mutually exclusive",
        ));
    }
    *slot = Some(bridge);
    Ok(())
}
fn admit_categorical_bridge(
    bridge: &PreparedCategoricalBridge,
    binding: &SourceActionBinding,
    lanes: usize,
    device: &candle_core::Device,
) -> Result<()> {
    let metadata = bridge.native().metadata();
    if metadata.tokenizer_sha256 != binding.tokenizer_sha256()
        || metadata.dialogue_protocol != *binding.protocol()
        || bridge.native().lanes() != lanes
        || !bridge.device().same_device(device)
    {
        return Err(invalid(
            "categorical bank read-state bridge binding/lanes/device differs",
        ));
    }
    Ok(())
}

/// Target-free result. All native scores and the complete legal action pool
/// exist before `loss` may inspect a target. Context ownership is unambiguous:
/// source banks keep it inside Copy; no-source continuations retain it directly.
pub struct BankGenerateOutput {
    pub actions: VocabularyActionTrace,
    pub generate: GenerateLearningOutput,
    pub copy: Option<ComposedCopyBankOutput>,
    pub no_source_context: Option<ContextQ4Output>,
    pub causal_token_ids: Vec<u32>,
    pub final_state_codes: Vec<H4Code>,
    pub copy_token_ids: Vec<u32>,
    pub copy_scores_q24: Vec<i64>,
    pub credit_scope: &'static str,
    /// Hard occurrence ordinal and selected-source transport, absent on the legacy/no-source path.
    pub read_state_bridge: Option<(usize, BridgeLearningOutput)>,
    pub continuation: Option<BankContinuationOutput>,
}

fn same_binding(a: &SourceActionBinding, b: &SourceActionBinding) -> bool {
    a.tokenizer_sha256() == b.tokenizer_sha256()
        && a.protocol() == b.protocol()
        && a.vocab_size() == b.vocab_size()
        && a.eos_token_id() == b.eos_token_id()
        && a.period_token_id() == b.period_token_id()
}

impl<'a, 'source> PreparedBankGenerate<'a, 'source> {
    pub fn new(
        realizer: &'a PreparedSourceRealizer<'source>,
        generate: &'a GenerateLearningWeights,
        prepared_generate: &'a PreparedGenerateLearning,
        canonical_exp_bytes: &[u8],
    ) -> Result<Self> {
        if !same_binding(realizer.binding(), generate.binding())
            || prepared_generate.native.metadata().tokenizer_sha256
                != generate.binding().tokenizer_sha256()
            || prepared_generate.native.metadata().dialogue_protocol
                != *generate.binding().protocol()
            || prepared_generate.native.vocab_size() != generate.vocab_size()
            || prepared_generate.native.lanes() != generate.lanes()
        {
            return Err(invalid(
                "bank Generate tokenizer/protocol/native decoder binding differs",
            ));
        }
        let pool = NativeVocabularyActions::new(generate.binding().clone(), canonical_exp_bytes)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(Self {
            realizer,
            generate,
            prepared_generate,
            pool,
            prefix_temporal_utility: false,
            read_state_bridge: None,
            read_selector_credit: true,
            continuation: None,
        })
    }

    /// Install a separately bound factor. The first rung credits only its shared
    /// coefficients with frozen context/prototypes; callers may explicitly enable
    /// the retained-state utility below. Prepare again after each optimizer update.
    pub fn with_continuation_field(
        mut self,
        weights: &'a ContinuationLearningWeights,
        prepared: &'a PreparedContinuationLearning,
    ) -> Result<Self> {
        if !same_binding(weights.binding(), self.generate.binding())
            || weights.lanes() != self.generate.lanes()
            || !weights.device().same_device(self.generate.device())
            || !prepared.device().same_device(self.generate.device())
        {
            return Err(invalid("bank continuation binding/lanes/device differs"));
        }
        prepared.validate_binding(&self.realizer.execution_binding()?)?;
        prepared.validate_generate(&self.prepared_generate.native)?;
        self.continuation = Some(ContinuationBranch {
            weights,
            prepared,
            context_credit: false,
        });
        Ok(self)
    }

    /// Optional local full120 conditional pullback; it is not prototype credit
    /// or a derivative of the global recurrent posterior.
    pub fn with_continuation_context_credit(mut self, enabled: bool) -> Result<Self> {
        let branch = self
            .continuation
            .as_mut()
            .ok_or_else(|| invalid("bank continuation field must be installed first"))?;
        branch.context_credit = enabled;
        Ok(self)
    }

    fn apply_continuation(
        &self,
        generated: &mut GenerateLearningOutput,
        query: &[u32],
        actual_prefix: &[u32],
    ) -> Result<Option<BankContinuationOutput>> {
        let Some(branch) = self.continuation else {
            return Ok(None);
        };
        let token_ids: Vec<u32> = query.iter().chain(actual_prefix).copied().collect();
        if query.is_empty()
            || token_ids
                .iter()
                .any(|id| !self.generate.binding().admits_token(*id))
        {
            return Err(invalid(
                "bank continuation query/prefix token IDs empty/invalid",
            ));
        }
        let local_context = self.realizer.context_output(&token_ids, false)?;
        let (states, choices) = final_retained_state(&local_context, self.generate.lanes())?;
        let field = if branch.context_credit {
            branch
                .weights
                .forward_prepared_state_choices(branch.prepared, &states, &choices)?
        } else {
            branch
                .weights
                .forward_prepared_coefficients_only(branch.prepared, &states)?
        };
        add_continuation_q24(&mut generated.scores_q24, &field.delta_scores_q24)?;
        generated.raw_scores = (&generated.raw_scores + &field.delta_raw_scores)?;
        // One final clip of the combined raw score; each branch is not clipped separately.
        generated.clipped_scores = generated.raw_scores.clamp(-8f32, 8f32)?;
        Ok(Some(BankContinuationOutput {
            token_ids,
            local_context,
            final_state_codes: states,
            field,
        }))
    }

    /// Opt into exact prefix table utility on the full retained-state carrier.
    /// Native forward scores are unchanged; cue/readout credit remains legacy.
    pub fn with_prefix_temporal_utility(mut self, enabled: bool) -> Self {
        self.prefix_temporal_utility = enabled;
        self
    }

    /// Opt into a target-free hard read followed by signed H4 state transport.
    /// Offline selector credit uses detached alternative poststates; only the
    /// factual selected bridge receives parameter and context-state adjoints.
    pub fn with_read_state_bridge(
        mut self,
        weights: &'a BridgeLearningWeights,
        prepared: &'a PreparedBridgeLearning,
    ) -> Result<Self> {
        if !same_binding(weights.binding(), self.generate.binding())
            || weights.lanes() != self.generate.lanes()
            || !weights.device().same_device(self.generate.device())
        {
            return Err(invalid(
                "bank read-state bridge binding/lanes/device differs",
            ));
        }
        configure_bridge(
            &mut self.read_state_bridge,
            ReadStateBridge::Legacy(weights, prepared),
        )?;
        Ok(self)
    }

    /// Opt into the authenticated frozen categorical map's conditional query
    /// and source full120 pullback. No bridge variables are learned. Hard route,
    /// native transport, alternatives and the complete alias pool are unchanged.
    /// This is an offline conditional surrogate, not a derivative of argmax.
    /// It cannot be combined with `with_read_state_bridge` in either call order.
    pub fn with_categorical_read_state_bridge(
        mut self,
        prepared: &'a PreparedCategoricalBridge,
    ) -> Result<Self> {
        admit_categorical_bridge(
            prepared,
            self.generate.binding(),
            self.generate.lanes(),
            self.generate.device(),
        )?;
        configure_bridge(
            &mut self.read_state_bridge,
            ReadStateBridge::Categorical(prepared),
        )?;
        Ok(self)
    }

    /// Offline adjoint control only. Native selection, transport and scores are
    /// identical with either setting; disabling it removes only selector credit.
    /// Factual bridge/Generate and direct emission-pool adjoints remain.
    pub fn with_read_selector_credit(mut self, enabled: bool) -> Self {
        self.read_selector_credit = enabled;
        self
    }

    /// Query/prefix/context are causal state inputs, never source candidates.
    /// The old Copy preparer intentionally rejects empty/context-only banks;
    /// use explicit `forward_no_source` instead of inventing a Source record.
    pub fn forward_bank(
        &mut self,
        segments: &[SourceBankSegment<'_>],
        query: &[u32],
        actual_prefix: &[u32],
        cue: &NativeCueCarrier<'_>,
        prefix: &NativePrefixTransport<'_>,
    ) -> Result<BankGenerateOutput> {
        let mut copy = if self.prefix_temporal_utility {
            self.realizer
                .forward_bank_composed_copy_with_prefix_utility(
                    segments,
                    query,
                    actual_prefix,
                    cue,
                    prefix,
                )?
        } else {
            self.realizer
                .forward_bank_composed_copy(segments, query, actual_prefix, cue, prefix)?
        };
        let bank = &copy.trace.cue_bank.bank;
        let time = copy.context.trace.time;
        if copy.context.trace.batch != 1
            || bank.context.tokens.len() != time
            || bank.context.heads != copy.context.trace.heads
            || bank.context.lanes_per_head != copy.context.trace.lanes_per_head
            || bank.context.states != copy.context.trace.states
            || bank.context.actions != copy.context.trace.actions
            || bank.heads.len() != copy.context.trace.heads
            || bank
                .heads
                .iter()
                .any(|h| h.scores_q24.len() != bank.candidates.len())
            || bank.candidates.is_empty()
            || copy.copy_raw.dims() != [bank.candidates.len()]
            || !copy.copy_raw.device().same_device(self.generate.device())
        {
            return Err(invalid(
                "bank Generate Copy/context replay shape/device differs",
            ));
        }
        let mut ids = Vec::with_capacity(bank.candidates.len());
        let mut scores = Vec::with_capacity(bank.candidates.len());
        let mut starts = Vec::with_capacity(segments.len());
        let mut position = 0usize;
        for segment in segments {
            starts.push(position);
            let length = match segment {
                SourceBankSegment::Source { view, .. } => view.emitted_token_ids().len(),
                SourceBankSegment::Context { token_ids, .. } => token_ids.len(),
            };
            position = position
                .checked_add(length)
                .ok_or_else(|| invalid("bank segment position overflow"))?;
        }
        for (j, candidate) in bank.candidates.iter().enumerate() {
            let SourceBankSegment::Source { frame, view, event } = segments
                .get(candidate.segment_index)
                .ok_or_else(|| invalid("bank candidate source segment absent"))?
            else {
                return Err(invalid("bank candidate points into context segment"));
            };
            let offset = usize::try_from(candidate.occurrence.token_offset)
                .map_err(|e| invalid(e.to_string()))?;
            if candidate.event != *event
                || candidate.occurrence.record != frame.identity.record
                || candidate.occurrence.commit != frame.identity.commit
                || view.emitted_token_ids().get(offset) != Some(&candidate.occurrence.token_id)
                || starts[candidate.segment_index].checked_add(offset)
                    != Some(candidate.context_position)
            {
                return Err(invalid("bank candidate occurrence provenance differs"));
            }
            if candidate.bank_index != j
                || candidate.context_position >= time
                || bank.context.tokens[candidate.context_position] != candidate.occurrence.token_id
                || !self
                    .generate
                    .binding()
                    .admits_token(candidate.occurrence.token_id)
            {
                return Err(invalid(
                    "bank Generate candidate occurrence/token order differs",
                ));
            }
            let score = bank.heads.iter().try_fold(0i64, |sum, head| {
                let value = head
                    .scores_q24
                    .get(j)
                    .ok_or_else(|| invalid("bank Generate Copy head length differs"))?;
                sum.checked_add(*value)
                    .ok_or_else(|| invalid("bank Generate Copy head sum overflow"))
            })?;
            ids.push(candidate.occurrence.token_id);
            scores.push(score);
        }
        let causal_token_ids = bank.context.tokens.clone();
        let (mut states, mut logits) = final_retained_state(&copy.context, self.generate.lanes())?;
        let read_state_bridge = if let Some(prepared) = self.read_state_bridge {
            let selected = hard_read_index(&scores)?;
            let candidate = &bank.candidates[selected];
            let (source, source_choices) = retained_state_at(
                &copy.context,
                candidate.context_position,
                self.generate.lanes(),
            )?;
            let bridge = prepared.forward(&states, &source, &logits, &source_choices)?;
            let mut alternatives = Vec::with_capacity(bank.candidates.len());
            for candidate in &bank.candidates {
                let (source, _) = retained_state_at(
                    &copy.context,
                    candidate.context_position,
                    self.generate.lanes(),
                )?;
                let mut post = vec![H4Code::IDENTITY; states.len()];
                let mut actions = post.clone();
                let mut action_scores = vec![0i64; states.len() * ROOT_COUNT];
                let mut counts = Default::default();
                prepared
                    .native()
                    .apply_into(
                        &states,
                        &source,
                        &mut post,
                        &mut actions,
                        &mut action_scores,
                        &mut counts,
                    )
                    .map_err(|e| invalid(e.to_string()))?;
                alternatives.push(post);
            }
            logits = if self.read_selector_credit {
                add_selector_credit(
                    &bridge.state_choices,
                    &copy.copy_raw,
                    &alternatives,
                    selected,
                )?
            } else {
                bridge.state_choices.clone()
            };
            states = bridge.post_state_codes.clone();
            Some((selected, bridge))
        } else {
            None
        };
        let mut generated = self.generate.forward_prepared_state_choices(
            self.prepared_generate,
            &states,
            &logits,
        )?;
        let continuation = self.apply_continuation(&mut generated, query, actual_prefix)?;
        if !ids.is_empty()
            && self
                .continuation
                .is_some_and(|b| b.prepared.native.applies_to_copy())
        {
            let field = &continuation
                .as_ref()
                .ok_or_else(|| invalid("shared continuation witness absent"))?
                .field;
            let hard = Tensor::from_vec(
                field.delta_scores_q24.clone(),
                field.delta_scores_q24.len(),
                field.delta_raw_scores.device(),
            )?;
            let (adjusted, graph) = join_shared_continuation_copy(
                &scores,
                &ids,
                Some(&copy.copy_raw),
                &hard,
                &field.delta_raw_scores,
            )?;
            scores = adjusted;
            copy.copy_raw =
                graph.ok_or_else(|| invalid("shared continuation Copy graph absent"))?;
        }
        let actions = self
            .pool
            .reduce_trace(&generated.scores_q24, &ids, &scores)
            .map_err(|e| invalid(e.to_string()))?;
        Ok(BankGenerateOutput {
            actions,
            generate: generated,
            copy: Some(copy),
            no_source_context: None,
            causal_token_ids,
            final_state_codes: states,
            copy_token_ids: ids,
            copy_scores_q24: scores,
            read_state_bridge,
            continuation,
            credit_scope: if self
                .read_state_bridge
                .is_some_and(|bridge| bridge.categorical())
            {
                if self.read_selector_credit {
                    "hard-native-allbank-occurrence-read;fixed-authenticated-categorical-map;conditional-query-and-source-full120-pushforward;many-to-one-index-add;no-action-softmax-coefficient-credit-or-extra-query-carry;detached-contrast-poststate-selector-credit;local-decoder-state-sensitivity-not-candidate-loss;complete-token-alias-pool/1"
                } else {
                    "hard-native-allbank-occurrence-read;fixed-authenticated-categorical-map;conditional-query-and-source-full120-pushforward;many-to-one-index-add;no-action-softmax-coefficient-credit-or-extra-query-carry;selector-adjoint-disabled;complete-token-alias-pool/1"
                }
            } else if self.read_state_bridge.is_some() && !self.read_selector_credit {
                "hard-native-allbank-occurrence-read;signed-H4-selected-source-state-transport;selector-adjoint-disabled;factual-bridge-and-emission-pool-credit/1"
            } else if self.read_state_bridge.is_some() {
                "hard-native-allbank-occurrence-read;signed-H4-selected-source-state-transport;detached-contrast-poststate-selector-credit;local-decoder-state-sensitivity-not-candidate-loss/1"
            } else if self.prefix_temporal_utility {
                PREFIX_TEMPORAL_CREDIT_SCOPE
            } else {
                CREDIT_SCOPE
            },
        })
    }

    /// Ordinary causal continuation with zero admitted Copy occurrences. IDs
    /// must already contain the actual history/query/prefix in caller order.
    /// Empty input is rejected; there is no fabricated identity/source token.
    pub fn forward_no_source(&mut self, causal_ids: &[u32]) -> Result<BankGenerateOutput> {
        if self.continuation.is_some() {
            return Err(invalid(
                "configured continuation requires explicit query/prefix boundary",
            ));
        }
        self.forward_no_source_inner(causal_ids, &[], &[])
    }

    /// No-source continuation with a declared causal query/prefix suffix. The
    /// boundary is supplied by the caller, not inferred from semantic tokens.
    pub fn forward_no_source_with_query(
        &mut self,
        causal_ids: &[u32],
        query: &[u32],
        actual_prefix: &[u32],
    ) -> Result<BankGenerateOutput> {
        validate_query_suffix(causal_ids, query, actual_prefix)?;
        self.forward_no_source_inner(causal_ids, query, actual_prefix)
    }

    fn forward_no_source_inner(
        &mut self,
        causal_ids: &[u32],
        query: &[u32],
        actual_prefix: &[u32],
    ) -> Result<BankGenerateOutput> {
        if causal_ids.is_empty()
            || causal_ids
                .iter()
                .any(|id| !self.generate.binding().admits_token(*id))
        {
            return Err(invalid(
                "bank Generate no-source causal token IDs are empty/invalid",
            ));
        }
        let context = self.realizer.context_output(causal_ids, false)?;
        let (states, logits) = final_retained_state(&context, self.generate.lanes())?;
        let mut generated = self.generate.forward_prepared_state_choices(
            self.prepared_generate,
            &states,
            &logits,
        )?;
        let continuation = self.apply_continuation(&mut generated, query, actual_prefix)?;
        let actions = self
            .pool
            .reduce_trace(&generated.scores_q24, &[], &[])
            .map_err(|e| invalid(e.to_string()))?;
        Ok(BankGenerateOutput {
            actions,
            generate: generated,
            copy: None,
            no_source_context: Some(context),
            causal_token_ids: causal_ids.to_vec(),
            final_state_codes: states,
            copy_token_ids: vec![],
            copy_scores_q24: vec![],
            credit_scope: NO_SOURCE_CREDIT_SCOPE,
            read_state_bridge: None,
            continuation,
        })
    }
}

impl BankGenerateOutput {
    pub fn context(&self) -> Result<&ContextQ4Output> {
        match (&self.copy, &self.no_source_context) {
            (Some(copy), None) => Ok(&copy.context),
            (None, Some(context)) => Ok(context),
            _ => Err(invalid("bank Generate context ownership differs")),
        }
    }
    /// Separate opt-in offline diagnostic; default loss/native forward are unchanged.
    /// Caller must establish that this output is an entry (empty actual prefix).
    pub fn preclip_entry_margin_diagnostic(
        &self,
        target: u32,
    ) -> Result<PreclipEntryMarginDiagnostic> {
        preclip_entry_margin_diagnostic(
            &self.actions,
            &self.generate.raw_scores,
            self.copy.as_ref().map(|c| &c.copy_raw),
            target,
        )
    }
    /// Explicit offline score-adjoint comparison; default loss is unchanged.
    pub fn loss_with_credit(&self, target: u32, credit: VocabularyScoreAdjoint) -> Result<Tensor> {
        vocabulary_marginal_loss_with_credit(
            &self.actions,
            &self.generate.raw_scores,
            self.copy.as_ref().map(|c| &c.copy_raw),
            target,
            credit,
        )
    }
    /// Target enters only after the native score vectors and all legal actions
    /// have been produced. Generate and Copy aliases share the same marginal.
    pub fn loss(&self, target: u32) -> Result<Tensor> {
        vocabulary_marginal_loss(
            &self.actions,
            &self.generate.raw_scores,
            self.copy.as_ref().map(|c| &c.copy_raw),
            target,
        )
    }
}

fn validate_query_suffix(causal: &[u32], query: &[u32], actual: &[u32]) -> Result<()> {
    let suffix: Vec<u32> = query.iter().chain(actual).copied().collect();
    if query.is_empty() || !causal.ends_with(&suffix) {
        return Err(invalid(
            "no-source continuation query/prefix is not a causal suffix",
        ));
    }
    Ok(())
}

fn add_continuation_q24(scores: &mut [i64], delta: &[i64]) -> Result<()> {
    if scores.len() != delta.len() {
        return Err(invalid("bank continuation score shape differs"));
    }
    for (score, add) in scores.iter_mut().zip(delta) {
        *score = score
            .checked_add(*add)
            .ok_or_else(|| invalid("bank continuation score overflow"))?;
    }
    Ok(())
}

fn hard_read_index(scores: &[i64]) -> Result<usize> {
    let mut winner = 0;
    let mut best = *scores
        .first()
        .ok_or_else(|| invalid("hard read has no candidates"))?;
    for (i, &score) in scores.iter().enumerate().skip(1) {
        if score > best {
            best = score;
            winner = i;
        }
    }
    Ok(winner)
}

/// Alternative states are host-authenticated native results. Their detached
/// contrasts carry only selector-score adjoints, never bridge/state adjoints.
fn add_selector_credit(
    factual: &Tensor,
    scores: &Tensor,
    alternatives: &[Vec<H4Code>],
    selected: usize,
) -> Result<Tensor> {
    let selected_state = alternatives
        .get(selected)
        .ok_or_else(|| invalid("selector factual ordinal absent"))?;
    let lanes = selected_state.len();
    if lanes == 0
        || factual.dims() != [lanes, ROOT_COUNT]
        || scores.dims() != [alternatives.len()]
        || alternatives.iter().any(|s| s.len() != lanes)
        || !factual.device().same_device(scores.device())
    {
        return Err(invalid("selector carrier shape/device differs"));
    }
    let mut contrasts = Vec::with_capacity(alternatives.len() * lanes * ROOT_COUNT);
    for state in alternatives {
        for (code, actual) in state.iter().zip(selected_state) {
            for r in 0..ROOT_COUNT {
                contrasts.push(
                    f32::from(u8::from(r == usize::from(code.index())))
                        - f32::from(u8::from(r == usize::from(actual.index()))),
                );
            }
        }
    }
    let basis = Tensor::from_vec(
        contrasts,
        (alternatives.len(), lanes * ROOT_COUNT),
        scores.device(),
    )?
    .detach();
    let p = candle_nn::ops::softmax(scores, 0)?.reshape((alternatives.len(), 1))?;
    let mean = basis
        .broadcast_mul(&p)?
        .sum(0)?
        .reshape((lanes, ROOT_COUNT))?;
    Ok((factual + (&mean - mean.detach())?)?)
}

fn final_retained_state(context: &ContextQ4Output, lanes: usize) -> Result<(Vec<H4Code>, Tensor)> {
    let at = context
        .trace
        .time
        .checked_sub(1)
        .ok_or_else(|| invalid("bank context empty"))?;
    retained_state_at(context, at, lanes)
}

fn retained_state_at(
    context: &ContextQ4Output,
    at: usize,
    lanes: usize,
) -> Result<(Vec<H4Code>, Tensor)> {
    let t = &context.trace;
    let width = t
        .heads
        .checked_mul(t.lanes_per_head)
        .ok_or_else(|| invalid("bank Generate lane width overflow"))?;
    if t.batch != 1
        || t.time == 0
        || at >= t.time
        || width != lanes
        || t.states.len() != t.time
        || t.states.iter().any(|s| s.len() != width)
        || context.state_choices.dims() != [1, t.time, t.heads, t.lanes_per_head, ROOT_COUNT]
        || context.latent_roots.dims() != [1, t.time, t.heads, t.lanes_per_head, 4]
        || context.state_choices.dtype() != DType::F32
        || !context
            .state_choices
            .device()
            .same_device(context.latent_roots.device())
    {
        return Err(invalid(
            "bank Generate actual retained context shape differs",
        ));
    }
    let state = t
        .states
        .get(at)
        .ok_or_else(|| invalid("bank Generate retained final state absent"))?
        .iter()
        .map(|r| H4Code::try_from(*r).map_err(|e| invalid(e.to_string())))
        .collect::<Result<Vec<_>>>()?;
    let logits = context
        .state_choices
        .narrow(1, at, 1)?
        .reshape((width, ROOT_COUNT))?
        .contiguous()?;
    Ok((state, logits))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_context::NativeContextTrace;
    use candle_core::{Device, Var};
    use uor_r4_integer::{
        geometric_no_read::CANONICAL_BASIS_Q25, geometric_potential::AddressLane,
    };
    #[test]
    fn fixed_continuation_admission_rejects_equal_payload_different_owner() -> Result<()> {
        // Same metadata/payload value is not the same sidecar-owning admission.
        let first = std::rc::Rc::new("identical source metadata");
        let second = std::rc::Rc::new("identical source metadata");
        admit_fixed_owner(&first, &first.clone())?;
        assert!(admit_fixed_owner(&first, &second).is_err());
        Ok(())
    }

    fn shared_copy_join_parity(
        device: &Device,
    ) -> Result<(
        VocabularyActionTrace,
        Vec<i64>,
        Vec<f32>,
        Vec<f32>,
        Vec<f32>,
    )> {
        let binding =
            SourceActionBinding::new(BRIDGE_TOK.as_bytes()).map_err(|e| invalid(e.to_string()))?;
        // Four q=+/-7 lanes at shift22 admit +/-7 score units. Target4 has noCopy.
        let delta = Var::from_vec(vec![0f32, 0., 0., -7., 7.], 5, device)?;
        let hard = Tensor::from_vec(vec![0i64, 0, 0, -(7 << 24), 7 << 24], 5, device)?;
        let old = Var::from_vec(vec![9f32, 9.], 2, device)?;
        let (copy, graph) = join_shared_continuation_copy(
            &[9 << 24, 9 << 24],
            &[3, 3],
            Some(old.as_tensor()),
            &hard,
            delta.as_tensor(),
        )?;
        assert_eq!(copy, vec![2 << 24, 2 << 24]);
        let graph = graph.ok_or_else(|| invalid("testCopygraphabsent"))?;
        let grads = graph.sum_all()?.backward()?;
        assert_eq!(
            grads
                .get(delta.as_tensor())
                .ok_or_else(|| invalid("testUgradientabsent"))?
                .to_vec1::<f32>()?,
            vec![0., 0., 0., 2., 0.]
        );
        assert_eq!(
            grads
                .get(old.as_tensor())
                .ok_or_else(|| invalid("testupstreamgradientabsent"))?
                .to_vec1::<f32>()?,
            vec![1., 1.]
        );
        let gg = [0, 0, 0, -(7 << 24), 7 << 24];
        let mut pool = NativeVocabularyActions::new(binding, &fixed_exp_bytes())
            .map_err(|e| invalid(e.to_string()))?;
        let legacy = pool
            .reduce_trace(&gg, &[3, 3], &[9 << 24, 9 << 24])
            .map_err(|e| invalid(e.to_string()))?;
        let shared = pool
            .reduce_trace(&gg, &[3, 3], &copy)
            .map_err(|e| invalid(e.to_string()))?;
        assert_ne!(legacy.summary.chosen_token_id, 4);
        assert_eq!(shared.summary.chosen_token_id, 4);
        let u_gradient = grads
            .get(delta.as_tensor())
            .ok_or_else(|| invalid("testUgradientabsent"))?
            .to_vec1::<f32>()?;
        let upstream_gradient = grads
            .get(old.as_tensor())
            .ok_or_else(|| invalid("testupstreamgradientabsent"))?
            .to_vec1::<f32>()?;
        Ok((
            shared,
            copy,
            graph.to_vec1::<f32>()?,
            u_gradient,
            upstream_gradient,
        ))
    }

    #[test]
    fn shared_continuation_copy_clip_ceiling_and_copy_adjoint() -> Result<()> {
        shared_copy_join_parity(&Device::Cpu)?;
        Ok(())
    }

    #[cfg(feature = "cuda")]
    #[test]
    #[ignore = "requires an explicitly leased CUDA device; unavailability is an error"]
    fn continuation_cuda_shared_copy_native_anchor_and_duplicate_adjoint_match_cpu() -> Result<()> {
        let cuda = Device::new_cuda(0)?;
        assert_eq!(
            shared_copy_join_parity(&Device::Cpu)?,
            shared_copy_join_parity(&cuda)?
        );
        Ok(())
    }

    fn fixed_exp_bytes() -> Vec<u8> {
        (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect()
    }

    #[test]
    fn fixed_continuation_join_matches_native_zero_nonzero_full_alias_mass() -> Result<()> {
        use uor_r4_integer::geometric_source_realizer::{ArtifactIdentity, NativeArtifactBinding};
        let binding =
            SourceActionBinding::new(BRIDGE_TOK.as_bytes()).map_err(|e| invalid(e.to_string()))?;
        let device = Device::Cpu;
        let generate =
            GenerateLearningWeights::seeded(binding.clone(), 2, 73, &device)?.export_native()?;
        let native_binding = NativeArtifactBinding {
            metadata_sha256: "a".repeat(64),
            identity: ArtifactIdentity {
                tokenizer_sha256: binding.tokenizer_sha256().to_owned(),
                parent_checkpoint_manifest_sha256: "b".repeat(64),
                parent_model_sha256: "c".repeat(64),
                parent_config_sha256: "d".repeat(64),
            },
        };
        let weights = ContinuationLearningWeights::zeroed(&binding, &native_binding, 2, &device)?;
        let state = [
            H4Code::try_from(31).map_err(|e| invalid(e.to_string()))?,
            H4Code::try_from(57).map_err(|e| invalid(e.to_string()))?,
        ];
        let mut base = vec![0; binding.vocab_size()];
        generate
            .score_into(&state, &mut base, &mut Default::default())
            .map_err(|e| invalid(e.to_string()))?;
        let base_tensor = Tensor::from_vec(base.clone(), base.len(), &device)?;
        // Copy duplicate aliases remain in the same denominator, not deduped.
        let copy_ids = [4, 4, 3];
        let copy_scores = [1 << 24, -(1 << 24), 0];
        for nonzero in [false, true] {
            if nonzero {
                weights.unary.set(&Tensor::from_vec(
                    (0..240)
                        .map(|i| ((i * 7) % 15) as f32 * 0.25 - 1.75)
                        .collect::<Vec<_>>(),
                    (2, 120),
                    &device,
                )?)?;
            }
            let prepared = weights.prepare_native(&native_binding, &generate)?;
            let delta = weights.forward_prepared_coefficients_only_on_device(&prepared, &state)?;
            let (actual_scores, raw) = join_fixed_continuation_scores(&base_tensor, &delta)?;
            let mut expected_delta = vec![0; base.len()];
            prepared
                .native
                .score_delta_into(
                    &state,
                    &generate,
                    &mut expected_delta,
                    &mut Default::default(),
                )
                .map_err(|e| invalid(e.to_string()))?;
            let expected = base
                .iter()
                .zip(expected_delta)
                .map(|(&a, b)| a + b)
                .collect::<Vec<_>>();
            assert_eq!(actual_scores, expected);
            assert_eq!(
                raw.to_vec1::<f32>()?,
                expected
                    .iter()
                    .map(|&v| (v as f64 / 16_777_216.) as f32)
                    .collect::<Vec<_>>()
            );
            let mut actual_pool = NativeVocabularyActions::new(binding.clone(), &fixed_exp_bytes())
                .map_err(|e| invalid(e.to_string()))?;
            let mut expected_pool =
                NativeVocabularyActions::new(binding.clone(), &fixed_exp_bytes())
                    .map_err(|e| invalid(e.to_string()))?;
            assert_eq!(
                actual_pool
                    .reduce_trace(&actual_scores, &copy_ids, &copy_scores)
                    .map_err(|e| invalid(e.to_string()))?,
                expected_pool
                    .reduce_trace(&expected, &copy_ids, &copy_scores)
                    .map_err(|e| invalid(e.to_string()))?
            );
            // Genuine U coefficient credit survives the independent hard anchor.
            let gradients = raw.sum_all()?.backward()?;
            assert!(
                gradients
                    .get(weights.unary.as_tensor())
                    .ok_or_else(|| invalid("fixed continuation U gradient absent"))?
                    .abs()?
                    .sum_all()?
                    .to_scalar::<f32>()?
                    > 0.
            );
        }
        Ok(())
    }

    #[test]
    fn fixed_continuation_refresh_rejects_changed_parent_and_generate_identity() -> Result<()> {
        use uor_r4_integer::geometric_source_realizer::{ArtifactIdentity, NativeArtifactBinding};
        let binding =
            SourceActionBinding::new(BRIDGE_TOK.as_bytes()).map_err(|e| invalid(e.to_string()))?;
        let generate = GenerateLearningWeights::seeded(binding.clone(), 2, 73, &Device::Cpu)?
            .export_native()?;
        let parent = NativeArtifactBinding {
            metadata_sha256: "a".repeat(64),
            identity: ArtifactIdentity {
                tokenizer_sha256: binding.tokenizer_sha256().to_owned(),
                parent_checkpoint_manifest_sha256: "b".repeat(64),
                parent_model_sha256: "c".repeat(64),
                parent_config_sha256: "d".repeat(64),
            },
        };
        let weights = ContinuationLearningWeights::zeroed(&binding, &parent, 2, &Device::Cpu)?;
        let prepared = weights.prepare_native(&parent, &generate)?;
        let sha = prepared.field_generate_sha();
        admit_fixed_continuation_refresh(
            &weights,
            &prepared,
            &binding,
            &parent,
            sha,
            generate.metadata(),
            &Device::Cpu,
        )?;
        let mut wrong = parent.clone();
        wrong.metadata_sha256 = "e".repeat(64);
        assert!(admit_fixed_continuation_refresh(
            &weights,
            &prepared,
            &binding,
            &wrong,
            sha,
            generate.metadata(),
            &Device::Cpu
        )
        .is_err());
        assert!(admit_fixed_continuation_refresh(
            &weights,
            &prepared,
            &binding,
            &parent,
            &"f".repeat(64),
            generate.metadata(),
            &Device::Cpu
        )
        .is_err());
        Ok(())
    }
    const BRIDGE_TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn source_copy_bridge() -> Result<(
        SourceActionBinding,
        PreparedCategoricalBridge,
        BridgeLearningWeights,
        PreparedBridgeLearning,
    )> {
        let binding =
            SourceActionBinding::new(BRIDGE_TOK.as_bytes()).map_err(|e| invalid(e.to_string()))?;
        let masters = BridgeLearningWeights::zeroed(&binding, 1, &Device::Cpu)?;
        // T(d)=d: native q * (q^-1 * k) equals k in every query frame.
        let mut relative = vec![0f32; ROOT_COUNT * ROOT_COUNT];
        for d in 0..ROOT_COUNT {
            relative[d * ROOT_COUNT + d] = 0.25;
        }
        masters.relative.set(&Tensor::from_vec(
            relative,
            (1, ROOT_COUNT, ROOT_COUNT),
            &Device::Cpu,
        )?)?;
        let artifact = masters.export_categorical_actions()?;
        let bytes = artifact
            .native
            .to_bytes()
            .map_err(|e| invalid(e.to_string()))?;
        let categorical = PreparedCategoricalBridge::from_bytes(
            &bytes,
            &binding,
            &artifact.receipt.native_sha256,
            &Device::Cpu,
        )?;
        let legacy = BridgeLearningWeights::from_native(&artifact.native, &binding, &Device::Cpu)?;
        let prepared = legacy.prepare_native()?;
        Ok((binding, categorical, legacy, prepared))
    }
    #[test]
    fn bank_categorical_configuration_rejects_conflicts_in_both_orders_and_binding() -> Result<()> {
        let (binding, categorical, weights, native) = source_copy_bridge()?;
        admit_categorical_bridge(&categorical, &binding, 1, &Device::Cpu)?;
        assert!(admit_categorical_bridge(&categorical, &binding, 2, &Device::Cpu).is_err());
        let other = SourceActionBinding::new(BRIDGE_TOK.replace("\"a\"", "\"b\"").as_bytes())
            .map_err(|e| invalid(e.to_string()))?;
        assert!(admit_categorical_bridge(&categorical, &other, 1, &Device::Cpu).is_err());
        for (first, second) in [
            (
                ReadStateBridge::Legacy(&weights, &native),
                ReadStateBridge::Categorical(&categorical),
            ),
            (
                ReadStateBridge::Categorical(&categorical),
                ReadStateBridge::Legacy(&weights, &native),
            ),
        ] {
            let mut slot = None;
            configure_bridge(&mut slot, first)?;
            assert!(configure_bridge(&mut slot, second).is_err());
            assert_eq!(
                slot.ok_or_else(|| invalid("configured bridge absent"))?
                    .categorical(),
                first.categorical()
            );
        }
        Ok(())
    }
    #[test]
    fn bank_categorical_dispatch_preserves_native_transport_and_selector_forward() -> Result<()> {
        let (_, categorical, weights, prepared) = source_copy_bridge()?;
        let q = [H4Code::try_from(17).map_err(|e| invalid(e.to_string()))?];
        let k = [H4Code::try_from(83).map_err(|e| invalid(e.to_string()))?];
        let onehot = |code: H4Code| -> Result<Tensor> {
            let mut v = vec![0f32; ROOT_COUNT];
            v[usize::from(code.index())] = 1.;
            Ok(Tensor::from_vec(v, (1, ROOT_COUNT), &Device::Cpu)?)
        };
        let qc = Var::from_tensor(&onehot(q[0])?)?;
        let kc = Var::from_tensor(&onehot(k[0])?)?;
        let legacy = ReadStateBridge::Legacy(&weights, &prepared).forward(
            &q,
            &k,
            qc.as_tensor(),
            kc.as_tensor(),
        )?;
        let configured = ReadStateBridge::Categorical(&categorical);
        let out = configured.forward(&q, &k, qc.as_tensor(), kc.as_tensor())?;
        assert_eq!(out.post_state_codes, k);
        assert_eq!(out.post_state_codes, legacy.post_state_codes);
        assert_eq!(out.action_codes, legacy.action_codes);
        assert_eq!(out.action_scores_q24, legacy.action_scores_q24);
        assert_eq!(
            out.state_choices.to_vec2::<f32>()?,
            legacy.state_choices.to_vec2::<f32>()?
        );
        let mut posts = Vec::new();
        for source in [k, q, k] {
            let mut post = [H4Code::IDENTITY];
            let mut actions = post;
            configured
                .native()
                .apply_into(
                    &q,
                    &source,
                    &mut post,
                    &mut actions,
                    &mut vec![0; ROOT_COUNT],
                    &mut Default::default(),
                )
                .map_err(|e| invalid(e.to_string()))?;
            posts.push(post.to_vec());
        }
        let selector = Var::from_vec(vec![2f32, -1., 2.], 3, &Device::Cpu)?;
        let selected = hard_read_index(&[2, -1, 2])?;
        assert_eq!(selected, 0);
        let with_selector =
            add_selector_credit(&out.state_choices, selector.as_tensor(), &posts, selected)?;
        assert_eq!(
            with_selector.to_vec2::<f32>()?,
            out.state_choices.to_vec2::<f32>()?
        );
        // Source-copy conditional pullback: the normalized query tangent is
        // zero while source utility is preserved. Selector does not detach it.
        let utility = Tensor::from_vec(
            (0..ROOT_COUNT)
                .map(|i| (i % 13) as f32 - 6.)
                .collect::<Vec<_>>(),
            (1, ROOT_COUNT),
            &Device::Cpu,
        )?;
        for carrier in [&out.state_choices, &with_selector] {
            let grads = carrier.mul(&utility)?.sum_all()?.backward()?;
            let qg = grads
                .get(qc.as_tensor())
                .ok_or_else(|| invalid("query gradient absent"))?
                .to_vec2::<f32>()?;
            assert!(qg[0].iter().all(|v| *v == qg[0][0]));
            assert_eq!(
                grads
                    .get(kc.as_tensor())
                    .ok_or_else(|| invalid("source gradient absent"))?
                    .to_vec2::<f32>()?,
                utility.to_vec2::<f32>()?
            );
            assert!(grads.get(weights.relative.as_tensor()).is_none());
        }
        Ok(())
    }

    #[test]
    fn continuation_no_source_boundary_rejects_noncausal_or_empty_query() -> Result<()> {
        validate_query_suffix(&[9, 10, 4, 5, 7], &[4, 5], &[7])?;
        validate_query_suffix(&[9, 10, 4, 5], &[4, 5], &[])?;
        assert!(validate_query_suffix(&[4, 5, 7], &[], &[4, 5, 7]).is_err());
        assert!(validate_query_suffix(&[9, 10, 4, 5, 7], &[4, 5], &[8]).is_err());
        assert!(validate_query_suffix(&[9, 10, 4, 5, 7], &[5, 4], &[7]).is_err());
        assert!(validate_query_suffix(&[4, 5], &[4, 5], &[7]).is_err());
        Ok(())
    }

    #[test]
    fn continuation_combined_q24_preserves_zero_and_rejects_overflow_shape() -> Result<()> {
        let mut scores = [1 << 24, -(3 << 24), 7 << 20];
        let before = scores;
        add_continuation_q24(&mut scores, &[0, 0, 0])?;
        assert_eq!(scores, before);
        add_continuation_q24(&mut scores, &[-(1 << 24), 2 << 24, -(7 << 20)])?;
        assert_eq!(scores, [0, -(1 << 24), 0]);
        assert!(add_continuation_q24(&mut scores, &[0]).is_err());
        assert!(add_continuation_q24(&mut [i64::MAX], &[1]).is_err());
        Ok(())
    }

    #[test]
    fn hard_read_preserves_ordinal_ties_and_full_i64_order() -> Result<()> {
        assert_eq!(hard_read_index(&[i64::MAX - 1, i64::MAX, i64::MAX])?, 1);
        assert_eq!(hard_read_index(&[0, 0])?, 0);
        assert!(hard_read_index(&[]).is_err());
        Ok(())
    }

    #[test]
    fn selector_contrasts_are_zero_for_identical_states_and_isolate_adjoints() -> Result<()> {
        let device = Device::Cpu;
        let state = H4Code::IDENTITY;
        let other = H4Code::try_from(2).map_err(|e| invalid(e.to_string()))?;
        let mut values = vec![0f32; ROOT_COUNT];
        values[usize::from(state.index())] = 1.;
        let factual = Var::from_vec(values.clone(), (1, ROOT_COUNT), &device)?;
        let scores = Var::from_vec(vec![0f32, 0.], 2, &device)?;
        let utility = Tensor::from_vec(
            (0..ROOT_COUNT).map(|i| i as f32).collect::<Vec<_>>(),
            (1, ROOT_COUNT),
            &device,
        )?;
        for alternatives in [
            vec![vec![state], vec![state]],
            vec![vec![state], vec![other]],
        ] {
            let output =
                add_selector_credit(factual.as_tensor(), scores.as_tensor(), &alternatives, 0)?;
            assert_eq!(output.to_vec2::<f32>()?, vec![values.clone()]);
            let g = (&output * &utility)?.sum_all()?.backward()?;
            assert_eq!(
                g.get(factual.as_tensor())
                    .ok_or_else(|| invalid("factual carrier credit absent"))?
                    .to_vec2::<f32>()?,
                utility.to_vec2::<f32>()?
            );
            let credit = g
                .get(scores.as_tensor())
                .ok_or_else(|| invalid("selector score credit absent"))?
                .to_vec1::<f32>()?;
            if alternatives[0] == alternatives[1] {
                assert!(credit.iter().all(|x| *x == 0.));
            } else {
                assert!(credit.iter().all(|x| x.is_finite()));
                assert!(credit.iter().any(|x| *x != 0.));
                assert!((credit.iter().sum::<f32>()).abs() < 1e-6);
            }
        }
        Ok(())
    }
    #[test]
    fn bank_generate_final_retained_state_is_last_time_head_major_and_not_observation() -> Result<()>
    {
        let choices = Var::zeros((1, 2, 2, 2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let states = vec![vec![2u8, 3, 4, 5], vec![6, 7, 8, 9]];
        let latent = states
            .iter()
            .flatten()
            .flat_map(|r| CANONICAL_BASIS_Q25[usize::from(*r)].map(|x| x as f32 / 33_554_432.))
            .collect::<Vec<_>>();
        let context = ContextQ4Output {
            state_logits: choices.as_tensor().clone(),
            state_choices: choices.as_tensor().clone(),
            latent_roots: Tensor::from_vec(latent, (1, 2, 2, 2, 4), &Device::Cpu)?,
            root_logits: Tensor::zeros((1, 2, 2, 2, ROOT_COUNT), DType::F32, &Device::Cpu)?,
            category_logits: Tensor::zeros((1, 2, 2, 2, 33), DType::F32, &Device::Cpu)?,
            trace: NativeContextTrace {
                batch: 1,
                time: 2,
                heads: 2,
                lanes_per_head: 2,
                states,
                actions: vec![vec![1; 4]; 2],
                emitted_roots: vec![1; 8],
                categories: vec![0; 8],
                codes: vec![AddressLane::new(1, 0, false).map_err(|e| invalid(e.to_string()))?; 8],
                coefficient_reads: 0,
            },
        };
        let (codes, final_choices) = final_retained_state(&context, 4)?;
        assert_eq!(
            codes.iter().map(|r| r.index()).collect::<Vec<_>>(),
            vec![6, 7, 8, 9]
        );
        let gradient = final_choices.sum_all()?.backward()?;
        let values = gradient
            .get(choices.as_tensor())
            .ok_or_else(|| invalid("final-state test missing choice gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(values[..4 * ROOT_COUNT].iter().all(|x| *x == 0.));
        assert!(values[4 * ROOT_COUNT..].iter().all(|x| *x == 1.));
        let (source_codes, source_choices) = retained_state_at(&context, 0, 4)?;
        assert_eq!(
            source_codes.iter().map(|r| r.index()).collect::<Vec<_>>(),
            vec![2, 3, 4, 5]
        );
        let source_gradient = source_choices.sum_all()?.backward()?;
        let source_values = source_gradient
            .get(choices.as_tensor())
            .ok_or_else(|| invalid("source-state test missing choice gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert!(source_values[..4 * ROOT_COUNT].iter().all(|x| *x == 1.));
        assert!(source_values[4 * ROOT_COUNT..].iter().all(|x| *x == 0.));
        assert!(retained_state_at(&context, 2, 4).is_err());
        assert!(final_retained_state(&context, 8).is_err());
        Ok(())
    }
}
