//! A learned, state-conditioned lexical decoder for one owned answer.
//!
//! # Why this replaces a finite context key
//!
//! The retained `lexical_realization` prototype indexed a single argmax table by causal *flags*
//! (relation, requested history, derived/prior-difference bits, copy stage and a clipped count of
//! emitted vocabulary words). Its own doc comment states the obstruction: the actual generated token
//! identities and the evidence content do not enter the key. While the copy stage and the context are
//! fixed, inserting another word leaves the key unchanged, so a deterministic argmax repeats the same
//! choice and an external cap has to intervene. Two different values with identical flags cannot
//! change an uncopied word, and two equal-length prefixes that differ only in the symbols they
//! actually emitted cannot condition different continuations.
//!
//! This module supplies the missing dependency directly, following the responsibility split in the
//! principal brief:
//!
//! ```text
//! c       = causal pinned request + selected evidence/derived result + provenance
//! h_0     = learned_init(c)
//! a_t     = learned_readout(h_t, c, exact_copy_state)
//! x_t     = execute(a_t)          // a generated token, an exact owned token, or Stop
//! h_(t+1) = learned_transition(h_t, action_symbol(a_t), emitted_token_row, typed_facts)
//! ```
//!
//! * `c` is a *content* vector: a position-resolved embedding of the selected owned payload tokens
//!   and, for a consumed computation, of the operand the computation started from, plus a **typed
//!   causal block**. Position-dependent rows make the feature order-sensitive, a dedicated tail
//!   position carries the final token of a truncated payload, and an unfitted token occupies an
//!   explicit reserved row rather than contributing nothing. The typed block exposes exact,
//!   causally-available distinctions the compressed embedding cannot rediscover: requested history,
//!   whether the answer is derived, whether an older committed value was superseded, whether a store
//!   mutation was actually committed, whether the derived query key differs from the operand key, and
//!   the exact equality / prefix / divergence of the selected and operand identities. These are typed
//!   observations, not a supplied answer class: the learner still decides whether and how to verbalise
//!   them.
//! * `h_t` is a bounded integer state vector. `learned_transition` consumes the executed action event
//!   *and the actual emitted token identity*, so an inserted word and a copied word both advance the
//!   recurrence by the learned row of the token the session really emitted. Two equal-length copied
//!   spans over different fitted tokens therefore reach different states. An unfitted copied token
//!   still shares the reserved row; that residual collision is declared, not hidden.
//!
//! Exact copy identity, cursor and payload bytes stay separately owned by the scoped session; this
//! decoder never replaces identity with lossy state. A copied token carries both its exact byte (owned
//! elsewhere) and its learned identity into the recurrence.
//!
//! # Serving contract (D0-b)
//!
//! All weight maps are ternary (`TernaryLinear`: two bits per weight, power-of-two per-row scale) and
//! the learned embeddings are small integers stored as `i8` table reads. Serving therefore executes
//! table reads, integer additions/subtractions, shifts, comparisons and an argmax: no multiplier
//! instruction and no floating point. The bounded clamp is a compare/select, not arithmetic.
//!
//! # Learning
//!
//! Fitting is offline and explicitly floating point: a small recurrent decoder is trained by teacher
//! forcing with Adam and *quantisation-aware* straight-through estimators, so the served integer model
//! has the same quantized affine arithmetic used by the fitting forward pass. This is a dense
//! recurrent model implemented with bounded low-bit additive maps: the state width is declared by
//! `SlFitConfig`, map codes are ternary, embeddings are 4-bit, and biases are integer accumulators.
//! It is not a geometric-routing mechanism; no geometric advantage follows from this representation.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::lexical_realization::RealizationAction;
use super::lowbit::TernaryLinear;

/// Artifact format version.
///
/// Version 3 corrects three observation defects of version 2 with the same retained recurrence and
/// integer serving path: the content feature is position-resolved (so reordered payloads differ), the
/// final token of a long payload is carried in a dedicated tail position (so a truncated suffix is
/// not erased), and unfitted tokens occupy an explicit reserved row rather than contributing zero.
/// Version 3 additionally consumes the actual emitted token identity in the recurrent feedback and
/// exposes exact typed causal facts instead of relying on a compressed embedding to rediscover them.
pub const SL_VERSION: u8 = 3;
/// Declared maximum number of shared learned vocabulary slots.
pub const SL_MAX_SLOTS: usize = 8;
/// Declared bound on how many content tokens of one evidence item enter the content feature. The
/// selected payload and the operand are each truncated to this many tokens, so a long payload cannot
/// make the served feature width unbounded.
pub const SL_MAX_CONTENT_TOKENS: usize = 4;
/// Declared number of content positions: the first `SL_MAX_CONTENT_TOKENS` tokens plus one dedicated
/// tail position carrying the final token, so a long payload's suffix survives truncation.
pub const SL_CONTENT_POSITIONS: usize = SL_MAX_CONTENT_TOKENS + 1;
/// Width of the typed causal block: requested history, provenance (`derived`, `prior_differs`,
/// `committed`, `key_changed`) and the exact selected/operand identity relations.
pub const SL_F_DIM: usize = 15;
/// The reserved content row index for any token that was not fitted. It is a real learned row, so an
/// unsupported token is distinct from a meaningful zero.
pub const SL_OOV_ROW: usize = 0;
/// Declared bound on the magnitude of `i8` embedding entries (a 4-bit signed range).
pub const SL_EMB_BOUND: i32 = 7;
/// Declared upper bound on a ternary row's power-of-two scale, so `acc << shift` cannot overflow an
/// `i32` for the declared feature widths.
pub const SL_MAX_SHIFT: u32 = 8;

/// Row index of the `Copy` action in the readout.
pub const SL_COPY: usize = 0;
/// Row index of the `Stop` action in the readout.
pub const SL_STOP: usize = 1;
/// Row index of the first `Insert` action; slot `s` is `SL_INSERT_BASE + s`.
pub const SL_INSERT_BASE: usize = 2;

/// Feedback symbol of the first step.
const SL_SYM_START: usize = 0;
/// Feedback symbol for a copied owned token; the byte identity is owned by the session.
const SL_SYM_COPY: usize = 1;
/// Feedback symbol for `Insert(s)`.
const SL_SYM_INSERT_BASE: usize = 2;

/// One declared development example: a causal content/provenance key and the realized action
/// sequence the declared response text exhibits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlSequence {
    /// Selected owned payload tokens (the value answered).
    pub sel: Vec<u32>,
    /// Operand tokens the consumed computation started from; empty when the answer is not derived.
    pub res: Vec<u32>,
    /// Requested history: 0 current, 1 previous assertion, 2 previous distinct, 3 initial.
    pub history: u8,
    /// The answer value came from consuming a computed result.
    pub derived: bool,
    /// An older committed value for the same address was superseded before this one.
    pub prior_differs: bool,
    /// A store mutation was actually committed for this address before this answer. Distinct from
    /// `prior_differs`: a superseded record with an equal value still committed a mutation.
    pub committed: bool,
    /// The computation's derived query key differs exactly from the operand key. Distinct from
    /// `derived`: a computation may follow a key back to the operand's own address.
    pub key_changed: bool,
    /// The realized action sequence, one action per emission step, ending in `Stop`.
    pub actions: Vec<RealizationAction>,
}

impl SlSequence {
    /// The copy stage at each step, derived from the action sequence exactly as the session derives it
    /// from its cursor: 0 before the owned span, 1 inside it, 2 once it is complete.
    pub fn copy_stages(&self) -> Vec<u8> {
        let total = self
            .actions
            .iter()
            .filter(|a| matches!(a, RealizationAction::Copy))
            .count();
        let mut copied = 0usize;
        let mut stages = Vec::with_capacity(self.actions.len());
        for action in &self.actions {
            let stage = if copied == 0 {
                0
            } else if copied < total {
                1
            } else {
                2
            };
            stages.push(stage);
            if matches!(action, RealizationAction::Copy) {
                copied += 1;
            }
        }
        stages
    }

    /// The set of copy stages the session can present: the span is complete once the declared Copies
    /// have been emitted.
    pub fn copy_count(&self) -> usize {
        self.actions
            .iter()
            .filter(|a| matches!(a, RealizationAction::Copy))
            .count()
    }

    /// The typed causal facts of this example.
    pub fn facts(&self) -> SlFacts {
        SlFacts {
            history: self.history,
            derived: self.derived,
            prior_differs: self.prior_differs,
            committed: self.committed,
            key_changed: self.key_changed,
        }
    }

    /// The token the session actually emits at step `i`, given the shared vocabulary slots. A `Copy`
    /// step emits the next owned payload token in cursor order; an `Insert(slot)` step emits that
    /// slot's token. This is the identity the recurrence consumes, shared with the content embedding.
    pub fn emitted_token(&self, i: usize, slots: &[u32]) -> Option<u32> {
        match self.actions.get(i)? {
            RealizationAction::Copy => {
                let cursor = self.actions[..i]
                    .iter()
                    .filter(|a| matches!(a, RealizationAction::Copy))
                    .count();
                self.sel.get(cursor).copied()
            }
            RealizationAction::Insert(slot) => slots.get(*slot as usize).copied(),
            RealizationAction::Stop => None,
        }
    }
}

/// The causally available typed facts of one answer. Every field is an exact observation of the
/// session, not a supplied answer class: the learner still decides whether and how to verbalise them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SlFacts {
    pub history: u8,
    pub derived: bool,
    pub prior_differs: bool,
    pub committed: bool,
    pub key_changed: bool,
}

/// The typed causal block for one example. `known` reports whether a token was fitted; it is used
/// only for bounded counts and never to change an equality or a provenance fact.
pub fn typed_causal_block(
    sel: &[u32],
    res: &[u32],
    facts: SlFacts,
    known: &dyn Fn(u32) -> bool,
) -> Vec<i32> {
    let cap = SL_MAX_CONTENT_TOKENS as i32;
    let mut f = vec![0i32; SL_F_DIM];
    f[(facts.history as usize).min(3)] = 1;
    f[4] = i32::from(facts.derived);
    f[5] = i32::from(facts.prior_differs);
    f[6] = i32::from(facts.committed);
    f[7] = i32::from(facts.key_changed);
    f[8] = i32::from(!sel.is_empty() && sel == res);
    f[9] = sel
        .iter()
        .zip(res)
        .take_while(|(a, b)| a == b)
        .count()
        .min(SL_MAX_CONTENT_TOKENS) as i32;
    f[10] = (0..sel.len().max(res.len()))
        .filter(|i| sel.get(*i) != res.get(*i))
        .count()
        .min(SL_MAX_CONTENT_TOKENS) as i32;
    f[11] = (sel.len() as i32).min(cap);
    f[12] = (res.len() as i32).min(cap);
    f[13] = sel
        .iter()
        .filter(|t| !known(**t))
        .count()
        .min(SL_MAX_CONTENT_TOKENS) as i32;
    f[14] = res
        .iter()
        .filter(|t| !known(**t))
        .count()
        .min(SL_MAX_CONTENT_TOKENS) as i32;
    f
}

/// Fit configuration. Every field is declared before fitting and stored with the report.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlFitConfig {
    pub h_dim: usize,
    pub e_dim: usize,
    pub epochs: usize,
    pub lr: f32,
    pub seed: u64,
    /// Extra weight on the `Stop` target, so learned halting is not drowned by the many `Copy` steps.
    pub stop_weight: f32,
    /// Extra weight on `Insert` targets.
    pub insert_weight: f32,
}

impl Default for SlFitConfig {
    fn default() -> Self {
        Self {
            h_dim: 16,
            e_dim: 16,
            epochs: 400,
            lr: 0.05,
            seed: 0x5eed_1eaf_1234_abcd,
            stop_weight: 4.0,
            insert_weight: 2.0,
        }
    }
}

/// Fit outcome, for the run record.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlFitReport {
    pub sequences: usize,
    pub steps: usize,
    pub epochs: usize,
    pub first_loss: f32,
    pub final_loss: f32,
    /// The served objective, evaluated through the exported integer artifact.
    pub served_loss: f32,
    pub action_correct: usize,
    pub action_total: usize,
    pub h_dim: usize,
    pub e_dim: usize,
    pub content_tokens: usize,
}

/// A serde-friendly ternary linear map: the packed two-bit codes and the per-row scales.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlLinear {
    pub rows: usize,
    pub cols: usize,
    pub packed: Vec<u8>,
    pub shift: Vec<u32>,
}

impl SlLinear {
    fn from_ternary(t: &TernaryLinear) -> Self {
        Self {
            rows: t.rows,
            cols: t.cols,
            packed: t.packed().to_vec(),
            shift: (0..t.rows).map(|r| t.shift(r)).collect(),
        }
    }

    fn to_ternary(&self) -> Result<TernaryLinear, String> {
        TernaryLinear::from_packed(
            self.packed.clone(),
            self.shift.clone(),
            self.rows,
            self.cols,
        )
    }

    fn forward_i32(&self, x: &[i32]) -> Result<Vec<i32>, String> {
        Ok(self.to_ternary()?.forward_i32(x))
    }
}

/// The learned state-conditioned lexical decoder artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateLexicalModel {
    pub version: u8,
    /// Maximum token id the embedding index may reference.
    pub vocab: usize,
    /// Learned shared insert vocabulary, one token per slot.
    pub slots: Vec<u32>,
    /// Content-token index: index 0 is reserved for an unfitted token (the OOV row) and holds no
    /// token id; index `c >= 1` holds the token id whose learned rows are
    /// `c * SL_CONTENT_POSITIONS + p` for position `p`.
    pub content_tokens: Vec<u32>,
    pub h_dim: usize,
    pub e_dim: usize,
    pub f_dim: usize,
    pub c_dim: usize,
    /// Bound on the integer state magnitude.
    pub h_clamp: i32,
    /// Bound on the integer content feature magnitude.
    pub m_clamp: i32,
    /// The one shared learned content embedding, row-major
    /// `content_tokens.len() * SL_CONTENT_POSITIONS * (e_dim / 2)`. A token's row is
    /// position-dependent, so a reordered payload produces a different feature; position
    /// `SL_MAX_CONTENT_TOKENS` is the tail slot carrying a truncated payload's final token. The
    /// selected payload contributes to the first half of the content feature and the consumed
    /// computation's operand to the second. The same rows also feed the recurrent token-identity
    /// block, so a learned token identity is shared between the content observation and the emitted
    /// feedback.
    pub e: Vec<i8>,
    pub wi: SlLinear,
    pub wt: SlLinear,
    pub wo: SlLinear,
    pub bi: Vec<i32>,
    pub bt: Vec<i32>,
    pub bo: Vec<i32>,
    /// Declared bound on learned insert words per answer, mirrored from the session contract.
    pub max_insert_words: u8,
}

impl StateLexicalModel {
    /// Number of actions scored by the readout.
    pub fn n_actions(&self) -> usize {
        SL_INSERT_BASE + self.slots.len()
    }

    fn flag_dim() -> usize {
        SL_F_DIM
    }

    fn copy_dim() -> usize {
        3
    }

    /// Width of the derived selected-minus-operand block. It is the declared difference between the
    /// two content halves, so the bounded clamp can threshold a content difference directly.
    fn diff_dim(&self) -> usize {
        self.e_dim / 2
    }

    fn readout_cols(&self) -> usize {
        self.h_dim + self.e_dim + self.diff_dim() + Self::flag_dim() + Self::copy_dim()
    }

    fn init_cols(&self) -> usize {
        self.e_dim + self.diff_dim() + Self::flag_dim()
    }

    /// The declared difference between the selected and operand content halves.
    fn difference_block(&self, m: &[i32]) -> Vec<i32> {
        let half = self.e_dim / 2;
        (0..half).map(|i| m[i] - m[half + i]).collect()
    }

    fn transition_cols(&self) -> usize {
        self.h_dim + Self::symbol_dim(self.slots.len()) + self.diff_dim() + Self::flag_dim()
    }

    fn symbol_dim(n_slots: usize) -> usize {
        SL_SYM_INSERT_BASE + n_slots
    }

    /// The learned content row index for a token id, or `None` when no row was fitted for it. Index
    /// 0 is reserved for unfitted tokens, so it is never returned here.
    fn embed_row(&self, token: u32) -> Option<usize> {
        self.content_tokens
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, t)| **t == token)
            .map(|(i, _)| i)
    }

    /// The embedding row for `token` at content position `p`. An unfitted token uses the reserved
    /// OOV row, so unsupported content is a real learned row rather than a meaningful zero.
    #[allow(dead_code)]
    fn content_row(&self, token: u32, p: usize) -> usize {
        let c = self.embed_row(token).unwrap_or(SL_OOV_ROW);
        c * SL_CONTENT_POSITIONS + p.min(SL_CONTENT_POSITIONS - 1)
    }

    /// The ordered (token-row, position) terms of one token slice: the first `SL_MAX_CONTENT_TOKENS`
    /// tokens in order, plus the final token in the dedicated tail position when the slice is longer.
    fn content_terms(&self, tokens: &[u32]) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = tokens
            .iter()
            .take(SL_MAX_CONTENT_TOKENS)
            .enumerate()
            .map(|(p, t)| (self.embed_row(*t).unwrap_or(SL_OOV_ROW), p))
            .collect();
        if tokens.len() > SL_MAX_CONTENT_TOKENS {
            let last = tokens[tokens.len() - 1];
            out.push((
                self.embed_row(last).unwrap_or(SL_OOV_ROW),
                SL_MAX_CONTENT_TOKENS,
            ));
        }
        out
    }

    /// The content feature `m` for a selected payload and a computation operand. Each token
    /// contributes its learned row for its own position, so reordering changes the feature and a
    /// truncated payload keeps its final token in the tail position. `res` is empty for a direct read.
    pub fn content_feature(&self, sel: &[u32], res: &[u32]) -> Vec<i32> {
        let half = self.e_dim / 2;
        let mut m = vec![0i32; self.e_dim];
        for (c, p) in self.content_terms(sel) {
            let row = (c * SL_CONTENT_POSITIONS + p) * half;
            for (d, v) in self.e[row..row + half].iter().enumerate() {
                m[d] += i32::from(*v);
            }
        }
        for (c, p) in self.content_terms(res) {
            let row = (c * SL_CONTENT_POSITIONS + p) * half;
            for (d, v) in self.e[row..row + half].iter().enumerate() {
                m[half + d] += i32::from(*v);
            }
        }
        clamp_in_place(&mut m, self.m_clamp);
        m
    }

    /// The typed causal block, using this artifact's content rows for the bounded unknown counts.
    pub fn typed_block(&self, sel: &[u32], res: &[u32], facts: SlFacts) -> Vec<i32> {
        typed_causal_block(sel, res, facts, &|t| self.embed_row(t).is_some())
    }

    /// `h_0 = learned_init(c)`.
    pub fn init_state(&self, m: &[i32], f: &[i32]) -> Vec<i32> {
        let mut input = Vec::with_capacity(self.init_cols());
        input.extend_from_slice(m);
        input.extend_from_slice(&self.difference_block(m));
        input.extend_from_slice(f);
        let mut h = self
            .wi
            .forward_i32(&input)
            .expect("validated init map shape");
        for (h_i, b) in h.iter_mut().zip(&self.bi) {
            *h_i += *b;
        }
        clamp_in_place(&mut h, self.h_clamp);
        h
    }

    /// `a_t = learned_readout(h_t, c, exact_copy_state)` as unnormalised row scores.
    pub fn readout(&self, h: &[i32], m: &[i32], f: &[i32], copy_stage: u8) -> Vec<i32> {
        let mut input = Vec::with_capacity(self.readout_cols());
        input.extend_from_slice(h);
        input.extend_from_slice(m);
        input.extend_from_slice(&self.difference_block(m));
        input.extend_from_slice(f);
        let mut copy = vec![0i32; Self::copy_dim()];
        copy[(copy_stage as usize).min(2)] = 1;
        input.extend_from_slice(&copy);
        let mut logits = self
            .wo
            .forward_i32(&input)
            .expect("validated readout map shape");
        for (l, b) in logits.iter_mut().zip(&self.bo) {
            *l += *b;
        }
        logits
    }

    /// `h_(t+1) = learned_transition(h_t, action_symbol, emitted_token, c)` for the executed event.
    /// The actual emitted token's shared learned identity and the typed causal block both advance the
    /// state, so a copied token is not collapsed into one Copy symbol and the post-copy decision can
    /// read the causal facts from the state as well as from the readout. An unfitted token uses the
    /// reserved row.
    pub fn transition(&self, h: &[i32], symbol: usize, token: Option<u32>, f: &[i32]) -> Vec<i32> {
        let half = self.e_dim / 2;
        let mut input = Vec::with_capacity(self.transition_cols());
        input.extend_from_slice(h);
        let mut x = vec![0i32; Self::symbol_dim(self.slots.len())];
        let idx = symbol.min(x.len() - 1);
        x[idx] = 1;
        input.extend_from_slice(&x);
        // The identity block reuses the token's position-0 learned row, so content and feedback share
        // one learned token representation.
        let c = token.and_then(|t| self.embed_row(t)).unwrap_or(SL_OOV_ROW);
        let row = (c * SL_CONTENT_POSITIONS) * half;
        input.extend(self.e[row..row + half].iter().map(|v| i32::from(*v)));
        input.extend_from_slice(f);
        let mut next = self
            .wt
            .forward_i32(&input)
            .expect("validated transition map shape");
        for (n, b) in next.iter_mut().zip(&self.bt) {
            *n += *b;
        }
        clamp_in_place(&mut next, self.h_clamp);
        next
    }

    /// The feedback symbol that `execute(action)` feeds back.
    pub fn symbol_of(action: RealizationAction) -> usize {
        match action {
            RealizationAction::Copy => SL_SYM_COPY,
            RealizationAction::Insert(slot) => SL_SYM_INSERT_BASE + slot as usize,
            // Stop terminates, so its symbol is never fed back; the start symbol is the fallback.
            RealizationAction::Stop => SL_SYM_START,
        }
    }

    /// The table's own decision at one step, with a deterministic tie-break toward the lower index.
    pub fn decide(
        &self,
        h: &[i32],
        m: &[i32],
        f: &[i32],
        copy_stage: u8,
    ) -> (RealizationAction, Vec<i32>) {
        let logits = self.readout(h, m, f, copy_stage);
        let mut best = 0usize;
        for (i, score) in logits.iter().enumerate() {
            if *score > logits[best] {
                best = i;
            }
        }
        let action = match best {
            SL_COPY => RealizationAction::Copy,
            SL_STOP => RealizationAction::Stop,
            _ => RealizationAction::Insert((best - SL_INSERT_BASE) as u8),
        };
        (action, logits)
    }

    /// A stable digest of the integer state, for diagnostics only.
    pub fn state_digest(h: &[i32]) -> u64 {
        let mut acc = 0xcbf2_9ce4_8422_2325u64;
        for v in h {
            acc ^= (*v as i64 as u64).wrapping_mul(0x100_0000_01b3);
            acc = acc.rotate_left(27).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
        acc
    }

    /// Structural validation of a loaded or fitted artifact.
    pub fn validate(&self, max_vocab: usize) -> Result<(), String> {
        if self.version != SL_VERSION {
            return Err("unsupported state-lexical artifact version".into());
        }
        if self.vocab == 0 || self.vocab > max_vocab {
            return Err("state-lexical vocabulary is out of range".into());
        }
        if self.slots.len() > SL_MAX_SLOTS {
            return Err("state-lexical slot count exceeds the declared bound".into());
        }
        if self.slots.iter().any(|t| *t as usize >= self.vocab) {
            return Err("state-lexical slot token is outside the vocabulary".into());
        }
        if self.content_tokens.is_empty() {
            return Err("state-lexical artifact needs at least one content token".into());
        }
        if self
            .content_tokens
            .iter()
            .any(|t| *t as usize >= self.vocab)
        {
            return Err("state-lexical content token is outside the vocabulary".into());
        }
        if self.e_dim == 0
            || self.e_dim % 2 != 0
            || self.h_dim == 0
            || self.f_dim != Self::flag_dim()
            || self.c_dim != Self::copy_dim()
        {
            return Err("state-lexical artefact declares inconsistent feature widths".into());
        }
        let half = self.e_dim / 2;
        let embedding_len = self
            .content_tokens
            .len()
            .checked_mul(SL_CONTENT_POSITIONS)
            .and_then(|n| n.checked_mul(half))
            .ok_or("state-lexical embedding dimensions overflow")?;
        let init_cols = self
            .e_dim
            .checked_add(half)
            .and_then(|n| n.checked_add(Self::flag_dim()))
            .ok_or("state-lexical input dimensions overflow")?;
        let transition_cols = self
            .h_dim
            .checked_add(Self::symbol_dim(self.slots.len()))
            .and_then(|n| n.checked_add(half))
            .and_then(|n| n.checked_add(Self::flag_dim()))
            .ok_or("state-lexical transition dimensions overflow")?;
        let readout_cols = self
            .h_dim
            .checked_add(init_cols)
            .and_then(|n| n.checked_add(Self::copy_dim()))
            .ok_or("state-lexical readout dimensions overflow")?;
        if self.e.len() != embedding_len {
            return Err("state-lexical embedding table shape disagrees with its index".into());
        }
        if self.e.iter().any(|v| (*v as i32).abs() > SL_EMB_BOUND) {
            return Err("state-lexical embedding exceeds its declared 4-bit bound".into());
        }
        if self.h_clamp <= 0 || self.m_clamp <= 0 {
            return Err("state-lexical clamps must be positive".into());
        }
        let n = self.n_actions();
        if self.wi.rows != self.h_dim
            || self.wi.cols != init_cols
            || self.wt.rows != self.h_dim
            || self.wt.cols != transition_cols
            || self.wo.rows != n
            || self.wo.cols != readout_cols
        {
            return Err("state-lexical linear map shapes are inconsistent".into());
        }
        if self.bi.len() != self.h_dim || self.bt.len() != self.h_dim || self.bo.len() != n {
            return Err("state-lexical bias shape is inconsistent".into());
        }
        for map in [&self.wi, &self.wt, &self.wo] {
            if map.shift.len() != map.rows {
                return Err("state-lexical scale length disagrees with rows".into());
            }
            if map.shift.iter().any(|s| *s > SL_MAX_SHIFT) {
                return Err("state-lexical scale exceeds the declared serving bound".into());
            }
            // Check products before the older generic constructor performs its shape arithmetic.
            let weights = map
                .rows
                .checked_mul(map.cols)
                .ok_or("state-lexical packed dimensions overflow")?;
            if map.packed.len() != weights.div_ceil(4) {
                return Err("state-lexical packed length disagrees with dimensions".into());
            }
            map.to_ternary()?;
        }
        // Certify the accumulator before executing add/sub/shift and adding a bias. A shift
        // bound alone is insufficient: a valid-shaped artifact can contain i32::MAX biases or
        // an unsafe recurrent clamp. These conservative absolute bounds cover every causal input.
        let content_bound = self.m_clamp.min(SL_EMB_BOUND * SL_CONTENT_POSITIONS as i32);
        let mut context_bounds = vec![content_bound as u128; self.e_dim];
        context_bounds.extend(vec![2 * content_bound as u128; half]);
        context_bounds.extend(vec![SL_MAX_CONTENT_TOKENS.max(1) as u128; Self::flag_dim()]);
        validate_affine_range(&self.wi, &self.bi, &context_bounds)?;
        let mut transition_bounds = vec![self.h_clamp as u128; self.h_dim];
        transition_bounds.extend(vec![1; Self::symbol_dim(self.slots.len())]);
        transition_bounds.extend(vec![SL_EMB_BOUND as u128; half]);
        transition_bounds.extend(vec![SL_MAX_CONTENT_TOKENS.max(1) as u128; Self::flag_dim()]);
        validate_affine_range(&self.wt, &self.bt, &transition_bounds)?;
        let mut readout_bounds = vec![self.h_clamp as u128; self.h_dim];
        readout_bounds.extend(context_bounds);
        readout_bounds.extend([1; 3]);
        validate_affine_range(&self.wo, &self.bo, &readout_bounds)?;
        if self.max_insert_words == 0 || self.max_insert_words > SL_MAX_SLOTS as u8 {
            return Err("state-lexical insert-word bound is out of range".into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let model: StateLexicalModel =
            serde_json::from_slice(bytes).map_err(|e| format!("state-lexical artifact: {e}"))?;
        model.validate(max_vocab)?;
        Ok(model)
    }

    /// Teacher-forced action agreement through the exported integer artifact: the fit's own witness
    /// that the served arithmetic reproduces the declared sequences.
    pub fn teacher_forced_agreement(&self, examples: &[SlSequence]) -> (usize, usize) {
        let mut correct = 0usize;
        let mut total = 0usize;
        for ex in examples {
            let m = self.content_feature(&ex.sel, &ex.res);
            let f = self.typed_block(&ex.sel, &ex.res, ex.facts());
            let mut h = self.init_state(&m, &f);
            let stages = ex.copy_stages();
            for (i, action) in ex.actions.iter().enumerate() {
                let (chosen, _) = self.decide(&h, &m, &f, stages[i]);
                if chosen == *action {
                    correct += 1;
                }
                total += 1;
                h = self.transition(
                    &h,
                    Self::symbol_of(*action),
                    ex.emitted_token(i, &self.slots),
                    &f,
                );
            }
        }
        (correct, total)
    }
}

/// Certify all additions and the final shift/bias without running the i32 kernel.
fn validate_affine_range(map: &SlLinear, bias: &[i32], bounds: &[u128]) -> Result<(), String> {
    let weights = map.to_ternary()?;
    for (r, b) in bias.iter().enumerate() {
        let mut sum = 0u128;
        for (c, bound) in bounds.iter().enumerate() {
            if weights.weight(r, c) != 0 {
                sum = sum
                    .checked_add(*bound)
                    .ok_or("state-lexical accumulator bound overflow")?;
            }
        }
        let shifted = sum
            .checked_shl(map.shift[r])
            .and_then(|n| n.checked_add(i64::from(*b).unsigned_abs() as u128))
            .ok_or("state-lexical shifted bound overflow")?;
        if sum > i32::MAX as u128 || shifted > i32::MAX as u128 {
            return Err("state-lexical affine map exceeds its safe i32 accumulator range".into());
        }
    }
    Ok(())
}

fn clamp_in_place(v: &mut [i32], bound: i32) {
    for x in v.iter_mut() {
        if *x > bound {
            *x = bound;
        } else if *x < -bound {
            *x = -bound;
        }
    }
}

// ---------------------------------------------------------------------------
// Offline fitting: teacher forcing with Adam and quantisation-aware STEs.
// ---------------------------------------------------------------------------

const Q8_LO: f32 = -(SL_EMB_BOUND as f32);
const Q8_HI: f32 = SL_EMB_BOUND as f32;

/// Round-and-clamp to the 4-bit signed range.
fn q8(x: f32) -> f32 {
    x.round().clamp(Q8_LO, Q8_HI)
}

/// Mirror of `TernaryLinear::quantize`'s per-row scale so training and serving agree exactly.
fn ternary_scale(row: &[f32]) -> f32 {
    let amax = row.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    if amax > 0.0 {
        let s = amax.log2().floor().clamp(0.0, SL_MAX_SHIFT as f32);
        2f32.powi(s as i32)
    } else {
        1.0
    }
}

/// Quantise one row to ternary codes (straight through), writing the effective weights
/// `code * scale` so a forward pass can use a plain inner product.
fn q_ternary_row(row: &[f32], out: &mut [f32]) {
    let scale = ternary_scale(row);
    for (o, v) in out.iter_mut().zip(row) {
        let q = (*v / scale).round().clamp(-1.0, 1.0);
        *o = q * scale;
    }
}

/// Quantise a whole row-major matrix.
fn q_ternary(w: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    let mut out = vec![0f32; rows * cols];
    for r in 0..rows {
        q_ternary_row(
            &w[r * cols..(r + 1) * cols],
            &mut out[r * cols..(r + 1) * cols],
        );
    }
    out
}

/// Offline QAT affine evaluation. Bias rounding uses an identity STE in backward, matching
/// the integer biases exported to serving rather than changing them only at export time.
fn quantized_affine(weights: &[f32], input: &[f32], bias: &[f32]) -> Vec<f32> {
    weights
        .chunks_exact(input.len())
        .zip(bias)
        .map(|(row, b)| row.iter().zip(input).map(|(w, x)| w * x).sum::<f32>() + b.round())
        .collect()
}

/// Derivative of the declared globally normalized weighted action cross entropy.
fn weighted_target_gradient(p: &[f32], target: usize, weight: f32, total_weight: f32) -> Vec<f32> {
    p.iter()
        .enumerate()
        .map(|(i, probability)| {
            (probability - if i == target { 1.0 } else { 0.0 }) * weight / total_weight
        })
        .collect()
}

/// Deterministic xorshift64* RNG for reproducible initialisation.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn scaled(&mut self, s: f32) -> f32 {
        let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        ((u * 2.0 - 1.0) as f32) * s
    }
}

/// Adam state for one parameter tensor.
struct Adam {
    m: Vec<f32>,
    v: Vec<f32>,
    t: u64,
}

impl Adam {
    fn new(n: usize) -> Self {
        Self {
            m: vec![0.0; n],
            v: vec![0.0; n],
            t: 0,
        }
    }

    fn step(&mut self, p: &mut [f32], g: &[f32], lr: f32) {
        self.t += 1;
        let (b1, b2, eps) = (0.9f32, 0.999f32, 1e-8f32);
        let bc1 = 1.0 - b1.powi(self.t as i32);
        let bc2 = 1.0 - b2.powi(self.t as i32);
        for i in 0..p.len() {
            self.m[i] = b1 * self.m[i] + (1.0 - b1) * g[i];
            self.v[i] = b2 * self.v[i] + (1.0 - b2) * g[i] * g[i];
            p[i] -= lr * (self.m[i] / bc1) / ((self.v[i] / bc2).sqrt() + eps);
        }
    }
}

/// A hand-written float model used only for offline fitting.
struct FloatModel {
    h_dim: usize,
    e_dim: usize,
    f_dim: usize,
    c_dim: usize,
    n_slots: usize,
    n_actions: usize,
    h_clamp: f32,
    m_clamp: f32,
    e: Vec<f32>,
    wi: Vec<f32>,
    wt: Vec<f32>,
    wo: Vec<f32>,
    bi: Vec<f32>,
    bt: Vec<f32>,
    bo: Vec<f32>,
}

impl FloatModel {
    fn new(content_rows: usize, n_slots: usize, cfg: &SlFitConfig, rng: &mut Rng) -> Self {
        let h_dim = cfg.h_dim;
        let e_dim = cfg.e_dim;
        let (f_dim, c_dim) = (SL_F_DIM, 3usize);
        let n_actions = SL_INSERT_BASE + n_slots;
        let half = e_dim / 2;
        let mut emb = Vec::with_capacity(content_rows * half);
        for _ in 0..content_rows * half {
            emb.push(rng.scaled(2.4));
        }
        let mk =
            |n: usize, rng: &mut Rng| -> Vec<f32> { (0..n).map(|_| rng.scaled(1.1)).collect() };
        Self {
            h_dim,
            e_dim,
            f_dim,
            c_dim,
            n_slots,
            n_actions,
            h_clamp: 255.0,
            m_clamp: 255.0,
            e: emb,
            wi: mk(h_dim * (e_dim + half + f_dim), rng),
            wt: mk(
                h_dim * (h_dim + SL_SYM_INSERT_BASE + n_slots + half + f_dim),
                rng,
            ),
            wo: mk(n_actions * (h_dim + e_dim + half + f_dim + c_dim), rng),
            bi: vec![0.0; h_dim],
            bt: vec![0.0; h_dim],
            bo: vec![0.0; n_actions],
        }
    }

    fn diff_dim(&self) -> usize {
        self.e_dim / 2
    }
    fn wi_cols(&self) -> usize {
        self.e_dim + self.diff_dim() + self.f_dim
    }
    fn wt_cols(&self) -> usize {
        self.h_dim + SL_SYM_INSERT_BASE + self.n_slots + self.diff_dim() + self.f_dim
    }
    fn wo_cols(&self) -> usize {
        self.h_dim + self.e_dim + self.diff_dim() + self.f_dim + self.c_dim
    }
}

fn action_index(action: RealizationAction) -> usize {
    match action {
        RealizationAction::Copy => SL_COPY,
        RealizationAction::Stop => SL_STOP,
        RealizationAction::Insert(s) => SL_INSERT_BASE + s as usize,
    }
}

fn symbol_index(action: RealizationAction) -> usize {
    match action {
        RealizationAction::Copy => SL_SYM_COPY,
        RealizationAction::Insert(s) => SL_SYM_INSERT_BASE + s as usize,
        RealizationAction::Stop => SL_SYM_START,
    }
}

fn action_weight(action: RealizationAction, stop_weight: f32, insert_weight: f32) -> f32 {
    match action {
        RealizationAction::Copy => 1.0,
        RealizationAction::Stop => stop_weight,
        RealizationAction::Insert(_) => insert_weight,
    }
}

/// The fitted artifact and its fit report.
pub struct SlFitOutcome {
    pub model: StateLexicalModel,
    pub report: SlFitReport,
}

/// Fit the state-conditioned decoder by teacher forcing with Adam and quantisation-aware STEs.
///
/// The content-token index is taken from the examples, with an ordinary token-0 row retained.
/// `oov_tokens` declares token ids that must stay *unfamiliar*: they are excluded from the content
/// index so an unknown-token control is faithful rather than silently fitted. Every other distinct
/// observed token adds one row; unknown serving tokens use the reserved OOV row.
pub fn fit_state_lexical(
    examples: &[SlSequence],
    vocab: usize,
    slots: Vec<u32>,
    max_insert_words: u8,
    cfg: &SlFitConfig,
    oov_tokens: &[u32],
) -> Result<SlFitOutcome, String> {
    if examples.is_empty() {
        return Err("state-lexical fit requires at least one example".into());
    }
    if slots.len() > SL_MAX_SLOTS
        || slots.iter().any(|t| *t as usize >= vocab)
        || vocab == 0
        || max_insert_words == 0
        || max_insert_words > SL_MAX_SLOTS as u8
        || cfg.h_dim == 0
        || cfg.e_dim == 0
        || cfg.e_dim % 2 != 0
        || cfg.epochs == 0
        || !cfg.lr.is_finite()
        || cfg.lr <= 0.0
        || !cfg.stop_weight.is_finite()
        || cfg.stop_weight <= 0.0
        || !cfg.insert_weight.is_finite()
        || cfg.insert_weight <= 0.0
    {
        return Err(
            "state-lexical fit has invalid vocabulary, dimensions or optimizer configuration"
                .into(),
        );
    }
    // These are authored action sequences, including content-blanked controls; their Copies need
    // not equal sel.len(), but every sequence must have exactly one final Stop and valid slots.
    if examples.iter().any(|ex| {
        ex.history > 3
            || ex.sel.iter().chain(&ex.res).any(|t| *t as usize >= vocab)
            || ex.actions.last() != Some(&RealizationAction::Stop)
            || ex.actions[..ex.actions.len().saturating_sub(1)]
                .iter()
                .any(|a| matches!(a, RealizationAction::Stop))
            || ex
                .actions
                .iter()
                .any(|a| matches!(a, RealizationAction::Insert(s) if *s as usize >= slots.len()))
    }) {
        return Err("state-lexical fit has an invalid supervised sequence".into());
    }
    let total_weight: f32 = examples
        .iter()
        .flat_map(|ex| &ex.actions)
        .map(|a| action_weight(*a, cfg.stop_weight, cfg.insert_weight))
        .sum();
    if !total_weight.is_finite() || total_weight <= 0.0 {
        return Err("state-lexical objective weight is not finite and positive".into());
    }
    let half = cfg.e_dim / 2;
    let init_cols = cfg
        .e_dim
        .checked_add(half)
        .and_then(|n| n.checked_add(SL_F_DIM))
        .ok_or("state-lexical fit input dimensions overflow")?;
    let transition_cols = cfg
        .h_dim
        .checked_add(SL_SYM_INSERT_BASE + slots.len())
        .and_then(|n| n.checked_add(half))
        .ok_or("state-lexical fit transition dimensions overflow")?;
    let readout_cols = cfg
        .h_dim
        .checked_add(init_cols)
        .and_then(|n| n.checked_add(3))
        .ok_or("state-lexical fit readout dimensions overflow")?;
    cfg.h_dim
        .checked_mul(init_cols)
        .ok_or("state-lexical fit initialization size overflows")?;
    cfg.h_dim
        .checked_mul(transition_cols)
        .ok_or("state-lexical fit transition size overflows")?;
    (SL_INSERT_BASE + slots.len())
        .checked_mul(readout_cols)
        .ok_or("state-lexical fit readout size overflows")?;
    // Index 0 is the reserved OOV row (it holds a placeholder, never a fitted token).
    let mut content_tokens = vec![0u32];
    for ex in examples {
        for t in ex.sel.iter().chain(ex.res.iter()) {
            if !oov_tokens.contains(t) && !content_tokens[1..].contains(t) {
                content_tokens.push(*t);
            }
        }
    }
    let content_len = content_tokens.len();
    let content_rows = content_len
        .checked_mul(SL_CONTENT_POSITIONS)
        .ok_or("state-lexical fit content rows overflow")?;
    content_rows
        .checked_mul(half)
        .ok_or("state-lexical fit embedding size overflows")?;
    // The learned row of `t` at content position `p`; an unfitted token uses the reserved row.
    let content_row = |t: u32, p: usize| {
        let c = content_tokens
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, x)| **x == t)
            .map(|(i, _)| i)
            .unwrap_or(SL_OOV_ROW);
        c * SL_CONTENT_POSITIONS + p.min(SL_CONTENT_POSITIONS - 1)
    };
    let terms = |tokens: &[u32]| -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = tokens
            .iter()
            .take(SL_MAX_CONTENT_TOKENS)
            .enumerate()
            .map(|(p, t)| (content_row(*t, p), p))
            .collect();
        if tokens.len() > SL_MAX_CONTENT_TOKENS {
            out.push((
                content_row(tokens[tokens.len() - 1], SL_MAX_CONTENT_TOKENS),
                SL_MAX_CONTENT_TOKENS,
            ));
        }
        out
    };
    let known = |t: u32| !oov_tokens.contains(&t) && content_tokens[1..].contains(&t);

    let mut rng = Rng(cfg.seed);
    let mut fm = FloatModel::new(content_rows, slots.len(), cfg, &mut rng);

    let n_wi = fm.wi.len();
    let n_wt = fm.wt.len();
    let n_wo = fm.wo.len();
    let mut a_e = Adam::new(fm.e.len());
    let mut a_wi = Adam::new(n_wi);
    let mut a_wt = Adam::new(n_wt);
    let mut a_wo = Adam::new(n_wo);
    let mut a_bi = Adam::new(fm.bi.len());
    let mut a_bt = Adam::new(fm.bt.len());
    let mut a_bo = Adam::new(fm.bo.len());

    let (wi_cols, wt_cols, wo_cols) = (fm.wi_cols(), fm.wt_cols(), fm.wo_cols());
    let total_steps: usize = examples.iter().map(|e| e.actions.len()).sum();
    let (mut first_loss, mut final_loss) = (f32::NAN, f32::NAN);

    for epoch in 0..cfg.epochs {
        let lr_t = cfg.lr;
        let e_q: Vec<f32> = fm.e.iter().map(|x| q8(*x)).collect();
        let wi_q = q_ternary(&fm.wi, fm.h_dim, wi_cols);
        let wt_q = q_ternary(&fm.wt, fm.h_dim, wt_cols);
        let wo_q = q_ternary(&fm.wo, fm.n_actions, wo_cols);

        let mut g_e = vec![0f32; fm.e.len()];
        let mut g_wi = vec![0f32; n_wi];
        let mut g_wt = vec![0f32; n_wt];
        let mut g_wo = vec![0f32; n_wo];
        let mut g_bi = vec![0f32; fm.bi.len()];
        let mut g_bt = vec![0f32; fm.bt.len()];
        let mut g_bo = vec![0f32; fm.bo.len()];

        let mut epoch_loss = 0.0f32;
        let mut weight_sum = 0f32;

        for ex in examples {
            // ---- forward, keeping per-step activations ----
            let half = fm.e_dim / 2;
            let mut m = vec![0f32; fm.e_dim];
            let sel_terms = terms(&ex.sel);
            let res_terms = terms(&ex.res);
            for (row, _) in &sel_terms {
                let base = row * half;
                for d in 0..half {
                    m[d] += e_q[base + d];
                }
            }
            for (row, _) in &res_terms {
                let base = row * half;
                for d in 0..half {
                    m[half + d] += e_q[base + d];
                }
            }
            let m_mask: Vec<bool> = m.iter().map(|v| v.abs() < fm.m_clamp).collect();
            clamp_in_place_f(&mut m, fm.m_clamp);
            let f: Vec<f32> = typed_causal_block(&ex.sel, &ex.res, ex.facts(), &known)
                .iter()
                .map(|v| *v as f32)
                .collect();
            let mut u0 = vec![0f32; wi_cols];
            u0[..fm.e_dim].copy_from_slice(&m);
            for i in 0..half {
                u0[fm.e_dim + i] = m[i] - m[half + i];
            }
            u0[fm.e_dim + half..].copy_from_slice(&f);
            let mut h = quantized_affine(&wi_q, &u0, &fm.bi);
            let mut h_hist: Vec<Vec<f32>> = Vec::with_capacity(ex.actions.len() + 1);
            let mut h_masks: Vec<Vec<bool>> = Vec::with_capacity(ex.actions.len() + 1);
            let mut masks = Vec::with_capacity(fm.h_dim);
            for r in 0..fm.h_dim {
                masks.push(h[r].abs() < fm.h_clamp);
            }
            clamp_in_place_f(&mut h, fm.h_clamp);
            h_hist.push(h.clone());
            h_masks.push(masks);

            let stages = ex.copy_stages();
            let mut cache: Vec<(Vec<f32>, Vec<f32>, usize, f32)> =
                Vec::with_capacity(ex.actions.len());
            for (i, action) in ex.actions.iter().enumerate() {
                let y = action_index(*action);
                let mut z = Vec::with_capacity(wo_cols);
                z.extend_from_slice(&h);
                z.extend_from_slice(&m);
                for i in 0..half {
                    z.push(m[i] - m[half + i]);
                }
                z.extend_from_slice(&f);
                let mut copy = vec![0f32; fm.c_dim];
                copy[(stages[i] as usize).min(2)] = 1.0;
                z.extend_from_slice(&copy);
                let mut logits = quantized_affine(&wo_q, &z, &fm.bo);
                let max = logits.iter().fold(f32::NEG_INFINITY, |a, b| a.max(*b));
                let target_logit = logits[y];
                let mut sum = 0.0f32;
                for l in logits.iter_mut() {
                    *l = (*l - max).exp();
                    sum += *l;
                }
                let p: Vec<f32> = logits.iter().map(|l| l / sum).collect();
                let weight = action_weight(*action, cfg.stop_weight, cfg.insert_weight);
                epoch_loss += weight * ((max - target_logit) + sum.ln());
                weight_sum += weight;
                cache.push((z, p, y, weight));

                let x = symbol_index(*action);
                let mut input = Vec::with_capacity(wt_cols);
                input.extend_from_slice(&h);
                let mut xs = vec![0f32; SL_SYM_INSERT_BASE + fm.n_slots];
                xs[x] = 1.0;
                input.extend_from_slice(&xs);
                let tok_row = ex
                    .emitted_token(i, &slots)
                    .map(|t| content_row(t, 0))
                    .unwrap_or(SL_OOV_ROW * SL_CONTENT_POSITIONS);
                let tok_base = tok_row * half;
                input.extend_from_slice(&e_q[tok_base..tok_base + half]);
                input.extend_from_slice(&f);
                h = quantized_affine(&wt_q, &input, &fm.bt);
                let mut mask = Vec::with_capacity(fm.h_dim);
                for r in 0..fm.h_dim {
                    mask.push(h[r].abs() < fm.h_clamp);
                }
                clamp_in_place_f(&mut h, fm.h_clamp);
                h_hist.push(h.clone());
                h_masks.push(mask);
            }

            // ---- backward ----
            let mut gh = vec![0f32; fm.h_dim];
            let mut gm = vec![0f32; fm.e_dim];
            for t in (0..cache.len()).rev() {
                let (z, p, y, weight) = &cache[t];
                let output_gradient = weighted_target_gradient(p, *y, *weight, total_weight);
                for r in 0..fm.n_actions {
                    let dlogit = output_gradient[r];
                    if dlogit == 0.0 {
                        continue;
                    }
                    g_bo[r] += dlogit;
                    let base = r * wo_cols;
                    for c in 0..wo_cols {
                        g_wo[base + c] += dlogit * z[c];
                    }
                    for d in 0..fm.h_dim {
                        gh[d] += dlogit * wo_q[base + d];
                    }
                    for d in 0..fm.e_dim {
                        gm[d] += dlogit * wo_q[base + fm.h_dim + d];
                    }
                    for i in 0..half {
                        let g = dlogit * wo_q[base + fm.h_dim + fm.e_dim + i];
                        gm[i] += g;
                        gm[half + i] -= g;
                    }
                }
                if t > 0 {
                    let prev = ex.actions[t - 1];
                    let x = symbol_index(prev);
                    let mut input = Vec::with_capacity(wt_cols);
                    input.extend_from_slice(&h_hist[t - 1]);
                    let mut xs = vec![0f32; SL_SYM_INSERT_BASE + fm.n_slots];
                    xs[x] = 1.0;
                    input.extend_from_slice(&xs);
                    let prev_tok_row = ex
                        .emitted_token(t - 1, &slots)
                        .map(|t| content_row(t, 0))
                        .unwrap_or(SL_OOV_ROW * SL_CONTENT_POSITIONS);
                    let prev_tok_base = prev_tok_row * half;
                    input.extend_from_slice(&e_q[prev_tok_base..prev_tok_base + half]);
                    input.extend_from_slice(&f);
                    let mut dpre = vec![0f32; fm.h_dim];
                    for r in 0..fm.h_dim {
                        dpre[r] = if h_masks[t][r] { gh[r] } else { 0.0 };
                        g_bt[r] += dpre[r];
                        for c in 0..wt_cols {
                            g_wt[r * wt_cols + c] += dpre[r] * input[c];
                        }
                    }
                    // The emitted token's shared identity row receives the feedback gradient too.
                    let emb_col = fm.h_dim + SL_SYM_INSERT_BASE + fm.n_slots;
                    for j in 0..half {
                        let mut acc = 0.0f32;
                        for r in 0..fm.h_dim {
                            acc += dpre[r] * wt_q[r * wt_cols + emb_col + j];
                        }
                        g_e[prev_tok_base + j] += acc;
                    }
                    let mut new_gh = vec![0f32; fm.h_dim];
                    for c in 0..fm.h_dim {
                        let mut acc = 0.0f32;
                        for r in 0..fm.h_dim {
                            acc += dpre[r] * wt_q[r * wt_cols + c];
                        }
                        new_gh[c] = acc;
                    }
                    gh = new_gh;
                } else {
                    let mut dpre = vec![0f32; fm.h_dim];
                    for r in 0..fm.h_dim {
                        dpre[r] = if h_masks[0][r] { gh[r] } else { 0.0 };
                        g_bi[r] += dpre[r];
                    }
                    for r in 0..fm.h_dim {
                        for c in 0..wi_cols {
                            g_wi[r * wi_cols + c] += dpre[r] * u0[c];
                        }
                    }
                    for d in 0..fm.e_dim {
                        let mut acc = 0.0f32;
                        for r in 0..fm.h_dim {
                            acc += dpre[r] * wi_q[r * wi_cols + d];
                        }
                        gm[d] += acc;
                    }
                    for i in 0..half {
                        let mut acc = 0.0f32;
                        for r in 0..fm.h_dim {
                            acc += dpre[r] * wi_q[r * wi_cols + fm.e_dim + i];
                        }
                        gm[i] += acc;
                        gm[half + i] -= acc;
                    }
                }
            }
            let half = fm.e_dim / 2;
            for (row, _p) in &sel_terms {
                let base = row * half;
                for d in 0..half {
                    if m_mask[d] && gm[d] != 0.0 {
                        g_e[base + d] += gm[d];
                    }
                }
            }
            for (row, _p) in &res_terms {
                let base = row * half;
                for d in 0..half {
                    if m_mask[half + d] && gm[half + d] != 0.0 {
                        g_e[base + d] += gm[half + d];
                    }
                }
            }
        }

        let avg = epoch_loss / weight_sum.max(1e-9);
        if epoch == 0 {
            first_loss = avg;
        }
        final_loss = avg;

        a_e.step(&mut fm.e, &g_e, lr_t);
        a_wi.step(&mut fm.wi, &g_wi, lr_t);
        a_wt.step(&mut fm.wt, &g_wt, lr_t);
        a_wo.step(&mut fm.wo, &g_wo, lr_t);
        a_bi.step(&mut fm.bi, &g_bi, lr_t);
        a_bt.step(&mut fm.bt, &g_bt, lr_t);
        a_bo.step(&mut fm.bo, &g_bo, lr_t);

        // Keep the mastered weights inside a range whose ternary scales stay within the serving
        // bound, and embeddings inside a range their quantiser can express.
        for w in fm
            .wi
            .iter_mut()
            .chain(fm.wt.iter_mut())
            .chain(fm.wo.iter_mut())
        {
            *w = w.clamp(-120.0, 120.0);
        }
        for w in fm.e.iter_mut() {
            *w = w.clamp(-4.0 * Q8_HI, 4.0 * Q8_HI);
        }
    }

    // ---- export the quantised integer artifact ----
    let e_q_i8: Vec<i8> = fm.e.iter().map(|v| q8(*v) as i8).collect();
    let wi = TernaryLinear::quantize(&fm.wi, fm.h_dim, wi_cols);
    let wt = TernaryLinear::quantize(&fm.wt, fm.h_dim, wt_cols);
    let wo = TernaryLinear::quantize(&fm.wo, fm.n_actions, wo_cols);
    let model = StateLexicalModel {
        version: SL_VERSION,
        vocab,
        slots,
        content_tokens: content_tokens.clone(),
        h_dim: fm.h_dim,
        e_dim: fm.e_dim,
        f_dim: fm.f_dim,
        c_dim: fm.c_dim,
        h_clamp: fm.h_clamp as i32,
        m_clamp: fm.m_clamp as i32,
        e: e_q_i8,
        wi: SlLinear::from_ternary(&wi),
        wt: SlLinear::from_ternary(&wt),
        wo: SlLinear::from_ternary(&wo),
        bi: fm.bi.iter().map(|v| v.round() as i32).collect(),
        bt: fm.bt.iter().map(|v| v.round() as i32).collect(),
        bo: fm.bo.iter().map(|v| v.round() as i32).collect(),
        max_insert_words,
    };
    model.validate(vocab)?;

    let (correct, total) = model.teacher_forced_agreement(examples);
    let served_loss = served_cross_entropy(&model, examples, cfg.stop_weight, cfg.insert_weight);

    Ok(SlFitOutcome {
        model,
        report: SlFitReport {
            sequences: examples.len(),
            steps: total_steps,
            epochs: cfg.epochs,
            first_loss,
            final_loss,
            served_loss,
            action_correct: correct,
            action_total: total,
            h_dim: cfg.h_dim,
            e_dim: cfg.e_dim,
            content_tokens: content_len,
        },
    })
}

/// Bound one gradient tensor by absolute value before an Adam step.
#[allow(dead_code)]
fn clip(g: &[f32]) -> Vec<f32> {
    const LIMIT: f32 = 8.0;
    g.iter().map(|v| v.clamp(-LIMIT, LIMIT)).collect()
}

fn clamp_in_place_f(v: &mut [f32], bound: f32) {
    for x in v.iter_mut() {
        *x = x.clamp(-bound, bound);
    }
}

/// The served cross-entropy, computed over the exported integer logits so a reported number belongs
/// to the artifact that is actually loaded.
fn served_cross_entropy(
    model: &StateLexicalModel,
    examples: &[SlSequence],
    stop_weight: f32,
    insert_weight: f32,
) -> f32 {
    let mut loss = 0f32;
    let mut weight_sum = 0f32;
    for ex in examples {
        let m = model.content_feature(&ex.sel, &ex.res);
        let f = model.typed_block(&ex.sel, &ex.res, ex.facts());
        let mut h = model.init_state(&m, &f);
        let stages = ex.copy_stages();
        for (i, action) in ex.actions.iter().enumerate() {
            let logits = model.readout(&h, &m, &f, stages[i]);
            let max = logits.iter().copied().max().unwrap_or(0);
            let mut sum = 0.0f64;
            for l in &logits {
                sum += (f64::from(*l) - f64::from(max)).exp();
            }
            let y = action_index(*action);
            let nll = f64::from(max) - f64::from(logits[y]) + sum.ln();
            let w = action_weight(*action, stop_weight, insert_weight);
            loss += w * nll as f32;
            weight_sum += w;
            h = model.transition(
                &h,
                StateLexicalModel::symbol_of(*action),
                ex.emitted_token(i, &model.slots),
                &f,
            );
        }
    }
    loss / weight_sum.max(1e-9)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy(n: usize) -> Vec<RealizationAction> {
        vec![RealizationAction::Copy; n]
    }

    /// A tiny declared language. Slot 0 opens, slot 1 is the present copula, slot 2 the past copula,
    /// slot 3 the unchanged-connective, slot 4 the changed-connective and slot 5 a trailing word that
    /// follows the copied span. The last three are selected by the *content* and the emitted stream
    /// with every provenance flag held equal.
    fn slots() -> Vec<u32> {
        vec![101, 102, 103, 104, 105, 106]
    }

    fn direct(value: &[u32]) -> SlSequence {
        let mut a = vec![RealizationAction::Insert(0), RealizationAction::Insert(1)];
        a.extend(copy(value.len()));
        a.push(RealizationAction::Stop);
        SlSequence {
            sel: value.to_vec(),
            res: vec![],
            history: 0,
            derived: false,
            prior_differs: false,
            committed: false,
            key_changed: false,
            actions: a,
        }
    }

    fn previous(value: &[u32]) -> SlSequence {
        let mut a = vec![RealizationAction::Insert(0), RealizationAction::Insert(2)];
        a.extend(copy(value.len()));
        a.push(RealizationAction::Stop);
        SlSequence {
            sel: value.to_vec(),
            res: vec![],
            history: 1,
            derived: false,
            prior_differs: true,
            committed: true,
            key_changed: false,
            actions: a,
        }
    }

    /// A consumed computation: `operand` is where the computation started, `sel` is the result it
    /// consumed. When `operand == sel` the value is unchanged and the derived key resolved back to
    /// the operand's own address. A committed change is additionally *continued after the copied
    /// span* with a trailing marker, so the learner exercises a vocabulary decision that follows the
    /// owned tokens and is conditioned on the computation's actual effect.
    fn derived(operand: &[u32], value: &[u32], unchanged: bool) -> SlSequence {
        let word = if unchanged { 3 } else { 4 };
        let mut a = vec![
            RealizationAction::Insert(0),
            RealizationAction::Insert(word),
        ];
        a.extend(copy(value.len()));
        if !unchanged {
            a.push(RealizationAction::Insert(5));
        }
        a.push(RealizationAction::Stop);
        SlSequence {
            sel: value.to_vec(),
            res: operand.to_vec(),
            history: 0,
            derived: true,
            prior_differs: false,
            committed: false,
            key_changed: !unchanged,
            actions: a,
        }
    }

    fn examples() -> Vec<SlSequence> {
        let values: [&[u32]; 3] = [&[11, 12], &[21], &[31, 32, 33]];
        let mut out = Vec::new();
        for v in values {
            out.push(direct(v));
            out.push(previous(v));
            out.push(derived(v, v, true));
        }
        // A changed consumed value: the operand differs from the result it consumed, so the answer is
        // continued after the copied span.
        out.push(derived(&[41, 42], &[11, 12], false));
        out.push(derived(&[43], &[21], false));
        out.push(derived(&[44, 45, 46], &[31, 32, 33], false));
        out
    }

    fn fit() -> (StateLexicalModel, SlFitReport) {
        let cfg = SlFitConfig {
            epochs: 8000,
            h_dim: 48,
            e_dim: 48,
            ..Default::default()
        };
        let out = fit_state_lexical(&examples(), 4096, slots(), 4, &cfg, &[]).expect("fit");
        (out.model, out.report)
    }

    /// Greedy served rollout with the session's structural invariants: `Copy` is forced inside the
    /// span and `Stop` is forced before it is complete.
    fn rollout(model: &StateLexicalModel, ex: &SlSequence) -> Vec<RealizationAction> {
        let m = model.content_feature(&ex.sel, &ex.res);
        let f = model.typed_block(&ex.sel, &ex.res, ex.facts());
        let mut h = model.init_state(&m, &f);
        let total = ex.copy_count();
        let mut copied = 0usize;
        let mut actions = Vec::new();
        for step in 0..32 {
            let stage = if copied == 0 {
                0
            } else if copied < total {
                1
            } else {
                2
            };
            let (mut action, _) = model.decide(&h, &m, &f, stage);
            if stage == 1 && !matches!(action, RealizationAction::Copy) {
                action = RealizationAction::Copy;
            } else if stage < 2 && matches!(action, RealizationAction::Stop) {
                action = RealizationAction::Copy;
            } else if stage == 0 && matches!(action, RealizationAction::Stop) {
                action = RealizationAction::Copy;
            }
            actions.push(action);
            match action {
                RealizationAction::Copy => copied += 1,
                RealizationAction::Stop => break,
                RealizationAction::Insert(_) => {}
            }
            if copied > total {
                break;
            }
            h = model.transition(
                &h,
                StateLexicalModel::symbol_of(action),
                ex.emitted_token(step, &model.slots),
                &f,
            );
        }
        actions
    }

    fn arithmetic_fixture() -> StateLexicalModel {
        let map = |rows, cols| {
            SlLinear::from_ternary(&TernaryLinear::quantize(
                &vec![1.0; rows * cols],
                rows,
                cols,
            ))
        };
        StateLexicalModel {
            version: SL_VERSION,
            vocab: 4096,
            slots: vec![101],
            // Index 0 is the reserved OOV row; index 1 is token 11.
            content_tokens: vec![0, 11],
            h_dim: 1,
            e_dim: 2,
            f_dim: SL_F_DIM,
            c_dim: 3,
            h_clamp: 255,
            m_clamp: 255,
            e: vec![1; 2 * SL_CONTENT_POSITIONS],
            wi: map(1, 2 + 1 + SL_F_DIM),
            wt: map(1, 1 + SL_SYM_INSERT_BASE + 1 + 1 + SL_F_DIM),
            wo: map(SL_INSERT_BASE + 1, 1 + (2 + 1 + SL_F_DIM) + 3),
            bi: vec![0],
            bt: vec![0],
            bo: vec![0; SL_INSERT_BASE + 1],
            max_insert_words: 4,
        }
    }

    #[test]
    fn artifact_load_rejects_overflowing_dimensions_biases_and_recurrent_ranges() {
        let model = arithmetic_fixture();
        model.validate(4096).unwrap();
        for kind in 0..4 {
            let mut broken = model.clone();
            match kind {
                0 => broken.e_dim = usize::MAX - 1,
                1 => broken.h_dim = usize::MAX,
                2 => broken.bi[0] = i32::MAX,
                _ => broken.h_clamp = i32::MAX,
            }
            let bytes = broken.to_bytes().unwrap();
            assert!(
                StateLexicalModel::from_bytes(&bytes, 4096).is_err(),
                "case {kind}"
            );
        }
    }

    #[test]
    fn quantized_training_affine_matches_integer_export_with_fractional_biases() {
        let weights = vec![0.8, -1.1, 0.1, -1.4, 0.2, 0.9];
        let biases = vec![0.6, -0.6];
        let input = vec![2, -3, 1];
        let trained = quantized_affine(
            &q_ternary(&weights, 2, 3),
            &input.iter().map(|x| *x as f32).collect::<Vec<_>>(),
            &biases,
        );
        let mut served = TernaryLinear::quantize(&weights, 2, 3).forward_i32(&input);
        for (value, bias) in served.iter_mut().zip(&biases) {
            *value += bias.round() as i32;
        }
        assert_eq!(
            trained,
            served.iter().map(|x| *x as f32).collect::<Vec<_>>()
        );
        // Raw fractional biases would differ here, which was the previous fit/export mismatch.
        assert!(trained.iter().all(|x| x.fract() == 0.0));
    }

    #[test]
    fn weighted_objective_gradient_matches_finite_difference_and_example_permutation() {
        let examples = [(1.0f32, 0usize, 1.0f32), (-2.0, 1, 7.0)];
        let total_weight: f32 = examples.iter().map(|x| x.2).sum();
        let gradient = |theta: f32, reversed: bool| {
            let mut sum = 0.0;
            let order = if reversed { [1, 0] } else { [0, 1] };
            for i in order {
                let (x, y, weight) = examples[i];
                let p0 = 1.0 / (1.0 + (-theta * x).exp());
                sum += weighted_target_gradient(&[p0, 1.0 - p0], y, weight, total_weight)[0] * x;
            }
            sum
        };
        let loss = |theta: f32| {
            examples
                .iter()
                .map(|(x, y, weight)| {
                    let p0 = 1.0 / (1.0 + (-theta * x).exp());
                    -weight * if *y == 0 { p0.ln() } else { (1.0 - p0).ln() }
                })
                .sum::<f32>()
                / total_weight
        };
        let theta = 0.3;
        let finite = (loss(theta + 0.001) - loss(theta - 0.001)) / 0.002;
        assert!((gradient(theta, false) - finite).abs() < 0.0001);
        assert_eq!(gradient(theta, false), gradient(theta, true));
    }

    #[test]
    fn served_action_loss_keeps_large_finite_errors_without_probability_floor() {
        let mut model = arithmetic_fixture();
        for map in [&mut model.wi, &mut model.wt, &mut model.wo] {
            map.packed.fill(0);
        }
        model.bo = vec![i32::MAX, 0, -i32::MAX];
        model.validate(4096).unwrap();
        let example = SlSequence {
            sel: Vec::new(),
            res: Vec::new(),
            history: 0,
            derived: false,
            prior_differs: false,
            committed: false,
            key_changed: false,
            actions: vec![RealizationAction::Stop],
        };
        let loss = served_cross_entropy(&model, &[example], 1.0, 1.0);
        assert!(
            loss.is_finite() && loss > 1.0e9,
            "clipped or invalid loss: {loss}"
        );
    }

    #[test]
    fn malformed_fit_inputs_return_errors_before_entering_the_optimizer() {
        let valid = direct(&[11]);
        let cfg = SlFitConfig {
            epochs: 1,
            h_dim: 1,
            e_dim: 2,
            ..Default::default()
        };
        let mut no_stop = valid.clone();
        no_stop.actions.pop();
        let mut bad_slot = valid.clone();
        bad_slot.actions[0] = RealizationAction::Insert(255);
        for ex in [no_stop, bad_slot] {
            assert!(fit_state_lexical(&[ex], 4096, slots(), 4, &cfg, &[]).is_err());
        }
        let invalid_cfg = SlFitConfig {
            lr: f32::NAN,
            ..cfg
        };
        assert!(fit_state_lexical(&[valid], 4096, slots(), 4, &invalid_cfg, &[]).is_err());
    }

    #[test]
    fn the_fitted_decoder_reproduces_every_declared_sequence_through_the_served_path() {
        let (model, report) = fit();
        if report.action_correct != report.action_total {
            let (m_, f_) = (&model, report);
            for ex in examples() {
                let got = rollout(m_, &ex);
                if got != ex.actions {
                    eprintln!(
                        "MISMATCH sel={:?} res={:?} hist={} facts={:?} got={:?} want={:?}",
                        ex.sel,
                        ex.res,
                        ex.history,
                        ex.facts(),
                        got,
                        ex.actions
                    );
                }
            }
            eprintln!("report {}/{}", f_.action_correct, f_.action_total);
        }
        assert_eq!(
            report.action_correct, report.action_total,
            "served teacher-forced accuracy {}/{}",
            report.action_correct, report.action_total
        );
        for ex in examples() {
            assert_eq!(
                rollout(&model, &ex),
                ex.actions,
                "served rollout disagrees with the declared text for {:?}",
                ex
            );
        }
    }

    /// A declared witness that the recurrent transition consumes *both* the action symbol and the
    /// emitted token identity. The learned evidence that this capacity is used is
    /// `copied_token_identity_enters_the_recurrence`; this test pins the representation itself.
    fn transition_witness() -> StateLexicalModel {
        let row = |values: &[f32], cols: usize| {
            SlLinear::from_ternary(&TernaryLinear::quantize(values, 1, cols))
        };
        let ones = |rows: usize, cols: usize| {
            SlLinear::from_ternary(&TernaryLinear::quantize(
                &vec![1.0; rows * cols],
                rows,
                cols,
            ))
        };
        let half = 1usize;
        let cols = 3usize; // content rows: OOV, token 11, token 12
        let mut e = vec![0i8; cols * SL_CONTENT_POSITIONS * half];
        e[1 * SL_CONTENT_POSITIONS * half] = 2; // token 11, position 0
        e[2 * SL_CONTENT_POSITIONS * half] = -2; // token 12, position 0
        StateLexicalModel {
            version: SL_VERSION,
            vocab: 4096,
            slots: vec![101, 102],
            content_tokens: vec![0, 11, 12],
            h_dim: 1,
            e_dim: 2,
            f_dim: SL_F_DIM,
            c_dim: 3,
            h_clamp: 255,
            m_clamp: 255,
            e,
            wi: ones(1, 2 + half + SL_F_DIM),
            // Columns: h, start, copy, insert-slot-0, insert-slot-1, emitted-token identity, typed block.
            wt: {
                let mut w = vec![0.0f32; 6];
                w[2] = 1.0;
                w[3] = -1.0;
                w[5] = 1.0;
                w.extend(std::iter::repeat(0.0).take(SL_F_DIM));
                row(&w, 6 + SL_F_DIM)
            },
            wo: ones(SL_INSERT_BASE + 2, 1 + (2 + half + SL_F_DIM) + 3),
            bi: vec![0],
            bt: vec![0],
            bo: vec![0; SL_INSERT_BASE + 2],
            max_insert_words: 4,
        }
    }

    #[test]
    fn equal_length_prefixes_over_different_symbols_reach_different_states() {
        // The old clipped-count key aliased equal-length prefixes. The transition consumes the actual
        // action symbol and the emitted token identity, so both must be able to change the next state.
        let model = transition_witness();
        model.validate(4096).unwrap();
        let f = model.typed_block(&[], &[], SlFacts::default());
        let m = model.content_feature(&[], &[]);
        let h0 = model.init_state(&m, &f);
        let slot0 = model.transition(&h0, SL_SYM_INSERT_BASE, Some(11), &f);
        let slot1 = model.transition(&h0, SL_SYM_INSERT_BASE + 1, Some(11), &f);
        assert_ne!(
            slot0, slot1,
            "the emitted action symbol must enter the state"
        );
        let tok11 = model.transition(&h0, SL_SYM_INSERT_BASE, Some(11), &f);
        let tok12 = model.transition(&h0, SL_SYM_INSERT_BASE, Some(12), &f);
        assert_ne!(
            tok11, tok12,
            "the emitted token identity must enter the state"
        );
    }

    #[test]
    fn copied_token_identity_enters_the_recurrence() {
        // Two equal-length copied spans whose tokens are distinct fitted values. Version 2 collapsed
        // both to one Copy symbol; version 3 feeds the emitted token's shared learned identity, so the
        // states after the same number of copies must differ.
        let (model, _) = fit();
        let f = model.typed_block(&[], &[], SlFacts::default());
        let m = model.content_feature(&[], &[]);
        let h0 = model.init_state(&m, &f);
        let a = model.transition(&h0, SL_SYM_COPY, Some(11), &f);
        let b = model.transition(&h0, SL_SYM_COPY, Some(21), &f);
        assert_ne!(
            a, b,
            "a copied token's identity must advance the recurrence differently"
        );
    }

    #[test]
    fn the_content_feature_is_ordered_suffix_aware_and_distinguishes_unknowns() {
        let (model, _) = fit();
        // Reordered payloads over the same tokens must not share a feature.
        assert_ne!(
            model.content_feature(&[11, 12], &[]),
            model.content_feature(&[12, 11], &[])
        );
        // A suffix beyond the truncation window survives in the tail position.
        assert_ne!(
            model.content_feature(&[11, 12, 21, 31, 41], &[]),
            model.content_feature(&[11, 12, 21, 31, 99], &[])
        );
        // A repeated unknown is exactly equal to itself but not to a different unknown, and the typed
        // block records the exact relation even where the content rows collide.
        let known = |t: u32| model.embed_row(t).is_some();
        let same = typed_causal_block(&[900, 901], &[900, 901], SlFacts::default(), &known);
        let diff = typed_causal_block(&[900, 901], &[900, 902], SlFacts::default(), &known);
        assert_ne!(
            same, diff,
            "exact unknown identity relation must be observable"
        );
        assert_eq!(same[8], 1, "exact equality is reported");
        assert_eq!(diff[8], 0);
    }

    #[test]
    fn authored_derived_key_cases_select_different_uncopied_words() {
        // Both probes are derived, current-history, prior_differs = false. Their
        // content AND key_changed differ, so this checks fitted case behavior,
        // not an isolated content effect.
        let (model, _) = fit();
        let unchanged = derived(&[11, 12], &[11, 12], true);
        let changed = derived(&[41, 42], &[11, 12], false);
        assert_eq!(rollout(&model, &unchanged), unchanged.actions);
        assert_eq!(rollout(&model, &changed), changed.actions);
        assert_ne!(
            rollout(&model, &unchanged),
            rollout(&model, &changed),
            "the authored derived-key cases must choose different words"
        );
    }

    #[test]
    fn the_artifact_round_trips_and_rejects_a_corrupt_bound() {
        let (model, _) = fit();
        let bytes = model.to_bytes().expect("serialize");
        let reloaded = StateLexicalModel::from_bytes(&bytes, 4096).expect("reload");
        assert_eq!(model, reloaded);
        let mut corrupt = model.clone();
        corrupt.version = SL_VERSION + 1;
        assert!(corrupt.validate(4096).is_err());
        let mut wide = model.clone();
        wide.wi.shift[0] = 99;
        assert!(wide.validate(4096).is_err());
        let mut huge = model.clone();
        huge.e[0] = 100;
        assert!(huge.validate(4096).is_err());
    }

    #[test]
    fn every_served_weight_is_ternary_and_every_embedding_is_four_bit() {
        let (model, _) = fit();
        for map in [&model.wi, &model.wt, &model.wo] {
            let t = map.to_ternary().expect("ternary");
            for r in 0..t.rows {
                for c in 0..t.cols {
                    let w = t.weight(r, c);
                    assert!(w == -1 || w == 0 || w == 1, "non-ternary weight {w}");
                }
                assert!(t.shift(r) <= SL_MAX_SHIFT);
            }
        }
        for v in model.e.iter() {
            assert!((*v as i32).abs() <= SL_EMB_BOUND);
        }
    }
}
