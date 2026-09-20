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

/// The served integer selector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalSelector {
    /// Learned per-token descriptor root.
    pub q_roots: Vec<u8>,
    pub w: [i32; EXACT_FEATS],
    pub rank: [i32; RANKS],
    pub bias: i32,
    pub sb: [i32; ACTS],
    pub noread: i32,
}

impl RelationalSelector {
    /// The matched exact-recurrence comparator: identical features, actions and dose, no relation
    /// term and no learned descriptor.
    pub fn without_relation(&self) -> Self {
        let mut s = self.clone();
        s.rank = [0; RANKS];
        s
    }

    pub fn validate(&self) -> Result<(), String> {
        for (k, v) in self.w.iter().enumerate() {
            if v.abs() > WEIGHT_MAX {
                return Err(format!("exact weight {k} = {v} outside +/-{WEIGHT_MAX}"));
            }
        }
        for (k, v) in self.rank.iter().enumerate() {
            if v.abs() > WEIGHT_MAX {
                return Err(format!("rank {k} = {v} outside +/-{WEIGHT_MAX}"));
            }
        }
        for (k, v) in self.sb.iter().enumerate() {
            if v.abs() > WEIGHT_MAX {
                return Err(format!("strength bias {k} = {v} outside +/-{WEIGHT_MAX}"));
            }
        }
        if self.bias.abs() > WEIGHT_MAX || self.noread.abs() > WEIGHT_MAX {
            return Err("selector bias outside the declared bound".into());
        }
        if self.q_roots.is_empty() {
            return Err("empty descriptor".into());
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

    /// `None` is NoRead; otherwise `(candidate index, strength index)`.
    pub fn choose(&self, cands: &[Cand], rel: &[usize]) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize, i32)> = None;
        for (k, c) in cands.iter().enumerate() {
            for a in 0..ACTS {
                let s = self.score(c, rel[k], a);
                if best.is_none_or(|(_, _, pr)| s > pr) {
                    best = Some((k, a, s));
                }
            }
        }
        match best {
            Some((k, a, s)) if s > self.noread => Some((k, a)),
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
            mode: RelMode::Geometric,
            code_of: Vec::new(),
        }
    }

    /// Switch the comparison arm. `code_of` is required for `Categorical`.
    pub fn with_mode(mut self, mode: RelMode, code_of: Vec<u8>) -> Self {
        self.mode = mode;
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
        match self.mode {
            RelMode::ExactOnly => 0,
            RelMode::Geometric => relation(
                &self.table,
                self.roots[qr] as usize,
                self.roots[kr] as usize,
            ),
            RelMode::Categorical => {
                let a = *self.code_of.get(qr).unwrap_or(&0) as usize;
                let b = *self.code_of.get(kr).unwrap_or(&0) as usize;
                (a * 7 + b) % RANKS
            }
        }
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

    /// Bounded discrete coordinate search on the descriptor map, against the same objective.
    ///
    /// For each token in a deterministic order every one of the 120 roots is evaluated on the
    /// positions that reference that token as a role; the best strictly-improving root is kept.
    /// Positions are indexed by role token, so only affected positions are rescored.
    pub fn refine_descriptor(&mut self, positions: &[TrainPos], rounds: usize) -> f64 {
        if self.mode != RelMode::Geometric {
            return 0.0;
        }
        let mut by_q: Vec<Vec<usize>> = vec![Vec::new(); self.vocab];
        let mut by_k: Vec<Vec<usize>> = vec![Vec::new(); self.vocab];
        for (i, p) in positions.iter().enumerate() {
            if p.q_role != usize::MAX {
                by_q[p.q_role].push(i);
            }
            for k in p.k_role.iter() {
                if *k != usize::MAX && *k < self.vocab {
                    by_k[*k].push(i);
                }
            }
        }
        let before: f64 = positions.iter().map(|p| self.position_loss(p)).sum();
        for _ in 0..rounds {
            for t in 0..self.vocab {
                if by_q[t].is_empty() && by_k[t].is_empty() {
                    continue;
                }
                let cur = self.roots[t];
                let mut best_loss = f64::INFINITY;
                let mut best_root = cur;
                let affected: Vec<usize> = by_q[t].iter().chain(by_k[t].iter()).copied().collect();
                for r in 0..RANKS {
                    self.roots[t] = r as u8;
                    self.descriptor_evaluations += 1;
                    let mut l = 0.0f64;
                    for &i in affected.iter() {
                        l += self.position_loss(&positions[i]);
                    }
                    if l < best_loss {
                        best_loss = l;
                        best_root = r as u8;
                    }
                }
                self.roots[t] = best_root;
                if best_root != cur {
                    self.descriptor_moves += 1;
                }
            }
        }
        let after: f64 = positions.iter().map(|p| self.position_loss(p)).sum();
        after - before
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
            w: std::array::from_fn(|k| q(self.w[k])),
            rank: std::array::from_fn(|k| q(self.rank[k])),
            bias: q(self.bias),
            sb: std::array::from_fn(|k| q(self.sb[k])),
            noread: q(self.noread),
        }
    }

    /// How many descriptor roots moved from the declared initialisation.
    pub fn descriptor_moved(&self) -> usize {
        self.roots
            .iter()
            .zip(self.initial_roots.iter())
            .filter(|(a, b)| a != b)
            .count()
    }

    pub fn checkpoint_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLRK");
        o.extend_from_slice(&1u32.to_le_bytes());
        o.extend_from_slice(&(self.vocab as u32).to_le_bytes());
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
        if u32_at(&mut c)? != 1 {
            return Err("unsupported relational checkpoint version".into());
        }
        if u32_at(&mut c)? as usize != self.vocab {
            return Err("relational checkpoint vocabulary differs".into());
        }
        self.roots.copy_from_slice(take(&mut c, self.vocab)?);
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
}
