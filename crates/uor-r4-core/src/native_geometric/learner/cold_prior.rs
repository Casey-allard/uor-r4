//! Always-present learned exact-token local prior plus the causal memory residual.
//!
//! # The question
//!
//! The order-2 addressed memory is cold on most real-text positions: at `V=4096`, `dv=128`, 64-token
//! windows, 93.8% of prose targets read a bucket that was never written, and an unwritten bucket is the
//! zero vector, so the memory-only path assigns uniform logits there. The missing channel is a learned
//! prior over the *current context* that is present whether or not the memory route is live.
//!
//! # The computation
//!
//! At the prediction of `x[t+1]`, with `a` the ordered residue-pair address of `(x[t-1], x[t])`:
//!
//! ```text
//! prior  = bounded_relu(E_old[prev] + E_new[cur])     // prev is the absent marker when t == 0
//! memory = S[a]                                       // exactly zero when the route is unwritten
//! hidden = normalize(prior) + normalize(memory)       // per channel, BEFORE the sum
//! logits = shared_lowbit_readout(hidden) + quantized_bias
//! ```
//!
//! `E_old` and `E_new` are position-specific tables indexed by the **full token id**, so tokens that
//! share a residue are still distinguishable in the prior. `E_old` has one extra row, `pad_row(vocab)`,
//! which is the explicit absent-prefix marker and is outside the real id range (real token `0` is row
//! `0`, and the two are tested to differ).
//!
//! Each channel is normalised **separately** by a power-of-two right shift derived from its own
//! magnitude. Normalising only the sum would let a large memory read erase a small prior.
//!
//! # Fixed-point contract (declared before fitting)
//!
//! * Both channels are integers in one common unit.
//! * `prior` is a bounded ReLU: negative to `0`, clipped above at [`PRIOR_CLAMP`].
//! * Each channel is shifted right by `bitlen(max|v|) - norm_bits` bits, or by `0` when normalization
//!   is disabled (`norm_bits == 0`) or the channel is all zero. The shift is fixed per position, not
//!   learned.
//! * `hidden` is clipped to `[-H_CLAMP, H_CLAMP]`.
//! * `logits[r] = (Σ_j w_o[r][j] * hidden[j]) << shift_o[r] + bias[r]`, so the decoder engages no
//!   multiplier and the bias is an exact integer.
//! * Offline training uses floats and a straight-through estimator; the served form is integer only.
//!   The declared effective-weight surrogate is `w -> s·Q(w/s)` with the scale frozen, whose
//!   derivative is one, so a master's gradient is the effective weight's gradient.
//!
//! This is a hidden-state residual feeding one shared decoder. Adding a residual adds logits and is
//! product-like after the softmax; it is **not** a probability mixture.
//!
//! # What it is not
//!
//! The memory term is still modulo-aliased pair indexing, not exact occurrence/version memory, and it
//! is not group transport. No geometric advantage is claimed for the prior.

#![forbid(unsafe_code)]

use super::group_table::GROUP_ORDER;
use super::lowbit::TernaryLinear;
use super::lowbit_core::{adam_update, quantize_codes, softmax_f32, xorshift_unit, TrainConfig};

/// Address radix: one `2I` element per position.
pub const RADIX: usize = GROUP_ORDER;

/// Upper bound on the bounded ReLU in the prior channel.
pub const PRIOR_CLAMP: i32 = 4096;
/// Upper bound on `|hidden|` after the two channels are summed.
pub const H_CLAMP: i32 = 8192;
/// Table version for export/reload.
pub const FORMAT_VERSION: u32 = 1;
const MAGIC: &[u8; 4] = b"CPR1";

/// Which channels the served path evaluates. Interventions use these; they are not learned gates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channels {
    pub prior: bool,
    pub memory: bool,
}

impl Channels {
    pub const BOTH: Self = Self {
        prior: true,
        memory: true,
    };
    pub const PRIOR_ONLY: Self = Self {
        prior: true,
        memory: false,
    };
    pub const MEMORY_ONLY: Self = Self {
        prior: false,
        memory: true,
    };
    pub const NEITHER: Self = Self {
        prior: false,
        memory: false,
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColdPriorConfig {
    pub vocab: usize,
    pub dv: usize,
    pub norm_bits: u32,
    pub channels: Channels,
}

impl ColdPriorConfig {
    pub fn new(vocab: usize, dv: usize) -> Self {
        Self {
            vocab,
            dv,
            norm_bits: 6,
            channels: Channels::BOTH,
        }
    }

    /// Row index of the absent-prefix marker in `E_old`.
    #[inline]
    pub fn pad_row(&self) -> usize {
        self.vocab
    }

    #[inline]
    pub fn n_addr(&self) -> usize {
        RADIX * RADIX
    }

    #[inline]
    pub fn state_len(&self) -> usize {
        self.n_addr() * self.dv + self.n_addr()
    }
}

#[inline]
fn bounded_relu(x: i32, cap: i32) -> i32 {
    if x < 0 {
        0
    } else if x > cap {
        cap
    } else {
        x
    }
}

#[inline]
fn clamp(x: i32, lo: i32, hi: i32) -> i32 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

/// Power-of-two right shift that keeps about `norm_bits` significant bits. `0` disables normalization.
#[inline]
fn norm_shift(max_abs: i32, norm_bits: u32) -> u32 {
    if norm_bits > 0 && max_abs > 0 {
        (32 - (max_abs as u32).leading_zeros()).saturating_sub(norm_bits)
    } else {
        0
    }
}

/// The served integer form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColdPriorCore {
    pub cfg: ColdPriorConfig,
    /// `2I` element per token id.
    pub elements: Vec<u16>,
    /// `E_old`: `vocab + 1` rows; row `vocab` is the absent-prefix marker.
    pub e_old: TernaryLinear,
    /// `E_new`: `vocab` rows, the current token.
    pub e_new: TernaryLinear,
    /// Memory write values.
    pub w_v: TernaryLinear,
    /// The shared low-bit readout.
    pub w_o: TernaryLinear,
    /// Exported integer bias, in decoder logit units.
    pub bias: Vec<i32>,
}

impl ColdPriorCore {
    #[inline]
    pub fn address(&self, prev: u32, cur: u32) -> usize {
        let p = self.elements[(prev as usize).min(self.cfg.vocab - 1)] as usize;
        let c = self.elements[(cur as usize).min(self.cfg.vocab - 1)] as usize;
        p * RADIX + c
    }

    /// `E_old` row for the predecessor: the absent marker when there is no predecessor.
    #[inline]
    pub fn prev_row(&self, prev: Option<u32>) -> usize {
        match prev {
            Some(t) => (t as usize).min(self.cfg.vocab - 1),
            None => self.cfg.pad_row(),
        }
    }

    pub fn initial_state(&self) -> Vec<i32> {
        vec![0i32; self.cfg.state_len()]
    }

    /// Write `value(next)` at the address of `(prev, cur)`. One selected bucket, adds only.
    pub fn observe(&self, s: &mut [i32], prev: u32, cur: u32, next: u32) {
        let dv = self.cfg.dv;
        let a = self.address(prev, cur);
        let t = (next as usize).min(self.cfg.vocab - 1);
        let sh = self.w_v.shift(t);
        let base = a * dv;
        for j in 0..dv {
            s[base + j] += self.w_v.weight(t, j) << sh;
        }
        s[self.cfg.n_addr() * dv + a] += 1;
    }

    /// The two channels before addition, each already normalised to its own declared budget.
    pub fn channels(
        &self,
        s: &[i32],
        prev: Option<u32>,
        cur: u32,
    ) -> (Vec<i32>, Vec<i32>, u32, u32) {
        let dv = self.cfg.dv;
        let nb = self.cfg.norm_bits;

        let mut p = vec![0i32; dv];
        if self.cfg.channels.prior {
            let pr = self.prev_row(prev);
            let cs = self.e_new.shift((cur as usize).min(self.cfg.vocab - 1));
            let ps = self.e_old.shift(pr);
            for (j, slot) in p.iter_mut().enumerate() {
                let v =
                    (self.e_old.weight(pr, j) << ps) + (self.e_new.weight(cur as usize, j) << cs);
                *slot = bounded_relu(v, PRIOR_CLAMP);
            }
        }
        let p_max = p.iter().fold(0i32, |m, v| m.max(v.abs()));
        let p_shift = norm_shift(p_max, nb);
        if p_shift > 0 {
            for v in p.iter_mut() {
                *v >>= p_shift;
            }
        }

        let mut m = vec![0i32; dv];
        if self.cfg.channels.memory {
            let a = self.address(prev.unwrap_or(0), cur);
            let base = a * dv;
            m.copy_from_slice(&s[base..base + dv]);
        }
        let m_max = m.iter().fold(0i32, |mx, v| mx.max(v.abs()));
        let m_shift = norm_shift(m_max, nb);
        if m_shift > 0 {
            for v in m.iter_mut() {
                *v >>= m_shift;
            }
        }

        (p, m, p_shift, m_shift)
    }

    /// `hidden = prior + memory`, clipped.
    pub fn hidden(&self, s: &[i32], prev: Option<u32>, cur: u32) -> Vec<i32> {
        let (p, m, _, _) = self.channels(s, prev, cur);
        p.iter()
            .zip(&m)
            .map(|(&a, &b)| clamp(a + b, -H_CLAMP, H_CLAMP))
            .collect()
    }

    /// Integer logits: the shared readout over `hidden`, plus the exported integer bias.
    pub fn logits(&self, s: &[i32], prev: Option<u32>, cur: u32) -> Vec<i32> {
        let h = self.hidden(s, prev, cur);
        let mut out = self.w_o.forward_i32(&h);
        for (r, v) in out.iter_mut().enumerate() {
            *v += self.bias[r];
        }
        out
    }

    /// Predict the next token after the whole prefix, feeding each step causally.
    pub fn forward(&self, tokens: &[u32]) -> Vec<i32> {
        let mut s = self.initial_state();
        let mut last = Vec::new();
        for i in 0..tokens.len() {
            if i >= 2 {
                self.observe(&mut s, tokens[i - 2], tokens[i - 1], tokens[i]);
            }
            last = self.logits(
                &s,
                if i == 0 { None } else { Some(tokens[i - 1]) },
                tokens[i],
            );
        }
        last
    }

    /// Bytes of packed weight storage plus shifts, for the touched-bytes accounting.
    pub fn weight_bytes(&self) -> usize {
        let tables = [&self.e_old, &self.e_new, &self.w_v, &self.w_o];
        tables.iter().map(|t| t.weight_bytes()).sum::<usize>()
            + tables.iter().map(|t| t.rows * 4).sum::<usize>()
            + self.bias.len() * 4
    }

    pub fn to_bytes(&self, tokenizer_digest: &[u8; 32], seed: u64) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&(self.cfg.vocab as u32).to_le_bytes());
        out.extend_from_slice(&(self.cfg.dv as u32).to_le_bytes());
        out.extend_from_slice(&self.cfg.norm_bits.to_le_bytes());
        out.extend_from_slice(&(self.cfg.channels.prior as u8).to_le_bytes());
        out.extend_from_slice(&(self.cfg.channels.memory as u8).to_le_bytes());
        out.extend_from_slice(&PRIOR_CLAMP.to_le_bytes());
        out.extend_from_slice(&H_CLAMP.to_le_bytes());
        out.extend_from_slice(&seed.to_le_bytes());
        out.extend_from_slice(tokenizer_digest);
        out.extend_from_slice(&(self.elements.len() as u32).to_le_bytes());
        for e in &self.elements {
            out.extend_from_slice(&e.to_le_bytes());
        }
        for t in [&self.e_old, &self.e_new, &self.w_v, &self.w_o] {
            out.extend_from_slice(&(t.rows as u32).to_le_bytes());
            out.extend_from_slice(&(t.cols as u32).to_le_bytes());
            for r in 0..t.rows {
                out.extend_from_slice(&t.shift(r).to_le_bytes());
            }
            out.extend_from_slice(t.packed());
        }
        for b in &self.bias {
            out.extend_from_slice(&b.to_le_bytes());
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            if *c + n > bytes.len() {
                return Err("truncated artifact".into());
            }
            let s = &bytes[*c..*c + n];
            *c += n;
            Ok(s)
        };
        if take(&mut c, 4)? != MAGIC {
            return Err("bad magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        let i32_at = |c: &mut usize| -> Result<i32, String> {
            Ok(i32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        let version = u32_at(&mut c)?;
        if version != FORMAT_VERSION {
            return Err(format!("unsupported version {version}"));
        }
        let vocab = u32_at(&mut c)? as usize;
        let dv = u32_at(&mut c)? as usize;
        let norm_bits = u32_at(&mut c)?;
        let prior = take(&mut c, 1)?[0] == 1;
        let memory = take(&mut c, 1)?[0] == 1;
        let prior_clamp = i32_at(&mut c)?;
        let h_clamp = i32_at(&mut c)?;
        if prior_clamp != PRIOR_CLAMP || h_clamp != H_CLAMP {
            return Err("artifact clamp convention differs from this build".into());
        }
        let _seed = u64::from_le_bytes(take(&mut c, 8)?.try_into().unwrap());
        let _digest = take(&mut c, 32)?.to_vec();
        let nel = u32_at(&mut c)? as usize;
        let mut elements = Vec::with_capacity(nel);
        for _ in 0..nel {
            elements.push(u16::from_le_bytes(take(&mut c, 2)?.try_into().unwrap()));
        }
        let mut tables = Vec::new();
        for _ in 0..4 {
            let rows = u32_at(&mut c)? as usize;
            let cols = u32_at(&mut c)? as usize;
            let mut shift = Vec::with_capacity(rows);
            for _ in 0..rows {
                shift.push(u32_at(&mut c)?);
            }
            let packed = take(&mut c, (rows * cols).div_ceil(4))?.to_vec();
            tables.push(TernaryLinear::from_packed(packed, shift, rows, cols)?);
        }
        let bias: Vec<i32> = (0..vocab)
            .map(|_| i32_at(&mut c))
            .collect::<Result<_, _>>()?;
        let [w_o, w_v, e_new, e_old] =
            match (tables.pop(), tables.pop(), tables.pop(), tables.pop()) {
                (Some(a), Some(b), Some(d), Some(e)) => [a, b, d, e],
                _ => return Err("missing tables".into()),
            };
        Ok(Self {
            cfg: ColdPriorConfig {
                vocab,
                dv,
                norm_bits,
                channels: Channels { prior, memory },
            },
            elements,
            e_old,
            e_new,
            w_v,
            w_o,
            bias,
        })
    }
}

// ---------------------------------------------------------------------------
// Offline training.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ColdPriorTrainer {
    pub cfg: ColdPriorConfig,
    pub elements: Vec<u16>,
    pub cfg_train: TrainConfig,
    pub step: u64,
    seed: u64,

    // Masters (f32).
    pub e_old: Vec<f32>,
    pub e_new: Vec<f32>,
    pub wv: Vec<f32>,
    pub wo: Vec<f32>,
    pub bias: Vec<f32>,

    // Gradients and Adam state.
    ge_old: Vec<f32>,
    ge_new: Vec<f32>,
    gwv: Vec<f32>,
    gwo: Vec<f32>,
    gbias: Vec<f32>,
    me_old: Vec<f32>,
    me_new: Vec<f32>,
    mwv: Vec<f32>,
    mwo: Vec<f32>,
    mbias: Vec<f32>,
    ve_old: Vec<f32>,
    ve_new: Vec<f32>,
    vwv: Vec<f32>,
    vwo: Vec<f32>,
    vbias: Vec<f32>,

    scratch: Vec<f32>,
    dscratch: Vec<f32>,
    touched: Vec<u32>,
}

impl ColdPriorTrainer {
    pub fn new(cfg: ColdPriorConfig, seed: u64) -> Result<Self, String> {
        if cfg.vocab == 0 || cfg.dv == 0 {
            return Err("vocab and dv must be non-zero".into());
        }
        let mut st = seed ^ 0x9E37_79B9_7F4A_7C15;
        if st == 0 {
            st = 0x1234_5678_9ABC_DEF0;
        }
        let mut fill = |n: usize| -> Vec<f32> { (0..n).map(|_| xorshift_unit(&mut st)).collect() };
        let dv = cfg.dv;
        let v = cfg.vocab;
        Ok(Self {
            elements: (0..v).map(|t| (t % RADIX) as u16).collect(),
            e_old: fill((v + 1) * dv),
            e_new: fill(v * dv),
            wv: fill(v * dv),
            wo: fill(v * dv),
            bias: vec![0.0; v],
            ge_old: vec![0.0; (v + 1) * dv],
            ge_new: vec![0.0; v * dv],
            gwv: vec![0.0; v * dv],
            gwo: vec![0.0; v * dv],
            gbias: vec![0.0; v],
            me_old: vec![0.0; (v + 1) * dv],
            me_new: vec![0.0; v * dv],
            mwv: vec![0.0; v * dv],
            mwo: vec![0.0; v * dv],
            mbias: vec![0.0; v],
            ve_old: vec![0.0; (v + 1) * dv],
            ve_new: vec![0.0; v * dv],
            vwv: vec![0.0; v * dv],
            vwo: vec![0.0; v * dv],
            vbias: vec![0.0; v],
            scratch: vec![0.0; cfg.state_len()],
            dscratch: vec![0.0; cfg.state_len()],
            touched: Vec::new(),
            cfg_train: TrainConfig::default(),
            step: 0,
            seed,
            cfg,
        })
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Quantise the masters into the served core. Mirrors the served computation exactly.
    pub fn to_core(&self) -> Result<ColdPriorCore, String> {
        let v = self.cfg.vocab;
        let dv = self.cfg.dv;
        let e_old = TernaryLinear::quantize(&self.e_old, v + 1, dv);
        let e_new = TernaryLinear::quantize(&self.e_new, v, dv);
        let w_v = TernaryLinear::quantize(&self.wv, v, dv);
        let w_o = TernaryLinear::quantize(&self.wo, v, dv);
        // The bias is exported in decoder logit units; it is quantised to whole logit units so the
        // served value is an exact integer the artifact can carry.
        let bias: Vec<i32> = (0..v).map(|r| self.bias[r].round() as i32).collect();
        Ok(ColdPriorCore {
            cfg: self.cfg.clone(),
            elements: self.elements.clone(),
            e_old,
            e_new,
            w_v,
            w_o,
            bias,
        })
    }

    fn zero_grads(&mut self) {
        for g in self
            .ge_old
            .iter_mut()
            .chain(&mut self.ge_new)
            .chain(&mut self.gwv)
            .chain(&mut self.gwo)
        {
            *g = 0.0;
        }
        for g in self.gbias.iter_mut() {
            *g = 0.0;
        }
    }

    /// One window: forward, cross-entropy, backward, returning the mean loss over scored targets.
    ///
    /// Causality is preserved: at step `i` the write of `tokens[i]` happens before the read that
    /// predicts `tokens[i+1]`, and no later token is ever read.
    pub fn accumulate(&mut self, tokens: &[u32]) -> f32 {
        let n = tokens.len();
        if n < 3 {
            return 0.0;
        }
        let v = self.cfg.vocab;
        let dv = self.cfg.dv;
        let nb = self.cfg.norm_bits;
        let use_mem = self.cfg.channels.memory;
        let use_prior = self.cfg.channels.prior;
        let cbase = self.cfg.n_addr() * dv;
        let inv = 1.0f32 / (n - 2) as f32;

        let (qeo, seo) = quantize_codes(&self.e_old, v + 1, dv);
        let (qen, sen) = quantize_codes(&self.e_new, v, dv);
        let (qv, sv) = quantize_codes(&self.wv, v, dv);
        let (qo, so) = quantize_codes(&self.wo, v, dv);

        // Reset only the buckets the previous window touched.
        for k in 0..self.touched.len() {
            let a = self.touched[k] as usize;
            let base = a * dv;
            for j in 0..dv {
                self.scratch[base + j] = 0.0;
                self.dscratch[base + j] = 0.0;
            }
            self.scratch[cbase + a] = 0.0;
        }
        self.touched.clear();

        // Forward. Keep per-position what the backward needs.
        struct Pos {
            p_shift: u32,
            m_shift: u32,
            h: Vec<f32>,
            p_mask: Vec<f32>,
            h_mask: Vec<f32>,
            read_a: usize,
            prev_row: usize,
            cur: usize,
        }
        let mut positions: Vec<Pos> = Vec::with_capacity(n - 2);
        for i in 0..(n - 1) {
            if use_mem && i >= 2 {
                let a = self.address(tokens[i - 2], tokens[i - 1]);
                if self.scratch[cbase + a] == 0.0 {
                    self.touched.push(a as u32);
                }
                let t = tokens[i] as usize;
                let base = a * dv;
                for j in 0..dv {
                    self.scratch[base + j] += qv[t * dv + j] as f32 * sv[t];
                }
                self.scratch[cbase + a] += 1.0;
            }
            let cur = (tokens[i] as usize) % v;
            let prev_row = if i == 0 {
                v
            } else {
                tokens[i - 1] as usize % v
            };

            let mut p_raw = vec![0f32; dv];
            if use_prior {
                for j in 0..dv {
                    p_raw[j] = qeo[prev_row * dv + j] as f32 * seo[prev_row]
                        + qen[cur * dv + j] as f32 * sen[cur];
                }
            }
            // bounded_relu with its STE mask.
            let mut p_mask = vec![0f32; dv];
            let mut p = vec![0f32; dv];
            for j in 0..dv {
                let x = p_raw[j];
                if x <= 0.0 {
                    p[j] = 0.0;
                    p_mask[j] = 0.0;
                } else if x >= PRIOR_CLAMP as f32 {
                    p[j] = PRIOR_CLAMP as f32;
                    p_mask[j] = 0.0;
                } else {
                    p[j] = x;
                    p_mask[j] = 1.0;
                }
            }
            let p_max = p.iter().fold(0f32, |m, x| m.max(x.abs()));
            let p_shift = norm_shift(p_max as i32, nb);
            let p_dec = 1.0f32 / (1u64 << p_shift) as f32;
            for x in p.iter_mut() {
                *x = (*x * p_dec).floor();
            }

            let read_a = self.address(if i == 0 { 0 } else { tokens[i - 1] }, tokens[i]);
            let mut m = vec![0f32; dv];
            if use_mem {
                m.copy_from_slice(&self.scratch[read_a * dv..read_a * dv + dv]);
            }
            let m_max = m.iter().fold(0f32, |mx, x| mx.max(x.abs()));
            let m_shift = norm_shift(m_max as i32, nb);
            let m_dec = 1.0f32 / (1u64 << m_shift) as f32;
            for x in m.iter_mut() {
                *x = (*x * m_dec).floor();
            }

            let mut h = vec![0f32; dv];
            let mut h_mask = vec![0f32; dv];
            for j in 0..dv {
                let raw = p[j] + m[j];
                if raw <= -(H_CLAMP as f32) {
                    h[j] = -(H_CLAMP as f32);
                    h_mask[j] = 0.0;
                } else if raw >= H_CLAMP as f32 {
                    h[j] = H_CLAMP as f32;
                    h_mask[j] = 0.0;
                } else {
                    h[j] = raw;
                    h_mask[j] = 1.0;
                }
            }
            positions.push(Pos {
                p_shift,
                m_shift,
                h,
                p_mask,
                h_mask,
                read_a,
                prev_row,
                cur,
            });
        }

        // Backward. The memory state is a pure accumulator, so a write's gradient is the sum of the
        // read gradients at its bucket over *later* steps. The pass therefore runs in reverse and
        // distributes each write only after every later read has been accumulated.
        let mut loss = 0f64;
        for i in (0..(n - 2)).rev() {
            let pos = &positions[i];
            let mut logits = vec![0f32; v];
            for r in 0..v {
                let mut acc = 0f32;
                for j in 0..dv {
                    acc += qo[r * dv + j] as f32 * pos.h[j];
                }
                logits[r] = acc * so[r] + self.bias[r];
            }
            let mut p = softmax_f32(&logits);
            let target = (tokens[i + 1] as usize) % v;
            loss += -(p[target].max(1e-9)).ln() as f64;
            p[target] -= 1.0;
            let g: Vec<f32> = p.iter().map(|x| x * inv).collect();

            let mut dh = vec![0f32; dv];
            for r in 0..v {
                let gr = g[r];
                if gr == 0.0 {
                    continue;
                }
                self.gbias[r] += gr;
                let rbase = r * dv;
                for j in 0..dv {
                    self.gwo[rbase + j] += gr * pos.h[j];
                    dh[j] += gr * (qo[rbase + j] as f32 * so[r]);
                }
            }
            // Hidden clamp, then the two channels.
            for j in 0..dv {
                dh[j] *= pos.h_mask[j];
            }
            // Memory channel: the gradient w.r.t. the pre-normalisation bucket value.
            if use_mem {
                let m_dec = 1.0f32 / (1u64 << pos.m_shift) as f32;
                for j in 0..dv {
                    self.dscratch[pos.read_a * dv + j] += dh[j] * m_dec;
                }
                self.touched.push(pos.read_a as u32);
            }
            // Prior channel: through the bounded-ReLU mask, then the effective-weight surrogate.
            if use_prior {
                let p_dec = 1.0f32 / (1u64 << pos.p_shift) as f32;
                for j in 0..dv {
                    let d = dh[j] * pos.p_mask[j] * p_dec;
                    self.ge_old[pos.prev_row * dv + j] += d;
                    self.ge_new[pos.cur * dv + j] += d;
                }
            }
            // The write made at this step now sees exactly the reads accumulated so far, which are the
            // reads at steps `k >= i`. The write at step `i` precedes the read at step `i`, so a read at
            // step `i` at the same address does count.
            if use_mem && i >= 2 {
                let wa = self.address(tokens[i - 2], tokens[i - 1]);
                let t = (tokens[i] as usize) % v;
                let wbase = wa * dv;
                for j in 0..dv {
                    self.gwv[t * dv + j] += self.dscratch[wbase + j];
                }
            }
        }

        (loss / (n - 2) as f64) as f32
    }

    fn normalize_and_clip(&mut self, batch: usize) {
        let scale = 1.0f32 / batch.max(1) as f32;
        let mut sumsq = 0f64;
        for g in self
            .ge_old
            .iter_mut()
            .chain(&mut self.ge_new)
            .chain(&mut self.gwv)
            .chain(&mut self.gwo)
        {
            *g *= scale;
            sumsq += (*g as f64) * (*g as f64);
        }
        for g in self.gbias.iter_mut() {
            *g *= scale;
        }
        if self.cfg_train.grad_clip > 0.0 {
            let norm = libm::sqrt(sumsq);
            let clip = self.cfg_train.grad_clip as f64;
            if norm > clip && norm > 0.0 {
                let s = (clip / norm) as f32;
                for g in self
                    .ge_old
                    .iter_mut()
                    .chain(&mut self.ge_new)
                    .chain(&mut self.gwv)
                    .chain(&mut self.gwo)
                {
                    *g *= s;
                }
            }
        }
    }

    /// One optimizer step over a batch of windows.
    pub fn train_batch(&mut self, batch: &[Vec<u32>]) -> f32 {
        if batch.is_empty() {
            return 0.0;
        }
        self.zero_grads();
        let mut loss = 0.0f32;
        for seq in batch {
            loss += self.accumulate(seq);
        }
        self.normalize_and_clip(batch.len());
        self.step += 1;
        let cfg = self.cfg_train.clone();
        let step = self.step;
        adam_update(
            &mut self.e_old,
            &self.ge_old,
            &mut self.me_old,
            &mut self.ve_old,
            &cfg,
            step,
        );
        adam_update(
            &mut self.e_new,
            &self.ge_new,
            &mut self.me_new,
            &mut self.ve_new,
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
            &mut self.wo,
            &self.gwo,
            &mut self.mwo,
            &mut self.vwo,
            &cfg,
            step,
        );
        adam_update(
            &mut self.bias,
            &self.gbias,
            &mut self.mbias,
            &mut self.vbias,
            &cfg,
            step,
        );
        loss / batch.len() as f32
    }

    /// Mean cross-entropy on a held-out window under the *quantised* served weights.
    pub fn loss(&self, core: &ColdPriorCore, tokens: &[u32]) -> f32 {
        let n = tokens.len();
        if n < 3 {
            return 0.0;
        }
        let mut s = core.initial_state();
        let mut total = 0f64;
        for i in 0..(n - 2) {
            if i >= 2 {
                core.observe(&mut s, tokens[i - 2], tokens[i - 1], tokens[i]);
            }
            let prev = if i == 0 { None } else { Some(tokens[i - 1]) };
            let logits = core.logits(&s, prev, tokens[i]);
            let lf: Vec<f32> = logits.iter().map(|&x| x as f32).collect();
            let p = softmax_f32(&lf);
            let target = (tokens[i + 1] as usize) % core.cfg.vocab;
            total += -(p[target].max(1e-9)).ln() as f64;
        }
        (total / (n - 2) as f64) as f32
    }

    #[inline]
    fn address(&self, prev: u32, cur: u32) -> usize {
        let p = self.elements[(prev as usize).min(self.cfg.vocab - 1)] as usize;
        let c = self.elements[(cur as usize).min(self.cfg.vocab - 1)] as usize;
        p * RADIX + c
    }

    /// Distinct successors counted per written bucket in the last `accumulate`, for diagnostics.
    pub fn touched_buckets(&self) -> usize {
        self.touched.len()
    }

    /// A cheap diagnostic: the mean absolute gradient of the two new tables before clipping.
    pub fn prior_grad_norm(&self) -> f32 {
        let s: f64 = self
            .ge_old
            .iter()
            .chain(&self.ge_new)
            .map(|g| (*g as f64) * (*g as f64))
            .sum();
        libm::sqrt(s) as f32
    }
}

/// A control: the best constant (unigram) predictor on the same scored targets, as a cross-entropy.
pub fn unigram_ce(
    counts: &std::collections::HashMap<u32, u64>,
    total: u64,
    vocab: usize,
    targets: &[u32],
) -> f32 {
    let mut bits = 0f64;
    for &t in targets {
        let c = counts.get(&t).copied().unwrap_or(0) as f64;
        let p = (c + 1.0) / (total as f64 + vocab as f64);
        bits += -(p.ln());
    }
    (bits / targets.len().max(1) as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(vocab: usize, dv: usize) -> ColdPriorConfig {
        ColdPriorConfig::new(vocab, dv)
    }

    /// A trainer whose tables are set to a deterministic pattern with magnitude `amp`.
    fn trainer(
        vocab: usize,
        dv: usize,
        channels: Channels,
        amp: f32,
        seed: u64,
    ) -> ColdPriorTrainer {
        let mut c = cfg(vocab, dv);
        c.channels = channels;
        let mut t = ColdPriorTrainer::new(c, seed).expect("build");
        for r in 0..(vocab + 1) {
            for j in 0..dv {
                t.e_old[r * dv + j] = if (r + j) % 2 == 0 { amp } else { -amp };
            }
        }
        for r in 0..vocab {
            for j in 0..dv {
                t.e_new[r * dv + j] = if (r * 3 + j) % 4 == 0 { amp } else { -amp };
                t.wv[r * dv + j] = if (r + 2 * j) % 3 == 0 { amp } else { -amp };
                t.wo[r * dv + j] = if (r * 2 + j) % 5 == 0 { amp } else { -amp };
            }
        }
        t
    }

    #[test]
    fn absent_marker_is_a_distinct_row_from_real_token_zero() {
        let c = cfg(4096, 8);
        assert_eq!(
            c.pad_row(),
            4096,
            "the marker must lie outside the real id range"
        );
        assert_ne!(c.pad_row(), 0, "real token zero is row 0, not the marker");
        let core = ColdPriorTrainer::new(c, 1)
            .expect("build")
            .to_core()
            .expect("core");
        assert_eq!(core.e_old.rows, 4097, "E_old carries the extra marker row");
        assert_eq!(core.e_new.rows, 4096);
        assert_eq!(core.prev_row(None), 4096);
        assert_eq!(core.prev_row(Some(0)), 0);
    }

    #[test]
    fn distinct_full_ids_sharing_a_residue_differ_in_the_prior() {
        // Tokens 1 and 121 share residue 1 mod 120. Full-id rows must separate them.
        assert_eq!(1 % RADIX, 121 % RADIX);
        let t = trainer(256, 8, Channels::NEITHER, 1.0, 7);
        let core = t.to_core().expect("core");
        let mut s = core.initial_state();
        let a = core.logits(&s, Some(1), 5);
        let b = core.logits(&s, Some(121), 5);
        // With priors disabled the residue pair is identical, so they must agree...
        assert_eq!(
            a, b,
            "modulo-aliased memory cannot separate equal-residue predecessors"
        );
        // ...and with the prior on they must not.
        let mut c2 = cfg(256, 8);
        c2.channels = Channels::PRIOR_ONLY;
        let mut t2 = ColdPriorTrainer::new(c2, 7).expect("build");
        for j in 0..8 {
            t2.e_old[1 * 8 + j] = 1.0;
            t2.e_old[121 * 8 + j] = -1.0;
        }
        let core2 = t2.to_core().expect("core");
        let mut s2 = core2.initial_state();
        assert_ne!(
            core2.logits(&mut s2, Some(1), 5),
            core2.logits(&mut s2, Some(121), 5),
            "full-id prior rows must distinguish equal-modulo predecessors"
        );
        let _ = &mut s;
    }

    #[test]
    fn order_reversal_changes_the_address_and_the_prediction() {
        let t = trainer(64, 8, Channels::BOTH, 1.0, 11);
        let core = t.to_core().expect("core");
        assert_ne!(core.address(3, 9), core.address(9, 3));
        let mut s = core.initial_state();
        core.observe(&mut s, 3, 9, 17);
        core.observe(&mut s, 9, 3, 21);
        assert_ne!(core.logits(&s, Some(3), 9), core.logits(&s, Some(9), 3));
    }

    #[test]
    fn a_future_token_cannot_change_an_earlier_prediction() {
        let t = trainer(64, 16, Channels::BOTH, 1.0, 13);
        let core = t.to_core().expect("core");
        let base = [2u32, 7, 1, 4, 0, 3];
        let mut changed = base;
        changed[5] = 9;
        for i in 0..5usize {
            assert_eq!(
                core.forward(&base[..=i]),
                core.forward(&changed[..=i]),
                "prediction at {i} must not see a later token"
            );
        }
    }

    #[test]
    fn state_is_reset_between_windows() {
        let mut t = trainer(32, 8, Channels::BOTH, 1.0, 17);
        let a: Vec<u32> = vec![1, 2, 3, 1, 2, 3];
        let b: Vec<u32> = vec![3, 2, 1, 3, 2, 1];
        let first = t.accumulate(&a);
        let _ = t.accumulate(&b);
        let again = t.accumulate(&a);
        assert!((first - again).abs() < 1e-6, "windows must not share state");
    }

    #[test]
    fn same_tail_with_changed_earlier_writes_is_distinguishable() {
        let t = trainer(32, 8, Channels::MEMORY_ONLY, 1.0, 19);
        let core = t.to_core().expect("core");
        let a = [0u32, 1, 2, 0, 1];
        let b = [0u32, 1, 3, 0, 1];
        assert_eq!(core.address(a[3], a[4]), core.address(b[3], b[4]));
        assert_ne!(
            core.forward(&a),
            core.forward(&b),
            "earlier writes must separate histories sharing a query pair"
        );
    }

    #[test]
    fn a_repeated_write_can_produce_a_negative_read() {
        let mut c = cfg(32, 4);
        c.channels = Channels::MEMORY_ONLY;
        let mut t = ColdPriorTrainer::new(c, 23).expect("build");
        // Row for token 5 is entirely negative, so repeated writes accumulate negative values.
        for j in 0..4 {
            t.wv[5 * 4 + j] = -3.0;
        }
        let core = t.to_core().expect("core");
        let mut s = core.initial_state();
        core.observe(&mut s, 1, 2, 5);
        core.observe(&mut s, 1, 2, 5);
        let a = core.address(1, 2);
        assert!(
            s[a * 4..a * 4 + 4].iter().all(|&v| v < 0),
            "repeated negative writes must read back negative"
        );
        // A different, unwritten route reads exactly zero.
        let s2 = core.initial_state();
        let (p, m, _, _) = core.channels(&s2, Some(1), 2);
        let _ = p;
        assert!(
            m.iter().all(|&v| v == 0),
            "an unwritten route must read exactly zero"
        );
    }

    #[test]
    fn cold_positions_agree_with_memory_disabled() {
        // The required invariant: where nothing was written the memory channel is zero, so the full
        // model and the memory-disabled intervention must agree exactly.
        let t = trainer(64, 8, Channels::BOTH, 1.0, 29);
        let full = t.to_core().expect("core");
        let mut c = cfg(64, 8);
        c.channels = Channels::PRIOR_ONLY;
        let t2 = {
            let mut x = trainer(64, 8, Channels::PRIOR_ONLY, 1.0, 29);
            x.cfg = c;
            x
        };
        let prior_only = t2.to_core().expect("core");
        // A non-repeating sequence has no live route anywhere.
        let seq = [3u32, 9, 21, 40, 55, 62];
        let mut s_full = full.initial_state();
        let mut s_po = prior_only.initial_state();
        for i in 0..seq.len() - 1 {
            if i >= 2 {
                full.observe(&mut s_full, seq[i - 2], seq[i - 1], seq[i]);
                prior_only.observe(&mut s_po, seq[i - 2], seq[i - 1], seq[i]);
            }
            let prev = if i == 0 { None } else { Some(seq[i - 1]) };
            assert_eq!(
                full.logits(&s_full, prev, seq[i]),
                prior_only.logits(&s_po, prev, seq[i]),
                "cold position {i} must not differ when memory is disabled"
            );
        }
    }

    #[test]
    fn hidden_and_prior_are_bounded() {
        let t = trainer(32, 8, Channels::BOTH, 30.0, 31);
        let core = t.to_core().expect("core");
        let mut s = core.initial_state();
        for _ in 0..40 {
            core.observe(&mut s, 1, 2, 3);
        }
        let (p, m, _, _) = core.channels(&s, Some(1), 2);
        // Each channel is normalised to its own budget, so neither is erased by the other.
        assert!(
            p.iter().all(|v| v.abs() < (1 << 8)),
            "prior channel must be normalised"
        );
        assert!(
            m.iter().any(|v| *v != 0),
            "memory channel must survive normalisation"
        );
        let h = core.hidden(&s, Some(1), 2);
        assert!(
            h.iter().all(|v| v.abs() <= H_CLAMP),
            "hidden must be clamped"
        );
    }

    #[test]
    fn export_and_reload_are_integer_exact() {
        let t = trainer(128, 16, Channels::BOTH, 3.0, 37);
        let core = t.to_core().expect("core");
        let digest = [7u8; 32];
        let bytes = core.to_bytes(&digest, t.seed());
        let back = ColdPriorCore::from_bytes(&bytes).expect("reload");
        assert_eq!(back, core, "shipped and reloaded cores must be identical");
        let seq = [5u32, 12, 5, 12, 99, 5];
        assert_eq!(
            core.forward(&seq),
            back.forward(&seq),
            "integer logit parity"
        );
    }

    #[test]
    fn the_surrogate_gradient_reaches_both_new_tables() {
        let mut t = trainer(64, 8, Channels::BOTH, 2.0, 41);
        // Make the prior strictly positive so the bounded-ReLU mask is open at every entry; otherwise a
        // sign-cancelling fixture yields a legitimately zero prior gradient and tests nothing about the
        // surrogate.
        t.e_old.iter_mut().for_each(|w| *w = 2.0);
        t.e_new.iter_mut().for_each(|w| *w = 2.0);
        assert_eq!(t.prior_grad_norm(), 0.0, "gradients start cleared");
        let loss = t.accumulate(&[1, 4, 1, 4, 1, 4, 7, 4]);
        assert!(loss.is_finite() && loss > 0.0);
        assert!(
            t.prior_grad_norm() > 0.0,
            "the prior tables must receive gradient"
        );
        assert!(
            t.gwo.iter().any(|g| *g != 0.0),
            "the shared readout must receive gradient"
        );
        assert!(
            t.gwv.iter().any(|g| *g != 0.0),
            "the memory write table must receive gradient on a live route"
        );
    }

    #[test]
    fn training_reduces_loss_on_a_repeating_context() {
        // A tiny but real learning check: the prior alone must be able to fit a repeating pattern.
        let mut c = cfg(64, 16);
        c.channels = Channels::PRIOR_ONLY;
        let mut t = ColdPriorTrainer::new(c, 43).expect("build");
        t.cfg_train.lr = 0.05;
        let seq: Vec<u32> = (0..64).map(|i| if i % 2 == 0 { 3 } else { 11 }).collect();
        let batch = vec![seq.clone(), seq.clone()];
        let first = t.train_batch(&batch);
        for _ in 0..40 {
            t.train_batch(&batch);
        }
        let core = t.to_core().expect("core");
        let after = t.loss(&core, &seq);
        assert!(
            after < first,
            "the quantised prior-only model must improve: {first} -> {after}"
        );
    }

    #[test]
    fn all_four_interventions_run() {
        for channels in [
            Channels::BOTH,
            Channels::PRIOR_ONLY,
            Channels::MEMORY_ONLY,
            Channels::NEITHER,
        ] {
            let t = trainer(32, 8, channels, 1.0, 47);
            let core = t.to_core().expect("core");
            let out = core.forward(&[1, 2, 3, 1, 2, 3]);
            assert_eq!(out.len(), 32);
        }
        // Bias-only: neither channel, so logits are exactly the exported bias — the constant control.
        let t = trainer(32, 8, Channels::NEITHER, 1.0, 47);
        let core = t.to_core().expect("core");
        assert_eq!(core.forward(&[1, 2, 3]), core.bias);
    }
}
