//! Bounded contiguous span candidates over the retained signed geometric match
//! relation. Exact source/word occurrence identity is separate from byte extent.
use super::{completion, scheduling, span_boundary as boundary};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime as lexical,
    ordered_state::runtime as ordered,
    relational_attention::runtime::{Error, Result},
    relative_language::runtime as reader,
};
use serde::{Deserialize, Serialize};
pub const FEATURES: usize = 19;
pub const MAX_RULES: usize = 8;
pub const MAX_SPAN_WORDS: usize = 3;
pub const MAX_SPAN_BYTES: usize = 50;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: completion::Artifact,
    pub parent_digest: [u8; 32],
    pub rules: Vec<u32>,
    pub context_words: Vec<boundary::Word>,
    pub feature_names: Vec<String>,
    pub source_digest: [u8; 32],
    pub data_digest: [u8; 32],
    pub training: String,
}
pub fn feature_names() -> Vec<String> {
    reader::FEATURE_NAMES
        .iter()
        .map(|s| s.to_string())
        .chain(["complete_local_payload_run".into()])
        .collect()
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        self.parent.validate(g)?;
        boundary::validate(g, &self.context_words)?;
        if self.schema != 2
            || self.parent_digest != *blake3::hash(&self.parent.encode()?).as_bytes()
            || self.rules.len() > MAX_RULES
            || self
                .rules
                .iter()
                .any(|&r| r == 0 || r >> FEATURES != 0 || r.count_ones() > 5)
            || self.feature_names != feature_names()
        {
            return Err(Error::Artifact);
        }
        Ok(())
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
        self.rules.iter().any(|&r| r & f == r)
    }
    pub fn reader(&self) -> &reader::Artifact {
        &self.parent.parent.parent.parent
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    ReadDisabled,
    ScorerDisabled,
    ExtentDisabled,
    BoundaryContextDisabled,
    LocalBoundaryDisabled,
    SingleWordOnly,
    PayloadFirstWord,
    CursorDisabled,
    StopDisabled,
    ExactIdentity,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Candidate {
    pub value: lexical::Candidate,
    pub last_word: usize,
    pub features: u32,
}
/// Contract only the proposed interval to one vertex for the existing topology.
/// All intervals are enumerated. Existing coverage/order predicates observe the
/// interval extent; no new feature, admission filter or answer boundary is supplied.
pub(super) fn span_features(matches: &[Vec<usize>], words: usize, first: usize, end: usize) -> u32 {
    let covered: Vec<_> = (0..words)
        .map(|j| matches.iter().any(|p| p.contains(&j)))
        .collect();
    let inside = covered[first..end].iter().any(|&x| x);
    let reduced: Vec<Vec<usize>> = matches
        .iter()
        .map(|p| {
            p.iter()
                .map(|&j| {
                    if j < first {
                        j
                    } else if j < end {
                        first
                    } else {
                        j - (end - first) + 1
                    }
                })
                .collect()
        })
        .collect();
    let mut f = reader::topology(
        &reduced,
        words - (end - first) + 1,
        first,
        reader::Control::Full,
    );
    if inside {
        f &= !1;
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
    let q = reader::words(g, question, a.reader().parent.parent.operators)?;
    if q.is_empty() || q.len() > reader::MAX_WORDS {
        return Ok(vec![]);
    }
    let mut result = Vec::new();
    for (source, raw) in records.iter().enumerate() {
        let words = reader::words(g, raw, ordered::CANONICAL)?;
        if words.is_empty() || words.len() > reader::MAX_WORDS {
            return Err(Error::Shape);
        }
        let mut matches = vec![vec![]; q.len()];
        for (i, q) in q.iter().enumerate() {
            for (j, k) in words.iter().enumerate() {
                let yes = if c == Control::ExactIdentity {
                    q.geometry.occurrences == k.geometry.occurrences
                } else {
                    ordered::distance(m, &q.geometry, &k.geometry, false)? == 0
                };
                if yes {
                    matches[i].push(j)
                }
            }
        }
        let mut barriers: Vec<_> = (0..words.len())
            .map(|j| matches.iter().any(|p| p.contains(&j)))
            .collect();
        if c != Control::BoundaryContextDisabled {
            for (j, word) in words.iter().enumerate() {
                barriers[j] |= boundary::contains(
                    m,
                    &a.context_words,
                    &word.geometry,
                    c == Control::ExactIdentity,
                )?;
            }
        }
        for first in 0..words.len() {
            for end in first + 1..=words.len().min(first + MAX_SPAN_WORDS) {
                if c == Control::SingleWordOnly && end != first + 1 {
                    continue;
                }
                let start = words[first].start;
                let stop = words[end - 1].end;
                if stop - start > MAX_SPAN_BYTES {
                    continue;
                }
                // Only whitespace joins words in a span; do not silently cross sentences.
                if (first..end - 1).any(|i| {
                    !raw[words[i].end..words[i + 1].start]
                        .iter()
                        .all(u8::is_ascii_whitespace)
                }) {
                    continue;
                }
                let features = span_features(
                    &matches,
                    words.len(),
                    first,
                    if c == Control::ExtentDisabled {
                        first + 1
                    } else {
                        end
                    },
                );
                let local = boundary::complete_run(
                    &barriers,
                    first,
                    if c == Control::ExtentDisabled {
                        first + 1
                    } else {
                        end
                    },
                );
                let expanded = features
                    | if local && c != Control::LocalBoundaryDisabled {
                        1 << 18
                    } else {
                        0
                    };
                result.push(Candidate {
                    value: lexical::Candidate {
                        source,
                        word: first,
                        features,
                        bytes: raw[start..stop].to_vec(),
                        start,
                        end: stop,
                    },
                    last_word: end,
                    features: expanded,
                });
            }
        }
    }
    Ok(result)
}
pub fn route(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<lexical::Route> {
    let disabled = match c {
        Control::ReadDisabled => Some(lexical::RouteStatus::ReadDisabled),
        Control::ScorerDisabled => Some(lexical::RouteStatus::ScorerDisabled),
        _ => None,
    };
    if let Some(status) = disabled {
        return Ok(lexical::Route {
            status,
            selected: None,
            compatible: vec![],
        });
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
    let accepted: Vec<_> = candidates(a, g, m, records, question, c)?
        .into_iter()
        .filter(|v| a.matches(v.features))
        .collect();
    let compatible = accepted
        .iter()
        .map(|v| [v.value.source, v.value.word])
        .collect();
    let status = match accepted.len() {
        0 => lexical::RouteStatus::NoCompatibleCandidate,
        1 => lexical::RouteStatus::Selected,
        _ => lexical::RouteStatus::Ambiguous,
    };
    let mut selected = if accepted.len() == 1 {
        accepted.into_iter().next().map(|v| v.value)
    } else {
        None
    };
    if c == Control::PayloadFirstWord {
        if let Some(v) = &mut selected {
            if let Some(i) = v.bytes.iter().position(u8::is_ascii_whitespace) {
                v.bytes.truncate(i);
            }
        }
    }
    Ok(lexical::Route {
        status,
        selected,
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
    let execution = match c {
        Control::CursorDisabled => scheduling::Control::CursorDisabled,
        Control::StopDisabled => scheduling::Control::StopDisabled,
        _ => scheduling::Control::Full,
    };
    completion::generate_routed(
        &a.parent,
        g,
        m,
        records,
        prompt,
        completion::Control::Full,
        execution,
        |q, _| route(a, g, m, records, q, c),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn span_extent_retains_coverage_and_excludes_matched_words() {
        let matches = vec![vec![], vec![0], vec![1], vec![2]];
        let one = span_features(&matches, 6, 3, 4);
        let all = span_features(&matches, 6, 3, 6);
        assert_eq!(one & (1 << 15), 0);
        assert_ne!(all & (1 << 15), 0);
        assert_eq!(span_features(&matches, 6, 2, 5) & 1, 0);
        assert_eq!(
            span_features(&matches, 4, 3, 4),
            reader::topology(&matches, 4, 3, reader::Control::Full)
        );
    }
}
