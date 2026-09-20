//! Prior-only learned contextual residual above a **frozen** low-bit unigram bias.
//!
//! # The contract, and the defect this repairs
//!
//! There is exactly **one** predictive forward. Training does not re-derive the score; it calls
//! [`PriorCore::trace`], the same integer computation greedy serving uses:
//!
//! ```text
//! Z[r] = (Σ_j W[r][j] · h[j]) + B[r]      // one integer score, shared by training and serving
//! B[r] = bias_code[r] · 2^bias_scale_bits  // frozen, signed <=4-bit code, declared dyadic scale
//! z[r] = Z[r] · 2^-F                       // offline probability logits, one artifact-bound F
//! bits  = (logsumexp(z) - z[target]) / ln 2
//! ```
//!
//! An earlier version kept a second, duplicated floating training forward that computed
//! `R + b·2^(s−F)` — at `F = 10` the contextual term was 1,024× the served value — while the backward
//! applied `2^-F` anyway. Zero output codes hid it at initialization. That parallel definition is
//! gone; [`tests::training_and_serving_scores_are_the_same_computation`] fails if it returns.
//!
//! `2^-F/ln 2` is applied **once**, at credit to the integer score, and the batch mean divides by the
//! actual number of scored targets once. Greedy serving is `argmax(Z)` with no scale multiply.
//!
//! # Supervised targets
//!
//! Real-text scoring is **all** `n−1` next-token predictions ([`targets`]). A fixture may declare an
//! explicit per-position mask ([`targets_masked`]) shared by training, loss and accuracy; the
//! balanced fixture scores only the final position, because scoring the whole triple trains targets
//! that contradict each other (see [`targets_masked`] and the fixture tests).

#![forbid(unsafe_code)]

use super::group_table::GROUP_ORDER;
use super::lowbit::TernaryLinear;
use super::lowbit_core::{adam_update, xorshift_unit, TrainConfig};

pub const RADIX: usize = GROUP_ORDER;
/// Upper bound on the bounded ReLU, in prior-channel units.
pub const PRIOR_CLAMP: i32 = 4096;
/// Artifact format version for this contract.
pub const FORMAT_VERSION: u32 = 2;
const MAGIC: &[u8; 4] = b"CPL2";
/// Common offset for the frozen bias code. Declared, not tuned on development.
pub const BIAS_OFFSET: f64 = 7.0;
/// Bias codes are `round(ln p + c)`, clamped to a signed 4-bit range.
pub const BIAS_CODE_MAX: i32 = 7;
/// Largest vocabulary this module will load or construct.
pub const MAX_VOCAB: usize = 1 << 20;
/// Largest value width this module will load or construct.
pub const MAX_DV: usize = 1 << 14;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub vocab: usize,
    pub dv: usize,
    pub norm_bits: u32,
    /// One artifact-bound exponent: `z = Z · 2^-F`.
    pub f_bits: u32,
    /// Dyadic scale of the frozen bias: `B = code · 2^bias_scale_bits`.
    pub bias_scale_bits: u32,
}

impl Config {
    pub fn new(vocab: usize, dv: usize) -> Self {
        Self {
            vocab,
            dv,
            norm_bits: 6,
            f_bits: 10,
            bias_scale_bits: 10,
        }
    }

    #[inline]
    pub fn pad_row(&self) -> usize {
        self.vocab
    }

    /// Declared envelope. A shift bound alone is not an overflow bound: `7 << 30` does not fit i32.
    pub fn validate(&self) -> Result<(), String> {
        if self.vocab < 2 || self.vocab > MAX_VOCAB {
            return Err(format!("vocab {} outside 2..={MAX_VOCAB}", self.vocab));
        }
        if self.dv == 0 || self.dv > MAX_DV {
            return Err(format!("dv {} outside 1..={MAX_DV}", self.dv));
        }
        if self.norm_bits > 30 {
            return Err("norm_bits <= 30".into());
        }
        if self.f_bits > 30 {
            return Err("f_bits <= 30".into());
        }
        // The bias is `code · 2^s` with |code| <= 7; require the product to fit i32 with headroom.
        if self.bias_scale_bits > 27 {
            return Err(format!(
                "bias_scale_bits {} cannot represent |code| <= 7 in i32",
                self.bias_scale_bits
            ));
        }
        // `norm_bits = 0` disables the shift, so the hidden value is bounded by PRIOR_CLAMP rather
        // than by a unit bound; validation must use the bound the forward actually has.
        let h_max = if self.norm_bits == 0 {
            PRIOR_CLAMP as i64
        } else {
            1i64 << self.norm_bits
        };
        let acc = (self.dv as i64)
            .checked_mul(h_max)
            .ok_or("dv · 2^norm_bits overflow")?;
        if acc > (i32::MAX as i64) / 4 {
            return Err(format!(
                "dv · 2^norm_bits = {acc} exceeds the declared accumulator envelope"
            ));
        }
        Ok(())
    }

    /// Per-row readout bound for a declared row shift, checked against the i32 envelope.
    fn check_row_shift(&self, shift: u32) -> Result<(), String> {
        let h_max = if self.norm_bits == 0 {
            PRIOR_CLAMP as i64
        } else {
            1i64 << self.norm_bits
        };
        let acc = (self.dv as i64) * h_max;
        let bound = acc
            .checked_shl(shift)
            .ok_or_else(|| format!("row shift {shift} overflows the bound"))?;
        if bound > (i32::MAX as i64) / 4 {
            return Err(format!(
                "row shift {shift} makes the readout bound {bound}, outside the i32 envelope"
            ));
        }
        Ok(())
    }
}

#[inline]
fn norm_shift(max_abs: i32, norm_bits: u32) -> u32 {
    if norm_bits > 0 && max_abs > 0 {
        (32 - (max_abs as u32).leading_zeros()).saturating_sub(norm_bits)
    } else {
        0
    }
}

/// The exact integer score and the intermediate trace for one prediction.
///
/// The trace is what the backward pass consumes, so the gradient is the declared surrogate of the
/// *served* computation rather than of a parallel one.
#[derive(Clone, Debug)]
pub struct Trace {
    /// Exact integer scores `Z`.
    pub z: Vec<i32>,
    /// Normalised hidden vector `h`, integer.
    pub h: Vec<i32>,
    /// Bounded-ReLU STE mask for `h`: 1 where the derivative passes through. Stored as `u8` so the
    /// greedy path carries no floating-point auxiliary value.
    pub mask: Vec<u8>,
    /// Right shift applied to `h`. Stored as an integer; the offline backward converts it once.
    pub shift: u32,
    pub prev: usize,
    pub cur: usize,
    /// False when the contextual residual was disabled, so `z` is the frozen bias only.
    pub used_context: bool,
}

/// The single target iterator for real-text scoring: prediction position `i` over `0..n-1`, context
/// ending at `tokens[i]`, target `tokens[i+1]`, absent-prefix row at `i == 0`.
pub fn targets(
    tokens: &[u32],
    vocab: usize,
) -> impl Iterator<Item = (usize, usize, usize, u32)> + '_ {
    targets_masked(tokens, vocab, None)
}

/// The same iterator restricted by an explicit per-position mask shared by training, loss and
/// accuracy. `mask` has length `n-1`; `None` scores every position.
///
/// A mask exists because a supervised fixture can author contradictory labels. Scoring the whole
/// triple `[x, y, (x+y)%4]` trains `(x,y) −> y`, which for a fixed `x` has four equally likely
/// targets, and rotating the triple introduces `b−a` and `a−b` labels that conflict with `a+b`. That
/// is a task defect, not a quantization result, so the fixture scores only the final position.
pub fn targets_masked<'a>(
    tokens: &'a [u32],
    vocab: usize,
    mask: Option<&'a [bool]>,
) -> impl Iterator<Item = (usize, usize, usize, u32)> + 'a {
    let pad = vocab;
    (0..tokens.len().saturating_sub(1))
        .filter(move |i| {
            mask.map(|m| m.get(*i).copied().unwrap_or(false))
                .unwrap_or(true)
        })
        .map(move |i| {
            let prev = if i == 0 {
                pad
            } else {
                (tokens[i - 1] as usize).min(vocab - 1)
            };
            let cur = (tokens[i] as usize).min(vocab - 1);
            (i, prev, cur, tokens[i + 1])
        })
}

/// The fixture mask: score only the final position of a `[x, y, (x+y)%4]` triple.
pub fn fixture_mask(n: usize) -> Vec<bool> {
    let mut m = vec![false; n.saturating_sub(1)];
    if let Some(last) = m.last_mut() {
        *last = true;
    }
    m
}

/// The served integer form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorCore {
    pub cfg: Config,
    pub elements: Vec<u16>,
    /// `vocab + 1` rows; row `vocab` is the absent-prefix marker.
    pub e_old: TernaryLinear,
    pub e_new: TernaryLinear,
    pub w_o: TernaryLinear,
    /// Frozen signed <=4-bit bias codes.
    pub bias_codes: Vec<i8>,
}

impl PriorCore {
    /// **The** predictive forward. Training, evaluation and greedy serving all come through here.
    pub fn trace(&self, prev: usize, cur: usize, use_context: bool) -> Trace {
        let v = self.cfg.vocab;
        let dv = self.cfg.dv;
        let mut z: Vec<i32> = (0..v)
            .map(|r| (self.bias_codes[r] as i32) << self.cfg.bias_scale_bits)
            .collect();
        let mut h = vec![0i32; dv];
        let mut mask = vec![0u8; dv];
        let mut shift = 0u32;
        if use_context {
            let ps = self.e_old.shift(prev);
            let cs = self.e_new.shift(cur);
            let mut mask_in = vec![0u8; dv];
            for (j, m) in mask_in.iter_mut().enumerate() {
                // Checked wider sum: two shifted ternary coefficients can exceed i32 before the clamp,
                // so the temporary is i64 and the result is bounded here.
                let x = (self.e_old.weight(prev, j) as i64) * (1i64 << ps)
                    + (self.e_new.weight(cur, j) as i64) * (1i64 << cs);
                let x = x.clamp(-(PRIOR_CLAMP as i64), PRIOR_CLAMP as i64) as i32;
                // Bounded ReLU. The mask is the STE of this bound, so the value must actually be
                // bounded: recording the mask without applying the clamp left negative activations in
                // the hidden vector, which the authored capacity witness immediately exposed.
                h[j] = if x <= 0 {
                    0
                } else if x >= PRIOR_CLAMP {
                    PRIOR_CLAMP
                } else {
                    x
                };
                *m = if x <= 0 || x >= PRIOR_CLAMP { 0 } else { 1 };
            }
            let m_max = h.iter().fold(0i32, |a, &x| a.max(x.abs()));
            shift = norm_shift(m_max, self.cfg.norm_bits);
            for j in 0..dv {
                h[j] >>= shift;
            }
            mask = mask_in;
            for (r, slot) in z.iter_mut().enumerate() {
                let s = self.w_o.shift(r);
                let mut acc: i32 = 0;
                for j in 0..dv {
                    match self.w_o.weight(r, j) {
                        1 => acc += h[j],
                        -1 => acc -= h[j],
                        _ => {}
                    }
                }
                *slot += acc << s;
            }
        }
        Trace {
            z,
            h,
            mask,
            shift,
            prev,
            cur,
            used_context: use_context,
        }
    }

    pub fn int_logits(&self, prev: usize, cur: usize, use_context: bool) -> Vec<i32> {
        self.trace(prev, cur, use_context).z
    }

    pub fn context_features(&self, prev: usize, cur: usize) -> Vec<i32> {
        self.trace(prev, cur, true).h
    }

    /// `(logsumexp(z) − z[target]) / ln 2` with `z = Z · 2^-F`; uncapped and unfloored.
    pub fn bits_one(&self, z_int: &[i32], target: u32) -> f64 {
        let scale = (-(self.cfg.f_bits as f64)).exp2();
        let t = (target as usize).min(self.cfg.vocab - 1);
        let mut max = f64::NEG_INFINITY;
        for &zz in z_int {
            max = max.max(zz as f64 * scale);
        }
        let mut sum = 0f64;
        for &zz in z_int {
            sum += (zz as f64 * scale - max).exp();
        }
        (max + sum.ln() - z_int[t] as f64 * scale) / std::f64::consts::LN_2
    }

    /// Total bits and scored-target count over a window under an explicit mask.
    pub fn bits_masked(
        &self,
        tokens: &[u32],
        mask: Option<&[bool]>,
        use_context: bool,
    ) -> (f64, usize) {
        let mut total = 0f64;
        let mut n = 0usize;
        for (_i, prev, cur, target) in targets_masked(tokens, self.cfg.vocab, mask) {
            let tr = self.trace(prev, cur, use_context);
            total += self.bits_one(&tr.z, target);
            n += 1;
        }
        (total, n)
    }

    pub fn bits(&self, tokens: &[u32], use_context: bool) -> (f64, usize) {
        self.bits_masked(tokens, None, use_context)
    }

    /// Greedy integer serving: `argmax(Z)`, no runtime scale multiply.
    pub fn argmax(&self, prev: usize, cur: usize) -> usize {
        let z = self.int_logits(prev, cur, true);
        let mut best = 0usize;
        for (r, &x) in z.iter().enumerate() {
            if x > z[best] {
                best = r;
            }
        }
        best
    }

    pub fn generate(&self, prompt: &[u32], n_new: usize, use_context: bool) -> Vec<u32> {
        let mut toks = prompt.to_vec();
        for _ in 0..n_new {
            let i = toks.len() - 1;
            let prev = if i == 0 {
                self.cfg.pad_row()
            } else {
                (toks[i - 1] as usize).min(self.cfg.vocab - 1)
            };
            let cur = (toks[i] as usize).min(self.cfg.vocab - 1);
            let z = self.int_logits(prev, cur, use_context);
            let mut best = 0usize;
            for (r, &x) in z.iter().enumerate() {
                if x > z[best] {
                    best = r;
                }
            }
            toks.push(best as u32);
        }
        toks[prompt.len()..].to_vec()
    }

    pub fn weight_bytes(&self) -> usize {
        let t = [&self.e_old, &self.e_new, &self.w_o];
        t.iter()
            .map(|x| x.weight_bytes() + x.rows * 4)
            .sum::<usize>()
            + self.bias_codes.len()
    }

    /// Shared validation, applied to **both** constructed and reloaded cores so a constructed core
    /// cannot bypass the loader's checks.
    pub fn validate(&self) -> Result<(), String> {
        self.cfg.validate()?;
        if self.elements.len() != self.cfg.vocab {
            return Err("element table length != vocab".into());
        }
        if self.elements.iter().any(|e| *e as usize >= RADIX) {
            return Err("element outside the 2I radix".into());
        }
        if self.bias_codes.len() != self.cfg.vocab {
            return Err("bias code count != vocab".into());
        }
        if self
            .bias_codes
            .iter()
            .any(|b| !(-BIAS_CODE_MAX..=BIAS_CODE_MAX).contains(&(*b as i32)))
        {
            return Err("bias code outside the declared 4-bit range".into());
        }
        let want = [
            (self.cfg.vocab + 1, self.cfg.dv),
            (self.cfg.vocab, self.cfg.dv),
            (self.cfg.vocab, self.cfg.dv),
        ];
        for (k, t) in [&self.e_old, &self.e_new, &self.w_o].iter().enumerate() {
            if (t.rows, t.cols) != want[k] {
                return Err(format!(
                    "table {k} shape {}x{} != {}x{}",
                    t.rows, t.cols, want[k].0, want[k].1
                ));
            }
            for r in 0..t.rows {
                let sh = t.shift(r);
                if sh > 30 {
                    return Err(format!("table {k} row {r} shift {sh} out of range"));
                }
                if k == 2 {
                    self.cfg.check_row_shift(sh)?;
                }
            }
        }
        Ok(())
    }

    /// Serialise with the metadata the contract depends on, including seed and tokenizer digest.
    pub fn to_bytes(&self, tokenizer_digest: &[u8; 32], seed: u64) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(MAGIC);
        o.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        o.extend_from_slice(&(self.cfg.vocab as u32).to_le_bytes());
        o.extend_from_slice(&(self.cfg.dv as u32).to_le_bytes());
        o.extend_from_slice(&self.cfg.norm_bits.to_le_bytes());
        o.extend_from_slice(&self.cfg.f_bits.to_le_bytes());
        o.extend_from_slice(&self.cfg.bias_scale_bits.to_le_bytes());
        o.extend_from_slice(&seed.to_le_bytes());
        o.extend_from_slice(tokenizer_digest);
        o.extend_from_slice(&(self.elements.len() as u32).to_le_bytes());
        for e in &self.elements {
            o.extend_from_slice(&e.to_le_bytes());
        }
        for t in [&self.e_old, &self.e_new, &self.w_o] {
            o.extend_from_slice(&(t.rows as u32).to_le_bytes());
            o.extend_from_slice(&(t.cols as u32).to_le_bytes());
            for r in 0..t.rows {
                o.extend_from_slice(&t.shift(r).to_le_bytes());
            }
            o.extend_from_slice(t.packed());
        }
        for b in &self.bias_codes {
            o.push(*b as u8);
        }
        o
    }

    /// Metadata recovered on load, so a loaded model can be tied back to its inputs.
    pub fn metadata(&self, bytes: &[u8]) -> Result<(u64, [u8; 32]), String> {
        if bytes.len() < 4 + 4 + 4 + 4 + 4 + 4 + 4 + 8 + 32 {
            return Err("truncated header".into());
        }
        let mut c = 4 + 4 + 4 + 4 + 4 + 4 + 4;
        let seed = u64::from_le_bytes(bytes[c..c + 8].try_into().unwrap());
        c += 8;
        let mut d = [0u8; 32];
        d.copy_from_slice(&bytes[c..c + 32]);
        Ok((seed, d))
    }

    /// Bounded, checked load. Rejects shapes, ranges, envelope violations and trailing bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != MAGIC {
            return Err("bad magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        if u32_at(&mut c)? != FORMAT_VERSION {
            return Err("unsupported version".into());
        }
        let cfg = Config {
            vocab: u32_at(&mut c)? as usize,
            dv: u32_at(&mut c)? as usize,
            norm_bits: u32_at(&mut c)?,
            f_bits: u32_at(&mut c)?,
            bias_scale_bits: u32_at(&mut c)?,
        };
        cfg.validate()?;
        let _seed = u64::from_le_bytes(take(&mut c, 8)?.try_into().unwrap());
        let _digest = take(&mut c, 32)?.to_vec();
        let nel = u32_at(&mut c)? as usize;
        if nel != cfg.vocab {
            return Err(format!("element table {nel} != vocab {}", cfg.vocab));
        }
        // Checked size arithmetic before any large allocation.
        let elems_bytes = nel.checked_mul(2).ok_or("element size overflow")?;
        if c.checked_add(elems_bytes)
            .map(|e| e > bytes.len())
            .unwrap_or(true)
        {
            return Err("element table past end".into());
        }
        let mut elements = Vec::with_capacity(nel);
        for _ in 0..nel {
            let e = u16::from_le_bytes(take(&mut c, 2)?.try_into().unwrap());
            if e as usize >= RADIX {
                return Err(format!("element {e} outside the 2I radix"));
            }
            elements.push(e);
        }
        let rows_expect = [cfg.vocab + 1, cfg.vocab, cfg.vocab];
        let mut tables = Vec::new();
        for (k, want) in rows_expect.iter().enumerate() {
            let rows = u32_at(&mut c)? as usize;
            let cols = u32_at(&mut c)? as usize;
            if rows != *want || cols != cfg.dv {
                return Err(format!(
                    "table {k} shape {rows}x{cols} != {want}x{}",
                    cfg.dv
                ));
            }
            let mut shift = Vec::with_capacity(rows);
            for _ in 0..rows {
                let s = u32_at(&mut c)?;
                if s > 30 {
                    return Err(format!("table {k} shift {s} out of range"));
                }
                if k == 2 {
                    cfg.check_row_shift(s)?;
                }
                shift.push(s);
            }
            let packed_len = rows
                .checked_mul(cols)
                .map(|n| n.div_ceil(4))
                .ok_or("table size overflow")?;
            let packed = take(&mut c, packed_len)?.to_vec();
            tables.push(TernaryLinear::from_packed(packed, shift, rows, cols)?);
        }
        let mut bias_codes = Vec::with_capacity(cfg.vocab);
        for _ in 0..cfg.vocab {
            let b = take(&mut c, 1)?[0] as i8;
            if !(-BIAS_CODE_MAX..=BIAS_CODE_MAX).contains(&(b as i32)) {
                return Err(format!("bias code {b} outside the declared 4-bit range"));
            }
            bias_codes.push(b);
        }
        if c != bytes.len() {
            return Err(format!("{} trailing bytes", bytes.len() - c));
        }
        let [e_old, e_new, w_o] = match (tables.pop(), tables.pop(), tables.pop()) {
            (Some(a), Some(b), Some(d)) => [d, b, a],
            _ => return Err("missing tables".into()),
        };
        let core = Self {
            cfg,
            elements,
            e_old,
            e_new,
            w_o,
            bias_codes,
        };
        core.validate()?;
        Ok(core)
    }
}

/// Frozen bias codes from a fit-only unigram distribution.
pub fn bias_codes_from_counts(counts: &[u64], total: u64, vocab: usize) -> Vec<i8> {
    (0..vocab)
        .map(|r| {
            let p = (counts[r] as f64 + 1.0) / (total as f64 + vocab as f64);
            (p.ln() + BIAS_OFFSET)
                .round()
                .max(-(BIAS_CODE_MAX as f64))
                .min(BIAS_CODE_MAX as f64) as i8
        })
        .collect()
}

/// **Authored capacity witness** for the 16 ordered pairs `(a, b) -> (a + b) mod 4`.
///
/// An exact mathematical construction for the declared operations, kept entirely separate from every
/// learned arm. Index 16 hidden coordinates by `(a, b)`: `E_old(x)[a,b] = +1` at `x == a` else `-1`;
/// `E_new(y)[a,b] = 0` at `y == b` else `-1`; the absent-prefix row is `-1` throughout. The sum is
/// `+1` only at the matching pair, so a bounded ReLU yields a one-hot pair feature. `W[r,a,b] = +1`
/// when `r == (a+b) mod 4`, with output shift 12, `F = 10`, `norm_bits = 6` and a flat bias. Unused
/// `dv = 32` coordinates stay zero.
///
/// This proves **finite representability only**. It is never used to initialise a learning arm and is
/// not a learned or geometric result.
pub fn authored_witness(vocab: usize, dv: usize) -> Result<PriorCore, String> {
    if vocab != 4 {
        return Err("the witness is defined for vocab 4".into());
    }
    if dv < 16 {
        return Err("the witness needs at least 16 coordinates".into());
    }
    let mut cfg = Config::new(vocab, dv);
    cfg.bias_scale_bits = 10;
    cfg.validate()?;
    let pairs = |idx: usize| -> (usize, usize) { (idx / 4, idx % 4) };
    // E_old: rows 0..=3 are the tokens, row 4 is the absent-prefix marker.
    let mut e_old_master = vec![0f32; (vocab + 1) * dv];
    for row in 0..=vocab {
        for p in 0..16usize {
            let (a, _b) = pairs(p);
            e_old_master[row * dv + p] = if row < vocab && row == a { 1.0 } else { -1.0 };
        }
    }
    let mut e_new_master = vec![0f32; vocab * dv];
    for y in 0..vocab {
        for p in 0..16usize {
            let (_a, b) = pairs(p);
            e_new_master[y * dv + p] = if y == b { 0.0 } else { -1.0 };
        }
    }
    // W[r][a,b] = +1 exactly on the correct class. Shifting the output row by 12 makes the margin 4
    // after the shared 2^-F = 2^-10 scaling: (1 << 12) * 2^-10 = 4.
    let mut wo_master = vec![0f32; vocab * dv];
    for r in 0..vocab {
        for p in 0..16usize {
            let (a, b) = pairs(p);
            if r == (a + b) % 4 {
                wo_master[r * dv + p] = (1i32 << 12) as f32;
            }
        }
    }
    Ok(PriorCore {
        cfg,
        elements: (0..vocab).map(|t| (t % RADIX) as u16).collect(),
        e_old: TernaryLinear::quantize(&e_old_master, vocab + 1, dv),
        e_new: TernaryLinear::quantize(&e_new_master, vocab, dv),
        w_o: TernaryLinear::quantize(&wo_master, vocab, dv),
        bias_codes: vec![0i8; vocab],
    })
}

// ---------------------------------------------------------------------------
// Offline training
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct PriorTrainer {
    pub cfg: Config,
    pub elements: Vec<u16>,
    pub cfg_train: TrainConfig,
    pub step: u64,
    seed: u64,
    pub e_old: Vec<f32>,
    pub e_new: Vec<f32>,
    pub wo: Vec<f32>,
    bias_codes: Vec<i8>,
    ge_old: Vec<f32>,
    ge_new: Vec<f32>,
    gwo: Vec<f32>,
    me_old: Vec<f32>,
    me_new: Vec<f32>,
    mwo: Vec<f32>,
    ve_old: Vec<f32>,
    ve_new: Vec<f32>,
    vwo: Vec<f32>,
    /// Fisher–Yates schedule state: permutation, cursor, pass.
    perm: Vec<usize>,
    pub cursor: usize,
    pub pass: u32,
    pub rng: u64,
    pub data_identity: [u8; 32],
}

impl PriorTrainer {
    pub fn new(
        cfg: Config,
        seed: u64,
        bias_codes: Vec<i8>,
        output_master_bound: f32,
    ) -> Result<Self, String> {
        cfg.validate()?;
        if bias_codes.len() != cfg.vocab {
            return Err("bias code count != vocab".into());
        }
        let v = cfg.vocab;
        let dv = cfg.dv;
        let mut st = seed ^ 0x9E37_79B9_7F4A_7C15;
        let mut fill = |n: usize, lo: f32, hi: f32| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    let u = xorshift_unit(&mut st);
                    (u * 0.5 + 0.5) * (hi - lo) + lo
                })
                .collect()
        };
        Ok(Self {
            elements: (0..v).map(|t| (t % RADIX) as u16).collect(),
            // Sign-varied so different tokens give different features; an all-positive initialization
            // quantises to identical codes on every row and carries no learnable context signal.
            e_old: fill((v + 1) * dv, -1.5, 1.5),
            e_new: fill(v * dv, -1.5, 1.5),
            // Sub-threshold but asymmetric. Note the gradient to the prior tables is *not* zero for
            // arbitrary output masters: `(p_r − 1[r=t])·h` already carries class-dependent credit, so
            // the all-zero exported residual is a chosen starting point rather than a mathematical
            // requirement.
            wo: fill(v * dv, -output_master_bound, output_master_bound),
            bias_codes,
            ge_old: vec![0.0; (v + 1) * dv],
            ge_new: vec![0.0; v * dv],
            gwo: vec![0.0; v * dv],
            me_old: vec![0.0; (v + 1) * dv],
            me_new: vec![0.0; v * dv],
            mwo: vec![0.0; v * dv],
            ve_old: vec![0.0; (v + 1) * dv],
            ve_new: vec![0.0; v * dv],
            vwo: vec![0.0; v * dv],
            cfg_train: TrainConfig {
                lr: 0.05,
                ..TrainConfig::default()
            },
            step: 0,
            seed,
            cfg,
            perm: Vec::new(),
            cursor: 0,
            pass: 0,
            rng: seed ^ 0xD1CE_B00D,
            data_identity: [0u8; 32],
        })
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The single quantisation used by training and serving: build the served core from the masters.
    pub fn core_view(&self) -> Result<PriorCore, String> {
        let v = self.cfg.vocab;
        let dv = self.cfg.dv;
        Ok(PriorCore {
            cfg: self.cfg.clone(),
            elements: self.elements.clone(),
            e_old: TernaryLinear::quantize(&self.e_old, v + 1, dv),
            e_new: TernaryLinear::quantize(&self.e_new, v, dv),
            w_o: TernaryLinear::quantize(&self.wo, v, dv),
            bias_codes: self.bias_codes.clone(),
        })
    }

    pub fn to_core(&self) -> Result<PriorCore, String> {
        self.core_view()
    }

    fn zero_grads(&mut self) {
        for g in self
            .ge_old
            .iter_mut()
            .chain(&mut self.ge_new)
            .chain(&mut self.gwo)
        {
            *g = 0.0;
        }
    }

    /// Forward via the **served** primitive and backward as the declared surrogate of it.
    pub fn accumulate_with(
        &mut self,
        core: &PriorCore,
        tokens: &[u32],
        mask: Option<&[bool]>,
    ) -> (f64, usize) {
        let v = self.cfg.vocab;
        let dv = self.cfg.dv;
        let scale = (-(self.cfg.f_bits as f64)).exp2();
        let inv_ln2 = 1.0 / std::f64::consts::LN_2;
        let mut total = 0f64;
        let mut n = 0usize;
        for (_i, prev, cur, target) in targets_masked(tokens, v, mask) {
            let tr = core.trace(prev, cur, true);
            let t = (target as usize).min(v - 1);
            total += core.bits_one(&tr.z, target);
            n += 1;

            let z_off: Vec<f64> = tr.z.iter().map(|&x| x as f64 * scale).collect();
            let max = z_off.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mut sum = 0f64;
            for &x in &z_off {
                sum += (x - max).exp();
            }
            let inv_sum = 1.0 / sum;
            // d bits / d Z : the softmax error times 2^-F/ln2, applied exactly once at integer-score
            // credit. Downstream propagation already carries 2^-F.
            let dz: Vec<f64> = z_off
                .iter()
                .enumerate()
                .map(|(r, &x)| {
                    ((x - max).exp() * inv_sum - if r == t { 1.0 } else { 0.0 }) * inv_ln2 * scale
                })
                .collect();

            let mut dh = vec![0f64; dv];
            for r in 0..v {
                let g = dz[r];
                if g == 0.0 {
                    continue;
                }
                let s = core.w_o.shift(r);
                let sgn = (1i64 << s) as f64;
                for j in 0..dv {
                    self.gwo[r * dv + j] += (g * tr.h[j] as f64) as f32;
                    dh[j] += g * (core.w_o.weight(r, j) as f64 * sgn);
                }
            }
            // Bounded-ReLU STE then the per-position downshift; the effective-weight surrogate has
            // derivative one, so this is the gradient of the master.
            for j in 0..dv {
                let dec = 1.0 / (1u64 << tr.shift) as f64;
                let d = dh[j] * tr.mask[j] as f64 * dec;
                self.ge_old[prev * dv + j] += d as f32;
                self.ge_new[cur * dv + j] += d as f32;
            }
        }
        (total, n)
    }

    /// One Adam step over a batch, dividing the batch mean by the actual scored-target count once.
    pub fn train_batch_masked(&mut self, batch: &[Vec<u32>], mask: Option<&[bool]>) -> f64 {
        if batch.is_empty() {
            return 0.0;
        }
        let core = match self.core_view() {
            Ok(c) => c,
            Err(_) => return f64::NAN,
        };
        self.zero_grads();
        let mut bits = 0f64;
        let mut n = 0usize;
        for seq in batch {
            let (b, k) = self.accumulate_with(&core, seq, mask);
            bits += b;
            n += k;
        }
        if n == 0 {
            return f64::NAN;
        }
        let mean = bits / n as f64;
        let s = 1.0f32 / n as f32;
        let mut sumsq = 0f64;
        for g in self
            .ge_old
            .iter_mut()
            .chain(&mut self.ge_new)
            .chain(&mut self.gwo)
        {
            *g *= s;
            sumsq += (*g as f64) * (*g as f64);
        }
        if self.cfg_train.grad_clip > 0.0 {
            let norm = libm::sqrt(sumsq);
            let clip = self.cfg_train.grad_clip as f64;
            if norm > clip && norm > 0.0 {
                let f = (clip / norm) as f32;
                for g in self
                    .ge_old
                    .iter_mut()
                    .chain(&mut self.ge_new)
                    .chain(&mut self.gwo)
                {
                    *g *= f;
                }
            }
        }
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
            &mut self.wo,
            &self.gwo,
            &mut self.mwo,
            &mut self.vwo,
            &cfg,
            step,
        );
        mean
    }

    pub fn train_batch_bits(&mut self, batch: &[Vec<u32>]) -> f64 {
        self.train_batch_masked(batch, None)
    }

    /// A real Fisher–Yates permutation of `0..n`, seeded and reproducible.
    fn shuffle(&mut self, n: usize) {
        self.perm = (0..n).collect();
        for i in (1..n).rev() {
            self.rng = self
                .rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = ((self.rng >> 33) as usize) % (i + 1);
            self.perm.swap(i, j);
        }
    }

    /// The next batch: every window appears exactly once per pass, and a short final batch is returned
    /// short rather than wrapped to fill.
    pub fn next_batch(&mut self, windows: &[Vec<u32>], batch: usize) -> Vec<Vec<u32>> {
        if windows.is_empty() || batch == 0 {
            return Vec::new();
        }
        if self.perm.len() != windows.len() || self.cursor >= windows.len() {
            if self.perm.len() == windows.len() && self.cursor >= windows.len() {
                self.pass += 1;
            }
            self.shuffle(windows.len());
            self.cursor = 0;
        }
        let end = (self.cursor + batch).min(windows.len());
        let out = self.perm[self.cursor..end]
            .iter()
            .map(|&idx| windows[idx].clone())
            .collect();
        self.cursor = end;
        if self.cursor >= windows.len() {
            self.cursor = windows.len();
        }
        out
    }

    /// Whether a full pass has been consumed.
    pub fn pass_complete(&self, windows: &[Vec<u32>]) -> bool {
        !windows.is_empty() && self.perm.len() == windows.len() && self.cursor >= windows.len()
    }

    /// A complete, resumable checkpoint: every master, both moments, the full optimizer
    /// configuration, numerical config, frozen bias, schedule state and data identity.
    pub fn checkpoint_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"CPCK");
        o.extend_from_slice(&3u32.to_le_bytes());
        o.extend_from_slice(&(self.cfg.vocab as u32).to_le_bytes());
        o.extend_from_slice(&(self.cfg.dv as u32).to_le_bytes());
        o.extend_from_slice(&self.cfg.norm_bits.to_le_bytes());
        o.extend_from_slice(&self.cfg.f_bits.to_le_bytes());
        o.extend_from_slice(&self.cfg.bias_scale_bits.to_le_bytes());
        o.extend_from_slice(&self.step.to_le_bytes());
        o.extend_from_slice(&(self.cursor as u64).to_le_bytes());
        o.extend_from_slice(&(self.pass as u64).to_le_bytes());
        o.extend_from_slice(&self.rng.to_le_bytes());
        for x in [
            self.cfg_train.lr,
            self.cfg_train.beta1,
            self.cfg_train.beta2,
            self.cfg_train.weight_decay,
            self.cfg_train.grad_clip,
        ] {
            o.extend_from_slice(&x.to_le_bytes());
        }
        o.extend_from_slice(&self.seed.to_le_bytes());
        o.extend_from_slice(&self.data_identity);
        // The active permutation, so a mid-pass restore continues the same schedule exactly.
        o.extend_from_slice(&(self.perm.len() as u64).to_le_bytes());
        for p in &self.perm {
            o.extend_from_slice(&(*p as u64).to_le_bytes());
        }
        for v in [
            &self.e_old,
            &self.e_new,
            &self.wo,
            &self.me_old,
            &self.me_new,
            &self.mwo,
            &self.ve_old,
            &self.ve_new,
            &self.vwo,
        ] {
            o.extend_from_slice(&(v.len() as u64).to_le_bytes());
            for f in v.iter() {
                o.extend_from_slice(&f.to_le_bytes());
            }
        }
        o.extend_from_slice(&(self.bias_codes.len() as u64).to_le_bytes());
        for b in &self.bias_codes {
            o.push(*b as u8);
        }
        o
    }

    /// Load a checkpoint. Validates identity and config **before** mutating anything, so a rejected
    /// load leaves the live trainer untouched.
    pub fn resume_from(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated checkpoint".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"CPCK" {
            return Err("bad checkpoint magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        let u64_at = |c: &mut usize| -> Result<u64, String> {
            Ok(u64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
        };
        let f32_at = |c: &mut usize| -> Result<f32, String> {
            Ok(f32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        if u32_at(&mut c)? != 3 {
            return Err("unsupported checkpoint version".into());
        }
        if u32_at(&mut c)? as usize != self.cfg.vocab
            || u32_at(&mut c)? as usize != self.cfg.dv
            || u32_at(&mut c)? != self.cfg.norm_bits
            || u32_at(&mut c)? != self.cfg.f_bits
            || u32_at(&mut c)? != self.cfg.bias_scale_bits
        {
            return Err("checkpoint numerical config differs".into());
        }
        let step = u64_at(&mut c)?;
        let cursor = u64_at(&mut c)? as usize;
        let pass = u64_at(&mut c)? as u32;
        let rng = u64_at(&mut c)?;
        let lr = f32_at(&mut c)?;
        let beta1 = f32_at(&mut c)?;
        let beta2 = f32_at(&mut c)?;
        let weight_decay = f32_at(&mut c)?;
        let grad_clip = f32_at(&mut c)?;
        let _seed = u64_at(&mut c)?;
        let mut identity = [0u8; 32];
        identity.copy_from_slice(take(&mut c, 32)?);
        let perm_len = u64_at(&mut c)? as usize;
        if perm_len > MAX_VOCAB {
            return Err("checkpoint permutation is implausibly long".into());
        }
        let mut perm = Vec::with_capacity(perm_len);
        let mut seen = vec![false; perm_len];
        for _ in 0..perm_len {
            let p = u64_at(&mut c)? as usize;
            if p >= perm_len || std::mem::replace(&mut seen[p], true) {
                return Err("checkpoint permutation is not a permutation of 0..n".into());
            }
            perm.push(p);
        }
        if self.data_identity != [0u8; 32] && identity != self.data_identity {
            return Err("checkpoint data identity differs".into());
        }
        // Stage the parameter blocks, validating lengths, before touching live state.
        let mut blocks: Vec<Vec<f32>> = Vec::new();
        for want in [
            (self.cfg.vocab + 1) * self.cfg.dv,
            self.cfg.vocab * self.cfg.dv,
            self.cfg.vocab * self.cfg.dv,
            (self.cfg.vocab + 1) * self.cfg.dv,
            self.cfg.vocab * self.cfg.dv,
            self.cfg.vocab * self.cfg.dv,
            (self.cfg.vocab + 1) * self.cfg.dv,
            self.cfg.vocab * self.cfg.dv,
            self.cfg.vocab * self.cfg.dv,
        ] {
            let len = u64_at(&mut c)? as usize;
            if len != want {
                return Err("checkpoint parameter length differs".into());
            }
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                let x = f32_at(&mut c)?;
                if !x.is_finite() {
                    return Err("checkpoint holds a non-finite parameter".into());
                }
                v.push(x);
            }
            blocks.push(v);
        }
        let nb = u64_at(&mut c)? as usize;
        if nb != self.bias_codes.len() {
            return Err("checkpoint bias length differs".into());
        }
        let mut bias = Vec::with_capacity(nb);
        for _ in 0..nb {
            bias.push(take(&mut c, 1)?[0] as i8);
        }
        if c != bytes.len() {
            return Err(format!("{} trailing checkpoint bytes", bytes.len() - c));
        }
        if !(0.0..1.0).contains(&beta1) || !(0.0..1.0).contains(&beta2) {
            return Err("checkpoint beta outside [0,1)".into());
        }
        if !lr.is_finite() || lr < 0.0 || !grad_clip.is_finite() || grad_clip < 0.0 {
            return Err("checkpoint optimizer fields are invalid".into());
        }
        if !weight_decay.is_finite() || weight_decay < 0.0 {
            return Err("checkpoint weight decay is invalid".into());
        }
        // Second moments must be non-negative and finite; Adam divides by their square root.
        if blocks
            .iter()
            .skip(6)
            .any(|b| b.iter().any(|x| !x.is_finite() || *x < 0.0))
        {
            return Err("checkpoint second moments are invalid".into());
        }
        // Commit.
        let mut it = blocks.into_iter();
        self.e_old = it.next().unwrap();
        self.e_new = it.next().unwrap();
        self.wo = it.next().unwrap();
        self.me_old = it.next().unwrap();
        self.me_new = it.next().unwrap();
        self.mwo = it.next().unwrap();
        self.ve_old = it.next().unwrap();
        self.ve_new = it.next().unwrap();
        self.vwo = it.next().unwrap();
        self.bias_codes = bias;
        self.step = step;
        self.cursor = cursor;
        self.pass = pass;
        self.rng = rng;
        self.cfg_train.lr = lr;
        self.cfg_train.beta1 = beta1;
        self.cfg_train.beta2 = beta2;
        self.cfg_train.weight_decay = weight_decay;
        self.cfg_train.grad_clip = grad_clip;
        self.data_identity = identity;
        self.perm = perm;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(v: usize, dv: usize) -> Config {
        Config::new(v, dv)
    }

    fn flat_bias(v: usize) -> Vec<i8> {
        vec![1i8; v]
    }

    #[test]
    fn the_iterator_scores_exactly_n_minus_one_targets() {
        let toks: Vec<u32> = (0..64).collect();
        let it: Vec<_> = targets(&toks, 64).collect();
        assert_eq!(it.len(), 63);
        assert_eq!(
            it[0].1, 64,
            "the first prediction uses the absent-prefix row"
        );
        assert_eq!(it[0].3, 1);
        assert_eq!(it[62].3, 63);
        let masked: Vec<_> = targets_masked(&toks, 64, Some(&fixture_mask(64))).collect();
        assert_eq!(
            masked.len(),
            1,
            "the fixture mask scores the final position only"
        );
        assert_eq!(masked[0].3, 63);
    }

    #[test]
    fn training_and_serving_scores_are_the_same_computation() {
        // Nonzero contextual output, nonflat bias, F > 0, a nonunit row shift and active bounds.
        let v = 4usize;
        let dv = 32usize;
        let mut t = PriorTrainer::new(cfg(v, dv), 13, vec![3i8, -1, 0, 4], 0.45).unwrap();
        // Drive the output masters across the threshold so the residual is nonzero.
        let seqs: Vec<Vec<u32>> = (0..4u32)
            .flat_map(|x| (0..4u32).map(move |y| vec![x, y, (x + y) % 4]))
            .collect();
        let mask = fixture_mask(3);
        for _ in 0..60 {
            t.train_batch_masked(&seqs, Some(&mask));
        }
        let core = t.to_core().unwrap();
        assert!(
            core.w_o.packed().iter().any(|b| *b != 0),
            "the test is vacuous without a nonzero contextual readout"
        );
        for s in &seqs {
            let tr = core.trace(core.cfg.pad_row(), s[0] as usize, true);
            let bias_only = core.int_logits(core.cfg.pad_row(), s[0] as usize, false);
            assert_ne!(tr.z, bias_only, "the contextual term must change the score");
            let (serve_total, serve_n) = core.bits_masked(s, Some(&mask), true);
            let mut trainer_bits = 0f64;
            let mut trainer_n = 0usize;
            let mut t2 = t.clone();
            t2.zero_grads();
            let (b, n) = t2.accumulate_with(&core, s, Some(&mask));
            trainer_bits += b;
            trainer_n += n;
            assert_eq!(trainer_n, serve_n);
            assert!(
                (trainer_bits - serve_total).abs() < 1e-12,
                "training loss must equal served loss: {trainer_bits} vs {serve_total}"
            );
        }
        // And the removed parallel definition is detectably wrong: omitting the shared 2^-F from the
        // contextual term changes the loss.
        let scale = (-(core.cfg.f_bits as f64)).exp2();
        let s = &seqs[5];
        let tr = core.trace(core.cfg.pad_row(), s[0] as usize, true);
        let _ = scale;
        let wrong: Vec<i32> =
            tr.z.iter()
                .zip(core.int_logits(core.cfg.pad_row(), s[0] as usize, false))
                .map(|(&z, b)| b + ((z - b) << core.cfg.f_bits))
                .collect();
        let right_bits = core.bits_one(&tr.z, s[2]);
        let wrong_bits = core.bits_one(&wrong, s[2]);
        assert!(
            (right_bits - wrong_bits).abs() > 1e-6,
            "the unscaled-context variant must differ, or the parity test proves nothing"
        );
    }

    #[test]
    fn uniform_integer_scores_are_exactly_twelve_bits_at_v4096() {
        let core = PriorTrainer::new(cfg(4096, 4), 1, vec![0i8; 4096], 0.0)
            .unwrap()
            .to_core()
            .unwrap();
        let z = vec![0i32; 4096];
        for t in [0u32, 1, 2048, 4095] {
            assert!((core.bits_one(&z, t) - 12.0).abs() < 1e-12);
        }
    }

    #[test]
    fn a_common_offset_changes_nothing_and_extreme_errors_are_uncapped() {
        let core = PriorTrainer::new(cfg(16, 4), 1, flat_bias(16), 0.0)
            .unwrap()
            .to_core()
            .unwrap();
        let base = vec![0i32; 16];
        let shifted: Vec<i32> = base.iter().map(|v| v + 5000).collect();
        assert!((core.bits_one(&base, 3) - core.bits_one(&shifted, 3)).abs() < 1e-12);
        let mut confident = vec![100_000i32; 16];
        confident[5] = -100_000;
        assert!(core.bits_one(&confident, 5) > 29.8974);
    }

    #[test]
    fn the_balanced_fixture_has_pure_contexts_and_uniform_targets() {
        let mut targets_seen: Vec<u32> = Vec::new();
        let mut per_context: std::collections::HashMap<(u32, u32), Vec<u32>> = Default::default();
        for x in 0..4u32 {
            for y in 0..4u32 {
                let s = vec![x, y, (x + y) % 4];
                let scored: Vec<_> = targets_masked(&s, 4, Some(&fixture_mask(3))).collect();
                assert_eq!(scored.len(), 1, "exactly one target per context");
                let (_i, prev, cur, t) = scored[0];
                assert_eq!(prev, x as usize);
                assert_eq!(cur, y as usize);
                per_context.entry((x, y)).or_default().push(t);
                targets_seen.push(t);
            }
        }
        assert!(
            per_context.values().all(|v| v.len() == 1),
            "every context has one target"
        );
        let mut counts = [0usize; 4];
        for t in &targets_seen {
            counts[*t as usize] += 1;
        }
        assert_eq!(
            counts,
            [4, 4, 4, 4],
            "all four targets are equally frequent"
        );
        // Either single input alone leaves a uniform target distribution.
        for x in 0..4u32 {
            let mut c = [0usize; 4];
            for y in 0..4u32 {
                c[((x + y) % 4) as usize] += 1;
            }
            assert_eq!(c, [1, 1, 1, 1], "x alone must not determine the target");
        }
    }

    #[test]
    fn the_authored_witness_represents_all_sixteen_pairs_exactly() {
        let core = authored_witness(4, 32).expect("witness");
        let mut correct = 0usize;
        for x in 0..4u32 {
            for y in 0..4u32 {
                let want = (x + y) % 4;
                let z = core.int_logits(x as usize, y as usize, true);
                let mut best = 0usize;
                for (r, &v) in z.iter().enumerate() {
                    if v > z[best] {
                        best = r;
                    }
                }
                assert_eq!(best as u32, want, "witness prediction for ({x},{y})");
                let margin = z[want as usize] - z[(want as usize + 1) % 4];
                assert_eq!(margin, 4096, "margin 4 in logits for ({x},{y}); z={z:?}");
                correct += 1;
            }
        }
        assert_eq!(correct, 16);
        // p_correct = 1/(1 + 3 e^-4) and CE = log2(1 + 3 e^-4).
        let e = core.bits_one(&core.int_logits(0, 0, true), 0);
        let want = (1.0 + 3.0 * (-4.0f64).exp()).log2();
        assert!((e - want).abs() < 1e-9, "witness CE {e} vs {want}");
        // Export/reload must preserve it.
        let bytes = core.to_bytes(&[5u8; 32], 1);
        assert_eq!(PriorCore::from_bytes(&bytes).unwrap(), core);
    }

    #[test]
    fn artifact_roundtrip_is_exact_and_malformed_inputs_are_rejected() {
        let t = PriorTrainer::new(cfg(64, 16), 9, flat_bias(64), 0.3).unwrap();
        let core = t.to_core().unwrap();
        let bytes = core.to_bytes(&[3u8; 32], t.seed());
        assert_eq!(PriorCore::from_bytes(&bytes).unwrap(), core);
        assert!(PriorCore::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(PriorCore::from_bytes(&extra).is_err());
        let mut bad = bytes.clone();
        bad[0] = b'X';
        assert!(PriorCore::from_bytes(&bad).is_err());
        let mut bad_code = bytes.clone();
        let n = bad_code.len();
        bad_code[n - 1] = 99;
        assert!(PriorCore::from_bytes(&bad_code).is_err());
        // A real shift-field mutation: the e_old shift block begins right after the element table.
        let mut bad_shift = bytes.clone();
        // Header = 4 magic + 4 version + 5 config u32 + 8 seed + 32 digest + 4 element count.
        let element_end = 4 + 4 + 5 * 4 + 8 + 32 + 4 + 64 * 2;
        // e_old (65x16) then e_new (64x16) then w_o's rows/cols; the shift block starts after that.
        let e_old_at = element_end + 4 + 4;
        bad_shift[e_old_at..e_old_at + 4].copy_from_slice(&31u32.to_le_bytes());
        assert!(
            PriorCore::from_bytes(&bad_shift).is_err(),
            "a shift of 31 must be rejected, not silently accepted"
        );
        // An envelope violation in the readout shift block must also be rejected.
        let mut bad_row = bytes.clone();
        let wo_shift_at = element_end
            + (4 + 4 + 65 * 4 + (65 * 16usize).div_ceil(4))
            + (4 + 4 + 64 * 4 + (64 * 16usize).div_ceil(4))
            + 4
            + 4;
        bad_row[wo_shift_at..wo_shift_at + 4].copy_from_slice(&30u32.to_le_bytes());
        assert!(
            PriorCore::from_bytes(&bad_row).is_err(),
            "row shift 30 breaks the i32 envelope"
        );
    }

    #[test]
    fn the_schedule_is_a_permutation_with_a_partial_final_batch() {
        let v = 4usize;
        let windows: Vec<Vec<u32>> = (0..7u32).map(|k| vec![k, 1000 + k, 2000 + k]).collect();
        let mut t = PriorTrainer::new(cfg(v, 8), 21, flat_bias(v), 0.3).unwrap();
        let mut seen: Vec<usize> = Vec::new();
        let mut order: Vec<Vec<u32>> = Vec::new();
        loop {
            let b = t.next_batch(&windows, 3);
            if b.is_empty() {
                break;
            }
            order.extend(b.clone());
            if b.len() < 3 {
                break;
            }
        }
        assert_eq!(
            order.len(),
            windows.len(),
            "a pass covers every window exactly once"
        );
        for w in &windows {
            assert_eq!(
                order.iter().filter(|x| *x == w).count(),
                1,
                "no replacement"
            );
        }
        let first: Vec<_> = {
            let mut t2 = PriorTrainer::new(cfg(v, 8), 21, flat_bias(v), 0.3).unwrap();
            let mut o = Vec::new();
            loop {
                let b = t2.next_batch(&windows, 3);
                if b.is_empty() {
                    break;
                }
                o.extend(b.clone());
                if b.len() < 3 {
                    break;
                }
            }
            o
        };
        assert_eq!(seen.len(), 0);
        seen.clear();
        assert_eq!(first, order, "the schedule is reproducible from the seed");
        let _ = seen;
    }

    #[test]
    fn a_resumed_split_run_equals_an_uninterrupted_run_in_full_state() {
        let v = 16usize;
        let windows: Vec<Vec<u32>> = (0..7)
            .map(|k| (0..6).map(|j| ((k * 3 + j) % v) as u32).collect())
            .collect();
        let mut a = PriorTrainer::new(cfg(v, 8), 11, flat_bias(v), 0.3).unwrap();
        a.cfg_train.beta1 = 0.8;
        a.cfg_train.beta2 = 0.99;
        a.cfg_train.weight_decay = 0.001;
        // Seven windows with batch 3 gives a partial final batch inside one pass.
        for _ in 0..3 {
            let b = a.next_batch(&windows, 3);
            a.train_batch_bits(&b);
        }
        let ckpt = a.checkpoint_bytes();
        let mut c = PriorTrainer::new(cfg(v, 8), 11, flat_bias(v), 0.3).unwrap();
        c.cfg_train.beta1 = 0.8;
        c.cfg_train.beta2 = 0.99;
        c.cfg_train.weight_decay = 0.001;
        if let Err(e) = c.resume_from(&ckpt) {
            panic!("resume failed: {e}");
        }
        assert_eq!(c.e_old, a.e_old, "masters");
        assert_eq!(c.me_old, a.me_old, "first moments");
        assert_eq!(c.ve_old, a.ve_old, "second moments");
        assert_eq!(c.cfg_train.beta1, 0.8);
        assert_eq!(c.cfg_train.weight_decay, 0.001);
        // Continue both and compare the next batches and the resulting state.
        let nb_a = a.next_batch(&windows, 3);
        let nb_c = c.next_batch(&windows, 3);
        assert_eq!(nb_a, nb_c, "schedule state");
        a.train_batch_bits(&nb_a);
        c.train_batch_bits(&nb_c);
        if a.e_old != c.e_old {
            let i = a
                .e_old
                .iter()
                .zip(&c.e_old)
                .position(|(x, y)| x != y)
                .unwrap();
            eprintln!(
                "first e_old divergence at {i}: {} vs {} | steps {} vs {} cursors {} vs {} pass {} vs {} rng {} vs {} perm_eq {}",
                a.e_old[i], c.e_old[i], a.step, c.step, a.cursor, c.cursor, a.pass, c.pass, a.rng, c.rng, a.perm == c.perm
            );
        }
        assert_eq!(a.e_old, c.e_old);
        assert_eq!(a.wo, c.wo);
        assert_eq!(a.step, c.step);
        assert_eq!(a.to_core().unwrap(), c.to_core().unwrap());
        // A rejected load must not partially mutate the live trainer.
        let mut d = PriorTrainer::new(cfg(v, 8), 11, flat_bias(v), 0.3).unwrap();
        let before = d.e_old.clone();
        let mut broken = ckpt.clone();
        broken.extend_from_slice(&[0, 0, 0, 0]);
        assert!(d.resume_from(&broken).is_err());
        assert_eq!(d.e_old, before, "a failed load must not change the trainer");
        assert_eq!(d.step, 0);
    }

    #[test]
    fn the_ste_gradient_matches_a_hand_computed_credit_path() {
        // Closed-form credit for one scored position, with the quantizer/shift choices frozen:
        //   gZ[r]  = (p[r] - 1[r=t]) * 2^-F / ln2
        //   gWo[r][j] = gZ[r] * h[j]
        //   gE[j]  = (sum_r gZ[r] * Wo_effective[r][j]) * mask[j] * 2^-shift
        let v = 4usize;
        let dv = 8usize;
        let mut t = PriorTrainer::new(cfg(v, dv), 29, vec![2i8, -3, 1, 0], 0.45).unwrap();
        let seq: Vec<u32> = vec![3, 1, 0];
        let mask = fixture_mask(3);
        // Larger embedding magnitudes give nonunit row scales, so the hidden sum exceeds 2^norm_bits
        // and the per-position normalization shift is actually active.
        for x in t.e_old.iter_mut() {
            *x *= 60.0;
        }
        for x in t.e_new.iter_mut() {
            *x *= 60.0;
        }
        // Train briefly so the residual, the bias and the masks are all active.
        for _ in 0..40 {
            t.train_batch_masked(std::slice::from_ref(&seq), Some(&mask));
        }
        let core = t.to_core().unwrap();
        let tr = core.trace(seq[0] as usize, seq[1] as usize, true);
        let active = tr.mask.iter().filter(|m| **m == 1).count();
        let clamped = tr.mask.iter().filter(|m| **m == 0).count();
        assert!(active > 0, "the fixture must have active ReLU coordinates");
        assert!(
            clamped > 0,
            "the fixture must have clamped ReLU coordinates"
        );
        assert!(
            tr.shift > 0,
            "the fixture must exercise a nonunit normalization shift"
        );
        let bias_only = core.int_logits(seq[0] as usize, seq[1] as usize, false);
        assert_ne!(tr.z, bias_only, "the residual must be nonzero");
        assert!(
            core.bias_codes.iter().any(|b| *b != core.bias_codes[0]),
            "nonflat bias"
        );

        let f = core.cfg.f_bits as i32;
        let scale = (2f64).powi(-f);
        let tgt = seq[2] as usize;
        let zoff: Vec<f64> = tr.z.iter().map(|&x| x as f64 * scale).collect();
        let max = zoff.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ex: Vec<f64> = zoff.iter().map(|x| (x - max).exp()).collect();
        let sum: f64 = ex.iter().sum();
        let gz: Vec<f64> = (0..v)
            .map(|r| {
                (ex[r] / sum - if r == tgt { 1.0 } else { 0.0 }) / std::f64::consts::LN_2 * scale
            })
            .collect();
        let mut want_wo = vec![0f64; v * dv];
        let mut want_e = vec![0f64; dv];
        for r in 0..v {
            let s = core.w_o.shift(r);
            let sgn = (1i64 << s) as f64;
            for j in 0..dv {
                want_wo[r * dv + j] = gz[r] * tr.h[j] as f64;
                want_e[j] += gz[r] * (core.w_o.weight(r, j) as f64 * sgn);
            }
        }
        let dec = 1.0 / (1u64 << tr.shift) as f64;
        for j in 0..dv {
            want_e[j] *= tr.mask[j] as f64 * dec;
        }
        let mut t2 = t.clone();
        t2.zero_grads();
        let (_b, nn) = t2.accumulate_with(&core, &seq, Some(&mask));
        assert_eq!(nn, 1);
        for i in 0..v * dv {
            assert!(
                (t2.gwo[i] as f64 - want_wo[i]).abs() < 1e-4,
                "gWo[{i}]: {} vs {}",
                t2.gwo[i],
                want_wo[i]
            );
        }
        let prev = seq[0] as usize;
        for j in 0..dv {
            assert!(
                (t2.ge_old[prev * dv + j] as f64 - want_e[j]).abs() < 1e-4,
                "gE_old[{j}]: {} vs {}",
                t2.ge_old[prev * dv + j],
                want_e[j]
            );
            assert!((t2.ge_new[seq[1] as usize * dv + j] as f64 - want_e[j]).abs() < 1e-4);
        }
        assert!(
            want_e.iter().any(|x| x.abs() > 1e-9),
            "the fixture must be non-vacuous"
        );
        // The embedding credit is zero when the exported output weights are all zero, which is why a
        // zero-code initialization cannot train the prior tables at step 0.
        let zero = PriorTrainer::new(cfg(v, dv), 29, vec![0i8; v], 0.0).unwrap();
        let zc = zero.to_core().unwrap();
        assert!(zc.w_o.packed().iter().all(|b| *b == 0));
    }

    #[test]
    fn a_mid_pass_checkpoint_continues_exactly() {
        let v = 70usize;
        let windows: Vec<Vec<u32>> = (0..11u32)
            .map(|k| (0..6u32).map(|j| k * 6 + j).collect())
            .collect();
        let mut a = PriorTrainer::new(cfg(v, 8), 31, flat_bias(v), 0.3).unwrap();
        a.cfg_train.beta1 = 0.7;
        a.cfg_train.beta2 = 0.98;
        a.cfg_train.weight_decay = 0.002;
        // Two batches of 4 in an 11-window pass: firmly mid-pass, then a partial batch at the end.
        let mut consumed = Vec::new();
        for _ in 0..2 {
            let b = a.next_batch(&windows, 4);
            consumed.extend(b.clone());
            a.train_batch_bits(&b);
        }
        assert!(
            !a.pass_complete(&windows),
            "the checkpoint must be mid-pass"
        );
        let ckpt = a.checkpoint_bytes();
        let mut c = PriorTrainer::new(cfg(v, 8), 31, flat_bias(v), 0.3).unwrap();
        c.cfg_train.beta1 = 0.7;
        c.cfg_train.beta2 = 0.98;
        c.cfg_train.weight_decay = 0.002;
        c.resume_from(&ckpt).expect("mid-pass resume");
        assert_eq!(c.perm, a.perm, "the active permutation must be restored");
        assert_eq!(c.cursor, a.cursor);
        assert_eq!(c.e_old, a.e_old);
        assert_eq!(c.me_old, a.me_old);
        assert_eq!(c.ve_old, a.ve_old);
        // Continue to the end of the pass and across the boundary; consumed IDs must match exactly.
        let mut consumed_b = consumed.clone();
        loop {
            let nb_a = a.next_batch(&windows, 4);
            let nb_c = c.next_batch(&windows, 4);
            assert_eq!(nb_a, nb_c, "next batch after resume");
            if nb_a.is_empty() {
                break;
            }
            consumed_b.extend(nb_a.clone());
            a.train_batch_bits(&nb_a);
            c.train_batch_bits(&nb_c);
            if nb_a.len() < 4 {
                // cross the pass boundary once more
                let nb_a2 = a.next_batch(&windows, 4);
                let nb_c2 = c.next_batch(&windows, 4);
                assert_eq!(nb_a2, nb_c2, "first batch of the next pass");
                break;
            }
        }
        if a.e_old != c.e_old {
            let i = a
                .e_old
                .iter()
                .zip(&c.e_old)
                .position(|(x, y)| x != y)
                .unwrap();
            eprintln!(
                "first e_old divergence at {i}: {} vs {} | steps {} vs {} cursors {} vs {} pass {} vs {} rng {} vs {} perm_eq {}",
                a.e_old[i], c.e_old[i], a.step, c.step, a.cursor, c.cursor, a.pass, c.pass, a.rng, c.rng, a.perm == c.perm
            );
        }
        assert_eq!(a.e_old, c.e_old);
        assert_eq!(a.wo, c.wo);
        assert_eq!(a.step, c.step);
        assert_eq!(a.to_core().unwrap(), c.to_core().unwrap());
        assert_eq!(
            consumed_b.len(),
            consumed.len() + 3,
            "11 = 4 + 4 + 3 consumed"
        );
        for w in &windows {
            assert_eq!(
                consumed_b.iter().filter(|x| *x == w).count(),
                1,
                "no replacement"
            );
        }
        // Invalid state must be refused before mutating the live trainer.
        // A genuine rejection: truncation is structural, unlike flipping a bias-code byte, which is a
        // perfectly valid load.
        let broken = ckpt[..ckpt.len() - 1].to_vec();
        let mut d = PriorTrainer::new(cfg(v, 8), 31, flat_bias(v), 0.3).unwrap();
        let before = (d.e_old.clone(), d.me_old.clone(), d.step, d.cursor);
        let _ = d.resume_from(&broken);
        assert_eq!(before.0, d.e_old);
        assert_eq!(before.1, d.me_old);
        assert_eq!(before.2, d.step);
        assert_eq!(before.3, d.cursor);
    }

    #[test]
    fn the_frozen_bias_reproduces_the_fit_unigram_up_to_quantization() {
        let mut counts = vec![1u64; 16];
        counts[0] = 10_000;
        counts[7] = 500;
        let total: u64 = counts.iter().sum();
        let codes = bias_codes_from_counts(&counts, total, 16);
        let core = PriorTrainer::new(cfg(16, 8), 1, codes, 0.0)
            .unwrap()
            .to_core()
            .unwrap();
        let seq: Vec<u32> = vec![0, 7, 0, 7, 0, 3, 7, 0];
        let (total_bits, n) = core.bits(&seq, true);
        let bits = total_bits / n as f64;
        let z0 = core.int_logits(core.cfg.pad_row(), 0, true);
        let z1 = core.int_logits(0, 7, true);
        assert_eq!(
            z0, z1,
            "with flat bias the score is constant across positions"
        );
        let exact: f64 = seq[1..]
            .iter()
            .map(|&t| {
                let p = (counts[t as usize] as f64 + 1.0) / (total as f64 + 16.0);
                -p.ln() / std::f64::consts::LN_2
            })
            .sum::<f64>()
            / n as f64;
        eprintln!("authored-sequence quantization gap: {bits:.4} vs {exact:.4} bits");
        assert!((bits - exact).abs() < 1.2);
    }
}
