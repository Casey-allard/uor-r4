//! Candidate-conditioned occurrence correspondence over the existing geometric
//! equality relation. A witness never uses the proposed answer as known context.
//! Learned span predicates are evaluated per witness, without mixing features.
use super::{completion, phrase, runtime::PayloadWindow, scheduling, span, span_boundary};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime as lexical,
    ordered_state::runtime as ordered,
    relational_attention::runtime::{Error, Result},
    relative_language::runtime as reader,
};
use serde::{Deserialize, Serialize};

/// Counts all visited partial assignments across all candidate spans/records.
/// Exhaustion rejects the entire route; no truncated search can claim uniqueness.
pub const MAX_SEARCH_NODES: usize = 16_384;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Full,
    UnionMatches,
    NonInjective,
    ReadDisabled,
    ExactIdentity,
    UpdateDisabled,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Witness {
    pub query_to_source: Vec<Option<usize>>,
    pub features: u32,
    pub barriers: Vec<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub span: span::Candidate,
    pub witnesses: Vec<Witness>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Search {
    pub candidates: Vec<Candidate>,
    pub nodes: usize,
}
struct Budget {
    nodes: usize,
    limit: usize,
}
impl Budget {
    fn visit(&mut self) -> Result<()> {
        if self.nodes == self.limit {
            return Err(Error::CorrespondenceLimit { limit: self.limit });
        }
        self.nodes += 1;
        Ok(())
    }
}
/// Every originally matched query occurrence must receive a distinct occurrence
/// outside the proposed answer. Originally unmatched words retain None; candidate
/// exclusion cannot silently reclassify a known query word as unmatched syntax.
fn correspondences(
    matches: &[Vec<usize>],
    context: &[bool],
    first: usize,
    end: usize,
    injective: bool,
    budget: &mut Budget,
) -> Result<Vec<Witness>> {
    if matches.len() > reader::MAX_WORDS
        || context.len() > reader::MAX_WORDS
        || first >= end
        || end > context.len()
        || matches.iter().flatten().any(|&j| j >= context.len())
    {
        return Err(Error::Shape);
    }
    let options: Vec<Vec<usize>> = matches
        .iter()
        .map(|p| {
            let mut positions: Vec<_> = p
                .iter()
                .copied()
                .filter(|&j| j < first || j >= end)
                .collect();
            positions.sort_unstable();
            positions.dedup();
            positions
        })
        .collect();
    budget.visit()?;
    if matches
        .iter()
        .zip(&options)
        .any(|(original, outside)| !original.is_empty() && outside.is_empty())
    {
        return Ok(vec![]);
    }
    struct Walker<'a> {
        options: &'a [Vec<usize>],
        context: &'a [bool],
        first: usize,
        end: usize,
        injective: bool,
        budget: &'a mut Budget,
        witnesses: Vec<Witness>,
    }
    impl Walker<'_> {
        fn walk(
            &mut self,
            query: usize,
            used: u16,
            assignment: &mut Vec<Option<usize>>,
        ) -> Result<()> {
            self.budget.visit()?;
            if query == self.options.len() {
                let relation: Vec<Vec<usize>> = assignment
                    .iter()
                    .map(|j| j.iter().copied().collect())
                    .collect();
                let mut barriers = self.context.to_vec();
                for j in assignment.iter().flatten() {
                    barriers[*j] = true;
                }
                let features =
                    span::span_features(&relation, self.context.len(), self.first, self.end)
                        | if span_boundary::complete_run(&barriers, self.first, self.end) {
                            1 << 18
                        } else {
                            0
                        };
                self.witnesses.push(Witness {
                    query_to_source: assignment.clone(),
                    features,
                    barriers,
                });
            } else if self.options[query].is_empty() {
                assignment.push(None);
                self.walk(query + 1, used, assignment)?;
                assignment.pop();
            } else {
                for &j in &self.options[query] {
                    let bit = 1u16 << j;
                    if self.injective && used & bit != 0 {
                        continue;
                    }
                    assignment.push(Some(j));
                    self.walk(query + 1, used | bit, assignment)?;
                    assignment.pop();
                }
            }
            Ok(())
        }
    }
    let mut walker = Walker {
        options: &options,
        context,
        first,
        end,
        injective,
        budget,
        witnesses: vec![],
    };
    walker.walk(0, 0, &mut Vec::with_capacity(matches.len()))?;
    Ok(walker.witnesses)
}

pub fn candidates(
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<Search> {
    if m.geometry_digest() != g.id() {
        return Err(Error::Geometry);
    }
    let old_control = if c == Control::ExactIdentity {
        span::Control::ExactIdentity
    } else {
        span::Control::Full
    };
    // Reuse exactly the retained span admission/window/byte-bound enumeration.
    let spans = span::candidates(a, g, m, records, question, old_control)?;
    let q = reader::words(g, question, a.reader().parent.parent.operators)?;
    let mut budget = Budget {
        nodes: 0,
        limit: MAX_SEARCH_NODES,
    };
    let mut result = Vec::with_capacity(spans.len());
    for (source, raw) in records.iter().enumerate() {
        let words = reader::words(g, raw, ordered::CANONICAL)?;
        let mut matches = vec![vec![]; q.len()];
        for (i, qw) in q.iter().enumerate() {
            for (j, sw) in words.iter().enumerate() {
                if if c == Control::ExactIdentity {
                    qw.geometry.occurrences == sw.geometry.occurrences
                } else {
                    ordered::distance(m, &qw.geometry, &sw.geometry, false)? == 0
                } {
                    matches[i].push(j);
                }
            }
        }
        let context: Vec<bool> = words
            .iter()
            .map(|word| {
                span_boundary::contains(
                    m,
                    &a.context_words,
                    &word.geometry,
                    c == Control::ExactIdentity,
                )
            })
            .collect::<Result<_>>()?;
        for candidate in spans.iter().filter(|v| v.value.source == source) {
            let witnesses = correspondences(
                &matches,
                &context,
                candidate.value.word,
                candidate.last_word,
                c != Control::NonInjective,
                &mut budget,
            )?;
            result.push(Candidate {
                span: candidate.clone(),
                witnesses,
            });
        }
    }
    Ok(Search {
        candidates: result,
        nodes: budget.nodes,
    })
}
fn select_offers(search: Search, accepts: impl Fn(u32) -> bool) -> lexical::Route {
    let mut accepted = Vec::new();
    for candidate in search.candidates {
        // Several witnesses for one exact source/span still represent one offer.
        // The first accepting witness is only a deterministic trace representative.
        if let Some(witness) = candidate.witnesses.iter().find(|w| accepts(w.features)) {
            let mut value = candidate.span.value;
            value.features = witness.features & ((1 << 18) - 1);
            accepted.push(value);
        }
    }
    let compatible = accepted.iter().map(|v| [v.source, v.word]).collect();
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
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<lexical::Route> {
    if c == Control::UnionMatches || c == Control::ReadDisabled {
        return span::route(
            a,
            g,
            m,
            records,
            question,
            if c == Control::ReadDisabled {
                span::Control::ReadDisabled
            } else {
                span::Control::Full
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
    Ok(select_offers(
        candidates(a, g, m, records, question, c)?,
        |f| a.matches(f),
    ))
}
pub fn generate(
    a: &span::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    c: Control,
) -> Result<completion::Generated> {
    if c == Control::UnionMatches {
        return phrase::generate(a, g, m, records, prompt, phrase::Control::Full);
    }
    a.validate(g)?;
    completion::generate_routed_payload(
        &a.parent,
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
        |query, _| route(a, g, m, records, query, c),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn enumerate(
        matches: &[Vec<usize>],
        context: &[bool],
        first: usize,
        end: usize,
        injective: bool,
    ) -> Result<Vec<Witness>> {
        correspondences(
            matches,
            context,
            first,
            end,
            injective,
            &mut Budget {
                nodes: 0,
                limit: MAX_SEARCH_NODES,
            },
        )
    }
    #[test]
    fn overlapping_text_keeps_answer_occurrences_out_of_known_context() -> Result<()> {
        // who did ruby amber help? / ruby amber did help ruby birch.
        let matches = vec![vec![], vec![2], vec![0, 4], vec![1], vec![3]];
        let witnesses = enumerate(
            &matches,
            &[false, false, true, true, false, false],
            4,
            6,
            true,
        )?;
        assert_eq!(witnesses.len(), 1);
        assert_eq!(
            witnesses[0].query_to_source,
            vec![None, Some(2), Some(0), Some(1), Some(3)]
        );
        assert_eq!(witnesses[0].features & 327682, 327682);
        assert!(!witnesses[0].barriers[4]);
        assert!(enumerate(
            &matches,
            &[false, false, true, true, false, false],
            0,
            2,
            true
        )?
        .is_empty());
        Ok(())
    }
    #[test]
    fn repeated_query_occurrences_require_distinct_source_occurrences() -> Result<()> {
        let matches = vec![vec![], vec![1], vec![0], vec![0], vec![2]];
        let context = [false, true, true, false];
        assert!(enumerate(&matches, &context, 3, 4, true)?.is_empty());
        let noninjective = enumerate(&matches, &context, 3, 4, false)?;
        assert_eq!(noninjective.len(), 1);
        assert_eq!(noninjective[0].features & 327682, 327682);
        let repeated = vec![vec![], vec![2], vec![0, 1], vec![0, 1], vec![3]];
        let valid = enumerate(&repeated, &[false, false, true, true, false], 4, 5, true)?;
        assert_eq!(valid.len(), 2);
        for w in valid {
            assert_ne!(w.query_to_source[2], w.query_to_source[3]);
        }
        Ok(())
    }
    #[test]
    fn identical_endpoints_use_query_order_in_both_roles() -> Result<()> {
        let context = [false, false, true, true, false, false];
        for (matches, correct, wrong) in [
            (
                vec![vec![], vec![2], vec![0, 4], vec![1, 5], vec![3]],
                (4, 6),
                (0, 2),
            ),
            (
                vec![vec![], vec![2], vec![3], vec![0, 4], vec![1, 5]],
                (0, 2),
                (4, 6),
            ),
        ] {
            assert!(enumerate(&matches, &context, correct.0, correct.1, true)?
                .iter()
                .any(|w| w.features & 327682 == 327682));
            assert!(enumerate(&matches, &context, wrong.0, wrong.1, true)?
                .iter()
                .all(|w| w.features & 327682 != 327682));
        }
        Ok(())
    }
    #[test]
    fn witnesses_do_not_mix_features_or_multiply_one_offer() {
        let candidate = |source, features: Vec<u32>| Candidate {
            span: span::Candidate {
                value: lexical::Candidate {
                    source,
                    word: 3,
                    start: 12,
                    end: 16,
                    bytes: b"ruby".to_vec(),
                    features: 0,
                },
                last_word: 4,
                features: 0,
            },
            witnesses: features
                .into_iter()
                .map(|features| Witness {
                    query_to_source: vec![],
                    barriers: vec![],
                    features,
                })
                .collect(),
        };
        let accepts = |f| f & 327682 == 327682;
        let split = select_offers(
            Search {
                candidates: vec![candidate(0, vec![2, 327680])],
                nodes: 2,
            },
            accepts,
        );
        assert_eq!(split.status, lexical::RouteStatus::NoCompatibleCandidate);
        let same = select_offers(
            Search {
                candidates: vec![candidate(0, vec![327682, 327683])],
                nodes: 2,
            },
            accepts,
        );
        assert_eq!(same.status, lexical::RouteStatus::Selected);
        assert_eq!(same.compatible, vec![[0, 3]]);
        let separate = select_offers(
            Search {
                candidates: vec![candidate(0, vec![327682]), candidate(1, vec![327682])],
                nodes: 2,
            },
            accepts,
        );
        assert_eq!(separate.status, lexical::RouteStatus::Ambiguous);
        assert!(separate.selected.is_none());
    }
    #[test]
    fn exhaustive_correspondence_limit_returns_error_not_partial_offers() {
        let matches = vec![vec![0, 1, 2]; 3];
        let mut budget = Budget { nodes: 0, limit: 5 };
        assert!(matches!(
            correspondences(&matches, &[false; 4], 3, 4, true, &mut budget),
            Err(Error::CorrespondenceLimit { limit: 5 })
        ));
        assert_eq!(budget.nodes, 5);
        assert!(matches!(
            enumerate(&[vec![16]], &[false; 4], 3, 4, true),
            Err(Error::Shape)
        ));
    }
}
