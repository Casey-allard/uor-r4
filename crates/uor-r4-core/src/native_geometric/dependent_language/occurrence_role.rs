//! Output-credit learned local occurrence roles on exact geometric anchors.
//! The prior remains the fallback. No answer-span or lexical exception enters
//! role observation. Query roles are transported by each injective witness.
use super::{
    completion, correspondence, occurrence, runtime::PayloadWindow, scheduling, span_boundary,
    span_data, span_learning,
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
use std::collections::BTreeMap;
pub const MAX_ROWS: usize = 1024;
const CONTENT: u8 = 64;
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
pub struct Row {
    pub key: Key,
    pub context: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: correspondence::Artifact,
    pub parent_digest: [u8; 32],
    pub table: Vec<Row>,
    pub query_table: Vec<Row>,
    #[serde(default)]
    pub require_available_query_coverage: bool,
    pub anchors: Vec<span_boundary::Word>,
    pub data_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn reader(&self) -> &reader::Artifact {
        self.parent.reader()
    }
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        self.parent.validate(g)?;
        if !matches!(self.schema, 2 | 3)
            || (self.schema == 2 && self.require_available_query_coverage)
            || self.table.len() > MAX_ROWS
            || self.query_table.len() > MAX_ROWS
            || self.parent_digest != *blake3::hash(&self.parent.encode()?).as_bytes()
        {
            return Err(Error::Artifact);
        }
        span_boundary::validate(g, &self.anchors)?;
        let n = self.parent.parent.context_words.len();
        let valid_anchor =
            |v: u8| usize::from(v) < self.anchors.len() || v == CONTENT || v == EDGE || v == ANY;
        for table in [&self.table, &self.query_table] {
            for (i, r) in table.iter().enumerate() {
                if r.context
                    || usize::from(r.key.center) >= n
                    || !valid_anchor(r.key.left)
                    || !valid_anchor(r.key.right)
                    || (i > 0 && table[i - 1].key >= r.key)
                {
                    return Err(Error::Artifact);
                }
            }
        }
        Ok(())
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
}
/// Identity indices are artifact-bound canonical geometry, not semantic distances.
fn observations(
    a: &correspondence::Artifact,
    anchors: &[span_boundary::Word],
    g: &BoundGeometry,
    m: &Metric,
    raw: &[u8],
    exact: bool,
) -> Result<Vec<Key>> {
    if m.geometry_digest() != g.id() {
        return Err(Error::Geometry);
    }
    let words = reader::words(g, raw, ordered::CANONICAL)?;
    if words.len() > reader::MAX_WORDS {
        return Err(Error::Shape);
    }
    let identify = |known: &[span_boundary::Word], w: &ordered::Query| -> Result<u8> {
        for (i, k) in known.iter().enumerate() {
            if span_boundary::contains(m, std::slice::from_ref(k), w, exact)? {
                return Ok(i as u8);
            }
        }
        Ok(CONTENT)
    };
    let centers: Vec<_> = words
        .iter()
        .map(|w| identify(&a.parent.context_words, &w.geometry))
        .collect::<Result<_>>()?;
    let ids: Vec<_> = words
        .iter()
        .map(|w| identify(anchors, &w.geometry))
        .collect::<Result<_>>()?;
    Ok(centers
        .into_iter()
        .enumerate()
        .map(|(i, center)| {
            let mut k = key(&ids, i);
            k.center = center;
            k
        })
        .collect())
}
fn learn_anchors(
    parent: &correspondence::Artifact,
    g: &BoundGeometry,
    train: &[span_data::Example],
) -> Result<Vec<span_boundary::Word>> {
    let mut words: std::collections::BTreeSet<Vec<u8>> = parent
        .parent
        .context_words
        .iter()
        .map(|w| w.bytes.clone())
        .collect();
    for e in train {
        for record in &e.records {
            for w in reader::words(g, record, ordered::CANONICAL)? {
                words.insert(w.bytes);
            }
        }
    }
    let anchors: Vec<_> = words
        .into_iter()
        .map(|bytes| {
            Ok(span_boundary::Word {
                geometry: ordered::Query::encode(g, &bytes, ordered::CANONICAL)?,
                bytes,
            })
        })
        .collect::<Result<_>>()?;
    span_boundary::validate(g, &anchors)?;
    Ok(anchors)
}
fn key(ids: &[u8], i: usize) -> Key {
    Key {
        center: ids[i],
        left: if i == 0 { EDGE } else { ids[i - 1] },
        right: if i + 1 == ids.len() { EDGE } else { ids[i + 1] },
    }
}
fn matches_key(rule: &Key, observation: &Key) -> bool {
    rule.center == observation.center
        && (rule.left == ANY || rule.left == observation.left)
        && (rule.right == ANY || rule.right == observation.right)
}
fn induce(counts: &BTreeMap<Key, [usize; 2]>) -> Vec<Row> {
    let mut options = std::collections::BTreeSet::new();
    for (key, count) in counts {
        if count[0] == 0 {
            continue;
        }
        for left in [key.left, ANY] {
            for right in [key.right, ANY] {
                let r = Key {
                    center: key.center,
                    left,
                    right,
                };
                if !counts.iter().any(|(k, c)| c[1] > 0 && matches_key(&r, k)) {
                    options.insert(r);
                }
            }
        }
    }
    // Retain only maximally general rules consistent with every negative credit.
    options
        .iter()
        .filter(|r| {
            !options
                .iter()
                .any(|other| other != *r && matches_key(other, r))
        })
        .map(|key| Row {
            key: key.clone(),
            context: false,
        })
        .collect()
}
pub fn source_context(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    record: &[u8],
    exact: bool,
) -> Result<Vec<bool>> {
    roles(
        &a.table,
        &observations(&a.parent, &a.anchors, g, m, record, exact)?,
    )
}
fn roles(table: &[Row], observations: &[Key]) -> Result<Vec<bool>> {
    Ok(observations
        .iter()
        .map(|k| k.center != CONTENT && !table.iter().any(|r| matches_key(&r.key, k)))
        .collect())
}
pub fn query_context(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    question: &[u8],
    exact: bool,
) -> Result<Vec<bool>> {
    roles(
        &a.query_table,
        &observations(&a.parent, &a.anchors, g, m, question, exact)?,
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credit {
    pub key: Key,
    pub content: usize,
    pub context: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub rows: usize,
    pub compatible_spans: usize,
    pub skipped_ambiguous_occurrences: usize,
    pub skipped_multi_source_rows: usize,
    pub conflicts: usize,
    pub learned_content_rows: usize,
    pub credits: Vec<Credit>,
}
/// Only unanimous membership across all output-compatible latent spans supplies
/// role credit. Equal emitted bytes never choose an occurrence. Conflict retains
/// the prior and is reported; no positive-overrides-negative policy is used.
pub fn fit(
    parent: correspondence::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[span_data::Example],
) -> Result<(Artifact, Fit)> {
    parent.validate(g)?;
    if train.is_empty() {
        return Err(Error::Shape);
    }
    let anchors = learn_anchors(&parent, g, train)?;
    let prepared = span_learning::prepare(&parent.parent, g, m, train)?;
    let mut counts: BTreeMap<Key, [usize; 2]> = BTreeMap::new();
    let mut f = Fit {
        rows: train.len(),
        compatible_spans: 0,
        skipped_ambiguous_occurrences: 0,
        skipped_multi_source_rows: 0,
        conflicts: 0,
        learned_content_rows: 0,
        credits: vec![],
    };
    for (e, p) in train.iter().zip(prepared) {
        let yes: Vec<_> = p
            .candidates
            .iter()
            .zip(&p.compatible)
            .filter_map(|(c, &ok)| ok.then_some(c))
            .collect();
        f.compatible_spans += yes.len();
        let first = yes.first().ok_or(Error::Shape)?;
        if yes.iter().any(|c| c.value.source != first.value.source) {
            f.skipped_multi_source_rows += 1;
            continue;
        }
        let keys = observations(
            &parent,
            &anchors,
            g,
            m,
            &e.records[first.value.source],
            false,
        )?;
        for (i, k) in keys.iter().enumerate() {
            if k.center == CONTENT {
                continue;
            }
            let inside = yes
                .iter()
                .filter(|c| i >= c.value.word && i < c.last_word)
                .count();
            if inside != 0 && inside != yes.len() {
                f.skipped_ambiguous_occurrences += 1;
                continue;
            }
            let count = counts.entry(k.clone()).or_default();
            count[usize::from(inside == 0)] += 1;
        }
    }
    if counts.len() > MAX_ROWS {
        return Err(Error::Artifact);
    }
    let table = induce(&counts);
    if table.len() > MAX_ROWS {
        return Err(Error::Artifact);
    }
    f.learned_content_rows = table.len();
    for (k, c) in counts {
        f.conflicts += usize::from(c[0] > 0 && c[1] > 0);
        f.credits.push(Credit {
            key: k.clone(),
            content: c[0],
            context: c[1],
        });
    }
    let a=Artifact{schema:2,parent_digest:*blake3::hash(&parent.encode()?).as_bytes(),parent,table,query_table:vec![],require_available_query_coverage:false,anchors,data_digest:*blake3::hash(&serde_json::to_vec(train).map_err(|_|Error::Artifact)?).as_bytes(),source_digest:*blake3::hash(concat!(include_str!("occurrence_role.rs"),include_str!("occurrence.rs"),include_str!("span_learning.rs")).as_bytes()).as_bytes(),training:"One bounded local-role count and consistent-neighbor-pattern fit using actual byte/EOS-compatible latent spans. Center inherited context identity and immediate neighboring exact canonical training-word anchors; unknown-neighbor and record-edge markers. All training words are retained as anchors, regardless of their prior role. Unanimous occurrence membership only. Each center identity is retained; either neighboring observation can be wildcarded only when no context-labeled credit is admitted. All maximally general consistent content rules are retained. Conflicts and uncovered observations retain inherited context. Parent parameters frozen. Learned lexical-context recombination, not identity-independent or general-language transfer.".into()};
    a.validate(g)?;
    Ok((a, f))
}
/// Learn query occurrence roles from unanimous matches outside every actual
/// output-compatible span, using the frozen learned source role map. Unlike
/// source credit, being outside the answer is not treated as a context label.
pub fn fit_queries(
    mut a: Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[span_data::Example],
) -> Result<(Artifact, Fit)> {
    a.validate(g)?;
    if train.is_empty() {
        return Err(Error::Shape);
    }
    let prepared = span_learning::prepare(&a.parent.parent, g, m, train)?;
    let mut counts: BTreeMap<Key, [usize; 2]> = BTreeMap::new();
    let mut f = Fit {
        rows: train.len(),
        compatible_spans: 0,
        skipped_ambiguous_occurrences: 0,
        skipped_multi_source_rows: 0,
        conflicts: 0,
        learned_content_rows: 0,
        credits: vec![],
    };
    for (e, p) in train.iter().zip(prepared) {
        let yes: Vec<_> = p
            .candidates
            .iter()
            .zip(&p.compatible)
            .filter_map(|(c, &ok)| ok.then_some(c))
            .collect();
        f.compatible_spans += yes.len();
        if yes.is_empty() {
            return Err(Error::Shape);
        }
        let keys = observations(&a.parent, &a.anchors, g, m, &e.prompt, false)?;
        let q = reader::words(g, &e.prompt, ordered::CANONICAL)?;
        for (i, k) in keys.iter().enumerate() {
            if k.center == CONTENT {
                continue;
            }
            let mut seen = std::collections::BTreeSet::new();
            let mut supported = true;
            for c in &yes {
                let raw = &e.records[c.value.source];
                let sw = reader::words(g, raw, ordered::CANONICAL)?;
                let sr = source_context(&a, g, m, raw, false)?;
                let mut matches = 0;
                for (j, w) in sw.iter().enumerate() {
                    if j >= c.value.word && j < c.last_word {
                        continue;
                    }
                    if ordered::distance(m, &q[i].geometry, &w.geometry, false)? == 0 {
                        seen.insert(sr[j]);
                        matches += 1
                    }
                }
                supported &= matches > 0;
            }
            if !supported || seen.len() != 1 {
                f.skipped_ambiguous_occurrences += 1;
                continue;
            }
            let context = seen.contains(&true);
            counts.entry(k.clone()).or_default()[usize::from(context)] += 1;
        }
    }
    if counts.len() > MAX_ROWS {
        return Err(Error::Artifact);
    }
    a.query_table = induce(&counts);
    f.learned_content_rows = a.query_table.len();
    for (k, c) in counts {
        f.conflicts += usize::from(c[0] > 0 && c[1] > 0);
        f.credits.push(Credit {
            key: k,
            content: c[0],
            context: c[1],
        });
    }
    a.data_digest = *blake3::hash(
        &[
            a.data_digest.as_slice(),
            serde_json::to_vec(train)
                .map_err(|_| Error::Artifact)?
                .as_slice(),
        ]
        .concat(),
    )
    .as_bytes();
    a.training.push_str(" Second bounded coordinate fit learns query roles from unanimous matching source-role observations outside output-compatible spans. Frozen source table/anchors; query/source role agreement required per witness. No occurrence chosen from equal-byte alternatives.");
    a.validate(g)?;
    Ok((a, f))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoverageFit {
    pub rows: usize,
    pub before_correct: usize,
    pub enabled_correct: usize,
    pub prior_successes_lost: usize,
    pub selected: bool,
    pub data_digest: [u8; 32],
}
/// Two explicit discrete forwards choose the coverage observation only when it
/// improves actual final answers without losing any earlier correct training row.
/// Source/query role tables and all parent parameters remain fixed.
pub fn fit_coverage(
    mut a: Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[span_data::Example],
) -> Result<(Artifact, CoverageFit)> {
    a.validate(g)?;
    if train.is_empty() {
        return Err(Error::Shape);
    }
    a.schema = 3;
    a.require_available_query_coverage = false;
    let mut enabled = a.clone();
    enabled.require_available_query_coverage = true;
    let mut f = CoverageFit {
        rows: train.len(),
        before_correct: 0,
        enabled_correct: 0,
        prior_successes_lost: 0,
        selected: false,
        data_digest: *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?)
            .as_bytes(),
    };
    for e in train {
        let before = generate(&a, g, m, &e.records, &e.prompt, Control::Full)?;
        let after = generate(&enabled, g, m, &e.records, &e.prompt, Control::Full)?;
        let correct = |out: &completion::Generated| {
            out.outcome == completion::Outcome::Answered
                && !out.trace.exhausted
                && out.trace.tokens == span_learning::target(e)
        };
        let old = correct(&before);
        let new = correct(&after);
        f.before_correct += usize::from(old);
        f.enabled_correct += usize::from(new);
        f.prior_successes_lost += usize::from(old && !new);
    }
    f.selected = f.enabled_correct > f.before_correct && f.prior_successes_lost == 0;
    a.require_available_query_coverage = f.selected;
    a.data_digest =
        *blake3::hash(&[a.data_digest.as_slice(), f.data_digest.as_slice()].concat()).as_bytes();
    a.source_digest = *blake3::hash(
        concat!(
            include_str!("occurrence_role.rs"),
            include_str!("occurrence.rs"),
            include_str!("span_learning.rs")
        )
        .as_bytes(),
    )
    .as_bytes();
    a.training.push_str(" Third observation: one discrete false/true coverage fit on actual final outputs; all query occurrences with any source identity match must be assigned in the selected source. Unmatched question words remain explicit. Freeze source/query roles; choose only strict improvement with zero lost training successes. This is finite available-anchor coverage, not general grammatical necessity.");
    a.validate(g)?;
    Ok((a, f))
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    ExactIdentity,
    RoleDisabled,
    ReadDisabled,
    UpdateDisabled,
    QueryRolesDisabled,
    CoverageDisabled,
}
pub fn route(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<lexical::Route> {
    if c == Control::RoleDisabled || c == Control::ReadDisabled {
        return correspondence::route(
            &a.parent,
            g,
            m,
            records,
            question,
            if c == Control::ReadDisabled {
                correspondence::Control::ReadDisabled
            } else {
                correspondence::Control::Full
            },
        );
    }
    match reader::words(g, question, a.reader().parent.parent.operators) {
        Ok(q) if !q.is_empty() && q.len() <= reader::MAX_WORDS => {}
        Ok(_) | Err(Error::Shape) => {
            return Ok(lexical::Route {
                status: lexical::RouteStatus::UnsupportedWordWindow,
                selected: None,
                compatible: vec![],
            })
        }
        Err(e) => return Err(e),
    }
    let exact = c == Control::ExactIdentity;
    let contexts: Vec<_> = records
        .iter()
        .map(|r| source_context(a, g, m, r, exact))
        .collect::<Result<_>>()?;
    let contexts: [Vec<bool>; 4] = contexts.try_into().map_err(|_| Error::Shape)?;
    let query_roles = query_context(a, g, m, question, exact)?;
    let query_words = reader::words(g, question, a.reader().parent.parent.operators)?;
    let mut available = vec![false; query_words.len()];
    if a.require_available_query_coverage && c != Control::CoverageDisabled {
        for record in records {
            for word in reader::words(g, record, ordered::CANONICAL)? {
                for (i, q) in query_words.iter().enumerate() {
                    available[i] |= if exact {
                        q.geometry.occurrences == word.geometry.occurrences
                    } else {
                        ordered::distance(m, &q.geometry, &word.geometry, false)? == 0
                    };
                }
            }
        }
    }

    let search = occurrence::candidates_with_context(
        &a.parent.parent,
        g,
        m,
        records,
        question,
        if exact {
            occurrence::Control::ExactIdentity
        } else {
            occurrence::Control::Full
        },
        Some(&contexts),
    )?;
    let mut accepted = vec![];
    for candidate in search.candidates {
        for witness in candidate.witnesses {
            if available
                .iter()
                .zip(&witness.query_to_source)
                .any(|(&known, j)| known && j.is_none())
            {
                continue;
            }

            let roles = &contexts[candidate.span.value.source];
            let witness_roles: Vec<_> = witness
                .query_to_source
                .iter()
                .map(|j| j.map(|j| roles[j]).unwrap_or(false))
                .collect();
            if c != Control::QueryRolesDisabled
                && witness
                    .query_to_source
                    .iter()
                    .enumerate()
                    .any(|(i, j)| j.is_some() && query_roles[i] != witness_roles[i])
            {
                continue;
            }
            let sig =
                correspondence::run_signature(&witness.query_to_source, &witness_roles, roles)?;
            let features = witness.features
                | if sig.ordered_unit_adjacency {
                    correspondence::STRUCTURE_BIT
                } else {
                    0
                };
            if a.parent.matches(features) {
                let mut v = candidate.span.value.clone();
                v.features = features & ((1 << 18) - 1);
                accepted.push(v);
                break;
            }
        }
    }
    let compatible = accepted.iter().map(|v| [v.source, v.word]).collect();
    let status = match accepted.len() {
        0 => lexical::RouteStatus::NoCompatibleCandidate,
        1 => lexical::RouteStatus::Selected,
        _ => lexical::RouteStatus::Ambiguous,
    };
    Ok(lexical::Route {
        status,
        selected: if accepted.len() == 1 {
            accepted.pop()
        } else {
            None
        },
        compatible,
    })
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
    completion::generate_routed_payload(
        &a.parent.parent.parent,
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
        PayloadWindow::Phrase,
        |b, _| Ok(b.to_vec()),
        |q, _| route(a, g, m, records, q, c),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn anchor_identity_preserves_the_boolean_collision() {
        let name = [CONTENT, 0, 1, 2, CONTENT];
        let auxiliary = [CONTENT, 0, 2, CONTENT];
        assert_ne!(key(&name, 1), key(&auxiliary, 1));
        assert_eq!(key(&name, 0).left, EDGE);
        assert_eq!(key(&name, 4).right, EDGE);
    }
    #[test]
    fn learned_patterns_recombine_neighbors_without_overriding_conflicts() {
        let mut c = BTreeMap::new();
        c.insert(
            Key {
                center: 0,
                left: EDGE,
                right: 1,
            },
            [2, 0],
        );
        c.insert(
            Key {
                center: 0,
                left: CONTENT,
                right: 2,
            },
            [0, 3],
        );
        c.insert(
            Key {
                center: 0,
                left: CONTENT,
                right: 1,
            },
            [1, 1],
        );
        let rows = induce(&c);
        assert!(rows.iter().any(|r| matches_key(
            &r.key,
            &Key {
                center: 0,
                left: EDGE,
                right: 0
            }
        )));
        for (k, count) in &c {
            if count[1] > 0 {
                assert!(!rows.iter().any(|r| matches_key(&r.key, k)));
            }
        }
        assert!(!rows.iter().any(|r| matches_key(
            &r.key,
            &Key {
                center: 3,
                left: EDGE,
                right: 1
            }
        )));
    }
}
