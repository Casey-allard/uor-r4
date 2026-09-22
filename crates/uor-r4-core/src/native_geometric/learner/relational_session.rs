//! Learned relational access and content-dependent session control.
//!
//! A request names a **relation** and an **entity**. The session must choose which fact to retrieve,
//! decide from the retrieved content whether to continue, and emit a complete answer. Three things
//! stay separate:
//!
//! * **exact memory** - records are `(role, key, value)` token triples; candidate admission is an
//!   exact identity join on the key, never a lossy descriptor;
//! * **learned relational compatibility** - a relation is carried as an exact group element `A[rel]`
//!   and a record role as `Z[role]`; a candidate matches the requested relation when
//!   `inverse(A[rel]) * Z[role]` is the identity. The matched control replaces the group relative
//!   element with a learned categorical table over the same supervision;
//! * **owned evidence and control state** - a captured payload is an immutable owned value with
//!   provenance; continuation is decided from observed content, not from a supplied schedule.
//!
//! The model never receives a target, a gold hop count, a next-source pointer or a fixture family at
//! serving. Gold intermediate actions may supervise offline fitting and are declared as such.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::group_table::{group_table, GROUP_ORDER, ROW_STRIDE};

/// Declared execution cap: a safety limit, reported separately from learned completion.
pub const REL_MAX_HOPS: u8 = 6;

/// One typed session action or terminal reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RelAction {
    Read,
    Continue,
    Emit,
    Stop,
    Unresolved,
    Exhausted,
}

/// An immutable owned payload snapshot with its provenance. The owned value remains usable after its
/// origin is evicted; resolving the current ring is a separate liveness diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CapturedPayload {
    pub seq: u32,
    pub abs: u32,
    pub payload: u32,
    pub version: u32,
}

/// One declared development trace: the request, the facts a correct session must reach, and the
/// observed intermediate roles. `depth` is the number of reads the task actually needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RelExample {
    pub relation: u32,
    pub entity: u32,
    pub first_role: u32,
    pub first_value: u32,
    pub follow_relation: u32,
    pub second_role: u32,
    pub second_value: u32,
    pub depth: u8,
    /// Observed value type from the declared entity registry, independently of the action label.
    pub content_is_entity: bool,
}

/// Receipt for one relational-policy fit.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RelFitReport {
    pub examples: usize,
    pub relations: usize,
    pub roles: usize,
    pub scorer_initial: usize,
    pub scorer_final: usize,
    pub follow_entries: usize,
    pub relation_pairs_learned: usize,
    pub policy_initial: usize,
    pub policy_final: usize,
    pub accepted_moves: usize,
}

/// Fitted relation codes, follow map and observed-type policy; score coefficients are fixed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalModel {
    /// Additive `mod GROUP_ORDER` compatibility instead of the exact group relative element.
    pub cyclic: bool,
    /// Learned categorical compatibility `(relation, role) -> 0/1` used by the matched control.
    pub categorical: bool,
    pub relation_domain: Vec<u32>,
    pub relation_code: Vec<u8>,
    pub role_domain: Vec<u32>,
    pub role_code: Vec<u8>,
    /// Learned follow-up relation per request relation.
    pub follow: Vec<(u32, u32)>,
    /// Fixed scorer weights: `[relation match, exact key identity, content is a key, bias]`.
    pub weights: [i32; 4],
    /// Learned continuation decision indexed by `content_is_key as usize`.
    pub continue_policy: [bool; 2],
    /// Learned categorical table over `(relation index, role index)`.
    pub categorical_table: Vec<[u8; 8]>,
}

impl RelationalModel {
    pub fn relation_index(&self, relation: u32) -> Option<usize> {
        self.relation_domain.iter().position(|t| *t == relation)
    }

    pub fn role_index(&self, role: u32) -> Option<usize> {
        self.role_domain.iter().position(|t| *t == role)
    }

    pub fn relation_element(&self, relation: u32) -> Option<usize> {
        self.relation_index(relation)
            .map(|i| self.relation_code[i] as usize)
    }

    pub fn role_element(&self, role: u32) -> Option<usize> {
        self.role_index(role).map(|i| self.role_code[i] as usize)
    }

    /// `inverse(A[relation]) * Z[role]`: the identity exactly when the role answers that relation.
    pub fn relative(&self, relation: u32, role: u32) -> Option<usize> {
        let a = self.relation_element(relation)?;
        let z = self.role_element(role)?;
        if self.cyclic {
            return Some((z + GROUP_ORDER - a) % GROUP_ORDER);
        }
        let t = group_table();
        let inv = t.inverse[a] as usize;
        Some(t.product[inv * ROW_STRIDE + z] as usize)
    }

    /// Learned relation/role compatibility, `+1` when compatible and `-1` otherwise.
    pub fn relation_matches(&self, relation: u32, role: u32) -> i32 {
        if self.categorical {
            match (self.relation_index(relation), self.role_index(role)) {
                (Some(r), Some(c)) => {
                    let byte = self
                        .categorical_table
                        .get(r)
                        .map(|row| row.get(c).copied().unwrap_or(0))
                        .unwrap_or(0);
                    if byte != 0 {
                        1
                    } else {
                        -1
                    }
                }
                _ => -1,
            }
        } else {
            let identity = if self.cyclic {
                0
            } else {
                group_table().identity as usize
            };
            match self.relative(relation, role) {
                Some(r) if r == identity => 1,
                Some(_) => -1,
                None => -1,
            }
        }
    }

    pub fn follow_of(&self, relation: u32) -> Option<u32> {
        self.follow
            .iter()
            .find(|(r, _)| *r == relation)
            .map(|(_, f)| *f)
    }

    /// Candidate score. Admission is an exact identity join performed by the caller; this ranks the
    /// admitted candidates and is deliberately separable from admission.
    pub fn score(&self, relation: u32, role: u32, exact_key: bool, content_is_key: bool) -> i32 {
        self.weights[0] * self.relation_matches(relation, role)
            + self.weights[1] * i32::from(exact_key)
            + self.weights[2] * i32::from(content_is_key)
            + self.weights[3]
    }

    pub fn should_continue(&self, content_is_key: bool) -> bool {
        self.continue_policy[usize::from(content_is_key)]
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"RLRM");
        o.extend_from_slice(&2u32.to_le_bytes());
        o.push(u8::from(self.cyclic));
        o.push(u8::from(self.categorical));
        for domain in [&self.relation_domain, &self.role_domain] {
            o.extend_from_slice(&(domain.len() as u32).to_le_bytes());
            for t in domain.iter() {
                o.extend_from_slice(&t.to_le_bytes());
            }
        }
        for code in [&self.relation_code, &self.role_code] {
            o.extend_from_slice(&(code.len() as u32).to_le_bytes());
            o.extend_from_slice(code);
        }
        o.extend_from_slice(&(self.follow.len() as u32).to_le_bytes());
        for (r, f) in self.follow.iter() {
            o.extend_from_slice(&r.to_le_bytes());
            o.extend_from_slice(&f.to_le_bytes());
        }
        for w in self.weights.iter() {
            o.extend_from_slice(&w.to_le_bytes());
        }
        o.push(u8::from(self.continue_policy[0]));
        o.push(u8::from(self.continue_policy[1]));
        o.extend_from_slice(&(self.categorical_table.len() as u32).to_le_bytes());
        for row in self.categorical_table.iter() {
            o.extend_from_slice(row);
        }
        o
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
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
        if take(&mut c, 4)? != b"RLRM" {
            return Err("bad relational artifact magic".into());
        }
        if u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) != 2 {
            return Err("unsupported relational artifact version".into());
        }
        let flag = |b: u8| match b {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("invalid relational boolean".to_string()),
        };
        let cyclic = flag(take(&mut c, 1)?[0])?;
        let categorical = flag(take(&mut c, 1)?[0])?;
        let mut domains = Vec::new();
        for _ in 0..2 {
            let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
            if n == 0 || n.checked_mul(4).ok_or("domain overflow")? > bytes.len().saturating_sub(c)
            {
                return Err("invalid relational domain".into());
            }
            let mut d = Vec::with_capacity(n);
            for _ in 0..n {
                let t = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
                if t as usize >= max_vocab {
                    return Err("relational token outside the vocabulary".into());
                }
                d.push(t);
            }
            if d.windows(2).any(|w| w[0] >= w[1]) {
                return Err("relational domain must be strictly increasing".into());
            }
            domains.push(d);
        }
        let mut codes = Vec::new();
        for _ in 0..2 {
            let n = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
            if n == 0 {
                return Err("empty relational code map".into());
            }
            codes.push(take(&mut c, n)?.to_vec());
        }
        let relation_domain = domains.remove(0);
        let role_domain = domains.remove(0);
        let relation_code = codes.remove(0);
        let role_code = codes.remove(0);
        if role_domain.len() > 8 {
            return Err("categorical role capacity exceeded".into());
        }
        if relation_code.len() != relation_domain.len() || role_code.len() != role_domain.len() {
            return Err("relational code length disagrees with its domain".into());
        }
        if relation_code
            .iter()
            .chain(role_code.iter())
            .any(|v| *v as usize >= GROUP_ORDER)
        {
            return Err("relational element outside the declared group domain".into());
        }
        let nf = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if nf.checked_mul(8).ok_or("follow overflow")? > bytes.len().saturating_sub(c) {
            return Err("truncated relational follow map".into());
        }
        let mut follow = Vec::with_capacity(nf);
        for _ in 0..nf {
            let r = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            let f = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
            if !relation_domain.contains(&r) || !relation_domain.contains(&f) {
                return Err("follow map refers outside the relation domain".into());
            }
            follow.push((r, f));
        }
        if follow.windows(2).any(|w| w[0].0 >= w[1].0) {
            return Err("relational follow map must be strictly ordered".into());
        }
        let mut weights = [0i32; 4];
        for w in weights.iter_mut() {
            *w = i32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap());
        }
        if weights != [1, 1, 0, 0] {
            return Err("unsupported fixed score coefficients".into());
        }
        let continue_policy = [flag(take(&mut c, 1)?[0])?, flag(take(&mut c, 1)?[0])?];
        let nt = u32::from_le_bytes(take(&mut c, 4)?.try_into().unwrap()) as usize;
        if nt != relation_domain.len() {
            return Err("relational categorical table has the wrong row count".into());
        }
        let raw = take(&mut c, nt * 8)?;
        let categorical_table: Vec<[u8; 8]> = (0..nt)
            .map(|i| std::array::from_fn(|j| raw[i * 8 + j]))
            .collect();
        if categorical_table.iter().flatten().any(|v| *v > 1) {
            return Err("relational categorical table is out of range".into());
        }
        if c != bytes.len() {
            return Err("trailing bytes in the relational artifact".into());
        }
        Ok(RelationalModel {
            cyclic,
            categorical,
            relation_domain,
            role_domain,
            relation_code,
            role_code,
            follow,
            weights,
            continue_policy,
            categorical_table,
        })
    }
}

/// Identities required to restore a session. The namespace belongs to the supplied world;
/// changing records does not invalidate an already owned payload, but a new namespace does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RelBinding {
    pub model: [u8; 32],
    pub geometry: [u8; 32],
    pub tokenizer: [u8; 32],
    pub world_namespace: u32,
}

/// All state consumed by the incremental runtime, including its next phase.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RelFrame {
    pub binding: RelBinding,
    pub captured: Option<CapturedPayload>,
    pub relation: Option<u32>,
    pub entity: Option<u32>,
    pub retained: Option<u32>,
    pub hops: u8,
    pub emitted: Vec<u32>,
    pub pending: Option<RelAction>,
    pub terminal: Option<RelAction>,
}

impl RelFrame {
    pub fn start(relation: u32, entity: u32, binding: RelBinding) -> Self {
        Self {
            binding,
            relation: Some(relation),
            entity: Some(entity),
            captured: None,
            retained: None,
            hops: 0,
            emitted: Vec::new(),
            pending: Some(RelAction::Read),
            terminal: None,
        }
    }

    pub fn capture(&mut self, payload: CapturedPayload) -> Result<(), String> {
        if self.terminal.is_some() || self.pending != Some(RelAction::Read) {
            return Err("capture requires the Read phase".into());
        }
        if payload.seq != self.binding.world_namespace || self.hops >= REL_MAX_HOPS {
            return Err("capture namespace or hop bound mismatch".into());
        }
        self.retained = Some(payload.payload);
        self.captured = Some(payload);
        self.hops += 1;
        self.pending = Some(RelAction::Continue);
        Ok(())
    }

    pub fn emit(&mut self) -> Result<u32, RelAction> {
        if let Some(t) = self.terminal {
            return Err(t);
        }
        if self.pending != Some(RelAction::Emit) {
            return Err(RelAction::Unresolved);
        }
        let v = self.retained.ok_or(RelAction::Unresolved)?;
        self.emitted.push(v);
        self.stop(RelAction::Stop);
        Ok(v)
    }

    pub fn stop(&mut self, reason: RelAction) {
        self.terminal = Some(reason);
        self.pending = None;
    }

    pub fn advance_relation(&mut self, next_relation: u32) -> Result<(), String> {
        if self.terminal.is_some()
            || self.pending != Some(RelAction::Continue)
            || self.retained.is_none()
        {
            return Err("advance requires a captured operand in Continue phase".into());
        }
        self.relation = Some(next_relation);
        self.entity = self.retained;
        self.pending = Some(RelAction::Read);
        Ok(())
    }

    /// Exact provenance diagnostic; equality of values alone is not source identity.
    pub fn origin_is_live(&self, current: Option<CapturedPayload>) -> bool {
        self.captured.is_some() && self.captured == current
    }

    pub fn validate(&self, max_vocab: usize, expected: &RelBinding) -> Result<(), String> {
        if &self.binding != expected {
            return Err("relational snapshot binding mismatch".into());
        }
        if self.hops > REL_MAX_HOPS {
            return Err("relational snapshot exceeds hop bound".into());
        }
        if self.relation.is_none() || self.entity.is_none() {
            return Err("missing relational request state".into());
        }
        if self
            .relation
            .iter()
            .chain(self.entity.iter())
            .chain(self.retained.iter())
            .chain(self.emitted.iter())
            .any(|v| *v as usize >= max_vocab)
        {
            return Err("relational snapshot token outside vocabulary".into());
        }
        if let Some(c) = self.captured {
            if c.seq != expected.world_namespace
                || self.retained != Some(c.payload)
                || c.payload as usize >= max_vocab
                || self.hops == 0
            {
                return Err("inconsistent captured relational operand".into());
            }
        } else if self.retained.is_some() || self.hops != 0 {
            return Err("missing captured relational operand".into());
        }
        match (self.pending, self.terminal) {
            (Some(RelAction::Read), None) => {}
            (Some(RelAction::Continue | RelAction::Emit), None) if self.captured.is_some() => {}
            (None, Some(RelAction::Stop | RelAction::Unresolved | RelAction::Exhausted)) => {}
            _ => return Err("invalid relational phase or terminal state".into()),
        }
        if self.emitted.len() > 1
            || (!self.emitted.is_empty() && self.terminal != Some(RelAction::Stop))
        {
            return Err("inconsistent relational emission state".into());
        }
        if self.terminal == Some(RelAction::Stop)
            && (self.emitted.len() != 1 || self.emitted.first().copied() != self.retained)
        {
            return Err("stopped frame has no matching emitted operand".into());
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bytes = b"RLRF\x02\x00\x00\x00".to_vec();
        bytes.extend(serde_json::to_vec(self).map_err(|e| e.to_string())?);
        Ok(bytes)
    }

    pub fn from_bytes(
        bytes: &[u8],
        max_vocab: usize,
        expected: &RelBinding,
    ) -> Result<Self, String> {
        if bytes.get(..8) != Some(b"RLRF\x02\x00\x00\x00") {
            return Err("bad relational frame magic/version".into());
        }
        let frame: Self = serde_json::from_slice(&bytes[8..]).map_err(|e| e.to_string())?;
        frame.validate(max_vocab, expected)?;
        Ok(frame)
    }
}

fn majority(values: &[u32]) -> Option<u32> {
    let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
    for v in values {
        *counts.entry(*v).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(v, _)| v)
}

/// A candidate as the ranker sees it: exact identity and observed content are separate from the
/// learned compatibility score.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Candidate {
    pub role: u32,
    pub key: u32,
    pub value: u32,
    pub exact_key: bool,
}

/// Rank the admitted candidates with the learned scorer. Admission itself is the caller's exact
/// identity join; this never widens the pool.
pub fn rank(
    model: &RelationalModel,
    relation: u32,
    content_is_key: bool,
    candidates: &[Candidate],
) -> Option<usize> {
    let mut best: Option<(i32, usize)> = None;
    for (i, cand) in candidates.iter().enumerate() {
        let s = model.score(relation, cand.role, cand.exact_key, content_is_key);
        if best.map(|(bs, _)| s > bs).unwrap_or(true) {
            best = Some((s, i));
        }
    }
    best.map(|(_, i)| i)
}

/// Fit the relational model from declared development traces.
pub fn learn_relational_model(
    examples: &[RelExample],
    cyclic: bool,
    categorical: bool,
) -> (RelationalModel, RelFitReport) {
    let relation_domain: Vec<u32> = examples
        .iter()
        .map(|e| e.relation)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let role_domain: Vec<u32> = examples
        .iter()
        .map(|e| e.first_role)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let n_rel = relation_domain.len();
    let n_role = role_domain.len();
    // Deterministic, non-degenerate start. Every served element is subsequently learned.
    let relation_code: Vec<u8> = (0..n_rel)
        .map(|i| ((i * 7 + 1) % GROUP_ORDER) as u8)
        .collect();
    let role_code: Vec<u8> = (0..n_role)
        .map(|i| ((i * 11 + 3) % GROUP_ORDER) as u8)
        .collect();
    // Observed role pairing: which role actually answered each relation.
    let mut pairing: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for e in examples {
        pairing.entry(e.relation).or_default().push(e.first_role);
    }
    let mut follow_pairs: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for e in examples.iter().filter(|e| e.depth >= 2) {
        follow_pairs
            .entry(e.relation)
            .or_default()
            .push(e.follow_relation);
    }
    let follow: Vec<(u32, u32)> = follow_pairs
        .into_iter()
        .filter_map(|(r, v)| majority(&v).map(|f| (r, f)))
        .collect();
    let mut model = RelationalModel {
        cyclic,
        categorical,
        relation_domain,
        relation_code,
        role_domain,
        role_code,
        follow,
        weights: [1, 1, 0, 0],
        continue_policy: [false, false],
        categorical_table: vec![[0u8; 8]; n_rel],
    };
    // Supervision: the observed role for each relation is compatible; the others are not.
    let correct = |m: &RelationalModel| -> usize {
        let mut hits = 0usize;
        for e in examples {
            if m.relation_matches(e.relation, e.first_role) > 0 {
                hits += 1;
            }
            for other in m.role_domain.iter() {
                if *other != e.first_role && m.relation_matches(e.relation, *other) > 0 {
                    hits = hits.saturating_sub(1);
                }
            }
        }
        hits
    };
    let initial = correct(&model);
    let mut best = initial;
    let mut accepted = 0usize;
    // Discrete coordinate search over the relation and role elements, then the categorical table.
    for _ in 0..3 {
        let mut improved = false;
        for i in 0..model.relation_code.len() {
            let saved = model.relation_code[i];
            let mut best_val = saved;
            let mut best_score = best;
            for cand in 0..GROUP_ORDER {
                model.relation_code[i] = cand as u8;
                let s = correct(&model);
                if s > best_score {
                    best_score = s;
                    best_val = cand as u8;
                }
            }
            model.relation_code[i] = best_val;
            if best_score > best {
                best = best_score;
                accepted += 1;
                improved = true;
            }
        }
        for j in 0..model.role_code.len() {
            let saved = model.role_code[j];
            let mut best_val = saved;
            let mut best_score = best;
            for cand in 0..GROUP_ORDER {
                model.role_code[j] = cand as u8;
                let s = correct(&model);
                if s > best_score {
                    best_score = s;
                    best_val = cand as u8;
                }
            }
            model.role_code[j] = best_val;
            if best_score > best {
                best = best_score;
                accepted += 1;
                improved = true;
            }
        }
        if !improved {
            break;
        }
    }
    // Categorical control: the same supervision, an explicitly non-geometric compatibility table.
    let mut cat_hits = 0usize;
    for r in 0..model.relation_domain.len() {
        let rel = model.relation_domain[r];
        let observed: BTreeSet<u32> = examples
            .iter()
            .filter(|e| e.relation == rel)
            .map(|e| e.first_role)
            .collect();
        let mut row = [0u8; 8];
        for (c, role) in model.role_domain.iter().enumerate() {
            if c < 8 && observed.contains(role) {
                row[c] = 1;
            }
        }
        model.categorical_table[r] = row;
        for (c, role) in model.role_domain.iter().enumerate() {
            let _ = role;
            if c < 8 && row[c] == 1 {
                cat_hits += examples.iter().filter(|e| e.relation == rel).count();
            }
        }
    }
    let _ = cat_hits;
    // Learned continuation decision over `content is a key of the world`.
    let mut decisions: BTreeMap<bool, BTreeMap<bool, usize>> = BTreeMap::new();
    for e in examples {
        let content_is_key = e.content_is_entity;
        *decisions
            .entry(content_is_key)
            .or_default()
            .entry(e.depth >= 2)
            .or_default() += 1;
    }
    let policy_initial = examples
        .iter()
        .filter(|e| model.should_continue(e.content_is_entity) == (e.depth >= 2))
        .count();
    for (feature, counts) in decisions.iter() {
        if let Some((decision, _)) = counts.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0))) {
            model.continue_policy[usize::from(*feature)] = *decision;
        }
    }
    let policy_final = examples
        .iter()
        .filter(|e| model.should_continue(e.content_is_entity) == (e.depth >= 2))
        .count();
    let report = RelFitReport {
        examples: examples.len(),
        relations: model.relation_domain.len(),
        roles: model.role_domain.len(),
        scorer_initial: initial,
        scorer_final: correct(&model),
        follow_entries: model.follow.len(),
        relation_pairs_learned: pairing.len(),
        policy_initial,
        policy_final,
        accepted_moves: accepted,
    };
    (model, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(relation: u32, entity: u32, role: u32, value: u32, depth: u8) -> RelExample {
        RelExample {
            relation,
            entity,
            first_role: role,
            first_value: value,
            follow_relation: relation + 1,
            second_role: role + 1,
            second_value: value + 100,
            depth,
            content_is_entity: value < 40,
        }
    }

    fn dev() -> Vec<RelExample> {
        // Four relations, four roles, one role answering each relation; two are entity-valued.
        vec![
            example(200, 10, 300, 11, 2),
            example(201, 10, 301, 12, 2),
            example(202, 10, 302, 40, 1),
            example(203, 10, 303, 41, 1),
            example(200, 11, 300, 12, 2),
            example(201, 11, 301, 10, 2),
            example(202, 11, 302, 42, 1),
            example(203, 11, 303, 43, 1),
        ]
    }

    #[test]
    fn the_learned_compatibility_selects_the_answered_role() {
        let ex = dev();
        let (m, report) = learn_relational_model(&ex, false, false);
        assert_eq!(
            report.scorer_final,
            ex.len(),
            "every observed role must match"
        );
        for e in ex.iter() {
            assert!(m.relation_matches(e.relation, e.first_role) > 0);
            for role in m.role_domain.iter() {
                if *role != e.first_role {
                    assert!(m.relation_matches(e.relation, *role) < 0);
                }
            }
        }
        let candidates = [
            Candidate {
                role: 301,
                key: 10,
                value: 12,
                exact_key: true,
            },
            Candidate {
                role: 300,
                key: 10,
                value: 11,
                exact_key: true,
            },
        ];
        assert_eq!(rank(&m, 200, false, &candidates), Some(1));
        assert_eq!(rank(&m, 201, false, &candidates), Some(0));
    }

    #[test]
    fn the_categorical_control_is_fitted_from_the_same_supervision() {
        let ex = dev();
        let (m, _) = learn_relational_model(&ex, false, true);
        assert!(m.categorical);
        for e in ex.iter() {
            assert!(m.relation_matches(e.relation, e.first_role) > 0);
            for role in m.role_domain.iter() {
                if *role != e.first_role {
                    assert!(m.relation_matches(e.relation, *role) < 0);
                }
            }
        }
    }

    #[test]
    fn the_follow_map_and_continuation_policy_are_learned() {
        let ex = dev();
        let (m, report) = learn_relational_model(&ex, false, false);
        assert_eq!(
            report.follow_entries, 2,
            "only entity-valued relations continue"
        );
        assert!(m.should_continue(true));
        assert!(!m.should_continue(false));
        assert_eq!(m.follow_of(200), Some(201));
        assert_eq!(m.follow_of(202), None);
    }

    #[test]
    fn the_model_and_frame_round_trip_and_reject_malformed_bytes() {
        let ex = dev();
        let (m, _) = learn_relational_model(&ex, false, false);
        let back = RelationalModel::from_bytes(&m.to_bytes(), 4096).unwrap();
        assert_eq!(back, m);
        let mut bad = m.to_bytes();
        bad[0] = b'X';
        assert!(RelationalModel::from_bytes(&bad, 4096).is_err());
        let mut bad = m.to_bytes();
        bad.truncate(bad.len() - 1);
        assert!(RelationalModel::from_bytes(&bad, 4096).is_err());
        let binding = RelBinding {
            world_namespace: 1,
            ..RelBinding::default()
        };
        let mut frame = RelFrame::start(200, 10, binding);
        let origin = CapturedPayload {
            seq: 1,
            abs: 4,
            payload: 11,
            version: 7,
        };
        frame.capture(origin).unwrap();
        let bytes = frame.to_bytes().unwrap();
        let mut back = RelFrame::from_bytes(&bytes, 4096, &binding).unwrap();
        assert_eq!(back, frame);
        assert!(back.origin_is_live(Some(origin)));
        assert!(!back.origin_is_live(Some(CapturedPayload {
            version: 8,
            ..origin
        })));
        assert!(!back.origin_is_live(Some(CapturedPayload { seq: 2, ..origin })));
        assert_eq!(back.retained, Some(11));
        back.pending = Some(RelAction::Emit);
        assert_eq!(back.emit(), Ok(11));
        assert!(back.capture(origin).is_err());
        let other = RelBinding {
            world_namespace: 2,
            ..binding
        };
        assert!(RelFrame::from_bytes(&bytes, 4096, &other).is_err());
        let mut malformed = frame.clone();
        malformed.captured = None;
        malformed.retained = None;
        malformed.hops = 0;
        malformed.terminal = Some(RelAction::Stop);
        assert!(RelFrame::from_bytes(&malformed.to_bytes().unwrap(), 4096, &binding).is_err());
        // Clearing the pending phase still cannot make an empty Stop a completed answer.
        malformed.pending = None;
        assert!(RelFrame::from_bytes(&malformed.to_bytes().unwrap(), 4096, &binding).is_err());
    }

    #[test]
    fn group_identity_and_cyclic_zero_have_their_actual_meaning() {
        let (mut m, _) = learn_relational_model(&dev(), false, false);
        m.relation_code[0] = 7;
        m.role_code[0] = 7;
        assert_eq!(m.relative(200, 300), Some(group_table().identity as usize));
        assert_eq!(m.relation_matches(200, 300), 1);
        m.cyclic = true;
        assert_eq!(m.relative(200, 300), Some(0));
        assert_eq!(m.relation_matches(200, 300), 1);
    }

    #[test]
    fn policy_features_are_not_reconstructed_from_action_labels() {
        let mut ex = dev();
        // A conflicting observation must remain visible instead of being rewritten from its label.
        ex[0].content_is_entity = false;
        let (_, report) = learn_relational_model(&ex, false, false);
        assert_eq!(report.policy_initial, 4);
        assert_eq!(report.policy_final, 7);
    }
}
