use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    ordered_state::runtime as ordered,
    recurrent_text::runtime::{self as recurrent, Action, Observation, State},
    relational_attention::runtime::{Error, Result},
};
use serde::{Deserialize, Serialize};
pub const WORDS: usize = 4;
pub const FEATURES: usize = 20;
pub const MAX_RULES: usize = 8;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    pub bytes: Vec<u8>,
    pub start: usize,
    pub end: usize,
    pub geometry: ordered::Query,
}
/// Lexical boundaries only: maximal ASCII alphabetic runs, preserving offsets.
/// There is no name/relation/question dictionary, stemmer or role parser.
pub fn words(g: &BoundGeometry, bytes: &[u8], ops: [ordered::Operator; 2]) -> Result<Vec<Word>> {
    if bytes.len() > 128 {
        return Err(Error::Shape);
    }
    let mut result = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_lowercase() {
            if !bytes[i].is_ascii_whitespace() && !b".?".contains(&bytes[i]) {
                return Err(Error::Shape);
            }
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_lowercase() {
            i += 1;
        }
        if i - start > 16 {
            return Err(Error::Shape);
        }
        let value = bytes[start..i].to_vec();
        result.push(Word {
            geometry: ordered::Query::encode(g, &value, ops)?,
            bytes: value,
            start,
            end: i,
        });
    }
    Ok(result)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: ordered::Artifact,
    pub parent_digest: [u8; 32],
    pub rules: Vec<u32>,
    pub data_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 1
            || self.rules.len() > MAX_RULES
            || self
                .rules
                .iter()
                .any(|&r| r == 0 || r >> FEATURES != 0 || r.count_ones() > 3)
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
    pub fn recurrent(&self) -> recurrent::Artifact {
        self.parent.parent.clone()
    }
    /// Host adapter for replaying the same active action parameters on old keys.
    pub fn ordered(&self) -> Result<ordered::Artifact> {
        let mut a = self.parent.clone();
        a.parent = self.recurrent();
        a.parent_digest = *blake3::hash(&a.parent.encode()?).as_bytes();
        Ok(a)
    }
    pub fn matches(&self, features: u32) -> bool {
        self.rules.iter().any(|&rule| features & rule == rule)
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    ReadDisabled,
    ScorerDisabled,
    OrderErased,
    RecordMiddleDisabled,
    PositionErased,
    ExactIdentity,
    FinalRootOnly,
    FeedbackDisabled,
    CursorDisabled,
    StopDisabled,
}
fn core_control(c: Control) -> recurrent::Control {
    match c {
        Control::ReadDisabled => recurrent::Control::ReadDisabled,
        Control::FeedbackDisabled => recurrent::Control::FeedbackDisabled,
        Control::CursorDisabled => recurrent::Control::CursorDisabled,
        Control::StopDisabled => recurrent::Control::StopDisabled,
        _ => recurrent::Control::Full,
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Candidate {
    pub source: usize,
    pub word: usize,
    pub features: u32,
    pub bytes: Vec<u8>,
    pub start: usize,
    pub end: usize,
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
    let mut q = words(g, question, a.parent.operators)?;
    if q.len() != WORDS {
        return Ok(Vec::new());
    }
    if c == Control::OrderErased {
        q.sort_by(|a, b| a.bytes.cmp(&b.bytes));
    }
    let mut candidates = Vec::new();
    for (source, record) in records.iter().enumerate() {
        let w = words(g, record, ordered::CANONICAL)?;
        if w.len() != WORDS {
            return Err(Error::Shape);
        }
        let mut features = 0u32;
        for (i, q) in q.iter().enumerate() {
            for (j, k) in w.iter().enumerate() {
                if c == Control::RecordMiddleDisabled && j == 2 {
                    continue;
                }
                let compatible = if c == Control::ExactIdentity {
                    q.geometry.occurrences == k.geometry.occurrences
                } else {
                    ordered::distance(m, &q.geometry, &k.geometry, c == Control::FinalRootOnly)?
                        == 0
                };
                if compatible {
                    features |= 1 << (i * WORDS + j);
                }
            }
        }
        for (word, value) in w.into_iter().enumerate() {
            candidates.push(Candidate {
                source,
                word,
                features: features
                    | if c == Control::PositionErased {
                        15 << 16
                    } else {
                        1 << (16 + word)
                    },
                bytes: value.bytes,
                start: value.start,
                end: value.end,
            });
        }
    }
    Ok(candidates)
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RouteStatus {
    Selected,
    NoCompatibleCandidate,
    Ambiguous,
    UnsupportedWordWindow,
    ReadDisabled,
    ScorerDisabled,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Route {
    pub status: RouteStatus,
    pub selected: Option<Candidate>,
    pub compatible: Vec<[usize; 2]>,
}
pub fn route(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    c: Control,
) -> Result<Route> {
    let status = if c == Control::ReadDisabled {
        Some(RouteStatus::ReadDisabled)
    } else if c == Control::ScorerDisabled {
        Some(RouteStatus::ScorerDisabled)
    } else {
        None
    };
    if let Some(status) = status {
        return Ok(Route {
            status,
            selected: None,
            compatible: vec![],
        });
    }
    // Generated feedback can leave this adapter's declared four-word window.
    // That is unsupported routing context, not a claim that a fact is absent.
    let supported = match words(g, question, a.parent.operators) {
        Ok(q) => q.len() == WORDS,
        Err(Error::Shape) => false,
        Err(e) => return Err(e),
    };
    if !supported {
        return Ok(Route {
            status: RouteStatus::UnsupportedWordWindow,
            selected: None,
            compatible: vec![],
        });
    }
    let all = candidates(a, g, m, records, question, c)?;
    if all.is_empty() {
        return Ok(Route {
            status: RouteStatus::UnsupportedWordWindow,
            selected: None,
            compatible: vec![],
        });
    }
    let accepted: Vec<_> = all.into_iter().filter(|x| a.matches(x.features)).collect();
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Query {
    pub bytes: Vec<u8>,
    pub geometry: ordered::Query,
}
impl Query {
    pub fn new(g: &BoundGeometry, b: &[u8], ops: [ordered::Operator; 2]) -> Result<Self> {
        Ok(Self {
            bytes: b.to_vec(),
            geometry: ordered::Query::encode(g, b, ops)?,
        })
    }
    fn push(&self, g: &BoundGeometry, b: u8, ops: [ordered::Operator; 2]) -> Result<Self> {
        let mut q = self.clone();
        q.geometry = q.geometry.push(g, b, ops)?;
        q.bytes.push(b);
        Ok(q)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Generated {
    pub actual: recurrent::Generated<Query>,
    pub routes: Vec<Route>,
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
    if words(g, question, a.parent.operators)?.len() != WORDS {
        return Err(Error::Shape);
    }
    for r in records {
        if words(g, r, ordered::CANONICAL)?.len() != WORDS {
            return Err(Error::Shape);
        }
    }
    let core = a.recurrent();
    let mut s = State::new(Query::new(g, question, a.parent.operators)?);
    let mut tokens = Vec::new();
    let mut steps = Vec::new();
    let mut routes = Vec::new();
    for _ in 0..recurrent::MAX_STEPS {
        let r = route(a, g, m, records, &s.query.bytes, c)?;
        let value = r
            .selected
            .as_ref()
            .and_then(|x| x.bytes.get(s.cursor))
            .copied();
        let present = value.is_some();
        let byte = value.unwrap_or(s.last);
        if present && s.pending.bytes.len() >= ordered::MAX_PREFIX {
            return Ok(Generated {
                actual: recurrent::Generated {
                    tokens,
                    steps,
                    exhausted: true,
                },
                routes,
            });
        }
        let next_query = if present {
            s.pending.push(g, byte, a.parent.operators)?
        } else {
            s.pending.clone()
        };
        let next = route(a, g, m, records, &next_query.bytes, c)?;
        let row = usize::from(byte)
            + 256 * usize::from(present)
            + 512
            + 1024 * usize::from(next.selected.is_some());
        let o = Observation {
            selected: r.selected.as_ref().map(|x| x.source),
            byte,
            present,
            next_query,
            next_available: next.selected.is_some(),
            row,
        };
        // Reuse the already learned byte/EOS decision for a lexical span.
        // This one-read adapter does not relearn a byte-indexed stopping policy.
        let symbol = crate::native_geometric::text_attention::runtime::symbol(
            &a.parent.parent.parent,
            byte,
            present,
        )?;
        let action = if symbol == 256 {
            Action::Stop
        } else {
            Action::Emit
        };
        let before = s.clone();
        let token =
            recurrent::execute_with_update(&core, &o, &mut s, action, core_control(c), |q, b| {
                q.push(g, b, a.parent.operators)
            })?;
        if let Some(t) = token {
            tokens.push(t);
        }
        steps.push(recurrent::Step {
            before,
            observation: o,
            action,
            token,
            after: s.clone(),
        });
        routes.push(r);
        if s.done || s.exhausted {
            return Ok(Generated {
                actual: recurrent::Generated {
                    tokens,
                    steps,
                    exhausted: s.exhausted,
                },
                routes,
            });
        }
    }
    Ok(Generated {
        actual: recurrent::Generated {
            tokens,
            steps,
            exhausted: true,
        },
        routes,
    })
}
