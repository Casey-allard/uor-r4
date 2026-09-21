//! A learned lexical decoder over the **relative computed result** of one geometric read.
//!
//! The residual readout scores `F(q1) - F(q0)`. For any finite group and any `F`, right
//! multiplication permutes the group, so `sum_g (F(g*a) - F(g)) = 0`: a difference-of-potentials
//! readout cannot supply a strictly positive frame-independent boost across a whole action orbit.
//! This module instead decodes the **relative result**
//!
//! ```text
//! q0 = bounded causal query state (frozen)
//! q1 = (q0 * T_bind[r]) * U_op[query] * V[value]     exact finite products
//! s  = inverse(q0) * q1 = T_bind[r] * U_op[query] * V[value]
//! emit = decoder[s]                                  learned token shortlist
//! ```
//!
//! `s` is the frame-independent content of the update. A **valid identity** `s = e` is an ordinary
//! emittable result and is deliberately distinct from `NoRead`: `NoRead` is the absence of a
//! grounded read or of a learned state, and it preserves the frozen local prior. Nothing here
//! replaces an old zero-residual artifact; this is a new, separately versioned interface.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};

/// Shortlist size stored per grounded state.
pub const DEC_K: usize = 4;
/// Bounded number of grounded states the decoder may carry. This is the resource constraint that
/// limits memorisation capacity; it does not itself force correct structural sharing.
pub const DEC_MAX_STATES: usize = 16;

/// Result of one decoder lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecOutcome {
    /// No grounded read, or no learned state for the relative result: keep the local prior.
    NoRead,
    /// A learned emission for this relative result.
    Emit { token: u32, score: i32 },
}

/// One grounded relative-result state with its learned token shortlist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecState {
    pub s: u8,
    pub valid: bool,
    pub tokens: Vec<u32>,
    pub scores: Vec<i32>,
}

/// The bounded learned result-to-token decoder.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ResultDecoder {
    /// Sorted strictly by `s`.
    pub states: Vec<DecState>,
}

impl ResultDecoder {
    pub fn decode(&self, s: usize) -> DecOutcome {
        let s = s.min(GROUP_ORDER - 1) as u8;
        match self.states.binary_search_by_key(&s, |st| st.s) {
            Ok(i) => {
                let st = &self.states[i];
                if !st.valid || st.tokens.is_empty() {
                    return DecOutcome::NoRead;
                }
                let mut best = 0usize;
                for j in 1..st.tokens.len() {
                    if st.scores[j] > st.scores[best]
                        || (st.scores[j] == st.scores[best] && st.tokens[j] < st.tokens[best])
                    {
                        best = j;
                    }
                }
                DecOutcome::Emit {
                    token: st.tokens[best],
                    score: st.scores[best],
                }
            }
            Err(_) => DecOutcome::NoRead,
        }
    }

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLRD");
        o.extend_from_slice(&1u32.to_le_bytes());
        o.extend_from_slice(&(self.states.len() as u32).to_le_bytes());
        for st in self.states.iter() {
            o.push(st.s);
            o.push(u8::from(st.valid));
            o.extend_from_slice(&(st.tokens.len() as u16).to_le_bytes());
            for t in st.tokens.iter() {
                o.extend_from_slice(&t.to_le_bytes());
            }
            for c in st.scores.iter() {
                o.extend_from_slice(&c.to_le_bytes());
            }
        }
        o
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated decoder artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLRD" {
            return Err("bad decoder artifact magic".into());
        }
        if u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) != 1 {
            return Err("unsupported decoder artifact version".into());
        }
        let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if n > GROUP_ORDER {
            return Err("decoder state count exceeds the declared domain".into());
        }
        let mut states = Vec::with_capacity(n);
        for _ in 0..n {
            let s = take(&mut c, 1)?[0];
            if s as usize >= GROUP_ORDER {
                return Err("decoder state outside the group domain".into());
            }
            let valid = take(&mut c, 1)?[0] != 0;
            let nt = u16::from_le_bytes(take(&mut c, 2)?.try_into().unwrap()) as usize;
            if nt > DEC_K {
                return Err("decoder shortlist exceeds the declared bound".into());
            }
            let mut tokens = Vec::with_capacity(nt);
            for _ in 0..nt {
                let t = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
                if t as usize >= max_vocab {
                    return Err("decoder token outside the vocabulary".into());
                }
                tokens.push(t);
            }
            let mut scores = Vec::with_capacity(nt);
            for _ in 0..nt {
                scores.push(i32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()));
            }
            if !valid && !tokens.is_empty() {
                return Err("invalid decoder state carries a shortlist".into());
            }
            states.push(DecState {
                s,
                valid,
                tokens,
                scores,
            });
        }
        if c != bytes.len() {
            return Err("trailing bytes in the decoder artifact".into());
        }
        if states.windows(2).any(|w| w[0].s >= w[1].s) {
            return Err("decoder states must be strictly ordered".into());
        }
        Ok(ResultDecoder { states })
    }
}

/// One supervised observation of the relative result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecObservation {
    pub s: usize,
    pub target: u32,
    /// Factual reader action, separate from intended-source supervision.
    pub read: bool,
    /// The intended source was actually selected, so this observation may supervise the decoder.
    pub grounded: bool,
}

/// Fit a bounded decoder from development outcomes: rank grounded states by support, keep at most
/// `max_states`, and store the `k` most frequent targets with their counts as integer scores.
pub fn fit_decoder(obs: &[DecObservation], max_states: usize, k: usize) -> ResultDecoder {
    let mut counts: BTreeMap<u8, BTreeMap<u32, i32>> = BTreeMap::new();
    for o in obs.iter().filter(|o| o.read && o.grounded) {
        *counts
            .entry(o.s.min(GROUP_ORDER - 1) as u8)
            .or_default()
            .entry(o.target)
            .or_default() += 1;
    }
    let mut ranked: Vec<(i32, u8)> = counts
        .iter()
        .map(|(s, m)| (m.values().sum::<i32>(), *s))
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    ranked.truncate(max_states.min(DEC_MAX_STATES));
    let mut states = Vec::new();
    for (_support, s) in ranked {
        let m = &counts[&s];
        let mut entries: Vec<(i32, u32)> = m.iter().map(|(t, c)| (*c, *t)).collect();
        entries.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        entries.truncate(k.min(DEC_K));
        let tokens = entries.iter().map(|(_, t)| *t).collect();
        let scores = entries.iter().map(|(c, _)| *c).collect();
        states.push(DecState {
            s,
            valid: true,
            tokens,
            scores,
        });
    }
    states.sort_by_key(|st| st.s);
    ResultDecoder { states }
}

/// Correct and total counts for the decoder over a set of observations (grounded or not).
pub fn decode_hits(dec: &ResultDecoder, obs: &[DecObservation]) -> (usize, usize) {
    let mut correct = 0usize;
    for o in obs.iter() {
        if o.read {
            if let DecOutcome::Emit { token, .. } = dec.decode(o.s) {
                if token == o.target {
                    correct += 1;
                }
            }
        }
    }
    (correct, obs.len())
}

/// One development example of the served derived state: the two observable inputs plus the
/// selected payload, with the intended-source flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecExample {
    pub r: usize,
    pub qrole: u32,
    pub payload: u32,
    pub target: u32,
    pub read: bool,
    pub grounded: bool,
}

/// The served derived-state model. `transport` is keyed by the binding relation, `op_code` by the
/// **observed query role token**, `value_code` by the selected payload token. The operation is read
/// from the actual causal prefix, never from a fixture label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedStateModel {
    pub transport: Vec<u8>,
    /// Additive `mod GROUP_ORDER` combination instead of the exact group product.
    pub cyclic: bool,
    /// Explicit factor contract: value-only decoding deliberately ignores query and binding.
    pub value_only: bool,
    /// v2 rejects unseen active operands; v1 loads retain their historical identity fallback.
    pub strict_operands: bool,
    pub op_domain: Vec<u32>,
    pub op_code: Vec<u8>,
    pub value_domain: Vec<u32>,
    pub value_code: Vec<u8>,
    pub decoder: ResultDecoder,
}

impl DerivedStateModel {
    /// Deterministic non-degenerate start. Every served value is subsequently learned; nothing here
    /// is the fixture's rule.
    pub fn seeded(examples: &[DecExample], cyclic: bool) -> Self {
        let rels: BTreeSet<usize> = examples.iter().map(|e| e.r % GROUP_ORDER).collect();
        let ops: BTreeSet<u32> = examples.iter().map(|e| e.qrole).collect();
        let values: BTreeSet<u32> = examples.iter().map(|e| e.payload).collect();
        let neutral = if cyclic { 0 } else { group_table().identity };
        let mut transport = vec![neutral; GROUP_ORDER];
        for r in rels {
            transport[r] = (r % GROUP_ORDER) as u8;
        }
        let op_domain: Vec<u32> = ops.into_iter().collect();
        let value_domain: Vec<u32> = values.into_iter().collect();
        let op_code = (0..op_domain.len())
            .map(|i| ((i * 7 + 1) % GROUP_ORDER) as u8)
            .collect();
        let value_code = (0..value_domain.len())
            .map(|j| ((j * 11 + 3) % GROUP_ORDER) as u8)
            .collect();
        DerivedStateModel {
            transport,
            cyclic,
            value_only: false,
            strict_operands: true,
            op_domain,
            op_code,
            value_domain,
            value_code,
            decoder: ResultDecoder::default(),
        }
    }

    #[inline]
    pub fn op_state(&self, qrole: u32) -> usize {
        match self.op_domain.iter().position(|t| *t == qrole) {
            Some(i) => self.op_code[i] as usize,
            None => self.neutral_element(),
        }
    }

    #[inline]
    pub fn value_state(&self, payload: u32) -> usize {
        match self.value_domain.iter().position(|t| *t == payload) {
            Some(i) => self.value_code[i] as usize,
            None => self.neutral_element(),
        }
    }

    fn neutral_element(&self) -> usize {
        if self.cyclic && self.strict_operands {
            0
        } else {
            group_table().identity as usize
        }
    }

    /// `s = inverse(q0) * q1 = T_bind[r] * U_op[query] * V[value]`, exact ordered products.
    #[inline]
    pub fn relative_result(&self, r: usize, qrole: u32, payload: u32) -> usize {
        if self.value_only {
            return self.value_state(payload);
        }
        let t = group_table();
        let a = self
            .transport
            .get(r % GROUP_ORDER)
            .copied()
            .unwrap_or(self.neutral_element() as u8) as usize;
        let o = self.op_state(qrole).min(GROUP_ORDER - 1);
        let v = self.value_state(payload).min(GROUP_ORDER - 1);
        if self.cyclic {
            return (a + o + v) % GROUP_ORDER;
        }
        let m = t.product[a.min(GROUP_ORDER - 1) * ROW_STRIDE + o] as usize;
        t.product[m * ROW_STRIDE + v] as usize
    }

    pub fn decode(&self, r: usize, qrole: u32, payload: u32) -> DecOutcome {
        match self.checked_relative_result(r, qrole, payload) {
            Ok(s) => self.decoder.decode(s),
            Err(_) => DecOutcome::NoRead,
        }
    }

    /// Distinguish an unknown active operand from a legitimate computed identity.
    pub fn checked_relative_result(
        &self,
        r: usize,
        qrole: u32,
        payload: u32,
    ) -> Result<usize, &'static str> {
        if self.strict_operands {
            if !self.value_only && !self.op_domain.contains(&qrole) {
                return Err("UnknownOperation");
            }
            if !self.value_domain.contains(&payload) {
                return Err("UnknownValue");
            }
        }
        Ok(self.relative_result(r, qrole, payload))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLDS");
        o.extend_from_slice(&2u32.to_le_bytes());
        o.extend_from_slice(&(self.transport.len() as u32).to_le_bytes());
        o.extend_from_slice(&self.transport);
        o.push(u8::from(self.cyclic));
        o.push(u8::from(self.value_only));
        o.push(u8::from(self.strict_operands));
        for domain in [&self.op_domain, &self.value_domain] {
            o.extend_from_slice(&(domain.len() as u32).to_le_bytes());
            for t in domain.iter() {
                o.extend_from_slice(&t.to_le_bytes());
            }
        }
        for code in [&self.op_code, &self.value_code] {
            o.extend_from_slice(&(code.len() as u32).to_le_bytes());
            o.extend_from_slice(code);
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
                return Err("truncated derived-state artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLDS" {
            return Err("bad derived-state artifact magic".into());
        }
        let version = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        if version != 1 && version != 2 {
            return Err("unsupported derived-state artifact version".into());
        }
        let nt = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if nt != GROUP_ORDER {
            return Err("derived-state transport has the wrong row count".into());
        }
        let transport = take(&mut c, nt)?.to_vec();
        let cyclic = take(&mut c, 1)?[0] != 0;
        let (value_only, strict_operands) = if version == 2 {
            let flags = take(&mut c, 2)?;
            if flags.iter().any(|f| *f > 1) {
                return Err("invalid derived-state factor/operand flags".into());
            }
            (flags[0] != 0, flags[1] != 0)
        } else {
            (false, false)
        };
        let mut domains = Vec::new();
        for _ in 0..2 {
            let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
            let minimum = n.checked_mul(4).ok_or("domain size overflow")?;
            if minimum > bytes.len().saturating_sub(c) {
                return Err("truncated derived-state domain".into());
            }
            let mut d = Vec::with_capacity(n);
            for _ in 0..n {
                let t = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
                if t as usize >= max_vocab {
                    return Err("derived-state domain token outside the vocabulary".into());
                }
                d.push(t);
            }
            if d.windows(2).any(|w| w[0] >= w[1]) {
                return Err("derived-state domain must be strictly increasing".into());
            }
            domains.push(d);
        }
        let mut codes = Vec::new();
        for _ in 0..2 {
            let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
            codes.push(take(&mut c, n)?.to_vec());
        }
        let op_domain = domains.remove(0);
        let value_domain = domains.remove(0);
        let op_code = codes.remove(0);
        let value_code = codes.remove(0);
        if op_code.len() != op_domain.len() || value_code.len() != value_domain.len() {
            return Err("derived-state code length disagrees with its domain".into());
        }
        if transport
            .iter()
            .chain(op_code.iter())
            .chain(value_code.iter())
            .any(|v| *v as usize >= GROUP_ORDER)
        {
            return Err("derived-state element outside the declared group domain".into());
        }
        let dl = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        let decoder = ResultDecoder::from_bytes(take(&mut c, dl)?, max_vocab)?;
        if version == 2 && decoder.state_count() > DEC_MAX_STATES {
            return Err("v2 derived decoder exceeds its state bound".into());
        }
        if c != bytes.len() {
            return Err("trailing bytes in the derived-state artifact".into());
        }
        Ok(DerivedStateModel {
            transport,
            cyclic,
            value_only,
            strict_operands,
            op_domain,
            op_code,
            value_domain,
            value_code,
            decoder,
        })
    }
}

/// Receipt for one bounded jointfit of the maps and the decoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecFitReport {
    pub initial_hits: usize,
    pub final_hits: usize,
    pub examples: usize,
    /// Supervised outer-development cells queried during every candidate-map evaluation.
    pub probe_hits: usize,
    pub probe_examples: usize,
    pub states: usize,
    pub accepted_moves: usize,
}

fn observe(model: &DerivedStateModel, examples: &[DecExample]) -> Vec<DecObservation> {
    examples
        .iter()
        .map(|e| DecObservation {
            s: model.relative_result(e.r, e.qrole, e.payload),
            target: e.target,
            read: e.read,
            grounded: e.grounded,
        })
        .collect()
}

/// Jointly fit the binding/operation/value maps **and** the decoder on development outcomes.
///
/// `fit` supplies the decoder supervision; `probe` is a disjoint set of development cells whose
/// classes are covered by `fit`. Both sets are supervised development: outer labels are queried
/// on every candidate map, so their accuracy is not held-out generalization. Objective,
/// lexicographic: outer hits, then fit hits, then fewer grounded states. This can encourage sharing
/// but does not prove compositional identification. `value_only` keeps the
/// binding and operation factors at the identity, so only factors the data justifies are retained.
pub fn fit_derived_state_model(
    fit: &[DecExample],
    probe: &[DecExample],
    max_states: usize,
    passes: usize,
    cyclic: bool,
    value_only: bool,
) -> (DerivedStateModel, DecFitReport) {
    // Initialize active operand domains from all supervised development observations. In
    // particular, an outer-only operation must have its own optimizable code, not identity fallback.
    let development: Vec<DecExample> = fit.iter().chain(probe).copied().collect();
    let mut model = DerivedStateModel::seeded(&development, cyclic);
    model.value_only = value_only;
    if value_only {
        model.transport = vec![model.neutral_element() as u8; GROUP_ORDER];
        model.op_code = vec![model.neutral_element() as u8; model.op_domain.len()];
    }
    let refit = |m: &mut DerivedStateModel| {
        m.decoder = fit_decoder(&observe(m, fit), max_states, DEC_K);
    };
    let score = |m: &DerivedStateModel| -> (usize, usize, usize) {
        let fh = decode_hits(&m.decoder, &observe(m, fit)).0;
        let ph = decode_hits(&m.decoder, &observe(m, probe)).0;
        (fh, ph, m.decoder.state_count())
    };
    refit(&mut model);
    let mut best = score(&model);
    let initial_hits = best.0;
    let mut accepted = 0usize;
    // Supervised outer objective first, then calibration fit, then fewer states.
    let better = |c: (usize, usize, usize), b: (usize, usize, usize)| {
        c.1 > b.1 || (c.1 == b.1 && c.0 > b.0) || (c.1 == b.1 && c.0 == b.0 && c.2 < b.2)
    };
    for _ in 0..passes.max(1) {
        let mut improved = false;
        let rels: BTreeSet<usize> = development.iter().map(|e| e.r % GROUP_ORDER).collect();
        if !value_only {
            for r in rels {
                if r >= model.transport.len() {
                    continue;
                }
                let saved = model.transport[r];
                let mut best_val = saved;
                let mut best_score = best;
                for cand in 0..GROUP_ORDER {
                    model.transport[r] = cand as u8;
                    refit(&mut model);
                    let s = score(&model);
                    if better(s, best_score) {
                        best_score = s;
                        best_val = cand as u8;
                    }
                }
                model.transport[r] = best_val;
                refit(&mut model);
                if better(best_score, best) {
                    best = best_score;
                    accepted += 1;
                    improved = true;
                }
            }
            for i in 0..model.op_code.len() {
                let saved = model.op_code[i];
                let mut best_val = saved;
                let mut best_score = best;
                for cand in 0..GROUP_ORDER {
                    model.op_code[i] = cand as u8;
                    refit(&mut model);
                    let s = score(&model);
                    if better(s, best_score) {
                        best_score = s;
                        best_val = cand as u8;
                    }
                }
                model.op_code[i] = best_val;
                refit(&mut model);
                if better(best_score, best) {
                    best = best_score;
                    accepted += 1;
                    improved = true;
                }
            }
        }
        for j in 0..model.value_code.len() {
            let saved = model.value_code[j];
            let mut best_val = saved;
            let mut best_score = best;
            for cand in 0..GROUP_ORDER {
                model.value_code[j] = cand as u8;
                refit(&mut model);
                let s = score(&model);
                if better(s, best_score) {
                    best_score = s;
                    best_val = cand as u8;
                }
            }
            model.value_code[j] = best_val;
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
    let report = DecFitReport {
        initial_hits,
        final_hits: best.0,
        examples: fit.len(),
        probe_hits: best.1,
        probe_examples: probe.len(),
        states: best.2,
        accepted_moves: accepted,
    };
    (model, report)
}

/// Hits and total for a model over examples.
pub fn model_hits(model: &DerivedStateModel, examples: &[DecExample]) -> (usize, usize) {
    let mut correct = 0usize;
    for e in examples.iter() {
        if e.read {
            if let DecOutcome::Emit { token, .. } = model.decode(e.r, e.qrole, e.payload) {
                if token == e.target {
                    correct += 1;
                }
            }
        }
    }
    (correct, examples.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(pairs: &[(usize, u32, bool)]) -> Vec<DecObservation> {
        pairs
            .iter()
            .map(|(s, t, g)| DecObservation {
                s: *s,
                target: *t,
                read: true,
                grounded: *g,
            })
            .collect()
    }

    #[test]
    fn decoder_round_trips_and_separates_identity_from_noread() {
        let identity = group_table().identity as usize;
        let d = fit_decoder(
            &obs(&[(identity, 7, true), (identity, 7, true), (5, 9, true)]),
            DEC_MAX_STATES,
            DEC_K,
        );
        assert_eq!(d.state_count(), 2);
        // The identity state is an ordinary emittable result, not NoRead.
        assert_eq!(d.decode(identity), DecOutcome::Emit { token: 7, score: 2 });
        // An unseen relative result is NoRead, so the local prior stands.
        assert_eq!(d.decode(11), DecOutcome::NoRead);
        let back = ResultDecoder::from_bytes(&d.to_bytes(), 32).unwrap();
        assert_eq!(back, d);
        let mut bad = d.to_bytes();
        bad[0] = b'X';
        assert!(ResultDecoder::from_bytes(&bad, 32).is_err());
        let mut bad = d.to_bytes();
        bad.truncate(bad.len() - 1);
        assert!(ResultDecoder::from_bytes(&bad, 32).is_err());
    }

    #[test]
    fn supervised_outer_operations_are_parameters_and_absent_reads_do_not_score() {
        let fit = [DecExample {
            r: 2,
            qrole: 10,
            payload: 20,
            target: 7,
            read: true,
            grounded: true,
        }];
        let outer = [DecExample {
            qrole: 11,
            ..fit[0]
        }];
        let (model, _) = fit_derived_state_model(&fit, &outer, DEC_MAX_STATES, 1, false, false);
        assert!(model.op_domain.contains(&10));
        assert!(model.op_domain.contains(&11));
        assert!(model.checked_relative_result(2, 11, 20).is_ok());
        assert_eq!(
            model.checked_relative_result(2, 12, 20),
            Err("UnknownOperation")
        );
        assert_eq!(
            model_hits(
                &model,
                &[DecExample {
                    read: false,
                    ..fit[0]
                }]
            ),
            (0, 1)
        );
    }

    #[test]
    fn v2_distinguishes_unknown_operands_from_identity_and_v1_keeps_legacy_policy() {
        let e = group_table().identity;
        let example = DecExample {
            r: 2,
            qrole: 10,
            payload: 20,
            target: 7,
            read: true,
            grounded: true,
        };
        let mut model = DerivedStateModel::seeded(&[example], false);
        model.transport.fill(e);
        model.op_code.fill(e);
        model.value_code.fill(e);
        model.decoder = fit_decoder(&obs(&[(e as usize, 7, true)]), DEC_MAX_STATES, DEC_K);
        assert_eq!(
            model.decode(2, 10, 20),
            DecOutcome::Emit { token: 7, score: 1 }
        );
        assert_eq!(model.decode(2, 11, 20), DecOutcome::NoRead);
        assert_eq!(model.decode(2, 10, 21), DecOutcome::NoRead);
        let encoded = model.to_bytes();
        assert_eq!(DerivedStateModel::from_bytes(&encoded, 32).unwrap(), model);
        // v1 had only the cyclic flag at this location and deliberately used identity fallback.
        let mut old = encoded;
        old[4..8].copy_from_slice(&1u32.to_le_bytes());
        old.drain(13 + GROUP_ORDER..15 + GROUP_ORDER);
        let legacy = DerivedStateModel::from_bytes(&old, 32).unwrap();
        assert!(!legacy.strict_operands);
        assert_eq!(
            legacy.decode(2, 11, 20),
            DecOutcome::Emit { token: 7, score: 1 }
        );
        model.value_only = true;
        assert_eq!(
            model.decode(2, 999, 20),
            DecOutcome::Emit { token: 7, score: 1 }
        );
        assert_eq!(model.decode(2, 999, 21), DecOutcome::NoRead);
        assert_eq!(
            DerivedStateModel::from_bytes(&model.to_bytes(), 1024).unwrap(),
            model
        );
    }

    #[test]
    fn relative_result_is_the_frame_free_ordered_product() {
        let t = group_table();
        let mut ex = Vec::new();
        for r in 0..3usize {
            for (q, p) in [(10u32, 20u32), (11, 21)] {
                ex.push(DecExample {
                    r,
                    qrole: q,
                    payload: p,
                    target: 1,
                    read: true,
                    grounded: true,
                });
            }
        }
        let m = DerivedStateModel::seeded(&ex, false);
        // The relative result must equal inverse(q0) * ((q0 * T) * U) * V for any q0: the frame
        // cancels, which is exactly what makes the decoder frame-independent.
        for q0 in [0usize, 3, 77, 119] {
            for e in ex.iter() {
                let a = m.transport[e.r % GROUP_ORDER] as usize;
                let o = m.op_state(e.qrole);
                let v = m.value_state(e.payload);
                let q1 = t.product[t.product
                    [t.product[q0 * ROW_STRIDE + a] as usize * ROW_STRIDE + o]
                    as usize
                    * ROW_STRIDE
                    + v] as usize;
                let inv = t.inverse[q0] as usize;
                let s = t.product[inv * ROW_STRIDE + q1] as usize;
                assert_eq!(s, m.relative_result(e.r, e.qrole, e.payload));
            }
        }
        let back = DerivedStateModel::from_bytes(&m.to_bytes(), 4096).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn the_state_bound_and_fit_improve_development_hits() {
        // Ten required classes over many cells: a per-cell memorisation would need more states than
        // the bound allows, so the fit must share states.
        let values = [100u32, 101, 102, 103, 104, 105, 106, 107];
        let roles = [10u32, 11, 12, 13, 14, 15, 16, 17];
        let mut dev = Vec::new();
        for (op, role) in roles.iter().enumerate() {
            for (vi, value) in values.iter().enumerate() {
                dev.push(DecExample {
                    r: op % 2,
                    qrole: *role,
                    payload: *value,
                    target: (op + vi) as u32 % 10,
                    read: true,
                    grounded: true,
                });
            }
        }
        let (model, report) = fit_derived_state_model(&dev, &[], DEC_MAX_STATES, 3, false, false);
        assert!(report.final_hits >= report.initial_hits);
        assert!(report.final_hits > 0);
        assert!(model.decoder.state_count() <= DEC_MAX_STATES);
        let (hits, total) = model_hits(&model, &dev);
        assert_eq!(hits, report.final_hits);
        assert_eq!(total, dev.len());
    }
}
