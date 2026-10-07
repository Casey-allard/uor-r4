//! Offline learning of token H4 tuples and shared native Generate factors.
//!
//! Serving is the core integer/table model. This graph anchors its exact Q24
//! scores and differentiates selected quarter-grid coefficients separately
//! from conditional full-120 state and prototype choices. Other lanes and the
//! opposite endpoint stay factual. This is a local normalized-choice surrogate,
//! not a derivative of argmax or a full recurrent joint posterior. Earlier
//! context recurrence retains its own declared approximation.
//!
//! `prepare_native` is explicit source/export admission: it downloads masters
//! and selects hard prototypes. Refresh it and all graphs after every optimizer
//! update. CUDA forward never downloads dynamic probabilities or adjoints.
//! The caller binds state codes and state_logits to the same fresh causal replay;
//! this module cannot establish the token history from a tensor shape.

use std::collections::BTreeMap;

use candle_core::{DType, Device, Tensor, Var};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::{
    geometric_generate::{GenerateReadCounts, NativeGeometricGenerate, MAX_LANES, SCORE_SHIFT},
    integrated_attention::geometry::{EnergyTables, LanePair},
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_vocabulary_actions::{VocabularyAction, VocabularyActionTrace},
    h4_tables::{H4Code, ROOT_COUNT},
};

use crate::{
    geometric_context::{q4_project_tensor, q4_shadow_ste},
    invalid, Result,
};

pub const SURROGATE: &str = "native-Q24-hard-anchor;selected-quarter-grid-coefficient-STE;detached-factors-full120-local-state-and-token-code-choice-credit;other-lanes-and-opposite-endpoint-factual;existing-softmax-choice-differential;common-final-clip8nat;fresh-export-required/1";
const Q24: f64 = 16_777_216.;
// Quarter-valued shadows times this factor equal integer nibble <<20 /2^24.
const QUARTER_TO_NATS: f64 = 0.25;

pub struct GenerateLearningWeights {
    binding: SourceActionBinding,
    lanes: usize,
    edges: Vec<LanePair>,
    /// Unquantized offline choice logits, never included in native serving.
    pub prototype_choices: Var,
    /// Quarter-valued master coefficients; strict projected range +/-1.75.
    pub unary: Var,
    pub pair: Var,
    pub bias: Var,
}

/// Frozen export admission. Its native model, not live float argmax, chooses
/// prototypes and supplies factual scores. An optimizer update invalidates it.
pub struct PreparedGenerateLearning {
    pub native: NativeGeometricGenerate,
    pub downloaded_master_bytes: usize,
    pub prototype_choice_sha256: String,
    cache: GenerateDeviceIndexCache,
}

struct GenerateDeviceIndexCache {
    payload_sha256: String,
    // Rows are factual state IDs, columns are prototype IDs, values inv(s)*p.
    relative: Tensor,
    index_multiplier: Tensor,
    prototypes: Vec<Tensor>,
    // Rows token IDs, columns alternative state IDs; independent of position.
    state_relatives: Vec<Tensor>,
    code_probability: Tensor,
    staged_bytes: usize,
    device_index_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateLearningCosts {
    pub vocabulary_rows: usize,
    pub lanes: usize,
    pub ordered_pairs: usize,
    pub conditional_choice_rows: usize,
    /// Static CPU geometry indices staged to the live device, not adjoints.
    pub staged_index_bytes: usize,
    pub staged_hard_score_bytes: usize,
    pub export_master_download_bytes: usize,
    pub max_conditional_utility_elements: usize,
    /// Zero CUDA dynamic-vector downloads; one scalar finite-status read in
    /// loss is reported separately from explicit prepare/export downloads.
    pub loss_status_scalars: usize,
    pub snapshot_staged_index_bytes: usize,
    pub snapshot_device_index_bytes: usize,
    pub per_position_device_index_elements: usize,
    pub snapshot_choice_probability_bytes: usize,
    pub generate_only_loss_staged_bytes_upper: usize,
}

pub struct GenerateLearningOutput {
    /// Full token-ID order including sparse holes; pool owns legal admission.
    pub raw_scores: Tensor,
    pub clipped_scores: Tensor,
    pub scores_q24: Vec<i64>,
    pub native_counts: GenerateReadCounts,
    pub costs: GenerateLearningCosts,
}

fn device_admit(device: &Device) -> Result<()> {
    if !device.is_cpu() && !device.is_cuda() {
        return Err(invalid(
            "Generate learning supports CPU reference or CUDA only",
        ));
    }
    #[cfg(not(feature = "cuda"))]
    if device.is_cuda() {
        return Err(invalid("Generate CUDA requested without cuda feature"));
    }
    Ok(())
}

fn quarter_codes(value: &Var) -> Result<Vec<i8>> {
    let values = value.flatten_all()?.to_vec1::<f32>()?;
    if values.iter().any(|x| !x.is_finite() || x.abs() > 1.75) {
        return Err(invalid(
            "Generate export coefficient outside strict quarter range",
        ));
    }
    Ok(values.into_iter().map(|x| (x * 4.).round() as i8).collect())
}

fn packed_nibbles(values: &[i8]) -> Result<Vec<u8>> {
    if values.iter().any(|x| !(-7..=7).contains(x)) {
        return Err(invalid(
            "Generate bias coefficient outside strict nibble range",
        ));
    }
    let mut out = vec![0; values.len().div_ceil(2)];
    for (i, &v) in values.iter().enumerate() {
        out[i / 2] |= ((v as u8) & 15) << ((i & 1) * 4);
    }
    Ok(out)
}

fn choice_digest(values: &[f32]) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.to_bits().to_le_bytes());
    }
    hex::encode(hash.finalize())
}

fn initial_token_geometry(vocab: usize, lanes: usize, seed: u64, balanced: bool) -> Vec<u8> {
    let mut prototypes = Vec::with_capacity(vocab * lanes);
    for token in 0..vocab {
        let mut digits = token;
        for lane in 0..lanes {
            // Preserve the existing seed/lane offset exactly, including its
            // host usize seed cast; this remains an offline initializer.
            let offset = ((seed as usize % ROOT_COUNT) + lane * 17) % ROOT_COUNT;
            let value = if balanced {
                token % ROOT_COUNT + lane * (token / ROOT_COUNT)
            } else {
                digits % ROOT_COUNT
            };
            prototypes.push(((value + offset) % ROOT_COUNT) as u8);
            digits /= ROOT_COUNT;
        }
    }
    prototypes
}

impl GenerateLearningWeights {
    /// Seeded nonzero shared potentials and distinct base-120 tuples where
    /// cardinality permits. Arbitrary initialization has no semantic claim.
    pub fn seeded(
        binding: SourceActionBinding,
        lanes: usize,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        Self::seeded_geometry(binding, lanes, seed, device, false)
    }

    /// Default-off label-free balanced token geometry. For t=a+120b,
    /// lane j receives (a+j*b+the existing seed/lane offset) mod120.
    /// Every disjoint adjacent pair has determinant1 and distinguishes all
    /// admitted token IDs up to4096. This is initialization, not semantics.
    /// Energy draw order, pair graph, coefficient margins and prototype gap
    /// are exactly the same as seeded().
    pub fn seeded_balanced_token_geometry(
        binding: SourceActionBinding,
        lanes: usize,
        seed: u64,
        device: &Device,
    ) -> Result<Self> {
        Self::seeded_geometry(binding, lanes, seed, device, true)
    }

    fn seeded_geometry(
        binding: SourceActionBinding,
        lanes: usize,
        seed: u64,
        device: &Device,
        balanced: bool,
    ) -> Result<Self> {
        device_admit(device)?;
        if !(1..=MAX_LANES).contains(&lanes) || !(1..=4096).contains(&binding.vocab_size()) {
            return Err(invalid("Generate seed configuration differs"));
        }
        let edges = (0..lanes / 2)
            .map(|i| LanePair {
                left: (2 * i) as u8,
                right: (2 * i + 1) as u8,
            })
            .collect::<Vec<_>>();
        let mut rng = seed ^ 0x9e3779b97f4a7c15;
        if rng == 0 {
            rng = 1;
        }
        let mut draw = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            (rng % 5) as i8 - 2
        };
        let mut energy =
            EnergyTables::zeroed(lanes as u8, edges).map_err(|e| invalid(e.to_string()))?;
        for l in 0..lanes {
            for r in 0..ROOT_COUNT {
                energy
                    .set_unary(l as u8, r as u8, draw())
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        for e in 0..energy.edges().len() {
            for a in 0..ROOT_COUNT {
                for b in 0..ROOT_COUNT {
                    energy
                        .set_pair(e, a as u8, b as u8, draw())
                        .map_err(|e| invalid(e.to_string()))?;
                }
            }
        }
        let v = binding.vocab_size();
        let prototypes = initial_token_geometry(v, lanes, seed, balanced);
        let native = NativeGeometricGenerate::compile(
            &binding,
            lanes,
            &prototypes,
            &vec![0; v.div_ceil(2)],
            energy,
        )
        .map_err(|e| invalid(e.to_string()))?;
        Self::from_native(binding, &native, device)
    }

    pub fn from_native(
        binding: SourceActionBinding,
        native: &NativeGeometricGenerate,
        device: &Device,
    ) -> Result<Self> {
        device_admit(device)?;
        // Re-admit exact bytes to establish tokenizer/algebra/payload identity.
        let native = NativeGeometricGenerate::from_bytes(
            &native.to_bytes().map_err(|e| invalid(e.to_string()))?,
            &binding,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let v = native.vocab_size();
        let l = native.lanes();
        let edges = native.energy().edges().to_vec();
        let mut choices = vec![0f32; v * l * ROOT_COUNT];
        for (i, &r) in native.prototypes().iter().enumerate() {
            choices[i * ROOT_COUNT + usize::from(r)] = 2.;
        }
        let mut unary = Vec::with_capacity(l * ROOT_COUNT);
        for lane in 0..l {
            for r in 0..ROOT_COUNT {
                unary.push(
                    f32::from(
                        native
                            .energy()
                            .get_unary(lane as u8, r as u8)
                            .map_err(|e| invalid(e.to_string()))?,
                    ) * 0.25,
                );
            }
        }
        let mut pair = Vec::with_capacity(edges.len() * ROOT_COUNT * ROOT_COUNT);
        for e in 0..edges.len() {
            for a in 0..ROOT_COUNT {
                for b in 0..ROOT_COUNT {
                    pair.push(
                        f32::from(
                            native
                                .energy()
                                .get_pair(e, a as u8, b as u8)
                                .map_err(|e| invalid(e.to_string()))?,
                        ) * 0.25,
                    );
                }
            }
        }
        let bias = (0..v)
            .map(|t| {
                native
                    .token_bias(t)
                    .map(|x| f32::from(x) * 0.25)
                    .map_err(|e| invalid(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            binding,
            lanes: l,
            edges,
            prototype_choices: Var::from_vec(choices, (v, l, ROOT_COUNT), device)?,
            unary: Var::from_vec(unary, (l, ROOT_COUNT), device)?,
            pair: Var::from_vec(
                pair,
                (native.energy().edges().len(), ROOT_COUNT, ROOT_COUNT),
                device,
            )?,
            bias: Var::from_vec(bias, v, device)?,
        })
    }

    pub fn device(&self) -> &Device {
        self.unary.device()
    }
    pub fn lanes(&self) -> usize {
        self.lanes
    }
    pub fn vocab_size(&self) -> usize {
        self.binding.vocab_size()
    }
    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn parameters(&self) -> BTreeMap<String, Var> {
        [
            ("generate.prototype_choices", self.prototype_choices.clone()),
            ("generate.unary", self.unary.clone()),
            ("generate.pair", self.pair.clone()),
            ("generate.bias", self.bias.clone()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
    }
    /// Project coefficient masters only. Prototype logits are unquantized;
    /// explicit export rejects nonfinite values before native admission.
    pub fn project_shadow_range(&self) -> Result<()> {
        let updates = [&self.unary, &self.pair, &self.bias]
            .into_iter()
            .filter(|v| v.elem_count() != 0)
            .map(|v| Ok((v, q4_project_tensor(v.as_tensor())?)))
            .collect::<Result<Vec<_>>>()?;
        for (var, value) in updates {
            var.set(&value)?;
        }
        Ok(())
    }
    pub fn export_native(&self) -> Result<NativeGeometricGenerate> {
        Ok(self.export_native_with_choice_digest()?.0)
    }
    fn export_native_with_choice_digest(&self) -> Result<(NativeGeometricGenerate, String)> {
        self.validate_shapes()?;
        let choices = self.prototype_choices.flatten_all()?.to_vec1::<f32>()?;
        if choices.iter().any(|x| !x.is_finite()) {
            return Err(invalid("Generate prototype master nonfinite"));
        }
        let choice_digest = choice_digest(&choices);
        let mut prototypes = Vec::with_capacity(self.vocab_size() * self.lanes);
        for row in choices.chunks_exact(ROOT_COUNT) {
            let mut chosen = 0;
            for r in 1..ROOT_COUNT {
                if row[r] > row[chosen] {
                    chosen = r;
                }
            }
            prototypes.push(chosen as u8);
        }
        let u = quarter_codes(&self.unary)?;
        let p = quarter_codes(&self.pair)?;
        let b = quarter_codes(&self.bias)?;
        let mut energy = EnergyTables::zeroed(self.lanes as u8, self.edges.clone())
            .map_err(|e| invalid(e.to_string()))?;
        for l in 0..self.lanes {
            for r in 0..ROOT_COUNT {
                energy
                    .set_unary(l as u8, r as u8, u[l * ROOT_COUNT + r])
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        for e in 0..self.edges.len() {
            for a in 0..ROOT_COUNT {
                for b in 0..ROOT_COUNT {
                    energy
                        .set_pair(
                            e,
                            a as u8,
                            b as u8,
                            p[(e * ROOT_COUNT + a) * ROOT_COUNT + b],
                        )
                        .map_err(|e| invalid(e.to_string()))?;
                }
            }
        }
        let native = NativeGeometricGenerate::compile(
            &self.binding,
            self.lanes,
            &prototypes,
            &packed_nibbles(&b)?,
            energy,
        )
        .map_err(|e| invalid(e.to_string()))?;
        Ok((native, choice_digest))
    }
    pub fn prepare_native(&self) -> Result<PreparedGenerateLearning> {
        let (native, prototype_choice_sha256) = self.export_native_with_choice_digest()?;
        let v = self.vocab_size();
        let mut relation = Vec::with_capacity(ROOT_COUNT * ROOT_COUNT);
        for state in 0..ROOT_COUNT {
            for prototype in 0..ROOT_COUNT {
                relation.push(u32::from(
                    native
                        .algebra()
                        .relative(state as u8, native.algebra().identity(), prototype as u8)
                        .map_err(|e| invalid(e.to_string()))?,
                ));
            }
        }
        let relative = Tensor::from_vec(relation, (ROOT_COUNT, ROOT_COUNT), self.device())?;
        let transposed = relative.t()?.contiguous()?;
        let mut prototypes = Vec::with_capacity(self.lanes);
        let mut state_relatives = Vec::with_capacity(self.lanes);
        for lane in 0..self.lanes {
            let ids = (0..v)
                .map(|token| u32::from(native.prototypes()[token * self.lanes + lane]))
                .collect::<Vec<_>>();
            let ids = Tensor::from_vec(ids, v, self.device())?;
            state_relatives.push(transposed.index_select(&ids, 0)?);
            prototypes.push(ids);
        }
        let payload_sha256 = native.metadata().payload_sha256.clone();
        Ok(PreparedGenerateLearning {
            native,
            prototype_choice_sha256,
            downloaded_master_bytes: 4
                * (self.prototype_choices.elem_count()
                    + self.unary.elem_count()
                    + self.pair.elem_count()
                    + self.bias.elem_count()),
            cache: GenerateDeviceIndexCache {
                payload_sha256,
                relative,
                index_multiplier: Tensor::new(ROOT_COUNT as u32, self.device())?,
                prototypes,
                state_relatives,
                code_probability: candle_nn::ops::softmax(self.prototype_choices.as_tensor(), 2)?,
                staged_bytes: 4 * (ROOT_COUNT * ROOT_COUNT + v * self.lanes + 1),
                device_index_bytes: 4
                    * (ROOT_COUNT * ROOT_COUNT + v * self.lanes + v * self.lanes * ROOT_COUNT + 1),
            },
        })
    }
    fn validate_shapes(&self) -> Result<()> {
        device_admit(self.device())?;
        if self.prototype_choices.dims() != [self.vocab_size(), self.lanes, ROOT_COUNT]
            || self.unary.dims() != [self.lanes, ROOT_COUNT]
            || self.pair.dims() != [self.edges.len(), ROOT_COUNT, ROOT_COUNT]
            || self.bias.dims() != [self.vocab_size()]
            || self
                .parameters()
                .values()
                .any(|v| v.dtype() != DType::F32 || !v.device().same_device(self.device()))
        {
            return Err(invalid("Generate master shape/device differs"));
        }
        Ok(())
    }

    /// No labels enter scoring or admission. CUDA snapshot freshness and the
    /// causal state graph binding are explicit caller responsibilities.
    pub fn forward_prepared(
        &self,
        prepared: &PreparedGenerateLearning,
        state: &[H4Code],
        state_logits: &Tensor,
    ) -> Result<GenerateLearningOutput> {
        self.forward_prepared_state_input(prepared, state, state_logits, false)
    }

    /// Explicit finite-state utility channel from the same causal context op.
    /// Its factual value is a retained-state onehot; its adjoint is a120-state
    /// utility, not already-softmaxed transition-score credit. This replaces
    /// the old state-logit attachment rather than adding duplicate credit.
    pub fn forward_prepared_state_choices(
        &self,
        prepared: &PreparedGenerateLearning,
        state: &[H4Code],
        state_choices: &Tensor,
    ) -> Result<GenerateLearningOutput> {
        self.forward_prepared_state_input(prepared, state, state_choices, true)
    }

    /// Frozen factual-state diagnostic: only selected Q4 field coefficients
    /// receive credit. Snapshot choice cache preparation remains paid, but no
    /// state/prototype utility branch is constructed or attached here.
    /// CUDA snapshot freshness is the caller's responsibility.
    pub fn forward_prepared_coefficients_only(
        &self,
        prepared: &PreparedGenerateLearning,
        state: &[H4Code],
    ) -> Result<GenerateLearningOutput> {
        self.validate_shapes()?;
        let native = &prepared.native;
        let cache = &prepared.cache;
        if native.lanes() != self.lanes
            || native.vocab_size() != self.vocab_size()
            || native.metadata().score_shift != SCORE_SHIFT
            || native.energy().edges() != self.edges
            || native.metadata().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || state.len() != self.lanes
            || !cache.relative.device().same_device(self.device())
            || native.metadata().payload_sha256 != cache.payload_sha256
        {
            return Err(invalid(
                "Generate coefficient-only snapshot/state binding differs",
            ));
        }
        if self.device().is_cpu()
            && self
                .export_native()?
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?
                != native.to_bytes().map_err(|e| invalid(e.to_string()))?
        {
            return Err(invalid("Generate coefficient-only payload is stale"));
        }
        let v = self.vocab_size();
        let mut counts = GenerateReadCounts::default();
        let mut hard = vec![0i64; v];
        native
            .score_into(state, &mut hard, &mut counts)
            .map_err(|e| invalid(e.to_string()))?;
        let unary = (q4_shadow_ste(self.unary.as_tensor())? * QUARTER_TO_NATS)?;
        let pair = if self.edges.is_empty() {
            self.pair.as_tensor().clone()
        } else {
            (q4_shadow_ste(self.pair.as_tensor())? * QUARTER_TO_NATS)?
        };
        let mut coefficient = (q4_shadow_ste(self.bias.as_tensor())? * QUARTER_TO_NATS)?;
        let mut factual = Vec::with_capacity(self.lanes);
        for lane in 0..self.lanes {
            let ids = cache
                .relative
                .narrow(0, usize::from(state[lane].index()), 1)?
                .reshape(ROOT_COUNT)?
                .index_select(&cache.prototypes[lane], 0)?;
            coefficient = (&coefficient
                + unary
                    .narrow(0, lane, 1)?
                    .reshape(ROOT_COUNT)?
                    .index_select(&ids, 0)?)?;
            factual.push(ids);
        }
        for (edge, endpoints) in self.edges.iter().enumerate() {
            let ids = pair_device_indices(
                &factual[usize::from(endpoints.left)],
                &factual[usize::from(endpoints.right)],
                &cache.index_multiplier,
            )?;
            coefficient = (&coefficient
                + pair
                    .narrow(0, edge, 1)?
                    .reshape(ROOT_COUNT * ROOT_COUNT)?
                    .index_select(&ids, 0)?)?;
        }
        let anchor = Tensor::from_vec(
            hard.iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            v,
            self.device(),
        )?;
        let raw_scores = (&anchor + (&coefficient - coefficient.detach())?)?;
        let clipped_scores = raw_scores.clamp(-8f32, 8f32)?;
        Ok(GenerateLearningOutput {
            raw_scores,
            clipped_scores,
            scores_q24: hard,
            native_counts: counts,
            costs: GenerateLearningCosts {
                vocabulary_rows: v,
                lanes: self.lanes,
                ordered_pairs: self.edges.len(),
                conditional_choice_rows: 0,
                staged_index_bytes: 0,
                staged_hard_score_bytes: 4 * v,
                export_master_download_bytes: prepared.downloaded_master_bytes,
                max_conditional_utility_elements: 0,
                loss_status_scalars: 2,
                snapshot_staged_index_bytes: cache.staged_bytes,
                snapshot_device_index_bytes: cache.device_index_bytes,
                per_position_device_index_elements: v * (self.lanes + self.edges.len()),
                snapshot_choice_probability_bytes: 4 * v * self.lanes * ROOT_COUNT,
                generate_only_loss_staged_bytes_upper: 16 * v + 4,
            },
        })
    }

    fn forward_prepared_state_input(
        &self,
        prepared: &PreparedGenerateLearning,
        state: &[H4Code],
        state_logits: &Tensor,
        explicit_choices: bool,
    ) -> Result<GenerateLearningOutput> {
        self.validate_shapes()?;
        let native = &prepared.native;
        let cache = &prepared.cache;
        if native.lanes() != self.lanes
            || native.vocab_size() != self.vocab_size()
            || native.metadata().score_shift != SCORE_SHIFT
            || native.energy().edges() != self.edges
            || native.metadata().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || state.len() != self.lanes
            || state_logits.dims() != [self.lanes, ROOT_COUNT]
            || state_logits.dtype() != DType::F32
            || !state_logits.device().same_device(self.device())
            || !cache.relative.device().same_device(self.device())
            || native.metadata().payload_sha256 != cache.payload_sha256
        {
            return Err(invalid(
                "Generate cached prepared state/binding/device differs",
            ));
        }
        if self.device().is_cpu() {
            if choice_digest(&self.prototype_choices.flatten_all()?.to_vec1::<f32>()?)
                != prepared.prototype_choice_sha256
            {
                return Err(invalid("Generate cached choice probabilities are stale"));
            }
            if self
                .export_native()?
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?
                != native.to_bytes().map_err(|e| invalid(e.to_string()))?
            {
                return Err(invalid("Generate prepared payload is stale"));
            }
            if state_logits
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .any(|x| !x.is_finite())
            {
                return Err(invalid("Generate state choices nonfinite"));
            }
        }
        let v = self.vocab_size();
        let mut counts = GenerateReadCounts::default();
        let mut hard = vec![0i64; v];
        native
            .score_into(state, &mut hard, &mut counts)
            .map_err(|e| invalid(e.to_string()))?;
        let u = (q4_shadow_ste(self.unary.as_tensor())? * QUARTER_TO_NATS)?;
        let p = if self.edges.is_empty() {
            self.pair.as_tensor().clone()
        } else {
            (q4_shadow_ste(self.pair.as_tensor())? * QUARTER_TO_NATS)?
        };
        let mut coefficient = (q4_shadow_ste(self.bias.as_tensor())? * QUARTER_TO_NATS)?;
        let mut factual_relatives = Vec::with_capacity(self.lanes);
        let mut code_relatives = Vec::with_capacity(self.lanes);
        let mut device_index_elements = 0usize;
        for lane in 0..self.lanes {
            let row = cache
                .relative
                .narrow(0, usize::from(state[lane].index()), 1)?
                .reshape(ROOT_COUNT)?;
            let factual = row.index_select(&cache.prototypes[lane], 0)?;
            coefficient = (&coefficient
                + u.narrow(0, lane, 1)?
                    .reshape(ROOT_COUNT)?
                    .index_select(&factual, 0)?)?;
            device_index_elements += v;
            factual_relatives.push(factual);
            code_relatives.push(row);
        }
        for (e, edge) in self.edges.iter().enumerate() {
            let ids = pair_device_indices(
                &factual_relatives[usize::from(edge.left)],
                &factual_relatives[usize::from(edge.right)],
                &cache.index_multiplier,
            )?;
            coefficient = (&coefficient
                + p.narrow(0, e, 1)?
                    .reshape(ROOT_COUNT * ROOT_COUNT)?
                    .index_select(&ids, 0)?)?;
            device_index_elements += v;
        }
        let state_probability = if explicit_choices {
            if self.device().is_cpu() {
                let actual = state_logits.flatten_all()?.to_vec1::<f32>()?;
                if actual.iter().enumerate().any(|(i, &v)| {
                    v != if i % ROOT_COUNT == usize::from(state[i / ROOT_COUNT].index()) {
                        1.
                    } else {
                        0.
                    }
                }) {
                    return Err(invalid(
                        "Generate retained state-choice onehot differs from factual state",
                    ));
                }
            }
            state_logits.clone()
        } else {
            candle_nn::ops::softmax(state_logits, 1)?
        };
        let ud = u.detach();
        let pd = p.detach();
        let mut input = Tensor::zeros(v, DType::F32, self.device())?;
        for lane in 0..self.lanes {
            let sr = &cache.state_relatives[lane];
            let cr = code_relatives[lane].reshape((1, ROOT_COUNT))?;
            let unary = ud.narrow(0, lane, 1)?.reshape(ROOT_COUNT)?;
            let mut su = unary
                .index_select(&sr.flatten_all()?, 0)?
                .reshape((v, ROOT_COUNT))?;
            // Unary code utility is shared across every token: no V-fold copy
            // of its index map or utility is needed until a pair term varies.
            let mut cu = unary
                .index_select(&cr.flatten_all()?, 0)?
                .reshape((1, ROOT_COUNT))?;
            for (e, edge) in self.edges.iter().enumerate() {
                let left = usize::from(edge.left);
                let right = usize::from(edge.right);
                if lane != left && lane != right {
                    continue;
                }
                let other = if lane == left { right } else { left };
                let other = factual_relatives[other].reshape((v, 1))?;
                let (sa, sb, ca, cb) = if lane == left {
                    (sr, &other, &cr, &other)
                } else {
                    (&other, sr, &other, &cr)
                };
                let si = pair_device_indices(sa, sb, &cache.index_multiplier)?.flatten_all()?;
                let ci = pair_device_indices(ca, cb, &cache.index_multiplier)?.flatten_all()?;
                let table = pd.narrow(0, e, 1)?.reshape(ROOT_COUNT * ROOT_COUNT)?;
                su = (&su + table.index_select(&si, 0)?.reshape((v, ROOT_COUNT))?)?;
                cu = cu.broadcast_add(&table.index_select(&ci, 0)?.reshape((v, ROOT_COUNT))?)?;
                device_index_elements += 2 * v * ROOT_COUNT;
            }
            let sp = state_probability.narrow(0, lane, 1)?;
            let cp = cache
                .code_probability
                .narrow(1, lane, 1)?
                .reshape((v, ROOT_COUNT))?;
            input = (&input + su.broadcast_mul(&sp)?.sum(1)?)?;
            input = (&input + cu.broadcast_mul(&cp)?.sum(1)?)?;
        }
        let anchor = Tensor::from_vec(
            hard.iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            v,
            self.device(),
        )?;
        let raw_scores =
            ((&anchor + (&coefficient - coefficient.detach())?)? + (&input - input.detach())?)?;
        let clipped_scores = raw_scores.clamp(-8f32, 8f32)?;
        Ok(GenerateLearningOutput {
            raw_scores,
            clipped_scores,
            scores_q24: hard,
            native_counts: counts,
            costs: GenerateLearningCosts {
                vocabulary_rows: v,
                lanes: self.lanes,
                ordered_pairs: self.edges.len(),
                conditional_choice_rows: 2 * v * self.lanes,
                staged_index_bytes: 0,
                staged_hard_score_bytes: 4 * v,
                export_master_download_bytes: prepared.downloaded_master_bytes,
                max_conditional_utility_elements: v * ROOT_COUNT,
                loss_status_scalars: 2,
                snapshot_staged_index_bytes: cache.staged_bytes,
                snapshot_device_index_bytes: cache.device_index_bytes,
                per_position_device_index_elements: device_index_elements,
                snapshot_choice_probability_bytes: 4 * v * self.lanes * ROOT_COUNT,
                generate_only_loss_staged_bytes_upper: 16 * v + 4,
            },
        })
    }

    /// Deliberately uncached CPU reference for exact index/gradient parity.
    pub fn forward_prepared_uncached_reference(
        &self,
        prepared: &PreparedGenerateLearning,
        state: &[H4Code],
        state_logits: &Tensor,
    ) -> Result<GenerateLearningOutput> {
        if !self.device().is_cpu() {
            return Err(invalid("uncached Generate path is CPU reference only"));
        }
        self.validate_shapes()?;
        let native = &prepared.native;
        if native.lanes() != self.lanes
            || native.vocab_size() != self.vocab_size()
            || native.metadata().score_shift != SCORE_SHIFT
            || native.energy().edges() != self.edges
            || native.metadata().tokenizer_sha256 != self.binding.tokenizer_sha256()
            || state.len() != self.lanes
            || state_logits.dims() != [self.lanes, ROOT_COUNT]
            || state_logits.dtype() != DType::F32
            || !state_logits.device().same_device(self.device())
        {
            return Err(invalid("Generate prepared state/binding/shape differs"));
        }
        if self.device().is_cpu() {
            if self
                .export_native()?
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?
                != native.to_bytes().map_err(|e| invalid(e.to_string()))?
            {
                return Err(invalid("Generate prepared payload is stale"));
            }
            if state_logits
                .flatten_all()?
                .to_vec1::<f32>()?
                .iter()
                .any(|x| !x.is_finite())
            {
                return Err(invalid("Generate state choices nonfinite"));
            }
        }
        let v = self.vocab_size();
        let mut counts = GenerateReadCounts::default();
        let mut hard = vec![0i64; v];
        native
            .score_into(state, &mut hard, &mut counts)
            .map_err(|e| invalid(e.to_string()))?;
        let u = (q4_shadow_ste(self.unary.as_tensor())? * QUARTER_TO_NATS)?.flatten_all()?;
        let p = if self.edges.is_empty() {
            self.pair.as_tensor().flatten_all()?
        } else {
            (q4_shadow_ste(self.pair.as_tensor())? * QUARTER_TO_NATS)?.flatten_all()?
        };
        let b = (q4_shadow_ste(self.bias.as_tensor())? * QUARTER_TO_NATS)?;
        let mut indices_bytes = 0;
        let mut relative = vec![0u8; v * self.lanes];
        for token in 0..v {
            for lane in 0..self.lanes {
                relative[token * self.lanes + lane] = native
                    .algebra()
                    .relative(
                        state[lane].index(),
                        native.algebra().identity(),
                        native.prototypes()[token * self.lanes + lane],
                    )
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        // Selected hard coefficient gradient only. Input utility factors below
        // are detached so alternative choices cannot double-count coefficients.
        let mut coefficient = b;
        for lane in 0..self.lanes {
            let ids = (0..v)
                .map(|t| (lane * ROOT_COUNT + usize::from(relative[t * self.lanes + lane])) as u32)
                .collect::<Vec<_>>();
            coefficient = (&coefficient + gather(&u, ids, self.device(), &mut indices_bytes)?)?;
        }
        for (e, edge) in self.edges.iter().enumerate() {
            let ids = (0..v)
                .map(|t| {
                    ((e * ROOT_COUNT
                        + usize::from(relative[t * self.lanes + usize::from(edge.left)]))
                        * ROOT_COUNT
                        + usize::from(relative[t * self.lanes + usize::from(edge.right)]))
                        as u32
                })
                .collect::<Vec<_>>();
            coefficient = (&coefficient + gather(&p, ids, self.device(), &mut indices_bytes)?)?;
        }
        let ud = u.detach();
        let pd = p.detach();
        let state_probability = candle_nn::ops::softmax(state_logits, 1)?;
        let code_probability = candle_nn::ops::softmax(self.prototype_choices.as_tensor(), 2)?;
        let mut input = Tensor::zeros(v, DType::F32, self.device())?;
        for lane in 0..self.lanes {
            // Each utility is V x120, not a four-dimensional projection. Only
            // terms touching this lane vary; common terms cancel in pullback.
            let mut state_ids = Vec::with_capacity(v * ROOT_COUNT);
            let mut code_ids = Vec::with_capacity(v * ROOT_COUNT);
            let mut sr = Vec::with_capacity(v * ROOT_COUNT);
            let mut cr = Vec::with_capacity(v * ROOT_COUNT);
            for token in 0..v {
                for r in 0..ROOT_COUNT {
                    let srel = native
                        .algebra()
                        .relative(
                            r as u8,
                            native.algebra().identity(),
                            native.prototypes()[token * self.lanes + lane],
                        )
                        .map_err(|e| invalid(e.to_string()))?;
                    let crel = native
                        .algebra()
                        .relative(state[lane].index(), native.algebra().identity(), r as u8)
                        .map_err(|e| invalid(e.to_string()))?;
                    state_ids.push((lane * ROOT_COUNT + usize::from(srel)) as u32);
                    code_ids.push((lane * ROOT_COUNT + usize::from(crel)) as u32);
                    sr.push(srel);
                    cr.push(crel);
                }
            }
            let mut su = gather(&ud, state_ids, self.device(), &mut indices_bytes)?
                .reshape((v, ROOT_COUNT))?;
            let mut cu = gather(&ud, code_ids, self.device(), &mut indices_bytes)?
                .reshape((v, ROOT_COUNT))?;
            for (e, edge) in self.edges.iter().enumerate() {
                let left = usize::from(edge.left);
                let right = usize::from(edge.right);
                if lane != left && lane != right {
                    continue;
                }
                let map = |alternatives: &[u8]| {
                    (0..v * ROOT_COUNT)
                        .map(|i| {
                            let t = i / ROOT_COUNT;
                            let a = if lane == left {
                                alternatives[i]
                            } else {
                                relative[t * self.lanes + left]
                            };
                            let b = if lane == right {
                                alternatives[i]
                            } else {
                                relative[t * self.lanes + right]
                            };
                            ((e * ROOT_COUNT + usize::from(a)) * ROOT_COUNT + usize::from(b)) as u32
                        })
                        .collect::<Vec<_>>()
                };
                su = (&su
                    + gather(&pd, map(&sr), self.device(), &mut indices_bytes)?
                        .reshape((v, ROOT_COUNT))?)?;
                cu = (&cu
                    + gather(&pd, map(&cr), self.device(), &mut indices_bytes)?
                        .reshape((v, ROOT_COUNT))?)?;
            }
            let sp = state_probability.narrow(0, lane, 1)?;
            let cp = code_probability
                .narrow(1, lane, 1)?
                .reshape((v, ROOT_COUNT))?;
            input = (&input + su.broadcast_mul(&sp)?.sum(1)?)?;
            input = (&input + (cu * cp)?.sum(1)?)?;
        }
        let anchor = Tensor::from_vec(
            hard.iter()
                .map(|&x| (x as f64 / Q24) as f32)
                .collect::<Vec<_>>(),
            v,
            self.device(),
        )?;
        let coefficient_delta = (&coefficient - coefficient.detach())?;
        let input_delta = (&input - input.detach())?;
        let raw_scores = ((&anchor + coefficient_delta)? + input_delta)?;
        let clipped_scores = raw_scores.clamp(-8f32, 8f32)?;
        Ok(GenerateLearningOutput {
            raw_scores,
            clipped_scores,
            scores_q24: hard,
            native_counts: counts,
            costs: GenerateLearningCosts {
                vocabulary_rows: v,
                lanes: self.lanes,
                ordered_pairs: self.edges.len(),
                conditional_choice_rows: v * self.lanes * 2,
                staged_index_bytes: indices_bytes,
                staged_hard_score_bytes: v * 4,
                export_master_download_bytes: prepared.downloaded_master_bytes,
                max_conditional_utility_elements: v * ROOT_COUNT,
                loss_status_scalars: 2,
                snapshot_staged_index_bytes: prepared.cache.staged_bytes,
                snapshot_device_index_bytes: prepared.cache.device_index_bytes,
                per_position_device_index_elements: 0,
                snapshot_choice_probability_bytes: 4 * v * self.lanes * ROOT_COUNT,
                generate_only_loss_staged_bytes_upper: 16 * v + 4,
            },
        })
    }
}

// Offline device U32 arithmetic only. All inputs are exact group IDs <120,
// hence resulting flat indices are <14400 and overflow cannot occur. Pair
// orientation stays declared left then right. No float index approximation.
fn pair_device_indices(left: &Tensor, right: &Tensor, multiplier: &Tensor) -> Result<Tensor> {
    Ok(left.broadcast_mul(multiplier)?.broadcast_add(right)?)
}

fn gather(table: &Tensor, ids: Vec<u32>, device: &Device, bytes: &mut usize) -> Result<Tensor> {
    *bytes = bytes
        .checked_add(ids.len() * 4)
        .ok_or_else(|| invalid("Generate staged index size overflow"))?;
    let n = ids.len();
    Ok(table.index_select(&Tensor::from_vec(ids, n, device)?, 0)?)
}

/// Shared trace ordering, shape, mass and raw-score graph admission.
fn admitted_vocabulary_scores(
    trace: &VocabularyActionTrace,
    generate_raw: &Tensor,
    copy_raw: Option<&Tensor>,
    target: u32,
) -> Result<(Tensor, u64)> {
    device_admit(generate_raw.device())?;
    if generate_raw.rank() != 1
        || generate_raw.dtype() != DType::F32
        || trace.policy != uor_r4_integer::geometric_vocabulary_actions::POLICY
    {
        return Err(invalid("Generate loss full score shape differs"));
    }
    let mut generate_indices = Vec::new();
    let mut copy_indices = Vec::new();
    let mut seen_copy = false;
    let mut summed_mass = 0u64;
    let mut target_action_mass = 0u64;
    for (offset, action) in trace.actions.iter().enumerate() {
        if action.action_offset != offset
            || action.score_q24 != action.raw_score_q24.clamp(-(8 << 24), 8 << 24)
            || action.weight_q31 == 0
        {
            return Err(invalid("Generate loss native action trace differs"));
        }
        summed_mass = summed_mass
            .checked_add(action.weight_q31)
            .ok_or_else(|| invalid("Generate loss action mass overflow"))?;
        if action.token_id == target {
            target_action_mass = target_action_mass
                .checked_add(action.weight_q31)
                .ok_or_else(|| invalid("Generate loss alias mass overflow"))?;
        }
        match action.action {
            VocabularyAction::Generate { token_id } => {
                if seen_copy
                    || token_id != action.token_id
                    || token_id as usize >= generate_raw.elem_count()
                    || generate_indices
                        .last()
                        .is_some_and(|previous| *previous >= token_id)
                {
                    return Err(invalid("Generate loss action order/identity differs"));
                }
                generate_indices.push(token_id);
            }
            VocabularyAction::Copy { source_offset } => {
                seen_copy = true;
                if source_offset != copy_indices.len() {
                    return Err(invalid("Generate loss Copy occurrence order differs"));
                }
                copy_indices.push(source_offset as u32);
            }
        }
    }
    if generate_indices.is_empty() || summed_mass != trace.summary.total_weight_q31 {
        return Err(invalid(
            "Generate loss empty admission or denominator differs",
        ));
    }
    let generate = generate_raw.index_select(
        &Tensor::from_vec(
            generate_indices.clone(),
            generate_indices.len(),
            generate_raw.device(),
        )?,
        0,
    )?;
    let combined = if copy_indices.is_empty() {
        if copy_raw.is_some_and(|c| c.elem_count() != 0) {
            return Err(invalid("Generate loss unexpected Copy scores"));
        }
        generate
    } else {
        let copy = copy_raw.ok_or_else(|| invalid("Generate loss missing Copy scores"))?;
        if copy.dims() != [copy_indices.len()]
            || copy.dtype() != DType::F32
            || !copy.device().same_device(generate_raw.device())
        {
            return Err(invalid("Generate loss Copy shape/device differs"));
        }
        Tensor::cat(&[&generate, copy], 0)?
    };
    let native_raw = Tensor::from_vec(
        trace
            .actions
            .iter()
            .map(|a| (a.raw_score_q24 as f64 / Q24) as f32)
            .collect::<Vec<_>>(),
        trace.actions.len(),
        generate_raw.device(),
    )?;
    // Bind this loss to the supplied native pool at the declared f32 boundary.
    // This scalar checks every raw action before the hard anchor could conceal
    // an unrelated trace. Exact integer provenance remains the caller's role.
    if (&combined - &native_raw)?
        .abs()?
        .max(0)?
        .to_scalar::<f32>()?
        != 0.
    {
        return Err(invalid(
            "Generate loss raw score graph differs from native pool",
        ));
    }
    Ok((combined, target_action_mass))
}

/// Offline score Jacobian only; both policies retain the same native clipped
/// forward and alias probability. This policy never changes serving.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VocabularyScoreAdjoint {
    #[default]
    Clipped,
    RawIdentity,
}

/// Flat action marginal with labels used only after a target-free native pool.
/// Native token mass is aggregated before the sole float probability boundary.
/// Copy scores may be frozen or carry separately implemented input credit.
/// No separate branch gate or independent Generate normalization is introduced.
pub fn vocabulary_marginal_loss(
    trace: &VocabularyActionTrace,
    generate_raw: &Tensor,
    copy_raw: Option<&Tensor>,
    target: u32,
) -> Result<Tensor> {
    vocabulary_marginal_loss_with_credit(
        trace,
        generate_raw,
        copy_raw,
        target,
        VocabularyScoreAdjoint::Clipped,
    )
}

/// Explicit opt-in backward comparison; no automatic training-policy adoption.
pub fn vocabulary_marginal_loss_with_credit(
    trace: &VocabularyActionTrace,
    generate_raw: &Tensor,
    copy_raw: Option<&Tensor>,
    target: u32,
    credit: VocabularyScoreAdjoint,
) -> Result<Tensor> {
    let (combined, target_action_mass) =
        admitted_vocabulary_scores(trace, generate_raw, copy_raw, target)?;
    let bounded = combined.clamp(-8f32, 8f32)?;
    // A single reduced status scalar is permitted and counted; never download
    // the live CUDA score vector or its adjoints for validation.
    if !combined.sqr()?.sum_all()?.to_scalar::<f32>()?.is_finite() {
        return Err(invalid("Generate marginal score status is nonfinite"));
    }
    let mass = trace
        .token_masses
        .iter()
        .find(|m| m.token_id == target)
        .ok_or_else(|| invalid("Generate target is not admitted by full legal pool"))?
        .weight_q31;
    let denominator = trace.summary.total_weight_q31;
    if mass == 0 || denominator == 0 || mass > denominator || mass != target_action_mass {
        return Err(invalid("Generate native target mass is invalid"));
    }
    let native_probability = (mass as f64 / denominator as f64) as f32;
    if !native_probability.is_finite() || native_probability <= 0. {
        return Err(invalid("Generate native probability cannot be represented"));
    }
    let hard_scores = Tensor::from_vec(
        trace
            .actions
            .iter()
            .map(|a| (a.score_q24 as f64 / Q24) as f32)
            .collect::<Vec<_>>(),
        trace.actions.len(),
        generate_raw.device(),
    )?;
    let credit_scores = match credit {
        VocabularyScoreAdjoint::Clipped => &bounded,
        VocabularyScoreAdjoint::RawIdentity => &combined,
    };
    let scores = (&hard_scores + (credit_scores - credit_scores.detach())?)?;
    let probability = candle_nn::ops::softmax(&scores, 0)?;
    let mask = Tensor::from_vec(
        trace
            .actions
            .iter()
            .map(|a| if a.token_id == target { 1f32 } else { 0f32 })
            .collect::<Vec<_>>(),
        trace.actions.len(),
        generate_raw.device(),
    )?;
    let soft_target = (probability * mask)?.sum_all()?;
    let anchored = (Tensor::new(native_probability, generate_raw.device())?
        + (&soft_target - soft_target.detach())?)?;
    Ok(anchored.log()?.neg()?)
}

/// Prospective offline diagnostic only: no automatic weight, optimizer update,
/// runtime gate or serving effect.
pub const PRECLIP_ENTRY_MARGIN_SCOPE: &str =
    "authentic-native-raw-Q24-at-f32-boundary;Generate-only-target;highest-wrong-Copy-first-occurrence-tie;zero-margin-unclipped-score-adjoint;offline-diagnostic/1";

pub struct PreclipEntryMarginDiagnostic {
    pub loss: Tensor,
    pub wrong_copy_source_offset: usize,
    pub wrong_copy_action_offset: usize,
    pub target_generate_action_offset: usize,
    pub raw_gap_q24: i64,
    pub wrong_copy_above_clip: bool,
    pub target_generate_outside_clip: bool,
    pub score_scope: &'static str,
}

/// Diagnostic on an already completed target-free forward. Reject Copy-covered
/// labels; selection uses the authentic integer trace, never a CUDA download.
/// Equal wrong Copy scores retain the first source occurrence. The trace does
/// not encode prefix length: the caller must establish empty-prefix entry scope.
/// At zero gap Candle's ReLU uses unit derivative (a valid hinge subgradient).
pub fn preclip_entry_margin_diagnostic(
    trace: &VocabularyActionTrace,
    generate_raw: &Tensor,
    copy_raw: Option<&Tensor>,
    target: u32,
) -> Result<PreclipEntryMarginDiagnostic> {
    let (combined, _) = admitted_vocabulary_scores(trace, generate_raw, copy_raw, target)?;
    if !combined.sqr()?.sum_all()?.to_scalar::<f32>()?.is_finite() {
        return Err(invalid("entry margin score status is nonfinite"));
    }
    let mut target_offset = None;
    let mut wrong_copy = None;
    for action in &trace.actions {
        match action.action {
            VocabularyAction::Generate { token_id } if token_id == target => {
                if target_offset.replace(action.action_offset).is_some() {
                    return Err(invalid("entry margin multiple Generate targets"));
                }
            }
            VocabularyAction::Copy { source_offset } => {
                if action.token_id == target {
                    return Err(invalid("entry margin target is Copy-covered"));
                }
                if wrong_copy.is_none_or(|(_, _, best)| action.raw_score_q24 > best) {
                    wrong_copy = Some((source_offset, action.action_offset, action.raw_score_q24));
                }
            }
            _ => {}
        }
    }
    let target_offset =
        target_offset.ok_or_else(|| invalid("entry margin Generate target absent"))?;
    let (source_offset, copy_offset, copy_score) =
        wrong_copy.ok_or_else(|| invalid("entry margin requires a wrong Copy occurrence"))?;
    let target_score = trace.actions[target_offset].raw_score_q24;
    let gap = copy_score
        .checked_sub(target_score)
        .ok_or_else(|| invalid("entry margin raw gap overflow"))?;
    let copy = combined.narrow(0, copy_offset, 1)?.sum_all()?;
    let generate = combined.narrow(0, target_offset, 1)?.sum_all()?;
    Ok(PreclipEntryMarginDiagnostic {
        loss: (&copy - &generate)?.relu()?,
        wrong_copy_source_offset: source_offset,
        wrong_copy_action_offset: copy_offset,
        target_generate_action_offset: target_offset,
        raw_gap_q24: gap,
        wrong_copy_above_clip: copy_score > (8 << 24),
        target_generate_outside_clip: !(-(8 << 24)..=(8 << 24)).contains(&target_score),
        score_scope: PRECLIP_ENTRY_MARGIN_SCOPE,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::geometric_no_read::CANONICAL_BASIS_Q25;
    const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"Ġ":6,"Ġa":7},"merges":["Ġ a"]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
    fn binding() -> Result<SourceActionBinding> {
        SourceActionBinding::new(TOK.as_bytes()).map_err(|e| invalid(e.to_string()))
    }
    fn code(x: u8) -> Result<H4Code> {
        H4Code::try_from(x).map_err(|e| invalid(e.to_string()))
    }
    fn grad(g: &candle_core::backprop::GradStore, t: &Tensor) -> Result<Vec<f32>> {
        Ok(g.get(t)
            .ok_or_else(|| invalid("Generate test missing gradient"))?
            .flatten_all()?
            .to_vec1::<f32>()?)
    }
    fn margin_fixture(copy_tokens: &[u32], copy_scores: &[i64]) -> Result<VocabularyActionTrace> {
        use uor_r4_integer::geometric_vocabulary_actions::NativeVocabularyActions;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut pool =
            NativeVocabularyActions::new(binding()?, &exp).map_err(|e| invalid(e.to_string()))?;
        pool.reduce_trace(&vec![0; binding()?.vocab_size()], copy_tokens, copy_scores)
            .map_err(|e| invalid(e.to_string()))
    }

    #[test]
    fn raw_alias_credit_unblocks_saturation_without_changing_forward() -> Result<()> {
        let trace = margin_fixture(&[5], &[9 << 24])?;
        let generate = Var::zeros(binding()?.vocab_size(), DType::F32, &Device::Cpu)?;
        let copy = Var::from_vec(vec![9f32], 1, &Device::Cpu)?;
        let old =
            vocabulary_marginal_loss(&trace, generate.as_tensor(), Some(copy.as_tensor()), 4)?;
        let raw = vocabulary_marginal_loss_with_credit(
            &trace,
            generate.as_tensor(),
            Some(copy.as_tensor()),
            4,
            VocabularyScoreAdjoint::RawIdentity,
        )?;
        assert_eq!(old.to_scalar::<f32>()?, raw.to_scalar::<f32>()?);
        assert_eq!(grad(&old.backward()?, copy.as_tensor())?, vec![0.]);
        assert!(grad(&raw.backward()?, copy.as_tensor())?[0] > 0.);
        Ok(())
    }

    #[test]
    fn raw_alias_credit_matches_unsaturated_and_conserves_target_aliases() -> Result<()> {
        let trace = margin_fixture(&[4, 4, 5], &[0, 0, 0])?;
        let generate = Var::zeros(binding()?.vocab_size(), DType::F32, &Device::Cpu)?;
        let copy = Var::zeros(3, DType::F32, &Device::Cpu)?;
        let old =
            vocabulary_marginal_loss(&trace, generate.as_tensor(), Some(copy.as_tensor()), 4)?;
        let raw = vocabulary_marginal_loss_with_credit(
            &trace,
            generate.as_tensor(),
            Some(copy.as_tensor()),
            4,
            VocabularyScoreAdjoint::RawIdentity,
        )?;
        assert_eq!(old.to_scalar::<f32>()?, raw.to_scalar::<f32>()?);
        let old_g = old.backward()?;
        let raw_g = raw.backward()?;
        let dg = grad(&raw_g, generate.as_tensor())?;
        let dc = grad(&raw_g, copy.as_tensor())?;
        assert_eq!(dg, grad(&old_g, generate.as_tensor())?);
        assert_eq!(dc, grad(&old_g, copy.as_tensor())?);
        assert!((dg[4] - dc[0]).abs() < 1e-6);
        assert!((dc[0] - dc[1]).abs() < 1e-6);
        assert!(dc[0] < 0. && dc[2] > 0.);
        assert!((dg.iter().sum::<f32>() + dc.iter().sum::<f32>()).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn preclip_entry_margin_restores_saturated_score_credit_without_forward_mutation() -> Result<()>
    {
        let trace = margin_fixture(&[5, 5, 7], &[9 << 24, 9 << 24, -(9 << 24)])?;
        let before = serde_json::to_vec(&trace).map_err(|e| invalid(e.to_string()))?;
        let generate = Var::zeros(binding()?.vocab_size(), DType::F32, &Device::Cpu)?;
        let copy = Var::from_vec(vec![9f32, 9., -9.], 3, &Device::Cpu)?;
        let raw_before = copy.to_vec1::<f32>()?;
        let old =
            vocabulary_marginal_loss(&trace, generate.as_tensor(), Some(copy.as_tensor()), 4)?;
        assert_eq!(grad(&old.backward()?, copy.as_tensor())?, vec![0., 0., 0.]);
        let diagnostic = preclip_entry_margin_diagnostic(
            &trace,
            generate.as_tensor(),
            Some(copy.as_tensor()),
            4,
        )?;
        assert_eq!(diagnostic.wrong_copy_source_offset, 0);
        assert_eq!(diagnostic.raw_gap_q24, 9 << 24);
        assert!(diagnostic.wrong_copy_above_clip);
        assert_eq!(diagnostic.loss.to_scalar::<f32>()?, 9.);
        let gradient = diagnostic.loss.backward()?;
        assert_eq!(grad(&gradient, copy.as_tensor())?, vec![1., 0., 0.]);
        let dg = grad(&gradient, generate.as_tensor())?;
        assert_eq!(dg[4], -1.);
        assert!(dg.iter().enumerate().all(|(i, x)| i == 4 || *x == 0.));
        assert_eq!(copy.to_vec1::<f32>()?, raw_before);
        assert_eq!(
            generate.to_vec1::<f32>()?,
            vec![0.; binding()?.vocab_size()]
        );
        assert_eq!(
            serde_json::to_vec(&trace).map_err(|e| invalid(e.to_string()))?,
            before
        );
        Ok(())
    }

    #[test]
    fn preclip_entry_margin_rejects_nonmatching_or_copy_covered_pools() -> Result<()> {
        let trace = margin_fixture(&[5], &[9 << 24])?;
        let generate = Tensor::zeros(binding()?.vocab_size(), DType::F32, &Device::Cpu)?;
        let copy = Tensor::from_vec(vec![9f32], 1, &Device::Cpu)?;
        assert!(preclip_entry_margin_diagnostic(&trace, &generate, Some(&copy), 5).is_err());
        assert!(preclip_entry_margin_diagnostic(&trace, &generate, Some(&copy), 99).is_err());
        let wrong = Tensor::from_vec(vec![8f32], 1, &Device::Cpu)?;
        assert!(preclip_entry_margin_diagnostic(&trace, &generate, Some(&wrong), 4).is_err());
        let nan = Tensor::from_vec(vec![f32::NAN], 1, &Device::Cpu)?;
        assert!(preclip_entry_margin_diagnostic(&trace, &generate, Some(&nan), 4).is_err());
        let empty = margin_fixture(&[], &[])?;
        assert!(preclip_entry_margin_diagnostic(&empty, &generate, None, 4).is_err());
        let mut corrupted = trace.clone();
        corrupted.actions[0].action_offset += 1;
        assert!(preclip_entry_margin_diagnostic(&corrupted, &generate, Some(&copy), 4).is_err());
        Ok(())
    }

    #[test]
    fn preclip_entry_margin_gap_boundary_follows_relu_credit() -> Result<()> {
        for (score, expected_credit) in [(-1f32, 0f32), (0., 1.)] {
            let trace = margin_fixture(&[5], &[(score as i64) << 24])?;
            let generate = Var::zeros(binding()?.vocab_size(), DType::F32, &Device::Cpu)?;
            let copy = Var::from_vec(vec![score], 1, &Device::Cpu)?;
            let diagnostic = preclip_entry_margin_diagnostic(
                &trace,
                generate.as_tensor(),
                Some(copy.as_tensor()),
                4,
            )?;
            assert_eq!(diagnostic.loss.to_scalar::<f32>()?, 0.);
            let gradient = diagnostic.loss.backward()?;
            assert_eq!(grad(&gradient, copy.as_tensor())?, vec![expected_credit]);
            let dg = grad(&gradient, generate.as_tensor())?;
            assert_eq!(dg[4], -expected_credit);
            assert!(dg.iter().enumerate().all(|(i, x)| i == 4 || *x == 0.));
        }
        Ok(())
    }

    #[test]
    fn legacy_seed_route_and_balanced_energy_gap_remain_identical() -> Result<()> {
        for lanes in [1, 3, 8] {
            for seed in [0, 19, u64::MAX] {
                let binding = binding()?;
                let old =
                    GenerateLearningWeights::seeded(binding.clone(), lanes, seed, &Device::Cpu)?;
                let balanced = GenerateLearningWeights::seeded_balanced_token_geometry(
                    binding.clone(),
                    lanes,
                    seed,
                    &Device::Cpu,
                )?;
                let native = old.export_native()?;
                let new = balanced.export_native()?;
                // Independent copy of the original seeded digit assignment.
                let mut original = Vec::new();
                for token in 0..binding.vocab_size() {
                    let mut digits = token;
                    for lane in 0..lanes {
                        let offset = ((seed as usize % ROOT_COUNT) + lane * 17) % ROOT_COUNT;
                        original.push(((digits % ROOT_COUNT + offset) % ROOT_COUNT) as u8);
                        digits /= ROOT_COUNT;
                    }
                }
                let reference = NativeGeometricGenerate::compile(
                    &binding,
                    lanes,
                    &original,
                    native.packed_biases(),
                    native.energy().clone(),
                )
                .map_err(|e| invalid(e.to_string()))?;
                assert_eq!(
                    native.to_bytes().map_err(|e| invalid(e.to_string()))?,
                    reference.to_bytes().map_err(|e| invalid(e.to_string()))?
                );
                assert_eq!(
                    serde_json::to_vec(native.energy())?,
                    serde_json::to_vec(new.energy())?
                );
                assert_eq!(native.packed_biases(), new.packed_biases());
                assert_eq!(new.energy().edges().len(), lanes / 2);
                for logits in [
                    old.prototype_choices.flatten_all()?.to_vec1::<f32>()?,
                    balanced.prototype_choices.flatten_all()?.to_vec1::<f32>()?,
                ] {
                    for row in logits.chunks_exact(ROOT_COUNT) {
                        assert_eq!(row.iter().filter(|&&v| v == 2.).count(), 1);
                        assert_eq!(row.iter().filter(|&&v| v == 0.).count(), ROOT_COUNT - 1);
                    }
                }
                assert_eq!(
                    new.prototypes(),
                    initial_token_geometry(binding.vocab_size(), lanes, seed, true)
                );
            }
        }
        Ok(())
    }

    #[test]
    fn balanced_all_eight_lanes_full4096_pair_identity_and_marginals() -> Result<()> {
        use std::collections::BTreeSet;
        let mut tokenizer: serde_json::Value = serde_json::from_str(TOK)?;
        let vocab = tokenizer["model"]["vocab"]
            .as_object_mut()
            .ok_or_else(|| invalid("test tokenizer vocab absent"))?;
        for token in 8..4096 {
            vocab.insert(format!("token{token:04}"), serde_json::json!(token));
        }
        let binding = SourceActionBinding::new(&serde_json::to_vec(&tokenizer)?)
            .map_err(|e| invalid(e.to_string()))?;
        assert_eq!(binding.vocab_size(), 4096);
        let weights =
            GenerateLearningWeights::seeded_balanced_token_geometry(binding, 8, 71, &Device::Cpu)?;
        let native = weights.export_native()?;
        let rows = native.prototypes().chunks_exact(8).collect::<Vec<_>>();
        assert_eq!(
            rows.iter()
                .map(|r| r.to_vec())
                .collect::<BTreeSet<_>>()
                .len(),
            4096
        );
        for pair in native.energy().edges() {
            let distinct = rows
                .iter()
                .map(|r| (r[usize::from(pair.left)], r[usize::from(pair.right)]))
                .collect::<BTreeSet<_>>();
            assert_eq!(distinct.len(), 4096);
            assert_eq!(pair.right - pair.left, 1);
        }
        for lane in 0..8 {
            let mut count = [0usize; ROOT_COUNT];
            for row in &rows {
                count[usize::from(row[lane])] += 1;
            }
            assert!(count.iter().all(|&n| n == 34 || n == 35));
            assert_eq!(count.iter().filter(|&&n| n == 35).count(), 16);
            assert_eq!(count.iter().sum::<usize>(), 4096);
        }
        // The same assignment admits smaller/odd lane counts; uniqueness is
        // capacity-qualified (one lane has only120 codes).
        for lanes in [1, 2, 3, 5, 7] {
            let codes = initial_token_geometry(4096, lanes, 71, true);
            assert_eq!(codes.len(), 4096 * lanes);
            assert!(codes.iter().all(|&r| usize::from(r) < ROOT_COUNT));
            if lanes >= 2 {
                assert_eq!(
                    codes
                        .chunks_exact(lanes)
                        .map(|r| (r[0], r[1]))
                        .collect::<BTreeSet<_>>()
                        .len(),
                    4096
                );
            }
        }
        Ok(())
    }

    #[test]
    fn generate_explicit_state_utility_preserves_native_and_separate_parameter_credit() -> Result<()>
    {
        let w = GenerateLearningWeights::seeded(binding()?, 2, 1, &Device::Cpu)?;
        let snapshot = w.prepare_native()?;
        let states = [code(4)?, code(5)?];
        let mut onehot = vec![0f32; 2 * ROOT_COUNT];
        for (lane, state) in states.iter().enumerate() {
            onehot[lane * ROOT_COUNT + usize::from(state.index())] = 1.;
        }
        let choices = Var::from_tensor(&Tensor::from_vec(onehot, (2, ROOT_COUNT), &Device::Cpu)?)?;
        let logits = Var::zeros((2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let old = w.forward_prepared(&snapshot, &states, logits.as_tensor())?;
        let new = w.forward_prepared_state_choices(&snapshot, &states, choices.as_tensor())?;
        assert_eq!(old.scores_q24, new.scores_q24);
        assert_eq!(
            old.raw_scores.to_vec1::<f32>()?,
            new.raw_scores.to_vec1::<f32>()?
        );
        let old_grad = old.raw_scores.sum_all()?.backward()?;
        let new_grad = new.raw_scores.sum_all()?.backward()?;
        assert!(new_grad.get(logits.as_tensor()).is_none());
        for (_, var) in w.parameters() {
            let a = grad(&old_grad, var.as_tensor())?;
            let b = grad(&new_grad, var.as_tensor())?;
            assert!(a.iter().zip(b).all(|(&a, b)| (a - b).abs() < 1e-6));
        }
        let utility = grad(&new_grad, choices.as_tensor())?;
        for lane in 0..2 {
            let mut expected = [0f64; ROOT_COUNT];
            for token in 0..w.vocab_size() {
                let mut counter = [0i64; ROOT_COUNT];
                snapshot
                    .native
                    .state_conditional_scores_into(
                        &states,
                        lane,
                        token,
                        &mut counter,
                        &mut GenerateReadCounts::default(),
                    )
                    .map_err(|e| invalid(e.to_string()))?;
                for (u, score) in expected.iter_mut().zip(counter) {
                    *u += score as f64 / Q24;
                }
            }
            let e_mean = expected.iter().sum::<f64>() / ROOT_COUNT as f64;
            let actual = &utility[lane * ROOT_COUNT..(lane + 1) * ROOT_COUNT];
            let a_mean = actual.iter().map(|&x| f64::from(x)).sum::<f64>() / ROOT_COUNT as f64;
            assert!(actual
                .iter()
                .zip(expected)
                .all(|(&a, e)| ((f64::from(a) - a_mean) - (e - e_mean)).abs() < 1e-6));
        }
        assert!(w
            .forward_prepared_state_choices(&snapshot, &states, logits.as_tensor())
            .is_err());
        Ok(())
    }
    #[test]
    fn generate_learning_hard_forward_export_and_selected_coefficient_credit() -> Result<()> {
        for seed in [1, 2, 3] {
            let w = GenerateLearningWeights::seeded(binding()?, 2, seed, &Device::Cpu)?;
            let snapshot = w.prepare_native()?;
            let states = [code(4)?, code(5)?];
            let z = Var::zeros((2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
            let out = w.forward_prepared(&snapshot, &states, z.as_tensor())?;
            assert_eq!(
                out.raw_scores.to_vec1::<f32>()?,
                out.scores_q24
                    .iter()
                    .map(|&s| (s as f64 / Q24) as f32)
                    .collect::<Vec<_>>()
            );
            let g = out.raw_scores.sum_all()?.backward()?;
            // Counterfactual paths detach factors: bias receives exactly one
            // selected coefficient derivative per vocabulary row, not twice.
            assert!(grad(&g, w.bias.as_tensor())?.iter().all(|&x| x == 0.25));
            assert!(grad(&g, z.as_tensor())?.iter().any(|x| x.abs() > 1e-6));
            assert!(grad(&g, w.prototype_choices.as_tensor())?
                .iter()
                .any(|x| x.abs() > 1e-6));
            let reload = NativeGeometricGenerate::from_bytes(
                &snapshot
                    .native
                    .to_bytes()
                    .map_err(|e| invalid(e.to_string()))?,
                w.binding(),
            )
            .map_err(|e| invalid(e.to_string()))?;
            let restored = GenerateLearningWeights::from_native(binding()?, &reload, &Device::Cpu)?;
            assert_eq!(
                restored
                    .export_native()?
                    .to_bytes()
                    .map_err(|e| invalid(e.to_string()))?,
                snapshot
                    .native
                    .to_bytes()
                    .map_err(|e| invalid(e.to_string()))?
            );
        }
        Ok(())
    }
    fn coefficients_only_loss_parity(device: &Device) -> Result<()> {
        use uor_r4_integer::geometric_vocabulary_actions::NativeVocabularyActions;
        let weights =
            GenerateLearningWeights::seeded_balanced_token_geometry(binding()?, 4, 19, device)?;
        let snapshot = weights.prepare_native()?;
        let states = [code(4)?, code(13)?, code(29)?, code(47)?];
        let mut values = vec![0f32; 4 * ROOT_COUNT];
        for (lane, state) in states.iter().enumerate() {
            values[lane * ROOT_COUNT + usize::from(state.index())] = 1.;
        }
        let choices = Var::from_tensor(&Tensor::from_vec(values, (4, ROOT_COUNT), device)?)?;
        let full =
            weights.forward_prepared_state_choices(&snapshot, &states, choices.as_tensor())?;
        let field = weights.forward_prepared_coefficients_only(&snapshot, &states)?;
        assert_eq!(full.scores_q24, field.scores_q24);
        assert_eq!(
            full.raw_scores.to_vec1::<f32>()?,
            field.raw_scores.to_vec1::<f32>()?
        );
        assert_eq!(field.costs.conditional_choice_rows, 0);
        assert_eq!(field.costs.max_conditional_utility_elements, 0);
        // Same offline canonical exp formula used by the native consumer;
        // public pool constructor authenticates bytes before any loss.
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut pool =
            NativeVocabularyActions::new(binding()?, &exp).map_err(|e| invalid(e.to_string()))?;
        let trace = pool
            .reduce_trace(&field.scores_q24, &[4, 4, 5], &[0, -(1 << 24), 0])
            .map_err(|e| invalid(e.to_string()))?;
        let copy = Tensor::from_vec(vec![0f32, -1., 0.], 3, device)?;
        let lf = vocabulary_marginal_loss(&trace, &full.raw_scores, Some(&copy), 4)?;
        let lc = vocabulary_marginal_loss(&trace, &field.raw_scores, Some(&copy), 4)?;
        assert_eq!(lf.to_scalar::<f32>()?, lc.to_scalar::<f32>()?);
        let gf = lf.backward()?;
        let gc = lc.backward()?;
        for (name, var) in weights.parameters() {
            if name == "generate.prototype_choices" {
                assert!(
                    gc.get(var.as_tensor()).is_none(),
                    "coefficient-only prototype credit"
                );
            } else {
                let a = grad(&gf, var.as_tensor())?;
                let b = grad(&gc, var.as_tensor())?;
                assert!(a.iter().any(|x| x.abs() > 1e-7), "{name} disconnected");
                for (x, y) in a.iter().zip(b) {
                    assert!((*x - y).abs() <= 2e-6 + 2e-5 * x.abs(), "{name}: {x} {y}");
                }
            }
        }
        assert!(
            gc.get(choices.as_tensor()).is_none(),
            "coefficient-only state credit"
        );
        Ok(())
    }
    #[test]
    fn coefficients_only_native_pool_loss_matches_full_selected_fields() -> Result<()> {
        coefficients_only_loss_parity(&Device::Cpu)
    }
    #[cfg(feature = "cuda")]
    #[test]
    fn coefficients_only_cuda_native_pool_loss_matches_full_selected_fields() -> Result<()> {
        coefficients_only_loss_parity(&Device::new_cuda(0)?)
    }
    #[test]
    fn generate_learning_cached_u32_indices_match_uncached_scores_and_all_gradients() -> Result<()>
    {
        let w = GenerateLearningWeights::seeded(binding()?, 4, 3, &Device::Cpu)?;
        let prepared = w.prepare_native()?;
        let z = Var::zeros((4, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let states = [code(4)?, code(5)?, code(8)?, code(13)?];
        let cached = w.forward_prepared(&prepared, &states, z.as_tensor())?;
        let reference = w.forward_prepared_uncached_reference(&prepared, &states, z.as_tensor())?;
        assert_eq!(
            cached.raw_scores.to_vec1::<f32>()?,
            reference.raw_scores.to_vec1::<f32>()?
        );
        assert_eq!(cached.costs.staged_index_bytes, 0);
        assert!(reference.costs.staged_index_bytes > 0);
        let a = cached.raw_scores.sqr()?.sum_all()?.backward()?;
        let b = reference.raw_scores.sqr()?.sum_all()?.backward()?;
        let mut parameters = w.parameters();
        parameters.insert("state".into(), z);
        for (name, var) in parameters {
            for (x, y) in grad(&a, var.as_tensor())?
                .into_iter()
                .zip(grad(&b, var.as_tensor())?)
            {
                assert!((x - y).abs() < 1e-6, "{name}: {x} {y}");
            }
        }
        // Repeated positions reuse the same snapshot and choice graph; code
        // probabilities must not be carried beyond an optimizer update.
        let changed = [code(13)?, code(8)?, code(5)?, code(4)?];
        let next = w.forward_prepared(
            &prepared,
            &changed,
            &Tensor::zeros((4, ROOT_COUNT), DType::F32, &Device::Cpu)?,
        )?;
        assert_eq!(next.costs.staged_index_bytes, 0);
        assert_ne!(cached.scores_q24, next.scores_q24);
        // Logits may change without moving native argmax, still invalidating
        // the cached normalized choice differential.
        let before = w
            .export_native()?
            .to_bytes()
            .map_err(|e| invalid(e.to_string()))?;
        w.prototype_choices
            .set(&(w.prototype_choices.as_tensor() * 1.5)?)?;
        assert_eq!(
            before,
            w.export_native()?
                .to_bytes()
                .map_err(|e| invalid(e.to_string()))?
        );
        assert!(w
            .forward_prepared(
                &prepared,
                &states,
                &Tensor::zeros((4, ROOT_COUNT), DType::F32, &Device::Cpu)?
            )
            .is_err());
        Ok(())
    }
    #[test]
    fn generate_learning_full120_preserves_even_harmonic_state_credit() -> Result<()> {
        let b = binding()?;
        let mut energy = EnergyTables::zeroed(1, vec![]).map_err(|e| invalid(e.to_string()))?;
        for (r, axes) in CANONICAL_BASIS_Q25.iter().enumerate() {
            let value = if axes[0].unsigned_abs() > 16_777_216 {
                1
            } else {
                -1
            };
            energy
                .set_unary(0, r as u8, value)
                .map_err(|e| invalid(e.to_string()))?;
        }
        let n = NativeGeometricGenerate::compile(
            &b,
            1,
            &vec![1; b.vocab_size()],
            &vec![0; b.vocab_size().div_ceil(2)],
            energy,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let w = GenerateLearningWeights::from_native(b, &n, &Device::Cpu)?;
        let prepared = w.prepare_native()?;
        let z = Var::zeros((1, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let out = w.forward_prepared(&prepared, &[code(1)?], z.as_tensor())?;
        let g = out.raw_scores.sum_all()?.backward()?;
        let dz = grad(&g, z.as_tensor())?;
        assert!(dz.iter().any(|x| x.abs() > 1e-4));
        // Even antipodal utility has zero four-coordinate projection while
        // retaining a nonzero full120 differential.
        for axis in 0..4 {
            let moment = dz
                .iter()
                .zip(CANONICAL_BASIS_Q25.iter())
                .map(|(&x, b)| f64::from(x) * f64::from(b[axis]) / 33_554_432.)
                .sum::<f64>();
            assert!(moment.abs() < 1e-6, "axis {axis}: {moment}");
        }
        let mut counter = [0i64; ROOT_COUNT];
        n.state_conditional_scores_into(
            &[code(1)?],
            0,
            0,
            &mut counter,
            &mut GenerateReadCounts::default(),
        )
        .map_err(|e| invalid(e.to_string()))?;
        let utilities = counter.map(|x| x as f64 / Q24);
        let mean = utilities.iter().sum::<f64>() / ROOT_COUNT as f64;
        for (actual, u) in dz.iter().zip(utilities) {
            let expected = (u - mean) * w.vocab_size() as f64 / ROOT_COUNT as f64;
            assert!((f64::from(*actual) - expected).abs() < 1e-6);
        }
        Ok(())
    }
    #[test]
    fn generate_learning_zero_tables_disconnect_choices_and_stale_snapshot_rejects() -> Result<()> {
        let b = binding()?;
        let n = NativeGeometricGenerate::compile(
            &b,
            2,
            &vec![1; b.vocab_size() * 2],
            &vec![0; b.vocab_size().div_ceil(2)],
            EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 1 }])
                .map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let w = GenerateLearningWeights::from_native(b, &n, &Device::Cpu)?;
        let p = w.prepare_native()?;
        let z = Var::zeros((2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let out = w.forward_prepared(&p, &[code(1)?, code(1)?], z.as_tensor())?;
        let g = out.raw_scores.sum_all()?.backward()?;
        assert!(grad(&g, z.as_tensor())?.iter().all(|x| *x == 0.));
        assert!(grad(&g, w.prototype_choices.as_tensor())?
            .iter()
            .all(|x| *x == 0.));
        w.bias.set(&Tensor::from_vec(
            vec![0.25f32; w.vocab_size()],
            w.vocab_size(),
            &Device::Cpu,
        )?)?;
        assert!(w
            .forward_prepared(&p, &[code(1)?, code(1)?], z.as_tensor())
            .is_err());
        w.bias.set(&Tensor::from_vec(
            vec![f32::NAN; w.vocab_size()],
            w.vocab_size(),
            &Device::Cpu,
        )?)?;
        assert!(w.export_native().is_err());
        Ok(())
    }
    #[test]
    fn generate_learning_loss_binds_raw_scores_and_clip_has_zero_saturated_credit() -> Result<()> {
        use uor_r4_integer::geometric_vocabulary_actions::{
            VocabularyActionMass, VocabularyReduction, VocabularyTokenMass,
        };
        let b = binding()?;
        let v = b.vocab_size();
        let one = 1u64 << 31;
        // Exact canonical-exp equal-zero-score case: every unnormalized weight
        // is 2^31, so this fixture needs no authored probability approximation.
        let actions = (0..v)
            .map(|i| VocabularyActionMass {
                action: VocabularyAction::Generate { token_id: i as u32 },
                action_offset: i,
                token_id: i as u32,
                raw_score_q24: 0,
                score_q24: 0,
                weight_q31: one,
            })
            .collect();
        let masses = (0..v)
            .map(|i| VocabularyTokenMass {
                token_id: i as u32,
                weight_q31: one,
                generate_weight_q31: one,
                copy_weight_q31: 0,
            })
            .collect();
        let trace = VocabularyActionTrace {
            policy: uor_r4_integer::geometric_vocabulary_actions::POLICY,
            tokenizer_sha256: b.tokenizer_sha256().into(),
            period_token_id: b.period_token_id(),
            eos_token_id: b.eos_token_id(),
            actions,
            token_masses: masses,
            summary: VocabularyReduction {
                legal_generate_actions: v,
                copy_actions: 0,
                max_score_q24: 0,
                total_weight_q31: one * v as u64,
                chosen_token_id: 0,
                chosen_weight_q31: one,
                generate_weight_q31: one * v as u64,
                copy_weight_q31: 0,
                chosen_generate_weight_q31: one,
                chosen_copy_weight_q31: 0,
                clipped_low_actions: 0,
                clipped_high_actions: 0,
                raw_max_score_q24: 0,
                raw_total_weight_q31: one * v as u64,
                raw_chosen_token_id: 0,
                raw_chosen_weight_q31: one,
                chosen_raw_mass_rank: 1,
                raw_chosen_clipped_mass_rank: 1,
                token_winner_changed_by_clip: false,
            },
        };
        let scores = Var::zeros(v, DType::F32, &Device::Cpu)?;
        let loss = vocabulary_marginal_loss(&trace, scores.as_tensor(), None, 4)?;
        assert!((loss.to_scalar::<f32>()? - (v as f32).ln()).abs() < 1e-6);
        assert!(grad(&loss.backward()?, scores.as_tensor())?
            .iter()
            .any(|x| x.abs() > 1e-6));
        let mut wrong = vec![0f32; v];
        wrong[4] = 0.0625;
        assert!(vocabulary_marginal_loss(
            &trace,
            &Tensor::from_vec(wrong, v, &Device::Cpu)?,
            None,
            4
        )
        .is_err());
        let clip = Var::from_vec(vec![-9f32, 0., 9.], 3, &Device::Cpu)?;
        let g = clip.clamp(-8f32, 8f32)?.sum_all()?.backward()?;
        assert_eq!(grad(&g, clip.as_tensor())?, vec![0., 1., 0.]);
        Ok(())
    }
    #[test]
    fn generate_learning_asymmetric_pair_credit_matches_core_conditional_oracle() -> Result<()> {
        let b = binding()?;
        let v = b.vocab_size();
        let mut energy = EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 1 }])
            .map_err(|e| invalid(e.to_string()))?;
        for left in 0..ROOT_COUNT {
            for right in 0..ROOT_COUNT {
                energy
                    .set_pair(
                        0,
                        left as u8,
                        right as u8,
                        ((3 * left + 7 * right) % 15) as i8 - 7,
                    )
                    .map_err(|e| invalid(e.to_string()))?;
            }
        }
        assert_ne!(
            energy
                .get_pair(0, 2, 7)
                .map_err(|e| invalid(e.to_string()))?,
            energy
                .get_pair(0, 7, 2)
                .map_err(|e| invalid(e.to_string()))?
        );
        let prototypes = (0..v)
            .flat_map(|t| [(t + 1) as u8, ((3 * t + 5) % ROOT_COUNT) as u8])
            .collect::<Vec<_>>();
        let native =
            NativeGeometricGenerate::compile(&b, 2, &prototypes, &vec![0; v.div_ceil(2)], energy)
                .map_err(|e| invalid(e.to_string()))?;
        let w = GenerateLearningWeights::from_native(b, &native, &Device::Cpu)?;
        let prepared = w.prepare_native()?;
        let state = [code(4)?, code(13)?];
        let token = 4usize;
        let state_values = (0..2 * ROOT_COUNT)
            .map(|i| ((i % 11) as f32 - 5.) * 0.125)
            .collect::<Vec<_>>();
        let z = Var::from_vec(state_values.clone(), (2, ROOT_COUNT), &Device::Cpu)?;
        let output = w.forward_prepared(&prepared, &state, z.as_tensor())?;
        let g = output
            .raw_scores
            .narrow(0, token, 1)?
            .sum_all()?
            .backward()?;
        let actual_state = grad(&g, z.as_tensor())?;
        let actual_code = grad(&g, w.prototype_choices.as_tensor())?;
        let code_logits = w.prototype_choices.flatten_all()?.to_vec1::<f32>()?;
        let expected = |logits: &[f32], scores: &[i64; ROOT_COUNT]| {
            let maximum = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let unnormalized = logits
                .iter()
                .map(|x| (f64::from(*x) - f64::from(maximum)).exp())
                .collect::<Vec<_>>();
            let total = unnormalized.iter().sum::<f64>();
            let probabilities = unnormalized.iter().map(|p| p / total).collect::<Vec<_>>();
            let utilities = scores.map(|s| s as f64 / Q24);
            let mean = probabilities
                .iter()
                .zip(utilities)
                .map(|(p, u)| p * u)
                .sum::<f64>();
            probabilities
                .into_iter()
                .zip(utilities)
                .map(|(p, u)| p * (u - mean))
                .collect::<Vec<_>>()
        };
        for lane in 0..2 {
            let mut state_scores = [0i64; ROOT_COUNT];
            let mut code_scores = [0i64; ROOT_COUNT];
            native
                .state_conditional_scores_into(
                    &state,
                    lane,
                    token,
                    &mut state_scores,
                    &mut GenerateReadCounts::default(),
                )
                .map_err(|e| invalid(e.to_string()))?;
            native
                .code_conditional_scores_into(
                    &state,
                    lane,
                    token,
                    &mut code_scores,
                    &mut GenerateReadCounts::default(),
                )
                .map_err(|e| invalid(e.to_string()))?;
            let se = expected(
                &state_values[lane * ROOT_COUNT..(lane + 1) * ROOT_COUNT],
                &state_scores,
            );
            let offset = (token * 2 + lane) * ROOT_COUNT;
            let ce = expected(&code_logits[offset..offset + ROOT_COUNT], &code_scores);
            assert!(se.iter().any(|x| x.abs() > 1e-5));
            assert!(ce.iter().any(|x| x.abs() > 1e-5));
            for r in 0..ROOT_COUNT {
                assert!(
                    (f64::from(actual_state[lane * ROOT_COUNT + r]) - se[r]).abs() < 2e-6,
                    "state lane{lane} root{r}"
                );
                assert!(
                    (f64::from(actual_code[offset + r]) - ce[r]).abs() < 2e-6,
                    "code lane{lane} root{r}"
                );
            }
        }
        for t in 0..v {
            if t != token {
                assert!(actual_code[t * 2 * ROOT_COUNT..(t + 1) * 2 * ROOT_COUNT]
                    .iter()
                    .all(|x| *x == 0.));
            }
        }
        Ok(())
    }
    #[test]
    fn generate_learning_multiple_copy_aliases_share_one_marginal_gradient() -> Result<()> {
        use uor_r4_integer::geometric_vocabulary_actions::{
            VocabularyActionMass, VocabularyReduction, VocabularyTokenMass,
        };
        let b = binding()?;
        let v = b.vocab_size();
        let one = 1u64 << 31;
        let copy_ids = [4u32, 4, 5];
        let count = v + copy_ids.len();
        let denominator = one * count as u64;
        let mut actions = (0..v)
            .map(|i| VocabularyActionMass {
                action: VocabularyAction::Generate { token_id: i as u32 },
                action_offset: i,
                token_id: i as u32,
                raw_score_q24: 0,
                score_q24: 0,
                weight_q31: one,
            })
            .collect::<Vec<_>>();
        for (offset, &token_id) in copy_ids.iter().enumerate() {
            actions.push(VocabularyActionMass {
                action: VocabularyAction::Copy {
                    source_offset: offset,
                },
                action_offset: v + offset,
                token_id,
                raw_score_q24: 0,
                score_q24: 0,
                weight_q31: one,
            });
        }
        let masses = (0..v)
            .map(|t| {
                let copies = copy_ids.iter().filter(|id| **id == t as u32).count() as u64;
                VocabularyTokenMass {
                    token_id: t as u32,
                    weight_q31: one * (1 + copies),
                    generate_weight_q31: one,
                    copy_weight_q31: one * copies,
                }
            })
            .collect();
        // All raw scores zero is the exact canonical-table case, including the
        // distractor and both same-target occurrences; no approximate weights.
        let trace = VocabularyActionTrace {
            policy: uor_r4_integer::geometric_vocabulary_actions::POLICY,
            tokenizer_sha256: b.tokenizer_sha256().into(),
            period_token_id: b.period_token_id(),
            eos_token_id: b.eos_token_id(),
            actions,
            token_masses: masses,
            summary: VocabularyReduction {
                legal_generate_actions: v,
                copy_actions: 3,
                max_score_q24: 0,
                total_weight_q31: denominator,
                chosen_token_id: 4,
                chosen_weight_q31: 3 * one,
                generate_weight_q31: v as u64 * one,
                copy_weight_q31: 3 * one,
                chosen_generate_weight_q31: one,
                chosen_copy_weight_q31: 2 * one,
                clipped_low_actions: 0,
                clipped_high_actions: 0,
                raw_max_score_q24: 0,
                raw_total_weight_q31: denominator,
                raw_chosen_token_id: 4,
                raw_chosen_weight_q31: 3 * one,
                chosen_raw_mass_rank: 1,
                raw_chosen_clipped_mass_rank: 1,
                token_winner_changed_by_clip: false,
            },
        };
        let generate = Var::zeros(v, DType::F32, &Device::Cpu)?;
        let copy = Var::zeros(3, DType::F32, &Device::Cpu)?;
        let loss =
            vocabulary_marginal_loss(&trace, generate.as_tensor(), Some(copy.as_tensor()), 4)?;
        assert!((f64::from(loss.to_scalar::<f32>()?) + (3. / count as f64).ln()).abs() < 1e-6);
        let g = loss.backward()?;
        for (t, actual) in grad(&g, generate.as_tensor())?.into_iter().enumerate() {
            let expected = 1. / count as f64 - if t == 4 { 1. / 3. } else { 0. };
            assert!(
                (f64::from(actual) - expected).abs() < 1e-6,
                "Generate token{t}"
            );
        }
        for (i, actual) in grad(&g, copy.as_tensor())?.into_iter().enumerate() {
            let expected = 1. / count as f64 - if copy_ids[i] == 4 { 1. / 3. } else { 0. };
            assert!(
                (f64::from(actual) - expected).abs() < 1e-6,
                "Copy occurrence{i}"
            );
        }
        Ok(())
    }
    #[cfg(feature = "cuda")]
    #[test]
    fn balanced_eight_lane_cuda_hard_state_adjoint_parity() -> Result<()> {
        // The carried base-120 digit and every lane must participate in credit.
        // Missing CUDA is an error, never a skipped numerical comparison.
        let gpu = Device::new_cuda(0)?;
        let mut tokenizer: serde_json::Value = serde_json::from_str(TOK)?;
        let vocab = tokenizer["model"]["vocab"]
            .as_object_mut()
            .ok_or_else(|| invalid("test tokenizer vocab absent"))?;
        for token in 8..141 {
            vocab.insert(format!("token{token:04}"), serde_json::json!(token));
        }
        let binding = SourceActionBinding::new(&serde_json::to_vec(&tokenizer)?)
            .map_err(|e| invalid(e.to_string()))?;
        assert_eq!(binding.vocab_size(), 141);
        let cpu = GenerateLearningWeights::seeded_balanced_token_geometry(
            binding.clone(),
            8,
            71,
            &Device::Cpu,
        )?;
        let native = cpu.export_native()?;
        let cuda = GenerateLearningWeights::from_native(binding, &native, &gpu)?;
        let states = [
            code(4)?,
            code(13)?,
            code(29)?,
            code(47)?,
            code(61)?,
            code(79)?,
            code(97)?,
            code(113)?,
        ];
        let mut onehot = vec![0f32; 8 * ROOT_COUNT];
        for (lane, state) in states.iter().enumerate() {
            onehot[lane * ROOT_COUNT + usize::from(state.index())] = 1.;
        }
        let zc = Var::from_tensor(&Tensor::from_vec(
            onehot.clone(),
            (8, ROOT_COUNT),
            &Device::Cpu,
        )?)?;
        let zg = Var::from_tensor(&Tensor::from_vec(onehot, (8, ROOT_COUNT), &gpu)?)?;
        let c =
            cpu.forward_prepared_state_choices(&cpu.prepare_native()?, &states, zc.as_tensor())?;
        let d =
            cuda.forward_prepared_state_choices(&cuda.prepare_native()?, &states, zg.as_tensor())?;
        assert_eq!(c.scores_q24, d.scores_q24);
        assert_eq!(
            c.raw_scores.to_vec1::<f32>()?,
            d.raw_scores.to_vec1::<f32>()?
        );
        let adjoints = (0..141)
            .map(|token| ((token % 17) as f32 - 8.) / 16. + (token / 17) as f32 / 128.)
            .collect::<Vec<_>>();
        let cg = (&c.raw_scores * Tensor::from_vec(adjoints.clone(), 141, &Device::Cpu)?)?
            .sum_all()?
            .backward()?;
        let dg = (&d.raw_scores * Tensor::from_vec(adjoints, 141, &gpu)?)?
            .sum_all()?
            .backward()?;
        let mut cp = cpu.parameters();
        let mut dp = cuda.parameters();
        cp.insert("state".into(), zc);
        dp.insert("state".into(), zg);
        assert_eq!(cp.len(), dp.len());
        for (name, a) in cp {
            let b = dp
                .get(&name)
                .ok_or_else(|| invalid("CUDA test parameter name missing"))?;
            assert_eq!(a.dims(), b.dims(), "{name}");
            let x = grad(&cg, a.as_tensor())?;
            let y = grad(&dg, b.as_tensor())?;
            assert_eq!(x.len(), y.len(), "{name}");
            for (&x, &y) in x.iter().zip(&y) {
                assert!(x.is_finite() && y.is_finite());
                assert!((x - y).abs() <= 2e-4 + 2e-4 * x.abs(), "{name}: {x} {y}");
            }
            if name == "generate.prototype_choices" {
                assert_eq!(x.len(), 141 * 8 * ROOT_COUNT);
                for lane in 0..8 {
                    assert!(
                        x.chunks_exact(8 * ROOT_COUNT).any(|token| {
                            token[lane * ROOT_COUNT..(lane + 1) * ROOT_COUNT]
                                .iter()
                                .any(|v| v.abs() > 1e-6)
                        }),
                        "prototype lane {lane} has no credit"
                    );
                }
            } else if name == "state" || name == "generate.unary" {
                assert_eq!(x.len(), 8 * ROOT_COUNT);
                for (lane, row) in x.chunks_exact(ROOT_COUNT).enumerate() {
                    assert!(
                        row.iter().any(|v| v.abs() > 1e-6),
                        "{name} lane {lane} has no credit"
                    );
                }
            } else if name == "generate.pair" {
                assert_eq!(x.len(), 4 * ROOT_COUNT * ROOT_COUNT);
                for (edge, row) in x.chunks_exact(ROOT_COUNT * ROOT_COUNT).enumerate() {
                    assert!(
                        row.iter().any(|v| v.abs() > 1e-6),
                        "pair edge {edge} has no credit"
                    );
                }
            } else {
                assert_eq!(name, "generate.bias");
                assert!(x.iter().any(|v| v.abs() > 1e-6));
            }
        }
        Ok(())
    }
    #[cfg(feature = "cuda")]
    #[test]
    fn generate_learning_cuda_explicit_hard_and_full_choice_gradient_parity() -> Result<()> {
        // Explicit CUDA test: no missing-device skip can be mistaken for PASS.
        let gpu = Device::new_cuda(0)?;
        let cpu = GenerateLearningWeights::seeded(binding()?, 2, 7, &Device::Cpu)?;
        let native = cpu.export_native()?;
        let cuda = GenerateLearningWeights::from_native(binding()?, &native, &gpu)?;
        let states = [code(4)?, code(5)?];
        let zc = Var::zeros((2, ROOT_COUNT), DType::F32, &Device::Cpu)?;
        let zg = Var::zeros((2, ROOT_COUNT), DType::F32, &gpu)?;
        let c = cpu.forward_prepared(&cpu.prepare_native()?, &states, zc.as_tensor())?;
        let d = cuda.forward_prepared(&cuda.prepare_native()?, &states, zg.as_tensor())?;
        assert_eq!(
            c.raw_scores.to_vec1::<f32>()?,
            d.raw_scores.to_vec1::<f32>()?
        );
        let cg = c.raw_scores.sqr()?.sum_all()?.backward()?;
        let dg = d.raw_scores.sqr()?.sum_all()?.backward()?;
        let mut cp = cpu.parameters();
        let mut dp = cuda.parameters();
        cp.insert("state".into(), zc);
        dp.insert("state".into(), zg);
        for (name, a) in cp {
            let b = dp
                .get(&name)
                .ok_or_else(|| invalid("CUDA test parameter name missing"))?;
            for (x, y) in grad(&cg, a.as_tensor())?
                .into_iter()
                .zip(grad(&dg, b.as_tensor())?)
            {
                assert!((x - y).abs() <= 2e-4 + 2e-4 * x.abs(), "{name}: {x} {y}");
            }
        }
        Ok(())
    }
}
