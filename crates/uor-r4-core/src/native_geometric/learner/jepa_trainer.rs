//! JEPA Latent State Prediction + Next-Token Autoregressive Learner (Milestone 2).
//!
//! Dual Objective:
//!   L = L_CE + \lambda * d_FS(s_{t+1}, \hat{s}_{t+1}) + \lambda_{fiber} * d_{U(1)}(\psi_{t+1}, \hat{\psi}_{t+1})
//! - L_CE: Next-token cross-entropy driving natural language lexical generation.
//! - d_FS: Quantum Fubini-Study geodesic distance on the Bloch sphere S2.
//! - d_{U(1)}: Fiber phase holonomy alignment on the circle U(1) to prevent topological memory collapse.
//! - VSA context engine integration: 4096-bit hypervectors for joint multi-scale context representation.
//!
//! Offline training uses pure Rust floating-point Adam optimization.
//! Model exports exact discrete Z[phi] / H4 tables for zero-GEMM runtime serving.

use super::binary_model::compute_syntactic_register_from_context;
use super::embedding::{
    canonical_h4_hopf_s2, canonical_h4_roots, ContinuousEmbedding, H4_ROOT_COUNT,
};
use super::transition_table::{DiscreteServingTable, MultiLaneTransitionTables};
use crate::native_geometric::engram::{EngramTable, MAX_ENGRAM_COLLOCATIONS};
use crate::native_geometric::hopf_metric::{
    HopfFiberPointQ30, UnitS2, UnitS2Q30, UnitS3, UnitS3Q30, EPSILON,
};
use crate::native_geometric::lattice_table::{ContinuousLatticeTables, HierarchicalLatticeTables};
use crate::native_geometric::vsa::{
    encode_attended_multiscale_context, encode_multiscale_context, Codebook, HierarchicalCodebook,
    Hypervector, Hypervector4096,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Fiber U(1) canonical direction vectors for the 120 canonical H4 roots.
pub fn canonical_h4_fiber_u1() -> &'static [[f64; 2]; H4_ROOT_COUNT] {
    static FIBER_U1: OnceLock<[[f64; 2]; H4_ROOT_COUNT]> = OnceLock::new();
    FIBER_U1.get_or_init(|| {
        let roots = canonical_h4_roots();
        let mut arr = [[0.0; 2]; H4_ROOT_COUNT];
        for (i, r) in roots.iter().enumerate() {
            let phase = r.fiber_phase();
            arr[i] = [libm::cos(phase), libm::sin(phase)];
        }
        arr
    })
}

/// Tracks bigram, trigram, 4-gram, 5-gram, and conditioned skip-bigram collocation counts during training.
#[derive(Clone, Debug, Default)]
pub struct CollocationTracker {
    pub bigrams: HashMap<(u32, u32), HashMap<u32, u32>>,
    pub trigrams: HashMap<(u32, u32, u32), HashMap<u32, u32>>,
    pub fourgrams: HashMap<[u32; 4], HashMap<u32, u32>>,
    pub fivegrams: HashMap<[u32; 5], HashMap<u32, u32>>,
    pub skip_bigrams: HashMap<(u32, u32, u8), HashMap<u32, u32>>,
}

#[inline]
fn update_syntactic_tokens(token: usize, is_uor: bool, q: &mut u8, d: &mut u8) {
    if is_uor && token <= 255 {
        match token {
            36 => *q ^= 1,
            42 | 93 | 125 => *d = (*d + 1).min(3),
            43 | 95 | 127 => *d = d.saturating_sub(1),
            46 | 61 | 60 => {
                if *d == 0 {
                    *d = 1;
                }
            }
            48 | 65 | 35 => *d = 0,
            _ => {}
        }
    } else if token <= 255 {
        match token as u8 {
            b'"' => *q ^= 1,
            b'(' | b'[' | b'{' => *d = (*d + 1).min(3),
            b')' | b']' | b'}' => *d = d.saturating_sub(1),
            b',' | b';' | b':' => {
                if *d == 0 {
                    *d = 1;
                }
            }
            b'.' | b'?' | b'!' => *d = 0,
            _ => {}
        }
    }
}

impl CollocationTracker {
    pub fn record<T: AsTokenIndex>(&mut self, tokens: &[T]) {
        let mut q = 0_u8;
        let mut d = 0_u8;
        let is_uor = tokens.iter().any(|&t| {
            t.as_token_index() == 36 && tokens.iter().all(|&x| x.as_token_index() <= 255)
        });

        for t in 0..tokens.len() {
            let curr_tok = tokens[t].as_token_index() as u32;
            update_syntactic_tokens(tokens[t].as_token_index(), is_uor, &mut q, &mut d);
            let cond = (q & 1) | ((d & 3) << 1);

            if t + 1 < tokens.len() {
                let next = tokens[t + 1].as_token_index() as u32;

                // Bigram (w_{t-1}, w_t) -> w_{t+1}
                if t >= 1 {
                    let w_prev = tokens[t - 1].as_token_index() as u32;
                    *self
                        .bigrams
                        .entry((w_prev, curr_tok))
                        .or_default()
                        .entry(next)
                        .or_insert(0) += 1;
                }

                // Trigram (w_{t-2}, w_{t-1}, w_t) -> w_{t+1}
                if t >= 2 {
                    let w_prev2 = tokens[t - 2].as_token_index() as u32;
                    let w_prev = tokens[t - 1].as_token_index() as u32;
                    *self
                        .trigrams
                        .entry((w_prev2, w_prev, curr_tok))
                        .or_default()
                        .entry(next)
                        .or_insert(0) += 1;
                }

                // 4-gram context (w_{t-3}, w_{t-2}, w_{t-1}, w_t) -> w_{t+1}
                if t >= 3 {
                    let w_prev3 = tokens[t - 3].as_token_index() as u32;
                    let w_prev2 = tokens[t - 2].as_token_index() as u32;
                    let w_prev = tokens[t - 1].as_token_index() as u32;
                    *self
                        .fourgrams
                        .entry([w_prev3, w_prev2, w_prev, curr_tok])
                        .or_default()
                        .entry(next)
                        .or_insert(0) += 1;
                }

                // 5-gram context (w_{t-4}, w_{t-3}, w_{t-2}, w_{t-1}, w_t) -> w_{t+1}
                if t >= 4 {
                    let w_prev4 = tokens[t - 4].as_token_index() as u32;
                    let w_prev3 = tokens[t - 3].as_token_index() as u32;
                    let w_prev2 = tokens[t - 2].as_token_index() as u32;
                    let w_prev = tokens[t - 1].as_token_index() as u32;
                    *self
                        .fivegrams
                        .entry([w_prev4, w_prev3, w_prev2, w_prev, curr_tok])
                        .or_default()
                        .entry(next)
                        .or_insert(0) += 1;
                }

                // Conditioned skip-bigrams for k in {2, 4, 8}
                for &k in &[2, 4, 8] {
                    if t >= k {
                        let w_skip = tokens[t - k].as_token_index() as u32;
                        *self
                            .skip_bigrams
                            .entry((w_skip, curr_tok, cond))
                            .or_default()
                            .entry(next)
                            .or_insert(0) += 1;
                    }
                }
            }
        }
    }

    pub fn build_engram_table(&self) -> EngramTable {
        EngramTable::from_frequencies_full(
            &self.fivegrams,
            &self.fourgrams,
            &self.trigrams,
            &self.bigrams,
            &self.skip_bigrams,
            MAX_ENGRAM_COLLOCATIONS,
            2,
        )
    }

    pub fn merge(&mut self, other: &Self) {
        for (k, v) in &other.bigrams {
            let entry = self.bigrams.entry(*k).or_default();
            for (&next, &count) in v {
                *entry.entry(next).or_insert(0) += count;
            }
        }
        for (k, v) in &other.trigrams {
            let entry = self.trigrams.entry(*k).or_default();
            for (&next, &count) in v {
                *entry.entry(next).or_insert(0) += count;
            }
        }
        for (k, v) in &other.fourgrams {
            let entry = self.fourgrams.entry(*k).or_default();
            for (&next, &count) in v {
                *entry.entry(next).or_insert(0) += count;
            }
        }
        for (k, v) in &other.fivegrams {
            let entry = self.fivegrams.entry(*k).or_default();
            for (&next, &count) in v {
                *entry.entry(next).or_insert(0) += count;
            }
        }
        for (k, v) in &other.skip_bigrams {
            let entry = self.skip_bigrams.entry(*k).or_default();
            for (&next, &count) in v {
                *entry.entry(next).or_insert(0) += count;
            }
        }
    }
}

pub const DEFAULT_VSA_SEED: u64 = 0x5653_415f_3230_3236;

pub fn default_vsa_weight() -> f64 {
    0.35
}

pub fn default_fiber_weight() -> f64 {
    0.25
}

pub fn default_vsa_seed() -> u64 {
    DEFAULT_VSA_SEED
}

/// Configuration for the Native Geometric JEPA Trainer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaTrainerConfig {
    pub vocab_size: usize,
    pub num_lanes: usize,
    pub context_window: usize,
    pub learning_rate: f64,
    pub jepa_weight: f64,
    pub weight_decay: f64,
    pub grad_clip: f64,
    #[serde(default = "default_vsa_weight")]
    pub vsa_weight: f64,
    #[serde(default = "default_fiber_weight")]
    pub fiber_weight: f64,
    #[serde(default = "default_vsa_seed")]
    pub vsa_seed: u64,
    #[serde(default)]
    pub warmup_steps: usize,
    #[serde(default)]
    pub total_steps: usize,
    #[serde(default)]
    pub min_lr: f64,
}

impl Default for JepaTrainerConfig {
    fn default() -> Self {
        Self {
            vocab_size: 257, // 256 bytes + EOS
            num_lanes: 2,
            context_window: 64,
            learning_rate: 0.005,
            jepa_weight: 0.3,
            weight_decay: 1e-4,
            grad_clip: 1.0,
            vsa_weight: default_vsa_weight(),
            fiber_weight: default_fiber_weight(),
            vsa_seed: default_vsa_seed(),
            warmup_steps: 0,
            total_steps: 0,
            min_lr: 0.0,
        }
    }
}

/// Training loss and metrics for one sequence.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrainingMetrics {
    pub total_loss: f64,
    pub ce_loss: f64,
    pub jepa_loss: f64,
    pub bits_per_byte: f64,
    pub tokens_processed: usize,
}

/// Adam optimizer state for parameter updates.
#[derive(Debug, Clone, Default)]
struct AdamMoments {
    m: Vec<f64>,
    v: Vec<f64>,
}

impl AdamMoments {
    fn with_capacity(len: usize) -> Self {
        Self {
            m: vec![0.0; len],
            v: vec![0.0; len],
        }
    }

    fn update(
        &mut self,
        params: &mut [f64],
        grads: &[f64],
        lr: f64,
        beta1: f64,
        beta2: f64,
        weight_decay: f64,
        step: u64,
    ) {
        let step_f = step as f64;
        let bias_correction1 = 1.0 - libm::pow(beta1, step_f);
        let bias_correction2 = 1.0 - libm::pow(beta2, step_f);

        for i in 0..params.len() {
            let g = grads[i];
            self.m[i] = beta1 * self.m[i] + (1.0 - beta1) * g;
            self.v[i] = beta2 * self.v[i] + (1.0 - beta2) * g * g;

            let m_hat = self.m[i] / (bias_correction1 + EPSILON);
            let v_hat = self.v[i] / (bias_correction2 + EPSILON);

            if v_hat > 1e-12 {
                params[i] -= lr * m_hat / (libm::sqrt(v_hat) + 1e-8);
            }
            if weight_decay > 0.0 {
                params[i] -= lr * weight_decay * params[i];
            }
        }
    }

    fn update_4(
        &mut self,
        params: &mut [[f64; 4]],
        grads: &[[f64; 4]],
        lr: f64,
        beta1: f64,
        beta2: f64,
        weight_decay: f64,
        step: u64,
    ) {
        let step_f = step as f64;
        let bias_correction1 = 1.0 - libm::pow(beta1, step_f);
        let bias_correction2 = 1.0 - libm::pow(beta2, step_f);

        for (i, p) in params.iter_mut().enumerate() {
            let g = grads[i];
            for j in 0..4 {
                let idx = i * 4 + j;
                self.m[idx] = beta1 * self.m[idx] + (1.0 - beta1) * g[j];
                self.v[idx] = beta2 * self.v[idx] + (1.0 - beta2) * g[j] * g[j];

                let m_hat = self.m[idx] / (bias_correction1 + EPSILON);
                let v_hat = self.v[idx] / (bias_correction2 + EPSILON);

                if v_hat > 1e-12 {
                    p[j] -= lr * m_hat / (libm::sqrt(v_hat) + 1e-8);
                }
                if weight_decay > 0.0 {
                    p[j] -= lr * weight_decay * p[j];
                }
            }
        }
    }

    fn update_5(
        &mut self,
        params: &mut [[f64; 5]],
        grads: &[[f64; 5]],
        lr: f64,
        beta1: f64,
        beta2: f64,
        weight_decay: f64,
        step: u64,
    ) {
        let step_f = step as f64;
        let bias_correction1 = 1.0 - libm::pow(beta1, step_f);
        let bias_correction2 = 1.0 - libm::pow(beta2, step_f);

        for (i, p) in params.iter_mut().enumerate() {
            let g = grads[i];
            for j in 0..5 {
                let idx = i * 5 + j;
                self.m[idx] = beta1 * self.m[idx] + (1.0 - beta1) * g[j];
                self.v[idx] = beta2 * self.v[idx] + (1.0 - beta2) * g[j] * g[j];

                let m_hat = self.m[idx] / (bias_correction1 + EPSILON);
                let v_hat = self.v[idx] / (bias_correction2 + EPSILON);

                if v_hat > 1e-12 {
                    p[j] -= lr * m_hat / (libm::sqrt(v_hat) + 1e-8);
                }
                if weight_decay > 0.0 {
                    p[j] -= lr * weight_decay * p[j];
                }
            }
        }
    }
}

/// Native Geometric Language Model parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeGeometricLearnerModel {
    pub embeddings: ContinuousEmbedding,
    pub tables: MultiLaneTransitionTables,
    /// Emission bias per vocabulary token.
    pub emission_bias: Vec<f64>,
    #[serde(skip)]
    pub grad_bias: Vec<f64>,
    /// State projection weights mapping 5D manifold state [x, y, z, u, v] to vocabulary logits: [vocab_size x 5].
    pub s2_readout: Vec<[f64; 5]>,
    #[serde(skip)]
    pub grad_s2_readout: Vec<[f64; 5]>,
    /// Learned transition weights for JEPA S2 state prediction:
    /// W_state_s: 3x3 matrix mapping s2_state -> hat_state
    pub jepa_w_state: [f64; 9],
    #[serde(skip)]
    pub grad_jepa_w_state: [f64; 9],
    /// W_state_u: 3x3 matrix mapping u_base -> hat_state
    pub jepa_w_token: [f64; 9],
    #[serde(skip)]
    pub grad_jepa_w_token: [f64; 9],
    /// Bias: 3-vector
    pub jepa_bias: [f64; 3],
    #[serde(skip)]
    pub grad_jepa_bias: [f64; 3],
    /// Learned transition weights for JEPA U(1) fiber prediction:
    /// W_fiber_state: 2x2 matrix
    pub jepa_fiber_w_state: [f64; 4],
    #[serde(skip)]
    pub grad_jepa_fiber_w_state: [f64; 4],
    /// W_fiber_token: 2x2 matrix
    pub jepa_fiber_w_token: [f64; 4],
    #[serde(skip)]
    pub grad_jepa_fiber_w_token: [f64; 4],
    /// Fiber bias: 2-vector
    pub jepa_fiber_bias: [f64; 2],
    #[serde(skip)]
    pub grad_jepa_fiber_bias: [f64; 2],
    /// VSA context codebook and learnable context coupling scale.
    pub vsa_codebook: Codebook<64>,
    pub vsa_scale: f64,
    #[serde(skip)]
    pub grad_vsa_scale: f64,
}

impl NativeGeometricLearnerModel {
    pub fn new(
        vocab_size: usize,
        num_lanes: usize,
        seed: u64,
        vsa_seed: u64,
        initial_vsa_scale: f64,
    ) -> Self {
        let embeddings = ContinuousEmbedding::new(vocab_size, seed);
        let tables = MultiLaneTransitionTables::new(num_lanes, seed.wrapping_add(101));

        let mut s2_readout = Vec::with_capacity(vocab_size);
        let mut rng = seed.wrapping_mul(6364136223846793005).wrapping_add(42);
        for _ in 0..vocab_size {
            let mut w = [0.0; 5];
            for item in w.iter_mut() {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                *item = ((rng >> 32) as i32 as f64) / (i32::MAX as f64) * 0.05;
            }
            s2_readout.push(w);
        }

        let vsa_codebook = Codebook::<64>::new(vocab_size, vsa_seed);

        Self {
            embeddings,
            tables,
            emission_bias: vec![0.0; vocab_size],
            grad_bias: vec![0.0; vocab_size],
            s2_readout,
            grad_s2_readout: vec![[0.0; 5]; vocab_size],
            jepa_w_state: [0.7, 0.0, 0.0, 0.0, 0.7, 0.0, 0.0, 0.0, 0.7],
            grad_jepa_w_state: [0.0; 9],
            jepa_w_token: [0.3, 0.0, 0.0, 0.0, 0.3, 0.0, 0.0, 0.0, 0.3],
            grad_jepa_w_token: [0.0; 9],
            jepa_bias: [0.0; 3],
            grad_jepa_bias: [0.0; 3],
            jepa_fiber_w_state: [0.8, 0.0, 0.0, 0.8],
            grad_jepa_fiber_w_state: [0.0; 4],
            jepa_fiber_w_token: [0.2, 0.0, 0.0, 0.2],
            grad_jepa_fiber_w_token: [0.0; 4],
            jepa_fiber_bias: [0.0; 2],
            grad_jepa_fiber_bias: [0.0; 2],
            vsa_codebook,
            vsa_scale: initial_vsa_scale,
            grad_vsa_scale: 0.0,
        }
    }

    pub fn zero_grad(&mut self) {
        self.embeddings.zero_grad();
        self.tables.zero_grad();
        for b in self.grad_bias.iter_mut() {
            *b = 0.0;
        }
        for g in self.grad_s2_readout.iter_mut() {
            *g = [0.0; 5];
        }
        self.grad_jepa_w_state = [0.0; 9];
        self.grad_jepa_w_token = [0.0; 9];
        self.grad_jepa_bias = [0.0; 3];
        self.grad_jepa_fiber_w_state = [0.0; 4];
        self.grad_jepa_fiber_w_token = [0.0; 4];
        self.grad_jepa_fiber_bias = [0.0; 2];
        self.grad_vsa_scale = 0.0;
    }

    /// Continuous JEPA latent state prediction:
    /// Predicts next geometric state \hat{s}_{t+1} = (\hat{s2}_{t+1}, \hat{u1}_{t+1}) on S2 and U(1)
    /// given current cumulative state (s2_state, u1_state) and current observed token embedding (u_base, token_u1).
    pub fn predict_jepa_step(
        &self,
        s2_state: UnitS2,
        u1_state: [f64; 2],
        u_base: UnitS2,
        token_u1: [f64; 2],
    ) -> (UnitS2, [f64; 2], f64, f64) {
        let ws = &self.jepa_w_state;
        let wt = &self.jepa_w_token;
        let b = &self.jepa_bias;

        let hat_vx = ws[0] * s2_state.x
            + ws[1] * s2_state.y
            + ws[2] * s2_state.z
            + wt[0] * u_base.x
            + wt[1] * u_base.y
            + wt[2] * u_base.z
            + b[0];
        let hat_vy = ws[3] * s2_state.x
            + ws[4] * s2_state.y
            + ws[5] * s2_state.z
            + wt[3] * u_base.x
            + wt[4] * u_base.y
            + wt[5] * u_base.z
            + b[1];
        let hat_vz = ws[6] * s2_state.x
            + ws[7] * s2_state.y
            + ws[8] * s2_state.z
            + wt[6] * u_base.x
            + wt[7] * u_base.y
            + wt[8] * u_base.z
            + b[2];

        let norm_sq = hat_vx * hat_vx + hat_vy * hat_vy + hat_vz * hat_vz;
        let r = libm::sqrt(norm_sq) + EPSILON;
        let inv_r = 1.0 / r;
        let hat_s2 = UnitS2 {
            x: hat_vx * inv_r,
            y: hat_vy * inv_r,
            z: hat_vz * inv_r,
        };

        let wfs = &self.jepa_fiber_w_state;
        let wft = &self.jepa_fiber_w_token;
        let bf = &self.jepa_fiber_bias;

        let hat_fx = wfs[0] * u1_state[0]
            + wfs[1] * u1_state[1]
            + wft[0] * token_u1[0]
            + wft[1] * token_u1[1]
            + bf[0];
        let hat_fy = wfs[2] * u1_state[0]
            + wfs[3] * u1_state[1]
            + wft[2] * token_u1[0]
            + wft[3] * token_u1[1]
            + bf[1];

        let norm_f_sq = hat_fx * hat_fx + hat_fy * hat_fy;
        let rf = libm::sqrt(norm_f_sq) + EPSILON;
        let inv_rf = 1.0 / rf;
        let hat_u1 = [hat_fx * inv_rf, hat_fy * inv_rf];

        (hat_s2, hat_u1, inv_r, inv_rf)
    }
}

/// Discrete Model exported for the serving hot path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedGeometricModel {
    pub vocab_size: usize,
    /// Nearest H4 root for each vocabulary token [0..255, EOS].
    pub token_to_root: Vec<u8>,
    /// Quantized discrete 120x120 compatibility tables per lane.
    pub discrete_tables: Vec<DiscreteServingTable>,
    /// Quantized emission bias in fixed-point / integer format.
    pub discrete_bias: Vec<i32>,
    /// Readout weights quantized to Q1.14: [vocab_size x 5] (3 base S2 dims + 2 fiber U(1) dims).
    pub discrete_s2_readout: Vec<[i16; 5]>,
    /// Quantized JEPA state transition matrix in Q1.14: [3 x 3].
    #[serde(default)]
    pub discrete_jepa_w_state: [i16; 9],
    /// Quantized JEPA token transition matrix in Q1.14: [3 x 3].
    #[serde(default)]
    pub discrete_jepa_w_token: [i16; 9],
    /// Quantized JEPA state bias in Q1.14: [3].
    #[serde(default)]
    pub discrete_jepa_bias: [i16; 3],
    /// Quantized JEPA U(1) fiber transition matrix in Q1.14: [2 x 2].
    #[serde(default)]
    pub discrete_jepa_fiber_w_state: [i16; 4],
    /// Quantized JEPA U(1) fiber token transition matrix in Q1.14: [2 x 2].
    #[serde(default)]
    pub discrete_jepa_fiber_w_token: [i16; 4],
    /// Quantized JEPA U(1) fiber bias in Q1.14: [2].
    #[serde(default)]
    pub discrete_jepa_fiber_bias: [i16; 2],
    /// VSA codebook seed for deterministic on-demand hypervector retrieval.
    #[serde(default = "default_vsa_seed")]
    pub vsa_seed: u64,
    /// VSA context scoring scale in Q1.15 format.
    #[serde(default)]
    pub vsa_scale_q15: i16,
    /// Hierarchical lattice codebook for bounded-shortlist zero-allocation routing.
    #[serde(default)]
    pub hierarchical_codebook: Option<HierarchicalCodebook<64>>,
    /// Flat integer engram table for zero-allocation O(1) collocation retrieval.
    #[serde(default)]
    pub engram_table: Option<EngramTable>,
    /// 2-level hierarchical lattice tables (coarse 120^3 root trigram + fine 480^2 cluster bigram).
    #[serde(default)]
    pub hierarchical_lattice: Option<HierarchicalLatticeTables>,
}

impl ExportedGeometricModel {
    /// O(1) query-key scoring across all lanes with zero runtime matrix multiplications.
    #[inline]
    pub fn score_compatibility(&self, query_token: usize, key_token: usize) -> i32 {
        if self.token_to_root.is_empty() {
            return 0;
        }
        let root_len = self.token_to_root.len();
        let q_root = self.token_to_root[query_token.min(root_len - 1)] as usize;
        let k_root = self.token_to_root[key_token.min(root_len - 1)] as usize;
        let mut total = 0_i32;
        for table in &self.discrete_tables {
            total += table.score(q_root, k_root) as i32;
        }
        total
    }

    /// Compute cumulative fixed-point S3 state and fiber-preserving Hopf projection over context tokens.
    pub fn context_hopf_fiber_q30(&self, context: &[usize]) -> HopfFiberPointQ30 {
        if self.token_to_root.is_empty() || context.is_empty() {
            return UnitS3Q30::IDENTITY.hopf_fiber_project();
        }
        let roots = super::embedding::canonical_h4_roots_q30();
        let mut s3 = UnitS3Q30::IDENTITY;
        let start = context.len().saturating_sub(64);
        let root_len = self.token_to_root.len();
        let mut step = 0;
        for &token in &context[start..] {
            let root_idx = self.token_to_root[token.min(root_len - 1)] as usize;
            let q_root_q30 = roots[root_idx % H4_ROOT_COUNT];
            s3 = s3.mul_q30(&q_root_q30);
            step += 1;
            if step % 8 == 0 {
                s3 = s3.normalized();
            }
        }
        if step % 8 != 0 {
            s3 = s3.normalized();
        }
        s3.hopf_fiber_project()
    }

    /// Compute cumulative fixed-point S2 Hopf projection over context tokens.
    pub fn context_hopf_s2(&self, context: &[usize]) -> UnitS2Q30 {
        self.context_hopf_fiber_q30(context).base
    }

    /// Score candidate token given recent context history across all attention lanes,
    /// fiber-preserving geometric state projection, emission bias, and VSA context similarity.
    #[inline]
    pub fn score_context_candidate(
        &self,
        context: &[usize],
        candidate: usize,
        fiber_pt: HopfFiberPointQ30,
    ) -> i32 {
        self.score_context_candidate_with_vsa(context, candidate, fiber_pt, None)
    }

    /// Score candidate token with optional precomputed VSA context vector to eliminate
    /// redundant multiscale context encoding in inner candidate loops.
    pub fn score_context_candidate_with_vsa(
        &self,
        context: &[usize],
        candidate: usize,
        fiber_pt: HopfFiberPointQ30,
        precomputed_vsa: Option<(&Codebook<64>, &Hypervector<64>)>,
    ) -> i32 {
        if self.token_to_root.is_empty() {
            return self.discrete_bias.get(candidate).copied().unwrap_or(0);
        }
        let root_len = self.token_to_root.len();
        let cand_root = self.token_to_root[candidate.min(root_len - 1)] as usize;
        let mut total = self.discrete_bias.get(candidate).copied().unwrap_or(0);

        let ctx_len = context.len();
        for (l, table) in self.discrete_tables.iter().enumerate() {
            let lag = l + 1;
            if ctx_len >= lag {
                let ctx_token = context[ctx_len - lag];
                let ctx_root = self.token_to_root[ctx_token.min(root_len - 1)] as usize;
                total += table.score(ctx_root, cand_root) as i32;
            }
        }

        if let Some(r) = self.discrete_s2_readout.get(candidate) {
            let s2 = fiber_pt.base.0;
            let u1 = fiber_pt.fiber_u1;
            let s2_proj = (s2[0] as i64 * r[0] as i64
                + s2[1] as i64 * r[1] as i64
                + s2[2] as i64 * r[2] as i64
                + u1[0] as i64 * r[3] as i64
                + u1[1] as i64 * r[4] as i64)
                >> 31;
            total += s2_proj as i32;
        }

        if self.vsa_scale_q15 != 0 && ctx_len > 0 {
            let sim_q15 = match precomputed_vsa {
                Some((cb, ctx_vec)) => {
                    if let Some(cand_vec) = cb.get_ref(candidate as u32) {
                        ctx_vec.bipolar_correlation_q15(cand_vec)
                    } else {
                        let cand_vec = cb.get(candidate as u32);
                        ctx_vec.bipolar_correlation_q15(&cand_vec)
                    }
                }
                None => {
                    let codebook = Codebook::<64>::on_demand(self.vocab_size, self.vsa_seed);
                    let mut buf = [0u32; 64];
                    let n = ctx_len.min(64);
                    for i in 0..n {
                        buf[i] = context[ctx_len - n + i] as u32;
                    }
                    let ctx_vec = encode_attended_multiscale_context(&buf[..n], &codebook, 64);
                    let cand_vec = codebook.get(candidate as u32);
                    ctx_vec.bipolar_correlation_q15(&cand_vec)
                }
            };
            let vsa_score = (self.vsa_scale_q15 as i32 * sim_q15 as i32) >> 16;
            total += vsa_score;
        }

        if let Some(engram) = &self.engram_table {
            if ctx_len >= 5 {
                let w_curr = context[ctx_len - 1] as u32;
                let w_prev = context[ctx_len - 2] as u32;
                let w_prev2 = context[ctx_len - 3] as u32;
                let w_prev3 = context[ctx_len - 4] as u32;
                let w_prev4 = context[ctx_len - 5] as u32;
                if let Some(cands) = engram.lookup_5gram(w_prev4, w_prev3, w_prev2, w_prev, w_curr)
                {
                    for &(c, q15) in cands {
                        if c == candidate as u32 {
                            total += q15 as i32;
                        }
                    }
                }
            }
            if ctx_len >= 4 {
                let w_curr = context[ctx_len - 1] as u32;
                let w_prev = context[ctx_len - 2] as u32;
                let w_prev2 = context[ctx_len - 3] as u32;
                let w_prev3 = context[ctx_len - 4] as u32;
                if let Some(cands) = engram.lookup_4gram(w_prev3, w_prev2, w_prev, w_curr) {
                    for &(c, q15) in cands {
                        if c == candidate as u32 {
                            total += q15 as i32;
                        }
                    }
                }
            }
            if ctx_len >= 3 {
                let w_curr = context[ctx_len - 1] as u32;
                let w_prev = context[ctx_len - 2] as u32;
                let w_prev2 = context[ctx_len - 3] as u32;
                if let Some(cands) = engram.lookup_trigram(w_prev2, w_prev, w_curr) {
                    for &(c, q15) in cands {
                        if c == candidate as u32 {
                            total += q15 as i32;
                        }
                    }
                }
            }
            if ctx_len >= 2 {
                let w_curr = context[ctx_len - 1] as u32;
                let w_prev = context[ctx_len - 2] as u32;
                if let Some(cands) = engram.lookup_bigram(w_prev, w_curr) {
                    for &(c, q15) in cands {
                        if c == candidate as u32 {
                            total += q15 as i32;
                        }
                    }
                }
            }
            let syn = compute_syntactic_register_from_context(context);
            for &k in &[2, 4, 8] {
                if ctx_len >= k + 1 {
                    let w_curr = context[ctx_len - 1] as u32;
                    let w_skip = context[ctx_len - (k + 1)] as u32;
                    if let Some(cands) = engram.lookup_skip(w_skip, w_curr, syn) {
                        for &(c, q15) in cands {
                            if c == candidate as u32 {
                                total += q15 as i32;
                            }
                        }
                    }
                }
            }
        }

        if let Some(lattice) = &self.hierarchical_lattice {
            let curr_tok = if ctx_len >= 1 {
                context[ctx_len - 1]
            } else {
                usize::MAX
            };
            let is_self = candidate == curr_tok;
            if ctx_len >= 2 {
                let prev_tok = context[ctx_len - 2];
                let r_prev = self.token_to_root[prev_tok.min(root_len - 1)] as usize;
                let r_curr = self.token_to_root[curr_tok.min(root_len - 1)] as usize;
                let c_curr = lattice.cluster_of(curr_tok);
                let c_cand = lattice.cluster_of(candidate);
                total += lattice.score_token(r_prev, r_curr, cand_root, c_curr, c_cand, is_self);
            } else if ctx_len == 1 {
                let r_curr = self.token_to_root[curr_tok.min(root_len - 1)] as usize;
                let c_curr = lattice.cluster_of(curr_tok);
                let c_cand = lattice.cluster_of(candidate);
                total += lattice.score_token(r_curr, r_curr, cand_root, c_curr, c_cand, is_self);
            }
        }

        total
    }

    /// Readout score for candidate token given fiber state with zero runtime floats.
    #[inline]
    pub fn score_readout(&self, candidate: usize, fiber_pt: HopfFiberPointQ30) -> i32 {
        if let Some(r) = self.discrete_s2_readout.get(candidate) {
            let s2 = fiber_pt.base.0;
            let u1 = fiber_pt.fiber_u1;
            let proj = (s2[0] as i64 * r[0] as i64
                + s2[1] as i64 * r[1] as i64
                + s2[2] as i64 * r[2] as i64
                + u1[0] as i64 * r[3] as i64
                + u1[1] as i64 * r[4] as i64)
                >> 31;
            proj as i32
        } else {
            0
        }
    }

    /// Readout score for candidate token given only S2 base vector.
    #[inline]
    pub fn score_readout_s2(&self, candidate: usize, s2_state: UnitS2Q30) -> i32 {
        if let Some(r) = self.discrete_s2_readout.get(candidate) {
            let s2 = s2_state.0;
            let proj = (s2[0] as i64 * r[0] as i64
                + s2[1] as i64 * r[1] as i64
                + s2[2] as i64 * r[2] as i64)
                >> 31;
            proj as i32
        } else {
            0
        }
    }

    /// Score continuation candidate against precomputed VSA context vector.
    #[inline]
    pub fn score_vsa_candidate(
        &self,
        codebook: &Codebook<64>,
        context_vec: &Hypervector4096,
        candidate: u32,
    ) -> i32 {
        if self.vsa_scale_q15 == 0 {
            return 0;
        }
        let cand_vec = codebook.get(candidate);
        let sim_q15 = context_vec.bipolar_correlation_q15(&cand_vec);
        (self.vsa_scale_q15 as i32 * sim_q15 as i32) >> 16
    }

    /// Compute S2 Hopf state directly from a ring buffer without heap allocations.
    pub fn context_s2_from_ring(&self, ring: &[u32], cursor: usize, length: usize) -> UnitS2Q30 {
        self.context_fiber_from_ring(ring, cursor, length).base
    }

    /// Compute full fiber-preserving Hopf state directly from a ring buffer without heap allocations.
    pub fn context_fiber_from_ring(
        &self,
        ring: &[u32],
        cursor: usize,
        length: usize,
    ) -> HopfFiberPointQ30 {
        if length == 0 || ring.is_empty() || self.token_to_root.is_empty() {
            return UnitS3Q30::IDENTITY.hopf_fiber_project();
        }
        let roots = super::embedding::canonical_h4_roots_q30();
        let mut s3 = UnitS3Q30::IDENTITY;
        let window = length.min(64).min(ring.len());
        let root_len = self.token_to_root.len();
        let cursor = cursor % ring.len();
        let mut step = 0;
        for lag in (1..=window).rev() {
            let index = if cursor >= lag {
                cursor - lag
            } else {
                ring.len() - (lag - cursor)
            };
            let token = ring[index] as usize;
            let root_idx = self.token_to_root[token.min(root_len - 1)] as usize;
            let q_root_q30 = roots[root_idx % H4_ROOT_COUNT];
            s3 = s3.mul_q30(&q_root_q30);
            step += 1;
            if step % 8 == 0 {
                s3 = s3.normalized();
            }
        }
        if step % 8 != 0 {
            s3 = s3.normalized();
        }
        s3.hopf_fiber_project()
    }

    /// Compute VSA context hypervector directly from a ring buffer without heap allocations.
    pub fn context_vsa_from_ring(
        &self,
        codebook: &Codebook<64>,
        ring: &[u32],
        cursor: usize,
        length: usize,
    ) -> Hypervector4096 {
        if length == 0 || ring.is_empty() {
            return Hypervector4096::zero();
        }
        let mut buf = [0u32; 64];
        let window = length.min(64).min(ring.len());
        let cursor = cursor % ring.len();
        for i in 0..window {
            let lag = window - i;
            let index = if cursor >= lag {
                cursor - lag
            } else {
                ring.len() - (lag - cursor)
            };
            buf[i] = ring[index];
        }
        encode_attended_multiscale_context(&buf[..window], codebook, 64)
    }

    /// Quantized JEPA transition predicting next S2 state and fiber U(1) state using
    /// integer bit shifts and multiplications only (zero runtime floats).
    pub fn predict_jepa_step_q30(
        &self,
        s2_state: UnitS2Q30,
        fiber_u1: [i32; 2],
        token_s2: UnitS2Q30,
        token_u1: [i32; 2],
    ) -> (UnitS2Q30, [i32; 2]) {
        let ws = &self.discrete_jepa_w_state;
        let wt = &self.discrete_jepa_w_token;
        let b = &self.discrete_jepa_bias;

        let s2 = s2_state.0;
        let u = token_s2.0;

        let hat_vx = ((ws[0] as i64 * s2[0] as i64
            + ws[1] as i64 * s2[1] as i64
            + ws[2] as i64 * s2[2] as i64
            + wt[0] as i64 * u[0] as i64
            + wt[1] as i64 * u[1] as i64
            + wt[2] as i64 * u[2] as i64)
            >> 14)
            + ((b[0] as i64) << 16);
        let hat_vy = ((ws[3] as i64 * s2[0] as i64
            + ws[4] as i64 * s2[1] as i64
            + ws[5] as i64 * s2[2] as i64
            + wt[3] as i64 * u[0] as i64
            + wt[4] as i64 * u[1] as i64
            + wt[5] as i64 * u[2] as i64)
            >> 14)
            + ((b[1] as i64) << 16);
        let hat_vz = ((ws[6] as i64 * s2[0] as i64
            + ws[7] as i64 * s2[1] as i64
            + ws[8] as i64 * s2[2] as i64
            + wt[6] as i64 * u[0] as i64
            + wt[7] as i64 * u[1] as i64
            + wt[8] as i64 * u[2] as i64)
            >> 14)
            + ((b[2] as i64) << 16);

        let pred_s2 = UnitS2Q30::normalize([hat_vx, hat_vy, hat_vz]);

        let wfs = &self.discrete_jepa_fiber_w_state;
        let wft = &self.discrete_jepa_fiber_w_token;
        let bf = &self.discrete_jepa_fiber_bias;

        let hat_fx = ((wfs[0] as i64 * fiber_u1[0] as i64
            + wfs[1] as i64 * fiber_u1[1] as i64
            + wft[0] as i64 * token_u1[0] as i64
            + wft[1] as i64 * token_u1[1] as i64)
            >> 14)
            + ((bf[0] as i64) << 16);
        let hat_fy = ((wfs[2] as i64 * fiber_u1[0] as i64
            + wfs[3] as i64 * fiber_u1[1] as i64
            + wft[2] as i64 * token_u1[0] as i64
            + wft[3] as i64 * token_u1[1] as i64)
            >> 14)
            + ((bf[1] as i64) << 16);

        let pred_fiber = UnitS2Q30::normalize_u1([hat_fx, hat_fy]);

        (pred_s2, pred_fiber)
    }

    /// Predict next geometric state s_{t+1} = (s2_{t+1}, u1_{t+1}) given current historical fiber
    /// state s_t and most recent observed token w_t using discrete integer JEPA weights.
    /// Zero runtime floats, zero heap allocations.
    pub fn predict_next_fiber(
        &self,
        hist_fiber: HopfFiberPointQ30,
        last_token: u32,
    ) -> HopfFiberPointQ30 {
        if self.discrete_jepa_w_state == [0; 9]
            && self.discrete_jepa_w_token == [0; 9]
            && self.discrete_jepa_bias == [0; 3]
            && self.discrete_jepa_fiber_w_state == [0; 4]
            && self.discrete_jepa_fiber_w_token == [0; 4]
            && self.discrete_jepa_fiber_bias == [0; 2]
        {
            return hist_fiber;
        }
        if self.token_to_root.is_empty() {
            return hist_fiber;
        }
        let root_len = self.token_to_root.len();
        let root_idx = self.token_to_root[(last_token as usize).min(root_len - 1)] as usize;
        let token_fiber =
            super::embedding::canonical_h4_fiber_roots_q30()[root_idx % H4_ROOT_COUNT];
        let (pred_s2, pred_u1) = self.predict_jepa_step_q30(
            hist_fiber.base,
            hist_fiber.fiber_u1,
            token_fiber.base,
            token_fiber.fiber_u1,
        );
        HopfFiberPointQ30 {
            base: pred_s2,
            fiber_u1: pred_u1,
            fiber_phase: crate::native_geometric::hopf_metric::atan2_q30(pred_u1[1], pred_u1[0]),
        }
    }

    /// Compute cumulative historical Hopf state s_t and forward-predict s_{t+1} given context history.
    /// Zero runtime floats, zero heap allocations.
    pub fn context_predicted_fiber_q30(&self, context: &[usize]) -> HopfFiberPointQ30 {
        let hist = self.context_hopf_fiber_q30(context);
        if let Some(&last_tok) = context.last() {
            self.predict_next_fiber(hist, last_tok as u32)
        } else {
            hist
        }
    }

    /// Compute cumulative historical Hopf state s_t from ring buffer, and if length > 0,
    /// forward-predict the next geometric state s_{t+1} = predict_jepa_step_q30(s2_t, u1_t, w_t).
    /// Zero runtime floats, zero heap allocations.
    pub fn context_predicted_fiber_from_ring(
        &self,
        ring: &[u32],
        cursor: usize,
        length: usize,
    ) -> HopfFiberPointQ30 {
        let hist_fiber = self.context_fiber_from_ring(ring, cursor, length);
        if length > 0 && !ring.is_empty() {
            let last_index = if cursor >= 1 {
                cursor - 1
            } else {
                ring.len() - 1
            };
            let last_token = ring[last_index];
            self.predict_next_fiber(hist_fiber, last_token)
        } else {
            hist_fiber
        }
    }
}

/// Maximum capacity of a leaf bucket (H4 root sector tokens).
pub const MAX_LEAF_BUCKET_SIZE: usize = 128;

/// Voronoi lattice partitioning of vocabulary tokens into coarse H4 root sectors and fine leaf buckets.
#[derive(Clone, Debug)]
pub struct VoronoiLatticePartition {
    pub token_to_root: Vec<usize>,
    pub token_to_leaf: Vec<usize>,
    pub token_to_leaf_idx: Vec<usize>,
    pub leaf_buckets: Vec<Vec<usize>>,
    pub leaf_to_root: Vec<usize>,
    pub root_anchors: Box<[Hypervector4096; H4_ROOT_COUNT]>,
}

impl VoronoiLatticePartition {
    pub fn build(
        vocab_size: usize,
        embeddings: &ContinuousEmbedding,
        vsa_codebook: &Codebook<64>,
    ) -> Self {
        let mut token_to_root = Vec::with_capacity(vocab_size);
        for v in 0..vocab_size {
            token_to_root.push(embeddings.nearest_h4_root(v));
        }

        let mut root_tokens: [Vec<usize>; H4_ROOT_COUNT] = core::array::from_fn(|_| Vec::new());
        for v in 0..vocab_size {
            root_tokens[token_to_root[v]].push(v);
        }

        let mut root_anchors_vec = vec![Hypervector4096::zero(); H4_ROOT_COUNT];
        let mut leaf_buckets = Vec::with_capacity(H4_ROOT_COUNT);
        let mut leaf_to_root = Vec::with_capacity(H4_ROOT_COUNT);
        let mut token_to_leaf = vec![0usize; vocab_size];
        let mut token_to_leaf_idx = vec![0usize; vocab_size];

        for r in 0..H4_ROOT_COUNT {
            assert!(
                root_tokens[r].len() <= MAX_LEAF_BUCKET_SIZE,
                "Root sector {} token count {} exceeds MAX_LEAF_BUCKET_SIZE {}",
                r,
                root_tokens[r].len(),
                MAX_LEAF_BUCKET_SIZE
            );
            if root_tokens[r].is_empty() {
                root_anchors_vec[r] = vsa_codebook.get_atom(&format!("h4_root_{}", r));
            } else {
                let token_vecs: Vec<Hypervector4096> = root_tokens[r]
                    .iter()
                    .map(|&t| vsa_codebook.get(t as u32))
                    .collect();
                root_anchors_vec[r] = Hypervector4096::bundle(&token_vecs);

                for (idx_in_leaf, &v) in root_tokens[r].iter().enumerate() {
                    token_to_leaf[v] = r;
                    token_to_leaf_idx[v] = idx_in_leaf;
                }
            }
            leaf_buckets.push(root_tokens[r].clone());
            leaf_to_root.push(r);
        }

        let root_anchors: Box<[Hypervector4096; H4_ROOT_COUNT]> =
            match root_anchors_vec.into_boxed_slice().try_into() {
                Ok(b) => b,
                Err(_) => unreachable!("exact length H4_ROOT_COUNT"),
            };

        Self {
            token_to_root,
            token_to_leaf,
            token_to_leaf_idx,
            leaf_buckets,
            leaf_to_root,
            root_anchors,
        }
    }
}

/// Pre-allocated per-thread scratch buffers eliminating inner-loop heap allocations.
#[derive(Clone, Debug)]
pub struct TrainingScratch {
    pub root_logits: [f64; H4_ROOT_COUNT],
    pub root_probs: [f64; H4_ROOT_COUNT],
    pub leaf_logits: [f64; MAX_LEAF_BUCKET_SIZE],
    pub leaf_probs: [f64; MAX_LEAF_BUCKET_SIZE],
    pub leaf_vsa_sim: [f64; MAX_LEAF_BUCKET_SIZE],
    pub lane_root_score: [f64; H4_ROOT_COUNT],
    pub coarse_step_lookup: [f64; H4_ROOT_COUNT],
    pub fine_step_lookup: [f64; crate::native_geometric::lattice_table::MAX_FINE_CLUSTERS],
    pub seq_q_base: Vec<UnitS3>,
    pub seq_u_base: Vec<UnitS2>,
    pub ctx_buf: [u32; 64],
}

impl TrainingScratch {
    pub fn new() -> Self {
        Self {
            root_logits: [0.0; H4_ROOT_COUNT],
            root_probs: [0.0; H4_ROOT_COUNT],
            leaf_logits: [0.0; MAX_LEAF_BUCKET_SIZE],
            leaf_probs: [0.0; MAX_LEAF_BUCKET_SIZE],
            leaf_vsa_sim: [0.0; MAX_LEAF_BUCKET_SIZE],
            lane_root_score: [0.0; H4_ROOT_COUNT],
            coarse_step_lookup: [0.0; H4_ROOT_COUNT],
            fine_step_lookup: [0.0; crate::native_geometric::lattice_table::MAX_FINE_CLUSTERS],
            seq_q_base: Vec::with_capacity(256),
            seq_u_base: Vec::with_capacity(256),
            ctx_buf: [0u32; 64],
        }
    }
}

impl Default for TrainingScratch {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-local gradient accumulator for Rayon parallel sequence training.
#[derive(Clone, Debug)]
pub struct GradAccumulator {
    pub grad_bias: Vec<f64>,
    pub grad_s2_readout: Vec<[f64; 5]>,
    pub grad_jepa_w_state: [f64; 9],
    pub grad_jepa_w_token: [f64; 9],
    pub grad_jepa_bias: [f64; 3],
    pub grad_jepa_fiber_w_state: [f64; 4],
    pub grad_jepa_fiber_w_token: [f64; 4],
    pub grad_jepa_fiber_bias: [f64; 2],
    pub grad_vsa_scale: f64,
    pub grad_tables: Vec<Vec<f64>>,
    pub grad_emb_base: Vec<[f64; 4]>,
    pub grad_emb_comp: Vec<[f64; 4]>,
    pub total_loss: f64,
    pub ce_loss: f64,
    pub jepa_loss: f64,
    pub tokens_processed: usize,
    pub token_counts: Vec<u32>,
}

impl GradAccumulator {
    pub fn new(vocab_size: usize, num_lanes: usize) -> Self {
        Self {
            grad_bias: vec![0.0; vocab_size],
            grad_s2_readout: vec![[0.0; 5]; vocab_size],
            grad_jepa_w_state: [0.0; 9],
            grad_jepa_w_token: [0.0; 9],
            grad_jepa_bias: [0.0; 3],
            grad_jepa_fiber_w_state: [0.0; 4],
            grad_jepa_fiber_w_token: [0.0; 4],
            grad_jepa_fiber_bias: [0.0; 2],
            grad_vsa_scale: 0.0,
            grad_tables: (0..num_lanes).map(|_| vec![0.0; 14_400]).collect(),
            grad_emb_base: vec![[0.0; 4]; vocab_size],
            grad_emb_comp: vec![[0.0; 4]; vocab_size],
            total_loss: 0.0,
            ce_loss: 0.0,
            jepa_loss: 0.0,
            tokens_processed: 0,
            token_counts: vec![0u32; vocab_size],
        }
    }

    pub fn zero(&mut self) {
        for b in self.grad_bias.iter_mut() {
            *b = 0.0;
        }
        for r in self.grad_s2_readout.iter_mut() {
            *r = [0.0; 5];
        }
        self.grad_jepa_w_state = [0.0; 9];
        self.grad_jepa_w_token = [0.0; 9];
        self.grad_jepa_bias = [0.0; 3];
        self.grad_jepa_fiber_w_state = [0.0; 4];
        self.grad_jepa_fiber_w_token = [0.0; 4];
        self.grad_jepa_fiber_bias = [0.0; 2];
        self.grad_vsa_scale = 0.0;
        for t in self.grad_tables.iter_mut() {
            for g in t.iter_mut() {
                *g = 0.0;
            }
        }
        for e in self.grad_emb_base.iter_mut() {
            *e = [0.0; 4];
        }
        for e in self.grad_emb_comp.iter_mut() {
            *e = [0.0; 4];
        }
        self.total_loss = 0.0;
        self.ce_loss = 0.0;
        self.jepa_loss = 0.0;
        self.tokens_processed = 0;
        for c in self.token_counts.iter_mut() {
            *c = 0;
        }
    }

    pub fn merge(&mut self, other: &Self) {
        for i in 0..self.grad_bias.len() {
            self.grad_bias[i] += other.grad_bias[i];
        }
        for i in 0..self.grad_s2_readout.len() {
            for j in 0..5 {
                self.grad_s2_readout[i][j] += other.grad_s2_readout[i][j];
            }
        }
        for i in 0..9 {
            self.grad_jepa_w_state[i] += other.grad_jepa_w_state[i];
            self.grad_jepa_w_token[i] += other.grad_jepa_w_token[i];
        }
        for i in 0..3 {
            self.grad_jepa_bias[i] += other.grad_jepa_bias[i];
        }
        for i in 0..4 {
            self.grad_jepa_fiber_w_state[i] += other.grad_jepa_fiber_w_state[i];
            self.grad_jepa_fiber_w_token[i] += other.grad_jepa_fiber_w_token[i];
        }
        for i in 0..2 {
            self.grad_jepa_fiber_bias[i] += other.grad_jepa_fiber_bias[i];
        }
        self.grad_vsa_scale += other.grad_vsa_scale;
        for (t1, t2) in self.grad_tables.iter_mut().zip(other.grad_tables.iter()) {
            for (g1, &g2) in t1.iter_mut().zip(t2.iter()) {
                *g1 += g2;
            }
        }
        for i in 0..self.grad_emb_base.len() {
            for j in 0..4 {
                self.grad_emb_base[i][j] += other.grad_emb_base[i][j];
                self.grad_emb_comp[i][j] += other.grad_emb_comp[i][j];
            }
        }
        for i in 0..self.token_counts.len() {
            if i < other.token_counts.len() {
                self.token_counts[i] = self.token_counts[i].saturating_add(other.token_counts[i]);
            }
        }
        self.total_loss += other.total_loss;
        self.ce_loss += other.ce_loss;
        self.jepa_loss += other.jepa_loss;
        self.tokens_processed += other.tokens_processed;
    }
}

/// Helper trait for converting diverse token representations to usize token indices.
pub trait AsTokenIndex: Copy + Send + Sync {
    fn as_token_index(self) -> usize;
}

impl AsTokenIndex for usize {
    #[inline(always)]
    fn as_token_index(self) -> usize {
        self
    }
}

impl AsTokenIndex for u16 {
    #[inline(always)]
    fn as_token_index(self) -> usize {
        self as usize
    }
}

impl AsTokenIndex for u32 {
    #[inline(always)]
    fn as_token_index(self) -> usize {
        self as usize
    }
}

/// Main trainer executing the JEPA + Autoregressive optimization loop.
pub struct JepaTrainer {
    pub model: NativeGeometricLearnerModel,
    pub config: JepaTrainerConfig,
    adam_emb_base: AdamMoments,
    adam_emb_comp: AdamMoments,
    adam_tables: Vec<AdamMoments>,
    adam_bias: AdamMoments,
    adam_readout: AdamMoments,
    adam_jepa_state: AdamMoments,
    adam_jepa_token: AdamMoments,
    adam_jepa_bias: AdamMoments,
    adam_jepa_fiber_state: AdamMoments,
    adam_jepa_fiber_token: AdamMoments,
    adam_jepa_fiber_bias: AdamMoments,
    adam_vsa: AdamMoments,
    pub collocations: CollocationTracker,
    pub continuous_lattice: Option<ContinuousLatticeTables>,
    pub step_count: u64,
    pub partition: VoronoiLatticePartition,
    pub token_counts: Vec<u64>,
}

impl JepaTrainer {
    pub fn new(config: JepaTrainerConfig, seed: u64) -> Self {
        let model = NativeGeometricLearnerModel::new(
            config.vocab_size,
            config.num_lanes,
            seed,
            config.vsa_seed,
            config.vsa_weight,
        );
        let v_size = config.vocab_size;
        let num_lanes = config.num_lanes;
        let partition =
            VoronoiLatticePartition::build(v_size, &model.embeddings, &model.vsa_codebook);

        Self {
            adam_emb_base: AdamMoments::with_capacity(v_size * 4),
            adam_emb_comp: AdamMoments::with_capacity(v_size * 4),
            adam_tables: (0..num_lanes)
                .map(|_| AdamMoments::with_capacity(H4_ROOT_COUNT * H4_ROOT_COUNT))
                .collect(),
            adam_bias: AdamMoments::with_capacity(v_size),
            adam_readout: AdamMoments::with_capacity(v_size * 5),
            adam_jepa_state: AdamMoments::with_capacity(9),
            adam_jepa_token: AdamMoments::with_capacity(9),
            adam_jepa_bias: AdamMoments::with_capacity(3),
            adam_jepa_fiber_state: AdamMoments::with_capacity(4),
            adam_jepa_fiber_token: AdamMoments::with_capacity(4),
            adam_jepa_fiber_bias: AdamMoments::with_capacity(2),
            adam_vsa: AdamMoments::with_capacity(1),
            collocations: CollocationTracker::default(),
            continuous_lattice: None,
            model,
            config,
            step_count: 0,
            partition,
            token_counts: vec![0u64; v_size],
        }
    }

    /// Refresh precomputed Voronoi lattice partitioning and cached token-to-root mappings.
    pub fn refresh_lattice_partitioning(&mut self) {
        self.partition = VoronoiLatticePartition::build(
            self.config.vocab_size,
            &self.model.embeddings,
            &self.model.vsa_codebook,
        );
    }

    /// Initialize continuous hierarchical lattice tables and fit empirical n-gram statistics.
    pub fn init_hierarchical_lattice(&mut self, sequences: &[Vec<usize>]) {
        self.refresh_lattice_partitioning();
        let token_to_root: Vec<u8> = self
            .partition
            .token_to_root
            .iter()
            .map(|&r| r as u8)
            .collect();
        let vsa_codebook = Codebook::<64>::new(self.config.vocab_size, self.config.vsa_seed);
        let hierarchical =
            HierarchicalCodebook::new(self.config.vocab_size, &token_to_root, &vsa_codebook);
        let (token_to_cluster, num_clusters) = hierarchical.build_token_to_cluster();
        let mut continuous = ContinuousLatticeTables::new(num_clusters, token_to_cluster);
        continuous.fit_empirical_ngram_statistics(sequences, &token_to_root);
        self.continuous_lattice = Some(continuous);
    }

    /// Initialize continuous hierarchical lattice tables and fit empirical n-gram statistics from u16 sequences.
    pub fn init_hierarchical_lattice_u16(&mut self, sequences: &[&[u16]]) {
        self.refresh_lattice_partitioning();
        let token_to_root: Vec<u8> = self
            .partition
            .token_to_root
            .iter()
            .map(|&r| r as u8)
            .collect();
        let vsa_codebook = Codebook::<64>::new(self.config.vocab_size, self.config.vsa_seed);
        let hierarchical =
            HierarchicalCodebook::new(self.config.vocab_size, &token_to_root, &vsa_codebook);
        let (token_to_cluster, num_clusters) = hierarchical.build_token_to_cluster();
        let mut continuous = ContinuousLatticeTables::new(num_clusters, token_to_cluster);
        continuous.fit_empirical_ngram_statistics_u16(sequences, &token_to_root);
        self.continuous_lattice = Some(continuous);
    }

    /// Calculate current scheduled learning rate given step count, warmup, and cosine decay.
    pub fn scheduled_lr(&self) -> f64 {
        let base_lr = self.config.learning_rate;
        if self.config.total_steps == 0 {
            return base_lr;
        }
        let step = self.step_count;
        let warmup = self.config.warmup_steps as u64;
        let total = self.config.total_steps as u64;
        let min_lr = if self.config.min_lr > 0.0 {
            self.config.min_lr
        } else {
            base_lr * 0.1
        };

        if step < warmup {
            let factor = (step + 1) as f64 / (warmup.max(1) as f64);
            (min_lr + (base_lr - min_lr) * factor).max(1e-7)
        } else if step >= total {
            min_lr
        } else {
            let progress = (step - warmup) as f64 / ((total - warmup).max(1) as f64);
            let cosine = 0.5 * (1.0 + libm::cos(core::f64::consts::PI * progress));
            min_lr + (base_lr - min_lr) * cosine
        }
    }

    /// Frequency-normalized bias centering across leaf buckets for Level 2 hierarchical softmax.
    /// Prevents large buckets from driving function words to negative infinity and small buckets from exploding.
    pub fn center_emission_biases(&mut self) {
        let total_count: u64 = self.token_counts.iter().sum();
        let vocab_size = self.config.vocab_size;
        let v_f = vocab_size.max(1) as f64;
        let total_count_f = total_count.max(1) as f64;

        for bucket in &self.partition.leaf_buckets {
            if bucket.is_empty() {
                continue;
            }
            let k = bucket.len() as f64;
            let sum_b: f64 = bucket.iter().map(|&v| self.model.emission_bias[v]).sum();
            let mean_b = sum_b / k;

            // Frequency normalization: empirical probability of this bucket vs expected uniform
            let bucket_count: u64 = bucket.iter().map(|&v| self.token_counts[v]).sum();
            let p_bucket = (bucket_count as f64 + 1.0) / (total_count_f + v_f);
            let avg_p_per_tok = p_bucket / k;
            let uniform_p = 1.0 / v_f;
            let ratio = (avg_p_per_tok / uniform_p).max(1e-4);
            // Centered target offset based on log frequency ratio, bounded within [-1.0, 1.0]
            let center_offset = (0.5 * libm::log(ratio)).clamp(-1.0, 1.0);

            for &v in bucket {
                // Center around bucket mean and anchor to frequency-normalized offset
                let b = self.model.emission_bias[v] - mean_b + center_offset;
                self.model.emission_bias[v] = b.clamp(-2.0, 2.0);
            }
        }
    }

    /// Record bigram and trigram collocations from a token sequence.
    pub fn record_collocations<T: AsTokenIndex>(&mut self, tokens: &[T]) {
        self.collocations.record(tokens);
        for &tok in tokens {
            let t = tok.as_token_index().min(self.config.vocab_size - 1);
            if t < self.token_counts.len() {
                self.token_counts[t] = self.token_counts[t].saturating_add(1);
            }
        }
    }

    /// Train a single token sequence into a preallocated gradient accumulator using scratch buffers.
    ///
    /// Executes 2-level hierarchical softmax (120 coarse root logits + <= 32 leaf bucket logits = 152 total),
    /// eliminating dense full-vocabulary math and all inner-loop heap allocations.
    pub fn train_sequence_into_accumulator<T: AsTokenIndex>(
        &self,
        tokens: &[T],
        acc: &mut GradAccumulator,
        scratch: &mut TrainingScratch,
    ) {
        if tokens.len() < 2 {
            return;
        }

        let num_steps = tokens.len() - 1;
        scratch.seq_q_base.clear();
        scratch.seq_u_base.clear();

        for &tok in tokens {
            let t = tok.as_token_index().min(self.config.vocab_size - 1);
            if t < acc.token_counts.len() {
                acc.token_counts[t] = acc.token_counts[t].saturating_add(1);
            }
            let (q, _) = self.model.embeddings.forward_quaternions(t);
            let (u, _) = self.model.embeddings.forward_hopf(t);
            scratch.seq_q_base.push(q);
            scratch.seq_u_base.push(u);
        }

        let mut s3_state = UnitS3::IDENTITY;
        let roots_s2 = canonical_h4_hopf_s2();
        let fiber_roots = canonical_h4_fiber_u1();

        let mut total_ce_loss = 0.0;
        let mut total_jepa_loss = 0.0;

        for t in 0..num_steps {
            let current_token = tokens[t].as_token_index().min(self.config.vocab_size - 1);
            let next_token = tokens[t + 1]
                .as_token_index()
                .min(self.config.vocab_size - 1);

            let q_base = scratch.seq_q_base[t];
            let u_base = scratch.seq_u_base[t];

            s3_state = s3_state.mul(&q_base);
            let s2_state = s3_state.hopf_map();
            let fiber_phase = s3_state.fiber_phase();
            let u1_state = [libm::cos(fiber_phase), libm::sin(fiber_phase)];

            let target_u = scratch.seq_u_base[t + 1];
            let target_phase = scratch.seq_q_base[t + 1].fiber_phase();
            let target_u1 = [libm::cos(target_phase), libm::sin(target_phase)];

            let token_phase = q_base.fiber_phase();
            let token_u1 = [libm::cos(token_phase), libm::sin(token_phase)];

            // Closed-Loop JEPA Next-State Prediction
            let (hat_s2, hat_u1, inv_r, inv_rf) = self
                .model
                .predict_jepa_step(s2_state, u1_state, u_base, token_u1);

            let cos_sim_s2 = hat_s2.x * target_u.x + hat_s2.y * target_u.y + hat_s2.z * target_u.z;
            let jepa_s2_loss = (1.0 - cos_sim_s2).max(0.0);

            let cos_sim_f = hat_u1[0] * target_u1[0] + hat_u1[1] * target_u1[1];
            let jepa_fiber_loss = (1.0 - cos_sim_f).max(0.0);

            let step_jepa_loss = jepa_s2_loss + self.config.fiber_weight * jepa_fiber_loss;
            total_jepa_loss += step_jepa_loss;

            // Multi-Lane Transition Lookup
            let mut lane_lag_roots = [None; 8];
            for (l, _) in self.model.tables.lanes.iter().enumerate().take(8) {
                if t >= l {
                    let lag_tok = tokens[t - l]
                        .as_token_index()
                        .min(self.config.vocab_size - 1);
                    let lag_root = self.partition.token_to_root[lag_tok];
                    lane_lag_roots[l] = Some(lag_root);
                }
            }

            for r in 0..H4_ROOT_COUNT {
                scratch.lane_root_score[r] = 0.0;
            }
            for (l, lane) in self.model.tables.lanes.iter().enumerate().take(8) {
                if let Some(lag_root) = lane_lag_roots[l] {
                    for r in 0..H4_ROOT_COUNT {
                        scratch.lane_root_score[r] += lane.score_discrete(lag_root, r) * 0.1;
                    }
                }
            }

            // VSA Context Hypervector (Zero Allocations via Fixed Scratch Buffer)
            let win = self.config.context_window.min(64);
            let ctx_start = t.saturating_sub(win.saturating_sub(1));
            let ctx_len = t - ctx_start + 1;
            for (i, &tok) in tokens[ctx_start..=t].iter().enumerate() {
                scratch.ctx_buf[i] = tok.as_token_index() as u32;
            }
            let ctx_hypervector = encode_multiscale_context(
                &scratch.ctx_buf[..ctx_len],
                &self.model.vsa_codebook,
                win,
            );

            // Coarse and Fine Lattice Lookups
            for r in 0..H4_ROOT_COUNT {
                scratch.coarse_step_lookup[r] = 0.0;
            }
            if let Some(lattice) = &self.continuous_lattice {
                let prev_tok = if t >= 1 {
                    tokens[t - 1]
                        .as_token_index()
                        .min(self.config.vocab_size - 1)
                } else {
                    current_token
                };
                let r_curr = self.partition.token_to_root[current_token];
                let r_prev = self.partition.token_to_root[prev_tok];
                let c_curr = lattice.cluster_of(current_token);

                let base_coarse = (r_prev % H4_ROOT_COUNT) * 14400 + (r_curr % H4_ROOT_COUNT) * 120;
                for r in 0..H4_ROOT_COUNT {
                    scratch.coarse_step_lookup[r] = lattice.coarse_logits[base_coarse + r];
                }
                if lattice.num_clusters > 0 {
                    let base_fine = (c_curr % lattice.num_clusters) * lattice.num_clusters;
                    let max_c = lattice
                        .num_clusters
                        .min(crate::native_geometric::lattice_table::MAX_FINE_CLUSTERS);
                    for c in 0..max_c {
                        scratch.fine_step_lookup[c] = lattice.fine_logits[base_fine + c];
                    }
                }
            }

            // LEVEL 1: 120 Coarse Root Logits & Softmax
            let target_root = self.partition.token_to_root[next_token];
            for r in 0..H4_ROOT_COUNT {
                let u_r = roots_s2[r];
                let f_r = fiber_roots[r];
                let geom_root = hat_s2.x * u_r.x
                    + hat_s2.y * u_r.y
                    + hat_s2.z * u_r.z
                    + hat_u1[0] * f_r[0]
                    + hat_u1[1] * f_r[1];

                let root_vsa_sim = ctx_hypervector.bipolar_cosine(&self.partition.root_anchors[r]);
                let score = geom_root
                    + scratch.lane_root_score[r]
                    + scratch.coarse_step_lookup[r]
                    + self.model.vsa_scale * root_vsa_sim;
                scratch.root_logits[r] = score;
            }

            let mut max_root = f64::NEG_INFINITY;
            for r in 0..H4_ROOT_COUNT {
                if scratch.root_logits[r] > max_root {
                    max_root = scratch.root_logits[r];
                }
            }
            let mut sum_exp_root = 0.0;
            for r in 0..H4_ROOT_COUNT {
                let e = libm::exp(scratch.root_logits[r] - max_root);
                scratch.root_probs[r] = e;
                sum_exp_root += e;
            }
            let inv_sum_root = 1.0 / (sum_exp_root + EPSILON);
            for r in 0..H4_ROOT_COUNT {
                scratch.root_probs[r] *= inv_sum_root;
            }

            let root_prob = scratch.root_probs[target_root].max(1e-12);
            let root_ce_loss = -libm::log(root_prob);

            // LEVEL 2: Target Leaf Bucket Logits & Softmax
            let target_leaf_idx = self.partition.token_to_leaf[next_token];
            let leaf_toks = &self.partition.leaf_buckets[target_leaf_idx];
            let leaf_len = leaf_toks.len();
            let target_idx_in_leaf = self.partition.token_to_leaf_idx[next_token];

            for (i, &v) in leaf_toks.iter().enumerate() {
                let r = &self.model.s2_readout[v];
                let geom_score = hat_s2.x * r[0]
                    + hat_s2.y * r[1]
                    + hat_s2.z * r[2]
                    + hat_u1[0] * r[3]
                    + hat_u1[1] * r[4];

                let v_hyp = self.model.vsa_codebook.get(v as u32);
                let vsa_sim = ctx_hypervector.bipolar_cosine(&v_hyp);
                scratch.leaf_vsa_sim[i] = vsa_sim;

                let fine_score = if self.continuous_lattice.is_some() {
                    let c_cand = self.continuous_lattice.as_ref().unwrap().cluster_of(v);
                    let c_idx =
                        c_cand.min(crate::native_geometric::lattice_table::MAX_FINE_CLUSTERS - 1);
                    scratch.fine_step_lookup[c_idx]
                } else {
                    0.0
                };

                let score = geom_score
                    + self.model.emission_bias[v]
                    + self.model.vsa_scale * vsa_sim
                    + fine_score;
                scratch.leaf_logits[i] = score;
            }

            let mut max_leaf = f64::NEG_INFINITY;
            for i in 0..leaf_len {
                if scratch.leaf_logits[i] > max_leaf {
                    max_leaf = scratch.leaf_logits[i];
                }
            }
            let mut sum_exp_leaf = 0.0;
            for i in 0..leaf_len {
                let e = libm::exp(scratch.leaf_logits[i] - max_leaf);
                scratch.leaf_probs[i] = e;
                sum_exp_leaf += e;
            }
            let inv_sum_leaf = 1.0 / (sum_exp_leaf + EPSILON);
            for i in 0..leaf_len {
                scratch.leaf_probs[i] *= inv_sum_leaf;
            }

            let leaf_prob = scratch.leaf_probs[target_idx_in_leaf].max(1e-12);
            let leaf_ce_loss = -libm::log(leaf_prob);

            let step_ce_loss = root_ce_loss + leaf_ce_loss;
            total_ce_loss += step_ce_loss;

            // GRADIENT ACCUMULATION
            let mut d_ce_hat_s2_x = 0.0;
            let mut d_ce_hat_s2_y = 0.0;
            let mut d_ce_hat_s2_z = 0.0;
            let mut d_ce_hat_u1_0 = 0.0;
            let mut d_ce_hat_u1_1 = 0.0;

            // Level 1 Root Gradients
            for r in 0..H4_ROOT_COUNT {
                let indicator = if r == target_root { 1.0 } else { 0.0 };
                let d_root = scratch.root_probs[r] - indicator;

                // Multi-lane table gradients
                for (l, _) in self.model.tables.lanes.iter().enumerate().take(8) {
                    if let Some(lag_root) = lane_lag_roots[l] {
                        let idx = lag_root * H4_ROOT_COUNT + r;
                        acc.grad_tables[l][idx] += d_root * 0.1;
                    }
                }

                let u_r = roots_s2[r];
                let f_r = fiber_roots[r];
                d_ce_hat_s2_x += d_root * u_r.x;
                d_ce_hat_s2_y += d_root * u_r.y;
                d_ce_hat_s2_z += d_root * u_r.z;
                d_ce_hat_u1_0 += d_root * f_r[0];
                d_ce_hat_u1_1 += d_root * f_r[1];

                let root_vsa_sim = ctx_hypervector.bipolar_cosine(&self.partition.root_anchors[r]);
                acc.grad_vsa_scale += d_root * root_vsa_sim;
            }

            // Level 2 Leaf Gradients
            for (i, &v) in leaf_toks.iter().enumerate() {
                let indicator = if i == target_idx_in_leaf { 1.0 } else { 0.0 };
                let d_leaf = scratch.leaf_probs[i] - indicator;

                acc.grad_bias[v] += d_leaf;

                let r = &self.model.s2_readout[v];
                acc.grad_s2_readout[v][0] += d_leaf * hat_s2.x;
                acc.grad_s2_readout[v][1] += d_leaf * hat_s2.y;
                acc.grad_s2_readout[v][2] += d_leaf * hat_s2.z;
                acc.grad_s2_readout[v][3] += d_leaf * hat_u1[0];
                acc.grad_s2_readout[v][4] += d_leaf * hat_u1[1];

                d_ce_hat_s2_x += d_leaf * r[0];
                d_ce_hat_s2_y += d_leaf * r[1];
                d_ce_hat_s2_z += d_leaf * r[2];
                d_ce_hat_u1_0 += d_leaf * r[3];
                d_ce_hat_u1_1 += d_leaf * r[4];

                acc.grad_vsa_scale += d_leaf * scratch.leaf_vsa_sim[i];
            }

            // Closed-loop backpropagation combining JEPA loss and CE readout loss
            let dot_ce_s2 =
                d_ce_hat_s2_x * hat_s2.x + d_ce_hat_s2_y * hat_s2.y + d_ce_hat_s2_z * hat_s2.z;
            let d_ce_vx = inv_r * (d_ce_hat_s2_x - dot_ce_s2 * hat_s2.x);
            let d_ce_vy = inv_r * (d_ce_hat_s2_y - dot_ce_s2 * hat_s2.y);
            let d_ce_vz = inv_r * (d_ce_hat_s2_z - dot_ce_s2 * hat_s2.z);

            let dot_ce_f = d_ce_hat_u1_0 * hat_u1[0] + d_ce_hat_u1_1 * hat_u1[1];
            let d_ce_fx = inv_rf * (d_ce_hat_u1_0 - dot_ce_f * hat_u1[0]);
            let d_ce_fy = inv_rf * (d_ce_hat_u1_1 - dot_ce_f * hat_u1[1]);

            let scale_jepa = self.config.jepa_weight;
            let d_vx = scale_jepa * inv_r * (-target_u.x + cos_sim_s2 * hat_s2.x) + d_ce_vx;
            let d_vy = scale_jepa * inv_r * (-target_u.y + cos_sim_s2 * hat_s2.y) + d_ce_vy;
            let d_vz = scale_jepa * inv_r * (-target_u.z + cos_sim_s2 * hat_s2.z) + d_ce_vz;

            let scale_fiber = scale_jepa * self.config.fiber_weight;
            let d_fx = scale_fiber * inv_rf * (-target_u1[0] + cos_sim_f * hat_u1[0]) + d_ce_fx;
            let d_fy = scale_fiber * inv_rf * (-target_u1[1] + cos_sim_f * hat_u1[1]) + d_ce_fy;

            acc.grad_jepa_bias[0] += d_vx;
            acc.grad_jepa_bias[1] += d_vy;
            acc.grad_jepa_bias[2] += d_vz;

            acc.grad_jepa_w_state[0] += d_vx * s2_state.x;
            acc.grad_jepa_w_state[1] += d_vx * s2_state.y;
            acc.grad_jepa_w_state[2] += d_vx * s2_state.z;
            acc.grad_jepa_w_state[3] += d_vy * s2_state.x;
            acc.grad_jepa_w_state[4] += d_vy * s2_state.y;
            acc.grad_jepa_w_state[5] += d_vy * s2_state.z;
            acc.grad_jepa_w_state[6] += d_vz * s2_state.x;
            acc.grad_jepa_w_state[7] += d_vz * s2_state.y;
            acc.grad_jepa_w_state[8] += d_vz * s2_state.z;

            let wt = &self.model.jepa_w_token;
            acc.grad_jepa_w_token[0] += d_vx * u_base.x;
            acc.grad_jepa_w_token[1] += d_vx * u_base.y;
            acc.grad_jepa_w_token[2] += d_vx * u_base.z;
            acc.grad_jepa_w_token[3] += d_vy * u_base.x;
            acc.grad_jepa_w_token[4] += d_vy * u_base.y;
            acc.grad_jepa_w_token[5] += d_vy * u_base.z;
            acc.grad_jepa_w_token[6] += d_vz * u_base.x;
            acc.grad_jepa_w_token[7] += d_vz * u_base.y;
            acc.grad_jepa_w_token[8] += d_vz * u_base.z;

            acc.grad_jepa_fiber_bias[0] += d_fx;
            acc.grad_jepa_fiber_bias[1] += d_fy;

            acc.grad_jepa_fiber_w_state[0] += d_fx * u1_state[0];
            acc.grad_jepa_fiber_w_state[1] += d_fx * u1_state[1];
            acc.grad_jepa_fiber_w_state[2] += d_fy * u1_state[0];
            acc.grad_jepa_fiber_w_state[3] += d_fy * u1_state[1];

            let wft = &self.model.jepa_fiber_w_token;
            acc.grad_jepa_fiber_w_token[0] += d_fx * token_u1[0];
            acc.grad_jepa_fiber_w_token[1] += d_fx * token_u1[1];
            acc.grad_jepa_fiber_w_token[2] += d_fy * token_u1[0];
            acc.grad_jepa_fiber_w_token[3] += d_fy * token_u1[1];

            // Embeddings base S2 gradients
            let project_grad = |w: [f64; 4], g: [f64; 4]| -> [f64; 4] {
                let norm_sq = w[0] * w[0] + w[1] * w[1] + w[2] * w[2] + w[3] * w[3];
                if norm_sq < EPSILON {
                    return [0.0; 4];
                }
                let norm = libm::sqrt(norm_sq);
                let u = [w[0] / norm, w[1] / norm, w[2] / norm, w[3] / norm];
                let dot_gu = g[0] * u[0] + g[1] * u[1] + g[2] * u[2] + g[3] * u[3];
                [
                    (g[0] - dot_gu * u[0]) / norm,
                    (g[1] - dot_gu * u[1]) / norm,
                    (g[2] - dot_gu * u[2]) / norm,
                    (g[3] - dot_gu * u[3]) / norm,
                ]
            };

            let hopf_adjoint = |q: UnitS3, g: [f64; 3]| -> [f64; 4] {
                let (a, b, c, d) = (q.a, q.b, q.c, q.d);
                [
                    2.0 * (c * g[0] - d * g[1] + a * g[2]),
                    2.0 * (d * g[0] + c * g[1] + b * g[2]),
                    2.0 * (a * g[0] + b * g[1] - c * g[2]),
                    2.0 * (b * g[0] - a * g[1] - d * g[2]),
                ]
            };

            let fiber_adjoint = |q: UnitS3, g: [f64; 2]| -> [f64; 4] {
                let norm_ab_sq = q.a * q.a + q.b * q.b;
                if norm_ab_sq > EPSILON {
                    let r1 = libm::sqrt(norm_ab_sq);
                    let u0 = q.a / r1;
                    let u1 = q.b / r1;
                    let dot = g[0] * u0 + g[1] * u1;
                    [(g[0] - dot * u0) / r1, (g[1] - dot * u1) / r1, 0.0, 0.0]
                } else {
                    let norm_cd_sq = q.c * q.c + q.d * q.d;
                    let r2 = libm::sqrt(norm_cd_sq) + EPSILON;
                    let v0 = q.c / r2;
                    let v1 = q.d / r2;
                    let dot = g[0] * v0 + g[1] * v1;
                    [0.0, 0.0, (g[0] - dot * v0) / r2, (g[1] - dot * v1) / r2]
                }
            };

            let grad_target_u = [
                -scale_jepa * hat_s2.x,
                -scale_jepa * hat_s2.y,
                -scale_jepa * hat_s2.z,
            ];
            let g_target_base_s3 = hopf_adjoint(scratch.seq_q_base[t + 1], grad_target_u);
            let dw_target_base = project_grad(
                self.model.embeddings.weights_base[next_token],
                g_target_base_s3,
            );
            for c in 0..4 {
                acc.grad_emb_base[next_token][c] += dw_target_base[c];
            }

            let grad_current_u = [
                wt[0] * d_vx + wt[3] * d_vy + wt[6] * d_vz,
                wt[1] * d_vx + wt[4] * d_vy + wt[7] * d_vz,
                wt[2] * d_vx + wt[5] * d_vy + wt[8] * d_vz,
            ];
            let g_curr_base_s3 = hopf_adjoint(q_base, grad_current_u);
            let dw_curr_base = project_grad(
                self.model.embeddings.weights_base[current_token],
                g_curr_base_s3,
            );
            for c in 0..4 {
                acc.grad_emb_base[current_token][c] += dw_curr_base[c];
            }

            // Embeddings fiber U(1) gradients
            let grad_target_f_s3 = fiber_adjoint(
                scratch.seq_q_base[t + 1],
                [-scale_fiber * hat_u1[0], -scale_fiber * hat_u1[1]],
            );
            let dw_target_f = project_grad(
                self.model.embeddings.weights_base[next_token],
                grad_target_f_s3,
            );
            for c in 0..4 {
                acc.grad_emb_base[next_token][c] += dw_target_f[c];
            }

            let grad_token_u1 = [wft[0] * d_fx + wft[2] * d_fy, wft[1] * d_fx + wft[3] * d_fy];
            let grad_curr_f_s3 = fiber_adjoint(q_base, grad_token_u1);
            let dw_curr_f = project_grad(
                self.model.embeddings.weights_base[current_token],
                grad_curr_f_s3,
            );
            for c in 0..4 {
                acc.grad_emb_base[current_token][c] += dw_curr_f[c];
            }
        }

        acc.total_loss += total_ce_loss + self.config.jepa_weight * total_jepa_loss;
        acc.ce_loss += total_ce_loss;
        acc.jepa_loss += total_jepa_loss;
        acc.tokens_processed += num_steps;
    }

    /// Apply accumulated gradients from parallel or sequential execution to model parameters and lattice.
    pub fn apply_accumulated_gradients(&mut self, acc: &GradAccumulator) {
        let norm = 1.0 / (acc.tokens_processed.max(1) as f64);
        for i in 0..self.config.vocab_size {
            if i < acc.token_counts.len() {
                self.token_counts[i] =
                    self.token_counts[i].saturating_add(acc.token_counts[i] as u64);
            }
            self.model.grad_bias[i] += acc.grad_bias[i] * norm;
            for j in 0..5 {
                self.model.grad_s2_readout[i][j] += acc.grad_s2_readout[i][j] * norm;
            }
            for j in 0..4 {
                self.model.embeddings.grad_base[i][j] += acc.grad_emb_base[i][j] * norm;
                self.model.embeddings.grad_companion[i][j] += acc.grad_emb_comp[i][j] * norm;
            }
        }
        for i in 0..9 {
            self.model.grad_jepa_w_state[i] += acc.grad_jepa_w_state[i] * norm;
            self.model.grad_jepa_w_token[i] += acc.grad_jepa_w_token[i] * norm;
        }
        for i in 0..3 {
            self.model.grad_jepa_bias[i] += acc.grad_jepa_bias[i] * norm;
        }
        for i in 0..4 {
            self.model.grad_jepa_fiber_w_state[i] += acc.grad_jepa_fiber_w_state[i] * norm;
            self.model.grad_jepa_fiber_w_token[i] += acc.grad_jepa_fiber_w_token[i] * norm;
        }
        for i in 0..2 {
            self.model.grad_jepa_fiber_bias[i] += acc.grad_jepa_fiber_bias[i] * norm;
        }
        self.model.grad_vsa_scale += acc.grad_vsa_scale * norm;

        for (l, lane) in self.model.tables.lanes.iter_mut().enumerate() {
            if l < acc.grad_tables.len() {
                for i in 0..14_400 {
                    lane.grad[i] += acc.grad_tables[l][i] * norm;
                }
            }
        }
    }

    /// Teacher-forced training forward and backward pass over an input token sequence.
    pub fn train_sequence(&mut self, tokens: &[usize]) -> TrainingMetrics {
        if tokens.len() < 2 {
            return TrainingMetrics::default();
        }

        self.collocations.record(tokens);
        let mut acc = GradAccumulator::new(self.config.vocab_size, self.config.num_lanes);
        let mut scratch = TrainingScratch::new();
        self.train_sequence_into_accumulator(tokens, &mut acc, &mut scratch);

        let tokens_processed = acc.tokens_processed;
        let total_loss = if tokens_processed > 0 {
            acc.total_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let ce_loss = if tokens_processed > 0 {
            acc.ce_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let jepa_loss = if tokens_processed > 0 {
            acc.jepa_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let bits_per_byte = ce_loss / core::f64::consts::LN_2;

        self.apply_accumulated_gradients(&acc);
        self.step_adam();

        TrainingMetrics {
            total_loss,
            ce_loss,
            jepa_loss,
            bits_per_byte,
            tokens_processed,
        }
    }

    /// Parallel sequence batch training across all CPU cores using Rayon chunks.
    pub fn train_batch_parallel_u16(&mut self, sequences: &[&[u16]]) -> TrainingMetrics {
        if sequences.is_empty() {
            return TrainingMetrics::default();
        }

        let vocab_size = self.config.vocab_size;
        let num_lanes = self.config.num_lanes;

        let combined_acc = sequences
            .par_iter()
            .fold(
                || {
                    (
                        GradAccumulator::new(vocab_size, num_lanes),
                        TrainingScratch::new(),
                    )
                },
                |(mut acc, mut scratch), seq| {
                    if seq.len() >= 2 {
                        self.train_sequence_into_accumulator(*seq, &mut acc, &mut scratch);
                    }
                    (acc, scratch)
                },
            )
            .map(|(acc, _)| acc)
            .reduce(
                || GradAccumulator::new(vocab_size, num_lanes),
                |mut a, b| {
                    a.merge(&b);
                    a
                },
            );

        let tokens_processed = combined_acc.tokens_processed;
        let total_loss = if tokens_processed > 0 {
            combined_acc.total_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let ce_loss = if tokens_processed > 0 {
            combined_acc.ce_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let jepa_loss = if tokens_processed > 0 {
            combined_acc.jepa_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let bits_per_byte = ce_loss / core::f64::consts::LN_2;

        self.apply_accumulated_gradients(&combined_acc);
        self.step_adam();

        TrainingMetrics {
            total_loss,
            ce_loss,
            jepa_loss,
            bits_per_byte,
            tokens_processed,
        }
    }

    /// Parallel sequence batch training across all CPU cores using Rayon for usize token sequences.
    pub fn train_batch_parallel_usize(&mut self, sequences: &[&[usize]]) -> TrainingMetrics {
        if sequences.is_empty() {
            return TrainingMetrics::default();
        }

        let vocab_size = self.config.vocab_size;
        let num_lanes = self.config.num_lanes;

        let combined_acc = sequences
            .par_iter()
            .fold(
                || {
                    (
                        GradAccumulator::new(vocab_size, num_lanes),
                        TrainingScratch::new(),
                    )
                },
                |(mut acc, mut scratch), seq| {
                    if seq.len() >= 2 {
                        self.train_sequence_into_accumulator(*seq, &mut acc, &mut scratch);
                    }
                    (acc, scratch)
                },
            )
            .map(|(acc, _)| acc)
            .reduce(
                || GradAccumulator::new(vocab_size, num_lanes),
                |mut a, b| {
                    a.merge(&b);
                    a
                },
            );

        let tokens_processed = combined_acc.tokens_processed;
        let total_loss = if tokens_processed > 0 {
            combined_acc.total_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let ce_loss = if tokens_processed > 0 {
            combined_acc.ce_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let jepa_loss = if tokens_processed > 0 {
            combined_acc.jepa_loss / (tokens_processed as f64)
        } else {
            0.0
        };
        let bits_per_byte = ce_loss / core::f64::consts::LN_2;

        self.apply_accumulated_gradients(&combined_acc);
        self.step_adam();

        TrainingMetrics {
            total_loss,
            ce_loss,
            jepa_loss,
            bits_per_byte,
            tokens_processed,
        }
    }

    /// Evaluates bits-per-byte on a held-out evaluation sequence without updating weights.
    pub fn evaluate_bpb<T: AsTokenIndex>(&self, tokens: &[T]) -> f64 {
        let engram_table = self.collocations.build_engram_table();
        self.evaluate_bpb_with_engram(tokens, &engram_table)
    }

    /// Evaluates bits-per-byte on a held-out evaluation sequence given a pre-built engram table.
    /// Uses 2-level hierarchical softmax and scratch buffers with zero inner-loop heap allocations.
    pub fn evaluate_bpb_with_engram<T: AsTokenIndex>(
        &self,
        tokens: &[T],
        engram_table: &EngramTable,
    ) -> f64 {
        if tokens.len() < 2 {
            return 0.0;
        }

        let mut total_ce_loss = 0.0;
        let num_steps = tokens.len() - 1;
        let mut s3_state = UnitS3::IDENTITY;

        let roots_s2 = canonical_h4_hopf_s2();
        let fiber_roots = canonical_h4_fiber_u1();

        let mut q = 0_u8;
        let mut d = 0_u8;
        let is_uor = tokens
            .iter()
            .any(|&t| t.as_token_index() == 36 || t.as_token_index() > 255);

        let mut root_logits = [0.0; H4_ROOT_COUNT];
        let mut leaf_logits = [0.0; MAX_LEAF_BUCKET_SIZE];
        let mut lane_root_score = [0.0; H4_ROOT_COUNT];
        let mut coarse_step_lookup = [0.0; H4_ROOT_COUNT];
        let mut fine_step_lookup = [0.0; crate::native_geometric::lattice_table::MAX_FINE_CLUSTERS];
        let mut ctx_buf = [0u32; 64];

        for t in 0..num_steps {
            let current_token = tokens[t].as_token_index().min(self.config.vocab_size - 1);
            let next_token = tokens[t + 1]
                .as_token_index()
                .min(self.config.vocab_size - 1);

            update_syntactic_tokens(current_token, is_uor, &mut q, &mut d);
            let condition = (q & 1) | ((d & 3) << 1);

            let (q_base, _) = self.model.embeddings.forward_quaternions(current_token);
            let (u_base, _) = self.model.embeddings.forward_hopf(current_token);
            s3_state = s3_state.mul(&q_base);
            let s2_state = s3_state.hopf_map();
            let fiber_phase = s3_state.fiber_phase();
            let u1_state = [libm::cos(fiber_phase), libm::sin(fiber_phase)];

            let token_phase = q_base.fiber_phase();
            let token_u1 = [libm::cos(token_phase), libm::sin(token_phase)];

            // Forward-predict next geometric state
            let (hat_s2, hat_u1, _, _) = self
                .model
                .predict_jepa_step(s2_state, u1_state, u_base, token_u1);

            for r in 0..H4_ROOT_COUNT {
                lane_root_score[r] = 0.0;
            }
            for (l, lane) in self.model.tables.lanes.iter().enumerate().take(8) {
                if t >= l {
                    let lag_token = tokens[t - l]
                        .as_token_index()
                        .min(self.config.vocab_size - 1);
                    let lag_root = self.partition.token_to_root[lag_token];
                    for r in 0..H4_ROOT_COUNT {
                        lane_root_score[r] += lane.score_discrete(lag_root, r) * 0.1;
                    }
                }
            }

            let win = self.config.context_window.min(64);
            let ctx_start = t.saturating_sub(win.saturating_sub(1));
            let ctx_len = t - ctx_start + 1;
            for (i, &tok) in tokens[ctx_start..=t].iter().enumerate() {
                ctx_buf[i] = tok.as_token_index() as u32;
            }
            let ctx_hypervector =
                encode_multiscale_context(&ctx_buf[..ctx_len], &self.model.vsa_codebook, win);

            for r in 0..H4_ROOT_COUNT {
                coarse_step_lookup[r] = 0.0;
            }
            if let Some(lattice) = &self.continuous_lattice {
                let prev_tok = if t >= 1 {
                    tokens[t - 1].as_token_index()
                } else {
                    current_token
                };
                let r_curr = self.partition.token_to_root[current_token];
                let r_prev = self.partition.token_to_root[prev_tok.min(self.config.vocab_size - 1)];
                let c_curr = lattice.cluster_of(current_token);

                let base_coarse = (r_prev % H4_ROOT_COUNT) * 14400 + (r_curr % H4_ROOT_COUNT) * 120;
                for r in 0..H4_ROOT_COUNT {
                    coarse_step_lookup[r] = lattice.coarse_logits[base_coarse + r];
                }
                if lattice.num_clusters > 0 {
                    let base_fine = (c_curr % lattice.num_clusters) * lattice.num_clusters;
                    let max_c = lattice
                        .num_clusters
                        .min(crate::native_geometric::lattice_table::MAX_FINE_CLUSTERS);
                    for c in 0..max_c {
                        fine_step_lookup[c] = lattice.fine_logits[base_fine + c];
                    }
                }
            }

            // Level 1: Root Softmax
            let target_root = self.partition.token_to_root[next_token];
            for r in 0..H4_ROOT_COUNT {
                let u_r = roots_s2[r];
                let f_r = fiber_roots[r];
                let geom_root = hat_s2.x * u_r.x
                    + hat_s2.y * u_r.y
                    + hat_s2.z * u_r.z
                    + hat_u1[0] * f_r[0]
                    + hat_u1[1] * f_r[1];
                let root_vsa_sim = ctx_hypervector.bipolar_cosine(&self.partition.root_anchors[r]);
                root_logits[r] = geom_root
                    + lane_root_score[r]
                    + coarse_step_lookup[r]
                    + self.model.vsa_scale * root_vsa_sim;
            }

            let mut max_root = f64::NEG_INFINITY;
            for r in 0..H4_ROOT_COUNT {
                if root_logits[r] > max_root {
                    max_root = root_logits[r];
                }
            }
            let mut sum_exp_root = 0.0;
            for r in 0..H4_ROOT_COUNT {
                sum_exp_root += libm::exp(root_logits[r] - max_root);
            }
            let root_prob =
                libm::exp(root_logits[target_root] - max_root) / (sum_exp_root + EPSILON);

            // Level 2: Leaf Bucket Softmax
            let target_leaf_idx = self.partition.token_to_leaf[next_token];
            let leaf_toks = &self.partition.leaf_buckets[target_leaf_idx];
            let leaf_len = leaf_toks.len();
            let target_idx_in_leaf = self.partition.token_to_leaf_idx[next_token];

            for (i, &v) in leaf_toks.iter().enumerate() {
                let r = &self.model.s2_readout[v];
                let geom_score = hat_s2.x * r[0]
                    + hat_s2.y * r[1]
                    + hat_s2.z * r[2]
                    + hat_u1[0] * r[3]
                    + hat_u1[1] * r[4];
                let v_hyp = self.model.vsa_codebook.get(v as u32);
                let vsa_sim = ctx_hypervector.bipolar_cosine(&v_hyp);

                let fine_score = if self.continuous_lattice.is_some() {
                    let c_cand = self.continuous_lattice.as_ref().unwrap().cluster_of(v);
                    let c_idx =
                        c_cand.min(crate::native_geometric::lattice_table::MAX_FINE_CLUSTERS - 1);
                    fine_step_lookup[c_idx]
                } else {
                    0.0
                };

                let score = geom_score
                    + self.model.emission_bias[v]
                    + self.model.vsa_scale * vsa_sim
                    + fine_score;

                leaf_logits[i] = score;
            }

            let mut max_leaf = f64::NEG_INFINITY;
            for i in 0..leaf_len {
                if leaf_logits[i] > max_leaf {
                    max_leaf = leaf_logits[i];
                }
            }
            let mut sum_exp_leaf = 0.0;
            for i in 0..leaf_len {
                sum_exp_leaf += libm::exp(leaf_logits[i] - max_leaf);
            }
            let leaf_prob =
                libm::exp(leaf_logits[target_idx_in_leaf] - max_leaf) / (sum_exp_leaf + EPSILON);

            let p_geom = (root_prob * leaf_prob).max(1e-12);

            let w_curr = tokens[t].as_token_index() as u32;
            let mut engram_match: Option<(&[(u32, i16)], f64)> = None;

            // 1. 5-gram lookup (highest specificity: weight 0.85)
            if t >= 4 {
                let w_prev = tokens[t - 1].as_token_index() as u32;
                let w_prev2 = tokens[t - 2].as_token_index() as u32;
                let w_prev3 = tokens[t - 3].as_token_index() as u32;
                let w_prev4 = tokens[t - 4].as_token_index() as u32;
                if let Some(cands) =
                    engram_table.lookup_5gram(w_prev4, w_prev3, w_prev2, w_prev, w_curr)
                {
                    engram_match = Some((cands, 0.85));
                }
            }

            // 2. 4-gram lookup (weight 0.75)
            if engram_match.is_none() && t >= 3 {
                let w_prev = tokens[t - 1].as_token_index() as u32;
                let w_prev2 = tokens[t - 2].as_token_index() as u32;
                let w_prev3 = tokens[t - 3].as_token_index() as u32;
                if let Some(cands) = engram_table.lookup_4gram(w_prev3, w_prev2, w_prev, w_curr) {
                    engram_match = Some((cands, 0.75));
                }
            }

            // 3. Trigram lookup (weight 0.60)
            if engram_match.is_none() && t >= 2 {
                let w_prev = tokens[t - 1].as_token_index() as u32;
                let w_prev2 = tokens[t - 2].as_token_index() as u32;
                if let Some(cands) = engram_table.lookup_trigram(w_prev2, w_prev, w_curr) {
                    engram_match = Some((cands, 0.60));
                }
            }

            // 4. Bigram lookup (weight 0.40)
            if engram_match.is_none() && t >= 1 {
                let w_prev = tokens[t - 1].as_token_index() as u32;
                if let Some(cands) = engram_table.lookup_bigram(w_prev, w_curr) {
                    engram_match = Some((cands, 0.40));
                }
            }

            // 5. Conditioned skip-bigram lookup (weight 0.30)
            if engram_match.is_none() {
                for &k in &[2, 4, 8] {
                    if t >= k {
                        let w_skip = tokens[t - k].as_token_index() as u32;
                        if let Some(cands) = engram_table.lookup_skip(w_skip, w_curr, condition) {
                            engram_match = Some((cands, 0.30));
                            break;
                        }
                    }
                }
            }

            let p_final = if let Some((cands, lambda)) = engram_match {
                let sum_q15: f64 = cands.iter().map(|&(_, q)| (q as f64).max(1.0)).sum();
                let p_engram = cands
                    .iter()
                    .find(|&&(c, _)| (c as usize) == next_token)
                    .map(|&(_, q)| (q as f64).max(1.0) / (sum_q15 + EPSILON))
                    .unwrap_or(0.0);
                if p_engram > 0.0 {
                    (lambda * p_engram + (1.0 - lambda) * p_geom).max(1e-12)
                } else {
                    p_geom
                }
            } else {
                p_geom
            };

            let step_ce_loss = -libm::log(p_final);
            total_ce_loss += step_ce_loss;
        }

        let avg_ce = total_ce_loss / (num_steps as f64);
        avg_ce / core::f64::consts::LN_2
    }

    /// Perform an Adam parameter optimization step.
    fn step_adam(&mut self) {
        self.step_count += 1;
        let lr = self.scheduled_lr();
        let beta1 = 0.9;
        let beta2 = 0.999;
        let wd = self.config.weight_decay;
        let step = self.step_count;

        // 1. Update embedding weights
        self.adam_emb_base.update_4(
            &mut self.model.embeddings.weights_base,
            &self.model.embeddings.grad_base,
            lr,
            beta1,
            beta2,
            0.0,
            step,
        );
        self.adam_emb_comp.update_4(
            &mut self.model.embeddings.weights_companion,
            &self.model.embeddings.grad_companion,
            lr,
            beta1,
            beta2,
            0.0,
            step,
        );

        // 2. Update 120x120 lane tables
        for (l, lane) in self.model.tables.lanes.iter_mut().enumerate() {
            self.adam_tables[l].update(&mut lane.logits, &lane.grad, lr, beta1, beta2, wd, step);
        }

        // 3. Update emission bias
        self.adam_bias.update(
            &mut self.model.emission_bias,
            &self.model.grad_bias,
            lr,
            beta1,
            beta2,
            wd,
            step,
        );

        // Enforce parameter regularization, continuous bias bounds (|b_v| <= 2.0),
        // and frequency-normalized bias centering so biases never clamp to the quantization rail (+-32,767).
        self.center_emission_biases();

        // 4. Update 5D readout weights
        self.adam_readout.update_5(
            &mut self.model.s2_readout,
            &self.model.grad_s2_readout,
            lr,
            beta1,
            beta2,
            wd,
            step,
        );

        // 5. Update learned JEPA predictor weights
        self.adam_jepa_state.update(
            &mut self.model.jepa_w_state,
            &self.model.grad_jepa_w_state,
            lr,
            beta1,
            beta2,
            wd,
            step,
        );
        self.adam_jepa_token.update(
            &mut self.model.jepa_w_token,
            &self.model.grad_jepa_w_token,
            lr,
            beta1,
            beta2,
            wd,
            step,
        );
        self.adam_jepa_bias.update(
            &mut self.model.jepa_bias,
            &self.model.grad_jepa_bias,
            lr,
            beta1,
            beta2,
            wd,
            step,
        );

        self.adam_jepa_fiber_state.update(
            &mut self.model.jepa_fiber_w_state,
            &self.model.grad_jepa_fiber_w_state,
            lr,
            beta1,
            beta2,
            wd,
            step,
        );
        self.adam_jepa_fiber_token.update(
            &mut self.model.jepa_fiber_w_token,
            &self.model.grad_jepa_fiber_w_token,
            lr,
            beta1,
            beta2,
            wd,
            step,
        );
        self.adam_jepa_fiber_bias.update(
            &mut self.model.jepa_fiber_bias,
            &self.model.grad_jepa_fiber_bias,
            lr,
            beta1,
            beta2,
            wd,
            step,
        );

        // Continuous bias bounds on JEPA biases
        for b in self.model.jepa_bias.iter_mut() {
            *b = (*b).clamp(-2.0, 2.0);
        }
        for b in self.model.jepa_fiber_bias.iter_mut() {
            *b = (*b).clamp(-2.0, 2.0);
        }

        // 6. Update VSA context scale
        let mut vsa_p = [self.model.vsa_scale];
        let vsa_g = [self.model.grad_vsa_scale];
        self.adam_vsa
            .update(&mut vsa_p, &vsa_g, lr, beta1, beta2, wd, step);
        self.model.vsa_scale = vsa_p[0];

        // 7. Update continuous hierarchical lattice tables
        if let Some(lattice) = &mut self.continuous_lattice {
            lattice.step_adam(lr, beta1, beta2, wd, step);
        }

        // 8. Zero gradients to prevent unbounded accumulation across batches
        self.model.zero_grad();
    }

    /// Export the trained continuous model to discrete tables for serving.
    pub fn export_discrete(&self) -> ExportedGeometricModel {
        let mut token_to_root = Vec::with_capacity(self.config.vocab_size);
        for token in 0..self.config.vocab_size {
            let root = self.model.embeddings.nearest_h4_root(token);
            token_to_root.push(root as u8);
        }

        let discrete_tables = self.model.tables.quantize();

        // Quantize bias and 5D readout (clamped to [-2.0, 2.0] so biases never clamp to rail)
        let discrete_bias = self
            .model
            .emission_bias
            .iter()
            .map(|&b| {
                (b.clamp(-2.0, 2.0) * 8192.0)
                    .clamp(-32767.0, 32767.0)
                    .round() as i32
            })
            .collect();

        let discrete_s2_readout = self
            .model
            .s2_readout
            .iter()
            .map(|r| {
                [
                    (r[0] * 16384.0).clamp(-32767.0, 32767.0).round() as i16,
                    (r[1] * 16384.0).clamp(-32767.0, 32767.0).round() as i16,
                    (r[2] * 16384.0).clamp(-32767.0, 32767.0).round() as i16,
                    (r[3] * 16384.0).clamp(-32767.0, 32767.0).round() as i16,
                    (r[4] * 16384.0).clamp(-32767.0, 32767.0).round() as i16,
                ]
            })
            .collect();

        let quantize_16 = |arr: &[f64]| -> Vec<i16> {
            arr.iter()
                .map(|&v| (v * 16384.0).clamp(-32767.0, 32767.0).round() as i16)
                .collect()
        };
        let mut discrete_jepa_w_state = [0i16; 9];
        for (i, &v) in quantize_16(&self.model.jepa_w_state).iter().enumerate() {
            discrete_jepa_w_state[i] = v;
        }
        let mut discrete_jepa_w_token = [0i16; 9];
        for (i, &v) in quantize_16(&self.model.jepa_w_token).iter().enumerate() {
            discrete_jepa_w_token[i] = v;
        }
        let mut discrete_jepa_bias = [0i16; 3];
        for (i, &v) in quantize_16(&self.model.jepa_bias).iter().enumerate() {
            discrete_jepa_bias[i] = v;
        }
        let mut discrete_jepa_fiber_w_state = [0i16; 4];
        for (i, &v) in quantize_16(&self.model.jepa_fiber_w_state)
            .iter()
            .enumerate()
        {
            discrete_jepa_fiber_w_state[i] = v;
        }
        let mut discrete_jepa_fiber_w_token = [0i16; 4];
        for (i, &v) in quantize_16(&self.model.jepa_fiber_w_token)
            .iter()
            .enumerate()
        {
            discrete_jepa_fiber_w_token[i] = v;
        }
        let mut discrete_jepa_fiber_bias = [0i16; 2];
        for (i, &v) in quantize_16(&self.model.jepa_fiber_bias).iter().enumerate() {
            discrete_jepa_fiber_bias[i] = v;
        }

        let vsa_scale_q15 = (self.model.vsa_scale * 16384.0)
            .clamp(-32767.0, 32767.0)
            .round() as i16;

        let vsa_codebook = Codebook::<64>::new(self.config.vocab_size, self.config.vsa_seed);
        let hierarchical =
            HierarchicalCodebook::new(self.config.vocab_size, &token_to_root, &vsa_codebook);
        let engram_table = self.collocations.build_engram_table();
        let hierarchical_lattice = self.continuous_lattice.as_ref().map(|l| l.quantize());

        ExportedGeometricModel {
            vocab_size: self.config.vocab_size,
            token_to_root,
            discrete_tables,
            discrete_bias,
            discrete_s2_readout,
            discrete_jepa_w_state,
            discrete_jepa_w_token,
            discrete_jepa_bias,
            discrete_jepa_fiber_w_state,
            discrete_jepa_fiber_w_token,
            discrete_jepa_fiber_bias,
            vsa_seed: self.config.vsa_seed,
            vsa_scale_q15,
            hierarchical_codebook: Some(hierarchical),
            engram_table: Some(engram_table),
            hierarchical_lattice,
        }
    }
}
