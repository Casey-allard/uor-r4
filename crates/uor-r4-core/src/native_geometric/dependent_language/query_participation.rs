//! Learned query-evidence participation and dependent replacement eligibility.
//! Exact anchored occurrence context is observed before candidate selection.
//! Neither table supplies an answer, span, source, or replacement index.
use super::{
    completion, correspondence, occurrence, occurrence_role, runtime as binding, scheduling,
    span_boundary,
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime as lexical,
    ordered_state::runtime as ordered,
    relational_attention::runtime::{Error, Result},
    relative_language::runtime as reader,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_ROWS: usize = 1024;
pub const MAX_TRAINING_ROWS: usize = 8192;
const UNKNOWN: u8 = 64;
const EDGE: u8 = 65;
const ANY: u8 = 66;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub center: u8,
    pub left: u8,
    pub right: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: occurrence_role::Artifact,
    pub parent_digest: [u8; 32],
    pub anchors: Vec<span_boundary::Word>,
    pub optional: Vec<Key>,
    pub replacement: Vec<Key>,
    pub data_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub training: String,
}

fn validate_table(table: &[Key], anchors: usize) -> Result<()> {
    let neighbor = |v: u8| usize::from(v) < anchors || matches!(v, UNKNOWN | EDGE | ANY);
    if anchors > span_boundary::MAX_CONTEXT_WORDS
        || table.len() > MAX_ROWS
        || table.iter().enumerate().any(|(i, k)| {
            usize::from(k.center) >= anchors
                || !neighbor(k.left)
                || !neighbor(k.right)
                || (i > 0 && table[i - 1] >= *k)
        })
    {
        return Err(Error::Artifact);
    }
    Ok(())
}

impl Artifact {
    pub fn reader(&self) -> &reader::Artifact {
        self.parent.reader()
    }
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        self.parent.validate(g)?;
        if self.schema != 1
            || self.parent_digest != *blake3::hash(&self.parent.encode()?).as_bytes()
        {
            return Err(Error::Artifact);
        }
        span_boundary::validate(g, &self.anchors)?;
        validate_table(&self.optional, self.anchors.len())?;
        validate_table(&self.replacement, self.anchors.len())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| Error::Artifact)
    }
    pub fn decode(bytes: &[u8], g: &BoundGeometry) -> Result<Self> {
        if bytes.len() > correspondence::MAX_ARTIFACT_BYTES {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(bytes).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
    pub fn required_mask(
        &self,
        g: &BoundGeometry,
        m: &Metric,
        query: &[u8],
        exact: bool,
    ) -> Result<Vec<bool>> {
        Ok(observations(self, g, m, query, exact)?
            .iter()
            .map(|k| !self.optional.iter().any(|r| matches_key(r, k)))
            .collect())
    }
    pub fn replacement_mask(
        &self,
        g: &BoundGeometry,
        m: &Metric,
        query: &[u8],
        exact: bool,
    ) -> Result<Vec<bool>> {
        Ok(observations(self, g, m, query, exact)?
            .iter()
            .map(|k| self.replacement.iter().any(|r| matches_key(r, k)))
            .collect())
    }
}

/// Only queries supplied for training create anchors. Evaluation cannot grow
/// this inventory. The same identity in two roles remains distinct by neighbors.
pub fn initialize(
    parent: occurrence_role::Artifact,
    g: &BoundGeometry,
    training_queries: &[Vec<u8>],
    source_digest: [u8; 32],
) -> Result<Artifact> {
    parent.validate(g)?;
    if training_queries.is_empty() || training_queries.len() > MAX_TRAINING_ROWS {
        return Err(Error::Shape);
    }
    let mut words = BTreeSet::new();
    for q in training_queries {
        let query = reader::words(g, q, ordered::CANONICAL)?;
        if query.is_empty() || query.len() > reader::MAX_WORDS || q.len() > 256 {
            return Err(Error::Shape);
        }
        words.extend(query.into_iter().map(|w| w.bytes));
    }
    let anchors = words
        .into_iter()
        .map(|bytes| {
            Ok(span_boundary::Word {
                geometry: ordered::Query::encode(g, &bytes, ordered::CANONICAL)?,
                bytes,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let a = Artifact {
        schema: 1,
        parent_digest: *blake3::hash(&parent.encode()?).as_bytes(),
        parent,
        anchors,
        optional: vec![],
        replacement: vec![],
        data_digest: *blake3::hash(
            &serde_json::to_vec(training_queries).map_err(|_| Error::Artifact)?,
        )
        .as_bytes(),
        source_digest,
        training: "Frozen parent source/query roles and operators. Query-only canonical training anchors. Required evidence defaults true; replacement eligibility defaults false. Two separate output-credit fits; unknown observations cannot acquire a positive rule.".into(),
    };
    a.validate(g)?;
    Ok(a)
}

fn key(ids: &[u8], i: usize) -> Key {
    Key {
        center: ids[i],
        left: if i == 0 { EDGE } else { ids[i - 1] },
        right: if i + 1 == ids.len() { EDGE } else { ids[i + 1] },
    }
}
pub fn observations(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    raw: &[u8],
    exact: bool,
) -> Result<Vec<Key>> {
    if m.geometry_digest() != g.id() {
        return Err(Error::Geometry);
    }
    let words = reader::words(g, raw, ordered::CANONICAL)?;
    if words.is_empty() || words.len() > reader::MAX_WORDS || raw.len() > 256 {
        return Err(Error::Shape);
    }
    let mut ids = Vec::with_capacity(words.len());
    for w in words {
        let mut identity = UNKNOWN;
        for (i, known) in a.anchors.iter().enumerate() {
            if span_boundary::contains(m, std::slice::from_ref(known), &w.geometry, exact)? {
                identity = i as u8;
                break;
            }
        }
        ids.push(identity);
    }
    Ok((0..ids.len()).map(|i| key(&ids, i)).collect())
}
fn matches_key(rule: &Key, observed: &Key) -> bool {
    rule.center == observed.center
        && (rule.left == ANY || rule.left == observed.left)
        && (rule.right == ANY || rule.right == observed.right)
}

/// Count index zero is positive rule credit, index one is contrary credit.
/// Retain all maximally general rules admitting no contrary training credit.
fn induce(counts: &BTreeMap<Key, [usize; 2]>, blockers: &BTreeSet<Key>) -> Vec<Key> {
    let mut options = BTreeSet::new();
    for (k, c) in counts {
        if c[0] == 0 || k.center == UNKNOWN {
            continue;
        }
        for left in [k.left, ANY] {
            for right in [k.right, ANY] {
                let rule = Key {
                    center: k.center,
                    left,
                    right,
                };
                if !blockers.iter().any(|k| matches_key(&rule, k))
                    && !counts
                        .iter()
                        .any(|(other, credit)| credit[1] > 0 && matches_key(&rule, other))
                {
                    options.insert(rule);
                }
            }
        }
    }
    options
        .iter()
        .filter(|r| {
            !options
                .iter()
                .any(|other| other != *r && matches_key(other, r))
        })
        .cloned()
        .collect()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectExample {
    pub records: [Vec<u8>; 4],
    pub question: Vec<u8>,
    pub answer: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplacementExample {
    pub records: [Vec<u8>; 4],
    pub question: Vec<u8>,
    /// Actual first-read payload collected by the training report.
    pub payload: Vec<u8>,
    pub answer: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credit {
    pub key: Key,
    pub positive: usize,
    pub contrary: usize,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Fit {
    pub rows: usize,
    pub compatible_alternatives: usize,
    pub skipped_no_support: usize,
    pub skipped_ambiguous_occurrences: usize,
    pub ambiguous_output_rows: usize,
    pub conflicts: usize,
    pub learned_rows: usize,
    pub bootstrap_recovered_rows: usize,
    pub ambiguity_blockers: Vec<Key>,
    pub credits: Vec<Credit>,
}
fn finish_fit(
    counts: BTreeMap<Key, [usize; 2]>,
    blockers: BTreeSet<Key>,
    f: &mut Fit,
) -> Result<Vec<Key>> {
    if counts.len() + blockers.len() > MAX_ROWS {
        return Err(Error::Artifact);
    }
    let table = induce(&counts, &blockers);
    if table.len() > MAX_ROWS {
        return Err(Error::Artifact);
    }
    f.learned_rows = table.len();
    f.ambiguity_blockers = blockers.into_iter().collect();
    for (key, c) in counts {
        f.conflicts += usize::from(c[0] > 0 && c[1] > 0);
        f.credits.push(Credit {
            key,
            positive: c[0],
            contrary: c[1],
        });
    }
    Ok(table)
}
fn charge_data<T: Serialize>(a: &mut Artifact, data: &T) -> Result<()> {
    a.data_digest = *blake3::hash(
        &[
            a.data_digest.as_slice(),
            serde_json::to_vec(data)
                .map_err(|_| Error::Artifact)?
                .as_slice(),
        ]
        .concat(),
    )
    .as_bytes();
    Ok(())
}

/// Direct byte-answer credit supplies unanimous query participation across all
/// compatible latent witnesses. A mixture of present and absent support cannot
/// label an occurrence optional or required. No grammar/name list is supplied.
pub fn fit_required(
    mut a: Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[DirectExample],
) -> Result<(Artifact, Fit)> {
    a.validate(g)?;
    if train.is_empty() || train.len() > MAX_TRAINING_ROWS {
        return Err(Error::Shape);
    }
    let mut counts: BTreeMap<Key, [usize; 2]> = BTreeMap::new();
    let mut blockers = BTreeSet::new();
    let mut f = Fit {
        rows: train.len(),
        ..Fit::default()
    };
    let mut supported = vec![false; train.len()];
    // At most two passes. Revisit only unreachable rows using the first-pass
    // optional table; suppressed observations cannot provide self-confirmation.
    for pass in 0..2 {
        for (row, e) in train.iter().enumerate() {
            if supported[row] {
                continue;
            }
            let required = if pass == 1 {
                Some(a.required_mask(g, m, &e.question, false)?)
            } else {
                None
            };
            let contexts = e
                .records
                .iter()
                .map(|r| occurrence_role::source_context(&a.parent, g, m, r, false))
                .collect::<Result<Vec<_>>>()?;
            let contexts: [Vec<bool>; 4] = contexts.try_into().map_err(|_| Error::Shape)?;
            let query_roles = occurrence_role::query_context(&a.parent, g, m, &e.question, false)?;
            let search = occurrence::candidates_with_context_required(
                &a.parent.parent.parent,
                g,
                m,
                &e.records,
                &e.question,
                occurrence::Control::Full,
                Some(&contexts),
                required.as_deref(),
            )?;
            let keys = observations(&a, g, m, &e.question, false)?;
            let mut support = vec![[0usize; 2]; keys.len()];
            let mut compatible = 0;
            for candidate in search.candidates {
                if candidate.span.value.bytes != e.answer {
                    continue;
                }
                let out = completion::generate_routed(
                    &a.parent.parent.parent.parent,
                    g,
                    m,
                    &e.records,
                    &e.question,
                    completion::Control::Full,
                    scheduling::Control::Full,
                    |_, _| {
                        Ok(lexical::Route {
                            status: lexical::RouteStatus::Selected,
                            selected: Some(candidate.span.value.clone()),
                            compatible: vec![[
                                candidate.span.value.source,
                                candidate.span.value.word,
                            ]],
                        })
                    },
                )?;
                let target: Vec<u16> = e
                    .answer
                    .iter()
                    .copied()
                    .map(u16::from)
                    .chain([256])
                    .collect();
                if out.outcome != completion::Outcome::Answered
                    || out.trace.exhausted
                    || out.trace.tokens != target
                {
                    continue;
                }
                let source_roles = &contexts[candidate.span.value.source];
                for witness in candidate.witnesses {
                    let witness_roles: Vec<_> = witness
                        .query_to_source
                        .iter()
                        .map(|j| j.map(|j| source_roles[j]).unwrap_or(false))
                        .collect();
                    if witness
                        .query_to_source
                        .iter()
                        .enumerate()
                        .any(|(i, j)| j.is_some() && query_roles[i] != witness_roles[i])
                    {
                        continue;
                    }
                    let sig = correspondence::run_signature(
                        &witness.query_to_source,
                        &witness_roles,
                        source_roles,
                    )?;
                    let features = witness.features
                        | if sig.ordered_unit_adjacency {
                            correspondence::STRUCTURE_BIT
                        } else {
                            0
                        };
                    if !a.parent.parent.matches(features) {
                        continue;
                    }
                    compatible += 1;
                    for (i, j) in witness.query_to_source.iter().enumerate() {
                        support[i][usize::from(j.is_some())] += 1;
                    }
                }
            }
            f.compatible_alternatives += compatible;
            if compatible == 0 {
                continue;
            }
            supported[row] = true;
            f.bootstrap_recovered_rows += usize::from(pass == 1);
            for (i, (k, s)) in keys.into_iter().zip(support).enumerate() {
                if required.as_ref().is_some_and(|mask| !mask[i]) {
                    continue;
                }
                if s[0] > 0 && s[1] > 0 {
                    f.skipped_ambiguous_occurrences += 1;
                    blockers.insert(k);
                    continue;
                }
                counts.entry(k).or_default()[usize::from(s[1] > 0)] += 1;
            }
        }
        if counts.len() + blockers.len() > MAX_ROWS {
            return Err(Error::Artifact);
        }
        a.optional = induce(&counts, &blockers);
    }
    f.skipped_no_support = supported.iter().filter(|&&yes| !yes).count();
    a.optional = finish_fit(counts, blockers, &mut f)?;
    charge_data(&mut a, &train)?;
    a.training.push_str(" Evidence participation fit: every frozen-role, frozen-selector, actual direct-byte/EOS-compatible candidate witness contributes present/absent query support; only unanimous support is credited. Optional local-context rules generalize only without contrary training credit or an ambiguous-support key. One bootstrap revisit of previously unsupported rows uses the first-pass optional table, retaining every original contrary credit and never crediting the suppressed positions themselves. Source tables, query role tables and parent selector unchanged.");
    a.validate(g)?;
    Ok((a, f))
}

/// Every inherited eligible splice is evaluated by actual suffix generation.
/// More than one correct splice supplies no positive site credit: answer bytes
/// do not select one of several occurrence identities. Incompatible sites alone
/// supply contrary credit in that case.
pub fn fit_replacements(
    mut a: Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[ReplacementExample],
) -> Result<(Artifact, Fit)> {
    a.validate(g)?;
    if train.is_empty() || train.len() > MAX_TRAINING_ROWS {
        return Err(Error::Shape);
    }
    let mut counts: BTreeMap<Key, [usize; 2]> = BTreeMap::new();
    let mut blockers = BTreeSet::new();
    let mut f = Fit {
        rows: train.len(),
        ..Fit::default()
    };
    for e in train {
        let keys = observations(&a, g, m, &e.question, false)?;
        let binding = &a.parent.parent.parent.parent.parent.parent;
        let updates = binding::updates_with_window(
            binding,
            g,
            m,
            &e.records,
            &e.question,
            &e.payload,
            binding::Control::Full,
            binding::PayloadWindow::Phrase,
        )?;
        let mut credits = Vec::new();
        for u in updates {
            if !binding.matches(u.features) {
                continue;
            }
            let generated = generate(
                &a,
                g,
                m,
                &e.records,
                &u.question,
                Control::ReplacementDisabled,
            )?;
            let target: Vec<u16> = e
                .answer
                .iter()
                .copied()
                .map(u16::from)
                .chain(std::iter::once(256))
                .collect();
            let correct = generated.outcome == completion::Outcome::Answered
                && !generated.trace.exhausted
                && generated.trace.tokens == target;
            credits.push((u.word, correct));
        }
        let yes = credits.iter().filter(|(_, ok)| *ok).count();
        f.compatible_alternatives += yes;
        f.skipped_no_support += usize::from(yes == 0);
        f.ambiguous_output_rows += usize::from(yes > 1);
        for (word, correct) in credits {
            let k = keys.get(word).ok_or(Error::Shape)?.clone();
            if correct && yes != 1 {
                f.skipped_ambiguous_occurrences += 1;
                blockers.insert(k);
                continue;
            }
            counts.entry(k).or_default()[usize::from(!correct)] += 1;
        }
    }
    a.replacement = finish_fit(counts, blockers, &mut f)?;
    charge_data(&mut a, &train)?;
    a.training.push_str(" Replacement fit: actual inherited-admitted splices with actual first-read payload are evaluated by complete suffix byte/EOS generation using learned evidence participation. Only a unique output-compatible splice gives positive eligibility credit. Ambiguous compatible sites block any covering positive rule, including wildcard generalization; incompatible sites provide contrary credit. Runtime intersects eligibility with the inherited updater, preserving exact occurrence indices.");
    a.validate(g)?;
    Ok((a, f))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Full,
    ExactIdentity,
    RequirementsDisabled,
    ReplacementDisabled,
    BothDisabled,
    ReadDisabled,
    UpdateDisabled,
}
fn parent_control(c: Control) -> occurrence_role::Control {
    match c {
        Control::ExactIdentity => occurrence_role::Control::ExactIdentity,
        Control::ReadDisabled => occurrence_role::Control::ReadDisabled,
        Control::UpdateDisabled => occurrence_role::Control::UpdateDisabled,
        _ => occurrence_role::Control::Full,
    }
}
pub fn route(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<lexical::Route> {
    if matches!(
        c,
        Control::RequirementsDisabled | Control::BothDisabled | Control::ReadDisabled
    ) {
        return occurrence_role::route(&a.parent, g, m, records, question, parent_control(c));
    }
    let required = a.required_mask(g, m, question, c == Control::ExactIdentity)?;
    occurrence_role::route_with_required(
        &a.parent,
        g,
        m,
        records,
        question,
        parent_control(c),
        Some(&required),
    )
}
pub fn allow_update(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    question: &[u8],
    update: &binding::UpdateCandidate,
    c: Control,
) -> Result<bool> {
    if matches!(c, Control::ReplacementDisabled | Control::BothDisabled) {
        return Ok(true);
    }
    let mask = a.replacement_mask(g, m, question, c == Control::ExactIdentity)?;
    mask.get(update.word).copied().ok_or(Error::Shape)
}
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    c: Control,
) -> Result<completion::Generated> {
    a.validate(g)?;
    if c == Control::BothDisabled {
        return occurrence_role::generate(
            &a.parent,
            g,
            m,
            records,
            prompt,
            occurrence_role::Control::Full,
        );
    }
    completion::generate_routed_payload_updates(
        &a.parent.parent.parent.parent,
        g,
        m,
        records,
        prompt,
        completion::Control::Full,
        if c == Control::UpdateDisabled {
            scheduling::Control::UpdateDisabled
        } else {
            scheduling::Control::Full
        },
        binding::PayloadWindow::Phrase,
        |b, _| Ok(b.to_vec()),
        |q, _| route(a, g, m, records, q, c),
        |q, u| allow_update(a, g, m, q, u, c),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_identity_and_neighbor_roles_remain_distinct() {
        let optional_prefix = key(&[3, 4, 5, 6], 0);
        let name = key(&[7, 3, 8], 1);
        assert_eq!(optional_prefix.center, name.center);
        assert_ne!(optional_prefix, name);
        assert_ne!(key(&[UNKNOWN, 3], 1), key(&[3], 0));
        assert_ne!(UNKNOWN, EDGE);
        assert_ne!(ANY, UNKNOWN);
    }
    #[test]
    fn contradictory_support_blocks_optional_and_replacement_generalization() {
        let prefix = Key {
            center: 3,
            left: EDGE,
            right: 4,
        };
        let name = Key {
            center: 3,
            left: 7,
            right: 8,
        };
        let ambiguous = Key {
            center: 3,
            left: EDGE,
            right: 8,
        };
        let counts = BTreeMap::from([
            (prefix.clone(), [3, 0]),
            (name.clone(), [0, 2]),
            (ambiguous.clone(), [1, 1]),
        ]);
        let rules = induce(&counts, &BTreeSet::new());
        assert!(rules.iter().any(|r| matches_key(r, &prefix)));
        assert!(!rules.iter().any(|r| matches_key(r, &name)));
        assert!(!rules.iter().any(|r| matches_key(r, &ambiguous)));
        assert!(!rules.iter().any(|r| matches_key(
            r,
            &Key {
                center: UNKNOWN,
                left: EDGE,
                right: 4
            }
        )));
    }
    #[test]
    fn ambiguous_uncredited_keys_block_positive_wildcards() {
        let known = Key {
            center: 2,
            left: 3,
            right: 4,
        };
        let ambiguous = Key {
            center: 2,
            left: 3,
            right: 5,
        };
        let counts = BTreeMap::from([(known.clone(), [3, 0])]);
        let blockers = BTreeSet::from([ambiguous.clone()]);
        let rules = induce(&counts, &blockers);
        assert!(rules.iter().any(|r| matches_key(r, &known)));
        assert!(!rules.iter().any(|r| matches_key(r, &ambiguous)));
        assert!(induce(&counts, &BTreeSet::from([known])).is_empty());
    }
    #[test]
    fn unknown_centers_and_duplicate_or_out_of_range_rows_are_invalid() {
        let valid = Key {
            center: 0,
            left: EDGE,
            right: ANY,
        };
        assert!(validate_table(std::slice::from_ref(&valid), 1).is_ok());
        assert!(validate_table(&[valid.clone(), valid], 1).is_err());
        assert!(validate_table(
            &[Key {
                center: UNKNOWN,
                left: EDGE,
                right: ANY
            }],
            64
        )
        .is_err());
        assert!(validate_table(
            &[Key {
                center: 0,
                left: 67,
                right: ANY
            }],
            1
        )
        .is_err());
    }
}
