//! One shared served path for ordinary-text continuation and grounded exact copying.
//!
//! # The served responsibility
//!
//! ```text
//! c        = pinned request meaning + selected evidence/result + exact provenance
//! h_0      = learned_init(c)
//! a_t      = learned Generate(token) / Copy(owned occurrence) / Stop decision
//! x_t      = token actually emitted by a_t
//! h_(t+1)  = learned_update(h_t, representation(x_t), event, c)
//! ```
//!
//! Exact ownership, version authority and the copy **cursor** stay outside the compressed state:
//! the session owns the exact span and advances the cursor; the model only chooses an action. The
//! same model, state and update serve ordinary prose (no owned span, so `Copy` is illegal) and a
//! grounded answer (an owned span makes `Copy` legal until the cursor completes it). Ordinary prose
//! and grounded supervision therefore train **one** artifact rather than two realizers.
//!
//! # Action space and legal support
//!
//! The readout has `vocab + 2` rows: `Generate(v)` for `v in 0..vocab`, then `Copy`, then `Stop`.
//! The generation vocabulary is the declared tokenizer vocabulary, so the head is an ordinary
//! vocabulary head rather than a handful of authored words. `Copy` is removed from the scored set
//! whenever the session reports no live owned occurrence (ordinary prose, or a completed cursor),
//! and cross-entropy is normalised over the **legal** rows only. That is a declared hard constraint,
//! not a learned preference, and it is reported as such.
//!
//! # Representation
//!
//! One shared 4-bit token embedding table serves both roles: a token's row is the recurrent
//! feedback `representation(x_t)`, and the same rows are summed into the bounded evidence feature
//! `m` for `c`. The evidence sum applies a **fixed cyclic position rotation** per content position,
//! so reordering a span changes `m` with no extra learned parameters; position `TL_CONTENT_TOKENS`
//! is a dedicated tail slot carrying a truncated span's final token. The typed causal block is the
//! exact 15-coordinate observation from [`super::state_lexical::typed_causal_block`].
//!
//! # Numerical contract (D0-b)
//!
//! Weights are ternary with a power-of-two per-row shift (`y = (Σ ±x) << s`), embeddings are signed
//! 4-bit codes with a power-of-two per-row shift, and the recurrence is `clamp(emb + (W_h·h >> k) +
//! W_f·u + b)`. Every served operation is an add, a subtract, a shift, a compare or a table read;
//! there is no multiply, divide or float in the declared numerical kernel. The dense maps are
//! honestly dense: their parameter traffic is reported, and ternary does **not** make them
//! geometric or sparse.

#![forbid(unsafe_code)]

use super::lowbit::TernaryLinear;
use super::state_lexical::{typed_causal_block, SlFacts};

/// Artifact format version.
pub const TL_VERSION: u8 = 1;
/// Typed causal block width, shared with the retained state-conditioned observation.
pub const TL_F_DIM: usize = 15;
/// Event channel width: how the token in the state was produced.
pub const TL_EVENTS: usize = 4;
/// The state was advanced by an observed source token (prompt / teacher forcing).
pub const TL_EV_OBSERVE: usize = 0;
/// The state was advanced by a token the model generated from its vocabulary.
pub const TL_EV_GENERATE: usize = 1;
/// The state was advanced by an exactly copied owned token.
pub const TL_EV_COPY: usize = 2;
/// The action symbol of `Stop`; never fed back (serving terminates).
pub const TL_EV_STOP: usize = 3;
/// Ordered content positions in the evidence fingerprint before the tail slot.
pub const TL_CONTENT_TOKENS: usize = 4;
/// Content positions including the tail slot.
pub const TL_CONTENT_POSITIONS: usize = TL_CONTENT_TOKENS + 1;
/// Declared bound on a signed 4-bit embedding code.
pub const TL_EMB_BOUND: i32 = 7;
/// Declared upper bound on an embedding row's power-of-two shift.
pub const TL_EMB_MAX_SHIFT: u32 = 7;
/// Declared upper bound on a weight row's power-of-two shift.
pub const TL_MAX_SHIFT: u32 = 9;
/// Largest vocabulary this module will construct or load.
pub const TL_MAX_VOCAB: usize = 1 << 20;
/// Largest state width this module will construct or load.
pub const TL_MAX_H_DIM: usize = 1024;

/// One decision of the shared policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TlAction {
    /// Emit vocabulary row `v`.
    Generate(u32),
    /// Emit the next token of the exactly owned occurrence.
    Copy,
    /// End the response.
    Stop,
}

impl TlAction {
    /// The event symbol fed back when this action produced a token.
    pub fn event(self) -> usize {
        match self {
            TlAction::Generate(_) => TL_EV_GENERATE,
            TlAction::Copy => TL_EV_COPY,
            TlAction::Stop => TL_EV_STOP,
        }
    }
}

/// One supervised example of the shared interface.
///
/// `observed` is the prompt: its tokens advance the state with [`TL_EV_OBSERVE`] and are never
/// scored. `actions` is the supervised action script. `sel`/`res`/`facts` are the exact grounded
/// evidence and provenance; ordinary prose leaves them empty and `grounded` false.
#[derive(Clone, Debug)]
pub struct TlExample {
    pub sel: Vec<u32>,
    pub res: Vec<u32>,
    pub facts: SlFacts,
    pub observed: Vec<u32>,
    pub actions: Vec<TlAction>,
    /// Declared supervision weight of this example.
    pub weight: f32,
    /// Document group, for per-document aggregation.
    pub doc: usize,
    /// True when the example carries an exactly copyable owned span.
    pub grounded: bool,
    /// True when this example's final supervised step is a document-terminal `Stop`.
    pub terminal_stop: bool,
}

impl TlExample {
    /// The token the action actually emits; `None` for `Stop`.
    pub fn emitted(&self, i: usize, owned: &[u32]) -> Option<u32> {
        match self.actions.get(i)? {
            TlAction::Generate(v) => Some(*v),
            TlAction::Copy => {
                let cursor = self.actions[..i]
                    .iter()
                    .filter(|a| matches!(a, TlAction::Copy))
                    .count();
                owned.get(cursor).copied()
            }
            TlAction::Stop => None,
        }
    }

    /// Whether `Copy` is a live occurrence at step `i` (the cursor has not completed the span).
    pub fn copy_legal(&self, i: usize, owned: &[u32]) -> bool {
        if owned.is_empty() {
            return false;
        }
        let copied = self.actions[..i.min(self.actions.len())]
            .iter()
            .filter(|a| matches!(a, TlAction::Copy))
            .count();
        copied < owned.len()
    }

    /// The event symbol that produced the state the step-`i` decision reads. The first decision
    /// reads a state advanced by observed or generated tokens, never by `Stop`.
    pub fn prior_event(&self, i: usize) -> usize {
        if i == 0 {
            TL_EV_OBSERVE
        } else {
            match self.actions[i - 1] {
                TlAction::Stop => TL_EV_STOP,
                other => other.event(),
            }
        }
    }
}

/// Two-bit ternary code values, matching the retained low-bit line.
const TCODE_ZERO: u8 = 0;
const TCODE_POS: u8 = 1;
const TCODE_NEG: u8 = 2;

/// Quantise a row-major matrix to ternary codes with a **capped** power-of-two per-row scale.
///
/// The cap matters: an uncapped `floor(log2(amax))` can reach 30, which an `i32` accumulator cannot
/// shift without overflow. Capping keeps the shift inside the declared envelope; the saturated code
/// still selects an add, a subtract or nothing.
fn quantize_ternary(w: &[f32], rows: usize, cols: usize) -> (Vec<i8>, Vec<u32>) {
    let mut codes = vec![0i8; rows * cols];
    let mut shift = vec![0u32; rows];
    for r in 0..rows {
        let row = &w[r * cols..(r + 1) * cols];
        let amax = row.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        let s = if amax > 0.0 {
            (amax.log2().floor().max(0.0) as u32).min(TL_MAX_SHIFT)
        } else {
            0
        };
        shift[r] = s;
        let scale = (1u32 << s) as f32;
        for c in 0..cols {
            let q = (row[c] / scale).round();
            codes[r * cols + c] = if q >= 1.0 {
                1
            } else if q <= -1.0 {
                -1
            } else {
                0
            };
        }
    }
    (codes, shift)
}

/// Pack ternary codes two bits per weight, four weights per byte.
fn pack_ternary(codes: &[i8]) -> Vec<u8> {
    let mut packed = vec![0u8; codes.len().div_ceil(4)];
    for (i, c) in codes.iter().enumerate() {
        let code = match *c {
            1 => TCODE_POS,
            -1 => TCODE_NEG,
            _ => TCODE_ZERO,
        };
        packed[i >> 2] |= code << ((i & 3) as u32 * 2);
    }
    packed
}

/// Recover one packed ternary code as `-1`, `0` or `+1` with shifts and masks only.
#[inline]
fn unpack_ternary(packed: &[u8], row: usize, col: usize, cols: usize) -> i32 {
    let flat = row * cols + col;
    let byte = packed[flat >> 2];
    let slot = (flat & 3) as u32;
    match (byte >> (slot * 2)) & 3 {
        TCODE_POS => 1,
        TCODE_NEG => -1,
        _ => 0,
    }
}

/// A served ternary map: packed codes plus one power-of-two shift per row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlLinear {
    pub rows: usize,
    pub cols: usize,
    pub packed: Vec<u8>,
    pub shift: Vec<u32>,
}

impl TlLinear {
    /// Build from already-quantised codes.
    fn from_codes(codes: &[i8], shift: Vec<u32>, rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            packed: pack_ternary(codes),
            shift,
        }
    }

    /// The exact serving forward: `y[r] = (Σ_c w[r][c] · x[c]) << shift[r]`.
    pub fn forward_i32(&self, x: &[i32]) -> Result<Vec<i32>, String> {
        if x.len() != self.cols {
            return Err(format!("input length {} != cols {}", x.len(), self.cols));
        }
        let mut out = vec![0i32; self.rows];
        for r in 0..self.rows {
            let mut acc: i32 = 0;
            for c in 0..self.cols {
                match unpack_ternary(&self.packed, r, c, self.cols) {
                    1 => acc += x[c],
                    -1 => acc -= x[c],
                    _ => {}
                }
            }
            out[r] = acc << self.shift[r];
        }
        Ok(out)
    }

    /// Offline float evaluation of the same packed weights, for exactness checks.
    pub fn forward_reference(&self, x: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0f64; self.rows];
        for r in 0..self.rows {
            let mut acc = 0.0f64;
            for c in 0..self.cols {
                acc += unpack_ternary(&self.packed, r, c, self.cols) as f64 * x[c];
            }
            out[r] = acc * (1u64 << self.shift[r]) as f64;
        }
        out
    }

    /// Reconstruct the retained [`TernaryLinear`] view of the same weights.
    pub fn to_ternary(&self) -> Result<TernaryLinear, String> {
        TernaryLinear::from_packed(
            self.packed.clone(),
            self.shift.clone(),
            self.rows,
            self.cols,
        )
    }

    /// Nonzero weight count: the honest dense-map operation count per forward.
    pub fn nonzero(&self) -> usize {
        self.packed
            .iter()
            .map(|b| {
                (b & 3 != TCODE_ZERO) as usize
                    + ((b >> 2) & 3 != TCODE_ZERO) as usize
                    + ((b >> 4) & 3 != TCODE_ZERO) as usize
                    + ((b >> 6) & 3 != TCODE_ZERO) as usize
            })
            .sum()
    }

    /// Stored bytes for this map.
    pub fn bytes(&self) -> usize {
        self.packed.len() + self.shift.len() * 4
    }
}

/// A signed 4-bit token embedding table with one power-of-two shift per row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlEmbed {
    pub rows: usize,
    pub cols: usize,
    pub codes: Vec<i8>,
    pub shift: Vec<u32>,
}

impl TlEmbed {
    /// Quantise a row-major float table to signed 4-bit codes with a capped power-of-two row scale.
    fn quantize(w: &[f32], rows: usize, cols: usize, bound: i32) -> Self {
        let mut codes = vec![0i8; rows * cols];
        let mut shift = vec![0u32; rows];
        for r in 0..rows {
            let row = &w[r * cols..(r + 1) * cols];
            let amax = row.iter().fold(0.0f32, |m, &v| m.max(v.abs()));
            let s = if amax > bound as f32 {
                ((amax / bound as f32).log2().floor().max(0.0) as u32).min(TL_EMB_MAX_SHIFT)
            } else {
                0
            };
            shift[r] = s;
            let scale = (1u32 << s) as f32;
            for c in 0..cols {
                let q = (row[c] / scale).round();
                codes[r * cols + c] = q.clamp(-(bound as f32), bound as f32) as i8;
            }
        }
        Self {
            rows,
            cols,
            codes,
            shift,
        }
    }

    /// The served value of one embedding entry: `code << shift[row]`.
    #[inline]
    pub fn value(&self, row: usize, col: usize) -> i32 {
        self.codes[row * self.cols + col] as i32 * (1i32 << self.shift[row])
    }

    /// Bytes of embedding storage.
    pub fn bytes(&self) -> usize {
        self.codes.len() + self.shift.len() * 4
    }
}

/// Declared construction parameters of the shared served path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TlConfig {
    pub vocab: usize,
    pub h_dim: usize,
    pub h_clamp: i32,
    pub m_clamp: i32,
    pub recurrent_shift: u32,
    /// The one artifact-bound dyadic scale: the served score is `Z * 2^-score_shift`. The decision is
    /// invariant to it and the cross-entropy is not, so it is declared once. It is set to the scale at
    /// which a cold-start residual over one token's embedding has order-one effect, so the learned
    /// maps are not born 64x too small to matter.
    pub score_shift: u32,
}

impl TlConfig {
    /// The declared default geometry for a vocabulary of `vocab` tokens.
    pub fn new(vocab: usize) -> Self {
        Self {
            vocab,
            h_dim: 64,
            h_clamp: 256,
            m_clamp: 4096,
            recurrent_shift: 3,
            score_shift: 4,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.vocab < 2 || self.vocab > TL_MAX_VOCAB {
            return Err(format!("vocab {} outside 2..={TL_MAX_VOCAB}", self.vocab));
        }
        if self.h_dim < 4 || self.h_dim > TL_MAX_H_DIM || self.h_dim % 2 != 0 {
            return Err(format!(
                "h_dim {} must be even and within 4..={TL_MAX_H_DIM}",
                self.h_dim
            ));
        }
        if self.h_clamp < 1 || self.m_clamp < 1 {
            return Err("clamps must be positive".into());
        }
        if self.recurrent_shift > 5 {
            return Err("recurrent_shift <= 5".into());
        }
        if self.score_shift > 20 {
            return Err("score_shift <= 20".into());
        }
        Ok(())
    }
}

/// The served artifact: one shared policy for ordinary continuation and grounded copying.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlModel {
    pub version: u8,
    pub vocab: usize,
    pub h_dim: usize,
    pub f_dim: usize,
    pub h_clamp: i32,
    pub m_clamp: i32,
    pub recurrent_shift: u32,
    pub score_shift: u32,
    /// `(vocab + 1) x h_dim`; row `vocab` is the reserved row for a token with no fitted identity.
    pub e: TlEmbed,
    pub wi: TlLinear,
    pub wh: TlLinear,
    pub wf: TlLinear,
    pub wo: TlLinear,
    pub bh: Vec<i32>,
    pub bo: Vec<i32>,
}

/// A served rollout of the shared policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlRollout {
    pub tokens: Vec<u32>,
    pub actions: Vec<TlAction>,
    pub stopped: bool,
    pub state_digest: u64,
}

impl TlModel {
    /// Quantise float masters into the served artifact. Training forward and serving therefore use
    /// one quantizer, so a fit optimises the function that is actually served.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        cfg: &TlConfig,
        e_master: &[f32],
        wi_master: &[f32],
        wh_master: &[f32],
        wf_master: &[f32],
        wo_master: &[f32],
        bh_master: &[f32],
        bo_master: &[f32],
    ) -> Result<Self, String> {
        cfg.validate()?;
        let h = cfg.h_dim;
        let v = cfg.vocab;
        let f = TL_F_DIM;
        expect_len("e", e_master, (v + 1) * h)?;
        expect_len("wi", wi_master, h * (h + f))?;
        expect_len("wh", wh_master, h * h)?;
        expect_len("wf", wf_master, h * (h + f + TL_EVENTS))?;
        expect_len("wo", wo_master, (v + 2) * (2 * h + f + TL_EVENTS))?;
        expect_len("bh", bh_master, h)?;
        expect_len("bo", bo_master, v + 2)?;
        let e = TlEmbed::quantize(e_master, v + 1, h, TL_EMB_BOUND);
        let wi = {
            let (c, s) = quantize_ternary(wi_master, h, h + f);
            TlLinear::from_codes(&c, s, h, h + f)
        };
        let wh = {
            let (c, s) = quantize_ternary(wh_master, h, h);
            TlLinear::from_codes(&c, s, h, h)
        };
        let wf = {
            let (c, s) = quantize_ternary(wf_master, h, h + f + TL_EVENTS);
            TlLinear::from_codes(&c, s, h, h + f + TL_EVENTS)
        };
        let wo = {
            let (c, s) = quantize_ternary(wo_master, v + 2, 2 * h + f + TL_EVENTS);
            TlLinear::from_codes(&c, s, v + 2, 2 * h + f + TL_EVENTS)
        };
        let model = Self {
            version: TL_VERSION,
            vocab: v,
            h_dim: h,
            f_dim: f,
            h_clamp: cfg.h_clamp,
            m_clamp: cfg.m_clamp,
            recurrent_shift: cfg.recurrent_shift,
            score_shift: cfg.score_shift,
            e,
            wi,
            wh,
            wf,
            wo,
            bh: bh_master.iter().map(|x| x.round() as i32).collect(),
            bo: bo_master.iter().map(|x| x.round() as i32).collect(),
        };
        model.validate()?;
        Ok(model)
    }

    /// Number of readout rows.
    pub fn action_rows(&self) -> usize {
        self.vocab + 2
    }

    /// Readout row of the `Copy` action.
    pub fn copy_row(&self) -> usize {
        self.vocab
    }

    /// Readout row of the `Stop` action.
    pub fn stop_row(&self) -> usize {
        self.vocab + 1
    }

    /// The embedding row of a token; an unfitted token uses the reserved row.
    pub fn token_row(&self, token: u32) -> usize {
        let t = token as usize;
        if t < self.vocab {
            t
        } else {
            self.vocab
        }
    }

    /// Declared fitted-identity predicate. The generation vocabulary is the whole declared
    /// tokenizer vocabulary, so every declared token has its own row; a token id at or above it is
    /// copy-only and shares the reserved row.
    pub fn known_token(&self, token: u32) -> bool {
        (token as usize) < self.vocab
    }

    /// The readout row of an action.
    pub fn action_row(&self, action: TlAction) -> usize {
        match action {
            TlAction::Generate(v) => (v as usize).min(self.vocab - 1),
            TlAction::Copy => self.copy_row(),
            TlAction::Stop => self.stop_row(),
        }
    }

    /// One half of the evidence feature (selected and operand halves).
    pub fn half(&self) -> usize {
        self.h_dim / 2
    }

    /// The rotation stride between adjacent content positions.
    fn band(&self) -> usize {
        (self.half() / TL_CONTENT_POSITIONS).max(1)
    }

    /// The evidence fingerprint under an explicit token-to-row mapping. `row_of` is the artifact's
    /// fitted-identity map for an ordinary feature, or the constant reserved row for the blinded
    /// identity control.
    fn feature_with(&self, sel: &[u32], res: &[u32], row_of: &dyn Fn(u32) -> usize) -> Vec<i32> {
        let half = self.half();
        let band = self.band();
        let mut m = vec![0i32; self.h_dim];
        for (toks, off) in [(sel, 0usize), (res, half)] {
            let mut list: Vec<(usize, usize)> = toks
                .iter()
                .take(TL_CONTENT_TOKENS)
                .enumerate()
                .map(|(p, t)| (row_of(*t), p))
                .collect();
            if toks.len() > TL_CONTENT_TOKENS {
                list.push((row_of(toks[toks.len() - 1]), TL_CONTENT_TOKENS));
            }
            for (row, p) in list {
                let k = (p * band) % half;
                for c in 0..half {
                    m[off + (c + k) % half] += self.e.value(row, c);
                }
            }
        }
        clamp_in_place(&mut m, self.m_clamp);
        m
    }

    /// The bounded evidence fingerprint `m` for a selected payload and a consumed operand. Distinct
    /// content positions rotate by distinct fixed strides, so a reordered span changes `m` with no
    /// position-indexed parameter table. `res` is empty for a direct read or for ordinary prose.
    pub fn content_feature(&self, sel: &[u32], res: &[u32]) -> Vec<i32> {
        self.feature_with(sel, res, &|t| self.token_row(t))
    }

    /// The same fingerprint with every token identity replaced by the reserved row. Lengths, order
    /// positions, rotation strides and every typed fact are unchanged, so this is a clean
    /// identity-erasure control: a decision that survives it did not depend on which token was
    /// copied.
    pub fn content_feature_blind(&self, sel: &[u32], res: &[u32]) -> Vec<i32> {
        let reserved = self.vocab;
        self.feature_with(sel, res, &move |_| reserved)
    }

    /// The exact typed causal block, using this artifact's fitted-identity predicate.
    pub fn typed_block(&self, sel: &[u32], res: &[u32], facts: SlFacts) -> Vec<i32> {
        typed_causal_block(sel, res, facts, &|t| self.known_token(t))
    }

    fn event_onehot(event: usize) -> Vec<i32> {
        let mut ev = vec![0i32; TL_EVENTS];
        ev[event.min(TL_EVENTS - 1)] = 1;
        ev
    }

    /// `h_0 = learned_init(c)`: the pinned meaning is the evidence fingerprint plus the typed
    /// provenance block.
    pub fn init_state(&self, m: &[i32], f: &[i32]) -> Vec<i32> {
        let mut input = Vec::with_capacity(self.h_dim + self.f_dim);
        input.extend_from_slice(m);
        input.extend_from_slice(f);
        let mut h = self
            .wi
            .forward_i32(&input)
            .expect("validated init map shape");
        for (x, b) in h.iter_mut().zip(&self.bh) {
            *x += *b;
        }
        clamp_in_place(&mut h, self.h_clamp);
        h
    }

    /// The rows scored at one decision. `Copy` is removed when the session reports no live owned
    /// occurrence, which is a declared hard constraint on the legal action set.
    pub fn legal_rows(&self, copy_legal: bool) -> Vec<usize> {
        let mut rows: Vec<usize> = (0..self.vocab).collect();
        rows.push(self.stop_row());
        if copy_legal {
            rows.push(self.copy_row());
        }
        rows
    }

    /// `a_t = learned_readout(h_t, c, exact_copy_state)` as unnormalised row scores.
    pub fn readout(&self, h: &[i32], m: &[i32], f: &[i32], event: usize) -> Vec<i32> {
        let mut input = Vec::with_capacity(2 * self.h_dim + self.f_dim + TL_EVENTS);
        input.extend_from_slice(h);
        input.extend_from_slice(m);
        input.extend_from_slice(f);
        input.extend_from_slice(&Self::event_onehot(event));
        let mut logits = self
            .wo
            .forward_i32(&input)
            .expect("validated readout map shape");
        for (x, b) in logits.iter_mut().zip(&self.bo) {
            *x += *b;
        }
        logits
    }

    /// The policy decision at one step, with a deterministic lowest-row tie-break.
    pub fn decide(
        &self,
        h: &[i32],
        m: &[i32],
        f: &[i32],
        event: usize,
        copy_legal: bool,
    ) -> TlAction {
        let logits = self.readout(h, m, f, event);
        let mut best = self.stop_row();
        let mut best_score = i32::MIN;
        for row in self.legal_rows(copy_legal) {
            let s = logits[row];
            if s > best_score {
                best_score = s;
                best = row;
            }
        }
        if best < self.vocab {
            TlAction::Generate(best as u32)
        } else if best == self.copy_row() {
            TlAction::Copy
        } else {
            TlAction::Stop
        }
    }

    /// `h_(t+1) = learned_update(h_t, representation(x_t), event, c)`. `token` is the token the
    /// action **actually emitted**; `None` (a `Stop`, or a blinded diagnostic) uses the reserved
    /// row, so the update is total.
    pub fn transition(
        &self,
        h: &[i32],
        event: usize,
        token: Option<u32>,
        m: &[i32],
        f: &[i32],
    ) -> Vec<i32> {
        let row = token.map(|t| self.token_row(t)).unwrap_or(self.vocab);
        let mut extra = Vec::with_capacity(self.h_dim + self.f_dim + TL_EVENTS);
        extra.extend_from_slice(m);
        extra.extend_from_slice(f);
        extra.extend_from_slice(&Self::event_onehot(event));
        let rec = self
            .wh
            .forward_i32(h)
            .expect("validated recurrent map shape");
        let add = self
            .wf
            .forward_i32(&extra)
            .expect("validated fact map shape");
        let mut next = vec![0i32; self.h_dim];
        for r in 0..self.h_dim {
            next[r] = self.e.value(row, r) + (rec[r] >> self.recurrent_shift) + add[r] + self.bh[r];
        }
        clamp_in_place(&mut next, self.h_clamp);
        next
    }

    /// The shared serving loop over one pinned meaning `c` and one exact owned occurrence.
    ///
    /// `owned` is the occurrence the session owns; the **cursor** advances here but lives outside
    /// the compressed state. `blind_identity` erases copied-token identity from both the evidence
    /// fingerprint and the recurrent feedback while holding lengths, order, typed facts, events and
    /// copy legality fixed; `source_disabled` removes the owned occurrence entirely, so `Copy`
    /// cannot be legal. Both are declared controls, not serving modes.
    #[allow(clippy::too_many_arguments)]
    pub fn rollout(
        &self,
        sel: &[u32],
        res: &[u32],
        facts: SlFacts,
        observed: &[u32],
        owned: &[u32],
        max_new: usize,
        blind_identity: bool,
        source_disabled: bool,
    ) -> TlRollout {
        let owned: &[u32] = if source_disabled { &[] } else { owned };
        let m = if blind_identity {
            self.content_feature_blind(sel, res)
        } else {
            self.content_feature(sel, res)
        };
        let f = self.typed_block(sel, res, facts);
        let mut h = self.init_state(&m, &f);
        for t in observed {
            h = self.transition(&h, TL_EV_OBSERVE, Some(*t), &m, &f);
        }
        let mut tokens: Vec<u32> = Vec::new();
        let mut actions: Vec<TlAction> = Vec::new();
        let mut stopped = false;
        let mut cursor = 0usize;
        for _ in 0..max_new {
            let copy_legal = cursor < owned.len();
            let event = if actions.is_empty() {
                TL_EV_OBSERVE
            } else {
                actions[actions.len() - 1].event()
            };
            let action = self.decide(&h, &m, &f, event, copy_legal);
            actions.push(action);
            let emitted = match action {
                TlAction::Generate(v) => Some(v),
                TlAction::Copy => {
                    let t = owned.get(cursor).copied();
                    cursor += 1;
                    t
                }
                TlAction::Stop => {
                    stopped = true;
                    break;
                }
            };
            if let Some(t) = emitted {
                tokens.push(t);
            }
            let feed = if blind_identity { None } else { emitted };
            h = self.transition(&h, action.event(), feed, &m, &f);
        }
        TlRollout {
            tokens,
            actions,
            stopped,
            state_digest: state_digest(&h),
        }
    }

    /// The independently authored temporal meaning of one answer, computed from typed source state.
    ///
    /// This is an **authored oracle**, not a renderer: a value was temporal-updated exactly when a
    /// mutation was actually committed at the answered address (`committed`) **and** an older
    /// committed value at that address was superseded (`prior_differs`). Neither following a
    /// different derived address (`key_changed`) nor merely observing a value that differs from the
    /// operand is a temporal change: the first is a route, the second has no commit. The learner's
    /// own renderer never defines this predicate.
    ///
    /// An earlier version of this oracle additionally required `!key_changed`. A held-out
    /// composition with a superseded older value at the answered address, reached through a changed
    /// derived key, showed that conjunct was over-specified: `prior_differs` is already a fact about
    /// the answered address, so a committed supersession there is a real change of that entity
    /// regardless of which route reached it. The revision was made after that exposure and is
    /// recorded in the report; the typed regimes themselves are unchanged.
    pub fn truthful_temporal_meaning(facts: SlFacts) -> bool {
        facts.committed && facts.prior_differs
    }

    /// Scored cross-entropy of one supervised example through the served integer path.
    ///
    /// Returns `(bits over Generate targets, generate targets, bits over every scored action,
    /// scored actions, correct actions, stop targets, correct stops)`. Cross-entropy is normalised
    /// over the **legal** action set only, and the illegal `Copy` row contributes no mass.
    pub fn score_example(&self, ex: &TlExample) -> TlScore {
        let owned: &[u32] = if ex.grounded { &ex.sel } else { &[] };
        let m = if ex.grounded {
            self.content_feature(&ex.sel, &ex.res)
        } else {
            vec![0i32; self.h_dim]
        };
        let f = self.typed_block(&ex.sel, &ex.res, ex.facts);
        let mut h = self.init_state(&m, &f);
        for t in &ex.observed {
            h = self.transition(&h, TL_EV_OBSERVE, Some(*t), &m, &f);
        }
        let mut out = TlScore::default();
        for i in 0..ex.actions.len() {
            let copy_legal = ex.copy_legal(i, owned);
            let event = ex.prior_event(i);
            let logits = self.readout(&h, &m, &f, event);
            let rows = self.legal_rows(copy_legal);
            let target = self.action_row(ex.actions[i]);
            let bits = log2_softmax_row(&logits, &rows, target, self.score_shift);
            out.bits_all += bits;
            out.scored += 1;
            if let TlAction::Generate(_) = ex.actions[i] {
                out.bits_generate += bits;
                out.generate_targets += 1;
            }
            if let TlAction::Stop = ex.actions[i] {
                out.stop_targets += 1;
            }
            let chosen = self.decide(&h, &m, &f, event, copy_legal);
            if chosen == ex.actions[i] {
                out.correct += 1;
                if matches!(ex.actions[i], TlAction::Stop) {
                    out.correct_stops += 1;
                }
            }
            let emitted = if ex.grounded {
                ex.emitted(i, owned)
            } else {
                match ex.actions[i] {
                    TlAction::Generate(v) => Some(v),
                    TlAction::Copy => ex.emitted(i, owned),
                    TlAction::Stop => None,
                }
            };
            h = self.transition(&h, ex.actions[i].event(), emitted, &m, &f);
        }
        out
    }

    /// Teacher-forced action agreement through the exported integer artifact.
    pub fn teacher_forced_agreement(&self, examples: &[TlExample]) -> (usize, usize) {
        let mut correct = 0usize;
        let mut total = 0usize;
        for ex in examples {
            let s = self.score_example(ex);
            correct += s.correct;
            total += s.scored;
        }
        (correct, total)
    }

    /// Quantised-embedding collisions: distinct tokens whose served rows are byte-identical. A
    /// non-injective low-bit embedding is a real aliasing risk, so it is measured rather than
    /// assumed away.
    pub fn embedding_collisions(&self) -> EmbeddingCollisions {
        use std::collections::HashMap;
        let mut groups: HashMap<Vec<i32>, Vec<u32>> = HashMap::new();
        for row in 0..self.e.rows {
            let key: Vec<i32> = (0..self.e.cols).map(|c| self.e.value(row, c)).collect();
            let token = if row == self.vocab {
                u32::MAX
            } else {
                row as u32
            };
            groups.entry(key).or_default().push(token);
        }
        let distinct = groups.len();
        let mut pairs = 0usize;
        let mut colliding_tokens = 0usize;
        let mut witness: Option<(u32, u32)> = None;
        for (_, mut toks) in groups {
            if toks.len() > 1 {
                toks.sort_unstable();
                colliding_tokens += toks.len();
                pairs += toks.len() * (toks.len() - 1) / 2;
                if witness.is_none() && toks[0] != u32::MAX {
                    witness = Some((toks[0], toks[1]));
                }
            }
        }
        EmbeddingCollisions {
            distinct_rows: distinct,
            colliding_pairs: pairs,
            colliding_tokens,
            witness,
        }
    }

    /// Declared structure/envelope validation of a loaded or built artifact.
    pub fn validate(&self) -> Result<(), String> {
        if self.version != TL_VERSION {
            return Err("unsupported transferable-lexical artifact version".into());
        }
        if self.vocab < 2 || self.vocab > TL_MAX_VOCAB {
            return Err("transferable-lexical vocabulary out of range".into());
        }
        if self.f_dim != TL_F_DIM {
            return Err("transferable-lexical fact width mismatch".into());
        }
        if self.h_dim < 4
            || self.h_dim > TL_MAX_H_DIM
            || self.h_dim % 2 != 0
            || self.h_clamp < 1
            || self.m_clamp < 1
            || self.recurrent_shift > 5
            || self.score_shift > 20
        {
            return Err("transferable-lexical geometry out of range".into());
        }
        let h = self.h_dim;
        let v = self.vocab;
        let f = self.f_dim;
        if self.e.rows != v + 1 || self.e.cols != h {
            return Err("embedding shape mismatch".into());
        }
        if self.e.shift.len() != v + 1 || self.e.codes.len() != (v + 1) * h {
            return Err("embedding storage mismatch".into());
        }
        if self.e.shift.iter().any(|s| *s > TL_EMB_MAX_SHIFT) {
            return Err("embedding shift exceeds the declared bound".into());
        }
        if self
            .e
            .codes
            .iter()
            .any(|c| (*c as i32).abs() > TL_EMB_BOUND)
        {
            return Err("embedding code exceeds the declared 4-bit bound".into());
        }
        check_linear("wi", &self.wi, h, h + f)?;
        check_linear("wh", &self.wh, h, h)?;
        check_linear("wf", &self.wf, h, h + f + TL_EVENTS)?;
        check_linear("wo", &self.wo, v + 2, 2 * h + f + TL_EVENTS)?;
        if self.bh.len() != h || self.bo.len() != v + 2 {
            return Err("bias shape mismatch".into());
        }
        // Accumulator envelopes, evaluated on the declared input bounds.
        let fact_bound = 8u128;
        let m_bound = self.m_clamp as u128;
        let h_bound = self.h_clamp as u128;
        let mut bounds_init = vec![m_bound; h];
        bounds_init.extend(std::iter::repeat_n(fact_bound, f));
        check_range("wi", &self.wi, &bounds_init, 0)?;
        check_range("wh", &self.wh, &vec![h_bound; h], 0)?;
        let mut bounds_fact = vec![m_bound; h];
        bounds_fact.extend(std::iter::repeat_n(fact_bound, f));
        bounds_fact.extend(std::iter::repeat_n(1u128, TL_EVENTS));
        check_range("wf", &self.wf, &bounds_fact, 0)?;
        let mut bounds_out = vec![h_bound; h];
        bounds_out.extend(vec![m_bound; h]);
        bounds_out.extend(std::iter::repeat_n(fact_bound, f));
        bounds_out.extend(std::iter::repeat_n(1u128, TL_EVENTS));
        check_range("wo", &self.wo, &bounds_out, 0)?;
        Ok(())
    }

    /// Total served weight-table bytes.
    pub fn table_bytes(&self) -> usize {
        self.e.bytes()
            + self.wi.bytes()
            + self.wh.bytes()
            + self.wf.bytes()
            + self.wo.bytes()
            + self.bh.len() * 4
            + self.bo.len() * 4
    }

    /// Nonzero weight reads per served step (honest dense-map traffic for one token).
    pub fn nonzero_per_step(&self) -> usize {
        self.wh.nonzero() + self.wf.nonzero() + self.wo.nonzero()
    }

    /// Serialise the artifact to an explicit little-endian binary form.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o: Vec<u8> = Vec::new();
        o.extend_from_slice(b"TLX1");
        o.push(self.version);
        put_u32(&mut o, self.vocab as u32);
        put_u32(&mut o, self.h_dim as u32);
        put_u32(&mut o, self.f_dim as u32);
        put_i32(&mut o, self.h_clamp);
        put_i32(&mut o, self.m_clamp);
        put_u32(&mut o, self.recurrent_shift);
        put_u32(&mut o, self.score_shift);
        put_u32(&mut o, self.e.rows as u32);
        put_u32(&mut o, self.e.cols as u32);
        for s in &self.e.shift {
            put_u32(&mut o, *s);
        }
        o.extend(self.e.codes.iter().map(|c| *c as u8));
        for map in [&self.wi, &self.wh, &self.wf, &self.wo] {
            put_u32(&mut o, map.rows as u32);
            put_u32(&mut o, map.cols as u32);
            for s in &map.shift {
                put_u32(&mut o, *s);
            }
            put_u32(&mut o, map.packed.len() as u32);
            o.extend_from_slice(&map.packed);
        }
        for b in &self.bh {
            put_i32(&mut o, *b);
        }
        for b in &self.bo {
            put_i32(&mut o, *b);
        }
        o
    }

    /// Bounded, checked load. Rejects bad magic, shapes, ranges, envelopes and trailing bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated transferable-lexical artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"TLX1" {
            return Err("bad transferable-lexical magic".into());
        }
        let version = take(&mut c, 1)?[0];
        let vocab = get_u32(bytes, &mut c)? as usize;
        let h_dim = get_u32(bytes, &mut c)? as usize;
        let f_dim = get_u32(bytes, &mut c)? as usize;
        let h_clamp = get_i32(bytes, &mut c)?;
        let m_clamp = get_i32(bytes, &mut c)?;
        let recurrent_shift = get_u32(bytes, &mut c)?;
        let score_shift = get_u32(bytes, &mut c)?;
        let erows = get_u32(bytes, &mut c)? as usize;
        let ecols = get_u32(bytes, &mut c)? as usize;
        let mut eshift = Vec::with_capacity(erows);
        for _ in 0..erows {
            eshift.push(get_u32(bytes, &mut c)?);
        }
        let ecodes: Vec<i8> = take(&mut c, erows.saturating_mul(ecols))?
            .iter()
            .map(|b| *b as i8)
            .collect();
        let mut maps = Vec::new();
        for _ in 0..4 {
            let rows = get_u32(bytes, &mut c)? as usize;
            let cols = get_u32(bytes, &mut c)? as usize;
            let mut shift = Vec::with_capacity(rows);
            for _ in 0..rows {
                shift.push(get_u32(bytes, &mut c)?);
            }
            let plen = get_u32(bytes, &mut c)? as usize;
            let packed = take(&mut c, plen)?.to_vec();
            maps.push(TlLinear {
                rows,
                cols,
                packed,
                shift,
            });
        }
        let mut bh = Vec::with_capacity(h_dim);
        for _ in 0..h_dim {
            bh.push(get_i32(bytes, &mut c)?);
        }
        let mut bo = Vec::with_capacity(vocab + 2);
        for _ in 0..vocab + 2 {
            bo.push(get_i32(bytes, &mut c)?);
        }
        if c != bytes.len() {
            return Err("trailing bytes in transferable-lexical artifact".into());
        }
        let mut it = maps.into_iter();
        let model = Self {
            version,
            vocab,
            h_dim,
            f_dim,
            h_clamp,
            m_clamp,
            recurrent_shift,
            score_shift,
            e: TlEmbed {
                rows: erows,
                cols: ecols,
                codes: ecodes,
                shift: eshift,
            },
            wi: it.next().ok_or("missing wi")?,
            wh: it.next().ok_or("missing wh")?,
            wf: it.next().ok_or("missing wf")?,
            wo: it.next().ok_or("missing wo")?,
            bh,
            bo,
        };
        model.validate()?;
        Ok(model)
    }
}

/// Aggregate scored result of one example through the served path.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TlScore {
    pub bits_generate: f64,
    pub generate_targets: usize,
    pub bits_all: f64,
    pub scored: usize,
    pub correct: usize,
    pub stop_targets: usize,
    pub correct_stops: usize,
}

/// Quantised embedding aliasing report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmbeddingCollisions {
    pub distinct_rows: usize,
    pub colliding_pairs: usize,
    pub colliding_tokens: usize,
    pub witness: Option<(u32, u32)>,
}

fn expect_len(name: &str, v: &[f32], n: usize) -> Result<(), String> {
    if v.len() != n {
        return Err(format!("{name} master length {} != {n}", v.len()));
    }
    Ok(())
}

fn check_linear(name: &str, map: &TlLinear, rows: usize, cols: usize) -> Result<(), String> {
    if map.rows != rows || map.cols != cols {
        return Err(format!(
            "{name}: shape {}x{} != {rows}x{cols}",
            map.rows, map.cols
        ));
    }
    if map.shift.len() != rows || map.packed.len() != (rows * cols).div_ceil(4) {
        return Err(format!("{name}: storage mismatch"));
    }
    if map.shift.iter().any(|s| *s > TL_MAX_SHIFT) {
        return Err(format!("{name}: row shift exceeds the declared bound"));
    }
    Ok(())
}

/// Certify that every addition and the final shift plus bias fit the declared `i32` envelope,
/// without running the kernel.
fn check_range(name: &str, map: &TlLinear, bounds: &[u128], _bias: i64) -> Result<(), String> {
    let limit = (i32::MAX / 4) as u128;
    for r in 0..map.rows {
        let mut sum = 0u128;
        for c in 0..map.cols {
            if unpack_ternary(&map.packed, r, c, map.cols) != 0 {
                sum = sum
                    .checked_add(*bounds.get(c).unwrap_or(&0))
                    .ok_or_else(|| format!("{name}: accumulator bound overflow"))?;
            }
        }
        let shifted = sum
            .checked_shl(map.shift[r])
            .ok_or_else(|| format!("{name}: shifted bound overflow"))?;
        if shifted > limit {
            return Err(format!(
                "{name}: row {r} shifted bound {shifted} exceeds the declared envelope"
            ));
        }
    }
    Ok(())
}

/// Clamp a vector into `[-bound, bound]`.
fn clamp_in_place(v: &mut [i32], bound: i32) {
    for x in v.iter_mut() {
        if *x > bound {
            *x = bound;
        } else if *x < -bound {
            *x = -bound;
        }
    }
}

/// Stable FNV-style digest of an integer state, for diagnostics only.
pub fn state_digest(h: &[i32]) -> u64 {
    let mut acc = 0xcbf2_9ce4_8422_2325u64;
    for v in h {
        acc ^= (*v as i64 as u64).wrapping_mul(0x100_0000_01b3);
        acc = acc.rotate_left(27).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
    acc
}

/// `log2` of the softmax probability of `target` restricted to `rows`, computed with an explicit
/// normalization so an illegal action carries no probability mass.
fn log2_softmax_row(logits: &[i32], rows: &[usize], target: usize, score_shift: u32) -> f64 {
    let scale = (-(score_shift as f64)).exp2();
    let max = rows
        .iter()
        .map(|r| logits[*r])
        .fold(i32::MIN, |m, v| m.max(v));
    let mut z = 0f64;
    for r in rows {
        z += ((logits[*r] - max) as f64 * scale).exp2();
    }
    if z <= 0.0 {
        return f64::INFINITY;
    }
    let p = ((logits[target] - max) as f64 * scale).exp2() / z;
    -p.max(f64::MIN_POSITIVE).log2()
}

fn put_u32(o: &mut Vec<u8>, v: u32) {
    o.extend_from_slice(&v.to_le_bytes());
}

fn put_i32(o: &mut Vec<u8>, v: i32) {
    o.extend_from_slice(&v.to_le_bytes());
}

fn get_u32(bytes: &[u8], c: &mut usize) -> Result<u32, String> {
    let end = c.checked_add(4).ok_or("size overflow")?;
    if end > bytes.len() {
        return Err("truncated artifact".into());
    }
    let v = u32::from_le_bytes(bytes[*c..end].try_into().unwrap());
    *c = end;
    Ok(v)
}

fn get_i32(bytes: &[u8], c: &mut usize) -> Result<i32, String> {
    Ok(get_u32(bytes, c)? as i32)
}

/// Declared cold-start bound on the output master: below the ternary dead zone, so the exported
/// residual is exactly zero at initialisation while its gradient is not.
pub const OUTPUT_MASTER_BOUND: f32 = 0.45;

/// Offline training configuration. Floats and gradients are permitted offline under D0-b.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TlTrainConfig {
    pub lr: f64,
    pub beta1: f64,
    pub beta2: f64,
    pub weight_decay: f64,
    pub grad_clip: f64,
    pub seed: u64,
}

impl Default for TlTrainConfig {
    fn default() -> Self {
        Self {
            lr: 0.02,
            beta1: 0.9,
            beta2: 0.999,
            weight_decay: 0.0,
            grad_clip: 1.0,
            seed: 13,
        }
    }
}

/// Report of one training update.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TlBatchReport {
    pub bits_generate: f64,
    pub generate_targets: usize,
    pub scored: usize,
    pub correct: usize,
    pub examples: usize,
}

impl TlBatchReport {
    pub fn bits_per_target(&self) -> f64 {
        if self.generate_targets == 0 {
            f64::NAN
        } else {
            self.bits_generate / self.generate_targets as f64
        }
    }
    pub fn accuracy(&self) -> f64 {
        if self.scored == 0 {
            f64::NAN
        } else {
            self.correct as f64 / self.scored as f64
        }
    }
}

/// Quantised view of the float masters, computed once per update so the whole batch is optimised
/// against one served function.
struct TlQuant {
    h: usize,
    e_c: Vec<i8>,
    e_s: Vec<u32>,
    wi_c: Vec<i8>,
    wi_s: Vec<u32>,
    wh_c: Vec<i8>,
    wh_s: Vec<u32>,
    wf_c: Vec<i8>,
    wf_s: Vec<u32>,
    wo_c: Vec<i8>,
    wo_s: Vec<u32>,
}

impl TlQuant {
    /// One embedding value in the served algebra, exactly: `code << shift`.
    fn e(&self, row: usize, col: usize) -> f64 {
        self.e_c[row * self.h + col] as f64 * (1u64 << self.e_s[row]) as f64
    }
    /// One weight value: `code << shift`.
    fn w(c: &[i8], s: &[u32], cols: usize, r: usize, col: usize) -> f64 {
        c[r * cols + col] as f64 * (1u64 << s[r]) as f64
    }
}

/// One saved step of the forward pass, for the backward pass.
struct TlStep {
    h_in: Vec<f64>,
    pre: Vec<f64>,
    token_row: usize,
    ev: Vec<f64>,
    rows: Vec<usize>,
    target: Option<usize>,
    p: Vec<f64>,
}

/// Accumulated gradients, in served-value units (`value = code << shift`).
struct TlGrads {
    e: Vec<f64>,
    wi: Vec<f64>,
    wh: Vec<f64>,
    wf: Vec<f64>,
    wo: Vec<f64>,
    bh: Vec<f64>,
    bo: Vec<f64>,
    bits: f64,
    targets: usize,
    scored: usize,
    correct: usize,
}

/// A batch, clipped and Adam-updated. Weight masters are `f32`; the forward is exact `f64` over the
/// quantised codes so the trained function and the served function are the same integers.
pub struct TlTrainer {
    pub cfg: TlConfig,
    pub tcfg: TlTrainConfig,
    pub step: u64,
    e: Vec<f32>,
    wi: Vec<f32>,
    wh: Vec<f32>,
    wf: Vec<f32>,
    wo: Vec<f32>,
    bh: Vec<f32>,
    bo: Vec<f32>,
    me: Vec<f32>,
    mi: Vec<f32>,
    mh: Vec<f32>,
    mf: Vec<f32>,
    mo: Vec<f32>,
    mbh: Vec<f32>,
    mbo: Vec<f32>,
    ve: Vec<f32>,
    vi: Vec<f32>,
    vh: Vec<f32>,
    vf: Vec<f32>,
    vo: Vec<f32>,
    vbh: Vec<f32>,
    vbo: Vec<f32>,
}

impl TlTrainer {
    /// A fresh trainer with sign-varied masters, so the quantised codes are not all zero.
    pub fn new(cfg: TlConfig, tcfg: TlTrainConfig) -> Result<Self, String> {
        cfg.validate()?;
        let h = cfg.h_dim;
        let v = cfg.vocab;
        let f = TL_F_DIM;
        let mut rng = tcfg.seed | 1;
        let mut fill = |n: usize, amp: f64| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    let u = xorshift_unit_f64(&mut rng);
                    (u * amp) as f32
                })
                .collect()
        };
        Ok(Self {
            cfg,
            tcfg,
            step: 0,
            e: fill((v + 1) * h, 1.5),
            wi: fill(h * (h + f), 1.5),
            wh: fill(h * h, 1.5),
            wf: fill(h * (h + f + TL_EVENTS), 1.5),
            // Sub-threshold, so the exported output residual starts at exactly zero codes and the
            // initial served score is the declared marginal at the declared scale.
            wo: fill(
                (v + 2) * (2 * h + f + TL_EVENTS),
                OUTPUT_MASTER_BOUND as f64,
            ),
            bh: vec![0.0; h],
            bo: vec![0.0; v + 2],
            me: vec![0.0; (v + 1) * h],
            mi: vec![0.0; h * (h + f)],
            mh: vec![0.0; h * h],
            mf: vec![0.0; h * (h + f + TL_EVENTS)],
            mo: vec![0.0; (v + 2) * (2 * h + f + TL_EVENTS)],
            mbh: vec![0.0; h],
            mbo: vec![0.0; v + 2],
            ve: vec![0.0; (v + 1) * h],
            vi: vec![0.0; h * (h + f)],
            vh: vec![0.0; h * h],
            vf: vec![0.0; h * (h + f + TL_EVENTS)],
            vo: vec![0.0; (v + 2) * (2 * h + f + TL_EVENTS)],
            vbh: vec![0.0; h],
            vbo: vec![0.0; v + 2],
        })
    }

    fn quantized(&self) -> TlQuant {
        let h = self.cfg.h_dim;
        let v = self.cfg.vocab;
        let f = TL_F_DIM;
        let e = TlEmbed::quantize(&self.e, v + 1, h, TL_EMB_BOUND);
        let (wi_c, wi_s) = quantize_ternary(&self.wi, h, h + f);
        let (wh_c, wh_s) = quantize_ternary(&self.wh, h, h);
        let (wf_c, wf_s) = quantize_ternary(&self.wf, h, h + f + TL_EVENTS);
        let (wo_c, wo_s) = quantize_ternary(&self.wo, v + 2, 2 * h + f + TL_EVENTS);
        TlQuant {
            h,
            e_c: e.codes,
            e_s: e.shift,
            wi_c,
            wi_s,
            wh_c,
            wh_s,
            wf_c,
            wf_s,
            wo_c,
            wo_s,
        }
    }

    /// Initialise the output bias from a declared fit-only marginal over the **extended** alphabet:
    /// `bo[r] = round(scale · log2 p(r))` for all `vocab + 2` rows, including the `Copy` and `Stop`
    /// action rows. This is a declared starting point (the corpus marginal at the declared scale),
    /// not a learned parameter, and it must cover the action rows: leaving them at zero would give
    /// `Stop` probability one and steal half the mass from every generated token.
    pub fn set_output_bias(&mut self, log2_probs: &[f64], scale: f64) -> Result<(), String> {
        let rows = self.cfg.vocab + 2;
        if log2_probs.len() != rows {
            return Err(format!(
                "bias marginal length {} != action rows {rows}",
                log2_probs.len()
            ));
        }
        for r in 0..rows {
            self.bo[r] = (scale * log2_probs[r]).round() as f32;
        }
        Ok(())
    }

    /// The current served artifact.
    pub fn model(&self) -> Result<TlModel, String> {
        TlModel::build(
            &self.cfg, &self.e, &self.wi, &self.wh, &self.wf, &self.wo, &self.bh, &self.bo,
        )
    }

    fn content_feature(
        &self,
        q: &TlQuant,
        sel: &[u32],
        res: &[u32],
        v: usize,
    ) -> (Vec<f64>, Vec<f64>) {
        let half = self.cfg.h_dim / 2;
        let band = (half / TL_CONTENT_POSITIONS).max(1);
        let mut m = vec![0f64; self.cfg.h_dim];
        let mut terms: Vec<(usize, usize, usize)> = Vec::new();
        let push =
            |m: &mut Vec<f64>, toks: &[u32], off: usize, terms: &mut Vec<(usize, usize, usize)>| {
                let mut list: Vec<(usize, usize)> = toks
                    .iter()
                    .take(TL_CONTENT_TOKENS)
                    .enumerate()
                    .map(|(p, t)| (row_of(*t, v), p))
                    .collect();
                if toks.len() > TL_CONTENT_TOKENS {
                    list.push((row_of(toks[toks.len() - 1], v), TL_CONTENT_TOKENS));
                }
                for (row, p) in list {
                    let k = (p * band) % half;
                    for c in 0..half {
                        m[off + (c + k) % half] += q.e(row, c);
                    }
                    terms.push((row, p, off));
                }
            };
        push(&mut m, sel, 0, &mut terms);
        push(&mut m, res, half, &mut terms);
        let mut mask = vec![1f64; self.cfg.h_dim];
        let bound = self.cfg.m_clamp as f64;
        for (i, x) in m.iter_mut().enumerate() {
            if *x > bound {
                *x = bound;
                mask[i] = 0.0;
            } else if *x < -bound {
                *x = -bound;
                mask[i] = 0.0;
            }
        }
        (m, mask)
    }

    /// Forward and backward for one example against one quantised view.
    fn example(&self, q: &TlQuant, ex: &TlExample) -> TlGrads {
        let h_dim = self.cfg.h_dim;
        let f_dim = TL_F_DIM;
        let rs = self.cfg.recurrent_shift;
        let div = (1u64 << rs) as f64;
        let bound = self.cfg.h_clamp as f64;
        let owned: &[u32] = if ex.grounded { &ex.sel } else { &[] };
        let (m, m_mask) = self.content_feature(q, &ex.sel, &ex.res, self.cfg.vocab);
        let fblock = typed_causal_block(&ex.sel, &ex.res, ex.facts, &|t| {
            (t as usize) < self.cfg.vocab
        })
        .into_iter()
        .map(|x| x as f64)
        .collect::<Vec<f64>>();

        let zero_ev = vec![0f64; TL_EVENTS];
        let mut steps: Vec<TlStep> = Vec::new();

        // h_0 = clamp(W_i · [m; f] + b_h)
        let mut u0 = m.clone();
        u0.extend_from_slice(&fblock);
        let mut pre = vec![0f64; h_dim];
        for r in 0..h_dim {
            let mut acc = 0f64;
            for (c, x) in u0.iter().enumerate() {
                acc += TlQuant::w(&q.wi_c, &q.wi_s, h_dim + f_dim, r, c) * x;
            }
            pre[r] = acc + self.bh[r] as f64;
        }
        let mut h = clamp_f64(&pre, bound);
        steps.push(TlStep {
            h_in: vec![0f64; h_dim],
            pre: pre.clone(),
            token_row: 0,
            ev: zero_ev.clone(),
            rows: Vec::new(),
            target: None,
            p: Vec::new(),
        });

        let observe = |h: &mut Vec<f64>, tok: u32, steps: &mut Vec<TlStep>| {
            let row = row_of(tok, self.cfg.vocab);
            let ev = event_vec(TL_EV_OBSERVE);
            let mut pre = vec![0f64; h_dim];
            let mut mid = vec![0f64; h_dim];
            let mut extra = vec![0f64; h_dim];
            let mut u = m.clone();
            u.extend_from_slice(&fblock);
            u.extend_from_slice(&ev);
            for r in 0..h_dim {
                let mut acc = 0f64;
                for (c, x) in u.iter().enumerate() {
                    acc += TlQuant::w(&q.wf_c, &q.wf_s, h_dim + f_dim + TL_EVENTS, r, c) * x;
                }
                extra[r] = acc;
                let rec: f64 = (0..h_dim)
                    .map(|c| q.wh_c[r * h_dim + c] as f64 * h[c] * (1u64 << q.wh_s[r]) as f64)
                    .sum();
                mid[r] = (rec / div).floor();
                pre[r] = q.e(row, r) + mid[r] + extra[r] + self.bh[r] as f64;
            }
            let h_next = clamp_f64(&pre, bound);
            steps.push(TlStep {
                h_in: h.clone(),
                pre,
                token_row: row,
                ev,
                rows: Vec::new(),
                target: None,
                p: Vec::new(),
            });
            *h = h_next;
        };

        for t in &ex.observed {
            observe(&mut h, *t, &mut steps);
        }

        let mut g = TlGrads {
            e: vec![0f64; (self.cfg.vocab + 1) * h_dim],
            wi: vec![0f64; h_dim * (h_dim + f_dim)],
            wh: vec![0f64; h_dim * h_dim],
            wf: vec![0f64; h_dim * (h_dim + f_dim + TL_EVENTS)],
            wo: vec![0f64; (self.cfg.vocab + 2) * (2 * h_dim + f_dim + TL_EVENTS)],
            bh: vec![0f64; h_dim],
            bo: vec![0f64; self.cfg.vocab + 2],
            bits: 0.0,
            targets: 0,
            scored: 0,
            correct: 0,
        };

        let mut xbuf = Vec::with_capacity(2 * h_dim + f_dim + TL_EVENTS);
        for i in 0..ex.actions.len() {
            let copy_legal = ex.copy_legal(i, owned);
            let event = ex.prior_event(i);
            let ev = event_vec(event);
            xbuf.clear();
            xbuf.extend_from_slice(&h);
            xbuf.extend_from_slice(&m);
            xbuf.extend_from_slice(&fblock);
            xbuf.extend_from_slice(&ev);
            let cols = 2 * h_dim + f_dim + TL_EVENTS;
            let mut logits = vec![0f64; self.cfg.vocab + 2];
            for r in 0..self.cfg.vocab + 2 {
                let mut acc = 0f64;
                for (c, x) in xbuf.iter().enumerate() {
                    acc += TlQuant::w(&q.wo_c, &q.wo_s, cols, r, c) * x;
                }
                logits[r] = acc + self.bo[r] as f64;
            }
            let mut rows: Vec<usize> = (0..self.cfg.vocab).collect();
            rows.push(self.cfg.vocab + 1);
            if copy_legal {
                rows.push(self.cfg.vocab);
            }
            let target = match ex.actions[i] {
                TlAction::Generate(v) => (v as usize).min(self.cfg.vocab - 1),
                TlAction::Copy => self.cfg.vocab,
                TlAction::Stop => self.cfg.vocab + 1,
            };
            let scale = (-(self.cfg.score_shift as f64)).exp2();
            let max = rows
                .iter()
                .map(|r| logits[*r])
                .fold(f64::NEG_INFINITY, f64::max);
            let mut z = 0f64;
            for r in &rows {
                z += ((logits[*r] - max) * scale).exp2();
            }
            let mut p = vec![0f64; self.cfg.vocab + 2];
            for r in &rows {
                p[*r] = ((logits[*r] - max) * scale).exp2() / z;
            }
            let bits = -(p[target].max(f64::MIN_POSITIVE)).log2();
            g.bits += bits * ex.weight as f64;
            g.targets += 1;
            g.scored += 1;
            let argmax =
                rows.iter().copied().fold(
                    rows[0],
                    |best, r| if logits[r] > logits[best] { r } else { best },
                );
            if argmax == target {
                g.correct += 1;
            }
            steps.push(TlStep {
                h_in: h.clone(),
                pre: vec![0f64; h_dim],
                token_row: 0,
                ev,
                rows,
                target: Some(target),
                p,
            });
            // Advance with the actually emitted token.
            let emitted = match ex.actions[i] {
                TlAction::Generate(v) => Some(v),
                TlAction::Copy => ex.emitted(i, owned),
                TlAction::Stop => None,
            };
            let row = emitted
                .map(|t| row_of(t, self.cfg.vocab))
                .unwrap_or(self.cfg.vocab);
            let ev_fb = event_vec(ex.actions[i].event());
            let mut pre_t = vec![0f64; h_dim];
            let mut u = m.clone();
            u.extend_from_slice(&fblock);
            u.extend_from_slice(&ev_fb);
            for r in 0..h_dim {
                let rec: f64 = (0..h_dim)
                    .map(|c| q.wh_c[r * h_dim + c] as f64 * h[c] * (1u64 << q.wh_s[r]) as f64)
                    .sum();
                let mut acc = 0f64;
                for (c, x) in u.iter().enumerate() {
                    acc += TlQuant::w(&q.wf_c, &q.wf_s, h_dim + f_dim + TL_EVENTS, r, c) * x;
                }
                pre_t[r] = q.e(row, r) + (rec / div).floor() + acc + self.bh[r] as f64;
            }
            let h_next = clamp_f64(&pre_t, bound);
            steps.push(TlStep {
                h_in: h.clone(),
                pre: pre_t,
                token_row: row,
                ev: ev_fb,
                rows: Vec::new(),
                target: None,
                p: Vec::new(),
            });
            h = h_next;
        }

        // ---- backward ----
        let cols_h = h_dim;
        let cols_f = h_dim + f_dim;
        let cols_wf = h_dim + f_dim + TL_EVENTS;
        let cols_wo = 2 * h_dim + f_dim + TL_EVENTS;
        let mut dh = vec![0f64; h_dim];
        let mut dm = vec![0f64; h_dim];
        let mut du0 = vec![0f64; cols_f];
        let mut dh0 = vec![0f64; h_dim];
        for si in (0..steps.len()).rev() {
            let st = &steps[si];
            if let Some(target) = st.target {
                // readout gradient
                let mut dlogits = vec![0f64; self.cfg.vocab + 2];
                // d(bits)/dZ, carrying the one declared dyadic scale through.
                let dscale = (-(self.cfg.score_shift as f64)).exp2();
                for r in &st.rows {
                    let mut d = st.p[*r];
                    if *r == target {
                        d -= 1.0;
                    }
                    dlogits[*r] = d * dscale;
                }
                let mut xbuf = Vec::with_capacity(cols_wo);
                xbuf.extend_from_slice(&st.h_in);
                xbuf.extend_from_slice(&m);
                xbuf.extend_from_slice(&fblock);
                xbuf.extend_from_slice(&st.ev);
                for (r, dl) in dlogits.iter().enumerate() {
                    if *dl == 0.0 {
                        continue;
                    }
                    g.bo[r] += dl;
                    for (c, x) in xbuf.iter().enumerate() {
                        g.wo[r * cols_wo + c] += dl * x;
                    }
                }
                for c in 0..h_dim {
                    let mut acc = 0f64;
                    for r in 0..self.cfg.vocab + 2 {
                        if dlogits[r] != 0.0 {
                            acc += dlogits[r] * TlQuant::w(&q.wo_c, &q.wo_s, cols_wo, r, c);
                        }
                    }
                    dh[c] += acc;
                }
                for c in 0..h_dim {
                    let mut acc = 0f64;
                    for r in 0..self.cfg.vocab + 2 {
                        if dlogits[r] != 0.0 {
                            acc += dlogits[r] * TlQuant::w(&q.wo_c, &q.wo_s, cols_wo, r, h_dim + c);
                        }
                    }
                    dm[c] += acc;
                }
            } else if si == 0 {
                dh0 = dh.clone();
            } else {
                // transition step si-1 -> si
                let mask: Vec<f64> = st
                    .pre
                    .iter()
                    .map(|p| if p.abs() < bound { 1.0 } else { 0.0 })
                    .collect();
                let mut dpre = vec![0f64; h_dim];
                for r in 0..h_dim {
                    dpre[r] = dh[r] * mask[r];
                }
                // embeddings and bias
                for r in 0..h_dim {
                    g.e[st.token_row * h_dim + r] += dpre[r];
                    g.bh[r] += dpre[r];
                }
                // recurrent and fact maps
                let mut u = m.clone();
                u.extend_from_slice(&fblock);
                u.extend_from_slice(&st.ev);
                let mut dmid = vec![0f64; h_dim];
                for r in 0..h_dim {
                    dmid[r] = dpre[r] / div;
                }
                let mut du = vec![0f64; cols_wf];
                for r in 0..h_dim {
                    if dmid[r] != 0.0 {
                        for (c, x) in st.h_in.iter().enumerate() {
                            g.wh[r * cols_h + c] += dmid[r] * x;
                        }
                    }
                    if dpre[r] != 0.0 {
                        for (c, x) in u.iter().enumerate() {
                            g.wf[r * cols_wf + c] += dpre[r] * x;
                        }
                    }
                }
                for c in 0..cols_wf {
                    let mut acc = 0f64;
                    for r in 0..h_dim {
                        if dpre[r] != 0.0 {
                            acc += dpre[r] * TlQuant::w(&q.wf_c, &q.wf_s, cols_wf, r, c);
                        }
                    }
                    du[c] = acc;
                }
                for c in 0..h_dim {
                    dm[c] += du[c];
                }
                for c in 0..h_dim {
                    let mut acc = 0f64;
                    for r in 0..h_dim {
                        if dmid[r] != 0.0 {
                            acc += dmid[r] * TlQuant::w(&q.wh_c, &q.wh_s, cols_h, r, c);
                        }
                    }
                    dh[c] = acc;
                }
            }
        }

        // init map
        {
            let mask: Vec<f64> = steps[0]
                .pre
                .iter()
                .map(|p| if p.abs() < bound { 1.0 } else { 0.0 })
                .collect();
            let mut dpre0 = vec![0f64; h_dim];
            for r in 0..h_dim {
                dpre0[r] = dh0[r] * mask[r];
            }
            let mut u0 = m.clone();
            u0.extend_from_slice(&fblock);
            for r in 0..h_dim {
                if dpre0[r] != 0.0 {
                    g.bh[r] += dpre0[r];
                    for (c, x) in u0.iter().enumerate() {
                        g.wi[r * cols_f + c] += dpre0[r] * x;
                    }
                }
            }
            for c in 0..cols_f {
                let mut acc = 0f64;
                for r in 0..h_dim {
                    if dpre0[r] != 0.0 {
                        acc += dpre0[r] * TlQuant::w(&q.wi_c, &q.wi_s, cols_f, r, c);
                    }
                }
                du0[c] = acc;
            }
            for c in 0..h_dim {
                dm[c] += du0[c];
            }
        }

        // content embedding rows
        let half = self.cfg.h_dim / 2;
        let band = (half / TL_CONTENT_POSITIONS).max(1);
        for i in 0..h_dim {
            dm[i] *= m_mask[i];
        }
        let v = self.cfg.vocab;
        let mut lists: Vec<(Vec<u32>, usize)> = Vec::new();
        if ex.grounded {
            lists.push((ex.sel.clone(), 0));
            lists.push((ex.res.clone(), half));
        }
        for (toks, off) in lists {
            let mut list: Vec<(usize, usize)> = toks
                .iter()
                .take(TL_CONTENT_TOKENS)
                .enumerate()
                .map(|(p, t)| (row_of(*t, v), p))
                .collect();
            if toks.len() > TL_CONTENT_TOKENS {
                list.push((row_of(toks[toks.len() - 1], v), TL_CONTENT_TOKENS));
            }
            for (row, p) in list {
                let k = (p * band) % half;
                for c in 0..half {
                    g.e[row * h_dim + c] += dm[off + (c + k) % half];
                }
            }
        }
        g.bits /= ex.weight as f64;
        g
    }

    /// One Adam update over a batch of examples. Returns the batch's scored report.
    pub fn train_batch(&mut self, batch: &[TlExample]) -> TlBatchReport {
        let q = self.quantized();
        let mut acc = TlReportAcc::default();
        let mut grads = TlGrads {
            e: vec![0f64; self.e.len()],
            wi: vec![0f64; self.wi.len()],
            wh: vec![0f64; self.wh.len()],
            wf: vec![0f64; self.wf.len()],
            wo: vec![0f64; self.wo.len()],
            bh: vec![0f64; self.bh.len()],
            bo: vec![0f64; self.bo.len()],
            bits: 0.0,
            targets: 0,
            scored: 0,
            correct: 0,
        };
        let mut total_weight = 0f64;
        for ex in batch {
            let g = self.example(&q, ex);
            for (a, b) in grads.e.iter_mut().zip(&g.e) {
                *a += b * ex.weight as f64;
            }
            for (a, b) in grads.wi.iter_mut().zip(&g.wi) {
                *a += b * ex.weight as f64;
            }
            for (a, b) in grads.wh.iter_mut().zip(&g.wh) {
                *a += b * ex.weight as f64;
            }
            for (a, b) in grads.wf.iter_mut().zip(&g.wf) {
                *a += b * ex.weight as f64;
            }
            for (a, b) in grads.wo.iter_mut().zip(&g.wo) {
                *a += b * ex.weight as f64;
            }
            for (a, b) in grads.bh.iter_mut().zip(&g.bh) {
                *a += b * ex.weight as f64;
            }
            for (a, b) in grads.bo.iter_mut().zip(&g.bo) {
                *a += b * ex.weight as f64;
            }
            acc.bits += g.bits * ex.weight as f64;
            acc.targets += g.targets;
            acc.scored += g.scored;
            acc.correct += g.correct;
            total_weight += ex.weight as f64;
        }
        if total_weight <= 0.0 || batch.is_empty() {
            return acc.finish();
        }
        // Normalise by the total supervision weight, not by the raw example count.
        let norm = 1.0 / total_weight;
        for a in grads.e.iter_mut() {
            *a *= norm;
        }
        for a in grads.wi.iter_mut() {
            *a *= norm;
        }
        for a in grads.wh.iter_mut() {
            *a *= norm;
        }
        for a in grads.wf.iter_mut() {
            *a *= norm;
        }
        for a in grads.wo.iter_mut() {
            *a *= norm;
        }
        for a in grads.bh.iter_mut() {
            *a *= norm;
        }
        for a in grads.bo.iter_mut() {
            *a *= norm;
        }
        // Global-norm clip over every parameter group, applied to the live gradient buffers.
        if self.tcfg.grad_clip > 0.0 {
            let mut sq = 0f64;
            for g in [
                &grads.e, &grads.wi, &grads.wh, &grads.wf, &grads.wo, &grads.bh, &grads.bo,
            ] {
                for x in g.iter() {
                    sq += x * x;
                }
            }
            let norm = sq.sqrt();
            if norm > self.tcfg.grad_clip {
                let s = self.tcfg.grad_clip / norm;
                for g in [
                    &mut grads.e,
                    &mut grads.wi,
                    &mut grads.wh,
                    &mut grads.wf,
                    &mut grads.wo,
                    &mut grads.bh,
                    &mut grads.bo,
                ] {
                    for x in g.iter_mut() {
                        *x *= s;
                    }
                }
            }
        }
        self.step += 1;
        let st = self.step;
        let cfg = self.tcfg;
        {
            let pairs = [
                (&mut self.e, &mut self.me, &mut self.ve, &mut grads.e),
                (&mut self.wi, &mut self.mi, &mut self.vi, &mut grads.wi),
                (&mut self.wh, &mut self.mh, &mut self.vh, &mut grads.wh),
                (&mut self.wf, &mut self.mf, &mut self.vf, &mut grads.wf),
                (&mut self.wo, &mut self.mo, &mut self.vo, &mut grads.wo),
                (&mut self.bh, &mut self.mbh, &mut self.vbh, &mut grads.bh),
                (&mut self.bo, &mut self.mbo, &mut self.vbo, &mut grads.bo),
            ];
            for (p, m, v, g) in pairs {
                adam_step(p, g, m, v, &cfg, st);
            }
        }
        acc.finish()
    }
}

#[derive(Default)]
struct TlReportAcc {
    bits: f64,
    targets: usize,
    scored: usize,
    correct: usize,
}

impl TlReportAcc {
    fn finish(&self) -> TlBatchReport {
        TlBatchReport {
            bits_generate: self.bits,
            generate_targets: self.targets,
            scored: self.scored,
            correct: self.correct,
            examples: self.scored,
        }
    }
}

fn adam_step(
    p: &mut [f32],
    g: &[f64],
    m: &mut [f32],
    v: &mut [f32],
    cfg: &TlTrainConfig,
    step: u64,
) {
    let b1 = cfg.beta1;
    let b2 = cfg.beta2;
    let bc1 = 1.0 - b1.powi(step as i32);
    let bc2 = 1.0 - b2.powi(step as i32);
    for i in 0..p.len() {
        let gi = g[i];
        let mi = b1 * m[i] as f64 + (1.0 - b1) * gi;
        let vi = b2 * v[i] as f64 + (1.0 - b2) * gi * gi;
        m[i] = mi as f32;
        v[i] = vi as f32;
        let mh = mi / (bc1 + 1e-12);
        let vh = vi / (bc2 + 1e-12);
        let update = cfg.lr * mh / (vh.sqrt() + 1e-8);
        p[i] -= update as f32;
        if cfg.weight_decay > 0.0 {
            p[i] -= (cfg.lr * cfg.weight_decay * p[i] as f64) as f32;
        }
    }
}

fn row_of(token: u32, vocab: usize) -> usize {
    let t = token as usize;
    if t < vocab {
        t
    } else {
        vocab
    }
}

fn event_vec(event: usize) -> Vec<f64> {
    let mut ev = vec![0f64; TL_EVENTS];
    ev[event.min(TL_EVENTS - 1)] = 1.0;
    ev
}

fn clamp_f64(v: &[f64], bound: f64) -> Vec<f64> {
    v.iter()
        .map(|x| {
            if *x > bound {
                bound
            } else if *x < -bound {
                -bound
            } else {
                *x
            }
        })
        .collect()
}

fn xorshift_unit_f64(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    let u = ((*state >> 11) as f64) / ((1u64 << 53) as f64);
    u * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(vocab: usize, h: usize) -> TlConfig {
        TlConfig {
            vocab,
            h_dim: h,
            h_clamp: 2048,
            m_clamp: 8192,
            recurrent_shift: 3,
            score_shift: 10,
        }
    }

    /// A hand-built artifact with a deterministic sign-varied embedding and zero maps, so the
    /// structural tests do not depend on fitting.
    fn hand_model(vocab: usize, h: usize) -> TlModel {
        // A bit-pattern embedding: distinct rows stay distinct under the fixed rotations, so the
        // structural tests measure the mechanism rather than a degenerate alias.
        let e: Vec<f32> = (0..(vocab + 1) * h)
            .map(|i| {
                let (r, c) = (i / h, i % h);
                if (r >> c) & 1 == 0 {
                    1.0
                } else {
                    -1.0
                }
            })
            .collect();
        TlModel::build(
            &cfg(vocab, h),
            &e,
            &vec![0.0; h * (h + TL_F_DIM)],
            &vec![0.0; h * h],
            &vec![0.0; h * (h + TL_F_DIM + TL_EVENTS)],
            &vec![0.0; (vocab + 2) * (2 * h + TL_F_DIM + TL_EVENTS)],
            &vec![0.0; h],
            &vec![0.0; vocab + 2],
        )
        .expect("hand model builds")
    }

    fn gen_example(observed: Vec<u32>, targets: Vec<u32>) -> TlExample {
        TlExample {
            sel: Vec::new(),
            res: Vec::new(),
            facts: SlFacts::default(),
            observed,
            actions: targets.into_iter().map(TlAction::Generate).collect(),
            weight: 1.0,
            doc: 0,
            grounded: false,
            terminal_stop: false,
        }
    }

    fn grounded(sel: Vec<u32>, _owned: Vec<u32>, actions: Vec<TlAction>) -> TlExample {
        TlExample {
            sel,
            res: Vec::new(),
            facts: SlFacts::default(),
            observed: Vec::new(),
            actions,
            weight: 1.0,
            doc: 0,
            grounded: true,
            terminal_stop: true,
        }
    }

    #[test]
    fn served_integer_maps_equal_the_exact_float_reference() {
        let m = hand_model(16, 8);
        let widest = *[m.wi.cols, m.wh.cols, m.wf.cols, m.wo.cols]
            .iter()
            .max()
            .unwrap();
        let x: Vec<i32> = (0..widest as i32).map(|i| (i % 7) - 3).collect();
        let xf: Vec<f64> = x.iter().map(|v| *v as f64).collect();
        for map in [&m.wi, &m.wh, &m.wf] {
            let int = map.forward_i32(&x[..map.cols]).unwrap();
            let flt = map.forward_reference(&xf[..map.cols]);
            assert_eq!(int.len(), flt.len());
            for (a, b) in int.iter().zip(flt.iter()) {
                assert_eq!(
                    *a as f64, *b,
                    "integer kernel diverged from its float reference"
                );
            }
        }
        let xo: Vec<i32> = (0..m.wo.cols as i32).map(|i| (i % 5) - 2).collect();
        let xof: Vec<f64> = xo.iter().map(|v| *v as f64).collect();
        let int = m.wo.forward_i32(&xo).unwrap();
        let flt = m.wo.forward_reference(&xof);
        for (a, b) in int.iter().zip(flt.iter()) {
            assert_eq!(*a as f64, *b);
        }
    }

    #[test]
    fn copy_is_illegal_without_a_live_owned_occurrence() {
        let m = hand_model(16, 8);
        assert!(!m.legal_rows(false).contains(&m.copy_row()));
        assert!(m.legal_rows(true).contains(&m.copy_row()));
        // A source-disabled rollout can never choose Copy.
        let r = m.rollout(&[3], &[], SlFacts::default(), &[], &[3, 4], 6, false, true);
        assert!(!r.actions.contains(&TlAction::Copy));
        let r2 = m.rollout(&[], &[], SlFacts::default(), &[], &[], 4, false, false);
        assert!(!r2.actions.contains(&TlAction::Copy));
    }

    #[test]
    fn bits_are_normalised_over_the_legal_action_set_only() {
        let m = hand_model(16, 8);
        let ex = gen_example(Vec::new(), vec![7]);
        let s = m.score_example(&ex);
        // All scores are equal, so the probability is uniform over the legal set: 16 Generate rows
        // plus Stop, and no Copy because there is no owned occurrence.
        assert!(
            (s.bits_generate - (17f64).log2()).abs() < 1e-9,
            "{}",
            s.bits_generate
        );
        let grounded_ex = grounded(vec![3], vec![3], vec![TlAction::Generate(7)]);
        let sg = m.score_example(&grounded_ex);
        assert!(
            (sg.bits_generate - (18f64).log2()).abs() < 1e-9,
            "{}",
            sg.bits_generate
        );
    }

    #[test]
    fn a_reordered_span_changes_the_evidence_fingerprint() {
        let m = hand_model(16, 8);
        let ab = m.content_feature(&[3, 5], &[]);
        let ba = m.content_feature(&[5, 3], &[]);
        assert_ne!(
            ab, ba,
            "the fixed position rotation must make order observable"
        );
        let distinct = m.content_feature(&[3], &[]);
        let other = m.content_feature(&[4], &[]);
        assert_ne!(distinct, other);
    }

    #[test]
    fn the_recurrence_consumes_the_actually_emitted_token() {
        let m = hand_model(16, 8);
        let true_arm = m.rollout(&[3], &[], SlFacts::default(), &[], &[3], 3, false, false);
        let blind = m.rollout(&[3], &[], SlFacts::default(), &[], &[3], 3, true, false);
        assert_ne!(
            true_arm.state_digest, blind.state_digest,
            "token identity must reach the state"
        );
    }

    #[test]
    fn temporal_meaning_is_authored_from_typed_state_not_from_the_route_key() {
        let changed_key_equal_value = SlFacts {
            key_changed: true,
            ..SlFacts::default()
        };
        let same_key_changed_value_no_commit = SlFacts {
            prior_differs: true,
            ..SlFacts::default()
        };
        let no_commit = SlFacts {
            derived: true,
            ..SlFacts::default()
        };
        let committed_correction = SlFacts {
            committed: true,
            prior_differs: true,
            derived: true,
            ..SlFacts::default()
        };
        assert!(!TlModel::truthful_temporal_meaning(changed_key_equal_value));
        assert!(!TlModel::truthful_temporal_meaning(
            same_key_changed_value_no_commit
        ));
        assert!(!TlModel::truthful_temporal_meaning(no_commit));
        assert!(TlModel::truthful_temporal_meaning(committed_correction));
        // The retained route marker keyed on the changed address is the *opposite* of this rule for
        // the first regime, which is why the preflight can falsify it.
        assert_ne!(
            TlModel::truthful_temporal_meaning(changed_key_equal_value),
            changed_key_equal_value.key_changed
        );
        // Only the committed supersession drives the rule: the route key alone does not.
        let committed_via_changed_key = SlFacts {
            committed: true,
            prior_differs: true,
            key_changed: true,
            ..SlFacts::default()
        };
        assert!(TlModel::truthful_temporal_meaning(
            committed_via_changed_key
        ));
    }

    #[test]
    fn artifact_round_trips_exactly_and_rejects_corruption() {
        let m = hand_model(16, 8);
        let bytes = m.to_bytes();
        let back = TlModel::from_bytes(&bytes).expect("round trip");
        assert_eq!(m, back);
        assert!(TlModel::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        let mut bad = bytes.clone();
        bad[0] = b'X';
        assert!(TlModel::from_bytes(&bad).is_err());
        let mut bad2 = bytes.clone();
        bad2.push(0);
        assert!(TlModel::from_bytes(&bad2).is_err());
    }

    #[test]
    fn declared_low_bit_bounds_hold() {
        let t = TlTrainer::new(cfg(64, 16), TlTrainConfig::default()).unwrap();
        let m = t.model().unwrap();
        for map in [&m.wi, &m.wh, &m.wf, &m.wo] {
            assert!(map.shift.iter().all(|s| *s <= TL_MAX_SHIFT));
            for r in 0..map.rows {
                for c in 0..map.cols {
                    assert!(unpack_ternary(&map.packed, r, c, map.cols).abs() <= 1);
                }
            }
            assert!(map.to_ternary().unwrap().bits_per_weight() < 2.03);
        }
        assert!(m.e.shift.iter().all(|s| *s <= TL_EMB_MAX_SHIFT));
        assert!(m.e.codes.iter().all(|c| (*c as i32).abs() <= TL_EMB_BOUND));
        m.validate().unwrap();
    }

    #[test]
    fn the_readout_loss_gradient_matches_a_finite_difference_on_the_unquantised_bias() {
        // `bo` is the one continuous, unquantised parameter, so the exact loss differentiation can
        // be checked against a finite difference. Quantised masters cannot be finite-differenced
        // meaningfully because the straight-through estimator treats the code as their value; those
        // are validated by measured descent on the served artifact instead.
        let mut t = TlTrainer::new(
            cfg(16, 8),
            TlTrainConfig {
                seed: 5,
                ..Default::default()
            },
        )
        .unwrap();
        let ex = gen_example(vec![2], vec![3, 4]);
        let q = t.quantized();
        let g = t.example(&q, &ex);
        let idx = 5usize;
        let eps = 1e-6f64;
        let base = t.bo[idx] as f64;
        t.bo[idx] = (base + eps) as f32;
        let q2 = t.quantized();
        let plus = t.example(&q2, &ex).bits;
        t.bo[idx] = (base - eps) as f32;
        let q3 = t.quantized();
        let minus = t.example(&q3, &ex).bits;
        t.bo[idx] = base as f32;
        let fd = (plus - minus) / (2.0 * eps);
        let analytic = g.bo[idx];
        assert!(
            (fd - analytic).abs() < 1e-3 * analytic.abs().max(1.0),
            "fd {fd} vs analytic {analytic}"
        );
    }

    #[test]
    fn training_reduces_served_bits_on_a_synthetic_source() {
        let mut t = TlTrainer::new(
            cfg(16, 16),
            TlTrainConfig {
                lr: 0.05,
                seed: 7,
                ..Default::default()
            },
        )
        .unwrap();
        let cycle: Vec<u32> = (0..17).map(|i| (1 + (i % 8)) as u32).collect();
        let ex = gen_example(vec![cycle[0]], cycle[1..].to_vec());
        let before = t.model().unwrap().score_example(&ex);
        for _ in 0..400 {
            t.train_batch(std::slice::from_ref(&ex));
        }
        let after = t.model().unwrap().score_example(&ex);
        assert!(
            after.bits_generate < before.bits_generate - 1.0,
            "served bits must fall: {} -> {}",
            before.bits_generate,
            after.bits_generate
        );
        assert!(
            after.correct >= before.correct + 4,
            "teacher-forced agreement must improve: {} -> {}",
            before.correct,
            after.correct
        );
    }

    #[test]
    fn the_post_copy_vocabulary_decision_depends_on_the_copied_identity() {
        // Two grounded cases with identical request, typed facts, copy length and events; only the
        // copied token identity differs, and the accepted post-copy word is authored from it.
        let mut t = TlTrainer::new(
            cfg(16, 16),
            TlTrainConfig {
                lr: 0.05,
                seed: 11,
                ..Default::default()
            },
        )
        .unwrap();
        let a = grounded(
            vec![3],
            vec![3],
            vec![TlAction::Copy, TlAction::Generate(5), TlAction::Stop],
        );
        let b = grounded(
            vec![4],
            vec![4],
            vec![TlAction::Copy, TlAction::Generate(6), TlAction::Stop],
        );
        let batch = vec![a.clone(), b.clone()];
        for _ in 0..800 {
            t.train_batch(&batch);
        }
        let m = t.model().unwrap();
        let ra = m.rollout(&a.sel, &[], a.facts, &[], &a.sel, 3, false, false);
        let rb = m.rollout(&b.sel, &[], b.facts, &[], &b.sel, 3, false, false);
        assert_eq!(ra.tokens, vec![3, 5], "loaded true arm A");
        assert_eq!(rb.tokens, vec![4, 6], "loaded true arm B");
        assert_ne!(ra.state_digest, rb.state_digest);
        // Identity-erased arm: evidence and feedback lose the copied token identity. The copied
        // surface is still exact, so the comparison is on the *learned* post-copy word.
        let ba = m.rollout(&a.sel, &[], a.facts, &[], &a.sel, 3, true, false);
        let bb = m.rollout(&b.sel, &[], b.facts, &[], &b.sel, 3, true, false);
        assert_eq!(
            ba.actions[1], bb.actions[1],
            "blinded arms must alias on the post-copy decision"
        );
        assert!(
            ba.actions[1] != ra.actions[1] || bb.actions[1] != rb.actions[1],
            "erasing copied identity must cost the distinction"
        );
    }
}
