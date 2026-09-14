//! Ordered content-run structure retained on each bounded occurrence witness.
//! Learned context-only words separate runs; no names or grammar list is added.
use super::{completion, occurrence, runtime::PayloadWindow, scheduling, span, span_boundary};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime as lexical,
    relational_attention::runtime::{Error, Result},
    relative_language::runtime as reader,
};
use serde::{Deserialize, Serialize};
pub const FEATURES: usize = 20;
pub const STRUCTURE_BIT: u32 = 1 << 19;
pub const MAX_RULES: usize = 8;
pub const MAX_SEARCH_NODES: usize = occurrence::MAX_SEARCH_NODES;
pub const MAX_ARTIFACT_BYTES: usize = 8 * 1024 * 1024;
pub fn feature_names() -> Vec<String> {
    span::feature_names()
        .into_iter()
        .chain([
            "matched_content_runs_preserve_ordered_unit_adjacency_after_source_context_projection"
                .into(),
        ])
        .collect()
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: span::Artifact,
    pub parent_digest: [u8; 32],
    pub rules: Vec<u32>,
    pub data_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub training: String,
    pub feature_names: Vec<String>,
}
fn validate_header(schema: u16, rules: &[u32], names: &[String]) -> Result<()> {
    if schema != 2
        || rules.len() > MAX_RULES
        || rules
            .iter()
            .any(|r| *r == 0 || *r >> FEATURES != 0 || r.count_ones() > 5)
        || names != feature_names()
    {
        return Err(Error::Artifact);
    }
    Ok(())
}
impl Artifact {
    pub fn from_parent(parent: span::Artifact) -> Result<Self> {
        Ok(Self{schema:2,parent_digest:*blake3::hash(&parent.encode()?).as_bytes(),rules:parent.rules.clone(),data_digest:parent.data_digest,parent,source_digest:*blake3::hash(include_str!("correspondence.rs").as_bytes()).as_bytes(),training:"Inherited occurrence predicates with ordered content-run witness observation after projecting out intervening inherited context-only source positions. Raw occurrence indices remain retained. No fit yet; parent rules/context/completion/updater/writer remain unchanged.".into(),feature_names:feature_names()})
    }
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        self.parent.validate(g)?;
        validate_header(self.schema, &self.rules, &self.feature_names)?;
        if self.parent_digest != *blake3::hash(&self.parent.encode()?).as_bytes() {
            return Err(Error::Artifact);
        }
        Ok(())
    }
    pub fn matches(&self, features: u32) -> bool {
        self.rules.iter().any(|r| features & r == *r)
    }
    pub fn reader(&self) -> &reader::Artifact {
        self.parent.reader()
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| Error::Artifact)
    }
    pub fn decode(bytes: &[u8], g: &BoundGeometry) -> Result<Self> {
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(bytes).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    UnionMatches,
    NonInjective,
    ReadDisabled,
    ExactIdentity,
    UpdateDisabled,
    StructureDisabled,
    NonInjectiveStructureDisabled,
    ContextProjectionDisabled,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunPosition {
    Unmatched,
    LearnedContext,
    Start,
    Continuation,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunStep {
    pub query_index: usize,
    pub source_index: Option<usize>,
    pub learned_context: bool,
    pub position: RunPosition,
    pub run: Option<usize>,
    pub source_gap: Option<i16>,
    pub projected_source_gap: Option<i16>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunSignature {
    pub steps: Vec<RunStep>,
    pub runs: usize,
    pub ordered_unit_adjacency: bool,
    pub raw_ordered_unit_adjacency: bool,
}
/// Within each consecutive query run of matched non-context words, every next
/// source occurrence must follow by one after removing intervening inherited
/// context-only source positions. Positive source order is always required;
/// repeated/reversed occurrences cannot become adjacent by projection. Context
/// or unmatched query occurrences break the run. Raw indices/gaps are retained.
pub fn run_signature(
    query_to_source: &[Option<usize>],
    query_context: &[bool],
    source_context: &[bool],
) -> Result<RunSignature> {
    let source_words = source_context.len();
    if query_to_source.len() != query_context.len()
        || query_to_source.len() > reader::MAX_WORDS
        || source_words > reader::MAX_WORDS
        || query_to_source.iter().flatten().any(|j| *j >= source_words)
    {
        return Err(Error::Shape);
    }
    let mut steps = Vec::with_capacity(query_to_source.len());
    let mut previous = None;
    let mut runs = 0;
    let mut good = true;
    let mut raw_good = true;
    for (query_index, (&source_index, &context)) in
        query_to_source.iter().zip(query_context).enumerate()
    {
        let (position, run, gap, projected_gap) = match source_index {
            None => {
                previous = None;
                (RunPosition::Unmatched, None, None, None)
            }
            Some(_) if context => {
                previous = None;
                (RunPosition::LearnedContext, None, None, None)
            }
            Some(source) => {
                if let Some(before) = previous {
                    let gap = source as i16 - before as i16;
                    let projected_gap = if gap > 0 {
                        gap - source_context[before + 1..source]
                            .iter()
                            .filter(|context| **context)
                            .count() as i16
                    } else {
                        gap
                    };
                    raw_good &= gap == 1;
                    good &= gap > 0 && projected_gap == 1;
                    previous = Some(source);
                    (
                        RunPosition::Continuation,
                        Some(runs - 1),
                        Some(gap),
                        Some(projected_gap),
                    )
                } else {
                    runs += 1;
                    previous = Some(source);
                    (RunPosition::Start, Some(runs - 1), None, None)
                }
            }
        };
        steps.push(RunStep {
            query_index,
            source_index,
            learned_context: context,
            position,
            run,
            source_gap: gap,
            projected_source_gap: projected_gap,
        });
    }
    Ok(RunSignature {
        steps,
        runs,
        ordered_unit_adjacency: good,
        raw_ordered_unit_adjacency: raw_good,
    })
}
pub fn query_context(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    question: &[u8],
    exact: bool,
) -> Result<Vec<bool>> {
    reader::words(g, question, a.reader().parent.parent.operators)?
        .iter()
        .map(|w| span_boundary::contains(m, &a.parent.context_words, &w.geometry, exact))
        .collect()
}
pub fn source_context(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    record: &[u8],
    exact: bool,
) -> Result<Vec<bool>> {
    reader::words(
        g,
        record,
        crate::native_geometric::ordered_state::runtime::CANONICAL,
    )?
    .iter()
    .map(|w| span_boundary::contains(m, &a.parent.context_words, &w.geometry, exact))
    .collect()
}
/// Returns occurrence search unchanged except bit19 on every witness. Legacy
/// UnionMatches has no individual assignment: its feature mask receives bit19
/// as satisfied and the same wrapper rules decide admission. That control removes
/// both assignment separation and run structure; StructureDisabled removes only
/// the new run constraint and retains injective occurrence witnesses.
pub fn candidates(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<occurrence::Search> {
    if c == Control::UnionMatches {
        let candidates = span::candidates(&a.parent, g, m, records, question, span::Control::Full)?
            .into_iter()
            .map(|candidate| {
                let features = candidate.features | STRUCTURE_BIT;
                occurrence::Candidate {
                    span: candidate,
                    witnesses: vec![occurrence::Witness {
                        query_to_source: vec![],
                        features,
                        barriers: vec![],
                    }],
                }
            })
            .collect();
        return Ok(occurrence::Search {
            candidates,
            nodes: 0,
        });
    }
    let control = match c {
        Control::ExactIdentity => occurrence::Control::ExactIdentity,
        Control::NonInjective | Control::NonInjectiveStructureDisabled => {
            occurrence::Control::NonInjective
        }
        _ => occurrence::Control::Full,
    };
    let mut search = occurrence::candidates(&a.parent, g, m, records, question, control)?;
    let context = query_context(a, g, m, question, c == Control::ExactIdentity)?;
    let contexts: Vec<_> = records
        .iter()
        .map(|r| source_context(a, g, m, r, c == Control::ExactIdentity))
        .collect::<Result<_>>()?;
    for candidate in &mut search.candidates {
        for witness in &mut candidate.witnesses {
            let signature = run_signature(
                &witness.query_to_source,
                &context,
                &contexts[candidate.span.value.source],
            )?;
            if matches!(
                c,
                Control::StructureDisabled | Control::NonInjectiveStructureDisabled
            ) || if c == Control::ContextProjectionDisabled {
                signature.raw_ordered_unit_adjacency
            } else {
                signature.ordered_unit_adjacency
            } {
                witness.features |= STRUCTURE_BIT;
            }
        }
    }
    Ok(search)
}
fn select(a: &Artifact, search: occurrence::Search) -> lexical::Route {
    let mut accepted = Vec::new();
    for candidate in search.candidates {
        if let Some(witness) = candidate.witnesses.iter().find(|w| a.matches(w.features)) {
            let mut value = candidate.span.value;
            value.features = witness.features & ((1 << 18) - 1);
            accepted.push(value);
        }
    }
    let compatible = accepted.iter().map(|c| [c.source, c.word]).collect();
    let status = match accepted.len() {
        0 => lexical::RouteStatus::NoCompatibleCandidate,
        1 => lexical::RouteStatus::Selected,
        _ => lexical::RouteStatus::Ambiguous,
    };
    lexical::Route {
        status,
        selected: if accepted.len() == 1 {
            accepted.pop()
        } else {
            None
        },
        compatible,
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
    if c == Control::ReadDisabled {
        return span::route(
            &a.parent,
            g,
            m,
            records,
            question,
            span::Control::ReadDisabled,
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
    Ok(select(a, candidates(a, g, m, records, question, c)?))
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
        &a.parent.parent,
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
        |bytes, _| Ok(bytes.to_vec()),
        |q, _| route(a, g, m, records, q, c),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_adjacency_retains_order_and_repeated_occurrence_gaps() -> Result<()> {
        let context = [false, true, false, false, true];
        let contiguous = run_signature(
            &[None, Some(2), Some(0), Some(1), Some(3)],
            &context,
            &[false; 6],
        )?;
        assert!(contiguous.ordered_unit_adjacency);
        assert_eq!(contiguous.runs, 1);
        assert_eq!(contiguous.steps[3].source_gap, Some(1));
        let split = run_signature(
            &[None, Some(2), Some(0), Some(4), Some(3)],
            &context,
            &[false; 6],
        )?;
        assert!(!split.ordered_unit_adjacency);
        assert_eq!(split.steps[3].source_gap, Some(4));
        assert!(
            !run_signature(
                &[None, Some(2), Some(1), Some(0), Some(3)],
                &context,
                &[false; 6]
            )?
            .ordered_unit_adjacency
        );
        assert!(
            !run_signature(&[Some(0), Some(0)], &[false, false], &[false; 1])?
                .ordered_unit_adjacency
        );
        Ok(())
    }
    #[test]
    fn learned_context_and_unmatched_positions_break_runs_without_global_monotonicity() -> Result<()>
    {
        // The retained styled positive has auxiliary before subject in query,
        // but the learned context boundaries leave each content run coherent.
        let styled = run_signature(
            &[None, None, Some(2), Some(1), Some(3), Some(4)],
            &[false, false, true, false, true, true],
            &[false; 6],
        )?;
        assert!(styled.ordered_unit_adjacency);
        assert!(
            run_signature(
                &[Some(4), Some(2), Some(0)],
                &[false, true, false],
                &[false; 5]
            )?
            .ordered_unit_adjacency
        );
        assert!(
            run_signature(
                &[Some(4), None, Some(0)],
                &[false, false, false],
                &[false; 5]
            )?
            .ordered_unit_adjacency
        );
        assert!(run_signature(&[], &[], &[])?.ordered_unit_adjacency);
        assert!(run_signature(&[Some(4)], &[false], &[false; 5])?.ordered_unit_adjacency);
        Ok(())
    }
    #[test]
    fn source_projection_skips_only_inherited_context_and_retains_raw_order() -> Result<()> {
        // who did bruno trust? / bruno did trust helen.  Trust remains a
        // content word here; only the already learned did position is removed.
        let legitimate = run_signature(
            &[None, Some(1), Some(0), Some(2)],
            &[false, true, false, false],
            &[false, true, false, false],
        )?;
        assert!(legitimate.ordered_unit_adjacency);
        assert!(!legitimate.raw_ordered_unit_adjacency);
        assert_eq!(legitimate.steps[3].source_gap, Some(2));
        assert_eq!(legitimate.steps[3].projected_source_gap, Some(1));
        // amber oscar did help amber amber: projecting did/help must retain
        // intervening content oscar, so a split known phrase stays invalid.
        let split = run_signature(
            &[None, Some(2), Some(3), Some(0), Some(4)],
            &[false, true, true, false, false],
            &[false, false, true, true, false, false],
        )?;
        assert!(!split.ordered_unit_adjacency);
        assert_eq!(split.steps[4].source_gap, Some(4));
        assert_eq!(split.steps[4].projected_source_gap, Some(2));
        let reverse = run_signature(&[Some(2), Some(0)], &[false, false], &[false, true, false])?;
        assert!(!reverse.ordered_unit_adjacency);
        assert_eq!(reverse.steps[1].projected_source_gap, Some(-2));
        assert!(
            !run_signature(&[Some(0), Some(0)], &[false, false], &[false])?.ordered_unit_adjacency
        );
        Ok(())
    }
    #[test]
    fn malformed_shapes_and_artifact_headers_are_rejected() -> Result<()> {
        assert!(run_signature(&[Some(0)], &[], &[false; 1]).is_err());
        assert!(run_signature(&[Some(16)], &[false], &[false; 16]).is_err());
        assert!(run_signature(&[Some(0)], &[false], &[false; 17]).is_err());
        let names = feature_names();
        validate_header(2, &[327682 | STRUCTURE_BIT], &names)?;
        for rules in [vec![0], vec![1 << 20], vec![63], vec![1; 9]] {
            assert!(validate_header(2, &rules, &names).is_err());
        }
        assert!(validate_header(1, &[327682], &names).is_err());
        assert!(validate_header(2, &[327682], &names[..19]).is_err());
        let g = BoundGeometry::canonical().map_err(|_| Error::Geometry)?;
        assert!(Artifact::decode(b"{}", &g).is_err());
        assert!(Artifact::decode(&vec![b' '; MAX_ARTIFACT_BYTES + 1], &g).is_err());
        let diagnostic = run_signature(
            &[None, Some(2), Some(0), Some(1)],
            &[false, true, false, false],
            &[false; 3],
        )?;
        let encoded = serde_json::to_vec(&diagnostic).map_err(|_| Error::Artifact)?;
        assert_eq!(
            diagnostic,
            serde_json::from_slice::<RunSignature>(&encoded).map_err(|_| Error::Artifact)?
        );
        Ok(())
    }
}
