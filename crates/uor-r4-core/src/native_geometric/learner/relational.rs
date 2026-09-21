//! Learned contextual geometric relational reader over exact token occurrences.
//!
//! ```text
//! q_t  = Q(x_{i-1})               learned query descriptor (ordered role root)
//! k_j  = Q(x_{j-1})               learned source descriptor (same learned map)
//! r_tj = inverse(q_t) * k_j       directed relative element, common-left invariant
//! s_tj = bias + rank[r_tj] + sum_e w_e*phi_e(j) + sb[a]
//! a_t  = NoRead or (exact occurrence reference j, bounded strength A[a])
//! ```
//!
//! Selection and bounded copy strength are learned against the **actual one-step action loss** of
//! the observed next token, `delta(a) = log(1 + p*(exp(a)-1)) - a*1[payload == target]`, so no
//! reward model and no reinforcement learning are needed. The descriptor map `Q` is learned by a
//! bounded discrete coordinate search on that same objective. Exact token/occurrence/payload
//! identity stays outside the descriptor and is read through a validated `(seq, abs)` reference.
#![forbid(unsafe_code)]

use super::group_table::{GROUP_ORDER, ROW_STRIDE};
use super::occurrence::{OccurrenceRef, OccurrenceRing};
use super::prefix_artifact::ExactGroupTable;

/// Number of group states.
pub const RANKS: usize = GROUP_ORDER;
/// Number of exact-evidence indicators.
pub const EXACT_FEATS: usize = 6;
/// Number of nonzero copy strengths. Strength zero is NoRead.
pub const ACTS: usize = 3;
/// Number of causal utility buckets for the action-dependent contextual interaction.
pub const CTX_BUCKETS: usize = 16;
/// Declared strength shifts (bits); the boost in nats is `2^(shift - f_bits)`.
pub const AMP_SHIFTS: [u32; 3] = [6, 10, 13];
/// Largest magnitude of a served coefficient (signed 4-bit).
pub const WEIGHT_MAX: i32 = 7;
/// Absent-token marker.
pub const NO_TOKEN: u32 = u32::MAX;

/// Admission counters, reported separately from ranking.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolStats {
    pub scanned: usize,
    pub matches: usize,
    pub admitted: usize,
    pub dropped_by_bound: usize,
    pub no_observed_successor: usize,
}

/// One admitted candidate of the shared causal pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cand {
    pub slot_ref: OccurrenceRef,
    pub abs: u32,
    pub payload: u32,
    pub x_cur: u32,
    pub x_prev: u32,
    pub x_prev2: u32,
    pub feats: [u8; EXACT_FEATS],
}

/// Exact-evidence indicators, supplied to every comparator so widening the pool blinds nobody.
#[allow(clippy::too_many_arguments)]
pub fn exact_features(
    cur: u32,
    c_prev: u32,
    c_prev2: u32,
    q_cur: u32,
    q_prev: u32,
    q_prev2: u32,
    payload: u32,
    i: u32,
    abs: u32,
    is_newest: bool,
) -> [u8; EXACT_FEATS] {
    let mut f = [0u8; EXACT_FEATS];
    f[0] = u8::from(cur == q_cur);
    f[1] = u8::from(c_prev != NO_TOKEN && q_prev != NO_TOKEN && c_prev == q_prev);
    f[2] = u8::from(c_prev2 != NO_TOKEN && q_prev2 != NO_TOKEN && c_prev2 == q_prev2);
    f[3] = u8::from(payload == q_cur);
    f[4] = u8::from(i >= 2 && abs + 2 == i);
    f[5] = u8::from(is_newest);
    f
}

/// The shared causal pool: admit a candidate sharing **any** exact context token with the query.
/// Never inspects the target.
pub fn admit_mixed(
    ring: &OccurrenceRing,
    i: u32,
    q_cur: u32,
    q_prev: u32,
    q_prev2: u32,
    max_candidates: usize,
) -> (Vec<Cand>, PoolStats) {
    let mut out: Vec<Cand> = Vec::new();
    let mut st = PoolStats::default();
    let written = ring.written();
    if written == 0 {
        return (out, st);
    }
    let low = ring.valid_from();
    let mut abs = written - 1;
    loop {
        if abs < low {
            break;
        }
        st.scanned += 1;
        if let Some(cur) = ring.get(abs) {
            let c_prev = if abs >= 1 {
                ring.get(abs - 1).unwrap_or(NO_TOKEN)
            } else {
                NO_TOKEN
            };
            let c_prev2 = if abs >= 2 {
                ring.get(abs - 2).unwrap_or(NO_TOKEN)
            } else {
                NO_TOKEN
            };
            let share = cur == q_cur
                || (c_prev != NO_TOKEN && c_prev == q_prev)
                || (c_prev2 != NO_TOKEN && c_prev2 == q_prev2);
            if share {
                st.matches += 1;
                if abs + 1 >= written {
                    st.no_observed_successor += 1;
                } else if out.len() < max_candidates {
                    let payload = ring.get(abs + 1).expect("inside the prefix");
                    let is_newest = out.is_empty();
                    out.push(Cand {
                        slot_ref: OccurrenceRef {
                            seq: ring.seq(),
                            abs,
                        },
                        abs,
                        payload,
                        x_cur: cur,
                        x_prev: c_prev,
                        x_prev2: c_prev2,
                        feats: exact_features(
                            cur, c_prev, c_prev2, q_cur, q_prev, q_prev2, payload, i, abs,
                            is_newest,
                        ),
                    });
                    st.admitted += 1;
                } else {
                    st.dropped_by_bound += 1;
                }
            }
        }
        if abs == 0 {
            break;
        }
        abs -= 1;
    }
    (out, st)
}

/// `inverse(a) * b` in the bound exact table.
#[inline]
pub fn relation(t: &ExactGroupTable, a: usize, b: usize) -> usize {
    let ia = t.inverse[a] as usize;
    t.product[ia * ROW_STRIDE + b] as usize
}

/// **The one relation-index function.** Training, hard-forward inference, export and reload all call
/// this with the same mode and code map, so no arm can be fitted at one address and read at another.
#[inline]
pub fn relation_index(
    t: &ExactGroupTable,
    mode: RelMode,
    code_of: &[u8],
    q_root: u8,
    k_root: u8,
    q_tok: usize,
    k_tok: usize,
) -> usize {
    match mode {
        RelMode::ExactOnly => 0,
        RelMode::Geometric => relation(t, q_root as usize, k_root as usize),
        RelMode::Categorical => {
            let a = *code_of.get(q_tok).unwrap_or(&0) as usize;
            let b = *code_of.get(k_tok).unwrap_or(&0) as usize;
            (a * 7 + b) % RANKS
        }
    }
}

/// How the relation index is formed. The three modes are the matched comparison arms.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RelMode {
    /// The directed relative group element `inverse(q)*k` over the learned descriptor map.
    Geometric,
    /// A learned table on an arbitrary categorical code that is unrelated to any geometry. Used
    /// as the matched nongeometric contextual control.
    Categorical,
    /// No relation term at all: the repaired exact-recurrence comparator.
    ExactOnly,
}

/// Receipt for one bounded contextual-interaction fit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CtxFit {
    pub objective_before: f64,
    pub objective_after: f64,
    pub evaluations: u64,
    pub entries_moved: usize,
}

/// The served integer selector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalSelector {
    /// Learned per-token descriptor root.
    pub q_roots: Vec<u8>,
    /// The arm's relation encoding. Exported and reloaded, never reconstructed by the caller.
    pub mode: RelMode,
    /// The arm's per-token code map (learned for the categorical arm).
    pub code_of: Vec<u8>,
    pub w: [i32; EXACT_FEATS],
    pub rank: [i32; RANKS],
    pub bias: i32,
    pub sb: [i32; ACTS],
    pub noread: i32,
    /// The small causal action-dependent interaction. Empty disables it (the factorised baseline).
    /// Otherwise `CTX_BUCKETS` rows of `[noread_offset, strength_0, strength_1, ...]`.
    pub ctx: Vec<[i32; ACTS + 1]>,
}

impl RelationalSelector {
    /// The matched exact-recurrence comparator: identical features, actions and dose, no relation
    /// term and no learned descriptor.
    pub fn without_relation(&self) -> Self {
        let mut s = self.clone();
        s.rank = [0; RANKS];
        s
    }

    /// Relation index of one candidate under this selector's own declared encoding.
    #[inline]
    pub fn rel_of_tokens(&self, t: &ExactGroupTable, p: &TrainPos, k: usize) -> usize {
        let qr = p.q_role;
        let kr = p.k_role[k];
        if qr == usize::MAX || kr == usize::MAX {
            return 0;
        }
        relation_index(
            t,
            self.mode,
            &self.code_of,
            *self.q_roots.get(qr).unwrap_or(&0),
            *self.q_roots.get(kr).unwrap_or(&0),
            qr,
            kr,
        )
    }

    /// Relation indices for a whole position.
    pub fn rels_of(&self, t: &ExactGroupTable, p: &TrainPos) -> Vec<usize> {
        (0..p.k_role.len())
            .map(|k| self.rel_of_tokens(t, p, k))
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        for (k, v) in self.w.iter().enumerate() {
            if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(v) {
                return Err(format!("exact weight {k} = {v} outside +/-{WEIGHT_MAX}"));
            }
        }
        for (k, v) in self.rank.iter().enumerate() {
            if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(v) {
                return Err(format!("rank {k} = {v} outside +/-{WEIGHT_MAX}"));
            }
        }
        for (k, v) in self.sb.iter().enumerate() {
            if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(v) {
                return Err(format!("strength bias {k} = {v} outside +/-{WEIGHT_MAX}"));
            }
        }
        if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(&self.bias)
            || !(-WEIGHT_MAX..=WEIGHT_MAX).contains(&self.noread)
        {
            return Err("selector bias outside the declared bound".into());
        }
        if self.q_roots.is_empty() {
            return Err("empty descriptor".into());
        }
        if !self.ctx.is_empty() {
            if self.ctx.len() != CTX_BUCKETS {
                return Err("contextual interaction table has the wrong bucket count".into());
            }
            for (b, row) in self.ctx.iter().enumerate() {
                for (j, v) in row.iter().enumerate() {
                    if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(v) {
                        return Err(format!(
                            "contextual entry ({b},{j}) = {v} outside +/-{WEIGHT_MAX}"
                        ));
                    }
                }
            }
        }
        if self.q_roots.iter().any(|&r| r as usize >= RANKS) {
            return Err("descriptor root outside the group domain".into());
        }
        match self.mode {
            RelMode::Categorical if self.code_of.len() != self.q_roots.len() => {
                return Err("categorical code map must match the descriptor vocabulary".into());
            }
            RelMode::Categorical if self.code_of.iter().any(|&c| c as usize >= RANKS) => {
                return Err("categorical code outside the declared domain".into());
            }
            RelMode::Categorical => {}
            _ if !self.code_of.is_empty() => {
                return Err("a non-categorical selector must not carry a code map".into());
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate the descriptor against the vocabulary of the pinned local model.
    pub fn validate_for_vocab(&self, vocab: usize) -> Result<(), String> {
        self.validate()?;
        if self.q_roots.len() != vocab {
            return Err("selector vocabulary differs from the local model".into());
        }
        Ok(())
    }

    #[inline]
    pub fn score(&self, c: &Cand, rel: usize, a: usize) -> i32 {
        let mut s = self.bias + self.rank[rel] + self.sb[a];
        for k in 0..EXACT_FEATS {
            if c.feats[k] != 0 {
                s += self.w[k];
            }
        }
        s
    }

    /// The **ctx-free** source score: only the frozen scalars that define source ranking. `sb` and
    /// `ctx` are excluded so the bucket key cannot drift while `ctx` is fitted.
    #[inline]
    pub fn source_score(&self, c: &Cand, rel: usize) -> i32 {
        let mut s = self.bias + self.rank[rel];
        for k in 0..EXACT_FEATS {
            if c.feats[k] != 0 {
                s += self.w[k];
            }
        }
        s
    }

    /// The frozen causal position bucket: the exact-context class of the top-ranked admitted
    /// candidate plus one strict source-margin bit. Uses no target, coverage, label or future token.
    pub fn bucket_of(&self, cands: &[Cand], rel: &[usize]) -> usize {
        if cands.is_empty() {
            return 0;
        }
        let mut best = i32::MIN;
        let mut second = i32::MIN;
        let mut top = 0usize;
        for (k, c) in cands.iter().enumerate() {
            let s = self.source_score(c, rel.get(k).copied().unwrap_or(0));
            if s > best {
                second = best;
                best = s;
                top = k;
            } else if s > second {
                second = s;
            }
        }
        let c = &cands[top];
        (c.feats[3] as usize)
            | (((c.feats[1] | c.feats[2]) as usize) << 1)
            | ((c.feats[5] as usize) << 2)
            | (((best > second) as usize) << 3)
    }

    #[inline]
    fn ctx_at(&self, bucket: usize, slot: usize) -> i32 {
        if self.ctx.len() != CTX_BUCKETS {
            return 0;
        }
        self.ctx[bucket.min(CTX_BUCKETS - 1)][slot]
    }

    /// Score of the read action `a > 0` in the declared bucket, including the interaction.
    #[inline]
    pub fn strength_score(&self, c: &Cand, rel: usize, a: usize, bucket: usize) -> i32 {
        self.source_score(c, rel) + self.sb[a] + self.ctx_at(bucket, a + 1)
    }

    /// Score of the NoRead alternative in the declared bucket, including the interaction.
    #[inline]
    pub fn noread_score(&self, bucket: usize) -> i32 {
        self.noread + self.ctx_at(bucket, 0)
    }

    /// Causal buckets for a whole position list, computed once from this (frozen) selector.
    pub fn buckets_of(&self, t: &ExactGroupTable, positions: &[TrainPos]) -> Vec<usize> {
        positions
            .iter()
            .map(|p| self.bucket_of(&p.cands, &self.rels_of(t, p)))
            .collect()
    }

    /// Expected served one-step action loss at one position under the integer scores, given a bucket.
    /// This is the same softmax objective the trainer uses, evaluated on the served integers.
    pub fn position_loss_soft(&self, t: &ExactGroupTable, p: &TrainPos, bucket: usize) -> f64 {
        if p.cands.is_empty() {
            return 0.0;
        }
        let n = p.cands.len() * ACTS;
        let mut s = vec![0.0f64; n + 1];
        let mut d = vec![0.0f64; n + 1];
        for (k, c) in p.cands.iter().enumerate() {
            let rel = self.rel_of_tokens(t, p, k);
            for a in 0..ACTS {
                s[k * ACTS + a] = self.strength_score(c, rel, a, bucket) as f64;
                d[k * ACTS + a] = p.delta[k][a];
            }
        }
        s[n] = self.noread_score(bucket) as f64;
        d[n] = 0.0;
        let max = s.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let pi: Vec<f64> = s.iter().map(|x| (x - max).exp()).collect();
        let z: f64 = pi.iter().sum();
        pi.iter().zip(d.iter()).map(|(a, b)| a * b).sum::<f64>() / z
    }

    /// **Bounded discrete coordinate search over the contextual interaction only.**
    ///
    /// Everything else in the selector is frozen; the search starts from `ctx = 0`, which reproduces
    /// the factorised baseline exactly. Each entry is swept over the whole declared 4-bit range in
    /// place, so the result is an optimum for the served integer scores on the supplied positions.
    /// Returns the total objective change and the search receipt.
    pub fn ctx_fit(
        &mut self,
        t: &ExactGroupTable,
        positions: &[TrainPos],
        buckets: &[usize],
        rounds: usize,
    ) -> CtxFit {
        self.ctx = vec![[0i32; ACTS + 1]; CTX_BUCKETS];
        let mut by_bucket: Vec<Vec<usize>> = vec![Vec::new(); CTX_BUCKETS];
        for (i, b) in buckets.iter().enumerate() {
            if !positions[i].cands.is_empty() {
                by_bucket[(*b).min(CTX_BUCKETS - 1)].push(i);
            }
        }
        let total_of = |sel: &Self, idx: &[usize]| -> f64 {
            idx.iter()
                .map(|i| sel.position_loss_soft(t, &positions[*i], buckets[*i]))
                .sum()
        };
        let objective = |sel: &Self| -> f64 {
            positions
                .iter()
                .zip(buckets)
                .map(|(p, b)| sel.position_loss_soft(t, p, *b))
                .sum()
        };
        let before = objective(self);
        let mut evaluations = 0u64;
        let mut entries_moved = 0usize;
        for _ in 0..rounds {
            for b in 0..CTX_BUCKETS {
                if by_bucket[b].is_empty() {
                    continue;
                }
                for slot in 0..=ACTS {
                    let start = self.ctx[b][slot];
                    let mut best_val = start;
                    let mut best_loss = total_of(self, &by_bucket[b]);
                    for v in -WEIGHT_MAX..=WEIGHT_MAX {
                        if v == start {
                            continue;
                        }
                        self.ctx[b][slot] = v;
                        evaluations += 1;
                        let l = total_of(self, &by_bucket[b]);
                        if l < best_loss - 1e-12 {
                            best_loss = l;
                            best_val = v;
                        }
                    }
                    self.ctx[b][slot] = best_val;
                    if best_val != start {
                        entries_moved += 1;
                    }
                }
            }
        }
        CtxFit {
            objective_before: before,
            objective_after: objective(self),
            evaluations,
            entries_moved,
        }
    }

    /// `None` is NoRead; otherwise `(candidate index, strength index)`.
    pub fn choose(&self, cands: &[Cand], rel: &[usize]) -> Option<(usize, usize)> {
        let b = self.bucket_of(cands, rel);
        let nr = self.noread_score(b);
        let mut best: Option<(usize, usize, i32)> = None;
        for (k, c) in cands.iter().enumerate() {
            for a in 0..ACTS {
                let s = self.strength_score(c, rel.get(k).copied().unwrap_or(0), a, b);
                if best.is_none_or(|(_, _, pr)| s > pr) {
                    best = Some((k, a, s));
                }
            }
        }
        match best {
            Some((k, a, s)) if s > nr => Some((k, a)),
            _ => None,
        }
    }
}

/// One training position: constants only, independent of the parameters being fitted.
#[derive(Clone, Debug)]
pub struct TrainPos {
    pub cands: Vec<Cand>,
    /// Query-role token index (`x_{i-1}`), or `usize::MAX` when absent.
    pub q_role: usize,
    /// Source-role token index (`x_{j-1}`) per candidate, or `usize::MAX` when absent.
    pub k_role: Vec<usize>,
    /// Actual one-step action loss per `(candidate, strength)`.
    pub delta: Vec<[f64; ACTS]>,
    pub group: u16,
}

/// Exact one-step action loss in nats.
pub fn action_loss(p: f64, boost: f64, correct: bool) -> f64 {
    let a = boost.max(0.0);
    (1.0 + p * (a.exp() - 1.0)).ln() - if correct { a } else { 0.0 }
}

/// Offline reference optimum `a_opt = clip(logit(r) - logit(p), 0, A)`. Reported only.
pub fn optimal_boost(p: f64, r: f64, a_max: f64) -> f64 {
    let p = p.clamp(1e-12, 1.0 - 1e-12);
    let r = r.clamp(1e-12, 1.0 - 1e-12);
    ((r / (1.0 - r)).ln() - (p / (1.0 - p)).ln()).clamp(0.0, a_max)
}

/// The trainer: a learned descriptor map, the selector scalars, Adam, and a bounded discrete
/// descriptor search on the same objective.
#[derive(Clone, Debug)]
pub struct RelationalTrainer {
    pub vocab: usize,
    pub table: ExactGroupTable,
    /// Hard descriptor root per token (the learned contextual map).
    pub roots: Vec<u8>,
    pub w: [f64; EXACT_FEATS],
    pub rank: [f64; RANKS],
    pub bias: f64,
    pub sb: [f64; ACTS],
    pub noread: f64,
    m_w: [f64; EXACT_FEATS],
    v_w: [f64; EXACT_FEATS],
    m_r: [f64; RANKS],
    v_r: [f64; RANKS],
    m_sb: [f64; ACTS],
    v_sb: [f64; ACTS],
    m_b: f64,
    v_b: f64,
    m_n: f64,
    v_n: f64,
    pub step: u64,
    pub epoch: u32,
    pub cursor: usize,
    pub rng: u64,
    pub lr: f64,
    pub data_identity: [u8; 32],
    pub config_identity: [u8; 32],
    pub descriptor_moves: u64,
    pub descriptor_evaluations: u64,
    pub initial_roots: Vec<u8>,
    /// Initial categorical code map, for an active-map movement count.
    pub initial_codes: Vec<u8>,
    /// Comparison arm.
    pub mode: RelMode,
    /// Arbitrary per-token code for the categorical control.
    pub code_of: Vec<u8>,
}

impl RelationalTrainer {
    pub fn new(
        vocab: usize,
        table: ExactGroupTable,
        lr: f64,
        seed: u64,
        data_identity: [u8; 32],
        config_identity: [u8; 32],
        init_roots: &[u8],
    ) -> Self {
        assert_eq!(init_roots.len(), vocab);
        Self {
            vocab,
            table,
            roots: init_roots.to_vec(),
            w: [0.0; EXACT_FEATS],
            rank: [0.0; RANKS],
            bias: 0.0,
            sb: [0.0; ACTS],
            noread: 0.0,
            m_w: [0.0; EXACT_FEATS],
            v_w: [0.0; EXACT_FEATS],
            m_r: [0.0; RANKS],
            v_r: [0.0; RANKS],
            m_sb: [0.0; ACTS],
            v_sb: [0.0; ACTS],
            m_b: 0.0,
            v_b: 0.0,
            m_n: 0.0,
            v_n: 0.0,
            step: 0,
            epoch: 0,
            cursor: 0,
            rng: seed | 1,
            lr,
            data_identity,
            config_identity,
            descriptor_moves: 0,
            descriptor_evaluations: 0,
            initial_roots: init_roots.to_vec(),
            initial_codes: Vec::new(),
            mode: RelMode::Geometric,
            code_of: Vec::new(),
        }
    }

    /// Switch the comparison arm. `code_of` is required for `Categorical`.
    pub fn with_mode(mut self, mode: RelMode, code_of: Vec<u8>) -> Self {
        self.mode = mode;
        self.initial_codes = code_of.clone();
        self.code_of = code_of;
        self
    }

    /// Relation index for one candidate under the current arm.
    #[inline]
    pub fn rel_of(&self, p: &TrainPos, k: usize) -> usize {
        let qr = p.q_role;
        let kr = p.k_role[k];
        if qr == usize::MAX || kr == usize::MAX {
            return 0;
        }
        relation_index(
            &self.table,
            self.mode,
            &self.code_of,
            self.roots[qr],
            self.roots[kr],
            qr,
            kr,
        )
    }

    fn score(&self, c: &Cand, rel: usize, a: usize) -> f64 {
        let mut s = self.bias + self.rank[rel] + self.sb[a];
        for e in 0..EXACT_FEATS {
            if c.feats[e] != 0 {
                s += self.w[e];
            }
        }
        s
    }

    /// Expected actual action loss at one position under the current parameters.
    pub fn position_loss(&self, p: &TrainPos) -> f64 {
        if p.cands.is_empty() {
            return 0.0;
        }
        let n = p.cands.len() * ACTS;
        let mut s = vec![0.0f64; n + 1];
        let mut d = vec![0.0f64; n + 1];
        for (k, c) in p.cands.iter().enumerate() {
            let rel = self.rel_of(p, k);
            for a in 0..ACTS {
                s[k * ACTS + a] = self.score(c, rel, a);
                d[k * ACTS + a] = p.delta[k][a];
            }
        }
        s[n] = self.noread;
        d[n] = 0.0;
        let max = s.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let pi: Vec<f64> = s.iter().map(|x| (x - max).exp()).collect();
        let z: f64 = pi.iter().sum();
        pi.iter().zip(d.iter()).map(|(a, b)| a * b).sum::<f64>() / z
    }

    /// One Adam step on the selector scalars with the exact policy gradient
    /// `pi_i (delta_i - E[delta])`.
    pub fn step(&mut self, positions: &[TrainPos], batch: &[usize]) -> f64 {
        let mut g_w = [0.0f64; EXACT_FEATS];
        let mut g_r = [0.0f64; RANKS];
        let mut g_sb = [0.0f64; ACTS];
        let mut g_b = 0.0f64;
        let mut g_n = 0.0f64;
        let mut total = 0.0f64;
        let mut count = 0usize;
        for &pi_idx in batch {
            let p = &positions[pi_idx];
            if p.cands.is_empty() {
                continue;
            }
            let n = p.cands.len() * ACTS;
            let mut s = vec![0.0f64; n + 1];
            let mut d = vec![0.0f64; n + 1];
            for (k, c) in p.cands.iter().enumerate() {
                let rel = self.rel_of(p, k);
                for a in 0..ACTS {
                    s[k * ACTS + a] = self.score(c, rel, a);
                    d[k * ACTS + a] = p.delta[k][a];
                }
            }
            s[n] = self.noread;
            d[n] = 0.0;
            let max = s.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mut pi: Vec<f64> = s.iter().map(|x| (x - max).exp()).collect();
            let z: f64 = pi.iter().sum();
            for x in pi.iter_mut() {
                *x /= z;
            }
            let expected: f64 = pi.iter().zip(d.iter()).map(|(a, b)| a * b).sum();
            total += expected;
            count += 1;
            let gs: Vec<f64> = (0..=n).map(|i| pi[i] * (d[i] - expected)).collect();
            let mut g_cand = vec![0.0f64; p.cands.len()];
            for (k, c) in p.cands.iter().enumerate() {
                for a in 0..ACTS {
                    let g = gs[k * ACTS + a];
                    g_cand[k] += g;
                    g_sb[a] += g;
                }
                g_b += g_cand[k];
                for e in 0..EXACT_FEATS {
                    if c.feats[e] != 0 {
                        g_w[e] += g_cand[k];
                    }
                }
                if self.mode != RelMode::ExactOnly {
                    g_r[self.rel_of(p, k)] += g_cand[k];
                }
            }
            g_n += gs[n];
        }
        if count == 0 {
            return f64::NAN;
        }
        let inv = 1.0 / count as f64;
        for x in g_w.iter_mut() {
            *x *= inv;
        }
        for x in g_r.iter_mut() {
            *x *= inv;
        }
        for x in g_sb.iter_mut() {
            *x *= inv;
        }
        g_b *= inv;
        g_n *= inv;
        let age = self.step + 1;
        let (b1, b2, eps) = (0.9f64, 0.999f64, 1e-8f64);
        let bc1 = 1.0 - b1.powi(age.min(100_000) as i32);
        let bc2 = 1.0 - b2.powi(age.min(100_000) as i32);
        let adam = |m: &mut f64, v: &mut f64, g: f64| -> f64 {
            *m = b1 * *m + (1.0 - b1) * g;
            *v = b2 * *v + (1.0 - b2) * g * g;
            (*m / bc1) / ((*v / bc2).sqrt() + eps)
        };
        let mut upd = [0.0f64; EXACT_FEATS];
        for e in 0..EXACT_FEATS {
            upd[e] = adam(&mut self.m_w[e], &mut self.v_w[e], g_w[e]);
        }
        let mut upr = [0.0f64; RANKS];
        for r in 0..RANKS {
            upr[r] = adam(&mut self.m_r[r], &mut self.v_r[r], g_r[r]);
        }
        let mut ups = [0.0f64; ACTS];
        for a in 0..ACTS {
            ups[a] = adam(&mut self.m_sb[a], &mut self.v_sb[a], g_sb[a]);
        }
        let ub = adam(&mut self.m_b, &mut self.v_b, g_b);
        let un = adam(&mut self.m_n, &mut self.v_n, g_n);
        for e in 0..EXACT_FEATS {
            self.w[e] -= self.lr * upd[e];
        }
        for r in 0..RANKS {
            self.rank[r] -= self.lr * upr[r];
        }
        for a in 0..ACTS {
            self.sb[a] -= self.lr * ups[a];
        }
        self.bias -= self.lr * ub;
        self.noread -= self.lr * un;
        self.step = age;
        total / count as f64
    }

    /// Bounded discrete coordinate search on the active map (geometric roots or categorical
    /// codes), against the same objective and with the same search dose for both modes.
    ///
    /// Affected positions are **deduplicated** (a query-role and source-role occurrence of the same
    /// token otherwise counts the position twice), the search starts from the current assignment as
    /// its incumbent, ties are preserved, and only strictly improving moves beyond `tol` are
    /// accepted. The returned value is the change in the exact full position objective.
    pub fn refine_descriptor(&mut self, positions: &[TrainPos], rounds: usize) -> f64 {
        if self.mode == RelMode::ExactOnly {
            return 0.0;
        }
        let mut by_role: Vec<Vec<usize>> = vec![Vec::new(); self.vocab];
        for (i, p) in positions.iter().enumerate() {
            if p.q_role != usize::MAX {
                by_role[p.q_role].push(i);
            }
            for k in p.k_role.iter() {
                if *k != usize::MAX && *k < self.vocab {
                    by_role[*k].push(i);
                }
            }
        }
        for v in by_role.iter_mut() {
            v.sort_unstable();
            v.dedup();
        }
        let objective = |tr: &Self| -> f64 { positions.iter().map(|p| tr.position_loss(p)).sum() };
        let before = objective(self);
        let tol = 1e-9f64;
        for _ in 0..rounds {
            for t in 0..self.vocab {
                if by_role[t].is_empty() {
                    continue;
                }
                let cur = match self.mode {
                    RelMode::Categorical => self.code_of[t],
                    _ => self.roots[t],
                };
                let mut best_loss: f64 = by_role[t]
                    .iter()
                    .map(|i| self.position_loss(&positions[*i]))
                    .sum();
                let mut best_root = cur;
                for r in 0..RANKS {
                    if r == cur as usize {
                        continue;
                    }
                    match self.mode {
                        RelMode::Categorical => self.code_of[t] = r as u8,
                        _ => self.roots[t] = r as u8,
                    }
                    self.descriptor_evaluations += 1;
                    let l: f64 = by_role[t]
                        .iter()
                        .map(|i| self.position_loss(&positions[*i]))
                        .sum();
                    if l < best_loss - tol {
                        best_loss = l;
                        best_root = r as u8;
                    }
                }
                match self.mode {
                    RelMode::Categorical => self.code_of[t] = best_root,
                    _ => self.roots[t] = best_root,
                }
                if best_root != cur {
                    self.descriptor_moves += 1;
                }
            }
        }
        objective(self) - before
    }

    /// Quantize to the served signed ≤4-bit selector with one declared global scale.
    pub fn quantize(&self) -> RelationalSelector {
        let m = self
            .w
            .iter()
            .chain(self.rank.iter())
            .chain(self.sb.iter())
            .chain([self.bias, self.noread].iter())
            .fold(0.0f64, |a, b| a.max(b.abs()));
        let mut scale = 1.0f64;
        if m > 0.0 {
            while (m * scale * 2.0).round() <= WEIGHT_MAX as f64 {
                scale *= 2.0;
            }
        }
        let q = |x: f64| ((x * scale).round() as i32).clamp(-WEIGHT_MAX, WEIGHT_MAX);
        RelationalSelector {
            q_roots: self.roots.clone(),
            mode: self.mode,
            code_of: self.code_of.clone(),
            w: std::array::from_fn(|k| q(self.w[k])),
            rank: std::array::from_fn(|k| q(self.rank[k])),
            bias: q(self.bias),
            sb: std::array::from_fn(|k| q(self.sb[k])),
            noread: q(self.noread),
            ctx: Vec::new(),
        }
    }

    /// How many entries of the active map moved from the declared initialization.
    pub fn descriptor_moved(&self) -> usize {
        let (current, initial) = match self.mode {
            RelMode::Geometric => (&self.roots, &self.initial_roots),
            RelMode::Categorical => (&self.code_of, &self.initial_codes),
            RelMode::ExactOnly => return 0,
        };
        current
            .iter()
            .zip(initial.iter())
            .filter(|(a, b)| a != b)
            .count()
    }

    pub fn checkpoint_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLRK");
        o.extend_from_slice(&2u32.to_le_bytes());
        o.extend_from_slice(&(self.vocab as u32).to_le_bytes());
        o.push(match self.mode {
            RelMode::Geometric => 0,
            RelMode::Categorical => 1,
            RelMode::ExactOnly => 2,
        });
        o.extend_from_slice(&(self.code_of.len() as u32).to_le_bytes());
        o.extend_from_slice(&self.code_of);
        o.extend_from_slice(&self.initial_roots);
        o.extend_from_slice(&(self.initial_codes.len() as u32).to_le_bytes());
        o.extend_from_slice(&self.initial_codes);
        o.extend_from_slice(&self.roots);
        for x in self.w.iter() {
            o.extend_from_slice(&x.to_le_bytes());
        }
        for x in self.rank.iter() {
            o.extend_from_slice(&x.to_le_bytes());
        }
        for x in self.sb.iter() {
            o.extend_from_slice(&x.to_le_bytes());
        }
        for x in [self.bias, self.noread] {
            o.extend_from_slice(&x.to_le_bytes());
        }
        for x in self.m_w.iter().chain(self.v_w.iter()) {
            o.extend_from_slice(&x.to_le_bytes());
        }
        for x in self.m_r.iter().chain(self.v_r.iter()) {
            o.extend_from_slice(&x.to_le_bytes());
        }
        for x in self.m_sb.iter().chain(self.v_sb.iter()) {
            o.extend_from_slice(&x.to_le_bytes());
        }
        for x in [self.m_b, self.v_b, self.m_n, self.v_n] {
            o.extend_from_slice(&x.to_le_bytes());
        }
        o.extend_from_slice(&self.step.to_le_bytes());
        o.extend_from_slice(&(self.cursor as u64).to_le_bytes());
        o.extend_from_slice(&(self.epoch as u64).to_le_bytes());
        o.extend_from_slice(&self.rng.to_le_bytes());
        o.extend_from_slice(&self.lr.to_le_bytes());
        o.extend_from_slice(&self.descriptor_moves.to_le_bytes());
        o.extend_from_slice(&self.descriptor_evaluations.to_le_bytes());
        o.extend_from_slice(&self.data_identity);
        o.extend_from_slice(&self.config_identity);
        o
    }

    pub fn resume_from(&mut self, bytes: &[u8]) -> Result<(), String> {
        // Rejection must leave the existing trainer usable and unchanged.
        let mut restored = self.clone();
        restored.restore_checkpoint(bytes)?;
        *self = restored;
        Ok(())
    }

    fn restore_checkpoint(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated relational checkpoint".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLRK" {
            return Err("bad relational checkpoint magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        let u64_at = |c: &mut usize| -> Result<u64, String> {
            Ok(u64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
        };
        let f64_at = |c: &mut usize| -> Result<f64, String> {
            Ok(f64::from_le_bytes(take(c, 8)?.try_into().unwrap()))
        };
        let version = u32_at(&mut c)?;
        if version != 1 && version != 2 {
            return Err("unsupported relational checkpoint version".into());
        }
        if u32_at(&mut c)? as usize != self.vocab {
            return Err("relational checkpoint vocabulary differs".into());
        }
        if version == 1 {
            // Version 1 did not carry a mode or categorical map. Historical geometric
            // checkpoints remain readable; other modes cannot be recovered faithfully.
            if self.mode != RelMode::Geometric || !self.code_of.is_empty() {
                return Err("legacy relational checkpoints support geometric mode only".into());
            }
        } else {
            let mode = match take(&mut c, 1)?[0] {
                0 => RelMode::Geometric,
                1 => RelMode::Categorical,
                2 => RelMode::ExactOnly,
                _ => return Err("unknown relational checkpoint mode".into()),
            };
            if mode != self.mode {
                return Err("relational checkpoint mode differs".into());
            }
            let n_codes = u32_at(&mut c)? as usize;
            if n_codes
                != if mode == RelMode::Categorical {
                    self.vocab
                } else {
                    0
                }
            {
                return Err("relational checkpoint code vocabulary differs".into());
            }
            self.code_of = take(&mut c, n_codes)?.to_vec();
            self.initial_roots = take(&mut c, self.vocab)?.to_vec();
            let n_initial = u32_at(&mut c)? as usize;
            if n_initial != n_codes {
                return Err("relational checkpoint initial code vocabulary differs".into());
            }
            self.initial_codes = take(&mut c, n_initial)?.to_vec();
            if self
                .code_of
                .iter()
                .chain(self.initial_roots.iter())
                .chain(self.initial_codes.iter())
                .any(|&r| r as usize >= RANKS)
            {
                return Err("relational checkpoint map value outside the declared domain".into());
            }
        }
        self.roots.copy_from_slice(take(&mut c, self.vocab)?);
        if self.roots.iter().any(|&r| r as usize >= RANKS) {
            return Err("relational checkpoint root outside the group domain".into());
        }
        for e in 0..EXACT_FEATS {
            self.w[e] = f64_at(&mut c)?;
        }
        for r in 0..RANKS {
            self.rank[r] = f64_at(&mut c)?;
        }
        for a in 0..ACTS {
            self.sb[a] = f64_at(&mut c)?;
        }
        self.bias = f64_at(&mut c)?;
        self.noread = f64_at(&mut c)?;
        for e in 0..EXACT_FEATS {
            self.m_w[e] = f64_at(&mut c)?;
        }
        for e in 0..EXACT_FEATS {
            self.v_w[e] = f64_at(&mut c)?;
        }
        for r in 0..RANKS {
            self.m_r[r] = f64_at(&mut c)?;
        }
        for r in 0..RANKS {
            self.v_r[r] = f64_at(&mut c)?;
        }
        for a in 0..ACTS {
            self.m_sb[a] = f64_at(&mut c)?;
        }
        for a in 0..ACTS {
            self.v_sb[a] = f64_at(&mut c)?;
        }
        self.m_b = f64_at(&mut c)?;
        self.v_b = f64_at(&mut c)?;
        self.m_n = f64_at(&mut c)?;
        self.v_n = f64_at(&mut c)?;
        self.step = u64_at(&mut c)?;
        self.cursor = u64_at(&mut c)? as usize;
        self.epoch = u64_at(&mut c)? as u32;
        self.rng = u64_at(&mut c)?;
        let lr = f64_at(&mut c)?;
        let moves = u64_at(&mut c)?;
        let evals = u64_at(&mut c)?;
        let mut data_identity = [0u8; 32];
        data_identity.copy_from_slice(take(&mut c, 32)?);
        let mut config_identity = [0u8; 32];
        config_identity.copy_from_slice(take(&mut c, 32)?);
        if c != bytes.len() {
            return Err("trailing bytes in the relational checkpoint".into());
        }
        if data_identity != self.data_identity {
            return Err("relational checkpoint data identity differs".into());
        }
        if config_identity != self.config_identity {
            return Err("relational checkpoint configuration identity differs".into());
        }
        if lr != self.lr {
            return Err("relational checkpoint learning rate differs".into());
        }
        self.descriptor_moves = moves;
        self.descriptor_evaluations = evals;
        Ok(())
    }

    /// Deterministic example order for one epoch, reconstructed from `(rng, epoch)`.
    pub fn next_batch(&mut self, n: usize, batch: usize) -> Vec<usize> {
        let order = epoch_order(n, self.rng, self.epoch);
        let out: Vec<usize> = (0..batch.min(n))
            .map(|k| order[(self.cursor + k) % n])
            .collect();
        self.cursor += batch;
        if self.cursor >= n {
            self.cursor %= n;
            self.epoch += 1;
            self.rng = mix_rng(self.rng, self.epoch);
        }
        out
    }
}

/// A versioned selector artifact. The arm's relation **encoding** travels with it, so a reloaded
/// selector cannot be evaluated at a different address from the one it was trained at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalArtifact {
    pub selector: RelationalSelector,
    pub local_artifact_digest: [u8; 32],
    pub tokenizer_digest: [u8; 32],
    pub data_digest: [u8; 32],
}

impl RelationalArtifact {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLR2");
        o.extend_from_slice(&3u32.to_le_bytes());
        o.push(match self.selector.mode {
            RelMode::Geometric => 0,
            RelMode::Categorical => 1,
            RelMode::ExactOnly => 2,
        });
        o.extend_from_slice(&(self.selector.q_roots.len() as u32).to_le_bytes());
        o.extend_from_slice(&self.selector.q_roots);
        o.extend_from_slice(&(self.selector.code_of.len() as u32).to_le_bytes());
        o.extend_from_slice(&self.selector.code_of);
        for v in self.selector.w.iter() {
            o.push(*v as i8 as u8);
        }
        for v in self.selector.rank.iter() {
            o.push(*v as i8 as u8);
        }
        for v in self.selector.sb.iter() {
            o.push(*v as i8 as u8);
        }
        o.push(self.selector.bias as i8 as u8);
        o.push(self.selector.noread as i8 as u8);
        // Version 3 adds the bounded causal contextual interaction (possibly empty).
        o.extend_from_slice(&(self.selector.ctx.len() as u32).to_le_bytes());
        for row in self.selector.ctx.iter() {
            for v in row.iter() {
                o.push(*v as i8 as u8);
            }
        }
        o.extend_from_slice(&self.local_artifact_digest);
        o.extend_from_slice(&self.tokenizer_digest);
        o.extend_from_slice(&self.data_digest);
        o
    }

    /// Independent load. Rejects a wrong magic/version, an out-of-range coefficient, a mode that
    /// disagrees with the stored code map, and any identity mismatch.
    pub fn from_bytes(
        bytes: &[u8],
        expect_local: &[u8; 32],
        expect_tokenizer: &[u8; 32],
    ) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated relational artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLR2" {
            return Err("bad relational artifact magic".into());
        }
        let u32_at = |c: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(c, 4)?.try_into().unwrap()))
        };
        let art_version = u32_at(&mut c)?;
        if art_version != 2 && art_version != 3 {
            return Err("unsupported relational artifact version".into());
        }
        let mode = match take(&mut c, 1)?[0] {
            0 => RelMode::Geometric,
            1 => RelMode::Categorical,
            2 => RelMode::ExactOnly,
            _ => return Err("unknown relational mode".into()),
        };
        let vocab = u32_at(&mut c)? as usize;
        if vocab == 0 || vocab > 1 << 20 {
            return Err("relational artifact vocabulary out of range".into());
        }
        let q_roots = take(&mut c, vocab)?.to_vec();
        let n_codes = u32_at(&mut c)? as usize;
        let code_of = take(&mut c, n_codes)?.to_vec();
        if mode == RelMode::Categorical && code_of.len() != vocab {
            return Err("categorical arm requires a per-token code map".into());
        }
        if mode != RelMode::Categorical && !code_of.is_empty() {
            return Err("a non-categorical arm must not carry a code map".into());
        }
        let mut w = [0i32; EXACT_FEATS];
        for slot in w.iter_mut() {
            let b = take(&mut c, 1)?[0] as i8 as i32;
            if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(&b) {
                return Err("exact weight outside the declared bound".into());
            }
            *slot = b;
        }
        let mut rank = [0i32; RANKS];
        for slot in rank.iter_mut() {
            let b = take(&mut c, 1)?[0] as i8 as i32;
            if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(&b) {
                return Err("rank outside the declared bound".into());
            }
            *slot = b;
        }
        let mut sb = [0i32; ACTS];
        for slot in sb.iter_mut() {
            let b = take(&mut c, 1)?[0] as i8 as i32;
            if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(&b) {
                return Err("strength bias outside the declared bound".into());
            }
            *slot = b;
        }
        let bias = take(&mut c, 1)?[0] as i8 as i32;
        let noread = take(&mut c, 1)?[0] as i8 as i32;
        if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(&bias)
            || !(-WEIGHT_MAX..=WEIGHT_MAX).contains(&noread)
        {
            return Err("selector bias outside the declared bound".into());
        }
        // Version 2 has no contextual interaction; version 3 carries it explicitly.
        let mut ctx: Vec<[i32; ACTS + 1]> = Vec::new();
        if art_version >= 3 {
            let n_rows = u32_at(&mut c)? as usize;
            if n_rows != 0 && n_rows != CTX_BUCKETS {
                return Err("contextual interaction table has the wrong bucket count".into());
            }
            let mut rows = Vec::with_capacity(n_rows);
            for _ in 0..n_rows {
                let mut row = [0i32; ACTS + 1];
                for slot in row.iter_mut() {
                    let v = take(&mut c, 1)?[0] as i8 as i32;
                    if !(-WEIGHT_MAX..=WEIGHT_MAX).contains(&v) {
                        return Err("contextual entry outside the declared bound".into());
                    }
                    *slot = v;
                }
                rows.push(row);
            }
            ctx = rows;
        }
        let mut local_artifact_digest = [0u8; 32];
        local_artifact_digest.copy_from_slice(take(&mut c, 32)?);
        let mut tokenizer_digest = [0u8; 32];
        tokenizer_digest.copy_from_slice(take(&mut c, 32)?);
        let mut data_digest = [0u8; 32];
        data_digest.copy_from_slice(take(&mut c, 32)?);
        if c != bytes.len() {
            return Err("trailing bytes in the relational artifact".into());
        }
        if &local_artifact_digest != expect_local {
            return Err("relational artifact local-baseline digest differs".into());
        }
        if &tokenizer_digest != expect_tokenizer {
            return Err("relational artifact tokenizer digest differs".into());
        }
        let artifact = Self {
            selector: RelationalSelector {
                q_roots,
                mode,
                code_of,
                w,
                rank,
                bias,
                sb,
                noread,
                ctx,
            },
            local_artifact_digest,
            tokenizer_digest,
            data_digest,
        };
        artifact.selector.validate()?;
        Ok(artifact)
    }
}

fn xorshift(st: &mut u64) -> u64 {
    *st ^= *st << 13;
    *st ^= *st >> 7;
    *st ^= *st << 17;
    *st
}

fn mix_rng(rng: u64, epoch: u32) -> u64 {
    let mut z = rng
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_mul(epoch as u64 + 1);
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    z.wrapping_mul(0x94D0_49BB_1331_11EB)
}

/// Fisher-Yates order for one epoch, reconstructed from `(rng, epoch)`.
pub fn epoch_order(n: usize, rng: u64, epoch: u32) -> Vec<usize> {
    let mut st = mix_rng(rng, epoch) | 1;
    let mut p: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (xorshift(&mut st) as usize) % (i + 1);
        p.swap(i, j);
    }
    p
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> ExactGroupTable {
        ExactGroupTable::build().expect("exact table")
    }

    fn cand(feats: [u8; EXACT_FEATS], payload: u32) -> Cand {
        Cand {
            slot_ref: OccurrenceRef { seq: 1, abs: 0 },
            abs: 0,
            payload,
            x_cur: 0,
            x_prev: 0,
            x_prev2: 0,
            feats,
        }
    }

    fn boost(a: usize) -> f64 {
        (1i64 << AMP_SHIFTS[a]) as f64 / 1024.0
    }

    #[test]
    fn relation_is_common_left_invariant_and_not_right_invariant() {
        let t = table();
        let (q, k, g) = (5usize, 11usize, 7usize);
        let r = relation(&t, q, k);
        let gq = t.product[g * ROW_STRIDE + q] as usize;
        let gk = t.product[g * ROW_STRIDE + k] as usize;
        assert_eq!(relation(&t, gq, gk), r, "common-left invariance");
        let qg = t.product[q * ROW_STRIDE + g] as usize;
        let kg = t.product[k * ROW_STRIDE + g] as usize;
        assert_ne!(relation(&t, qg, kg), r, "not common-right invariant");
        assert_eq!(relation(&t, q, q), t.identity as usize);
        assert_ne!(relation(&t, q, k), t.identity as usize);
    }

    #[test]
    fn action_loss_matches_the_closed_form_and_prefers_the_reference_optimum() {
        let p = 0.01f64;
        let a = 2.0f64;
        let correct = action_loss(p, a, true);
        let wrong = action_loss(p, a, false);
        assert!((wrong - (1.0 + p * (a.exp() - 1.0)).ln()).abs() < 1e-12);
        assert!((correct - wrong + a).abs() < 1e-12);
        assert!(correct < 0.0);
        assert!(wrong > 0.0);
        let r = 0.5f64;
        let aopt = optimal_boost(p, r, 8.0);
        let f = |a: f64| (1.0 + p * (a.exp() - 1.0)).ln() - r * a;
        let best = (0..=800)
            .map(|k| k as f64 / 100.0)
            .min_by(|x, y| f(*x).partial_cmp(&f(*y)).unwrap())
            .unwrap();
        assert!((best - aopt).abs() < 0.02, "grid {best} vs derived {aopt}");
    }

    #[test]
    fn admission_shares_any_exact_context_token_and_respects_the_bound() {
        let mut r = OccurrenceRing::new(128);
        for t in [9u32, 3, 7, 4, 7, 3] {
            r.observe(t);
        }
        let (c, st) = admit_mixed(&r, 6, 9, 3, 7, 24);
        // Shares cur=9 at abs 0; shares prev=3 at abs 2; shares prev2=7 at abs 4.
        assert_eq!(st.matches, 3, "matches {}", st.matches);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].abs, 4, "newest first");
        assert_eq!(c[2].abs, 0);
        for k in 1..c.len() {
            assert!(c[k - 1].abs > c[k].abs, "newest first");
        }
        for x in c.iter() {
            assert!(x.abs + 1 < 6, "successor inside the prefix");
        }
        let (c2, st2) = admit_mixed(&r, 6, 9, 3, 7, 2);
        assert_eq!(c2.len(), 2);
        assert!(st2.dropped_by_bound > 0);
    }

    #[test]
    fn no_read_is_available_and_beats_a_doomed_read() {
        let mut tr = RelationalTrainer::new(
            16,
            table(),
            0.1,
            13,
            [1u8; 32],
            [2u8; 32],
            &(0..16u8).collect::<Vec<u8>>(),
        );
        let s = tr.quantize();
        s.validate().unwrap();
        let c = cand([1, 1, 0, 0, 0, 1], 5);
        assert_eq!(s.choose(&[c], &[0]), None);
        let pos = TrainPos {
            cands: vec![c],
            q_role: 1,
            k_role: vec![2],
            delta: vec![[
                action_loss(0.01, boost(0), false),
                action_loss(0.01, boost(1), false),
                action_loss(0.01, boost(2), false),
            ]],
            group: 0,
        };
        let before = tr.position_loss(&pos);
        for _ in 0..50 {
            tr.step(std::slice::from_ref(&pos), &[0]);
        }
        assert!(tr.position_loss(&pos) < before);
        assert!(tr.noread > 0.0, "the policy must learn to abstain here");
    }

    #[test]
    fn a_correct_payload_learns_a_positive_read_at_a_bounded_strength() {
        let mut tr = RelationalTrainer::new(
            16,
            table(),
            0.05,
            7,
            [3u8; 32],
            [4u8; 32],
            &(0..16u8).collect::<Vec<u8>>(),
        );
        let c = cand([1, 1, 0, 0, 0, 1], 5);
        let pos = TrainPos {
            cands: vec![c],
            q_role: 1,
            k_role: vec![2],
            delta: vec![[
                action_loss(0.01, boost(0), true),
                action_loss(0.01, boost(1), true),
                action_loss(0.01, boost(2), true),
            ]],
            group: 0,
        };
        for _ in 0..80 {
            tr.step(std::slice::from_ref(&pos), &[0]);
        }
        let s = tr.quantize();
        s.validate().unwrap();
        assert!(
            s.choose(&pos.cands, &[0]).is_some(),
            "a beneficial read must be selected"
        );
        assert!(tr.position_loss(&pos) < 0.0);
    }

    #[test]
    fn descriptor_search_never_increases_the_objective_and_is_bounded() {
        let t = table();
        let vocab = 8usize;
        let mut tr = RelationalTrainer::new(
            vocab,
            t.clone(),
            0.05,
            5,
            [5u8; 32],
            [6u8; 32],
            &(0..vocab as u8).collect::<Vec<u8>>(),
        );
        let mk = |qr: usize, kr: usize, correct: bool| TrainPos {
            cands: vec![cand([0, 0, 0, 0, 0, 1], 3)],
            q_role: qr,
            k_role: vec![kr],
            delta: vec![[
                action_loss(0.01, boost(0), correct),
                action_loss(0.01, boost(1), correct),
                action_loss(0.01, boost(2), correct),
            ]],
            group: 0,
        };
        let mut positions = vec![mk(1, 2, true), mk(2, 1, true)];
        for (a, b) in [(1usize, 5usize), (2, 6), (1, 7), (2, 3), (5, 1), (6, 2)] {
            positions.push(mk(a, b, false));
        }
        let before: f64 = positions.iter().map(|p| tr.position_loss(p)).sum();
        let improvement = tr.refine_descriptor(&positions, 2);
        let after: f64 = positions.iter().map(|p| tr.position_loss(p)).sum();
        assert!(
            improvement <= 1e-12,
            "search must not increase the objective"
        );
        assert!(after <= before + 1e-12);
        assert!(tr.descriptor_evaluations > 0);
        assert!(tr.descriptor_evaluations <= (vocab * RANKS * 2) as u64);
        assert_ne!(relation(&t, 1, 2), relation(&t, 2, 1));
    }

    #[test]
    fn the_single_relation_function_agrees_with_the_trainers_own_index() {
        let t = table();
        let vocab = 8usize;
        let mut tr = RelationalTrainer::new(
            vocab,
            t.clone(),
            0.02,
            3,
            [1u8; 32],
            [2u8; 32],
            &(0..8u8).collect::<Vec<u8>>(),
        )
        .with_mode(
            RelMode::Categorical,
            (0..vocab).map(|k| (k * 5 + 1) as u8).collect(),
        );
        let p = TrainPos {
            cands: vec![cand([0; EXACT_FEATS], 1)],
            q_role: 2,
            k_role: vec![5],
            delta: vec![[0.1, 0.1, 0.1]],
            group: 0,
        };
        let via_trainer = tr.rel_of(&p, 0);
        let via_function = relation_index(
            &t,
            RelMode::Categorical,
            &tr.code_of,
            tr.roots[2],
            tr.roots[5],
            2,
            5,
        );
        assert_eq!(via_trainer, via_function);
        let mut g = tr.clone();
        g.mode = RelMode::Geometric;
        assert_ne!(
            g.rel_of(&p, 0),
            via_trainer,
            "arm encodings differ on the same inputs"
        );
    }

    #[test]
    fn artifact_round_trips_the_encoding_and_rejects_mismatches() {
        let t = table();
        let mut tr = RelationalTrainer::new(
            8,
            t,
            0.02,
            3,
            [1u8; 32],
            [2u8; 32],
            &(0..8u8).collect::<Vec<u8>>(),
        )
        .with_mode(
            RelMode::Categorical,
            (0..8).map(|k| (k * 3 + 2) as u8).collect(),
        );
        tr.rank[4] = 1.5;
        let sel = tr.quantize();
        let art = RelationalArtifact {
            selector: sel.clone(),
            local_artifact_digest: [7u8; 32],
            tokenizer_digest: [9u8; 32],
            data_digest: [3u8; 32],
        };
        let b = art.to_bytes();
        let back = RelationalArtifact::from_bytes(&b, &[7u8; 32], &[9u8; 32]).expect("round trip");
        assert_eq!(back, art);
        assert_eq!(back.selector.mode, RelMode::Categorical);
        assert_eq!(back.selector.code_of, sel.code_of);
        assert!(RelationalArtifact::from_bytes(&b, &[8u8; 32], &[9u8; 32]).is_err());
        assert!(RelationalArtifact::from_bytes(&b, &[7u8; 32], &[8u8; 32]).is_err());
        assert!(RelationalArtifact::from_bytes(&b[..b.len() - 1], &[7u8; 32], &[9u8; 32]).is_err());
        let mut trailing = b.clone();
        trailing.push(0);
        assert!(RelationalArtifact::from_bytes(&trailing, &[7u8; 32], &[9u8; 32]).is_err());
        let mut g = art.clone();
        g.selector.mode = RelMode::Geometric;
        assert!(RelationalArtifact::from_bytes(&g.to_bytes(), &[7u8; 32], &[9u8; 32]).is_err());
    }

    #[test]
    fn descriptor_search_is_deduplicated_strict_and_never_increases_the_objective() {
        let t = table();
        let vocab = 6usize;
        let mut tr = RelationalTrainer::new(
            vocab,
            t,
            0.05,
            5,
            [5u8; 32],
            [6u8; 32],
            &(0..vocab as u8).collect::<Vec<u8>>(),
        );
        let mk = |qr: usize, kr: usize, correct: bool| TrainPos {
            cands: vec![cand([0, 0, 0, 0, 0, 1], 3)],
            q_role: qr,
            k_role: vec![kr],
            delta: vec![[
                action_loss(0.01, boost(0), correct),
                action_loss(0.01, boost(1), correct),
                action_loss(0.01, boost(2), correct),
            ]],
            group: 0,
        };
        // Position 0 references token 1 as both query and source role: a naive concatenation would
        // count it twice for token 1.
        let positions = vec![
            mk(1, 1, true),
            mk(1, 2, true),
            mk(2, 1, true),
            mk(1, 3, false),
            mk(3, 1, false),
        ];
        let before: f64 = positions.iter().map(|p| tr.position_loss(p)).sum();
        let change = tr.refine_descriptor(&positions, 2);
        let after: f64 = positions.iter().map(|p| tr.position_loss(p)).sum();
        assert!(
            change <= 1e-12,
            "search must not increase the objective: {change}"
        );
        assert!(
            (after - before - change).abs() < 1e-9,
            "the returned change must be the exact objective change"
        );
        assert!(tr.descriptor_evaluations <= (vocab * RANKS * 2) as u64);
    }

    #[test]
    fn categorical_search_changes_the_map_and_preserves_ties_and_unrelated_tokens() {
        let mut tr =
            RelationalTrainer::new(4, table(), 0.05, 5, [5u8; 32], [6u8; 32], &[0, 1, 2, 3])
                .with_mode(RelMode::Categorical, vec![0; 4]);
        // A correct source benefits from a relation index whose learned weight is
        // positive. Initially the (0, 0) code pair addresses rank[0], not rank[1].
        tr.rank[1] = 2.0;
        let positions = vec![TrainPos {
            cands: vec![cand([0; EXACT_FEATS], 3)],
            q_role: 0,
            k_role: vec![1],
            delta: vec![[-0.1, -1.0, -2.0]],
            group: 0,
        }];
        let initial = tr.code_of.clone();
        let before = tr.position_loss(&positions[0]);
        let change = tr.refine_descriptor(&positions, 2);
        let after = tr.position_loss(&positions[0]);
        assert!(
            change < -1e-6,
            "a genuinely beneficial code move must be kept"
        );
        assert!(after < before);
        assert!((after - before - change).abs() < 1e-12);
        assert_ne!(tr.code_of, initial);
        assert_eq!(&tr.code_of[2..], &initial[2..]);
        assert!(tr.descriptor_moved() > 0);
        assert!(tr.descriptor_evaluations <= (2 * (RANKS - 1) * 2) as u64);

        // With a constant relation table every code is tied. Keep all incumbents.
        tr.rank = [0.0; RANKS];
        let incumbent = tr.code_of.clone();
        let moves = tr.descriptor_moves;
        assert_eq!(tr.refine_descriptor(&positions, 2), 0.0);
        assert_eq!(tr.code_of, incumbent);
        assert_eq!(tr.descriptor_moves, moves);
    }

    #[test]
    fn artifact_rejects_invalid_map_values_and_vocabulary_alignment() {
        let tr = RelationalTrainer::new(4, table(), 0.05, 5, [5u8; 32], [6u8; 32], &[0, 1, 2, 3])
            .with_mode(RelMode::Categorical, vec![0; 4]);
        let art = RelationalArtifact {
            selector: tr.quantize(),
            local_artifact_digest: [7u8; 32],
            tokenizer_digest: [9u8; 32],
            data_digest: [3u8; 32],
        };
        assert!(art.selector.validate_for_vocab(4).is_ok());
        assert!(art.selector.validate_for_vocab(3).is_err());
        let mut bad_root = art.clone();
        bad_root.selector.q_roots[0] = RANKS as u8;
        assert!(
            RelationalArtifact::from_bytes(&bad_root.to_bytes(), &[7u8; 32], &[9u8; 32]).is_err()
        );
        let mut bad_code = art.clone();
        bad_code.selector.code_of[0] = RANKS as u8;
        assert!(
            RelationalArtifact::from_bytes(&bad_code.to_bytes(), &[7u8; 32], &[9u8; 32]).is_err()
        );
        for coefficient in [WEIGHT_MAX + 1, -128] {
            let mut bad_weight = art.clone();
            bad_weight.selector.w[0] = coefficient;
            assert!(
                RelationalArtifact::from_bytes(&bad_weight.to_bytes(), &[7u8; 32], &[9u8; 32])
                    .is_err()
            );
        }
        let mut short_map = art;
        short_map.selector.code_of.pop();
        assert!(short_map.selector.validate().is_err());
        assert!(
            RelationalArtifact::from_bytes(&short_map.to_bytes(), &[7u8; 32], &[9u8; 32]).is_err()
        );
    }

    #[test]
    fn selector_rejects_extreme_coefficients_without_overflow() {
        let valid = RelationalSelector {
            q_roots: vec![0],
            mode: RelMode::Geometric,
            code_of: Vec::new(),
            w: [0; EXACT_FEATS],
            rank: [0; RANKS],
            bias: 0,
            sb: [0; ACTS],
            noread: 0,
            ctx: Vec::new(),
        };
        for field in 0..5 {
            let mut invalid = valid.clone();
            match field {
                0 => invalid.w[0] = i32::MIN,
                1 => invalid.rank[0] = i32::MIN,
                2 => invalid.sb[0] = i32::MIN,
                3 => invalid.bias = i32::MIN,
                _ => invalid.noread = i32::MIN,
            }
            assert!(invalid.validate().is_err());
        }
    }

    #[test]
    fn categorical_checkpoint_preserves_learned_codes_and_continuation() {
        let make = || {
            RelationalTrainer::new(4, table(), 0.05, 5, [5u8; 32], [6u8; 32], &[0, 1, 2, 3])
                .with_mode(RelMode::Categorical, vec![0; 4])
        };
        let mut tr = make();
        tr.rank[1] = 2.0;
        let positions = vec![TrainPos {
            cands: vec![cand([0; EXACT_FEATS], 3)],
            q_role: 0,
            k_role: vec![1],
            delta: vec![[-0.1, -1.0, -2.0]],
            group: 0,
        }];
        tr.refine_descriptor(&positions, 1);
        assert!(tr.descriptor_moved() > 0);
        tr.step(&positions, &[0]);
        let bytes = tr.checkpoint_bytes();
        let mut restored = make();
        restored.resume_from(&bytes).expect("categorical resume");
        assert_eq!(restored.checkpoint_bytes(), bytes);
        assert_eq!(restored.descriptor_moved(), tr.descriptor_moved());
        tr.step(&positions, &[0]);
        restored.step(&positions, &[0]);
        assert_eq!(restored.checkpoint_bytes(), tr.checkpoint_bytes());

        let mut wrong_mode = make();
        wrong_mode.mode = RelMode::ExactOnly;
        wrong_mode.code_of.clear();
        wrong_mode.initial_codes.clear();
        let before = wrong_mode.checkpoint_bytes();
        assert!(wrong_mode.resume_from(&bytes).is_err());
        assert_eq!(wrong_mode.checkpoint_bytes(), before);
    }

    #[test]
    fn legacy_checkpoint_is_explicitly_geometric_only() {
        let make =
            || RelationalTrainer::new(4, table(), 0.05, 5, [5u8; 32], [6u8; 32], &[0, 1, 2, 3]);
        let tr = make();
        let v2 = tr.checkpoint_bytes();
        // V1 had no mode, current codes, or initial-map provenance before roots.
        let mut legacy = v2[..12].to_vec();
        legacy[4..8].copy_from_slice(&1u32.to_le_bytes());
        legacy.extend_from_slice(&v2[12 + 1 + 4 + tr.vocab + 4..]);
        let mut geometric = make();
        geometric
            .resume_from(&legacy)
            .expect("legacy geometric resume");
        assert_eq!(geometric.checkpoint_bytes(), v2);
        let mut categorical = make().with_mode(RelMode::Categorical, vec![0; 4]);
        assert!(categorical.resume_from(&legacy).is_err());
    }

    #[test]
    fn checkpoint_round_trips_and_rejects_a_foreign_identity() {
        let mut tr = RelationalTrainer::new(
            8,
            table(),
            0.02,
            3,
            [7u8; 32],
            [8u8; 32],
            &(0..8u8).collect::<Vec<u8>>(),
        );
        let pos = TrainPos {
            cands: vec![cand([1, 0, 0, 0, 0, 0], 2)],
            q_role: 0,
            k_role: vec![1],
            delta: vec![[0.5, 0.25, 0.1]],
            group: 0,
        };
        for _ in 0..4 {
            tr.step(std::slice::from_ref(&pos), &[0]);
        }
        tr.refine_descriptor(std::slice::from_ref(&pos), 1);
        let b = tr.checkpoint_bytes();
        let mut foreign = RelationalTrainer::new(
            8,
            table(),
            0.02,
            3,
            [9u8; 32],
            [8u8; 32],
            &(0..8u8).collect::<Vec<u8>>(),
        );
        assert!(foreign.resume_from(&b).is_err());
        let mut same = RelationalTrainer::new(
            8,
            table(),
            0.02,
            3,
            [7u8; 32],
            [8u8; 32],
            &(0..8u8).collect::<Vec<u8>>(),
        );
        same.resume_from(&b).expect("resume");
        assert_eq!(same.roots, tr.roots);
        assert_eq!(same.rank, tr.rank);
        assert_eq!(same.step, tr.step);
        assert_eq!(same.checkpoint_bytes(), b);
    }

    #[test]
    fn the_without_relation_comparator_drops_only_the_relation_term() {
        let mut tr = RelationalTrainer::new(
            8,
            table(),
            0.02,
            3,
            [1u8; 32],
            [2u8; 32],
            &(0..8u8).collect::<Vec<u8>>(),
        );
        tr.rank[3] = 2.0;
        tr.w[0] = 1.5;
        tr.noread = -1.0;
        let full = tr.quantize();
        let exact = full.without_relation();
        assert_eq!(full.w, exact.w);
        assert_eq!(full.sb, exact.sb);
        assert_eq!(full.noread, exact.noread);
        assert_eq!(full.q_roots, exact.q_roots);
        assert!(exact.rank.iter().all(|v| *v == 0));
        let c = cand([1, 0, 0, 0, 0, 0], 1);
        assert_ne!(full.score(&c, 3, 0), exact.score(&c, 3, 0));
    }

    // ---- contextual utility interaction ---------------------------------------

    /// A candidate whose exact features select a specific causal bucket.
    fn bucketed(f3: u8, prev_match: u8, newest: u8, payload: u32) -> Cand {
        let mut f = [0u8; EXACT_FEATS];
        f[3] = f3;
        f[1] = prev_match;
        f[5] = newest;
        cand(f, payload)
    }

    #[test]
    fn an_all_zero_interaction_reproduces_the_factorised_selector_exactly() {
        let c = vec![bucketed(1, 1, 1, 5), bucketed(0, 1, 0, 7)];
        let rel = vec![3usize, 9usize];
        let base = RelationalSelector {
            q_roots: vec![0, 1, 2, 3],
            mode: RelMode::Geometric,
            code_of: Vec::new(),
            w: [1, -2, 3, 0, 2, -1],
            rank: std::array::from_fn(|k| (k as i32 % 5) - 2),
            bias: 1,
            sb: [2, 4, -1],
            noread: 0,
            ctx: Vec::new(),
        };
        let mut zeroed = base.clone();
        zeroed.ctx = vec![[0i32; ACTS + 1]; CTX_BUCKETS];
        for k in 0..c.len() {
            for a in 0..ACTS {
                assert_eq!(base.score(&c[k], rel[k], a), zeroed.score(&c[k], rel[k], a));
                assert_eq!(
                    base.strength_score(&c[k], rel[k], a, 0),
                    zeroed.strength_score(&c[k], rel[k], a, 0)
                );
            }
        }
        assert_eq!(base.choose(&c, &rel), zeroed.choose(&c, &rel));
        assert_eq!(base.noread_score(0), zeroed.noread_score(0));
    }

    #[test]
    fn the_bucket_key_ignores_the_interaction_and_the_strength_bias() {
        let c = vec![bucketed(1, 0, 1, 5), bucketed(0, 1, 0, 7)];
        let rel = vec![3usize, 9usize];
        let mut rank = [0i32; RANKS];
        rank[3] = 5;
        rank[9] = -5;
        let mut s = RelationalSelector {
            q_roots: vec![0; 4],
            mode: RelMode::Geometric,
            code_of: Vec::new(),
            w: [0; EXACT_FEATS],
            rank,
            bias: 0,
            sb: [0; ACTS],
            noread: 0,
            ctx: Vec::new(),
        };
        let b0 = s.bucket_of(&c, &rel);
        s.sb = [7, 7, 7];
        s.noread = -7;
        s.ctx = vec![[1i32; ACTS + 1]; CTX_BUCKETS];
        assert_eq!(
            s.bucket_of(&c, &rel),
            b0,
            "bucket must be a frozen causal key"
        );
        // The top-ranked candidate drives the exact-context bits: rel 9 is empty, so cand 0 wins.
        assert_eq!(b0 & 0b1, 1, "payload equals the current token");
        assert_eq!(b0 & 0b1000, 1 << 3, "a strict source margin exists");
        assert_eq!(b0 & 0b10, 0, "cand 0 has no two-neighbour agreement");
    }

    #[test]
    fn the_interaction_fit_moves_entries_and_lowers_the_served_objective() {
        // Two buckets with opposite truth: bucket A wants a read, bucket B must abstain. A single
        // global strength cannot express both; the interaction can.
        let t = table();
        let mut tr = RelationalTrainer::new(
            4,
            t.clone(),
            0.05,
            11,
            [8u8; 32],
            [9u8; 32],
            &(0..4u8).collect::<Vec<u8>>(),
        );
        for _ in 0..40 {
            tr.step(
                &[
                    TrainPos {
                        cands: vec![bucketed(1, 0, 0, 5)],
                        q_role: 1,
                        k_role: vec![2],
                        delta: vec![[
                            action_loss(0.001, boost(0), true),
                            action_loss(0.001, boost(1), true),
                            action_loss(0.001, boost(2), true),
                        ]],
                        group: 0,
                    },
                    TrainPos {
                        cands: vec![bucketed(0, 0, 0, 6)],
                        q_role: 1,
                        k_role: vec![2],
                        delta: vec![[
                            action_loss(0.001, boost(0), false),
                            action_loss(0.001, boost(1), false),
                            action_loss(0.001, boost(2), false),
                        ]],
                        group: 0,
                    },
                ],
                &[0, 1],
            );
        }
        let mut sel = tr.quantize();
        let positions = vec![
            TrainPos {
                cands: vec![bucketed(1, 0, 0, 5)],
                q_role: 1,
                k_role: vec![2],
                delta: vec![[
                    action_loss(0.001, boost(0), true),
                    action_loss(0.001, boost(1), true),
                    action_loss(0.001, boost(2), true),
                ]],
                group: 0,
            },
            TrainPos {
                cands: vec![bucketed(0, 0, 0, 6)],
                q_role: 1,
                k_role: vec![2],
                delta: vec![[
                    action_loss(0.001, boost(0), false),
                    action_loss(0.001, boost(1), false),
                    action_loss(0.001, boost(2), false),
                ]],
                group: 0,
            },
        ];
        let before: f64 = sel
            .buckets_of(&t, &positions)
            .iter()
            .zip(positions.iter())
            .map(|(b, p)| sel.position_loss_soft(&t, p, *b))
            .sum();
        let buckets = sel.buckets_of(&t, &positions);
        let receipt = sel.ctx_fit(&t, &positions, &buckets, 2);
        let after: f64 = buckets
            .iter()
            .zip(positions.iter())
            .map(|(b, p)| sel.position_loss_soft(&t, p, *b))
            .sum();
        assert!(receipt.evaluations > 0);
        assert!(
            receipt.entries_moved > 0,
            "an optimum-finding search must move"
        );
        assert!(after < before, "{after} !< {before}");
        sel.validate().unwrap();
        // The two constructed buckets genuinely differ, so the interaction must treat them apart.
        assert_ne!(buckets[0], buckets[1]);
        assert_ne!(
            sel.ctx[buckets[0]], sel.ctx[buckets[1]],
            "the controller must distinguish the two contexts"
        );
    }

    #[test]
    fn the_contextual_interaction_survives_the_versioned_artifact_round_trip() {
        let mut sel = RelationalSelector {
            q_roots: vec![0, 1, 2, 3],
            mode: RelMode::Geometric,
            code_of: Vec::new(),
            w: [1, 0, -1, 2, 0, 1],
            rank: [0; RANKS],
            bias: 3,
            sb: [1, 2, 3],
            noread: -1,
            ctx: Vec::new(),
        };
        let art = |s: RelationalSelector| RelationalArtifact {
            selector: s,
            local_artifact_digest: [1u8; 32],
            tokenizer_digest: [2u8; 32],
            data_digest: [3u8; 32],
        };
        // A factorised selector still round-trips on the new version.
        let back =
            RelationalArtifact::from_bytes(&art(sel.clone()).to_bytes(), &[1u8; 32], &[2u8; 32])
                .unwrap();
        assert!(back.selector.ctx.is_empty());
        assert_eq!(back.selector, sel);
        // A fitted interaction round-trips with every entry intact.
        sel.ctx = vec![[0i32; ACTS + 1]; CTX_BUCKETS];
        sel.ctx[5] = [2, -3, 4, 7];
        let bytes = art(sel.clone()).to_bytes();
        let back = RelationalArtifact::from_bytes(&bytes, &[1u8; 32], &[2u8; 32]).unwrap();
        assert_eq!(back.selector, sel);
        assert_eq!(back.selector.ctx[5], [2, -3, 4, 7]);
        // A malformed table is rejected rather than silently loaded.
        let mut bad = sel.clone();
        bad.ctx = vec![[0i32; ACTS + 1]; CTX_BUCKETS - 1];
        assert!(bad.validate().is_err());
        let mut bad = sel.clone();
        bad.ctx[0][0] = WEIGHT_MAX + 1;
        assert!(bad.validate().is_err());
        // Version 2 payloads (no interaction) remain readable.
        let head = 4
            + 4
            + 1
            + 4
            + sel.q_roots.len()
            + 4
            + sel.code_of.len()
            + EXACT_FEATS
            + RANKS
            + ACTS
            + 1
            + 1;
        let v3 = art(sel.clone()).to_bytes();
        assert_eq!(v3[head], (CTX_BUCKETS as u8), "ctx length prefix");
        let mut legacy = v3.clone();
        legacy[4] = 2; // version word, little-endian u32
        legacy.drain(head..head + 4 + CTX_BUCKETS * (ACTS + 1));
        let back = RelationalArtifact::from_bytes(&legacy, &[1u8; 32], &[2u8; 32]).unwrap();
        assert!(back.selector.ctx.is_empty());
        // A truncated interaction table is rejected rather than silently loaded.
        let mut trunc = v3.clone();
        trunc.truncate(head + 4 + CTX_BUCKETS * (ACTS + 1) - 1);
        assert!(RelationalArtifact::from_bytes(&trunc, &[1u8; 32], &[2u8; 32]).is_err());
    }
}
