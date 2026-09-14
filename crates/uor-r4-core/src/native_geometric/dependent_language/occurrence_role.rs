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
pub(crate) fn observations(
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
pub(crate) fn learn_anchors(
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

/// Reciprocal source views are joined before role labels are created. A source
/// occurrence outside one answer can still belong to the other endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PairedFit {
    pub fit: Fit,
    pub candidate_table: Vec<Row>,
    pub source_bundles: usize,
    pub qualified_bundles: usize,
    pub unsupported_bundles: usize,
    pub ambiguous_occurrences: usize,
    pub blocked_keys: usize,
    pub blocked_observations: Vec<Key>,
}

type PairedViews = BTreeMap<Vec<u8>, std::collections::BTreeSet<(usize, usize)>>;

/// `Some(false)` is content and `Some(true)` is context. An ambiguous view
/// withholds the occurrence's credit instead of choosing an equal-byte span.
fn paired_membership(views: &PairedViews, words: usize) -> Option<Vec<Option<bool>>> {
    let supported: Vec<_> = views
        .values()
        .filter_map(|spans| (spans.len() == 1).then(|| spans.iter().next()).flatten())
        .collect();
    let qualified = supported
        .iter()
        .enumerate()
        .any(|(i, a)| supported[i + 1..].iter().any(|b| a.1 <= b.0 || b.1 <= a.0));
    if !qualified
        || views.values().any(|spans| {
            spans.is_empty()
                || spans
                    .iter()
                    .any(|&(first, last)| first >= last || last > words)
        })
    {
        return None;
    }
    Some(
        (0..words)
            .map(|i| {
                let mut content = false;
                for spans in views.values() {
                    let inside = spans
                        .iter()
                        .filter(|&&(first, last)| first <= i && i < last)
                        .count();
                    if inside != 0 && inside != spans.len() {
                        return None;
                    }
                    content |= inside == spans.len();
                }
                Some(!content)
            })
            .collect(),
    )
}

fn induce_paired(
    counts: &BTreeMap<Key, [usize; 2]>,
    blocked: &std::collections::BTreeSet<Key>,
) -> Vec<Row> {
    let mut options = std::collections::BTreeSet::new();
    for (key, count) in counts {
        if count[0] == 0 {
            continue;
        }
        for left in [key.left, ANY] {
            for right in [key.right, ANY] {
                let rule = Key {
                    center: key.center,
                    left,
                    right,
                };
                if !counts
                    .iter()
                    .any(|(key, count)| count[1] > 0 && matches_key(&rule, key))
                    && !blocked.iter().any(|key| matches_key(&rule, key))
                {
                    options.insert(rule);
                }
            }
        }
    }
    options
        .iter()
        .filter(|rule| {
            !options
                .iter()
                .any(|other| other != *rule && matches_key(other, rule))
        })
        .map(|key| Row {
            key: key.clone(),
            context: false,
        })
        .collect()
}

fn constrain_unsupported(
    counts: &BTreeMap<Key, [usize; 2]>,
    unsupported: &std::collections::BTreeSet<Key>,
    blocked: &mut std::collections::BTreeSet<Key>,
) {
    blocked.extend(
        unsupported
            .iter()
            .filter(|key| {
                !counts
                    .get(*key)
                    .is_some_and(|count| (count[0] > 0) != (count[1] > 0))
            })
            .cloned(),
    );
}

/// Fit the existing role representation from paired raw questions and final
/// answers. Exact source bytes and occurrence indices join the views; rotations
/// and duplicate questions cannot manufacture reciprocal support. Two distinct
/// questions with unambiguous disjoint generated output spans are required
/// before any complementary context credit. No intermediate oracle labels enter.
pub fn fit_paired(
    parent: correspondence::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[span_data::Example],
) -> Result<(Artifact, PairedFit)> {
    parent.validate(g)?;
    if train.is_empty() || train.len() > 8192 {
        return Err(Error::Shape);
    }
    let anchors = learn_anchors(&parent, g, train)?;
    let prepared = span_learning::prepare(&parent.parent, g, m, train)?;
    let mut bundles: BTreeMap<Vec<u8>, PairedViews> = BTreeMap::new();
    let mut fit = Fit {
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
            .filter_map(|(candidate, &compatible)| compatible.then_some(candidate))
            .collect();
        fit.compatible_spans += yes.len();
        let first = yes.first().ok_or(Error::Shape)?;
        if yes
            .iter()
            .any(|candidate| candidate.value.source != first.value.source)
        {
            fit.skipped_multi_source_rows += 1;
            continue;
        }
        let views = bundles
            .entry(e.records[first.value.source].clone())
            .or_default();
        let spans = views.entry(e.prompt.clone()).or_default();
        spans.extend(yes.iter().map(|c| (c.value.word, c.last_word)));
    }
    let mut counts: BTreeMap<Key, [usize; 2]> = BTreeMap::new();
    let mut blocked = std::collections::BTreeSet::new();
    let mut unsupported = std::collections::BTreeSet::new();
    let mut report = PairedFit {
        fit,
        candidate_table: vec![],
        source_bundles: bundles.len(),
        qualified_bundles: 0,
        unsupported_bundles: 0,
        ambiguous_occurrences: 0,
        blocked_keys: 0,
        blocked_observations: vec![],
    };
    for (raw, views) in bundles {
        let keys = observations(&parent, &anchors, g, m, &raw, false)?;
        let Some(labels) = paired_membership(&views, keys.len()) else {
            report.unsupported_bundles += 1;
            // Incomplete views are not context labels. Their uncredited keys
            // constrain extrapolation after independent complete-bundle credit
            // has been collected below.
            unsupported.extend(keys.into_iter().filter(|key| key.center != CONTENT));
            continue;
        };
        report.qualified_bundles += 1;
        for (key, label) in keys.into_iter().zip(labels) {
            if key.center == CONTENT {
                continue;
            }
            match label {
                Some(context) => counts.entry(key).or_default()[usize::from(context)] += 1,
                None => {
                    report.ambiguous_occurrences += 1;
                    blocked.insert(key);
                }
            }
        }
    }
    if report.qualified_bundles == 0 {
        return Err(Error::Shape);
    }
    report.candidate_table = induce_paired(&counts, &blocked);
    constrain_unsupported(&counts, &unsupported, &mut blocked);
    if counts.len() > MAX_ROWS
        || blocked.len() > MAX_ROWS
        || report.candidate_table.len() > MAX_ROWS
    {
        return Err(Error::Artifact);
    }
    let table = induce_paired(&counts, &blocked);
    report.blocked_keys = blocked.len();
    report.blocked_observations = blocked.iter().cloned().collect();
    if table.len() > MAX_ROWS {
        return Err(Error::Artifact);
    }
    report.fit.skipped_ambiguous_occurrences = report.ambiguous_occurrences;
    report.fit.learned_content_rows = table.len();
    for (key, count) in counts {
        report.fit.conflicts += usize::from(count[0] > 0 && count[1] > 0);
        report.fit.credits.push(Credit {
            key,
            content: count[0],
            context: count[1],
        });
    }
    let artifact = Artifact {
        schema: 2,
        parent_digest: *blake3::hash(&parent.encode()?).as_bytes(),
        parent,
        table,
        query_table: vec![],
        require_available_query_coverage: false,
        anchors,
        data_digest: *blake3::hash(
            &serde_json::to_vec(train).map_err(|_| Error::Artifact)?,
        )
        .as_bytes(),
        source_digest: *blake3::hash(
            concat!(
                include_str!("occurrence_role.rs"),
                include_str!("occurrence.rs"),
                include_str!("span_learning.rs")
            )
            .as_bytes(),
        )
        .as_bytes(),
        training: "Bounded reciprocal-view occurrence-role fit. Raw source bytes and occurrence indices join distinct raw questions before labels. Actual final-answer/EOS-compatible generated spans remain latent; two distinct questions with unique disjoint spans qualify a source bundle. Unanimous answer membership in any view supplies content credit, outside all views supplies complementary context credit, and ambiguous membership withholds credit. Incomplete-bundle keys constrain wildcard extrapolation unless independently consistent exact-key credit exists; ambiguous memberships remain strict blockers. Exact local key conflicts retain the prior; no positive-overrides-negative rule. Neighbor-pattern induction and geometric observations are unchanged. Parent parameters frozen; authored reciprocal-view transfer is not general grammar.".into(),
    };
    artifact.validate(g)?;
    Ok((artifact, report))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceRuleTrial {
    pub pass: usize,
    pub rule: Row,
    pub correct: usize,
    pub gained: usize,
    pub lost: usize,
    pub selected: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceSelectionFit {
    pub rows: usize,
    pub initial_table: Vec<Row>,
    pub before_correct: usize,
    pub after_correct: usize,
    pub proposals: usize,
    pub actual_forwards: usize,
    pub trials: Vec<SourceRuleTrial>,
    pub removed_rules: Vec<Row>,
    pub final_table: Vec<Row>,
    pub stopping: String,
}

fn source_selection_score(before: &[bool], after: &[bool]) -> Result<(usize, usize, usize)> {
    if before.len() != after.len() {
        return Err(Error::Shape);
    }
    let correct = after.iter().filter(|&&yes| yes).count();
    let gained = before
        .iter()
        .zip(after)
        .filter(|&(&old, &new)| !old && new)
        .count();
    let lost = before
        .iter()
        .zip(after)
        .filter(|&(&old, &new)| old && !new)
        .count();
    Ok((correct, gained, lost))
}

/// Select among already induced source rules using actual final-answer/EOS
/// generation. The caller supplies the retained serving composition; all model
/// and query-role parameters other than the proposed source rule are frozen.
pub fn select_source_rules<F>(
    mut a: Artifact,
    train: &[span_data::Example],
    mut forward: F,
) -> Result<(Artifact, SourceSelectionFit)>
where
    F: FnMut(&Artifact, &span_data::Example) -> Result<completion::Generated>,
{
    const MAX_SELECTION_RULES: usize = 32;
    const MAX_SELECTION_ROWS: usize = 8192;
    if train.is_empty()
        || train.len() > MAX_SELECTION_ROWS
        || a.table.len() > MAX_SELECTION_RULES
        || a.table.iter().any(|row| row.context)
        || a.table.windows(2).any(|rows| rows[0].key >= rows[1].key)
    {
        return Err(Error::Shape);
    }
    let generated_correct = |generated: &completion::Generated, example: &span_data::Example| {
        generated.trace.tokens == span_learning::target(example)
            && !generated.trace.exhausted
            && generated.outcome == completion::Outcome::Answered
    };
    let mut before = Vec::with_capacity(train.len());
    for example in train {
        before.push(generated_correct(&forward(&a, example)?, example));
    }
    let before_correct = before.iter().filter(|&&yes| yes).count();
    let mut report = SourceSelectionFit {
        rows: train.len(),
        initial_table: a.table.clone(),
        before_correct,
        after_correct: before_correct,
        proposals: 0,
        actual_forwards: train.len(),
        trials: vec![],
        removed_rules: vec![],
        final_table: vec![],
        stopping: "no_safe_improving_removal".into(),
    };
    for pass in 0..MAX_SELECTION_RULES {
        if report.after_correct == train.len() {
            report.stopping = "all_training_outputs_correct".into();
            break;
        }
        if a.table.is_empty() {
            report.stopping = "no_source_rules_remaining".into();
            break;
        }
        let mut best: Option<(usize, usize, Vec<bool>)> = None;
        let mut best_correct = report.after_correct;
        for index in 0..a.table.len() {
            let mut proposal = a.clone();
            let rule = proposal.table.remove(index);
            let mut after = Vec::with_capacity(train.len());
            for example in train {
                after.push(generated_correct(&forward(&proposal, example)?, example));
                report.actual_forwards += 1;
            }
            let (correct, gained, lost) = source_selection_score(&before, &after)?;
            report.proposals += 1;
            report.trials.push(SourceRuleTrial {
                pass,
                rule,
                correct,
                gained,
                lost,
                selected: false,
            });
            if gained > 0 && lost == 0 && correct > best_correct {
                best_correct = correct;
                best = Some((index, report.trials.len() - 1, after));
            }
        }
        let Some((index, trial_index, after)) = best else {
            break;
        };
        report.trials[trial_index].selected = true;
        report.removed_rules.push(a.table.remove(index));
        report.after_correct = best_correct;
        before = after;
        if pass + 1 == MAX_SELECTION_RULES {
            report.stopping = "selection_pass_limit".into();
        }
    }
    report.final_table = a.table.clone();
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
    a.training.push_str(" Bounded actual-output source-rule selection starts with at most 32 induced candidate rules and at most 8192 direct examples. Each proposal removes one content rule while retaining every other parameter. Final byte/EOS generation must strictly improve and lose zero previously correct training outputs; largest safe gain wins with deterministic table-order ties. Baseline success flags are cached across at most 32 greedy passes. No word exception or intermediate span label selects a rule.");
    Ok((a, report))
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
pub struct QueryWitnessFit {
    pub initial: Fit,
    pub initial_table: Vec<Row>,
    pub refinement: Fit,
    pub refinement_rows: usize,
    pub surviving_witnesses: usize,
    pub unsupported_rows: usize,
    pub blocked_keys: usize,
    pub blocked_observations: Vec<Key>,
    pub frozen_content_patterns: Vec<Row>,
    pub frozen_context_patterns: Vec<Row>,
}

fn supported_pattern_role(content: &[Row], context: &[Row], key: &Key) -> Option<bool> {
    match (
        content.iter().any(|row| matches_key(&row.key, key)),
        context.iter().any(|row| matches_key(&row.key, key)),
    ) {
        (true, false) => Some(false),
        (false, true) => Some(true),
        _ => None,
    }
}

/// Refine ambiguous query roles using independently supported local patterns
/// and the existing injective witnesses. First-pass patterns are frozen before
/// witness selection; no selected witness can train its own prerequisite.
pub fn fit_queries_witnessed(
    a: Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[span_data::Example],
) -> Result<(Artifact, QueryWitnessFit)> {
    if train.is_empty() || train.len() > 8192 {
        return Err(Error::Shape);
    }
    let (mut a, initial) = fit_queries(a, g, m, train)?;
    let first_counts: BTreeMap<_, _> = initial
        .credits
        .iter()
        .map(|credit| (credit.key.clone(), [credit.content, credit.context]))
        .collect();
    let content_patterns = induce(&first_counts);
    let context_counts = first_counts
        .iter()
        .map(|(key, count)| (key.clone(), [count[1], count[0]]))
        .collect();
    let mut context_patterns = induce(&context_counts);
    for row in &mut context_patterns {
        row.context = true;
    }
    let unresolved = |key: &Key| {
        key.center != CONTENT
            && !first_counts
                .get(key)
                .is_some_and(|count| (count[0] > 0) != (count[1] > 0))
    };
    let mut selected = Vec::new();
    for example in train {
        let keys = observations(&a.parent, &a.anchors, g, m, &example.prompt, false)?;
        if keys.iter().any(&unresolved) {
            selected.push(example.clone());
        }
    }
    let prepared = span_learning::prepare(&a.parent.parent, g, m, &selected)?;
    let mut counts = first_counts.clone();
    let mut new_counts: BTreeMap<Key, [usize; 2]> = BTreeMap::new();
    let mut blocked = std::collections::BTreeSet::new();
    let mut report = QueryWitnessFit {
        initial,
        initial_table: a.query_table.clone(),
        refinement: Fit {
            rows: selected.len(),
            compatible_spans: 0,
            skipped_ambiguous_occurrences: 0,
            skipped_multi_source_rows: 0,
            conflicts: 0,
            learned_content_rows: 0,
            credits: vec![],
        },
        refinement_rows: selected.len(),
        surviving_witnesses: 0,
        unsupported_rows: 0,
        blocked_keys: 0,
        blocked_observations: vec![],
        frozen_content_patterns: content_patterns.clone(),
        frozen_context_patterns: context_patterns.clone(),
    };
    for (example, prepared) in selected.iter().zip(prepared) {
        let compatible: std::collections::BTreeSet<_> = prepared
            .candidates
            .iter()
            .zip(&prepared.compatible)
            .filter_map(|(candidate, &yes)| {
                yes.then_some((
                    candidate.value.source,
                    candidate.value.word,
                    candidate.last_word,
                ))
            })
            .collect();
        report.refinement.compatible_spans += compatible.len();
        if compatible.is_empty() {
            return Err(Error::Shape);
        }
        let keys = observations(&a.parent, &a.anchors, g, m, &example.prompt, false)?;
        let query_words = reader::words(g, &example.prompt, ordered::CANONICAL)?;
        let contexts: [Vec<bool>; 4] = example
            .records
            .iter()
            .map(|raw| source_context(&a, g, m, raw, false))
            .collect::<Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| Error::Shape)?;
        let mut available = vec![false; query_words.len()];
        if a.require_available_query_coverage {
            for record in &example.records {
                for word in reader::words(g, record, ordered::CANONICAL)? {
                    for (i, query) in query_words.iter().enumerate() {
                        available[i] |=
                            ordered::distance(m, &query.geometry, &word.geometry, false)? == 0;
                    }
                }
            }
        }
        let search = occurrence::candidates_with_context(
            &a.parent.parent,
            g,
            m,
            &example.records,
            &example.prompt,
            occurrence::Control::Full,
            Some(&contexts),
        )?;
        let mut seen = vec![std::collections::BTreeSet::new(); keys.len()];
        let mut supported = vec![true; keys.len()];
        let mut surviving = 0;
        for candidate in search.candidates {
            if !compatible.contains(&(
                candidate.span.value.source,
                candidate.span.value.word,
                candidate.span.last_word,
            )) {
                continue;
            }
            let source_roles = &contexts[candidate.span.value.source];
            for witness in candidate.witnesses {
                if available
                    .iter()
                    .zip(&witness.query_to_source)
                    .any(|(&required, source)| required && source.is_none())
                {
                    continue;
                }
                let roles: Vec<_> = witness
                    .query_to_source
                    .iter()
                    .map(|source| source.map(|j| source_roles[j]).unwrap_or(false))
                    .collect();
                if witness
                    .query_to_source
                    .iter()
                    .enumerate()
                    .any(|(i, source)| {
                        source.is_some()
                            && supported_pattern_role(
                                &content_patterns,
                                &context_patterns,
                                &keys[i],
                            )
                            .is_some_and(|expected| expected != roles[i])
                    })
                {
                    continue;
                }
                let signature =
                    correspondence::run_signature(&witness.query_to_source, &roles, source_roles)?;
                let features = witness.features
                    | if signature.ordered_unit_adjacency {
                        correspondence::STRUCTURE_BIT
                    } else {
                        0
                    };
                if !a.parent.matches(features) {
                    continue;
                }
                surviving += 1;
                for (i, source) in witness.query_to_source.iter().enumerate() {
                    if source.is_some() {
                        seen[i].insert(roles[i]);
                    } else {
                        supported[i] = false;
                    }
                }
            }
        }
        report.surviving_witnesses += surviving;
        report.unsupported_rows += usize::from(surviving == 0);
        for (i, key) in keys.into_iter().enumerate() {
            if !unresolved(&key) {
                continue;
            }
            if surviving == 0 || !supported[i] || seen[i].len() != 1 {
                report.refinement.skipped_ambiguous_occurrences += 1;
                blocked.insert(key);
                continue;
            }
            let context = seen[i].contains(&true);
            counts.entry(key.clone()).or_default()[usize::from(context)] += 1;
            new_counts.entry(key).or_default()[usize::from(context)] += 1;
        }
    }
    if counts.len() > MAX_ROWS || blocked.len() > MAX_ROWS {
        return Err(Error::Artifact);
    }
    a.query_table = induce_paired(&counts, &blocked);
    report.blocked_keys = blocked.len();
    report.blocked_observations = blocked.iter().cloned().collect();
    report.refinement.learned_content_rows = a.query_table.len();
    report.refinement.conflicts = counts
        .values()
        .filter(|count| count[0] > 0 && count[1] > 0)
        .count();
    report.refinement.credits = new_counts
        .into_iter()
        .map(|(key, count)| Credit {
            key,
            content: count[0],
            context: count[1],
        })
        .collect();
    a.training.push_str(" One frozen cross-example query refinement induces both context and content neighbor patterns from first-pass unambiguous credits. Only unsupported exact keys are reconsidered. Existing injective witnesses must match those frozen patterns, retained correspondence predicates and actual byte/EOS-compatible spans; coverage is retained when enabled. New credit is unanimous across surviving witnesses. Unknown-neighbor markers remain unknown, never grammatical labels. Ambiguous unsupported keys block wildcard extrapolation. Source table and parent parameters remain frozen.");
    a.validate(g)?;
    Ok((a, report))
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
    route_with_required(a, g, m, records, question, c, None)
}
/// A separate learned query-obligation mask can suppress optional evidence.
/// It retains occurrence roles, raw query indices and source/span identities.
pub(crate) fn route_with_required(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
    required: Option<&[bool]>,
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
    if required.is_some_and(|mask| mask.len() != query_words.len()) {
        return Err(Error::Shape);
    }
    let mut available = vec![false; query_words.len()];
    if a.require_available_query_coverage && c != Control::CoverageDisabled {
        for record in records {
            for word in reader::words(g, record, ordered::CANONICAL)? {
                for (i, q) in query_words.iter().enumerate() {
                    if required.is_some_and(|mask| !mask[i]) {
                        continue;
                    }
                    available[i] |= if exact {
                        q.geometry.occurrences == word.geometry.occurrences
                    } else {
                        ordered::distance(m, &q.geometry, &word.geometry, false)? == 0
                    };
                }
            }
        }
    }

    let search = occurrence::candidates_with_context_required(
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
        required,
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
    fn source_rule_selection_counts_regressions_separately_from_net_gain() -> Result<()> {
        let before = [true, false, false];
        assert_eq!(
            source_selection_score(&before, &[true, true, false])?,
            (2, 1, 0)
        );
        assert_eq!(
            source_selection_score(&before, &[false, true, true])?,
            (2, 2, 1)
        );
        assert_eq!(source_selection_score(&before, &before)?, (1, 0, 0));
        assert!(source_selection_score(&before, &[true]).is_err());
        Ok(())
    }

    #[test]
    fn unsupported_source_views_block_extrapolation_without_erasing_exact_credit() {
        let name = Key {
            center: 0,
            left: 1,
            right: 2,
        };
        let auxiliary_without_credit = Key {
            center: 0,
            left: 1,
            right: 3,
        };
        let counts = BTreeMap::from([(name.clone(), [1, 0])]);
        let unsupported = [name.clone(), auxiliary_without_credit.clone()].into();
        let mut blocked = std::collections::BTreeSet::new();
        constrain_unsupported(&counts, &unsupported, &mut blocked);
        assert_eq!(blocked, [auxiliary_without_credit.clone()].into());
        let rows = induce_paired(&counts, &blocked);
        assert!(rows.iter().any(|row| matches_key(&row.key, &name)));
        assert!(!rows
            .iter()
            .any(|row| matches_key(&row.key, &auxiliary_without_credit)));
        blocked.insert(name.clone());
        constrain_unsupported(&counts, &unsupported, &mut blocked);
        assert!(blocked.contains(&name));
    }

    #[test]
    fn frozen_query_patterns_preserve_unknown_and_conflicting_support() {
        let content = vec![Row {
            key: Key {
                center: 0,
                left: 1,
                right: ANY,
            },
            context: false,
        }];
        let context = vec![Row {
            key: Key {
                center: 0,
                left: ANY,
                right: 2,
            },
            context: false,
        }];
        assert_eq!(
            supported_pattern_role(
                &content,
                &context,
                &Key {
                    center: 0,
                    left: 1,
                    right: 3,
                }
            ),
            Some(false)
        );
        assert_eq!(
            supported_pattern_role(
                &content,
                &context,
                &Key {
                    center: 0,
                    left: 3,
                    right: 2,
                }
            ),
            Some(true)
        );
        for key in [
            Key {
                center: 0,
                left: 1,
                right: 2,
            },
            Key {
                center: 0,
                left: CONTENT,
                right: CONTENT,
            },
        ] {
            assert_eq!(supported_pattern_role(&content, &context, &key), None);
        }
    }

    #[test]
    fn paired_roles_join_endpoints_before_complementary_credit() {
        let views = BTreeMap::from([
            (b"object question".to_vec(), [(4, 6)].into()),
            (b"subject question".to_vec(), [(1, 2)].into()),
        ]);
        assert_eq!(
            paired_membership(&views, 7),
            Some(vec![
                Some(true),
                Some(false),
                Some(true),
                Some(true),
                Some(false),
                Some(false),
                Some(true),
            ])
        );
        let duplicate_or_single_view =
            BTreeMap::from([(b"same question".to_vec(), [(1, 2), (4, 6)].into())]);
        assert_eq!(paired_membership(&duplicate_or_single_view, 7), None);
        let overlapping = BTreeMap::from([
            (b"question one".to_vec(), [(1, 3)].into()),
            (b"question two".to_vec(), [(2, 4)].into()),
        ]);
        assert_eq!(paired_membership(&overlapping, 7), None);
    }

    #[test]
    fn paired_roles_keep_equal_byte_ambiguity_uncredited() {
        let views = BTreeMap::from([
            (b"object question".to_vec(), [(4, 5)].into()),
            (b"subject question".to_vec(), [(1, 2)].into()),
            (b"ambiguous question".to_vec(), [(1, 2), (3, 4)].into()),
        ]);
        assert_eq!(
            paired_membership(&views, 6),
            Some(vec![
                Some(true),
                None,
                Some(true),
                None,
                Some(false),
                Some(true),
            ])
        );
    }

    #[test]
    fn paired_pattern_ambiguity_blocks_wildcards_without_becoming_a_label() {
        let positive = Key {
            center: 0,
            left: 1,
            right: 2,
        };
        let ambiguous = Key {
            center: 0,
            left: 1,
            right: 3,
        };
        let conflict = Key {
            center: 0,
            left: 4,
            right: 2,
        };
        let counts = BTreeMap::from([(positive.clone(), [1, 0]), (conflict.clone(), [1, 1])]);
        let rows = induce_paired(&counts, &[ambiguous.clone()].into());
        assert!(rows.iter().any(|r| matches_key(&r.key, &positive)));
        assert!(!rows.iter().any(|r| matches_key(&r.key, &ambiguous)));
        assert!(!rows.iter().any(|r| matches_key(&r.key, &conflict)));
        assert_eq!(counts.len(), 2);
    }

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
