//! A low-bit recurrent core: the learned composition D0-b unlocks.
//!
//! # What this is
//!
//! A minimal autoregressive core that can actually be *trained* and then served with no multiplier:
//!
//! ```text
//!   h_t = relu( W_x[token_t] + W_h · h_{t-1} )      integer, ternaries, relu = max(0, .)
//!   logits = W_o · h_t                               integer, ternaries
//! ```
//!
//! Every matrix is a [`TernaryLinear`]: ternary weights at 2 bits with a power-of-two per-row
//! scale, so each layer is adds, subtracts and shifts. `relu` is a `max` against zero. There is no
//! floating point, no multiplier and no transcendental on the serving path, and the exactness of
//! the integer path against its floating reference is verified by test rather than asserted.
//!
//! # Why a recurrence, and why these three matrices
//!
//! A fixed-window feed-forward map cannot express "what came earlier", and the project has already
//! paid for that mistake: an additive scorer over count features is why the current model emits
//! repetitive fragments. A recurrent state is the smallest structure that can carry context
//! forward, and `W_h` is where the learned transition lives. `W_x` is the learned embedding —
//! replacing the `prime % 120` hash, which was a fixed function of the token id and therefore
//! carried no learned information at all. `W_o` is the learned readout.
//!
//! That is a genuine change in kind from the current model, not more of it: there is now a learned
//! function from context to a distribution, rather than a sum of count lookups.
//!
//! # Offline training: what [`LowBitCoreTrainer`] adds
//!
//! The serving core computes but cannot learn on its own. The trainer supplies the missing half:
//!
//! * **`f32` master weights.** Quantisation happens on every forward pass, using exactly the rule
//!   [`TernaryLinear::quantize`] applies, so the trainer optimises the function that will actually
//!   serve rather than a float approximation of it.
//! * **Straight-through estimator.** The ternary step is treated as the identity in the backward
//!   pass: `∂w_q/∂w = 1`. The per-row power-of-two scale is a function of the master row's `amax`,
//!   not a free parameter, so it is treated as a constant and cancelled by the identity — the
//!   gradient on a master weight is simply `∂L/∂y · x`, as in an ordinary linear layer.
//! * **BPTT.** The recurrence is unrolled and the gradient carried back through every step and
//!   through the `relu` mask.
//! * **Adam.** Bias-corrected, matching the update used elsewhere in this crate
//!   (`learner/jepa_trainer.rs`).
//!
//! # What remains deliberately absent
//!
//! * **Chat data.** This core will learn whatever it is given; TinyStories will not produce a
//!   conversational model.
//! * **Scale.** `dim` here is tens, not thousands. The point is a working, verifiable composition,
//!   not a claim of capability.
//!
//! No capability is claimed for this module. It is a core, and the evidence for it is that its
//! arithmetic is exact and its serving path is multiplier-free.

#![forbid(unsafe_code)]

use super::lowbit::TernaryLinear;

/// Dimension of the recurrent state.
pub const DEFAULT_STATE_DIM: usize = 64;

/// A ternary recurrent core in its serving form: three ternary matrices, integer arithmetic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowBitCore {
    pub vocab: usize,
    pub dim: usize,
    /// Learned embedding, `dim x vocab`.
    pub w_x: TernaryLinear,
    /// Learned recurrent transition, `dim x dim`.
    pub w_h: TernaryLinear,
    /// Learned readout, `vocab x dim`.
    pub w_o: TernaryLinear,
}

impl LowBitCore {
    /// Build from float master matrices, quantising to ternary with power-of-two scales.
    ///
    /// Offline only. This is the bridge from training (float) to serving (integer): the trainer
    /// holds `f32` masters and hands them here, and what comes out is what executes.
    pub fn from_f32(
        vocab: usize,
        dim: usize,
        wx: &[f32],
        wh: &[f32],
        wo: &[f32],
    ) -> Result<Self, String> {
        if vocab == 0 || dim == 0 {
            return Err("vocab and dim must be non-zero".into());
        }
        if wx.len() != dim * vocab {
            return Err(format!(
                "w_x must be {dim}x{vocab} = {}, got {}",
                dim * vocab,
                wx.len()
            ));
        }
        if wh.len() != dim * dim {
            return Err(format!(
                "w_h must be {dim}x{dim} = {}, got {}",
                dim * dim,
                wh.len()
            ));
        }
        if wo.len() != vocab * dim {
            return Err(format!(
                "w_o must be {vocab}x{dim} = {}, got {}",
                vocab * dim,
                wo.len()
            ));
        }
        Ok(Self {
            vocab,
            dim,
            w_x: TernaryLinear::quantize(wx, dim, vocab),
            w_h: TernaryLinear::quantize(wh, dim, dim),
            w_o: TernaryLinear::quantize(wo, vocab, dim),
        })
    }

    /// The zero state: all zeros, which `relu` leaves at zero.
    #[inline]
    pub fn initial_state(&self) -> Vec<i32> {
        vec![0i32; self.dim]
    }

    /// One recurrent step. `h` must have length `dim`; `token` must be in `0..vocab`.
    ///
    /// Multiplier-free: `W_x[token]` is a selected column read, `W_h · h` is adds/shifts, and
    /// `relu` is a max. Returns the next state.
    pub fn step(&self, h: &[i32], token: u32) -> Vec<i32> {
        assert_eq!(h.len(), self.dim, "state length must equal dim");
        let t = (token as usize).min(self.vocab - 1);

        // W_x[token]: column `t` of a dim x vocab ternary matrix, scaled per row. A selected
        // column read, not a contraction over the vocabulary.
        let mut next = vec![0i32; self.dim];
        for (r, slot) in next.iter_mut().enumerate() {
            let w = self.w_x.weight(r, t);
            *slot = w << self.w_x.shift(r);
        }

        // W_h · h, added in. Both terms are pre-scaled, so they combine as integers directly.
        let recurrent = self.w_h.forward_i32(h);
        for (r, v) in recurrent.iter().enumerate() {
            next[r] += *v;
        }

        // relu: a max against zero, no multiply and no float.
        for v in next.iter_mut() {
            if *v < 0 {
                *v = 0;
            }
        }
        next
    }

    /// Logits over the vocabulary for a state.
    pub fn logits(&self, h: &[i32]) -> Vec<i32> {
        self.w_o.forward_i32(h)
    }

    /// Run a whole token sequence and return the logits of the final step.
    ///
    /// This is the autoregressive forward pass: feed `tokens`, get a distribution for the next one.
    pub fn forward_i32(&self, tokens: &[u32]) -> Vec<i32> {
        let mut h = self.initial_state();
        for &t in tokens {
            h = self.step(&h, t);
        }
        self.logits(&h)
    }

    /// Mean next-token cross-entropy of a sequence under the quantised weights, in floating point.
    ///
    /// This is an **offline evaluation** instrument, not a serving operation: it exists so the
    /// trainer and the learning test can score the exact function the serving path executes, on the
    /// same sequence, without a softmax in the kernel. It uses the same quantised weights and the
    /// same scale-per-row arithmetic as [`LowBitCore::forward_i32`], so a low loss here is a
    /// statement about the served model and not about a separate float model.
    pub fn sequence_loss(&self, tokens: &[u32]) -> f32 {
        if tokens.len() < 2 {
            return 0.0;
        }
        let mut h = vec![0f32; self.dim];
        let preds = tokens.len() - 1;
        let mut total = 0f64;
        for i in 0..preds {
            let token = (tokens[i] as usize).min(self.vocab - 1);

            // h <- relu(W_x[:, token] + W_h · h), all in the quantised float algebra.
            let mut next = vec![0f32; self.dim];
            for (r, slot) in next.iter_mut().enumerate() {
                *slot = self.w_x.weight(r, token) as f32 * (1u64 << self.w_x.shift(r)) as f32;
            }
            for r in 0..self.dim {
                let mut acc = 0f32;
                for c in 0..self.dim {
                    acc += self.w_h.weight(r, c) as f32 * h[c];
                }
                next[r] += acc * (1u64 << self.w_h.shift(r)) as f32;
            }
            for v in next.iter_mut() {
                if *v < 0.0 {
                    *v = 0.0;
                }
            }
            h = next;

            let mut logits = vec![0f32; self.vocab];
            for (r, slot) in logits.iter_mut().enumerate() {
                let mut acc = 0f32;
                for c in 0..self.dim {
                    acc += self.w_o.weight(r, c) as f32 * h[c];
                }
                *slot = acc * (1u64 << self.w_o.shift(r)) as f32;
            }
            let p = softmax_f32(&logits);
            let target = (tokens[i + 1] as usize).min(self.vocab - 1);
            total += -(p[target].max(1e-9)).ln() as f64;
        }
        (total / preds as f64) as f32
    }

    /// Total packed weight bytes, for the bytes-per-token accounting.
    pub fn weight_bytes(&self) -> usize {
        self.w_x.weight_bytes() + self.w_h.weight_bytes() + self.w_o.weight_bytes()
    }

    /// Floating-point forward using the *quantised* weights in f32, for the trainer's diagnostics.
    /// Offline only; the real-valued state is not a serving path.
    pub fn forward_reference_f32(&self, tokens: &[u32]) -> Vec<f32> {
        let mut h = vec![0f32; self.dim];
        for &t in tokens {
            let token = (t as usize).min(self.vocab - 1);
            let mut next = vec![0f32; self.dim];
            for (r, slot) in next.iter_mut().enumerate() {
                *slot = self.w_x.weight(r, token) as f32 * (1u64 << self.w_x.shift(r)) as f32;
            }
            for r in 0..self.dim {
                let mut acc = 0f32;
                for c in 0..self.dim {
                    acc += self.w_h.weight(r, c) as f32 * h[c];
                }
                next[r] += acc * (1u64 << self.w_h.shift(r)) as f32;
            }
            for v in next.iter_mut() {
                if *v < 0.0 {
                    *v = 0.0;
                }
            }
            h = next;
        }
        let mut logits = vec![0f32; self.vocab];
        for (r, slot) in logits.iter_mut().enumerate() {
            let mut acc = 0f32;
            for c in 0..self.dim {
                acc += self.w_o.weight(r, c) as f32 * h[c];
            }
            *slot = acc * (1u64 << self.w_o.shift(r)) as f32;
        }
        logits
    }

    /// Floating-point forward using the *quantised* weights in f64, for verifying the integer path.
    pub fn float_forward_ste(&self, tokens: &[u32]) -> Vec<f64> {
        let mut h = vec![0.0f64; self.dim];
        for &t in tokens {
            let token = (t as usize).min(self.vocab - 1);
            let mut next = vec![0.0f64; self.dim];
            for (r, slot) in next.iter_mut().enumerate() {
                *slot = self.w_x.weight(r, token) as f64 * (1u64 << self.w_x.shift(r)) as f64;
            }
            let recurrent = self.w_h.forward_reference(&h);
            for (r, v) in recurrent.iter().enumerate() {
                next[r] += *v;
            }
            for v in next.iter_mut() {
                if *v < 0.0 {
                    *v = 0.0;
                }
            }
            h = next;
        }
        self.w_o.forward_reference(&h)
    }
}

// ---------------------------------------------------------------------------
// Offline training: f32 masters, quantise on the forward pass, STE backward.
// ---------------------------------------------------------------------------

/// Offline training configuration for [`LowBitCoreTrainer`].
#[derive(Debug, Clone, PartialEq)]
pub struct TrainConfig {
    /// Adam learning rate.
    pub lr: f32,
    pub beta1: f32,
    pub beta2: f32,
    /// Decoupled weight decay, applied after the Adam update.
    pub weight_decay: f32,
    /// Global gradient-norm clip. `0.0` disables clipping.
    pub grad_clip: f32,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            lr: 0.05,
            beta1: 0.9,
            beta2: 0.999,
            weight_decay: 0.0,
            grad_clip: 1.0,
        }
    }
}

/// Quantised view of a master matrix: ternary codes plus a per-row power-of-two scale.
///
/// This reproduces [`TernaryLinear::quantize`] exactly, computed on the fly for every forward pass
/// so that the trainer optimises the serving function rather than an approximation of it. The
/// backward pass does not differentiate this function; it treats it as the identity (STE).
fn quantize_codes(w: &[f32], rows: usize, cols: usize) -> (Vec<i8>, Vec<f32>) {
    let mut q = vec![0i8; rows * cols];
    let mut scale = vec![0f32; rows];
    for r in 0..rows {
        let row = &w[r * cols..(r + 1) * cols];
        let amax = row.iter().fold(0f32, |m, &v| m.max(v.abs()));
        let s = if amax > 0.0 {
            amax.log2().floor().max(0.0).min(30.0) as u32
        } else {
            0
        };
        let sc = (1u32 << s) as f32;
        scale[r] = sc;
        for c in 0..cols {
            let v = (row[c] / sc).round();
            q[r * cols + c] = if v >= 1.0 {
                1
            } else if v <= -1.0 {
                -1
            } else {
                0
            };
        }
    }
    (q, scale)
}

/// Numerically stable softmax over a logit vector. Offline only.
fn softmax_f32(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().fold(f32::NEG_INFINITY, |m, &v| m.max(v));
    let mut out: Vec<f32> = logits.iter().map(|&v| (v - max).exp()).collect();
    let sum: f32 = out.iter().sum();
    if sum > 0.0 {
        for v in out.iter_mut() {
            *v /= sum;
        }
    }
    out
}

/// Pre-activation `W_x[:, token] + W_h · h_prev` in the quantised algebra.
#[allow(clippy::too_many_arguments)]
fn preact_f32(
    h_prev: &[f32],
    token: usize,
    qx: &[i8],
    sx: &[f32],
    qh: &[i8],
    sh: &[f32],
    dim: usize,
    vocab: usize,
) -> Vec<f32> {
    let mut a = vec![0f32; dim];
    for r in 0..dim {
        let mut acc = 0f32;
        for c in 0..dim {
            acc += qh[r * dim + c] as f32 * h_prev[c];
        }
        a[r] = sx[r] * qx[r * vocab + token] as f32 + sh[r] * acc;
    }
    a
}

/// `W_o · h` in the quantised algebra.
fn logits_f32(h: &[f32], qo: &[i8], so: &[f32], vocab: usize, dim: usize) -> Vec<f32> {
    let mut out = vec![0f32; vocab];
    for r in 0..vocab {
        let mut acc = 0f32;
        for c in 0..dim {
            acc += qo[r * dim + c] as f32 * h[c];
        }
        out[r] = so[r] * acc;
    }
    out
}

fn xorshift_unit(state: &mut u64) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    let u = ((*state >> 11) as f64) / ((1u64 << 53) as f64);
    (u * 2.0 - 1.0) as f32
}

/// Bias-corrected Adam on one flat parameter matrix. Mirrors the update in
/// `learner/jepa_trainer.rs::AdamMoments::update`.
fn adam_update(
    params: &mut [f32],
    grads: &[f32],
    m: &mut [f32],
    v: &mut [f32],
    cfg: &TrainConfig,
    step: u64,
) {
    let b1 = cfg.beta1 as f64;
    let b2 = cfg.beta2 as f64;
    let bc1 = 1.0 - libm::pow(b1, step as f64);
    let bc2 = 1.0 - libm::pow(b2, step as f64);
    let lr = cfg.lr as f64;
    for i in 0..params.len() {
        let g = grads[i] as f64;
        let mi = b1 * m[i] as f64 + (1.0 - b1) * g;
        let vi = b2 * v[i] as f64 + (1.0 - b2) * g * g;
        m[i] = mi as f32;
        v[i] = vi as f32;
        let m_hat = mi / (bc1 + 1e-12);
        let v_hat = vi / (bc2 + 1e-12);
        if v_hat > 1e-12 {
            params[i] -= (lr * m_hat / (libm::sqrt(v_hat) + 1e-8)) as f32;
        }
        if cfg.weight_decay > 0.0 {
            params[i] -= (lr * cfg.weight_decay as f64 * params[i] as f64) as f32;
        }
    }
}

/// Offline trainer for [`LowBitCore`].
///
/// Holds `f32` masters for the three matrices. Every forward pass quantises them to ternary with a
/// per-row power-of-two scale, exactly as the serving core does. The backward pass is a
/// straight-through estimator through that quantisation, BPTT through the recurrence, and Adam on
/// the masters.
#[derive(Debug, Clone)]
pub struct LowBitCoreTrainer {
    pub vocab: usize,
    pub dim: usize,
    /// Embedding masters, row-major `dim x vocab`.
    pub wx: Vec<f32>,
    /// Recurrent masters, row-major `dim x dim`.
    pub wh: Vec<f32>,
    /// Readout masters, row-major `vocab x dim`.
    pub wo: Vec<f32>,
    pub cfg: TrainConfig,
    gwx: Vec<f32>,
    gwh: Vec<f32>,
    gwo: Vec<f32>,
    mwx: Vec<f32>,
    vwx: Vec<f32>,
    mwh: Vec<f32>,
    vwh: Vec<f32>,
    mwo: Vec<f32>,
    vwo: Vec<f32>,
    step: u64,
}

impl LowBitCoreTrainer {
    /// Build a trainer with pseudo-random masters drawn uniformly from `[-1, 1]`.
    ///
    /// The magnitude matters: the per-row scale is `2^floor(log2 amax)` clamped to a non-negative
    /// shift, so a row whose `amax` is far below `1` quantises entirely to zero and is dead. A
    /// `[-1, 1]` draw keeps every row live at initialisation.
    pub fn new(vocab: usize, dim: usize, seed: u64) -> Result<Self, String> {
        if vocab == 0 || dim == 0 {
            return Err("vocab and dim must be non-zero".into());
        }
        let mut st = seed ^ 0x9E37_79B9_7F4A_7C15;
        if st == 0 {
            st = 0x1234_5678_9ABC_DEF0;
        }
        let mut fill =
            |n: usize| -> Vec<f32> { (0..n).map(|_| xorshift_unit(&mut st)).collect::<Vec<f32>>() };
        Ok(Self {
            vocab,
            dim,
            wx: fill(dim * vocab),
            wh: fill(dim * dim),
            wo: fill(vocab * dim),
            cfg: TrainConfig::default(),
            gwx: vec![0f32; dim * vocab],
            gwh: vec![0f32; dim * dim],
            gwo: vec![0f32; vocab * dim],
            mwx: vec![0f32; dim * vocab],
            vwx: vec![0f32; dim * vocab],
            mwh: vec![0f32; dim * dim],
            vwh: vec![0f32; dim * dim],
            mwo: vec![0f32; vocab * dim],
            vwo: vec![0f32; vocab * dim],
            step: 0,
        })
    }

    /// Quantise the current masters into the serving core. This is the bridge from training to
    /// serving: what comes out is exactly what executes.
    pub fn to_core(&self) -> Result<LowBitCore, String> {
        LowBitCore::from_f32(self.vocab, self.dim, &self.wx, &self.wh, &self.wo)
    }

    /// Mean next-token cross-entropy under the current quantised masters, without updating.
    pub fn loss(&self, tokens: &[u32]) -> f32 {
        if tokens.len() < 2 {
            return 0.0;
        }
        let (qx, sx) = quantize_codes(&self.wx, self.dim, self.vocab);
        let (qh, sh) = quantize_codes(&self.wh, self.dim, self.dim);
        let (qo, so) = quantize_codes(&self.wo, self.vocab, self.dim);
        let mut h = vec![0f32; self.dim];
        let preds = tokens.len() - 1;
        let mut total = 0f64;
        for i in 0..preds {
            let token = (tokens[i] as usize).min(self.vocab - 1);
            let pre = preact_f32(&h, token, &qx, &sx, &qh, &sh, self.dim, self.vocab);
            for r in 0..self.dim {
                h[r] = pre[r].max(0.0);
            }
            let logits = logits_f32(&h, &qo, &so, self.vocab, self.dim);
            let p = softmax_f32(&logits);
            let target = (tokens[i + 1] as usize).min(self.vocab - 1);
            total += -(p[target].max(1e-9)).ln() as f64;
        }
        (total / preds as f64) as f32
    }

    /// Next-token top-1 accuracy over a sequence, under the quantised weights.
    ///
    /// Returns `(correct, total)` over the `len - 1` prediction positions. This is a more
    /// honest quality signal than the loss alone, because a core can lower loss slightly by
    /// matching the unigram marginal without ever predicting a specific continuation.
    pub fn next_token_accuracy(&self, tokens: &[u32]) -> (usize, usize) {
        if tokens.len() < 2 {
            return (0, 0);
        }
        let (qx, sx) = quantize_codes(&self.wx, self.dim, self.vocab);
        let (qh, sh) = quantize_codes(&self.wh, self.dim, self.dim);
        let (qo, so) = quantize_codes(&self.wo, self.vocab, self.dim);
        let mut h = vec![0f32; self.dim];
        let mut correct = 0usize;
        let total = tokens.len() - 1;
        for i in 0..total {
            let token = (tokens[i] as usize).min(self.vocab - 1);
            let pre = preact_f32(&h, token, &qx, &sx, &qh, &sh, self.dim, self.vocab);
            for r in 0..self.dim {
                h[r] = pre[r].max(0.0);
            }
            let logits = logits_f32(&h, &qo, &so, self.vocab, self.dim);
            let mut best = 0usize;
            for (j, &v) in logits.iter().enumerate() {
                if v > logits[best] {
                    best = j;
                }
            }
            if best == (tokens[i + 1] as usize).min(self.vocab - 1) {
                correct += 1;
            }
        }
        (correct, total)
    }

    /// Whether the argmax of the final prediction matches the final target token.
    pub fn final_token_correct(&self, tokens: &[u32]) -> bool {
        if tokens.len() < 2 {
            return false;
        }
        let core = match self.to_core() {
            Ok(c) => c,
            Err(_) => return false,
        };
        let logits = core.forward_reference_f32(&tokens[..tokens.len() - 1]);
        let mut best = 0usize;
        for (i, &v) in logits.iter().enumerate() {
            if v > logits[best] {
                best = i;
            }
        }
        best == (tokens[tokens.len() - 1] as usize).min(self.vocab - 1)
    }

    /// Accumulate gradients for one sequence and return its mean loss.
    fn accumulate(&mut self, tokens: &[u32]) -> f32 {
        let n = tokens.len();
        if n < 2 {
            return 0.0;
        }
        let (qx, sx) = quantize_codes(&self.wx, self.dim, self.vocab);
        let (qh, sh) = quantize_codes(&self.wh, self.dim, self.dim);
        let (qo, so) = quantize_codes(&self.wo, self.vocab, self.dim);

        // Forward, caching the post-relu state and the pre-activation of every step.
        let mut states: Vec<Vec<f32>> = Vec::with_capacity(n);
        let mut preacts: Vec<Vec<f32>> = Vec::with_capacity(n);
        let mut h = vec![0f32; self.dim];
        for &t in tokens {
            let token = (t as usize).min(self.vocab - 1);
            let pre = preact_f32(&h, token, &qx, &sx, &qh, &sh, self.dim, self.vocab);
            for r in 0..self.dim {
                h[r] = pre[r].max(0.0);
            }
            states.push(h.clone());
            preacts.push(pre);
        }

        let preds = n - 1;
        let inv = 1.0f32 / preds as f32;
        let zero = vec![0f32; self.dim];
        let mut loss = 0f64;
        // `dh` holds dL/dh_t, seeded by the future step and augmented by this step's readout.
        let mut dh = vec![0f32; self.dim];

        for t in (0..preds).rev() {
            let logits = logits_f32(&states[t], &qo, &so, self.vocab, self.dim);
            let mut p = softmax_f32(&logits);
            let target = (tokens[t + 1] as usize).min(self.vocab - 1);
            loss += -(p[target].max(1e-9)).ln() as f64;
            p[target] -= 1.0;

            // Readout. STE: the gradient on the master is `g_r * h_c`, with the row scale
            // cancelling against the quantiser's own `1/scale`. The gradient passed to earlier
            // layers uses the actual quantised weight, scale included.
            for r in 0..self.vocab {
                let g = p[r] * inv;
                if g == 0.0 {
                    continue;
                }
                let base = r * self.dim;
                let sr = so[r];
                for c in 0..self.dim {
                    self.gwo[base + c] += g * states[t][c];
                    dh[c] += g * (qo[base + c] as f32 * sr);
                }
            }

            // Recurrence: a_t = W_x[:, token_t] + W_h · h_{t-1}.
            let hprev = if t == 0 { &zero } else { &states[t - 1] };
            let token = (tokens[t] as usize).min(self.vocab - 1);
            for r in 0..self.dim {
                let da = if preacts[t][r] > 0.0 { dh[r] } else { 0.0 };
                if da == 0.0 {
                    continue;
                }
                self.gwx[r * self.vocab + token] += da * inv;
                let base = r * self.dim;
                for c in 0..self.dim {
                    self.gwh[base + c] += da * hprev[c] * inv;
                }
            }
            let mut dh_prev = vec![0f32; self.dim];
            for r in 0..self.dim {
                let da = if preacts[t][r] > 0.0 { dh[r] } else { 0.0 };
                if da == 0.0 {
                    continue;
                }
                let base = r * self.dim;
                let sr = sh[r];
                for c in 0..self.dim {
                    dh_prev[c] += da * (qh[base + c] as f32 * sr);
                }
            }
            dh = dh_prev;
        }
        (loss / preds as f64) as f32
    }

    fn zero_grads(&mut self) {
        for g in self.gwx.iter_mut() {
            *g = 0.0;
        }
        for g in self.gwh.iter_mut() {
            *g = 0.0;
        }
        for g in self.gwo.iter_mut() {
            *g = 0.0;
        }
    }

    /// Divide the accumulated gradients by the batch size and clip their global norm.
    fn normalize_and_clip(&mut self, batch: usize) {
        let scale = 1.0f32 / batch.max(1) as f32;
        let mut sumsq = 0f64;
        for g in self.gwx.iter_mut() {
            *g *= scale;
            sumsq += (*g as f64) * (*g as f64);
        }
        for g in self.gwh.iter_mut() {
            *g *= scale;
            sumsq += (*g as f64) * (*g as f64);
        }
        for g in self.gwo.iter_mut() {
            *g *= scale;
            sumsq += (*g as f64) * (*g as f64);
        }
        if self.cfg.grad_clip > 0.0 {
            let norm = libm::sqrt(sumsq);
            let clip = self.cfg.grad_clip as f64;
            if norm > clip && norm > 0.0 {
                let s = (clip / norm) as f32;
                for g in self.gwx.iter_mut() {
                    *g *= s;
                }
                for g in self.gwh.iter_mut() {
                    *g *= s;
                }
                for g in self.gwo.iter_mut() {
                    *g *= s;
                }
            }
        }
    }

    fn apply_adam(&mut self) {
        self.step += 1;
        let step = self.step;
        let cfg = self.cfg.clone();
        adam_update(
            &mut self.wx,
            &self.gwx,
            &mut self.mwx,
            &mut self.vwx,
            &cfg,
            step,
        );
        adam_update(
            &mut self.wh,
            &self.gwh,
            &mut self.mwh,
            &mut self.vwh,
            &cfg,
            step,
        );
        adam_update(
            &mut self.wo,
            &self.gwo,
            &mut self.mwo,
            &mut self.vwo,
            &cfg,
            step,
        );
    }

    /// Train on one batch: accumulate gradients over the sequences, then one Adam step.
    /// Returns the mean per-sequence loss before the update.
    pub fn train_batch(&mut self, batch: &[Vec<u32>]) -> f32 {
        if batch.is_empty() {
            return 0.0;
        }
        self.zero_grads();
        let mut total = 0f64;
        for seq in batch {
            total += self.accumulate(seq) as f64;
        }
        self.normalize_and_clip(batch.len());
        self.apply_adam();
        (total / batch.len() as f64) as f32
    }

    /// Train on a single sequence.
    pub fn train_sequence(&mut self, tokens: &[u32]) -> f32 {
        self.train_batch(&[tokens.to_vec()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic small core, built from pseudo-random float masters.
    fn tiny() -> LowBitCore {
        let vocab = 11usize;
        let dim = 8usize;
        let mut state = 0x1234_5678_9ABC_DEF0u64;
        let mut next_f32 = |scale: f32| -> f32 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let u = ((state >> 11) as f64 / (1u64 << 53) as f64) as f32;
            (u * 2.0 - 1.0) * scale
        };
        let wx: Vec<f32> = (0..dim * vocab).map(|_| next_f32(2.0)).collect();
        let wh: Vec<f32> = (0..dim * dim).map(|_| next_f32(1.0)).collect();
        let wo: Vec<f32> = (0..vocab * dim).map(|_| next_f32(3.0)).collect();
        LowBitCore::from_f32(vocab, dim, &wx, &wh, &wo).expect("build")
    }

    #[test]
    fn integer_forward_is_exactly_the_float_reference() {
        let core = tiny();
        let tokens: Vec<u32> = vec![0, 3, 7, 1, 10, 2, 2, 5];
        let got = core.forward_i32(&tokens);
        let want = core.float_forward_ste(&tokens);
        assert_eq!(got.len(), core.vocab);
        for j in 0..core.vocab {
            assert_eq!(
                got[j] as f64, want[j],
                "vocab {j}: serving path {} != reference {}",
                got[j], want[j]
            );
        }
    }

    #[test]
    fn relu_keeps_the_state_non_negative() {
        let core = tiny();
        let mut h = core.initial_state();
        for t in [1u32, 4, 9, 0, 3] {
            h = core.step(&h, t);
            assert!(
                h.iter().all(|&v| v >= 0),
                "relu must leave no negative state"
            );
        }
    }

    #[test]
    fn forward_is_deterministic_and_sensitive_to_order() {
        let core = tiny();
        let a = core.forward_i32(&[1, 2, 3]);
        let b = core.forward_i32(&[1, 2, 3]);
        assert_eq!(a, b, "same input must give the same output");
        // The recurrence means order matters; a bag-of-tokens core could not tell these apart.
        let c = core.forward_i32(&[3, 2, 1]);
        assert_ne!(a, c, "order must change the logits");
    }

    #[test]
    fn empty_sequence_gives_the_readout_of_the_zero_state() {
        let core = tiny();
        let logits = core.forward_i32(&[]);
        assert_eq!(logits.len(), core.vocab);
    }

    #[test]
    fn out_of_range_token_is_clamped_not_a_panic() {
        let core = tiny();
        let logits = core.forward_i32(&[u32::MAX]);
        assert_eq!(logits.len(), core.vocab);
    }

    #[test]
    fn weights_are_ternary_at_two_bits_and_the_core_is_small() {
        let core = tiny();
        let n = core.vocab * core.dim + core.dim * core.dim + core.vocab * core.dim;
        assert_eq!(core.weight_bytes(), n.div_ceil(4));
        for r in 0..core.dim {
            for c in 0..core.vocab {
                let w = core.w_x.weight(r, c);
                assert!(w == -1 || w == 0 || w == 1);
            }
        }
        // At 2 bits per weight, a 4096-vocabulary, 64-dim core would be about 130 KB of weights.
        assert!(core.weight_bytes() * 8 <= n * 4);
    }

    #[test]
    fn rejects_inconsistent_shapes() {
        let bad = LowBitCore::from_f32(4, 2, &[0.0; 7], &[0.0; 4], &[0.0; 8]);
        assert!(bad.is_err());
        assert!(LowBitCore::from_f32(0, 2, &[], &[0.0; 4], &[]).is_err());
    }

    // -----------------------------------------------------------------------
    // Training: does the backward pass actually learn, and does what it learns
    // survive the integer quantisation that serves it?
    // -----------------------------------------------------------------------

    fn xorshift_u64(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    /// A synthetic delayed-recall language: `[item, 5 x distractors, QUERY, item]`.
    ///
    /// The final prediction requires the *first* token to survive the distractor steps, so a
    /// core with no recurrence — whose state after `QUERY` is `relu(W_x[:, QUERY])`, independent of
    /// the item — cannot exceed the 1-in-4 chance rate. Tokens: `0` = query, `1..=4` = items,
    /// `5` = distractor.
    fn recall_batch(seed: u64, n: usize, distractors: usize) -> Vec<Vec<u32>> {
        let mut st = seed | 1;
        (0..n)
            .map(|_| {
                let item = 1 + (xorshift_u64(&mut st) % 4) as u32;
                let mut seq = vec![item];
                seq.extend(std::iter::repeat(5).take(distractors));
                seq.push(0);
                seq.push(item);
                seq
            })
            .collect()
    }

    fn mean_loss(t: &LowBitCoreTrainer, seqs: &[Vec<u32>]) -> f32 {
        let s: f64 = seqs.iter().map(|q| t.loss(q) as f64).sum();
        (s / seqs.len() as f64) as f32
    }

    #[test]
    fn learns_a_task_the_recurrence_is_required_for() {
        let vocab = 6usize;
        let dim = 16usize;
        let mut trainer = LowBitCoreTrainer::new(vocab, dim, 2026_0919).expect("build");
        trainer.cfg.lr = 0.02;

        let train = recall_batch(0xA5A5_1234, 64, 1);
        let held = recall_batch(0x0BAD_F00D, 32, 1);

        let initial = mean_loss(&trainer, &held);
        let mut last = initial;
        for _ in 0..400 {
            last = trainer.train_batch(&train);
        }
        let final_loss = mean_loss(&trainer, &held);
        assert!(
            last.is_finite() && final_loss.is_finite(),
            "training produced a non-finite loss"
        );
        assert!(
            final_loss < initial * 0.2 && final_loss < 0.25,
            "loss must fall on the held-out draw: initial {initial:.4} -> final {final_loss:.4} \
             (last training batch {last:.4})"
        );

        // Recall at the query position must be well above the 1-in-4 chance rate.
        let hits = held
            .iter()
            .filter(|s| trainer.final_token_correct(s))
            .count();
        let acc = hits as f32 / held.len() as f32;
        assert!(acc >= 0.9, "recall accuracy {acc:.2} must exceed chance");

        // Control: removing the recurrence removes the memory, and the loss must not recover.
        let mut ctrl = trainer.clone();
        for v in ctrl.wh.iter_mut() {
            *v = 0.0;
        }
        let ctrl_loss = mean_loss(&ctrl, &held);
        assert!(
            ctrl_loss > final_loss + 0.05,
            "the recurrence must be load-bearing: trained {final_loss:.4} vs no-recurrence control \
             {ctrl_loss:.4}"
        );

        // The trainer's quantised forward must be the same function the core serves.
        let core = trainer.to_core().expect("quantize");
        for seq in &held {
            let a = trainer.loss(seq);
            let b = core.sequence_loss(seq);
            assert!(
                (a - b).abs() < 1e-4,
                "trainer loss {a:.6} != core loss {b:.6}: the trainer is not optimising the served function"
            );
        }

        // And the trained integer serving path must still equal its float reference exactly.
        for seq in &held {
            let got = core.forward_i32(seq);
            let want = core.float_forward_ste(seq);
            assert_eq!(got.len(), vocab);
            for j in 0..vocab {
                assert_eq!(
                    got[j] as f64, want[j],
                    "trained integer path diverged from the float reference at vocab {j}"
                );
            }
        }
    }

    #[test]
    fn training_is_deterministic_for_a_fixed_seed() {
        let data = recall_batch(7, 16, 1);
        let mut a = LowBitCoreTrainer::new(6, 12, 99).expect("build");
        let mut b = LowBitCoreTrainer::new(6, 12, 99).expect("build");
        for _ in 0..20 {
            let la = a.train_batch(&data);
            let lb = b.train_batch(&data);
            assert_eq!(la, lb, "same seed must give the same loss trajectory");
        }
        assert_eq!(a.wx, b.wx);
        assert_eq!(a.wh, b.wh);
        assert_eq!(a.wo, b.wo);
    }

    #[test]
    fn gradients_are_produced_for_every_matrix() {
        let mut trainer = LowBitCoreTrainer::new(6, 10, 5).expect("build");
        let data = recall_batch(11, 4, 1);
        trainer.zero_grads();
        for seq in &data {
            trainer.accumulate(seq);
        }
        let nz = |g: &[f32]| g.iter().any(|v| v.abs() > 0.0);
        assert!(nz(&trainer.gwx), "no embedding gradient");
        assert!(nz(&trainer.gwh), "no recurrent gradient");
        assert!(nz(&trainer.gwo), "no readout gradient");
    }
}
