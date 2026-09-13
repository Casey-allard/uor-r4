use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime as prior,
    ordered_state::runtime as ordered,
    relational_attention::runtime::{Error, Result},
};
pub use prior::{words, Candidate, Generated, Route, RouteStatus};
use serde::{Deserialize, Serialize};
pub const MAX_WORDS: usize = 16;
pub const FEATURES: usize = 18;
pub const MAX_RULES: usize = 8;
/// Feature identities are artifact-bound. These are generic relations between
/// matched occurrences, not lexical classes, sentence templates or source slots.
pub const FEATURE_NAMES: [&str; FEATURES] = [
    "candidate_unmatched",
    "unmatched_query_is_prefix",
    "two_query_matches",
    "three_query_matches",
    "query_has_unmatched",
    "matches_on_both_sides",
    "candidate_before_all_matches",
    "candidate_after_all_matches",
    "candidate_immediately_before_first_match",
    "candidate_immediately_after_last_match",
    "match_order_increasing",
    "match_order_has_descent",
    "one_descent",
    "multiple_descents",
    "one_to_one_matches",
    "all_source_except_candidate_matched",
    "last_query_match_is_last_source_match",
    "first_query_match_is_first_source_match",
];
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: prior::Artifact,
    pub parent_digest: [u8; 32],
    pub rules: Vec<u32>,
    pub data_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub feature_names: Vec<String>,
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 2
            || self.rules.len() > MAX_RULES
            || self
                .rules
                .iter()
                .any(|&r| r == 0 || r >> FEATURES != 0 || r.count_ones() > 4)
            || self.feature_names != FEATURE_NAMES
            || *blake3::hash(&self.parent.encode()?).as_bytes() != self.parent_digest
        {
            return Err(Error::Artifact);
        }
        self.parent.validate(g)
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| Error::Artifact)
    }
    pub fn decode(b: &[u8], g: &BoundGeometry) -> Result<Self> {
        if b.len() > 4 * 1024 * 1024 {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(b).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
    pub fn matches(&self, f: u32) -> bool {
        self.rules.iter().any(|&r| f & r == r)
    }
    pub fn recurrent(&self) -> crate::native_geometric::recurrent_text::runtime::Artifact {
        self.parent.recurrent()
    }
    pub fn ordered(&self) -> Result<ordered::Artifact> {
        self.parent.ordered()
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    ReadDisabled,
    ScorerDisabled,
    OrderErased,
    CoverageDisabled,
    RelativePositionDisabled,
    MatchOrderDisabled,
    EndpointDisabled,
    ExactIdentity,
    FinalRootOnly,
    FeedbackDisabled,
    CursorDisabled,
    StopDisabled,
}
fn prior_control(c: Control) -> prior::Control {
    match c {
        Control::ReadDisabled => prior::Control::ReadDisabled,
        Control::FeedbackDisabled => prior::Control::FeedbackDisabled,
        Control::CursorDisabled => prior::Control::CursorDisabled,
        Control::StopDisabled => prior::Control::StopDisabled,
        _ => prior::Control::Full,
    }
}
/// Topology of a bounded bipartite word-match relation. Equality observations
/// come from signed-prefix Hamming distance; this function never reads word IDs.
fn topology(matches: &[Vec<usize>], source_words: usize, candidate: usize, c: Control) -> u32 {
    let qmatched: Vec<_> = matches.iter().map(|v| !v.is_empty()).collect();
    let positions: Vec<_> = matches.iter().flatten().copied().collect();
    if positions.is_empty() {
        return 0;
    }
    let first = *positions.iter().min().unwrap_or(&0);
    let last = *positions.iter().max().unwrap_or(&0);
    let count = qmatched.iter().filter(|&&b| b).count();
    let qfirst = qmatched.iter().position(|&b| b).unwrap_or(qmatched.len());
    let descents = positions.windows(2).filter(|p| p[0] >= p[1]).count();
    let mut unique = positions.clone();
    unique.sort_unstable();
    unique.dedup();
    let flags = [
        !positions.contains(&candidate),
        qmatched[qfirst..].iter().all(|&b| b),
        count >= 2,
        count >= 3,
        count < qmatched.len(),
        first < candidate && last > candidate,
        candidate < first,
        candidate > last,
        candidate + 1 == first,
        candidate == last + 1,
        descents == 0,
        descents > 0,
        descents == 1,
        descents > 1,
        positions.len() == count && unique.len() == count,
        unique.len() + 1 == source_words && !positions.contains(&candidate),
        positions.last() == Some(&last),
        positions.first() == Some(&first),
    ];
    let mut f = 0;
    for (i, yes) in flags.into_iter().enumerate() {
        if yes {
            f |= 1 << i;
        }
    }
    // Erasure supplies identical relation evidence to all candidates; it never
    // resolves ambiguity by source order. Coverage/order interventions remove
    // their features without changing admitted occurrences or payload bytes.
    match c {
        Control::CoverageDisabled => {
            f &= !((1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 14) | (1 << 15))
        }
        Control::RelativePositionDisabled => {
            f |= (1 << 0) | (1 << 5) | (1 << 6) | (1 << 7) | (1 << 8) | (1 << 9) | (1 << 15);
        }
        Control::EndpointDisabled => f &= !((1 << 16) | (1 << 17)),
        Control::MatchOrderDisabled => f &= !((1 << 10) | (1 << 11) | (1 << 12) | (1 << 13)),
        _ => {}
    }
    f
}
pub fn candidates(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<Vec<Candidate>> {
    if m.geometry_digest() != g.id() {
        return Err(Error::Geometry);
    }
    let mut q = words(g, question, a.parent.parent.operators)?;
    if q.is_empty() || q.len() > MAX_WORDS {
        return Ok(vec![]);
    }
    if c == Control::OrderErased {
        q.sort_by(|a, b| a.bytes.cmp(&b.bytes));
    }
    let mut out = Vec::new();
    for (source, record) in records.iter().enumerate() {
        let w = words(g, record, ordered::CANONICAL)?;
        if w.is_empty() || w.len() > MAX_WORDS {
            return Err(Error::Shape);
        }
        let mut matches = vec![vec![]; q.len()];
        for (i, q) in q.iter().enumerate() {
            for (j, k) in w.iter().enumerate() {
                let yes = if c == Control::ExactIdentity {
                    q.geometry.occurrences == k.geometry.occurrences
                } else {
                    ordered::distance(m, &q.geometry, &k.geometry, c == Control::FinalRootOnly)?
                        == 0
                };
                if yes {
                    matches[i].push(j);
                }
            }
        }
        for (word, value) in w.iter().enumerate() {
            out.push(Candidate {
                source,
                word,
                features: topology(&matches, w.len(), word, c),
                bytes: value.bytes.clone(),
                start: value.start,
                end: value.end,
            });
        }
    }
    Ok(out)
}
pub fn route(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<Route> {
    let status = match c {
        Control::ReadDisabled => Some(RouteStatus::ReadDisabled),
        Control::ScorerDisabled => Some(RouteStatus::ScorerDisabled),
        _ => None,
    };
    if let Some(status) = status {
        return Ok(Route {
            status,
            selected: None,
            compatible: vec![],
        });
    }
    match words(g, question, a.parent.parent.operators) {
        Ok(q) if !q.is_empty() && q.len() <= MAX_WORDS => {}
        Ok(_) | Err(Error::Shape) => {
            return Ok(Route {
                status: RouteStatus::UnsupportedWordWindow,
                selected: None,
                compatible: vec![],
            })
        }
        Err(e) => return Err(e),
    }
    let accepted: Vec<_> = candidates(a, g, m, records, question, c)?
        .into_iter()
        .filter(|x| a.matches(x.features))
        .collect();
    let compatible = accepted.iter().map(|x| [x.source, x.word]).collect();
    let status = match accepted.len() {
        0 => RouteStatus::NoCompatibleCandidate,
        1 => RouteStatus::Selected,
        _ => RouteStatus::Ambiguous,
    };
    Ok(Route {
        status,
        selected: if accepted.len() == 1 {
            accepted.into_iter().next()
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
    question: &[u8],
    c: Control,
) -> Result<Generated> {
    a.validate(g)?;
    for raw in records.iter().map(Vec::as_slice).chain([question]) {
        let w = words(g, raw, ordered::CANONICAL)?;
        if w.is_empty() || w.len() > MAX_WORDS {
            return Err(Error::Shape);
        }
    }
    prior::generate_with_route(&a.parent, g, question, prior_control(c), |q| {
        route(a, g, m, records, q, c)
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relative_topology_survives_unmatched_prefix_and_retains_order() {
        let base = vec![vec![], vec![1], vec![2], vec![3]];
        let shifted = vec![vec![], vec![], vec![3], vec![4], vec![5]];
        assert_eq!(
            topology(&base, 4, 0, Control::Full) & 0x7fff,
            topology(&shifted, 7, 2, Control::Full) & 0x7fff
        );
        let reversed = vec![vec![], vec![1], vec![3], vec![2]];
        assert_ne!(
            topology(&base, 4, 0, Control::Full),
            topology(&reversed, 4, 0, Control::Full)
        );
        assert_eq!(topology(&base, 4, 0, Control::CoverageDisabled) & 2, 0);
    }
    #[test]
    fn endpoint_retains_the_demonstrated_role_reversal_distinction() {
        let correct_object = vec![vec![], vec![1], vec![0], vec![2]];
        let wrong_object = vec![vec![], vec![1], vec![2], vec![0]];
        let a = topology(&correct_object, 4, 3, Control::Full);
        let b = topology(&wrong_object, 4, 3, Control::Full);
        assert_eq!(a & 0xffff, b & 0xffff);
        assert_ne!(a, b);
        assert_eq!(a & (1 << 16), 1 << 16);
        assert_eq!(
            topology(&correct_object, 4, 3, Control::EndpointDisabled),
            topology(&wrong_object, 4, 3, Control::EndpointDisabled)
        );
    }
}
