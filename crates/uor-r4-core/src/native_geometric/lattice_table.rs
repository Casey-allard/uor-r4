//! 2-Level Hierarchical Lattice Tables ($120^3 + 480^2$, L2 Cache Resident).
//!
//! Provides two-tier quantized conditional scoring on serving hot paths:
//! 1. Coarse Level: 120^3 (1,728,000 entries) root trigram table in i8 format (~1.728 MB).
//!    Captures canonical H4 600-cell root transitions across 3-step horizons.
//! 2. Fine Level: Up to 480^2 (230,400 entries) leaf cluster bigram residual in i16 format (~460.8 KB).
//!    Captures fine-grained lexical transitions between leaf buckets within sectors.
//! 3. Total serving memory footprint is ~2.19 MB, fitting comfortably in modern CPU L2/L3 cache.
//! 4. Combined score evaluation: `(coarse as i32) << 6 + (fine as i32)` in fixed-point Q-format,
//!    executing with zero runtime floats, zero matrix multiplications, and zero heap allocations.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Number of canonical H4 roots of the 600-cell on S3.
pub const COARSE_ROOT_COUNT: usize = 120;

/// Number of entries in the 3-step coarse root transition table: 120^3 = 1,728,000.
pub const COARSE_TABLE_SIZE: usize = COARSE_ROOT_COUNT * COARSE_ROOT_COUNT * COARSE_ROOT_COUNT;

/// Maximum number of fine clusters (leaf buckets across all sectors).
pub const MAX_FINE_CLUSTERS: usize = 480;

/// Quantization scale for coarse trigram table (i8): 1.0 logit -> 32.
pub const COARSE_SCALE: f64 = 32.0;

/// Quantization scale for fine residual bigram table (i16): 1.0 logit -> 2048.
pub const FINE_SCALE: f64 = 2048.0;

/// Bit shift to align coarse score (scale 32) with fine score (scale 2048): 32 * (1 << 6) = 2048.
pub const COMBINE_SHIFT: u32 = 6;

/// Number of fractional bits corresponding to FINE_SCALE = 2048 (2^11).
pub const FINE_SCALE_LOG2_BITS: u32 = 11;

/// Information-theoretic self-transition surprisal deficit for canonical |V| = 4096 vocabulary:
/// \Delta I = -\log_2(4096) \times FINE_SCALE = -12.0 \times 2048 = -24,576.
/// Mathematically decouples cluster-level transition density from identical word repetition
/// while preserving the underlying coarse root trigram geometry (coarse << COMBINE_SHIFT).
pub const SELF_TRANSITION_RESIDUAL: i32 = -24576;

/// Compute the information-theoretic self-transition surprisal deficit:
/// \Delta I(V) = -\log_2(V) \times FINE_SCALE
///
/// For vocabulary size V = 4096 and FINE_SCALE = 2048.0, \Delta I(4096) = -12.0 * 2048 = -24,576.
/// Computed using pure integer bitwise fixed-point arithmetic with 0 runtime floats,
/// 0 matrix multiplications, and 0 steady-state heap allocations.
#[inline]
pub fn self_transition_surprisal(vocab_size: usize) -> i32 {
    if vocab_size <= 1 {
        return 0;
    }
    let v = vocab_size as u64;
    let k = 63 - v.leading_zeros(); // integer floor(log2(v))

    if v.is_power_of_two() {
        let scaled = (k as i32) << FINE_SCALE_LOG2_BITS;
        return -scaled;
    }

    // Fixed-point extraction of fractional log2 mantissa in Q1.62:
    let mut m = v << (62 - k);
    let mut frac = 0i32;

    for i in (0..FINE_SCALE_LOG2_BITS).rev() {
        m = ((m as u128 * m as u128) >> 62) as u64;
        if m >= (1u64 << 63) {
            frac |= 1 << i;
            m >>= 1;
        }
    }

    // 12th fractional bit for nearest-integer rounding:
    let round_m = ((m as u128 * m as u128) >> 62) as u64;
    if round_m >= (1u64 << 63) {
        frac += 1;
    }

    let total = ((k as i32) << FINE_SCALE_LOG2_BITS) + frac;
    -total
}

/// Quantized, L2-cache-resident 2-level hierarchical lattice tables for discrete serving.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchicalLatticeTables {
    /// Number of active fine clusters (leaf buckets).
    pub num_clusters: usize,
    /// Mapping from token ID to fine cluster index [0..num_clusters).
    pub token_to_cluster: Vec<u16>,
    /// Coarse root trigram table: 120^3 = 1,728,000 entries in i8 (~1.728 MB).
    pub coarse_trigram: Vec<i8>,
    /// Fine cluster bigram residual table: num_clusters^2 entries in i16 (<= 460.8 KB).
    pub fine_residual: Vec<i16>,
}

impl HierarchicalLatticeTables {
    /// Construct a new hierarchical lattice table container.
    pub fn new(
        num_clusters: usize,
        token_to_cluster: Vec<u16>,
        coarse_trigram: Vec<i8>,
        fine_residual: Vec<i16>,
    ) -> Self {
        Self {
            num_clusters,
            token_to_cluster,
            coarse_trigram,
            fine_residual,
        }
    }

    /// Construct an empty table container.
    pub fn empty() -> Self {
        Self {
            num_clusters: 0,
            token_to_cluster: Vec::new(),
            coarse_trigram: Vec::new(),
            fine_residual: Vec::new(),
        }
    }

    /// O(1) lookup of fine cluster index for a discrete token.
    #[inline]
    pub fn cluster_of(&self, token: usize) -> usize {
        self.token_to_cluster.get(token).copied().unwrap_or(0) as usize
    }

    /// O(1) lookup of coarse root trigram score in i8 format (scale 32).
    #[inline]
    pub fn coarse_score(&self, r_prev: usize, r_curr: usize, r_cand: usize) -> i8 {
        let r0 = r_prev % COARSE_ROOT_COUNT;
        let r1 = r_curr % COARSE_ROOT_COUNT;
        let r2 = r_cand % COARSE_ROOT_COUNT;
        let idx = r0 * 14400 + r1 * 120 + r2;
        if idx < self.coarse_trigram.len() {
            self.coarse_trigram[idx]
        } else {
            0
        }
    }

    /// O(1) lookup of fine cluster bigram residual score in i16 format (scale 2048).
    #[inline]
    pub fn fine_score(&self, c_curr: usize, c_cand: usize) -> i16 {
        if self.num_clusters == 0 {
            return 0;
        }
        let c0 = c_curr % self.num_clusters;
        let c1 = c_cand % self.num_clusters;
        let idx = c0 * self.num_clusters + c1;
        if idx < self.fine_residual.len() {
            self.fine_residual[idx]
        } else {
            0
        }
    }

    /// O(1) combined hierarchical scoring in fixed-point format (scale 2048) with zero heap allocations.
    ///
    /// Combines coarse trigram and fine bigram residuals via bit shift: `(coarse << 6) + fine`.
    #[inline]
    pub fn score(
        &self,
        r_prev: usize,
        r_curr: usize,
        r_cand: usize,
        c_curr: usize,
        c_cand: usize,
    ) -> i32 {
        let coarse = self.coarse_score(r_prev, r_curr, r_cand) as i32;
        let fine = self.fine_score(c_curr, c_cand) as i32;
        (coarse << COMBINE_SHIFT) + fine
    }

    /// Dynamically computed information-theoretic self-transition surprisal deficit for this table:
    /// \Delta I(V) = -\log_2(V) \times FINE_SCALE
    #[inline]
    pub fn self_transition_residual(&self) -> i32 {
        self_transition_surprisal(self.token_to_cluster.len())
    }

    /// O(1) combined hierarchical scoring with exact token self-transition distinction.
    ///
    /// The fine residual table captures transitions between leaf clusters (C_1 -> C_2).
    /// For tokens within the same cluster (c_curr == c_cand), this represents transitions
    /// between distinct lexical items (t_1 != t_2 in C). When is_self_transition is true
    /// (exact identical token repetition t_cand == t_curr), the fine intra-cluster bonus
    /// is replaced by the dynamically computed self-transition surprisal deficit
    /// \Delta I(V) = -\log_2(V) \times FINE_SCALE, decoupling cluster-level transition density
    /// from identical word repetition while strictly preserving coarse H4 root trigram geometry.
    #[inline]
    pub fn score_token(
        &self,
        r_prev: usize,
        r_curr: usize,
        r_cand: usize,
        c_curr: usize,
        c_cand: usize,
        is_self_transition: bool,
    ) -> i32 {
        let coarse = self.coarse_score(r_prev, r_curr, r_cand) as i32;
        let fine = if is_self_transition {
            self.self_transition_residual()
        } else {
            self.fine_score(c_curr, c_cand) as i32
        };
        (coarse << COMBINE_SHIFT) + fine
    }

    /// Total memory footprint of serving tables in bytes.
    pub fn memory_bytes(&self) -> usize {
        self.token_to_cluster.len() * core::mem::size_of::<u16>()
            + self.coarse_trigram.len() * core::mem::size_of::<i8>()
            + self.fine_residual.len() * core::mem::size_of::<i16>()
    }
}

impl Default for HierarchicalLatticeTables {
    fn default() -> Self {
        Self::empty()
    }
}

/// Differentiable continuous 2-level hierarchical lattice tables used during training.
#[derive(Clone, Debug)]
pub struct ContinuousLatticeTables {
    /// Number of active fine clusters.
    pub num_clusters: usize,
    /// Token to cluster index mapping.
    pub token_to_cluster: Vec<u16>,
    /// Coarse root trigram continuous logits (1,728,000 floats).
    pub coarse_logits: Vec<f64>,
    /// Coarse root trigram gradients.
    pub coarse_grad: Vec<f64>,
    /// Adam first moment for coarse logits.
    pub coarse_m: Vec<f64>,
    /// Adam second moment for coarse logits.
    pub coarse_v: Vec<f64>,
    /// Sparse active coarse index tracker for fast Adam updates.
    pub active_coarse: Vec<usize>,
    /// Fine cluster bigram residual continuous logits (num_clusters^2 floats).
    pub fine_logits: Vec<f64>,
    /// Fine cluster bigram residual gradients.
    pub fine_grad: Vec<f64>,
    /// Adam first moment for fine logits.
    pub fine_m: Vec<f64>,
    /// Adam second moment for fine logits.
    pub fine_v: Vec<f64>,
    /// Sparse active fine index tracker for fast Adam updates.
    pub active_fine: Vec<usize>,
}

impl ContinuousLatticeTables {
    /// Create new continuous lattice tables for training.
    pub fn new(num_clusters: usize, token_to_cluster: Vec<u16>) -> Self {
        let fine_size = num_clusters * num_clusters;
        Self {
            num_clusters,
            token_to_cluster,
            coarse_logits: vec![0.0; COARSE_TABLE_SIZE],
            coarse_grad: vec![0.0; COARSE_TABLE_SIZE],
            coarse_m: vec![0.0; COARSE_TABLE_SIZE],
            coarse_v: vec![0.0; COARSE_TABLE_SIZE],
            active_coarse: Vec::with_capacity(1024),
            fine_logits: vec![0.0; fine_size],
            fine_grad: vec![0.0; fine_size],
            fine_m: vec![0.0; fine_size],
            fine_v: vec![0.0; fine_size],
            active_fine: Vec::with_capacity(1024),
        }
    }

    /// Lookup cluster for token.
    #[inline]
    pub fn cluster_of(&self, token: usize) -> usize {
        self.token_to_cluster.get(token).copied().unwrap_or(0) as usize
    }

    /// Continuous score lookup for a candidate: `coarse + fine`.
    #[inline]
    pub fn score_continuous(
        &self,
        r_prev: usize,
        r_curr: usize,
        r_cand: usize,
        c_curr: usize,
        c_cand: usize,
    ) -> f64 {
        let r0 = r_prev % COARSE_ROOT_COUNT;
        let r1 = r_curr % COARSE_ROOT_COUNT;
        let r2 = r_cand % COARSE_ROOT_COUNT;
        let c_idx = r0 * 14400 + r1 * 120 + r2;
        let coarse = self.coarse_logits.get(c_idx).copied().unwrap_or(0.0);

        let fine = if self.num_clusters > 0 {
            let f0 = c_curr % self.num_clusters;
            let f1 = c_cand % self.num_clusters;
            let f_idx = f0 * self.num_clusters + f1;
            self.fine_logits.get(f_idx).copied().unwrap_or(0.0)
        } else {
            0.0
        };

        coarse + fine
    }

    /// Backpropagate loss gradient to coarse and fine logits.
    #[inline]
    pub fn backward(
        &mut self,
        r_prev: usize,
        r_curr: usize,
        r_cand: usize,
        c_curr: usize,
        c_cand: usize,
        d_logit: f64,
    ) {
        let r0 = r_prev % COARSE_ROOT_COUNT;
        let r1 = r_curr % COARSE_ROOT_COUNT;
        let r2 = r_cand % COARSE_ROOT_COUNT;
        let c_idx = r0 * 14400 + r1 * 120 + r2;
        if c_idx < self.coarse_grad.len() {
            if self.coarse_grad[c_idx] == 0.0 {
                self.active_coarse.push(c_idx);
            }
            self.coarse_grad[c_idx] += d_logit;
        }

        if self.num_clusters > 0 {
            let f0 = c_curr % self.num_clusters;
            let f1 = c_cand % self.num_clusters;
            let f_idx = f0 * self.num_clusters + f1;
            if f_idx < self.fine_grad.len() {
                if self.fine_grad[f_idx] == 0.0 {
                    self.active_fine.push(f_idx);
                }
                self.fine_grad[f_idx] += d_logit;
            }
        }
    }

    /// Backpropagate loss gradient to coarse logit only.
    #[inline]
    pub fn backward_coarse(&mut self, r_prev: usize, r_curr: usize, r_cand: usize, d_logit: f64) {
        let r0 = r_prev % COARSE_ROOT_COUNT;
        let r1 = r_curr % COARSE_ROOT_COUNT;
        let r2 = r_cand % COARSE_ROOT_COUNT;
        let c_idx = r0 * 14400 + r1 * 120 + r2;
        if c_idx < self.coarse_grad.len() {
            if self.coarse_grad[c_idx] == 0.0 {
                self.active_coarse.push(c_idx);
            }
            self.coarse_grad[c_idx] += d_logit;
        }
    }

    /// Backpropagate loss gradient to fine logit only.
    #[inline]
    pub fn backward_fine(&mut self, c_curr: usize, c_cand: usize, d_logit: f64) {
        if self.num_clusters > 0 {
            let f0 = c_curr % self.num_clusters;
            let f1 = c_cand % self.num_clusters;
            let f_idx = f0 * self.num_clusters + f1;
            if f_idx < self.fine_grad.len() {
                if self.fine_grad[f_idx] == 0.0 {
                    self.active_fine.push(f_idx);
                }
                self.fine_grad[f_idx] += d_logit;
            }
        }
    }

    /// Reset all active gradients to zero.
    pub fn zero_grad(&mut self) {
        for &idx in &self.active_coarse {
            self.coarse_grad[idx] = 0.0;
        }
        self.active_coarse.clear();
        for &idx in &self.active_fine {
            self.fine_grad[idx] = 0.0;
        }
        self.active_fine.clear();
    }

    /// Step Adam optimizer on active coarse and fine parameters.
    pub fn step_adam(&mut self, lr: f64, beta1: f64, beta2: f64, wd: f64, step: u64) {
        if step == 0 {
            return;
        }
        let step_f = step as f64;
        let bias_correction1 = 1.0 - libm::pow(beta1, step_f);
        let bias_correction2 = 1.0 - libm::pow(beta2, step_f);
        let eps = 1e-8;

        self.active_coarse.sort_unstable();
        self.active_coarse.dedup();
        for &idx in &self.active_coarse {
            let grad = self.coarse_grad[idx].clamp(-1.0, 1.0);
            if grad == 0.0 && self.coarse_m[idx] == 0.0 && self.coarse_v[idx] == 0.0 {
                continue;
            }
            self.coarse_m[idx] = beta1 * self.coarse_m[idx] + (1.0 - beta1) * grad;
            self.coarse_v[idx] = beta2 * self.coarse_v[idx] + (1.0 - beta2) * grad * grad;

            let m_hat = self.coarse_m[idx] / bias_correction1;
            let v_hat = self.coarse_v[idx] / bias_correction2;
            let update = lr * m_hat / (libm::sqrt(v_hat) + eps);

            let param = &mut self.coarse_logits[idx];
            if wd != 0.0 {
                *param -= lr * wd * (*param);
            }
            *param -= update;
            self.coarse_grad[idx] = 0.0;
        }
        self.active_coarse.clear();

        self.active_fine.sort_unstable();
        self.active_fine.dedup();
        for &idx in &self.active_fine {
            let grad = self.fine_grad[idx].clamp(-1.0, 1.0);
            if grad == 0.0 && self.fine_m[idx] == 0.0 && self.fine_v[idx] == 0.0 {
                continue;
            }
            self.fine_m[idx] = beta1 * self.fine_m[idx] + (1.0 - beta1) * grad;
            self.fine_v[idx] = beta2 * self.fine_v[idx] + (1.0 - beta2) * grad * grad;

            let m_hat = self.fine_m[idx] / bias_correction1;
            let v_hat = self.fine_v[idx] / bias_correction2;
            let update = lr * m_hat / (libm::sqrt(v_hat) + eps);

            let param = &mut self.fine_logits[idx];
            if wd != 0.0 {
                *param -= lr * wd * (*param);
            }
            *param -= update;
            self.fine_grad[idx] = 0.0;
        }
        self.active_fine.clear();
    }

    /// Fit empirical Laplace-smoothed n-gram statistics from training sequences.
    ///
    /// Computes centered log-odds for:
    /// 1. Coarse H4 root trigrams: `P(r_3 | r_1, r_2)`.
    /// 2. Fine cluster bigrams: `P(c_2 | c_1)`.
    pub fn fit_empirical_ngram_statistics(
        &mut self,
        sequences: &[Vec<usize>],
        token_to_root: &[u8],
    ) {
        if sequences.is_empty() || token_to_root.is_empty() {
            return;
        }
        let vocab_size = token_to_root.len();

        // 1. Root Trigram Statistics
        let mut c_tri = vec![0u32; COARSE_TABLE_SIZE];
        let mut c_bi = vec![0u32; COARSE_ROOT_COUNT * COARSE_ROOT_COUNT];

        for seq in sequences {
            if seq.is_empty() {
                continue;
            }
            if seq.len() == 1 {
                let t0 = seq[0].min(vocab_size - 1);
                let r0 = token_to_root[t0] as usize % COARSE_ROOT_COUNT;
                c_bi[r0 * COARSE_ROOT_COUNT + r0] =
                    c_bi[r0 * COARSE_ROOT_COUNT + r0].saturating_add(1);
                c_tri[r0 * 14400 + r0 * 120 + r0] =
                    c_tri[r0 * 14400 + r0 * 120 + r0].saturating_add(1);
                continue;
            }
            for i in 1..seq.len() {
                let t_curr = seq[i].min(vocab_size - 1);
                let t_prev = seq[i - 1].min(vocab_size - 1);
                let r_curr = token_to_root[t_curr] as usize % COARSE_ROOT_COUNT;
                let r_prev = token_to_root[t_prev] as usize % COARSE_ROOT_COUNT;
                let r_prev2 = if i >= 2 {
                    let t_prev2 = seq[i - 2].min(vocab_size - 1);
                    token_to_root[t_prev2] as usize % COARSE_ROOT_COUNT
                } else {
                    r_prev
                };
                let bi_idx = r_prev2 * COARSE_ROOT_COUNT + r_prev;
                let tri_idx = r_prev2 * 14400 + r_prev * 120 + r_curr;
                c_bi[bi_idx] = c_bi[bi_idx].saturating_add(1);
                c_tri[tri_idx] = c_tri[tri_idx].saturating_add(1);
            }
        }

        let alpha = 0.5;
        let coarse_count_f = COARSE_ROOT_COUNT as f64;
        for r0 in 0..COARSE_ROOT_COUNT {
            for r1 in 0..COARSE_ROOT_COUNT {
                let bi_idx = r0 * COARSE_ROOT_COUNT + r1;
                let c_context = c_bi[bi_idx] as f64;
                if c_context > 0.0 {
                    let denom = c_context + alpha * coarse_count_f;
                    for r2 in 0..COARSE_ROOT_COUNT {
                        let tri_idx = r0 * 14400 + r1 * 120 + r2;
                        let count = c_tri[tri_idx] as f64;
                        let prob = (count + alpha) / denom;
                        let logit = libm::log(prob * coarse_count_f);
                        self.coarse_logits[tri_idx] = logit.clamp(-3.0, 3.0);
                    }
                }
            }
        }

        // 2. Fine Cluster Bigram Statistics
        let k = self.num_clusters;
        if k > 1 {
            let mut c_fine_bi = vec![0u32; k * k];
            let mut c_fine_uni = vec![0u32; k];

            for seq in sequences {
                if seq.len() < 2 {
                    continue;
                }
                for i in 1..seq.len() {
                    let t_prev = seq[i - 1].min(vocab_size - 1);
                    let t_curr = seq[i].min(vocab_size - 1);
                    let c_prev = self.cluster_of(t_prev) % k;
                    let c_curr = self.cluster_of(t_curr) % k;
                    c_fine_uni[c_prev] = c_fine_uni[c_prev].saturating_add(1);
                    c_fine_bi[c_prev * k + c_curr] =
                        c_fine_bi[c_prev * k + c_curr].saturating_add(1);
                }
            }

            let beta = 0.5;
            let k_f = k as f64;
            for c0 in 0..k {
                let c_context = c_fine_uni[c0] as f64;
                if c_context > 0.0 {
                    let denom = c_context + beta * k_f;
                    for c1 in 0..k {
                        let count = c_fine_bi[c0 * k + c1] as f64;
                        let prob = (count + beta) / denom;
                        let logit = libm::log(prob * k_f);
                        self.fine_logits[c0 * k + c1] = logit.clamp(-3.0, 3.0);
                    }
                }
            }
        }
    }

    /// Computes centered log-odds for u16 sequences:
    /// 1. Coarse H4 root trigrams: `P(r_3 | r_1, r_2)`.
    /// 2. Fine cluster bigrams: `P(c_2 | c_1)`.
    pub fn fit_empirical_ngram_statistics_u16(
        &mut self,
        sequences: &[&[u16]],
        token_to_root: &[u8],
    ) {
        if sequences.is_empty() || token_to_root.is_empty() {
            return;
        }
        let vocab_size = token_to_root.len();

        // 1. Root Trigram Statistics
        let mut c_tri = vec![0u32; COARSE_TABLE_SIZE];
        let mut c_bi = vec![0u32; COARSE_ROOT_COUNT * COARSE_ROOT_COUNT];

        for &seq in sequences {
            if seq.is_empty() {
                continue;
            }
            if seq.len() == 1 {
                let t0 = (seq[0] as usize).min(vocab_size - 1);
                let r0 = token_to_root[t0] as usize % COARSE_ROOT_COUNT;
                c_bi[r0 * COARSE_ROOT_COUNT + r0] =
                    c_bi[r0 * COARSE_ROOT_COUNT + r0].saturating_add(1);
                c_tri[r0 * 14400 + r0 * 120 + r0] =
                    c_tri[r0 * 14400 + r0 * 120 + r0].saturating_add(1);
                continue;
            }
            for i in 1..seq.len() {
                let t_curr = (seq[i] as usize).min(vocab_size - 1);
                let t_prev = (seq[i - 1] as usize).min(vocab_size - 1);
                let r_curr = token_to_root[t_curr] as usize % COARSE_ROOT_COUNT;
                let r_prev = token_to_root[t_prev] as usize % COARSE_ROOT_COUNT;
                let r_prev2 = if i >= 2 {
                    let t_prev2 = (seq[i - 2] as usize).min(vocab_size - 1);
                    token_to_root[t_prev2] as usize % COARSE_ROOT_COUNT
                } else {
                    r_prev
                };
                let bi_idx = r_prev2 * COARSE_ROOT_COUNT + r_prev;
                let tri_idx = r_prev2 * 14400 + r_prev * 120 + r_curr;
                c_bi[bi_idx] = c_bi[bi_idx].saturating_add(1);
                c_tri[tri_idx] = c_tri[tri_idx].saturating_add(1);
            }
        }

        let alpha = 0.5;
        let coarse_count_f = COARSE_ROOT_COUNT as f64;
        for r0 in 0..COARSE_ROOT_COUNT {
            for r1 in 0..COARSE_ROOT_COUNT {
                let bi_idx = r0 * COARSE_ROOT_COUNT + r1;
                let c_context = c_bi[bi_idx] as f64;
                if c_context > 0.0 {
                    let denom = c_context + alpha * coarse_count_f;
                    for r2 in 0..COARSE_ROOT_COUNT {
                        let tri_idx = r0 * 14400 + r1 * 120 + r2;
                        let count = c_tri[tri_idx] as f64;
                        let prob = (count + alpha) / denom;
                        let logit = libm::log(prob * coarse_count_f);
                        self.coarse_logits[tri_idx] = logit.clamp(-3.0, 3.0);
                    }
                }
            }
        }

        // 2. Fine Cluster Bigram Statistics
        let k = self.num_clusters;
        if k > 1 {
            let mut c_fine_bi = vec![0u32; k * k];
            let mut c_fine_uni = vec![0u32; k];

            for &seq in sequences {
                if seq.len() < 2 {
                    continue;
                }
                for i in 1..seq.len() {
                    let t_prev = (seq[i - 1] as usize).min(vocab_size - 1);
                    let t_curr = (seq[i] as usize).min(vocab_size - 1);
                    let c_prev = self.cluster_of(t_prev) % k;
                    let c_curr = self.cluster_of(t_curr) % k;
                    c_fine_uni[c_prev] = c_fine_uni[c_prev].saturating_add(1);
                    c_fine_bi[c_prev * k + c_curr] =
                        c_fine_bi[c_prev * k + c_curr].saturating_add(1);
                }
            }

            let beta = 0.5;
            let k_f = k as f64;
            for c0 in 0..k {
                let c_context = c_fine_uni[c0] as f64;
                if c_context > 0.0 {
                    let denom = c_context + beta * k_f;
                    for c1 in 0..k {
                        let count = c_fine_bi[c0 * k + c1] as f64;
                        let prob = (count + beta) / denom;
                        let logit = libm::log(prob * k_f);
                        self.fine_logits[c0 * k + c1] = logit.clamp(-3.0, 3.0);
                    }
                }
            }
        }
    }

    /// Quantize continuous lattice tables to fixed-point `HierarchicalLatticeTables`.
    pub fn quantize(&self) -> HierarchicalLatticeTables {
        let coarse_trigram = self
            .coarse_logits
            .iter()
            .map(|&x| (x * COARSE_SCALE).clamp(-127.0, 127.0).round() as i8)
            .collect();
        let fine_residual = self
            .fine_logits
            .iter()
            .map(|&x| (x * FINE_SCALE).clamp(-32767.0, 32767.0).round() as i16)
            .collect();

        HierarchicalLatticeTables {
            num_clusters: self.num_clusters,
            token_to_cluster: self.token_to_cluster.clone(),
            coarse_trigram,
            fine_residual,
        }
    }
}
