//! Target-free composition of actual full-bank Copy and native Generate.
//!
//! Both branches consume the same authentic causal ContextQ4Output. Generate
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

use candle_core::{DType, Tensor};
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

pub struct PreparedBankGenerate<'a, 'source> {
    realizer: &'a PreparedSourceRealizer<'source>,
    generate: &'a GenerateLearningWeights,
    prepared_generate: &'a PreparedGenerateLearning,
    pool: NativeVocabularyActions,
    prefix_temporal_utility: bool,
    read_state_bridge: Option<ReadStateBridge<'a>>,
    read_selector_credit: bool,
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
        })
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
        let copy = if self.prefix_temporal_utility {
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
        let generated = self.generate.forward_prepared_state_choices(
            self.prepared_generate,
            &states,
            &logits,
        )?;
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
        let generated = self.generate.forward_prepared_state_choices(
            self.prepared_generate,
            &states,
            &logits,
        )?;
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
