//! Offline Q4 learning of a shared signed-H4 continuation unary field.
//!
//! The caller owns a separate causal ContextQ4(query || actual emitted prefix)
//! replay from identity. It must supply that replay's retained states; this
//! scorer cannot infer query/prefix provenance from IDs or tensor shapes.
//! No labels, source records, answer lengths or first-token gates enter here.
//!
//! Authenticated packed coefficients and frozen Generate prototypes supply
//! exact I64 Q24 deltas through device table gathers. Selected quarter-grid
//! coefficient credit is added with a subtraction-first zero-valued term.
//! The optional hard-onehot full120 utility method credits local state choices;
//! it does not differentiate prototype choices or a global recurrent posterior.
//! Refresh prepare_native and all graphs after every optimizer update. CPU
//! checks packed-master freshness; CUDA freshness is an explicit caller duty.
//! The ordinary bank scorer must add both native deltas and graph deltas before
//! its sole final clip, Copy/Generate pool and token-alias marginal loss.

use std::collections::BTreeMap;

use candle_core::{DType, Device, Tensor, Var};
use serde::Serialize;
use uor_r4_core::native_geometric::learner::{
    geometric_continuation_field::{ContinuationReadCounts, NativeContinuationField},
    geometric_generate::{NativeGeometricGenerate, MAX_LANES, SCORE_SHIFT},
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::NativeArtifactBinding,
    h4_tables::{H4Code, ROOT_COUNT},
};

use crate::{
    geometric_context::{q4_project_tensor, q4_shadow_ste},
    invalid, Result,
};

pub const COEFFICIENT_CREDIT_SCOPE: &str = "authenticated-native-Q24-continuation-delta;shared-full120-unary-selected-quarter-grid-STE;frozen-causal-local-state-and-existing-token-prototypes;all-Generate-IDs-every-step;one-final-common-clip-and-token-alias-loss/1";
pub const STATE_CREDIT_SCOPE: &str = "authenticated-native-Q24-continuation-delta;selected-quarter-grid-STE-once;hard-retained-onehot120-local-state-utility;detached-unary-full120-conditional-utilities;frozen-existing-token-prototypes;no-input-renormalization;local-conditional-surrogate-not-global-posterior/1";
const Q24: f64 = 16_777_216.;
// V1 quarter shadow times1/4 is nibble<<20; v2 times1 is nibble<<22.
const QUARTER_TO_NATS: f64 = 0.25;

pub struct ContinuationLearningWeights {
    binding: SourceActionBinding,
    native_binding: NativeArtifactBinding,
    lanes: usize,
    shared_action: bool,
    /// Shared [lane, inv(local_state) * existing_token_prototype] masters.
    /// There are exactly 960 coefficients at eight lanes, with no token bias.
    pub unary: Var,
}

pub struct PreparedContinuationLearning {
    pub native: NativeContinuationField,
    /// Frozen exact Generate artifact to which this field is bound.
    pub generate: NativeGeometricGenerate,
    pub downloaded_master_bytes: usize,
    cache: ContinuationDeviceCache,
}

struct ContinuationDeviceCache {
    payload_sha256: String,
    // Rows are state IDs, columns prototype IDs; entries are inv(state)*proto.
    relative: Tensor,
    prototypes: Vec<Tensor>,
    // Frozen authenticated native nibble coefficients, already shifted by20.
    hard_unary_q24: Tensor,
    staged_index_bytes: usize,
    staged_factor_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContinuationLearningCosts {
    pub vocabulary_rows: usize,
    pub lanes: usize,
    pub shared_coefficients: usize,
    pub export_master_download_bytes: usize,
    pub snapshot_staged_index_bytes: usize,
    pub snapshot_staged_factor_bytes: usize,
    pub per_position_device_index_elements: usize,
    pub conditional_choice_rows: usize,
    pub max_conditional_utility_elements: usize,
    pub hard_score_backend: &'static str,
    /// Compatibility download for the existing host native action pool.
    pub hard_score_download_bytes: usize,
    pub native_reference_scores: usize,
    /// Scalar-only admission of optional retained-state onehot carriers.
    pub state_carrier_validation_scalars: usize,
    pub state_carrier_staged_bytes: usize,
    pub credit_scope: &'static str,
}

/// Device-resident coefficient-only result for an exact device alias pool.
/// It does not introduce state/prototype credit or imply bank integration.
pub struct ContinuationDeviceLearningOutput {
    /// Unclipped graph delta; add before the sole ordinary pool clip.
    pub delta_raw_scores: Tensor,
    /// Authenticated I64 Q24 delta in complete token-ID order, on the device.
    pub delta_scores_q24: Tensor,
    pub native_counts: ContinuationReadCounts,
    pub costs: ContinuationLearningCosts,
}

pub struct ContinuationLearningOutput {
    /// Unclipped nats. Add to ordinary Generate raw scores before final clip.
    pub delta_raw_scores: Tensor,
    /// Exact native Q24 deltas, in the same complete token-ID order.
    pub delta_scores_q24: Vec<i64>,
    /// Portable reference counts; zero for cached device table execution.
    pub native_counts: ContinuationReadCounts,
    pub costs: ContinuationLearningCosts,
}

fn device_admit(device: &Device) -> Result<()> {
    if !device.is_cpu() && !device.is_cuda() {
        return Err(invalid(
            "continuation learning supports CPU reference/CUDA only",
        ));
    }
    #[cfg(not(feature = "cuda"))]
    if device.is_cuda() {
        return Err(invalid("continuation CUDA requested without cuda feature"));
    }
    Ok(())
}

fn packed_unary(var: &Var) -> Result<Vec<u8>> {
    let values = var.flatten_all()?.to_vec1::<f32>()?;
    if values.iter().any(|x| !x.is_finite() || x.abs() > 1.75) {
        return Err(invalid(
            "continuation master outside strict finite quarter range",
        ));
    }
    let mut packed = vec![0; values.len().div_ceil(2)];
    for (i, value) in values.iter().enumerate() {
        let nibble = (((value * 4.).round() as i8) as u8) & 15;
        packed[i / 2] |= nibble << ((i & 1) * 4);
    }
    Ok(packed)
}

impl ContinuationLearningWeights {
    pub fn zeroed(
        binding: &SourceActionBinding,
        native_binding: &NativeArtifactBinding,
        lanes: usize,
        device: &Device,
    ) -> Result<Self> {
        Self::zeroed_policy(binding, native_binding, lanes, device, false)
    }
    /// Explicit shared-action v2; default zeroed remains legacy v1.
    pub fn zeroed_shared_action(
        binding: &SourceActionBinding,
        native_binding: &NativeArtifactBinding,
        lanes: usize,
        device: &Device,
    ) -> Result<Self> {
        Self::zeroed_policy(binding, native_binding, lanes, device, true)
    }
    fn zeroed_policy(
        binding: &SourceActionBinding,
        native_binding: &NativeArtifactBinding,
        lanes: usize,
        device: &Device,
        shared_action: bool,
    ) -> Result<Self> {
        device_admit(device)?;
        if !(1..=MAX_LANES).contains(&lanes)
            || !(1..=4096).contains(&binding.vocab_size())
            || native_binding.identity.tokenizer_sha256 != binding.tokenizer_sha256()
        {
            return Err(invalid(
                "continuation zero master lanes/parent/tokenizer differs",
            ));
        }
        Ok(Self {
            binding: binding.clone(),
            native_binding: native_binding.clone(),
            lanes,
            shared_action,
            unary: Var::zeros((lanes, ROOT_COUNT), DType::F32, device)?,
        })
    }

    pub fn from_native(
        native: &NativeContinuationField,
        generate: &NativeGeometricGenerate,
        binding: &SourceActionBinding,
        device: &Device,
    ) -> Result<Self> {
        device_admit(device)?;
        let native = NativeContinuationField::from_bytes(
            &native.to_bytes().map_err(|e| invalid(e.to_string()))?,
            &native.metadata().source_binding,
            generate,
        )
        .map_err(|e| invalid(e.to_string()))?;
        if generate.metadata().tokenizer_sha256 != binding.tokenizer_sha256()
            || generate.metadata().dialogue_protocol != *binding.protocol()
            || generate.vocab_size() != binding.vocab_size()
        {
            return Err(invalid(
                "continuation native/learning token binding differs",
            ));
        }
        let lanes = native.lanes();
        let mut values = Vec::with_capacity(lanes * ROOT_COUNT);
        for lane in 0..lanes {
            for r in 0..ROOT_COUNT {
                values.push(
                    f32::from(
                        native
                            .coefficient_unary(lane, r as u8)
                            .map_err(|e| invalid(e.to_string()))?,
                    ) * 0.25,
                );
            }
        }
        Ok(Self {
            binding: binding.clone(),
            native_binding: native.metadata().source_binding.clone(),
            lanes,
            shared_action: native.applies_to_copy(),
            unary: Var::from_vec(values, (lanes, ROOT_COUNT), device)?,
        })
    }

    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn native_binding(&self) -> &NativeArtifactBinding {
        &self.native_binding
    }
    pub fn applies_to_copy(&self) -> bool {
        self.shared_action
    }
    pub fn score_shift(&self) -> u32 {
        if self.shared_action {
            22
        } else {
            20
        }
    }
    fn quarter_to_score_units(&self) -> f64 {
        if self.shared_action {
            1.
        } else {
            QUARTER_TO_NATS
        }
    }
    pub fn lanes(&self) -> usize {
        self.lanes
    }
    pub fn vocab_size(&self) -> usize {
        self.binding.vocab_size()
    }
    pub fn device(&self) -> &Device {
        self.unary.device()
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        BTreeMap::from([("continuation.unary".into(), self.unary.clone())])
    }
    pub fn project_shadow_range(&self) -> Result<()> {
        self.unary
            .set(&q4_project_tensor(self.unary.as_tensor())?)?;
        Ok(())
    }
    pub fn packed_coefficients(&self) -> Result<Vec<u8>> {
        self.validate_shapes()?;
        packed_unary(&self.unary)
    }

    pub fn export_native(
        &self,
        current_binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
    ) -> Result<NativeContinuationField> {
        self.validate_generate(generate)?;
        // NativeSourceRealizer metadata changes under a legitimate context or
        // potential update, but its checkpoint/model/config identity does not.
        // The caller must supply that freshly saved current receipt explicitly.
        if current_binding.identity != self.native_binding.identity {
            return Err(invalid(
                "continuation current native parent base identity differs",
            ));
        }
        let packed = self.packed_coefficients()?;
        if self.shared_action {
            NativeContinuationField::compile_shared_action(current_binding, generate, &packed)
        } else {
            NativeContinuationField::compile(current_binding, generate, &packed)
        }
        .map_err(|e| invalid(e.to_string()))
    }

    pub fn prepare_native(
        &self,
        current_binding: &NativeArtifactBinding,
        generate: &NativeGeometricGenerate,
    ) -> Result<PreparedContinuationLearning> {
        let native = self.export_native(current_binding, generate)?;
        let mut relations = Vec::with_capacity(ROOT_COUNT * ROOT_COUNT);
        for state in 0..ROOT_COUNT {
            for prototype in 0..ROOT_COUNT {
                relations.push(u32::from(
                    generate
                        .algebra()
                        .relative(state as u8, generate.algebra().identity(), prototype as u8)
                        .map_err(|e| invalid(e.to_string()))?,
                ));
            }
        }
        let mut prototypes = Vec::with_capacity(self.lanes);
        for lane in 0..self.lanes {
            let ids = (0..self.vocab_size())
                .map(|token| u32::from(generate.prototypes()[token * self.lanes + lane]))
                .collect::<Vec<_>>();
            prototypes.push(Tensor::from_vec(ids, self.vocab_size(), self.device())?);
        }
        let mut factors = Vec::with_capacity(self.lanes * ROOT_COUNT);
        for lane in 0..self.lanes {
            for r in 0..ROOT_COUNT {
                factors.push(
                    i64::from(
                        native
                            .coefficient_unary(lane, r as u8)
                            .map_err(|e| invalid(e.to_string()))?,
                    ) << native.score_shift(),
                );
            }
        }
        let cache = ContinuationDeviceCache {
            payload_sha256: native.metadata().payload_sha256.clone(),
            relative: Tensor::from_vec(relations, (ROOT_COUNT, ROOT_COUNT), self.device())?,
            prototypes,
            hard_unary_q24: Tensor::from_vec(factors, (self.lanes, ROOT_COUNT), self.device())?,
            staged_index_bytes: 4 * (ROOT_COUNT * ROOT_COUNT + self.vocab_size() * self.lanes),
            staged_factor_bytes: 8 * self.lanes * ROOT_COUNT,
        };
        Ok(PreparedContinuationLearning {
            native,
            generate: generate.clone(),
            downloaded_master_bytes: 4 * self.unary.elem_count(),
            cache,
        })
    }

    fn validate_shapes(&self) -> Result<()> {
        device_admit(self.device())?;
        if self.unary.dims() != [self.lanes, ROOT_COUNT]
            || self.unary.dtype() != DType::F32
            || !(1..=MAX_LANES).contains(&self.lanes)
        {
            return Err(invalid("continuation master shape/dtype differs"));
        }
        Ok(())
    }
    fn validate_generate(&self, generate: &NativeGeometricGenerate) -> Result<()> {
        self.validate_shapes()?;
        if generate.lanes() != self.lanes
            || generate.vocab_size() != self.vocab_size()
            || generate.metadata().score_shift != SCORE_SHIFT
            || generate.metadata().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || generate.metadata().dialogue_protocol != *self.binding.protocol()
        {
            return Err(invalid(
                "continuation Generate snapshot/token binding differs",
            ));
        }
        Ok(())
    }
    fn validate_snapshot(
        &self,
        prepared: &PreparedContinuationLearning,
        state: &[H4Code],
    ) -> Result<()> {
        self.validate_generate(&prepared.generate)?;
        if state.len() != self.lanes
            || prepared.native.applies_to_copy() != self.shared_action
            || prepared.native.score_shift() != self.score_shift()
            || prepared.native.lanes() != self.lanes
            || prepared.native.vocab_size() != self.vocab_size()
            || prepared.native.metadata().source_binding.identity != self.native_binding.identity
            || prepared.native.metadata().generate_metadata != *prepared.generate.metadata()
            || prepared.native.metadata().payload_sha256 != prepared.cache.payload_sha256
            || !prepared.cache.relative.device().same_device(self.device())
            || !prepared
                .cache
                .hard_unary_q24
                .device()
                .same_device(self.device())
        {
            return Err(invalid(
                "continuation prepared parent/state/device binding differs",
            ));
        }
        // compile/from_bytes authenticated exact immutable Generate bytes at
        // preparation/install. Metadata comparison is sufficient here because
        // NativeGeometricGenerate exposes no mutable payload or metadata.
        if self.device().is_cpu() && self.packed_coefficients()? != prepared.native.packed_unary() {
            return Err(invalid(
                "continuation prepared native coefficients are stale",
            ));
        }
        Ok(())
    }

    /// First rung: coefficients only, with native local state and prototypes frozen.
    pub fn forward_prepared_coefficients_only(
        &self,
        prepared: &PreparedContinuationLearning,
        state: &[H4Code],
    ) -> Result<ContinuationLearningOutput> {
        self.forward_cached(prepared, state, None)
    }

    /// First rung without the compatibility per-position score download.
    /// The same snapshot/device admission and selected coefficient adjoint are
    /// used as the host API. CUDA callers must refresh after every update.
    pub fn forward_prepared_coefficients_only_on_device(
        &self,
        prepared: &PreparedContinuationLearning,
        state: &[H4Code],
    ) -> Result<ContinuationDeviceLearningOutput> {
        self.forward_cached_on_device(prepared, state, None)
    }

    /// Same hard forward plus full120 utility on the actual retained-state
    /// onehot carrier. Do not softmax this carrier a second time.
    pub fn forward_prepared_state_choices(
        &self,
        prepared: &PreparedContinuationLearning,
        state: &[H4Code],
        state_choices: &Tensor,
    ) -> Result<ContinuationLearningOutput> {
        self.forward_cached(prepared, state, Some(state_choices))
    }

    fn forward_cached(
        &self,
        prepared: &PreparedContinuationLearning,
        state: &[H4Code],
        choices: Option<&Tensor>,
    ) -> Result<ContinuationLearningOutput> {
        let device = self.forward_cached_on_device(prepared, state, choices)?;
        let delta_scores_q24 = device.delta_scores_q24.to_vec1::<i64>()?;
        let mut costs = device.costs;
        costs.hard_score_download_bytes = if self.device().is_cuda() {
            8 * self.vocab_size()
        } else {
            0
        };
        Ok(ContinuationLearningOutput {
            delta_raw_scores: device.delta_raw_scores,
            delta_scores_q24,
            native_counts: device.native_counts,
            costs,
        })
    }

    fn forward_cached_on_device(
        &self,
        prepared: &PreparedContinuationLearning,
        state: &[H4Code],
        choices: Option<&Tensor>,
    ) -> Result<ContinuationDeviceLearningOutput> {
        self.validate_snapshot(prepared, state)?;
        if let Some(choices) = choices {
            carrier_admit(choices, state, self.device())?;
        }
        let vocab = self.vocab_size();
        let cache = &prepared.cache;
        let unary = (q4_shadow_ste(self.unary.as_tensor())? * self.quarter_to_score_units())?;
        let mut hard = Tensor::zeros(vocab, DType::I64, self.device())?;
        let mut coefficients = Tensor::zeros(vocab, DType::F32, self.device())?;
        let mut utility = Tensor::zeros(vocab, DType::F32, self.device())?;
        for (lane, code) in state.iter().enumerate() {
            let factual = cache
                .relative
                .narrow(0, usize::from(code.index()), 1)?
                .reshape(ROOT_COUNT)?
                .index_select(&cache.prototypes[lane], 0)?;
            hard = (&hard
                + cache
                    .hard_unary_q24
                    .narrow(0, lane, 1)?
                    .reshape(ROOT_COUNT)?
                    .index_select(&factual, 0)?)?;
            let lane_unary = unary.narrow(0, lane, 1)?.reshape(ROOT_COUNT)?;
            coefficients = (&coefficients + lane_unary.index_select(&factual, 0)?)?;
            if let Some(choices) = choices {
                // [token, alternative local state], exact full signed-H4 table.
                let relatives = cache
                    .relative
                    .t()?
                    .contiguous()?
                    .index_select(&cache.prototypes[lane], 0)?;
                let alternatives = lane_unary
                    .detach()
                    .index_select(&relatives.flatten_all()?, 0)?
                    .reshape((vocab, ROOT_COUNT))?;
                utility = (&utility
                    + alternatives
                        .broadcast_mul(&choices.narrow(0, lane, 1)?)?
                        .sum(1)?)?;
            }
        }
        // Anchor and exact integers stay device-resident. Only the legacy
        // compatibility wrapper above materializes a host score vector.
        let anchor = hard.to_dtype(DType::F32)?.affine(1. / Q24, 0.)?.detach();
        let mut raw = (&anchor + (&coefficients - coefficients.detach())?)?;
        if choices.is_some() {
            raw = (&raw + (&utility - utility.detach())?)?;
        }
        let mut costs = self.costs(prepared, choices.is_some(), false);
        costs.hard_score_download_bytes = 0;
        Ok(ContinuationDeviceLearningOutput {
            delta_raw_scores: raw,
            delta_scores_q24: hard,
            native_counts: ContinuationReadCounts::default(),
            costs,
        })
    }

    /// Deliberately independent host geometry enumeration and portable native
    /// hard forward for focused CPU/device index, score and adjoint parity.
    pub fn forward_prepared_uncached_reference(
        &self,
        prepared: &PreparedContinuationLearning,
        state: &[H4Code],
        choices: Option<&Tensor>,
    ) -> Result<ContinuationLearningOutput> {
        self.validate_snapshot(prepared, state)?;
        if let Some(choices) = choices {
            carrier_admit(choices, state, self.device())?;
        }
        let vocab = self.vocab_size();
        let generate = &prepared.generate;
        let mut counts = ContinuationReadCounts::default();
        let mut hard = vec![0; vocab];
        prepared
            .native
            .score_delta_into(state, generate, &mut hard, &mut counts)
            .map_err(|e| invalid(e.to_string()))?;
        let unary = (q4_shadow_ste(self.unary.as_tensor())? * self.quarter_to_score_units())?;
        let mut coefficients = Tensor::zeros(vocab, DType::F32, self.device())?;
        let mut utility = Tensor::zeros(vocab, DType::F32, self.device())?;
        for lane in 0..self.lanes {
            let mut factual = Vec::with_capacity(vocab);
            let mut alternative = Vec::with_capacity(if choices.is_some() {
                vocab * ROOT_COUNT
            } else {
                0
            });
            for token in 0..vocab {
                let proto = generate.prototypes()[token * self.lanes + lane];
                factual.push(u32::from(
                    generate
                        .algebra()
                        .relative(state[lane].index(), generate.algebra().identity(), proto)
                        .map_err(|e| invalid(e.to_string()))?,
                ));
                if choices.is_some() {
                    for r in 0..ROOT_COUNT {
                        alternative.push(u32::from(
                            generate
                                .algebra()
                                .relative(r as u8, generate.algebra().identity(), proto)
                                .map_err(|e| invalid(e.to_string()))?,
                        ));
                    }
                }
            }
            let table = unary.narrow(0, lane, 1)?.reshape(ROOT_COUNT)?;
            coefficients = (&coefficients
                + table.index_select(&Tensor::from_vec(factual, vocab, self.device())?, 0)?)?;
            if let Some(choices) = choices {
                let values = table
                    .detach()
                    .index_select(
                        &Tensor::from_vec(alternative, vocab * ROOT_COUNT, self.device())?,
                        0,
                    )?
                    .reshape((vocab, ROOT_COUNT))?;
                utility =
                    (&utility + values.broadcast_mul(&choices.narrow(0, lane, 1)?)?.sum(1)?)?;
            }
        }
        let anchor = Tensor::from_vec(
            hard.iter()
                .map(|&v| (v as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            vocab,
            self.device(),
        )?;
        let mut raw = (&anchor + (&coefficients - coefficients.detach())?)?;
        if choices.is_some() {
            raw = (&raw + (&utility - utility.detach())?)?;
        }
        Ok(ContinuationLearningOutput {
            delta_raw_scores: raw,
            delta_scores_q24: hard,
            native_counts: counts,
            costs: self.costs(prepared, choices.is_some(), true),
        })
    }

    fn costs(
        &self,
        prepared: &PreparedContinuationLearning,
        choices: bool,
        reference: bool,
    ) -> ContinuationLearningCosts {
        let vocab = self.vocab_size();
        ContinuationLearningCosts {
            vocabulary_rows: vocab,
            lanes: self.lanes,
            shared_coefficients: self.lanes * ROOT_COUNT,
            export_master_download_bytes: prepared.downloaded_master_bytes,
            snapshot_staged_index_bytes: prepared.cache.staged_index_bytes,
            snapshot_staged_factor_bytes: prepared.cache.staged_factor_bytes,
            per_position_device_index_elements: vocab
                * self.lanes
                * (1 + usize::from(choices) * ROOT_COUNT),
            conditional_choice_rows: if choices { vocab * self.lanes } else { 0 },
            max_conditional_utility_elements: if choices { vocab * ROOT_COUNT } else { 0 },
            hard_score_backend: if reference {
                "cpu-native-reference"
            } else {
                "candle-i64-authenticated-unary-gathers"
            },
            hard_score_download_bytes: if self.device().is_cuda() && !reference {
                8 * vocab
            } else {
                0
            },
            native_reference_scores: if reference { vocab } else { 0 },
            state_carrier_validation_scalars: usize::from(choices),
            state_carrier_staged_bytes: if choices {
                4 * self.lanes * ROOT_COUNT
            } else {
                0
            },
            credit_scope: if self.shared_action {
                "authenticated-v2-shared-action-U;quarter-shadow-times1-score-units;every-physical-Copy-and-Generate-alias;one-common-clip;coefficient-only-unless-state-choices-explicit/2"
            } else if choices {
                STATE_CREDIT_SCOPE
            } else {
                COEFFICIENT_CREDIT_SCOPE
            },
        }
    }
}

impl PreparedContinuationLearning {
    pub fn field_generate_sha(&self) -> &str {
        self.native.generate_sha256()
    }
    pub fn native_binding(&self) -> &NativeArtifactBinding {
        &self.native.metadata().source_binding
    }
    pub fn device(&self) -> &Device {
        self.cache.relative.device()
    }
    /// Compare the whole independently supplied current execution receipt.
    pub fn validate_binding(&self, current: &NativeArtifactBinding) -> Result<()> {
        if self.native_binding() != current {
            return Err(invalid(
                "continuation prepared current native parent receipt differs",
            ));
        }
        Ok(())
    }
    /// Constructor/install check, including canonical Generate artifact bytes.
    /// Keep this outside per-token scoring so identity hashing is not serving work.
    pub fn validate_generate(&self, generate: &NativeGeometricGenerate) -> Result<()> {
        NativeContinuationField::from_bytes(
            &self.native.to_bytes().map_err(|e| invalid(e.to_string()))?,
            self.native_binding(),
            generate,
        )
        .map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
}

fn carrier_admit(choices: &Tensor, state: &[H4Code], device: &Device) -> Result<()> {
    if choices.dims() != [state.len(), ROOT_COUNT]
        || choices.dtype() != DType::F32
        || !choices.device().same_device(device)
    {
        return Err(invalid(
            "continuation retained-state carrier shape/dtype/device differs",
        ));
    }
    let mut hard = vec![0f32; state.len() * ROOT_COUNT];
    for (lane, code) in state.iter().enumerate() {
        hard[lane * ROOT_COUNT + usize::from(code.index())] = 1.;
    }
    let hard = Tensor::from_vec(hard, (state.len(), ROOT_COUNT), device)?;
    let mismatch = (choices - hard)?
        .abs()?
        .flatten_all()?
        .max(0)?
        .to_scalar::<f32>()?;
    if mismatch != 0. || !mismatch.is_finite() {
        return Err(invalid(
            "continuation choices differ from actual native retained state",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_generate_learning::{vocabulary_marginal_loss, GenerateLearningWeights};
    use uor_r4_integer::{
        geometric_source_realizer::ArtifactIdentity,
        geometric_vocabulary_actions::NativeVocabularyActions,
    };

    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"Ġ":6,"Ġa":7},"merges":["Ġ a"]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;

    fn fixture(device: &Device) -> Result<(ContinuationLearningWeights, NativeGeometricGenerate)> {
        let binding =
            SourceActionBinding::new(TOK.as_bytes()).map_err(|e| invalid(e.to_string()))?;
        let generate =
            GenerateLearningWeights::seeded(binding.clone(), 2, 37, device)?.export_native()?;
        let native_binding = NativeArtifactBinding {
            metadata_sha256: "a".repeat(64),
            identity: ArtifactIdentity {
                tokenizer_sha256: binding.tokenizer_sha256().into(),
                parent_checkpoint_manifest_sha256: "b".repeat(64),
                parent_model_sha256: "c".repeat(64),
                parent_config_sha256: "d".repeat(64),
            },
        };
        Ok((
            ContinuationLearningWeights::zeroed(&binding, &native_binding, 2, device)?,
            generate,
        ))
    }
    fn state(a: u8, b: u8) -> Result<Vec<H4Code>> {
        [a, b]
            .into_iter()
            .map(|v| H4Code::try_from(v).map_err(|e| invalid(e.to_string())))
            .collect()
    }
    fn patterned(weights: &ContinuationLearningWeights) -> Result<()> {
        weights.unary.set(&Tensor::from_vec(
            (0..weights.lanes * ROOT_COUNT)
                .map(|i| ((i * 7 + i / ROOT_COUNT * 3) % 15) as f32 * 0.25 - 1.75)
                .collect::<Vec<_>>(),
            (weights.lanes, ROOT_COUNT),
            weights.device(),
        )?)?;
        Ok(())
    }
    fn carrier(codes: &[H4Code], device: &Device) -> Result<Var> {
        let mut values = vec![0f32; codes.len() * ROOT_COUNT];
        for (lane, code) in codes.iter().enumerate() {
            values[lane * ROOT_COUNT + usize::from(code.index())] = 1.;
        }
        Ok(Var::from_vec(values, (codes.len(), ROOT_COUNT), device)?)
    }
    fn gradient(grads: &candle_core::backprop::GradStore, tensor: &Tensor) -> Result<Vec<f32>> {
        Ok(grads
            .get(tensor)
            .ok_or_else(|| invalid("continuation test gradient absent"))?
            .flatten_all()?
            .to_vec1::<f32>()?)
    }
    fn exp_bytes() -> Vec<u8> {
        (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect()
    }

    fn shared_action_copy_parity(device: &Device) -> Result<(Vec<i64>, Vec<f32>, Vec<f32>)> {
        let (legacy, generate) = fixture(device)?;
        let weights = ContinuationLearningWeights::zeroed_shared_action(
            legacy.binding(),
            legacy.native_binding(),
            2,
            device,
        )?;
        weights
            .unary
            .set(&Tensor::full(0.25f32, (2, ROOT_COUNT), device)?)?;
        let prepared = weights.prepare_native(weights.native_binding(), &generate)?;
        assert!(prepared.native.applies_to_copy());
        assert_eq!(prepared.native.score_shift(), 22);
        let restored = ContinuationLearningWeights::from_native(
            &prepared.native,
            &generate,
            weights.binding(),
            device,
        )?;
        assert!(restored.applies_to_copy());
        assert_eq!(restored.score_shift(), 22);
        assert_eq!(
            restored
                .export_native(restored.native_binding(), &generate)?
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?,
            prepared
                .native
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?
        );
        let delta =
            weights.forward_prepared_coefficients_only_on_device(&prepared, &state(31, 57)?)?;
        assert_eq!(
            delta.delta_scores_q24.to_vec1::<i64>()?,
            vec![2 << 22; weights.vocab_size()]
        );
        assert_eq!(
            delta.delta_raw_scores.to_vec1::<f32>()?,
            vec![0.5; weights.vocab_size()]
        );
        // A Copy-only upstream objective proves gathered U credit without GG.
        let copies = delta
            .delta_raw_scores
            .index_select(&Tensor::from_vec(vec![4u32, 4], 2, device)?, 0)?;
        let gradient = gradient(&copies.sum_all()?.backward()?, weights.unary.as_tensor())?;
        assert_eq!(gradient.iter().sum::<f32>(), 4.);
        assert!(legacy
            .forward_prepared_coefficients_only_on_device(&prepared, &state(31, 57)?)
            .is_err());
        Ok((
            delta.delta_scores_q24.to_vec1::<i64>()?,
            delta.delta_raw_scores.to_vec1::<f32>()?,
            gradient,
        ))
    }

    #[test]
    fn continuation_shared_action_restores_scale_and_copy_gather_credit() -> Result<()> {
        shared_action_copy_parity(&Device::Cpu)?;
        Ok(())
    }

    #[cfg(feature = "cuda")]
    #[test]
    #[ignore = "requires an explicitly leased CUDA device; unavailability is an error"]
    fn continuation_cuda_shared_action_units_and_copy_adjoint_match_cpu() -> Result<()> {
        let cuda = Device::new_cuda(0)?;
        assert_eq!(
            shared_action_copy_parity(&Device::Cpu)?,
            shared_action_copy_parity(&cuda)?
        );
        Ok(())
    }

    #[test]
    fn continuation_zero_preserves_native_generate_and_parent_binding() -> Result<()> {
        let (weights, generate) = fixture(&Device::Cpu)?;
        let prepared = weights.prepare_native(weights.native_binding(), &generate)?;
        let output = weights.forward_prepared_coefficients_only(&prepared, &state(119, 0)?)?;
        assert_eq!(output.delta_scores_q24, vec![0; weights.vocab_size()]);
        assert_eq!(
            output.delta_raw_scores.to_vec1::<f32>()?,
            vec![0.; weights.vocab_size()]
        );
        assert_eq!(output.costs.conditional_choice_rows, 0);
        let restored = ContinuationLearningWeights::from_native(
            &prepared.native,
            &generate,
            weights.binding(),
            &Device::Cpu,
        )?;
        assert_eq!(restored.native_binding(), weights.native_binding());
        assert_eq!(
            restored.packed_coefficients()?,
            weights.packed_coefficients()?
        );
        let mut wrong = weights.native_binding.clone();
        wrong.identity.parent_model_sha256 = "e".repeat(64);
        let wrong_weights =
            ContinuationLearningWeights::zeroed(weights.binding(), &wrong, 2, &Device::Cpu)?;
        assert!(wrong_weights
            .forward_prepared_coefficients_only(&prepared, &state(0, 0)?)
            .is_err());
        weights
            .unary
            .set(&Tensor::full(0.25f32, (2, ROOT_COUNT), &Device::Cpu)?)?;
        assert!(weights
            .forward_prepared_coefficients_only(&prepared, &state(0, 0)?)
            .is_err());
        Ok(())
    }

    #[test]
    fn continuation_device_gather_matches_native_all120_directed_states() -> Result<()> {
        let (weights, generate) = fixture(&Device::Cpu)?;
        patterned(&weights)?;
        let prepared = weights.prepare_native(weights.native_binding(), &generate)?;
        for code in 0..ROOT_COUNT {
            let codes = state(code as u8, (ROOT_COUNT - 1 - code) as u8)?;
            let cached = weights.forward_prepared_coefficients_only(&prepared, &codes)?;
            let reference = weights.forward_prepared_uncached_reference(&prepared, &codes, None)?;
            assert_eq!(cached.delta_scores_q24, reference.delta_scores_q24);
            assert_eq!(
                cached.delta_raw_scores.to_vec1::<f32>()?,
                reference.delta_raw_scores.to_vec1::<f32>()?
            );
        }
        Ok(())
    }

    fn coefficient_output_parity(device: &Device) -> Result<()> {
        let (weights, generate) = fixture(device)?;
        patterned(&weights)?;
        let prepared = weights.prepare_native(weights.native_binding(), &generate)?;
        let codes = state(119, 0)?;
        let host = weights.forward_prepared_coefficients_only(&prepared, &codes)?;
        let resident = weights.forward_prepared_coefficients_only_on_device(&prepared, &codes)?;
        assert_eq!(resident.delta_scores_q24.dtype(), DType::I64);
        assert!(resident.delta_scores_q24.device().same_device(device));
        assert!(resident.delta_raw_scores.device().same_device(device));
        assert_eq!(resident.costs.hard_score_download_bytes, 0);
        assert_eq!(resident.costs.conditional_choice_rows, 0);
        assert_eq!(
            resident.delta_scores_q24.to_vec1::<i64>()?,
            host.delta_scores_q24
        );
        assert_eq!(
            resident.delta_raw_scores.to_vec1::<f32>()?,
            host.delta_raw_scores.to_vec1::<f32>()?
        );
        // Signed, nonuniform upstream utilities expose indexing/adjoint drift
        // that an all-ones reduction can cancel.
        let utility = Tensor::from_vec(
            (0..weights.vocab_size())
                .map(|i| (i as f32 - 3.) * 0.375)
                .collect::<Vec<_>>(),
            weights.vocab_size(),
            device,
        )?;
        let hg = host.delta_raw_scores.mul(&utility)?.sum_all()?.backward()?;
        let dg = resident
            .delta_raw_scores
            .mul(&utility)?
            .sum_all()?
            .backward()?;
        assert_eq!(
            gradient(&hg, weights.unary.as_tensor())?,
            gradient(&dg, weights.unary.as_tensor())?
        );
        Ok(())
    }

    #[test]
    fn continuation_host_device_coefficients_values_and_gradients_match() -> Result<()> {
        coefficient_output_parity(&Device::Cpu)?;
        let (weights, generate) = fixture(&Device::Cpu)?;
        let prepared = weights.prepare_native(weights.native_binding(), &generate)?;
        weights
            .unary
            .set(&Tensor::full(0.25f32, (2, ROOT_COUNT), &Device::Cpu)?)?;
        assert!(weights
            .forward_prepared_coefficients_only_on_device(&prepared, &state(0, 0)?)
            .is_err());
        assert!(weights
            .forward_prepared_coefficients_only_on_device(&prepared, &state(0, 0)?[..1])
            .is_err());
        Ok(())
    }

    #[test]
    fn continuation_export_requires_explicit_current_receipt_and_same_base_parent() -> Result<()> {
        let (weights, generate) = fixture(&Device::Cpu)?;
        let mut current = weights.native_binding().clone();
        current.metadata_sha256 = "e".repeat(64);
        let prepared = weights.prepare_native(&current, &generate)?;
        assert_eq!(prepared.native_binding(), &current);
        assert_eq!(weights.native_binding().metadata_sha256, "a".repeat(64));
        prepared.validate_binding(&current)?;
        assert!(prepared.validate_binding(weights.native_binding()).is_err());
        weights.forward_prepared_coefficients_only(&prepared, &state(119, 0)?)?;
        current.identity.parent_model_sha256 = "f".repeat(64);
        assert!(weights.prepare_native(&current, &generate).is_err());
        Ok(())
    }

    #[test]
    fn continuation_coefficients_receive_ordinary_copy_generate_alias_loss_credit() -> Result<()> {
        let (weights, generate) = fixture(&Device::Cpu)?;
        let prepared = weights.prepare_native(weights.native_binding(), &generate)?;
        let local = state(83, 119)?;
        let delta = weights.forward_prepared_coefficients_only(&prepared, &local)?;
        let mut base = vec![0i64; weights.vocab_size()];
        generate
            .score_into(&state(0, 13)?, &mut base, &mut Default::default())
            .map_err(|e| invalid(e.to_string()))?;
        let full = base
            .iter()
            .zip(&delta.delta_scores_q24)
            .map(|(a, b)| a + b)
            .collect::<Vec<_>>();
        let base_graph = Tensor::from_vec(
            base.iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            weights.vocab_size(),
            &Device::Cpu,
        )?;
        let raw = (&base_graph + &delta.delta_raw_scores)?;
        let copy_raw = Tensor::from_vec(vec![0f32, 0.25], 2, &Device::Cpu)?;
        let mut pool = NativeVocabularyActions::new(weights.binding().clone(), &exp_bytes())
            .map_err(|e| invalid(e.to_string()))?;
        let trace = pool
            .reduce_trace(&full, &[5, 5], &[0, 1 << 22])
            .map_err(|e| invalid(e.to_string()))?;
        // Target4 has no Copy alias. It is inspected only after scoring all IDs.
        let loss = vocabulary_marginal_loss(&trace, &raw, Some(&copy_raw), 4)?;
        let grads = gradient(&loss.backward()?, weights.unary.as_tensor())?;
        assert!(grads.iter().any(|v| v.abs() > 0.));
        assert!(grads.iter().all(|v| v.is_finite()));
        // Coefficient-only exposes no local-state/prototype utility branch.
        assert_eq!(delta.costs.conditional_choice_rows, 0);
        let mut selected = vec![false; grads.len()];
        for lane in 0..weights.lanes() {
            for token in 0..weights.vocab_size() {
                let relative = generate
                    .algebra()
                    .relative(
                        local[lane].index(),
                        generate.algebra().identity(),
                        generate.prototypes()[token * weights.lanes() + lane],
                    )
                    .map_err(|e| invalid(e.to_string()))?;
                selected[lane * ROOT_COUNT + usize::from(relative)] = true;
            }
        }
        assert!(grads
            .iter()
            .zip(selected)
            .all(|(&g, present)| present || g == 0.));
        Ok(())
    }

    fn full120_parity(device: &Device) -> Result<()> {
        let (weights, generate) = fixture(device)?;
        patterned(&weights)?;
        let prepared = weights.prepare_native(weights.native_binding(), &generate)?;
        let codes = state(91, 119)?;
        let choices = carrier(&codes, device)?;
        let cached =
            weights.forward_prepared_state_choices(&prepared, &codes, choices.as_tensor())?;
        let reference = weights.forward_prepared_uncached_reference(
            &prepared,
            &codes,
            Some(choices.as_tensor()),
        )?;
        assert_eq!(cached.delta_scores_q24, reference.delta_scores_q24);
        assert_eq!(
            cached.delta_raw_scores.to_vec1::<f32>()?,
            reference.delta_raw_scores.to_vec1::<f32>()?
        );
        let cg = cached.delta_raw_scores.sum_all()?.backward()?;
        let rg = reference.delta_raw_scores.sum_all()?.backward()?;
        assert_eq!(
            gradient(&cg, weights.unary.as_tensor())?,
            gradient(&rg, weights.unary.as_tensor())?
        );
        let state_grad = gradient(&cg, choices.as_tensor())?;
        assert_eq!(state_grad, gradient(&rg, choices.as_tensor())?);
        assert!(state_grad.iter().any(|v| v.abs() > 0.));
        assert!(state_grad.iter().all(|v| v.is_finite()));
        assert!(weights
            .forward_prepared_state_choices(
                &prepared,
                &codes,
                &Tensor::zeros((2, ROOT_COUNT), DType::F32, device)?
            )
            .is_err());
        Ok(())
    }

    #[test]
    fn continuation_full120_local_state_and_coefficients_match_reference() -> Result<()> {
        full120_parity(&Device::Cpu)
    }

    #[cfg(feature = "cuda")]
    #[test]
    #[ignore = "requires an explicitly leased CUDA device; unavailability is an error"]
    fn continuation_cuda_i64_delta_and_full120_adjoint_match_cpu_reference() -> Result<()> {
        let cuda = Device::new_cuda(0)?;
        full120_parity(&cuda)?;
        coefficient_output_parity(&cuda)?;
        let (cpu_weights, cpu_generate) = fixture(&Device::Cpu)?;
        let (cuda_weights, cuda_generate) = fixture(&cuda)?;
        patterned(&cpu_weights)?;
        patterned(&cuda_weights)?;
        let cpu_prepared =
            cpu_weights.prepare_native(cpu_weights.native_binding(), &cpu_generate)?;
        let cuda_prepared =
            cuda_weights.prepare_native(cuda_weights.native_binding(), &cuda_generate)?;
        let codes = state(119, 0)?;
        let cpu = cpu_weights.forward_prepared_coefficients_only(&cpu_prepared, &codes)?;
        let gpu = cuda_weights.forward_prepared_coefficients_only(&cuda_prepared, &codes)?;
        assert_eq!(cpu.delta_scores_q24, gpu.delta_scores_q24);
        let gpu_resident =
            cuda_weights.forward_prepared_coefficients_only_on_device(&cuda_prepared, &codes)?;
        assert_eq!(
            gpu_resident.delta_scores_q24.to_vec1::<i64>()?,
            cpu.delta_scores_q24
        );
        assert_eq!(
            gpu_resident.delta_raw_scores.to_vec1::<f32>()?,
            cpu.delta_raw_scores.to_vec1::<f32>()?
        );
        assert_eq!(gpu_resident.costs.hard_score_download_bytes, 0);
        assert_eq!(
            cpu.delta_raw_scores.to_vec1::<f32>()?,
            gpu.delta_raw_scores.to_vec1::<f32>()?
        );
        assert_eq!(
            gpu.costs.hard_score_download_bytes,
            8 * cuda_weights.vocab_size()
        );
        Ok(())
    }
}
