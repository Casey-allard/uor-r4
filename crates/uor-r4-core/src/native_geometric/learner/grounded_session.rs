//! Grounded shared computation in an owned, resumable session.
//!
//! Three things are kept apart on purpose:
//!
//! * **exact evidence** - an immutable owned snapshot (sequence, absolute position, payload).
//!   Its origin can be checked against its owning ring; copied content survives origin eviction;
//! * **computed geometric value** - a retained result state advanced by shared primitive actions;
//! * **response phase and terminal status** - a typed, serializable frame, so a paused run and an
//!   uninterrupted run share the same transition.
//!
//! Part A of the mechanism is a *constructive factorization* of the observed transition graph.
//! The declared development labels give, for every observed primitive, a permutation of the typed
//! outcomes. When those permutations are bijective, distinct and closed they form a finite group,
//! and if that group is isomorphic to a subgroup of the served table we can bind
//!
//! ```text
//! Z[outcome] : typed outcome -> group element     (injective embedding)
//! A[primitive] = phi(permutation of that primitive)
//! E[payload]  = inverse(A[p]) * Z[first outcome of payload under p]
//! ```
//!
//! so that `Z[next] = A[primitive] * Z[current]` holds by construction rather than by search. The
//! factorization reads only declared training observations; it never reads the data generator's
//! codes, held-out labels or gold intermediate states.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};
use super::occurrence::{OccurrenceRef, OccurrenceRing};
use super::shared_transition::{Response, StExample, StepKind};

/// Typed terminal/degenerate status of a step or a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terminal {
    None,
    Stop,
    Exhausted,
    NoRead,
    UnknownValue,
    UnknownPrimitive,
    NoGrounding,
    StaleLease,
}

/// One bounded fitted factorization of the observed transition graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroundedFactorization {
    pub cyclic: bool,
    pub action_domain: Vec<u32>,
    pub action_code: Vec<u8>,
    pub outcome_domain: Vec<u32>,
    pub outcome_state: Vec<u8>,
    pub payload_domain: Vec<u32>,
    pub payload_state: Vec<u8>,
    pub reference_outcome: u32,
}

/// Receipt for one constructive factorization.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FactorReport {
    pub outcomes: usize,
    pub primitives: usize,
    pub group_order: usize,
    pub order_spectrum: Vec<(usize, usize)>,
    pub identity_primitive: Option<u32>,
    pub noncommuting_witness: Option<(u32, u32)>,
    pub isomorphic_candidates_checked: usize,
    pub transitions_checked: usize,
    pub transitions_consistent: usize,
    pub initial_checks: usize,
    pub initial_agreements: usize,
    pub gauge_fixed: bool,
}

fn compose(a: &[usize], b: &[usize]) -> Vec<usize> {
    // (a after b): applying b then a, matching `next = perm(primitive)[current]`.
    b.iter().map(|i| a[*i]).collect()
}

impl GroundedFactorization {
    pub fn outcome_index(&self, label: u32) -> Option<usize> {
        self.outcome_domain.iter().position(|t| *t == label)
    }

    pub fn state_for_outcome(&self, label: u32) -> Option<usize> {
        self.outcome_index(label)
            .map(|i| self.outcome_state[i] as usize)
    }

    pub fn outcome_for_state(&self, state: usize) -> Option<u32> {
        self.outcome_state
            .iter()
            .position(|s| *s as usize == state)
            .map(|i| self.outcome_domain[i])
    }

    pub fn initial_state(&self, payload: u32) -> Option<usize> {
        self.payload_domain
            .iter()
            .position(|t| *t == payload)
            .map(|i| self.payload_state[i] as usize)
    }

    pub fn apply(&self, state: usize, primitive: u32) -> Option<usize> {
        if state >= GROUP_ORDER {
            return None;
        }
        let code = self
            .action_domain
            .iter()
            .position(|t| *t == primitive)
            .map(|i| self.action_code[i] as usize)?;
        if self.cyclic {
            return Some((state + code) % GROUP_ORDER);
        }
        let t = group_table();
        Some(
            t.product[code.min(GROUP_ORDER - 1) * ROW_STRIDE + state.min(GROUP_ORDER - 1)] as usize,
        )
    }

    /// Complete served response for one ordered program. A missing outcome grounding, an unknown
    /// operand or an absent read stays typed rather than collapsing into a silent identity.
    pub fn serve(&self, payload: Option<u32>, primitives: &[u32]) -> Response {
        let mut out = Response {
            tokens: Vec::new(),
            states: Vec::new(),
            steps: Vec::new(),
            stopped: false,
        };
        let Some(payload) = payload else {
            out.steps.push(StepKind::NoRead);
            return out;
        };
        let Some(mut state) = self.initial_state(payload) else {
            out.steps.push(StepKind::UnknownValue);
            return out;
        };
        for p in primitives.iter() {
            state = match self.apply(state, *p) {
                Some(s) => s,
                None => {
                    out.steps.push(StepKind::UnknownPrimitive);
                    return out;
                }
            };
            match self.outcome_for_state(state) {
                Some(label) => {
                    out.tokens.push(label);
                    out.states.push(state);
                    out.steps.push(StepKind::Emit);
                }
                None => {
                    out.states.push(state);
                    out.steps.push(StepKind::NoGrounding);
                    return out;
                }
            }
        }
        // Completion here is explicit program exhaustion, reported as such.
        out.stopped = true;
        out.steps.push(StepKind::Stop);
        out
    }

    pub fn parameter_bytes(&self) -> usize {
        self.action_code.len()
            + self.outcome_state.len()
            + self.payload_state.len()
            + 4 * (self.action_domain.len() + self.outcome_domain.len() + self.payload_domain.len())
            + 2
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLGF");
        o.extend_from_slice(&1u32.to_le_bytes());
        o.push(u8::from(self.cyclic));
        o.extend_from_slice(&self.reference_outcome.to_le_bytes());
        for domain in [
            &self.action_domain,
            &self.outcome_domain,
            &self.payload_domain,
        ] {
            o.extend_from_slice(&(domain.len() as u32).to_le_bytes());
            for t in domain.iter() {
                o.extend_from_slice(&t.to_le_bytes());
            }
        }
        for code in [&self.action_code, &self.outcome_state, &self.payload_state] {
            o.extend_from_slice(&(code.len() as u32).to_le_bytes());
            o.extend_from_slice(code);
        }
        o
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated grounded-factorization artifact".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLGF" {
            return Err("bad grounded-factorization artifact magic".into());
        }
        if u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) != 1 {
            return Err("unsupported grounded-factorization artifact version".into());
        }
        let cyclic = take(&mut c, 1)?[0] != 0;
        let reference_outcome = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        let mut domains = Vec::new();
        for _ in 0..3 {
            let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
            if n.checked_mul(4).ok_or("domain overflow")? > bytes.len().saturating_sub(c) {
                return Err("truncated grounded-factorization domain".into());
            }
            let mut d = Vec::with_capacity(n);
            for _ in 0..n {
                let t = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
                if t as usize >= max_vocab {
                    return Err("grounded-factorization token outside the vocabulary".into());
                }
                d.push(t);
            }
            if d.windows(2).any(|w| w[0] >= w[1]) {
                return Err("grounded-factorization domain must be strictly increasing".into());
            }
            domains.push(d);
        }
        let mut codes = Vec::new();
        for _ in 0..3 {
            let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
            codes.push(take(&mut c, n)?.to_vec());
        }
        let action_domain = domains.remove(0);
        let outcome_domain = domains.remove(0);
        let payload_domain = domains.remove(0);
        let action_code = codes.remove(0);
        let outcome_state = codes.remove(0);
        let payload_state = codes.remove(0);
        if action_code.len() != action_domain.len()
            || outcome_state.len() != outcome_domain.len()
            || payload_state.len() != payload_domain.len()
        {
            return Err("grounded-factorization code length disagrees with its domain".into());
        }
        if action_code
            .iter()
            .chain(outcome_state.iter())
            .chain(payload_state.iter())
            .any(|v| *v as usize >= GROUP_ORDER)
        {
            return Err("grounded-factorization element outside the declared group domain".into());
        }
        if c != bytes.len() {
            return Err("trailing bytes in the grounded-factorization artifact".into());
        }
        let m = GroundedFactorization {
            cyclic,
            action_domain,
            action_code,
            outcome_domain,
            outcome_state,
            payload_domain,
            payload_state,
            reference_outcome,
        };
        if m.outcome_index(reference_outcome).is_none() {
            return Err("grounded-factorization reference outcome is not grounded".into());
        }
        if m.cyclic {
            return Err("additive grounded factorization is not implemented".into());
        }
        if m.outcome_state
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len()
            != m.outcome_state.len()
        {
            return Err("grounded outcome states must be injective".into());
        }
        Ok(m)
    }
}

/// Construct an exact factorization of a completely observed regular eight-outcome Q8 action.
///
/// The observed primitives supply permutations of the typed outcomes; the initial observations
/// supply `payload -> first outcome`. Any primitive or outcome not grounded by development is
/// absent from the served domains, so an unseen operand stays typed rather than becoming identity.
pub fn factor_observed_graph(
    examples: &[StExample],
    cyclic: bool,
) -> Result<(GroundedFactorization, FactorReport), String> {
    if cyclic {
        return Err("additive grounded factorization is not implemented".into());
    }
    if examples
        .iter()
        .any(|e| e.primitives.len() != e.targets.len())
    {
        return Err("every declared primitive requires one observed outcome".into());
    }
    let mut first: BTreeMap<(u32, u32), BTreeMap<u32, usize>> = BTreeMap::new();
    let mut recur: BTreeMap<(u32, u32), BTreeMap<u32, usize>> = BTreeMap::new();
    for ex in examples.iter() {
        for (j, (p, t)) in ex.primitives.iter().zip(ex.targets.iter()).enumerate() {
            if j == 0 {
                *first
                    .entry((ex.payload, *p))
                    .or_default()
                    .entry(*t)
                    .or_default() += 1;
            } else {
                *recur
                    .entry((ex.targets[j - 1], *p))
                    .or_default()
                    .entry(*t)
                    .or_default() += 1;
            }
        }
    }
    if first
        .values()
        .chain(recur.values())
        .any(|counts| counts.len() != 1)
    {
        return Err("conflicting declared transitions cannot be factored exactly".into());
    }
    let majority = |m: &BTreeMap<u32, usize>| -> u32 {
        *m.iter()
            .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
            .map(|(t, _)| t)
            .unwrap_or(&0)
    };
    let outcomes: Vec<u32> = examples
        .iter()
        .flat_map(|e| e.targets.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let primitives: Vec<u32> = examples
        .iter()
        .flat_map(|e| e.primitives.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let n = outcomes.len();
    if n < 2 || primitives.len() < 2 {
        return Err("factorization needs at least two typed outcomes and two primitives".into());
    }
    if !cyclic && n > GROUP_ORDER {
        return Err("more typed outcomes than the group domain".into());
    }
    let idx = |label: u32| outcomes.iter().position(|t| *t == label).unwrap_or(0);
    // Primitive permutations on the typed outcomes, from recurrent observations only.
    let mut perms: Vec<(u32, Vec<usize>)> = Vec::new();
    for p in primitives.iter() {
        let mut img: Vec<Option<usize>> = vec![None; n];
        for ((prev, q), counts) in recur.iter() {
            if q != p {
                continue;
            }
            let Some(pi) = outcomes.iter().position(|t| t == prev) else {
                continue;
            };
            let target = majority(counts);
            let Some(ti) = outcomes.iter().position(|t| *t == target) else {
                continue;
            };
            if let Some(existing) = img[pi] {
                if existing != ti {
                    return Err("non-invertible or conflicting observed transition".into());
                }
            }
            img[pi] = Some(ti);
        }
        if img.iter().any(|v| v.is_none()) {
            return Err("an observed primitive does not ground every typed outcome".into());
        }
        let perm: Vec<usize> = img.into_iter().map(|v| v.unwrap()).collect();
        if {
            let set: BTreeSet<usize> = perm.iter().copied().collect();
            set.len() != n
        } {
            return Err("an observed primitive permutation is not bijective".into());
        }
        perms.push((*p, perm));
    }
    // Abstract group closure under composition.
    let identity: Vec<usize> = (0..n).collect();
    let mut group: Vec<Vec<usize>> = vec![identity.clone()];
    let mut pending: Vec<Vec<usize>> = vec![identity.clone()];
    let mut guard = 0usize;
    while let Some(x) = pending.pop() {
        guard += 1;
        if guard > 4096 {
            return Err("observed permutations do not close under composition".into());
        }
        for (_, p) in perms.iter() {
            let y = compose(&x, p);
            if !group.contains(&y) {
                group.push(y.clone());
                pending.push(y);
            }
        }
    }
    if group.len() != n {
        return Err(format!(
            "observed primitive action is not regular: closure has {} elements for {} outcomes",
            group.len(),
            n
        ));
    }
    let order_of = |p: &Vec<usize>| -> usize {
        let mut x = identity.clone();
        for k in 1..=64 {
            x = compose(&x, p);
            if x == identity {
                return k;
            }
        }
        0
    };
    let mut spectrum: BTreeMap<usize, usize> = BTreeMap::new();
    for g in group.iter() {
        *spectrum.entry(order_of(g)).or_default() += 1;
    }
    let identity_primitive = perms
        .iter()
        .find(|(_, p)| *p == identity)
        .map(|(tok, _)| *tok);
    let noncommuting_witness = perms.iter().find_map(|(ta, a)| {
        perms
            .iter()
            .find(|(tb, b)| ta < tb && compose(a, b) != compose(b, a))
            .map(|(tb, _)| (*ta, *tb))
    });
    // Isomorphism into a verified subgroup of the served table.
    let q8: Vec<usize> = super::shared_transition::q8_witness()?
        .iter()
        .map(|v| *v as usize)
        .collect();
    let t = group_table();
    let hmul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
    let id_g = t.identity as usize;
    if q8.len() != n {
        return Err(format!(
            "observed outcome count {n} does not match the group witness size {}",
            q8.len()
        ));
    }
    let id_group = group.iter().position(|g| *g == identity).unwrap_or(0);
    let id_in_q8 = q8
        .iter()
        .position(|g| *g == id_g)
        .ok_or("the served group witness lacks the identity")?;
    let order: Vec<usize> = (0..group.len()).filter(|i| *i != id_group).collect();
    let cands: Vec<usize> = (0..q8.len()).filter(|i| *i != id_in_q8).collect();
    let (found, checked) =
        search_isomorphism(&group, id_group, &q8, id_in_q8, &order, &cands, &hmul);
    let phi = found.ok_or("no isomorphism from the observed action into the served group")?;
    // Outcome coordinates from a reference-state orbit; the reference outcome fixes the gauge.
    let reference_outcome = outcomes[0];
    let y0 = idx(reference_outcome);
    let mut outcome_state = vec![0u8; n];
    for (yi, _) in outcomes.iter().enumerate() {
        let element = group
            .iter()
            .position(|g| g[y0] == yi)
            .ok_or("observed action is not transitive on outcomes")?;
        outcome_state[yi] = phi[element] as u8;
    }
    // Action codes are the images of the primitive permutations.
    let mut action_code = Vec::with_capacity(perms.len());
    for (_, p) in perms.iter() {
        let gi = group
            .iter()
            .position(|g| g == p)
            .ok_or("primitive permutation is not in the closed group")?;
        action_code.push(phi[gi] as u8);
    }
    let action_domain: Vec<u32> = perms.iter().map(|(tok, _)| *tok).collect();
    // Initial states: E[x] = inverse(A[p]) * Z[first(x,p)], agreed across every observed primitive.
    let payload_domain: Vec<u32> = examples
        .iter()
        .map(|e| e.payload)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut payload_state = Vec::with_capacity(payload_domain.len());
    let mut initial_checks = 0usize;
    let mut initial_agreements = 0usize;
    for x in payload_domain.iter() {
        let mut assigned: Option<usize> = None;
        for (ai, (p, _)) in perms.iter().enumerate() {
            let Some(counts) = first.get(&(*x, *p)) else {
                continue;
            };
            let target = majority(counts);
            let zi = outcome_state[idx(target)] as usize;
            let a = action_code[ai] as usize;
            let inv = t.inverse[a] as usize;
            let e_state = hmul(inv, zi);
            initial_checks += 1;
            match assigned {
                None => assigned = Some(e_state),
                Some(prev) => {
                    if prev == e_state {
                        initial_agreements += 1;
                    } else {
                        return Err(format!(
                            "initial grounding disagrees across primitives for payload {x}"
                        ));
                    }
                }
            }
        }
        payload_state.push(assigned.ok_or("payload has no observed first transition")? as u8);
    }
    let model = GroundedFactorization {
        cyclic,
        action_domain,
        action_code,
        outcome_domain: outcomes.clone(),
        outcome_state,
        payload_domain,
        payload_state,
        reference_outcome,
    };
    // Verify the factorization on every declared observation.
    let mut transitions_checked = 0usize;
    let mut transitions_consistent = 0usize;
    for ex in examples.iter() {
        let mut state = match model.initial_state(ex.payload) {
            Some(s) => s,
            None => continue,
        };
        for (j, p) in ex.primitives.iter().enumerate() {
            let Some(next) = model.apply(state, *p) else {
                break;
            };
            transitions_checked += 1;
            let target = ex.targets.get(j).copied();
            if model.outcome_for_state(next) == target {
                transitions_consistent += 1;
            }
            state = next;
        }
    }
    if transitions_checked != transitions_consistent {
        return Err("factorization did not reproduce every declared transition".into());
    }
    let report = FactorReport {
        outcomes: n,
        primitives: perms.len(),
        group_order: group.len(),
        order_spectrum: spectrum.into_iter().collect(),
        identity_primitive,
        noncommuting_witness,
        isomorphic_candidates_checked: checked,
        transitions_checked,
        transitions_consistent,
        initial_checks,
        initial_agreements,
        gauge_fixed: model.state_for_outcome(reference_outcome) == Some(id_g),
    };
    Ok((model, report))
}

/// Exhaustive search for a group isomorphism `phi` from the observed action into the served
/// subgroup, with the abstract identity fixed. At most `8! = 40320` assignments, pruned by the
/// group order, so this terminates immediately for any regular order-8 action.
fn search_isomorphism(
    group: &[Vec<usize>],
    id_group: usize,
    q8: &[usize],
    id_in_q8: usize,
    order: &[usize],
    cands: &[usize],
    hmul: &dyn Fn(usize, usize) -> usize,
) -> (Option<Vec<usize>>, usize) {
    let homeomorphic = |phi: &Vec<usize>| -> bool {
        for a in 0..group.len() {
            for b in 0..group.len() {
                let ab = compose(&group[a], &group[b]);
                let Some(gi) = group.iter().position(|g| *g == ab) else {
                    return false;
                };
                if hmul(phi[a], phi[b]) != phi[gi] {
                    return false;
                }
            }
        }
        true
    };
    let mut phi = vec![usize::MAX; group.len()];
    phi[id_group] = q8[id_in_q8];
    // `used` is indexed by the candidate list, whose entries exclude the identity image.
    let mut used = vec![false; cands.len()];
    let mut checked = 0usize;
    let mut found: Option<Vec<usize>> = None;
    fn rec(
        k: usize,
        order: &[usize],
        cands: &[usize],
        phi: &mut Vec<usize>,
        used: &mut Vec<bool>,
        checked: &mut usize,
        homeomorphic: &dyn Fn(&Vec<usize>) -> bool,
        found: &mut Option<Vec<usize>>,
    ) {
        if found.is_some() {
            return;
        }
        if k == order.len() {
            *checked += 1;
            if homeomorphic(phi) {
                *found = Some(phi.clone());
            }
            return;
        }
        for (ci, cand) in cands.iter().enumerate() {
            if used[ci] {
                continue;
            }
            used[ci] = true;
            phi[order[k]] = *cand;
            rec(k + 1, order, cands, phi, used, checked, homeomorphic, found);
            used[ci] = false;
            phi[order[k]] = usize::MAX;
        }
    }
    let candidate_elements: Vec<usize> = cands.iter().map(|i| q8[*i]).collect();
    rec(
        0,
        order,
        &candidate_elements,
        &mut phi,
        &mut used,
        &mut checked,
        &homeomorphic,
        &mut found,
    );
    (found, checked)
}

/// An immutable owned-source snapshot; the legacy type name is retained. Its reference is
/// local to the owning ring. Copied payload remains usable after origin eviction; an origin
/// liveness check is a separate diagnostic, not a prerequisite for owned computation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLease {
    pub seq: u32,
    pub abs: u32,
    pub payload: u32,
}

impl SourceLease {
    pub fn acquire(ring: &OccurrenceRing, abs: u32) -> Result<Self, Terminal> {
        let reference = ring.reference(abs).ok_or(Terminal::StaleLease)?;
        let payload = ring.resolve(reference).ok_or(Terminal::StaleLease)?;
        Ok(Self {
            seq: reference.seq,
            abs: reference.abs,
            payload,
        })
    }
    pub fn origin_is_live(&self, ring: &OccurrenceRing) -> bool {
        ring.resolve(OccurrenceRef {
            seq: self.seq,
            abs: self.abs,
        }) == Some(self.payload)
    }
}

/// One resumable session frame: owned evidence, retained result, response phase and terminal status.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SessionFrame {
    pub lease: Option<SourceLease>,
    pub state: Option<usize>,
    pub outcome: Option<u32>,
    pub phase: u8,
    pub emitted: Vec<u32>,
    pub terminal: Option<Terminal>,
}

impl SessionFrame {
    pub fn start() -> Self {
        SessionFrame::default()
    }

    pub fn read(
        &mut self,
        lease: SourceLease,
        model: &GroundedFactorization,
    ) -> Result<(), Terminal> {
        if self.terminal.is_some() {
            return Err(self.terminal.unwrap());
        }
        match model.initial_state(lease.payload) {
            Some(s) => {
                self.lease = Some(lease);
                self.state = Some(s);
                self.outcome = model.outcome_for_state(s);
                self.phase = 0;
                Ok(())
            }
            None => {
                self.terminal = Some(Terminal::UnknownValue);
                Err(Terminal::UnknownValue)
            }
        }
    }

    pub fn apply_primitive(
        &mut self,
        primitive: u32,
        model: &GroundedFactorization,
    ) -> Result<(), Terminal> {
        if self.terminal.is_some() {
            return Err(self.terminal.unwrap());
        }
        let Some(state) = self.state else {
            self.terminal = Some(Terminal::NoRead);
            return Err(Terminal::NoRead);
        };
        match model.apply(state, primitive) {
            Some(next) => {
                self.state = Some(next);
                self.outcome = model.outcome_for_state(next);
                self.phase = self.phase.saturating_add(1);
                if self.outcome.is_none() {
                    self.terminal = Some(Terminal::NoGrounding);
                    return Err(Terminal::NoGrounding);
                }
                Ok(())
            }
            None => {
                self.terminal = Some(Terminal::UnknownPrimitive);
                Err(Terminal::UnknownPrimitive)
            }
        }
    }

    pub fn emit(&mut self) -> Result<u32, Terminal> {
        if self.terminal.is_some() {
            return Err(self.terminal.unwrap());
        }
        match self.outcome {
            Some(t) => {
                self.emitted.push(t);
                Ok(t)
            }
            None => {
                self.terminal = Some(Terminal::NoGrounding);
                Err(Terminal::NoGrounding)
            }
        }
    }

    pub fn stop(&mut self, reason: Terminal) {
        self.terminal = Some(reason);
    }

    /// Whether the snapshot's original occurrence is still live in its owning ring.
    /// A false result does not invalidate the copied payload.
    pub fn lease_is_live(&self, ring: &OccurrenceRing) -> bool {
        self.lease.is_some_and(|l| l.origin_is_live(ring))
    }

    /// Validate a restored frame against the actual loaded computation model. This is
    /// semantic validation; callers separately bind saved bytes to their model artifact.
    pub fn validate(&self, model: &GroundedFactorization) -> Result<(), String> {
        if self.terminal == Some(Terminal::None) {
            return Err("invalid explicit None terminal".into());
        }
        if let Some(lease) = self.lease {
            if model.initial_state(lease.payload).is_none() {
                return Err("snapshot operand is not grounded by this model".into());
            }
        }
        match self.state {
            Some(state) if state >= GROUP_ORDER => {
                return Err("frame state outside group domain".into())
            }
            Some(state) => {
                if self.lease.is_none() || model.outcome_for_state(state) != self.outcome {
                    return Err("frame result/outcome is inconsistent with its loaded model".into());
                }
            }
            None if self.outcome.is_some() => {
                return Err("frame outcome has no result state".into())
            }
            None => {}
        }
        if self
            .emitted
            .iter()
            .any(|t| model.outcome_index(*t).is_none())
        {
            return Err("frame emission is not grounded by this model".into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLSF");
        o.extend_from_slice(&1u32.to_le_bytes());
        match self.lease {
            Some(l) => {
                o.push(1);
                o.extend_from_slice(&l.seq.to_le_bytes());
                o.extend_from_slice(&l.abs.to_le_bytes());
                o.extend_from_slice(&l.payload.to_le_bytes());
            }
            None => o.push(0),
        }
        match self.state {
            Some(s) => {
                o.push(1);
                o.extend_from_slice(&(s as u32).to_le_bytes());
            }
            None => o.push(0),
        }
        match self.outcome {
            Some(t) => {
                o.push(1);
                o.extend_from_slice(&t.to_le_bytes());
            }
            None => o.push(0),
        }
        o.push(self.phase);
        o.extend_from_slice(&(self.emitted.len() as u32).to_le_bytes());
        for t in self.emitted.iter() {
            o.extend_from_slice(&t.to_le_bytes());
        }
        o.push(match self.terminal {
            None => 0,
            Some(Terminal::Stop) => 1,
            Some(Terminal::Exhausted) => 2,
            Some(Terminal::NoRead) => 3,
            Some(Terminal::UnknownValue) => 4,
            Some(Terminal::UnknownPrimitive) => 5,
            Some(Terminal::NoGrounding) => 6,
            Some(Terminal::StaleLease) => 7,
            Some(Terminal::None) => 8,
        });
        o
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let mut c = 0usize;
        let take = |c: &mut usize, n: usize| -> Result<&[u8], String> {
            let end = c.checked_add(n).ok_or("size overflow")?;
            if end > bytes.len() {
                return Err("truncated session frame".into());
            }
            let s = &bytes[*c..end];
            *c = end;
            Ok(s)
        };
        if take(&mut c, 4)? != b"RLSF" {
            return Err("bad session frame magic".into());
        }
        if u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) != 1 {
            return Err("unsupported session frame version".into());
        }
        let lease = if take(&mut c, 1)?[0] != 0 {
            let seq = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            let abs = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            let payload = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            if payload as usize >= max_vocab {
                return Err("session lease payload outside the vocabulary".into());
            }
            Some(SourceLease { seq, abs, payload })
        } else {
            None
        };
        let state = if take(&mut c, 1)?[0] != 0 {
            Some(u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize)
        } else {
            None
        };
        if state.is_some_and(|s| s >= GROUP_ORDER) {
            return Err("session state outside group domain".into());
        }
        let outcome = if take(&mut c, 1)?[0] != 0 {
            let t = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            if t as usize >= max_vocab {
                return Err("session outcome outside the vocabulary".into());
            }
            Some(t)
        } else {
            None
        };
        let phase = take(&mut c, 1)?[0];
        let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if n.checked_mul(4).ok_or("emitted overflow")? > bytes.len().saturating_sub(c) {
            return Err("truncated session emissions".into());
        }
        let mut emitted = Vec::with_capacity(n);
        for _ in 0..n {
            let t = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            if t as usize >= max_vocab {
                return Err("session emission outside the vocabulary".into());
            }
            emitted.push(t);
        }
        let terminal = match take(&mut c, 1)?[0] {
            0 => None,
            1 => Some(Terminal::Stop),
            2 => Some(Terminal::Exhausted),
            3 => Some(Terminal::NoRead),
            4 => Some(Terminal::UnknownValue),
            5 => Some(Terminal::UnknownPrimitive),
            6 => Some(Terminal::NoGrounding),
            7 => Some(Terminal::StaleLease),
            8 => Some(Terminal::None),
            _ => return Err("unknown session terminal tag".into()),
        };
        if c != bytes.len() {
            return Err("trailing bytes in the session frame".into());
        }
        Ok(SessionFrame {
            lease,
            state,
            outcome,
            phase,
            emitted,
            terminal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Declared development observations built from the witness subgroup's own regular left
    /// action. The typed outcomes are the witness indices; the primitive tokens are the witness
    /// indices themselves, so a valid isomorphism provably exists for the mechanism test.
    fn dihedral_examples() -> Vec<StExample> {
        let w: Vec<usize> = super::super::shared_transition::q8_witness()
            .expect("a witness subgroup must exist")
            .iter()
            .map(|v| *v as usize)
            .collect();
        let n = w.len();
        let t = group_table();
        let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
        let index = |g: usize| w.iter().position(|x| *x == g).unwrap_or(0);
        let id_idx = index(t.identity as usize);
        let perm = |p: usize| -> Vec<usize> { (0..n).map(|i| index(mul(w[p], w[i]))).collect() };
        let mut ex = Vec::new();
        for x in 0..n as u32 {
            for p in 0..n as u32 {
                ex.push(StExample {
                    payload: x,
                    primitives: vec![p],
                    targets: vec![perm(p as usize)[x as usize] as u32],
                });
            }
        }
        for y in 0..n as u32 {
            for p in 0..n as u32 {
                ex.push(StExample {
                    payload: y,
                    primitives: vec![id_idx as u32, p],
                    targets: vec![y, perm(p as usize)[y as usize] as u32],
                });
            }
        }
        ex
    }

    #[test]
    fn the_factorization_reproduces_every_observed_transition() {
        let ex = dihedral_examples();
        let (m, report) = factor_observed_graph(&ex, false).expect("factorization must succeed");
        assert_eq!(report.transitions_checked, report.transitions_consistent);
        assert!(report.isomorphic_candidates_checked > 0);
        for e in ex.iter() {
            let r = m.serve(Some(e.payload), &e.primitives);
            assert_eq!(
                r.tokens, e.targets,
                "served program must match the observation"
            );
            assert!(r.stopped);
        }
        let back = GroundedFactorization::from_bytes(&m.to_bytes(), 4096).unwrap();
        assert_eq!(back, m);
        let mut bad = m.to_bytes();
        bad[0] = b'X';
        assert!(GroundedFactorization::from_bytes(&bad, 4096).is_err());
    }

    #[test]
    fn outcome_state_is_injective_and_the_reference_fixes_the_gauge() {
        let ex = dihedral_examples();
        let (m, report) = factor_observed_graph(&ex, false).unwrap();
        let set: BTreeSet<u8> = m.outcome_state.iter().copied().collect();
        assert_eq!(set.len(), m.outcome_state.len());
        assert!(report.gauge_fixed);
        assert_eq!(
            m.outcome_for_state(m.state_for_outcome(m.reference_outcome).unwrap()),
            Some(m.reference_outcome)
        );
    }

    #[test]
    fn a_session_resumes_and_interleaves_without_sharing_state() {
        let ex = dihedral_examples();
        let (m, _) = factor_observed_graph(&ex, false).unwrap();
        let lease = SourceLease {
            seq: 3,
            abs: 7,
            payload: 0,
        };
        // Uninterrupted run.
        let mut a = SessionFrame::start();
        a.read(lease, &m).unwrap();
        a.apply_primitive(1, &m).unwrap();
        a.emit().unwrap();
        a.apply_primitive(2, &m).unwrap();
        a.emit().unwrap();
        a.stop(Terminal::Stop);
        // Paused and resumed at every point.
        let mut b = SessionFrame::start();
        b.read(lease, &m).unwrap();
        b = SessionFrame::from_bytes(&b.to_bytes(), 4096).unwrap();
        b.apply_primitive(1, &m).unwrap();
        b.emit().unwrap();
        b = SessionFrame::from_bytes(&b.to_bytes(), 4096).unwrap();
        b.apply_primitive(2, &m).unwrap();
        b.emit().unwrap();
        b.stop(Terminal::Stop);
        assert_eq!(a, b, "resume must share the uninterrupted transition");
        // Interleaving: advancing one frame must not change an unrelated frame.
        let mut c = SessionFrame::start();
        let other = SourceLease {
            seq: 9,
            abs: 2,
            payload: 3,
        };
        c.read(other, &m).unwrap();
        let snapshot = c.clone();
        let mut d = SessionFrame::start();
        d.read(lease, &m).unwrap();
        d.apply_primitive(1, &m).unwrap();
        d.apply_primitive(2, &m).unwrap();
        d.emit().unwrap();
        d.stop(Terminal::Stop);
        assert_eq!(
            c, snapshot,
            "an unrelated session must not share mutable state"
        );
        assert!(d.emitted.len() == 1 && c.emitted.is_empty());
        // Actual ring-backed snapshot liveness and eviction are exercised separately below.
    }

    #[test]
    fn unknown_operands_stay_typed_and_the_reader_absence_is_not_identity() {
        let ex = dihedral_examples();
        let (m, _) = factor_observed_graph(&ex, false).unwrap();
        let r = m.serve(Some(0), &[1, 999]);
        assert_eq!(r.steps.last(), Some(&StepKind::UnknownPrimitive));
        let r = m.serve(None, &[1]);
        assert_eq!(r.steps, vec![StepKind::NoRead]);
        let r = m.serve(Some(777), &[1]);
        assert_eq!(r.steps, vec![StepKind::UnknownValue]);
    }

    #[test]
    fn isomorphism_uses_actual_elements_of_a_reordered_witness() {
        let mut witness: Vec<usize> = super::super::shared_transition::q8_witness()
            .unwrap()
            .iter()
            .map(|x| *x as usize)
            .collect();
        let t = group_table();
        let mul = |a: usize, b: usize| t.product[a * ROW_STRIDE + b] as usize;
        let group: Vec<Vec<usize>> = witness
            .iter()
            .map(|a| {
                witness
                    .iter()
                    .map(|b| witness.iter().position(|x| *x == mul(*a, *b)).unwrap())
                    .collect()
            })
            .collect();
        let identity: Vec<usize> = (0..witness.len()).collect();
        let id_group = group.iter().position(|g| *g == identity).unwrap();
        witness.rotate_left(1); // Makes witness positions differ from served element IDs.
        let id_witness = witness
            .iter()
            .position(|g| *g == t.identity as usize)
            .unwrap();
        let order: Vec<usize> = (0..group.len()).filter(|i| *i != id_group).collect();
        let candidates: Vec<usize> = (0..witness.len()).filter(|i| *i != id_witness).collect();
        let phi = search_isomorphism(
            &group,
            id_group,
            &witness,
            id_witness,
            &order,
            &candidates,
            &mul,
        )
        .0
        .unwrap();
        for a in 0..group.len() {
            for b in 0..group.len() {
                let ab = group
                    .iter()
                    .position(|g| *g == compose(&group[a], &group[b]))
                    .unwrap();
                assert_eq!(mul(phi[a], phi[b]), phi[ab]);
            }
        }
    }

    #[test]
    fn exact_factorization_rejects_conflicting_labels_and_unsupported_algebra() {
        let mut examples = dihedral_examples();
        assert!(factor_observed_graph(&examples, true).is_err());
        let mut conflict = examples[0].clone();
        conflict.targets[0] = (conflict.targets[0] + 1) % 8;
        examples.push(conflict);
        assert!(factor_observed_graph(&examples, false).is_err());
    }

    #[test]
    fn factorization_is_invariant_to_observed_token_and_outcome_relabeling() {
        let mut examples = dihedral_examples();
        for ex in &mut examples {
            ex.payload = 100 + (ex.payload * 3 + 1) % 8;
            for p in &mut ex.primitives {
                *p = 200 + (*p * 5 + 2) % 8;
            }
            for y in &mut ex.targets {
                *y = 300 + (*y * 3 + 4) % 8;
            }
        }
        let (model, report) = factor_observed_graph(&examples, false).unwrap();
        assert_eq!(report.transitions_checked, report.transitions_consistent);
        for ex in &examples {
            assert_eq!(
                model.serve(Some(ex.payload), &ex.primitives).tokens,
                ex.targets
            );
        }
    }

    #[test]
    fn owned_snapshot_survives_eviction_while_exact_origin_liveness_changes() {
        let (model, _) = factor_observed_graph(&dihedral_examples(), false).unwrap();
        let mut ring = OccurrenceRing::new(2);
        ring.observe(0);
        let snapshot = SourceLease::acquire(&ring, 0).unwrap();
        let mut frame = SessionFrame::start();
        frame.read(snapshot, &model).unwrap();
        assert!(frame.lease_is_live(&ring));
        let nonidentity = model
            .action_domain
            .iter()
            .zip(&model.action_code)
            .find(|(_, code)| **code as usize != group_table().identity as usize)
            .map(|(p, _)| *p)
            .unwrap();
        frame.apply_primitive(nonidentity, &model).unwrap();
        assert_ne!(frame.state, model.initial_state(snapshot.payload));
        let saved = frame.to_bytes();
        let mut resumed = SessionFrame::from_bytes(&saved, 4096).unwrap();
        resumed.validate(&model).unwrap();
        ring.observe(7);
        ring.observe(0); // Same payload in wrapped slot must not resurrect abs 0.
        assert!(!resumed.lease_is_live(&ring));
        frame.apply_primitive(2, &model).unwrap();
        resumed.apply_primitive(2, &model).unwrap();
        assert_eq!(frame.emit().unwrap(), resumed.emit().unwrap());
        assert_eq!(frame, resumed);
        let mut bad = resumed.to_bytes();
        *bad.last_mut().unwrap() = 255;
        assert!(SessionFrame::from_bytes(&bad, 4096).is_err());
        resumed.state = Some(GROUP_ORDER);
        assert!(SessionFrame::from_bytes(&resumed.to_bytes(), 4096).is_err());
        resumed.state = frame.state;
        resumed.outcome = Some(4095);
        assert!(resumed.validate(&model).is_err());
    }
}
