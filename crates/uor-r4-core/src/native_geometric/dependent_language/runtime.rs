use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime as lexical,
    ordered_state::runtime as ordered,
    recurrent_text::runtime::{self as recurrent, Action, Observation, State},
    relational_attention::runtime::{Error, Result},
    relative_language::runtime as reader,
};
use serde::{Deserialize, Serialize};
pub const FEATURES: usize = 8;
pub const MAX_RULES: usize = 8;
pub const FEATURE_NAMES: [&str; FEATURES] = [
    "unmatched_in_context",
    "context_match_before",
    "context_match_after",
    "previous_word_matches",
    "next_word_matches",
    "first_word",
    "last_word",
    "matches_intermediate",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: reader::Artifact,
    pub parent_digest: [u8; 32],
    pub rules: Vec<u32>,
    pub feature_names: Vec<String>,
    pub source_digest: [u8; 32],
    pub data_digest: [u8; 32],
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
    pub fn recurrent(&self) -> recurrent::Artifact {
        self.parent.recurrent()
    }
    pub fn ordered(&self) -> Result<ordered::Artifact> {
        self.parent.ordered()
    }
    pub fn matches(&self, f: u32) -> bool {
        self.rules.iter().any(|&r| r & f == r)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Full,
    FirstReadDisabled,
    SecondReadDisabled,
    ScorerDisabled,
    UpdateDisabled,
    PayloadReversed,
    PositionDisabled,
    ContextMatchDisabled,
    SwapQuestions,
    ExactIdentity,
    FeedbackDisabled,
    CursorDisabled,
    StopDisabled,
}
/// An explicit protocol boundary, not a learned clause parser. Both complete
/// questions come from raw user bytes; no intermediate query/value is supplied.
pub fn questions(prompt: &[u8]) -> Result<[Vec<u8>; 2]> {
    if prompt.len() > 256 {
        return Err(Error::Shape);
    }
    let mut out = Vec::new();
    let mut start = 0;
    for (i, &b) in prompt.iter().enumerate() {
        if b == b'?' {
            let mut s = start;
            while s < i && prompt[s].is_ascii_whitespace() {
                s += 1;
            }
            if s == i || i + 1 - s > 128 {
                return Err(Error::Shape);
            }
            out.push(prompt[s..=i].to_vec());
            start = i + 1;
        }
    }
    if out.len() != 2 || prompt[start..].iter().any(|b| !b.is_ascii_whitespace()) {
        return Err(Error::Shape);
    }
    out.try_into().map_err(|_| Error::Shape)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateCandidate {
    pub word: usize,
    pub start: usize,
    pub end: usize,
    pub features: u32,
    pub question: Vec<u8>,
}
/// Payload admission is separate from the learned replacement-site score.
/// The historical entrypoint retains its one-word contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadWindow {
    Word,
    Phrase,
}
impl PayloadWindow {
    fn accepts(self, payload: &[u8]) -> bool {
        if self == Self::Word {
            return !payload.is_empty()
                && payload.len() <= 16
                && payload.iter().all(u8::is_ascii_lowercase);
        }
        if payload.is_empty() || payload.len() > 50 {
            return false;
        }
        let words: Vec<_> = payload.split(|b| *b == b' ').collect();
        words.len() <= 3
            && words
                .iter()
                .all(|w| !w.is_empty() && w.len() <= 16 && w.iter().all(u8::is_ascii_lowercase))
    }
}
pub fn splice(question: &[u8], start: usize, end: usize, payload: &[u8]) -> Result<Vec<u8>> {
    splice_with_window(question, start, end, payload, PayloadWindow::Word)
}
pub fn splice_with_window(
    question: &[u8],
    start: usize,
    end: usize,
    payload: &[u8],
    window: PayloadWindow,
) -> Result<Vec<u8>> {
    if question.len() > 128
        || start >= end
        || end > question.len()
        || !window.accepts(payload)
        || !question[start..end].iter().all(u8::is_ascii_lowercase)
        || (start > 0 && question[start - 1].is_ascii_lowercase())
        || (end < question.len() && question[end].is_ascii_lowercase())
    {
        return Err(Error::Shape);
    }
    let mut out = question[..start].to_vec();
    out.extend_from_slice(payload);
    out.extend_from_slice(&question[end..]);
    if out.len() > 128 {
        return Err(Error::Shape);
    }
    Ok(out)
}
pub fn updates(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    payload: &[u8],
    control: Control,
) -> Result<Vec<UpdateCandidate>> {
    updates_with_window(
        a,
        g,
        m,
        records,
        question,
        payload,
        control,
        PayloadWindow::Word,
    )
}
pub fn updates_with_window(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    question: &[u8],
    payload: &[u8],
    control: Control,
    window: PayloadWindow,
) -> Result<Vec<UpdateCandidate>> {
    if m.geometry_digest() != g.id() {
        return Err(Error::Geometry);
    }
    let ops = a.parent.parent.parent.operators;
    let q = reader::words(g, question, ops)?;
    if q.is_empty() || q.len() > reader::MAX_WORDS || !window.accepts(payload) {
        return Err(Error::Shape);
    }
    let mut context = Vec::new();
    for r in records {
        let w = reader::words(g, r, ordered::CANONICAL)?;
        if w.is_empty() || w.len() > reader::MAX_WORDS {
            return Err(Error::Shape);
        }
        context.extend(w);
    }
    let mut matched = vec![false; q.len()];
    for (i, x) in q.iter().enumerate() {
        for y in &context {
            if if control == Control::ExactIdentity {
                x.geometry.occurrences == y.geometry.occurrences
            } else {
                ordered::distance(m, &x.geometry, &y.geometry, false)? == 0
            } {
                matched[i] = true;
                break;
            }
        }
    }
    let payload_geo = ordered::Query::encode(g, payload, ordered::CANONICAL)?;
    let mut out = Vec::new();
    for (i, x) in q.iter().enumerate() {
        let flags = [
            !matched[i],
            matched[..i].iter().any(|&b| b),
            matched[i + 1..].iter().any(|&b| b),
            i > 0 && matched[i - 1],
            i + 1 < q.len() && matched[i + 1],
            i == 0,
            i + 1 == q.len(),
            ordered::distance(m, &x.geometry, &payload_geo, false)? == 0,
        ];
        let mut features = 0;
        for (j, b) in flags.into_iter().enumerate() {
            if b {
                features |= 1 << j;
            }
        }
        if control == Control::PositionDisabled {
            features |= 0x7e;
        }
        if control == Control::ContextMatchDisabled {
            features &= !0x1f;
        }
        let next = splice_with_window(question, x.start, x.end, payload, window)?;
        if window == PayloadWindow::Phrase
            && reader::words(g, &next, ops)?.len() > reader::MAX_WORDS
        {
            return Err(Error::Shape);
        }
        out.push(UpdateCandidate {
            word: i,
            start: x.start,
            end: x.end,
            features,
            question: next,
        });
    }
    Ok(out)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Completed,
    MissingFirstRead,
    NoUpdate,
    AmbiguousUpdate,
    TransitionFailed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generated {
    pub status: Status,
    pub first: lexical::Route,
    pub update: Option<UpdateCandidate>,
    pub transition: Option<recurrent::Step<lexical::Query>>,
    pub output: Option<lexical::Generated>,
}
impl Generated {
    pub fn tokens(&self) -> Vec<u16> {
        self.output
            .as_ref()
            .map(|o| o.actual.tokens.clone())
            .unwrap_or_default()
    }
    pub fn exhausted(&self) -> bool {
        self.status != Status::Completed || self.output.as_ref().is_none_or(|o| o.actual.exhausted)
    }
}
pub fn finish(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    first_question: &[u8],
    first: lexical::Route,
    update: UpdateCandidate,
    c: Control,
) -> Result<Generated> {
    let mut state = State::new(lexical::Query::new(
        g,
        first_question,
        a.parent.parent.parent.operators,
    )?);
    let next_query = if c == Control::UpdateDisabled {
        state.query.clone()
    } else {
        lexical::Query::new(g, &update.question, a.parent.parent.parent.operators)?
    };
    let next_available = reader::route(
        &a.parent,
        g,
        m,
        records,
        &next_query.bytes,
        if c == Control::SecondReadDisabled {
            reader::Control::ReadDisabled
        } else {
            reader::Control::Full
        },
    )?
    .selected
    .is_some();
    let before = state.clone();
    let o = Observation {
        selected: first.selected.as_ref().map(|v| v.source),
        byte: first
            .selected
            .as_ref()
            .and_then(|v| v.bytes.first())
            .copied()
            .unwrap_or(0),
        present: first.selected.is_some(),
        next_query,
        next_available,
        row: 0,
    };
    // One declared question boundary requests a typed silent Read. The learned
    // update picks its argument; read scheduling and the two-question protocol
    // are explicit here, not newly learned from the answer.
    let token = recurrent::execute_with_update(
        &a.parent.recurrent(),
        &o,
        &mut state,
        Action::Read,
        recurrent::Control::Full,
        |q, _| Ok(q.clone()),
    )?;
    let step = recurrent::Step {
        before,
        observation: o,
        action: Action::Read,
        token,
        after: state.clone(),
    };
    if state.exhausted || token.is_some() {
        return Ok(Generated {
            status: Status::TransitionFailed,
            first,
            update: Some(update),
            transition: Some(step),
            output: None,
        });
    }
    let lc = match c {
        Control::FeedbackDisabled => lexical::Control::FeedbackDisabled,
        Control::CursorDisabled => lexical::Control::CursorDisabled,
        Control::StopDisabled => lexical::Control::StopDisabled,
        _ => lexical::Control::Full,
    };
    let rc = match c {
        Control::SecondReadDisabled => reader::Control::ReadDisabled,
        Control::ExactIdentity => reader::Control::ExactIdentity,
        _ => reader::Control::Full,
    };
    let output = lexical::generate_from_state(&a.parent.parent, g, state, lc, |q| {
        reader::route(&a.parent, g, m, records, q, rc)
    })?;
    Ok(Generated {
        status: Status::Completed,
        first,
        update: Some(update),
        transition: Some(step),
        output: Some(output),
    })
}
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    prompt: &[u8],
    c: Control,
) -> Result<Generated> {
    a.validate(g)?;
    let mut qs = questions(prompt)?;
    if c == Control::SwapQuestions {
        qs.swap(0, 1);
    }
    for q in &qs {
        let words = reader::words(g, q, ordered::CANONICAL)?;
        if words.is_empty() || words.len() > reader::MAX_WORDS {
            return Err(Error::Shape);
        }
    }
    let rc = match c {
        Control::FirstReadDisabled => reader::Control::ReadDisabled,
        Control::ExactIdentity => reader::Control::ExactIdentity,
        _ => reader::Control::Full,
    };
    let first = reader::route(&a.parent, g, m, records, &qs[0], rc)?;
    let Some(value) = first.selected.as_ref() else {
        return Ok(Generated {
            status: Status::MissingFirstRead,
            first,
            update: None,
            transition: None,
            output: None,
        });
    };
    let mut payload = value.bytes.clone();
    if c == Control::PayloadReversed {
        payload.reverse();
    }
    let accepted: Vec<_> = updates(a, g, m, records, &qs[1], &payload, c)?
        .into_iter()
        .filter(|u| c != Control::ScorerDisabled && a.matches(u.features))
        .collect();
    if accepted.len() != 1 {
        return Ok(Generated {
            status: if accepted.is_empty() {
                Status::NoUpdate
            } else {
                Status::AmbiguousUpdate
            },
            first,
            update: None,
            transition: None,
            output: None,
        });
    }
    let update = accepted.into_iter().next().ok_or(Error::State)?;
    finish(a, g, m, records, &qs[0], first, update, c)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_question_boundary_is_explicit_and_bounded() -> Result<()> {
        assert_eq!(
            questions(b"who did mira help? who did they visit? ")?[1],
            b"who did they visit?".to_vec()
        );
        assert!(questions(b"one?").is_err());
        assert!(questions(b"one?two?three?").is_err());
        assert!(questions(b"one?two?tail").is_err());
        assert!(questions(&[b'?'; 257]).is_err());
        Ok(())
    }
    #[test]
    fn splice_preserves_word_boundary_and_rebinds_geometric_occurrences() -> Result<()> {
        let q = b"who did they trust?";
        let next = splice(q, 8, 12, b"felix")?;
        assert_eq!(next, b"who did felix trust?");
        assert!(splice(q, 9, 12, b"felix").is_err());
        assert!(splice(q, 8, 12, b"").is_err());
        assert!(splice(q, 8, 12, b"two words").is_err());
        let g = BoundGeometry::canonical().map_err(|_| Error::Geometry)?;
        let before = lexical::Query::new(&g, q, ordered::CANONICAL)?;
        let after = lexical::Query::new(&g, &next, ordered::CANONICAL)?;
        assert_ne!(before.geometry.occurrences, after.geometry.occurrences);
        assert_eq!(after.geometry.occurrences.len(), next.len());
        Ok(())
    }
}
