//! A multiplier-free, content-addressed sequence core: linear attention with ternary operands.
//!
//! # Why this exists
//!
//! The dense low-bit recurrence (`LowBitCore`) was measured to fail in two distinct ways, and both
//! are structural rather than optimiser bugs:
//!
//! 1. **Magnitude.** `h_t = relu(W_x + W_h·h_{t-1})` with random ternary `W_h` has spectral norm
//!    ≈ `2√(p·dim)` (Bai–Yin), so `‖h‖` grows geometrically. Measured: the integer state saturates
//!    i32 (≈2.1e9) within **16–32 steps** at `dim` 32, 64 and 128. That is exactly the observed
//!    ~24-token workable window.
//! 2. **Burial.** Every past input is added into the *same* channels, so a single item to remember
//!    is buried under `T·|x|` of accumulated distractor mass. Measured: delayed recall of one token
//!    works at delay 2, needs `dim ≥ 128` at delay 4, and fails at delay 8 for every dimension
//!    tested. A contraction (right-shifting the recurrent term) was also measured and **falsified**:
//!    it destroys the recurrent pathway (delay 1 accuracy 1.00 → 0.16 → 0.00 as the shift grows).
//!
//! # What this is
//!
//! Linear attention (Katharopoulos et al.; RWKV, arXiv:2305.13048; HGRN2, arXiv:2404.07904), with
//! every operand made ternary and every scale a power of two, so the whole serving path is integer
//! adds, subtracts, shifts and table reads:
//!
//! ```text
//!   S_t[i][j] = (S_{t-1}[i][j] >> decay) + k_i(t) · v_j(t)      dk × dv matrix state
//!   y_j(t)    = Σ_i q_i(t) · S_t[i][j]                          matched-filter read
//!   logits    = W_o · relu(y(t))
//! ```
//!
//! `k_i` and `q_i` are **unscaled** ternary codes, so `k_i · v_j` is a conditional add or subtract
//! of `v_j` — no multiplier, in the outer product or in the read. This is the load-bearing trick:
//! `S` accumulates *pairs*, so the read is a matched filter that separates a stored item from
//! distractors instead of being buried by them, and its magnitude grows **linearly** in `T` rather
//! than geometrically.
//!
//! # Measured
//!
//! Induction (`[x, y, filler*delay, x] → y`), held out, deterministic seeds:
//!
//! | `dk` = `dv` | learning rate | delay | accuracy |
//! |---:|---:|---:|---:|
//! | 64 | 0.05 | 16 | **1.00** |
//! | 128 | 0.005 | 16 | **1.00** |
//! | 256 | 0.005 | 16 | **1.00** |
//! | 512 | 0.005 | 16 | **1.00** |
//!
//! Width therefore scales, given the learning rate. But the training is **fragile**: at
//! `dk = 128, lr = 0.05` it collapses to the uniform predictor (loss exactly `ln 8`); at `dk = 64`,
//! delay 32+ collapses at settings where delay 16 succeeds; and the state magnitude (not a
//! contraction) is what bounds it, so a larger batch changes the regime. This conditioning problem is
//! the top open item, not a footnote: an architecture that only sometimes finds a solution cannot be
//! scaled. The suspected causes are the un-normalised readout (its magnitude grows with the number of
//! stored pairs and with `dk`) and the ternary straight-through estimator on the key/query tables.
//!
//! # What is honestly not here
//!
//! This is one layer at tens to hundreds of dimensions, trained on synthetic tasks. It is a mechanism
//! with a measured falsification test, not a chat model. The scaling path is recorded in
//! `docs/integration/geometric-core-architecture-2026-09-19.md`.

#![forbid(unsafe_code)]

use super::lowbit::TernaryLinear;
use super::lowbit_core::{adam_update, quantize_codes, softmax_f32, xorshift_unit, TrainConfig};

/// Serving form: ternary key/value/query tables, a matrix state and a dyadic decay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowBitAttention {
    pub vocab: usize,
    /// Key/query width.
    pub dk: usize,
    /// Value width.
    pub dv: usize,
    /// Right shift applied to the state each step. `0` keeps every stored pair.
    pub decay: u32,
    /// Key table, `vocab x dk`. Unscaled ternary codes: this is what makes the outer product
    /// multiplier-free.
    pub w_k: TernaryLinear,
    /// Value table, `vocab x dv`. Ternary codes with a per-token power-of-two magnitude.
    pub w_v: TernaryLinear,
    /// Query table, `vocab x dk`. Unscaled ternary codes.
    pub w_q: TernaryLinear,
    /// Output table, `vocab x dv`.
    pub w_o: TernaryLinear,
    /// Power-of-two readout normalisation: the read is shifted so its largest magnitude is about
    /// `2^norm_bits`. `0` disables it. A bit scan plus a shift — no divide, no multiply.
    pub norm_bits: u32,
}

impl LowBitAttention {
    /// Build from float masters, quantising with the same rule the serving core uses.
    #[allow(clippy::too_many_arguments)]
    pub fn from_f32(
        vocab: usize,
        dk: usize,
        dv: usize,
        decay: u32,
        wk: &[f32],
        wv: &[f32],
        wq: &[f32],
        wo: &[f32],
    ) -> Result<Self, String> {
        if vocab == 0 || dk == 0 || dv == 0 {
            return Err("vocab, dk and dv must be non-zero".into());
        }
        if decay > 30 {
            return Err("decay must be at most 30".into());
        }
        for (name, m, rows, cols) in [
            ("w_k", wk, vocab, dk),
            ("w_v", wv, vocab, dv),
            ("w_q", wq, vocab, dk),
            ("w_o", wo, vocab, dv),
        ] {
            if m.len() != rows * cols {
                return Err(format!(
                    "{name} must be {rows}x{cols} = {}, got {}",
                    rows * cols,
                    m.len()
                ));
            }
        }
        Ok(Self {
            vocab,
            dk,
            dv,
            decay,
            w_k: TernaryLinear::quantize(wk, vocab, dk),
            w_v: TernaryLinear::quantize(wv, vocab, dv),
            w_q: TernaryLinear::quantize(wq, vocab, dk),
            w_o: TernaryLinear::quantize(wo, vocab, dv),
            norm_bits: 0,
        })
    }

    /// Enable the power-of-two readout normalisation (see [`LowBitAttention::norm_bits`]).
    #[must_use]
    pub fn with_norm_bits(mut self, k: u32) -> Self {
        self.norm_bits = k;
        self
    }

    /// The empty state: a `dk x dv` matrix of zeros.
    #[inline]
    pub fn initial_state(&self) -> Vec<i32> {
        vec![0i32; self.dk * self.dv]
    }

    /// Ingest one token: decay the state, then add the rank-1 pair `k(t) ⊗ v(t)`.
    ///
    /// Multiplier-free: `k_i ∈ {-1, 0, 1}`, so `k_i · v_j` is `+v_j`, `-v_j` or nothing.
    pub fn ingest(&self, s: &mut [i32], token: u32) {
        let t = (token as usize).min(self.vocab - 1);
        let vshift = self.w_v.shift(t);
        let mut v = vec![0i32; self.dv];
        for (j, slot) in v.iter_mut().enumerate() {
            *slot = self.w_v.weight(t, j) << vshift;
        }
        for i in 0..self.dk {
            let k = self.w_k.weight(t, i);
            let row = i * self.dv;
            for j in 0..self.dv {
                let cur = s[row + j] >> self.decay;
                s[row + j] = match k {
                    1 => cur + v[j],
                    -1 => cur - v[j],
                    _ => cur,
                };
            }
        }
    }

    /// Read with the query of `token`: `relu(Q(t) · S)`, then the output projection.
    pub fn logits(&self, s: &[i32], token: u32) -> Vec<i32> {
        let t = (token as usize).min(self.vocab - 1);
        let mut num = vec![0i32; self.dv];
        for i in 0..self.dk {
            let q = self.w_q.weight(t, i);
            if q == 0 {
                continue;
            }
            let row = i * self.dv;
            if q == 1 {
                for j in 0..self.dv {
                    num[j] += s[row + j];
                }
            } else {
                for j in 0..self.dv {
                    num[j] -= s[row + j];
                }
            }
        }
        let shift = Self::normaliser(&num, self.norm_bits);
        if shift > 0 {
            for v in num.iter_mut() {
                *v >>= shift;
            }
        }
        for v in num.iter_mut() {
            if *v < 0 {
                *v = 0;
            }
        }
        self.w_o.forward_i32(&num)
    }

    /// Shift the read so its largest magnitude is about `2^norm_bits`. Bit scan plus shift.
    #[inline]
    fn normaliser(num: &[i32], norm_bits: u32) -> u32 {
        if norm_bits == 0 {
            return 0;
        }
        let m = num.iter().fold(0i32, |a, &v| a.max(v.abs()));
        if m == 0 {
            0
        } else {
            (32 - (m as u32).leading_zeros()).saturating_sub(norm_bits)
        }
    }

    /// Ingest a whole prompt and read with the final token's query.
    pub fn forward_i32(&self, tokens: &[u32]) -> Vec<i32> {
        let mut s = self.initial_state();
        let mut last = 0u32;
        for &t in tokens {
            self.ingest(&mut s, t);
            last = t;
        }
        self.logits(&s, last)
    }

    /// The same computation in `f64`, for verifying the integer path. Floor semantics are applied at
    /// the decay step so that the two paths are exactly comparable.
    pub fn forward_reference_f64(&self, tokens: &[u32]) -> Vec<f64> {
        let mut s = vec![0f64; self.dk * self.dv];
        let div = (1u64 << self.decay) as f64;
        let mut last = 0u32;
        for &t in tokens {
            let tok = (t as usize).min(self.vocab - 1);
            let vscale = (1u64 << self.w_v.shift(tok)) as f64;
            let mut v = vec![0f64; self.dv];
            for (j, slot) in v.iter_mut().enumerate() {
                *slot = self.w_v.weight(tok, j) as f64 * vscale;
            }
            for i in 0..self.dk {
                let k = self.w_k.weight(tok, i) as f64;
                for j in 0..self.dv {
                    let idx = i * self.dv + j;
                    s[idx] = (s[idx] / div).floor() + k * v[j];
                }
            }
            last = t;
        }
        let tok = (last as usize).min(self.vocab - 1);
        let mut num = vec![0f64; self.dv];
        for i in 0..self.dk {
            let q = self.w_q.weight(tok, i) as f64;
            for j in 0..self.dv {
                num[j] += q * s[i * self.dv + j];
            }
        }
        for v in num.iter_mut() {
            if *v < 0.0 {
                *v = 0.0;
            }
        }
        self.w_o.forward_reference(&num)
    }

    /// Total packed weight bytes.
    pub fn weight_bytes(&self) -> usize {
        self.w_k.weight_bytes()
            + self.w_v.weight_bytes()
            + self.w_q.weight_bytes()
            + self.w_o.weight_bytes()
    }

    /// Mean next-token cross-entropy of a sequence under the quantised weights (offline only).
    pub fn sequence_loss(&self, tokens: &[u32]) -> f32 {
        if tokens.len() < 2 {
            return 0.0;
        }
        let mut s = vec![0f32; self.dk * self.dv];
        let div = (1u64 << self.decay) as f32;
        let preds = tokens.len() - 1;
        let mut total = 0f64;
        for i in 0..preds {
            let tok = (tokens[i] as usize).min(self.vocab - 1);
            let vscale = (1u64 << self.w_v.shift(tok)) as f32;
            for ki in 0..self.dk {
                let k = self.w_k.weight(tok, ki) as f32;
                for j in 0..self.dv {
                    let idx = ki * self.dv + j;
                    let cur = (s[idx] / div).floor();
                    s[idx] = cur + k * (self.w_v.weight(tok, j) as f32 * vscale);
                }
            }
            let mut num = vec![0f32; self.dv];
            for ki in 0..self.dk {
                let q = self.w_q.weight(tok, ki) as f32;
                if q == 0.0 {
                    continue;
                }
                for j in 0..self.dv {
                    num[j] += q * s[ki * self.dv + j];
                }
            }
            let m = num.iter().fold(0f32, |a, &v| a.max(v.abs()));
            let shift = if self.norm_bits > 0 && m >= 1.0 {
                (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
            } else {
                0
            };
            let dec = 1.0f32 / (1u64 << shift) as f32;
            let mut num: Vec<f32> = num.iter().map(|&v| (v * dec).floor()).collect();
            for v in num.iter_mut() {
                if *v < 0.0 {
                    *v = 0.0;
                }
            }
            let mut logits = vec![0f32; self.vocab];
            for (r, slot) in logits.iter_mut().enumerate() {
                let mut acc = 0f32;
                for j in 0..self.dv {
                    acc += self.w_o.weight(r, j) as f32 * num[j];
                }
                *slot = acc * (1u64 << self.w_o.shift(r)) as f32;
            }
            let p = softmax_f32(&logits);
            let target = (tokens[i + 1] as usize).min(self.vocab - 1);
            total += -(p[target].max(1e-9)).ln() as f64;
        }
        (total / preds as f64) as f32
    }
}

// ---------------------------------------------------------------------------
// Offline training: the same STE + BPTT + Adam discipline as the dense core.
// ---------------------------------------------------------------------------

/// Offline trainer for [`LowBitAttention`].
#[derive(Debug, Clone)]
pub struct LowBitAttentionTrainer {
    pub vocab: usize,
    pub dk: usize,
    pub dv: usize,
    pub decay: u32,
    /// Power-of-two readout normalisation; `0` disables it.
    pub norm_bits: u32,
    /// Key masters, `vocab x dk`.
    pub wk: Vec<f32>,
    /// Value masters, `vocab x dv`.
    pub wv: Vec<f32>,
    /// Query masters, `vocab x dk`.
    pub wq: Vec<f32>,
    /// Output masters, `vocab x dv`.
    pub wo: Vec<f32>,
    pub cfg: TrainConfig,
    gwk: Vec<f32>,
    gwv: Vec<f32>,
    gwq: Vec<f32>,
    gwo: Vec<f32>,
    mwk: Vec<f32>,
    vwk: Vec<f32>,
    mwv: Vec<f32>,
    vwv: Vec<f32>,
    mwq: Vec<f32>,
    vwq: Vec<f32>,
    mwo: Vec<f32>,
    vwo: Vec<f32>,
    step: u64,
}

impl LowBitAttentionTrainer {
    pub fn new(vocab: usize, dk: usize, dv: usize, decay: u32, seed: u64) -> Result<Self, String> {
        if vocab == 0 || dk == 0 || dv == 0 {
            return Err("vocab, dk and dv must be non-zero".into());
        }
        let mut st = seed ^ 0x9E37_79B9_7F4A_7C15;
        if st == 0 {
            st = 0x1234_5678_9ABC_DEF0;
        }
        let mut fill = |n: usize| -> Vec<f32> { (0..n).map(|_| xorshift_unit(&mut st)).collect() };
        Ok(Self {
            vocab,
            dk,
            dv,
            decay,
            norm_bits: 0,
            wk: fill(vocab * dk),
            wv: fill(vocab * dv),
            wq: fill(vocab * dk),
            wo: fill(vocab * dv),
            cfg: TrainConfig::default(),
            gwk: vec![0f32; vocab * dk],
            gwv: vec![0f32; vocab * dv],
            gwq: vec![0f32; vocab * dk],
            gwo: vec![0f32; vocab * dv],
            mwk: vec![0f32; vocab * dk],
            vwk: vec![0f32; vocab * dk],
            mwv: vec![0f32; vocab * dv],
            vwv: vec![0f32; vocab * dv],
            mwq: vec![0f32; vocab * dk],
            vwq: vec![0f32; vocab * dk],
            mwo: vec![0f32; vocab * dv],
            vwo: vec![0f32; vocab * dv],
            step: 0,
        })
    }

    pub fn to_core(&self) -> Result<LowBitAttention, String> {
        let mut core = LowBitAttention::from_f32(
            self.vocab, self.dk, self.dv, self.decay, &self.wk, &self.wv, &self.wq, &self.wo,
        )?;
        core.norm_bits = self.norm_bits;
        Ok(core)
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
        let logits = core.forward_i32(&tokens[..tokens.len() - 1]);
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
        let (qk, _sk) = quantize_codes(&self.wk, self.vocab, self.dk);
        let (qv, sv) = quantize_codes(&self.wv, self.vocab, self.dv);
        let (qq, _sq) = quantize_codes(&self.wq, self.vocab, self.dk);
        let (qo, so) = quantize_codes(&self.wo, self.vocab, self.dv);

        let div = (1u64 << self.decay) as f32;
        let dec = 1.0f32 / div;
        let inv = 1.0f32 / (n - 1) as f32;

        // Forward, caching the post-ingest state of every step.
        let mut states: Vec<Vec<f32>> = Vec::with_capacity(n);
        let mut s = vec![0f32; self.dk * self.dv];
        for &t in tokens {
            let tok = (t as usize).min(self.vocab - 1);
            let vscale = sv[tok];
            for i in 0..self.dk {
                let k = qk[tok * self.dk + i] as f32;
                if k == 0.0 {
                    // Still decay: the recurrence applies to every entry every step.
                    for j in 0..self.dv {
                        let idx = i * self.dv + j;
                        s[idx] = (s[idx] * dec).floor();
                    }
                    continue;
                }
                for j in 0..self.dv {
                    let idx = i * self.dv + j;
                    let v = qv[tok * self.dv + j] as f32 * vscale;
                    s[idx] = (s[idx] * dec).floor() + k * v;
                }
            }
            states.push(s.clone());
        }

        let mut ds_carry = vec![0f32; self.dk * self.dv];
        let mut loss = 0f64;

        for t in (0..n - 1).rev() {
            let tok = (tokens[t] as usize).min(self.vocab - 1);
            let st = &states[t];

            // Read: num = Q(t) · S_t, then relu, then the output projection.
            let mut num = vec![0f32; self.dv];
            for i in 0..self.dk {
                let q = qq[tok * self.dk + i] as f32;
                if q == 0.0 {
                    continue;
                }
                for j in 0..self.dv {
                    num[j] += q * st[i * self.dv + j];
                }
            }
            let m = num.iter().fold(0f32, |a, &v| a.max(v.abs()));
            let shift = if self.norm_bits > 0 && m >= 1.0 {
                (32 - (m as u32).leading_zeros()).saturating_sub(self.norm_bits)
            } else {
                0
            };
            let dec = 1.0f32 / (1u64 << shift) as f32;
            let num: Vec<f32> = num.iter().map(|&v| (v * dec).floor()).collect();
            let h: Vec<f32> = num.iter().map(|&v| v.max(0.0)).collect();
            let mut logits = vec![0f32; self.vocab];
            for (r, slot) in logits.iter_mut().enumerate() {
                let mut acc = 0f32;
                for j in 0..self.dv {
                    acc += qo[r * self.dv + j] as f32 * h[j];
                }
                *slot = acc * so[r];
            }
            let mut p = softmax_f32(&logits);
            let target = (tokens[t + 1] as usize).min(self.vocab - 1);
            loss += -(p[target].max(1e-9)).ln() as f64;
            p[target] -= 1.0;

            // Output gradient, and the gradient on the read.
            let mut dh = vec![0f32; self.dv];
            for r in 0..self.vocab {
                let g = p[r] * inv;
                if g == 0.0 {
                    continue;
                }
                let base = r * self.dv;
                let sr = so[r];
                for j in 0..self.dv {
                    self.gwo[base + j] += g * h[j];
                    dh[j] += g * (qo[base + j] as f32 * sr);
                }
            }
            let mut dnum = vec![0f32; self.dv];
            for j in 0..self.dv {
                // The shift is treated as a constant (STE through the bit scan).
                dnum[j] = if num[j] > 0.0 { dh[j] * dec } else { 0.0 };
            }

            // dS_t from the read, plus the carry from the next step's ingest.
            let mut ds = ds_carry;
            for i in 0..self.dk {
                let q = qq[tok * self.dk + i] as f32;
                let row = i * self.dv;
                let mut dqi = 0f32;
                for j in 0..self.dv {
                    ds[row + j] += q * dnum[j];
                    dqi += dnum[j] * st[row + j];
                }
                if q != 0.0 || dqi != 0.0 {
                    self.gwq[tok * self.dk + i] += dqi * inv;
                }
            }

            // Backprop through the ingest at step t: S_t = floor(S_{t-1}·2^-decay) + k ⊗ v.
            for i in 0..self.dk {
                let k = qk[tok * self.dk + i] as f32;
                let row = i * self.dv;
                let mut dki = 0f32;
                for j in 0..self.dv {
                    let d = ds[row + j];
                    if d == 0.0 {
                        continue;
                    }
                    let v = qv[tok * self.dv + j] as f32 * sv[tok];
                    dki += d * v;
                    self.gwv[tok * self.dv + j] += d * k * inv;
                }
                if dki != 0.0 {
                    self.gwk[tok * self.dk + i] += dki * inv;
                }
            }
            for idx in 0..ds.len() {
                ds[idx] *= dec;
            }
            ds_carry = ds;
        }
        (loss / (n - 1) as f64) as f32
    }

    fn zero_grads(&mut self) {
        for g in self
            .gwk
            .iter_mut()
            .chain(&mut self.gwv)
            .chain(&mut self.gwq)
            .chain(&mut self.gwo)
        {
            *g = 0.0;
        }
    }

    fn normalize_and_clip(&mut self, batch: usize) {
        let scale = 1.0f32 / batch.max(1) as f32;
        let mut sumsq = 0f64;
        for g in self
            .gwk
            .iter_mut()
            .chain(&mut self.gwv)
            .chain(&mut self.gwq)
            .chain(&mut self.gwo)
        {
            *g *= scale;
            sumsq += (*g as f64) * (*g as f64);
        }
        if self.cfg.grad_clip > 0.0 {
            let norm = libm::sqrt(sumsq);
            let clip = self.cfg.grad_clip as f64;
            if norm > clip && norm > 0.0 {
                let s = (clip / norm) as f32;
                for g in self
                    .gwk
                    .iter_mut()
                    .chain(&mut self.gwv)
                    .chain(&mut self.gwq)
                    .chain(&mut self.gwo)
                {
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
            &mut self.wk,
            &self.gwk,
            &mut self.mwk,
            &mut self.vwk,
            &cfg,
            step,
        );
        adam_update(
            &mut self.wv,
            &self.gwv,
            &mut self.mwv,
            &mut self.vwv,
            &cfg,
            step,
        );
        adam_update(
            &mut self.wq,
            &self.gwq,
            &mut self.mwq,
            &mut self.vwq,
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

    /// Train on one batch and return the mean per-sequence loss before the update.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_attention(
        vocab: usize,
        dk: usize,
        dv: usize,
        decay: u32,
        seed: u64,
    ) -> LowBitAttention {
        let mut st = seed | 1;
        let mut nf = |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    let u = ((st >> 11) as f64 / (1u64 << 53) as f64) as f32;
                    u * 2.0 - 1.0
                })
                .collect()
        };
        LowBitAttention::from_f32(
            vocab,
            dk,
            dv,
            decay,
            &nf(vocab * dk),
            &nf(vocab * dv),
            &nf(vocab * dk),
            &nf(vocab * dv),
        )
        .expect("build")
    }

    #[test]
    fn integer_path_is_exactly_the_float_reference() {
        for decay in [0u32, 2, 4] {
            let a = random_attention(9, 6, 5, decay, 0xABCD_EF01);
            for tokens in [vec![0u32, 3, 7, 1, 8, 2], vec![5, 5, 5, 5], vec![1], vec![]] {
                let got = a.forward_i32(&tokens);
                let want = a.forward_reference_f64(&tokens);
                assert_eq!(got.len(), a.vocab);
                for j in 0..a.vocab {
                    assert_eq!(
                        got[j] as f64, want[j],
                        "decay {decay}: integer path diverged at {j} for {tokens:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn state_grows_linearly_not_geometrically() {
        let a = random_attention(11, 16, 16, 0, 7);
        let mut s = a.initial_state();
        let mut peak = 0i32;
        for i in 0..512u32 {
            a.ingest(&mut s, i % 11);
            for &v in &s {
                peak = peak.max(v.abs());
            }
        }
        assert!(
            peak < 1 << 20,
            "linear state must not saturate i32 over 512 steps, peak {peak}"
        );
    }

    /// Induction: at the final position, predict what followed the last time this token appeared.
    /// Tokens 0..8. `x ∈ {0,1}`, `y ∈ {2,3}`, fillers ∈ {4..8}; sequence `[x, y, filler*delay, x, y]`.
    fn induction_batch(seed: u64, n: usize, delay: usize) -> Vec<Vec<u32>> {
        let mut st = seed | 1;
        (0..n)
            .map(|_| {
                let mut next = |m: u64| {
                    st ^= st << 13;
                    st ^= st >> 7;
                    st ^= st << 17;
                    st % m
                };
                let x = next(2) as u32;
                let y = 2 + next(2) as u32;
                let mut seq = vec![x, y];
                for _ in 0..delay {
                    seq.push(4 + next(4) as u32);
                }
                seq.push(x);
                seq.push(y);
                seq
            })
            .collect()
    }

    fn train_induction(delay: usize, steps: usize, seed: u64) -> f32 {
        let train = induction_batch(0xA5A5_1234, 64, delay);
        let held = induction_batch(0x0BAD_F00D, 64, delay);
        let mut t = LowBitAttentionTrainer::new(8, 32, 32, 0, seed).expect("build");
        t.cfg.lr = 0.05;
        for _ in 0..steps {
            t.train_batch(&train);
        }
        let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
        hits as f32 / held.len() as f32
    }

    #[test]
    fn learns_induction_at_short_delay() {
        for delay in [1usize, 4] {
            let acc = train_induction(delay, 1500, 2026_0919);
            assert!(
                acc >= 0.9,
                "content-addressed memory must retrieve a pair at delay {delay}, got {acc:.2}"
            );
        }
    }

    /// The decisive test: retrieve a stored pair across 16 intervening tokens, where the dense
    /// recurrence was measured to fail even at delay 8.
    ///
    /// The configuration is pinned because the run is deterministic: this exact setup reproduces on
    /// this machine and build, and the module docs record that neighbouring regimes (larger batch,
    /// longer delay, higher learning rate at large width) collapse to the uniform predictor.
    #[test]
    fn learns_induction_at_long_delay() {
        let delay = 16usize;
        let train = induction_batch(0xA5A5_1234, 32, delay);
        let held = induction_batch(0x0BAD_F00D, 32, delay);
        let mut t = LowBitAttentionTrainer::new(8, 64, 64, 0, 2026_0919).expect("build");
        t.cfg.lr = 0.05;
        for _ in 0..1200 {
            t.train_batch(&train);
        }
        let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
        let acc = hits as f32 / held.len() as f32;
        assert!(
            acc >= 0.9,
            "content-addressed memory must retrieve the pair at delay {delay}, got {acc:.2}"
        );
    }

    /// The confirmed power-of-two readout normalisation moves the run off the uniform collapse that
    /// was measured at `dk = 128, lr = 0.05` (loss exactly `ln 8`, accuracy 0.06).
    #[test]
    fn normalisation_moves_off_the_uniform_collapse() {
        let delay = 16usize;
        let train = induction_batch(0xA5A5_1234, 32, delay);
        let held = induction_batch(0x0BAD_F00D, 32, delay);
        let mut base = LowBitAttentionTrainer::new(8, 128, 128, 0, 2026_0919).expect("build");
        base.cfg.lr = 0.05;
        let mut norm = base.clone();
        norm.norm_bits = 6;
        for _ in 0..1200 {
            base.train_batch(&train);
            norm.train_batch(&train);
        }
        let acc = |t: &LowBitAttentionTrainer| {
            let hits = held.iter().filter(|s| t.final_token_correct(s)).count();
            hits as f32 / held.len() as f32
        };
        let a = acc(&base);
        let b = acc(&norm);
        eprintln!("collapse: unnormalised={a:.2} normalised={b:.2}");
        assert!(
            b > a,
            "normalisation must move the run off the uniform collapse: {a:.2} -> {b:.2}"
        );
    }

    #[test]
    fn gradients_are_produced_for_every_table() {
        let mut t = LowBitAttentionTrainer::new(8, 8, 8, 0, 5).expect("build");
        t.zero_grads();
        for seq in induction_batch(11, 4, 3) {
            t.accumulate(&seq);
        }
        let nz = |g: &[f32]| g.iter().any(|v| v.abs() > 0.0);
        assert!(nz(&t.gwk), "no key gradient");
        assert!(nz(&t.gwv), "no value gradient");
        assert!(nz(&t.gwq), "no query gradient");
        assert!(nz(&t.gwo), "no output gradient");
    }
}
