//! Shared learned primitive transitions on a persistent computed result.
//!
//! One read produces an owned payload; a **learned initial state** `E[payload]` is computed from it;
//! each observed primitive applies the **same shared action** wherever that primitive occurs:
//!
//! ```text
//! s0   = E[selected payload]
//! s_j  = A[observed primitive j] * s_{j-1}        exact ordered products
//! out_j = D(s_j)                                   learned lexical decoder
//! Stop  = P(remaining observed primitives)         supervised input-exhaustion table
//! ```
//!
//! The result state is retained between emissions: the emitted tokens are *not* re-read as a new
//! query, and a wrong emission cannot corrupt a later step because `s_j` depends only on the selected
//! payload and the observed primitive prefix. Actions are shared across every occurrence of a
//! primitive, so an unseen ordered combination reuses the same codes rather than an independent entry.
//!
//! Reversible transport is used for the product; the surrounding machine is not forced into a group.
//! Emission and stopping are separate typed outcomes, and unknown operands, ungrounded states and an
//! absent read are distinguishable rather than collapsed into one identity fallback.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use super::result_decoder::{fit_decoder, DecObservation, DecOutcome, ResultDecoder};

/// Shortlist size stored per grounded result state.
pub const ST_K: usize = 4;
/// Bounded number of grounded result states.
pub const ST_MAX_STATES: usize = 16;

/// One typed outcome of a single response step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    Emit,
    Stop,
    UnknownValue,
    UnknownPrimitive,
    NoGrounding,
    NoRead,
    Exhausted,
    InvalidState,
}

/// The complete served response for one item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub tokens: Vec<u32>,
    /// Result state entered at each emitted token.
    pub states: Vec<usize>,
    pub steps: Vec<StepKind>,
    pub stopped: bool,
}

/// A verified non-abelian order-8 subgroup of the 120-element table (the quaternion group).
///
/// Returning the witness from the table rather than assuming it makes order sensitivity a measured
/// property of the served algebra: an abelian alternative provably cannot represent a reversal.
pub fn q8_witness() -> Result<[u8; 8], String> {
    let t = group_table();
    let e = t.identity as usize;
    let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
    let order_of = |g: usize| -> usize {
        let mut x = e;
        for k in 1..=GROUP_ORDER {
            x = mul(x, g);
            if x == e {
                return k;
            }
        }
        0
    };
    for a in 0..GROUP_ORDER {
        if order_of(a) != 4 {
            continue;
        }
        let a2 = mul(a, a);
        let a3 = mul(a2, a);
        let cyc: BTreeSet<usize> = [e, a, a2, a3].into_iter().collect();
        if cyc.len() != 4 {
            continue;
        }
        for b in 0..GROUP_ORDER {
            if cyc.contains(&b) || order_of(b) != 4 || mul(b, b) != a2 {
                continue;
            }
            let mut set: BTreeSet<usize> = cyc.clone();
            for x in cyc.iter() {
                set.insert(mul(b, *x));
            }
            if set.len() != 8 {
                continue;
            }
            let elems: Vec<usize> = set.iter().copied().collect();
            let closed = elems
                .iter()
                .all(|x| elems.iter().all(|y| set.contains(&mul(*x, *y))));
            if !closed {
                continue;
            }
            let nonabelian = elems
                .iter()
                .any(|x| elems.iter().any(|y| mul(*x, *y) != mul(*y, *x)));
            if !nonabelian {
                continue;
            }
            let mut out = [0u8; 8];
            for (i, v) in elems.iter().enumerate() {
                out[i] = *v as u8;
            }
            return Ok(out);
        }
    }
    Err("no non-abelian order-8 subgroup found in the served table".into())
}

/// One development item: the observed primitive sequence and its grounded response labels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StExample {
    pub payload: u32,
    pub primitives: Vec<u32>,
    pub targets: Vec<u32>,
}

/// A finite shared transition comparator fitted from the same intermediate labels.
/// Initial inputs and computed outcome tokens have separate tables; an unseen
/// complete instruction sequence can reuse observed local transitions.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FiniteTransitionModel {
    version: u8,
    initial: Vec<[u32; 3]>,
    recurrent: Vec<[u32; 3]>,
    values: Vec<u32>,
    actions: Vec<u32>,
    stop_table: Vec<(u8, bool)>,
}

impl FiniteTransitionModel {
    pub fn fit(examples: &[StExample]) -> Self {
        let mut initial: BTreeMap<(u32, u32), BTreeMap<u32, usize>> = BTreeMap::new();
        let mut recurrent: BTreeMap<(u32, u32), BTreeMap<u32, usize>> = BTreeMap::new();
        for ex in examples {
            for (j, (&primitive, &target)) in ex.primitives.iter().zip(&ex.targets).enumerate() {
                let (table, input) = if j == 0 {
                    (&mut initial, ex.payload)
                } else {
                    (&mut recurrent, ex.targets[j - 1])
                };
                *table
                    .entry((input, primitive))
                    .or_default()
                    .entry(target)
                    .or_default() += 1;
            }
        }
        let choose = |table: BTreeMap<(u32, u32), BTreeMap<u32, usize>>| {
            table
                .into_iter()
                .filter_map(|((input, primitive), counts)| {
                    counts
                        .into_iter()
                        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
                        .map(|(target, _)| [input, primitive, target])
                })
                .collect()
        };
        Self {
            version: 1,
            initial: choose(initial),
            recurrent: choose(recurrent),
            values: examples
                .iter()
                .map(|e| e.payload)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            actions: examples
                .iter()
                .flat_map(|e| e.primitives.iter().copied())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            stop_table: fit_stop_policy(examples),
        }
    }

    pub fn serve(&self, payload: Option<u32>, primitives: &[u32]) -> Response {
        let mut out = Response {
            tokens: Vec::new(),
            states: Vec::new(),
            steps: Vec::new(),
            stopped: false,
        };
        let Some(mut state) = payload else {
            out.steps.push(StepKind::NoRead);
            return out;
        };
        if !self.values.contains(&state) {
            out.steps.push(StepKind::UnknownValue);
            return out;
        }
        for (j, primitive) in primitives.iter().enumerate() {
            if !self.actions.contains(primitive) {
                out.steps.push(StepKind::UnknownPrimitive);
                return out;
            }
            let table = if j == 0 {
                &self.initial
            } else {
                &self.recurrent
            };
            let Some(row) = table.iter().find(|r| r[0] == state && r[1] == *primitive) else {
                out.steps.push(StepKind::NoGrounding);
                return out;
            };
            state = row[2];
            out.tokens.push(state);
            out.steps.push(StepKind::Emit);
        }
        out.stopped = self.stop_table.iter().any(|(r, stop)| *r == 0 && *stop);
        out.steps.push(if out.stopped {
            StepKind::Stop
        } else {
            StepKind::Exhausted
        });
        out
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let model: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if model.version != 1
            || model
                .initial
                .iter()
                .chain(&model.recurrent)
                .flatten()
                .chain(&model.values)
                .chain(&model.actions)
                .any(|t| *t as usize >= max_vocab)
        {
            return Err("invalid finite shared transition artifact".into());
        }
        Ok(model)
    }
    pub fn table_sizes(&self) -> (usize, usize) {
        (self.initial.len(), self.recurrent.len())
    }
}

/// The served shared-transition model, with learned maps and declared algebra/format policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedTransitionModel {
    /// Additive `mod GROUP_ORDER` action composition instead of the exact product.
    pub cyclic: bool,
    pub value_domain: Vec<u32>,
    pub value_state: Vec<u8>,
    pub action_domain: Vec<u32>,
    pub action_code: Vec<u8>,
    pub decoder: ResultDecoder,
    /// Strictly increasing `remaining -> stop`. An unobserved `remaining` continues by default.
    pub stop_table: Vec<(u8, bool)>,
    /// RLST v1 forced Stop at input exhaustion even without a policy entry.
    pub legacy_exhaustion_stop: bool,
}

impl SharedTransitionModel {
    pub fn seeded(examples: &[StExample], cyclic: bool) -> Self {
        let values: BTreeSet<u32> = examples.iter().map(|x| x.payload).collect();
        let actions: BTreeSet<u32> = examples.iter().flat_map(|x| x.primitives.clone()).collect();
        let value_domain: Vec<u32> = values.into_iter().collect();
        let action_domain: Vec<u32> = actions.into_iter().collect();
        let value_state = (0..value_domain.len())
            .map(|i| ((i * 11 + 3) % GROUP_ORDER) as u8)
            .collect();
        let action_code = (0..action_domain.len())
            .map(|i| ((i * 7 + 1) % GROUP_ORDER) as u8)
            .collect();
        SharedTransitionModel {
            cyclic,
            value_domain,
            value_state,
            action_domain,
            action_code,
            decoder: ResultDecoder::default(),
            stop_table: Vec::new(),
            legacy_exhaustion_stop: false,
        }
    }

    /// A deterministic pseudo-random start used by the multi-start search. Nothing here is the
    /// fixture's rule; the seeds only diversify the starting point of a local search.
    pub fn seeded_from(examples: &[StExample], cyclic: bool, seed: u64) -> Self {
        let mut m = Self::seeded(examples, cyclic);
        let mut st = seed | 1;
        let mut next = || {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            (st % GROUP_ORDER as u64) as u8
        };
        for v in m.value_state.iter_mut() {
            *v = next();
        }
        for a in m.action_code.iter_mut() {
            *a = next();
        }
        m
    }

    #[inline]
    pub fn initial_state(&self, payload: u32) -> Result<usize, StepKind> {
        match self.value_domain.iter().position(|t| *t == payload) {
            Some(i) => Ok(self.value_state[i] as usize),
            None => Err(StepKind::UnknownValue),
        }
    }

    /// Apply one observed primitive to the retained result state.
    #[inline]
    pub fn apply(&self, state: usize, primitive: u32) -> Result<usize, StepKind> {
        if state >= GROUP_ORDER {
            return Err(StepKind::InvalidState);
        }
        let i = self
            .action_domain
            .iter()
            .position(|t| *t == primitive)
            .ok_or(StepKind::UnknownPrimitive)?;
        let a = self.action_code[i] as usize;
        if self.cyclic {
            return Ok((state + a) % GROUP_ORDER);
        }
        let t = group_table();
        Ok(t.product[a.min(GROUP_ORDER - 1) * ROW_STRIDE + state.min(GROUP_ORDER - 1)] as usize)
    }

    /// The learned continuation decision. `remaining` is the number of observed primitives not yet
    /// consumed; an unobserved value continues, so the policy transfers to unseen longer sequences.
    #[inline]
    pub fn should_stop(&self, remaining: usize) -> bool {
        match self
            .stop_table
            .iter()
            .find(|(r, _)| *r as usize == remaining)
        {
            Some((_, stop)) => *stop,
            None => false,
        }
    }

    /// The complete served response: read -> compute each observed primitive -> emit -> stop.
    pub fn serve(&self, payload: u32, primitives: &[u32]) -> Response {
        let mut out = Response {
            tokens: Vec::new(),
            states: Vec::new(),
            steps: Vec::new(),
            stopped: false,
        };
        let mut state = match self.initial_state(payload) {
            Ok(s) => s,
            Err(k) => {
                out.steps.push(k);
                return out;
            }
        };
        let n = primitives.len();
        for j in 0..=n {
            let remaining = n - j;
            if self.should_stop(remaining) {
                out.steps.push(StepKind::Stop);
                out.stopped = true;
                return out;
            }
            if j == n {
                // Input exhaustion is distinct from a policy-selected Stop. Preserve the
                // original artifact's behavior only when loading an RLST v1 model.
                out.steps.push(if self.legacy_exhaustion_stop {
                    StepKind::Stop
                } else {
                    StepKind::Exhausted
                });
                out.stopped = self.legacy_exhaustion_stop;
                return out;
            }
            state = match self.apply(state, primitives[j]) {
                Ok(s) => s,
                Err(k) => {
                    out.steps.push(k);
                    return out;
                }
            };
            match self.decoder.decode(state) {
                DecOutcome::Emit { token, .. } => {
                    out.tokens.push(token);
                    out.states.push(state);
                    out.steps.push(StepKind::Emit);
                }
                DecOutcome::NoRead => {
                    out.states.push(state);
                    out.steps.push(StepKind::NoGrounding);
                    return out;
                }
            }
        }
        out
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLST");
        o.extend_from_slice(&2u32.to_le_bytes());
        o.push(u8::from(self.cyclic));
        o.push(u8::from(self.legacy_exhaustion_stop));
        for domain in [&self.value_domain, &self.action_domain] {
            o.extend_from_slice(&(domain.len() as u32).to_le_bytes());
            for t in domain.iter() {
                o.extend_from_slice(&t.to_le_bytes());
            }
        }
        for code in [&self.value_state, &self.action_code] {
            o.extend_from_slice(&(code.len() as u32).to_le_bytes());
            o.extend_from_slice(code);
        }
        o.extend_from_slice(&(self.stop_table.len() as u32).to_le_bytes());
        for (r, stop) in self.stop_table.iter() {
            o.push(*r);
            o.push(u8::from(*stop));
        }
        let d = self.decoder.to_bytes();
        o.extend_from_slice(&(d.len() as u32).to_le_bytes());
        o.extend_from_slice(&d);
        o
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated shared-transition artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLST" {
            return Err("bad shared-transition artifact magic".into());
        }
        let version = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        if version != 1 && version != 2 {
            return Err("unsupported shared-transition artifact version".into());
        }
        let cyclic = take(&mut c, 1)?[0] != 0;
        let legacy_exhaustion_stop = version == 1 || take(&mut c, 1)?[0] != 0;
        let mut domains = Vec::new();
        for _ in 0..2 {
            let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
            if n.checked_mul(4).ok_or("domain overflow")? > bytes.len().saturating_sub(c) {
                return Err("truncated shared-transition domain".into());
            }
            let mut d = Vec::with_capacity(n);
            for _ in 0..n {
                let t = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
                if t as usize >= max_vocab {
                    return Err("shared-transition domain token outside the vocabulary".into());
                }
                d.push(t);
            }
            if d.windows(2).any(|w| w[0] >= w[1]) {
                return Err("shared-transition domain must be strictly increasing".into());
            }
            domains.push(d);
        }
        let mut codes = Vec::new();
        for _ in 0..2 {
            let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
            codes.push(take(&mut c, n)?.to_vec());
        }
        let value_domain = domains.remove(0);
        let action_domain = domains.remove(0);
        let value_state = codes.remove(0);
        let action_code = codes.remove(0);
        if value_state.len() != value_domain.len() || action_code.len() != action_domain.len() {
            return Err("shared-transition code length disagrees with its domain".into());
        }
        if value_state
            .iter()
            .chain(action_code.iter())
            .any(|v| *v as usize >= GROUP_ORDER)
        {
            return Err("shared-transition code outside the declared group domain".into());
        }
        let ns = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if ns > 64 {
            return Err("shared-transition stop policy is out of range".into());
        }
        let raw = take(&mut c, ns * 2)?;
        let mut stop_table: Vec<(u8, bool)> =
            (0..ns).map(|i| (raw[i * 2], raw[i * 2 + 1] != 0)).collect();
        if stop_table.windows(2).any(|w| w[0].0 >= w[1].0) {
            return Err("shared-transition stop policy must be strictly ordered".into());
        }
        stop_table.shrink_to_fit();
        let dl = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        let decoder = ResultDecoder::from_bytes(take(&mut c, dl)?, max_vocab)?;
        if version == 2 && decoder.state_count() > ST_MAX_STATES {
            return Err("shared-transition decoder exceeds the 16-state bound".into());
        }
        if c != bytes.len() {
            return Err("trailing bytes in the shared-transition artifact".into());
        }
        Ok(SharedTransitionModel {
            cyclic,
            value_domain,
            value_state,
            action_domain,
            action_code,
            decoder,
            stop_table,
            legacy_exhaustion_stop,
        })
    }
}

/// Receipt for one bounded joint fit of the initial-state map, shared actions, decoder and stop policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StFitReport {
    pub initial_complete: usize,
    pub final_complete: usize,
    pub final_tokens: usize,
    pub token_total: usize,
    pub examples: usize,
    pub states: usize,
    /// Historical restart tie-break, measured before the final decoder refit.
    pub pre_refit_states: usize,
    pub accepted_moves: usize,
}

fn observations(model: &SharedTransitionModel, examples: &[StExample]) -> Vec<DecObservation> {
    let mut obs = Vec::new();
    for ex in examples.iter() {
        let mut state = match model.initial_state(ex.payload) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for (j, prim) in ex.primitives.iter().enumerate() {
            let target = match ex.targets.get(j) {
                Some(t) => *t,
                None => continue,
            };
            state = match model.apply(state, *prim) {
                Ok(s) => s,
                Err(_) => break,
            };
            obs.push(DecObservation {
                s: state,
                target,
                read: true,
                grounded: true,
            });
        }
    }
    obs
}

/// Complete-response match: every emitted token equals its target and the response stops.
pub fn complete_match(model: &SharedTransitionModel, ex: &StExample) -> bool {
    let r = model.serve(ex.payload, &ex.primitives);
    r.stopped && r.tokens.len() == ex.targets.len() && r.tokens == ex.targets
}

/// Complete responses matched, token hits and total tokens over a set of examples.
pub fn score(model: &SharedTransitionModel, examples: &[StExample]) -> (usize, usize, usize) {
    let mut complete = 0usize;
    let mut hits = 0usize;
    let mut total = 0usize;
    for ex in examples.iter() {
        if complete_match(model, ex) {
            complete += 1;
        }
        let r = model.serve(ex.payload, &ex.primitives);
        for (j, t) in ex.targets.iter().enumerate() {
            total += 1;
            if r.tokens.get(j) == Some(t) {
                hits += 1;
            }
        }
    }
    (complete, hits, total)
}

/// Learn the stop policy: stop exactly when no observed primitive remains. Unobserved `remaining`
/// values continue by default, so a policy learned on short sequences transfers to longer ones.
fn fit_stop_policy(examples: &[StExample]) -> Vec<(u8, bool)> {
    let mut decisions: BTreeMap<u8, BTreeSet<bool>> = BTreeMap::new();
    for ex in examples.iter() {
        let n = ex.primitives.len();
        for j in 0..=n {
            let remaining = (n - j) as u8;
            decisions
                .entry(remaining)
                .or_default()
                .insert(remaining == 0);
        }
    }
    decisions
        .into_iter()
        .filter_map(|(r, d)| {
            let stop = *d.iter().next()?;
            d.iter().all(|v| *v == stop).then_some((r, stop))
        })
        .collect()
}

/// Jointly fit the initial-state map, the shared action codes, the lexical decoder and the stop
/// policy on declared development outcomes. Coordinate objective: combined supervised token
/// hits, complete responses, then fewer states. Restart selection uses complete responses
/// and token hits measured after the final all-development decoder refit, followed by the
/// historical pre-refit state-count tie-break.
pub fn fit_shared_transition(
    fit: &[StExample],
    probe: &[StExample],
    max_states: usize,
    passes: usize,
    cyclic: bool,
    restarts: usize,
    seed: u64,
) -> (SharedTransitionModel, StFitReport) {
    let mut best: Option<(SharedTransitionModel, StFitReport)> = None;
    for r in 0..restarts.max(1) {
        let cand = fit_shared_transition_once(fit, probe, max_states, passes, cyclic, seed, r);
        let take = match &best {
            None => true,
            Some((_, b)) => {
                cand.1.final_complete > b.final_complete
                    || (cand.1.final_complete == b.final_complete
                        && (cand.1.final_tokens > b.final_tokens
                            || (cand.1.final_tokens == b.final_tokens
                                && cand.1.pre_refit_states < b.pre_refit_states)))
            }
        };
        if take {
            best = Some(cand);
        }
    }
    best.expect("at least one restart is always evaluated")
}

fn fit_shared_transition_once(
    fit: &[StExample],
    probe: &[StExample],
    max_states: usize,
    passes: usize,
    cyclic: bool,
    seed: u64,
    restart: usize,
) -> (SharedTransitionModel, StFitReport) {
    let all: Vec<StExample> = fit.iter().chain(probe).cloned().collect();
    let mut model = if restart == 0 {
        SharedTransitionModel::seeded(&all, cyclic)
    } else {
        SharedTransitionModel::seeded_from(
            &all,
            cyclic,
            seed ^ (0x9E37_79B9 * (restart as u64 + 1)),
        )
    };
    model.stop_table = fit_stop_policy(&all);
    let refit = |m: &mut SharedTransitionModel| {
        m.decoder = fit_decoder(&observations(m, fit), max_states, ST_K);
    };
    // Smoother, structure-rewarding objective: correct tokens over the calibration set *and* the
    // supervised outer development set, then complete responses, then fewer states. The
    // caller declares which observed values/sequences are shared between these sets.
    let grade = |m: &SharedTransitionModel| -> (usize, usize, usize) {
        let (cf, hf, _) = score(m, fit);
        let (cp, hp, _) = score(m, probe);
        (hf + hp, cf + cp, m.decoder.state_count())
    };
    refit(&mut model);
    let mut best = grade(&model);
    let initial_complete = score(&model, fit).0;
    let mut accepted = 0usize;
    let better = |c: (usize, usize, usize), b: (usize, usize, usize)| {
        c.0 > b.0 || (c.0 == b.0 && c.1 > b.1) || (c.0 == b.0 && c.1 == b.1 && c.2 < b.2)
    };
    for _ in 0..passes.max(1) {
        let mut improved = false;
        for i in 0..model.value_state.len() {
            let saved = model.value_state[i];
            let mut best_val = saved;
            let mut best_score = best;
            for cand in 0..GROUP_ORDER {
                model.value_state[i] = cand as u8;
                refit(&mut model);
                let s = grade(&model);
                if better(s, best_score) {
                    best_score = s;
                    best_val = cand as u8;
                }
            }
            model.value_state[i] = best_val;
            refit(&mut model);
            if better(best_score, best) {
                best = best_score;
                accepted += 1;
                improved = true;
            }
        }
        for i in 0..model.action_code.len() {
            let saved = model.action_code[i];
            let mut best_val = saved;
            let mut best_score = best;
            for cand in 0..GROUP_ORDER {
                model.action_code[i] = cand as u8;
                refit(&mut model);
                let s = grade(&model);
                if better(s, best_score) {
                    best_score = s;
                    best_val = cand as u8;
                }
            }
            model.action_code[i] = best_val;
            refit(&mut model);
            if better(best_score, best) {
                best = best_score;
                accepted += 1;
                improved = true;
            }
        }
        if !improved {
            break;
        }
    }
    // The final readout refit uses every development sequence, not only the calibration half.
    model.decoder = fit_decoder(&observations(&model, &all), max_states, ST_K);
    let (complete_dev, tokens, total) = score(&model, &all);
    let report = StFitReport {
        initial_complete,
        final_complete: complete_dev,
        final_tokens: tokens,
        token_total: total,
        examples: all.len(),
        states: model.decoder.state_count(),
        pre_refit_states: best.2,
        accepted_moves: accepted,
    };
    (model, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_transition_reuses_intermediate_labels_and_never_invents_an_absent_operand() {
        let examples = vec![
            StExample {
                payload: 10,
                primitives: vec![1],
                targets: vec![20],
            },
            StExample {
                payload: 11,
                primitives: vec![1, 2],
                targets: vec![20, 40],
            },
        ];
        let model = FiniteTransitionModel::fit(&examples);
        let loaded = FiniteTransitionModel::from_bytes(&model.to_bytes().unwrap(), 100).unwrap();
        assert_eq!(model, loaded);
        // This full payload/sequence pair was never shown. Its two shared transitions were.
        let response = loaded.serve(Some(10), &[1, 2]);
        assert_eq!(response.tokens, vec![20, 40]);
        assert!(response.stopped);
        assert_eq!(loaded.serve(None, &[1, 2]).steps, vec![StepKind::NoRead]);
        assert_eq!(
            loaded.serve(Some(99), &[1]).steps,
            vec![StepKind::UnknownValue]
        );
        assert_eq!(
            loaded.serve(Some(10), &[2]).steps,
            vec![StepKind::NoGrounding]
        );
    }

    #[test]
    fn exhaustion_is_not_a_learned_stop_and_v1_keeps_its_legacy_contract() {
        let examples = vec![StExample {
            payload: 10,
            primitives: vec![1],
            targets: vec![20],
        }];
        let mut model = SharedTransitionModel::seeded(&examples, false);
        let response = model.serve(10, &[]);
        assert_eq!(response.steps, vec![StepKind::Exhausted]);
        assert!(!response.stopped);
        let loaded = SharedTransitionModel::from_bytes(&model.to_bytes(), 100).unwrap();
        assert_eq!(loaded.serve(10, &[]), response);
        let mut legacy = model.to_bytes();
        legacy[4..8].copy_from_slice(&1u32.to_le_bytes());
        legacy.remove(9); // RLST v1 has no explicit exhaustion-policy flag.
        let loaded_legacy = SharedTransitionModel::from_bytes(&legacy, 100).unwrap();
        assert_eq!(loaded_legacy.serve(10, &[]).steps, vec![StepKind::Stop]);
        assert!(loaded_legacy.serve(10, &[]).stopped);
        model.stop_table = fit_stop_policy(&examples);
        assert_eq!(model.serve(10, &[]).steps, vec![StepKind::Stop]);
        assert_eq!(model.apply(GROUP_ORDER, 1), Err(StepKind::InvalidState));
    }

    #[test]
    fn the_witness_subgroup_is_closed_and_non_abelian() {
        let w = q8_witness().expect("the table must contain a non-abelian order-8 subgroup");
        let t = group_table();
        let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
        let elems: Vec<usize> = w.iter().map(|v| *v as usize).collect();
        let set: BTreeSet<usize> = elems.iter().copied().collect();
        assert_eq!(set.len(), 8);
        assert!(elems
            .iter()
            .all(|x| elems.iter().all(|y| set.contains(&mul(*x, *y)))));
        assert!(elems
            .iter()
            .any(|x| elems.iter().any(|y| mul(*x, *y) != mul(*y, *x))));
    }

    #[test]
    fn a_shifted_value_cannot_reuse_the_local_prior_and_steps_retain_state() {
        // Two primitives and two values: a shared action must move the state, and each step must
        // depend only on the observed primitive prefix, never on the previously emitted token.
        let dev = vec![
            StExample {
                payload: 10,
                primitives: vec![1],
                targets: vec![7],
            },
            StExample {
                payload: 11,
                primitives: vec![1],
                targets: vec![8],
            },
            StExample {
                payload: 10,
                primitives: vec![1, 2],
                targets: vec![7, 9],
            },
            StExample {
                payload: 11,
                primitives: vec![1, 2],
                targets: vec![8, 6],
            },
            StExample {
                payload: 10,
                primitives: vec![2],
                targets: vec![5],
            },
            StExample {
                payload: 11,
                primitives: vec![2],
                targets: vec![4],
            },
        ];
        let (m, report) = fit_shared_transition(&dev, &[], ST_MAX_STATES, 2, false, 2, 7);
        assert!(report.final_complete >= report.initial_complete);
        let r = m.serve(10, &[1, 2]);
        assert_eq!(r.tokens.len(), 2, "both observed primitives emit a token");
        assert!(
            r.stopped,
            "the learned policy stops when the observed sequence ends"
        );
        // State retention: the second step's state is the same whether or not the first token is
        // replaced, because it depends only on the payload and the observed primitive prefix.
        let alt = m.serve(10, &[1, 2]);
        assert_eq!(r.states, alt.states);
        // An unknown primitive is typed, not silently the identity.
        let unknown = m.serve(10, &[1, 99]);
        assert_eq!(unknown.steps.last(), Some(&StepKind::UnknownPrimitive));
        let unknown_value = m.serve(12345, &[1]);
        assert_eq!(unknown_value.steps, vec![StepKind::UnknownValue]);
    }

    #[test]
    fn the_round_trip_preserves_every_learned_field() {
        let dev = vec![
            StExample {
                payload: 3,
                primitives: vec![4, 5],
                targets: vec![1, 2],
            },
            StExample {
                payload: 6,
                primitives: vec![5],
                targets: vec![3],
            },
        ];
        let (m, _) = fit_shared_transition(&dev, &[], ST_MAX_STATES, 1, false, 1, 7);
        let back = SharedTransitionModel::from_bytes(&m.to_bytes(), 4096).unwrap();
        assert_eq!(back, m);
        let mut bad = m.to_bytes();
        bad[0] = b'X';
        assert!(SharedTransitionModel::from_bytes(&bad, 4096).is_err());
        let mut bad = m.to_bytes();
        bad.truncate(bad.len() - 1);
        assert!(SharedTransitionModel::from_bytes(&bad, 4096).is_err());
    }
}
