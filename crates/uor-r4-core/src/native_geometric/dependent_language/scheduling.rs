//! A learned shared action policy over raw-language reads and the retained
//! geometric transition. Punctuation supplies clauses, never a gold read count.
use super::runtime as binding;
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
pub const ROWS: usize = 8;
pub const MAX_STEPS: usize = 96;
pub const FEATURES: [&str; 3] = [
    "current_byte_present",
    "cursor_at_boundary",
    "usable_continuation",
];
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: binding::Artifact,
    pub parent_digest: [u8; 32],
    pub actions: Vec<u8>,
    pub source_digest: [u8; 32],
    pub data_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 1
            || self.actions.len() != ROWS
            || self.actions.iter().any(|&x| x > 2)
            || self.parent_digest != *blake3::hash(&self.parent.encode()?).as_bytes()
        {
            return Err(Error::Artifact);
        }
        self.parent.validate(g)
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| Error::Artifact)
    }
    pub fn decode(bytes: &[u8], g: &BoundGeometry) -> Result<Self> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(bytes).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Control {
    Full,
    PolicyDisabled,
    ReadDisabled,
    ContinuationDisabled,
    AlwaysRead,
    EmitInsteadOfRead,
    UpdateDisabled,
    PayloadReversed,
    ScorerDisabled,
    CursorDisabled,
    StopDisabled,
    ExactIdentity,
    FeedbackDisabled,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub core: State<lexical::Query>,
    pub clause: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observed {
    pub core: Observation<lexical::Query>,
    pub route: lexical::Route,
    pub update: Option<binding::UpdateCandidate>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub before: Frame,
    pub observation: Observed,
    pub action: Action,
    pub token: Option<u16>,
    pub after: Frame,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generated {
    pub tokens: Vec<u16>,
    pub steps: Vec<Step>,
    pub exhausted: bool,
}
/// Fixed lexical protocol for this experiment. It accepts one or two question
/// clauses; clause count does not select the action policy or desired output.
pub fn clauses(prompt: &[u8]) -> Result<Vec<Vec<u8>>> {
    if prompt.len() > 256 {
        return Err(Error::Shape);
    }
    let mut out = Vec::new();
    let mut start = 0;
    for (i, &byte) in prompt.iter().enumerate() {
        if byte == b'?' {
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
    if out.is_empty() || out.len() > 2 || prompt[start..].iter().any(|b| !b.is_ascii_whitespace()) {
        return Err(Error::Shape);
    }
    Ok(out)
}
pub fn start(a: &Artifact, g: &BoundGeometry, qs: &[Vec<u8>]) -> Result<Frame> {
    for q in qs {
        let words = reader::words(g, q, ordered::CANONICAL)?;
        if words.is_empty() || words.len() > reader::MAX_WORDS {
            return Err(Error::Shape);
        }
    }
    let q = qs.first().ok_or(Error::Shape)?;
    Ok(Frame {
        core: State::new(lexical::Query::new(
            g,
            q,
            a.parent.parent.parent.parent.operators,
        )?),
        clause: 0,
    })
}
fn read_control(c: Control) -> reader::Control {
    match c {
        Control::ReadDisabled => reader::Control::ReadDisabled,
        Control::ExactIdentity => reader::Control::ExactIdentity,
        _ => reader::Control::Full,
    }
}
pub fn observe(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Vec<u8>; 4],
    qs: &[Vec<u8>],
    s: &Frame,
    c: Control,
) -> Result<Observed> {
    let route = reader::route(
        &a.parent.parent,
        g,
        m,
        records,
        &s.core.query.bytes,
        read_control(c),
    )?;
    let value = route
        .selected
        .as_ref()
        .and_then(|x| x.bytes.get(s.core.cursor))
        .copied();
    let mut update = None;
    if let (Some(next), Some(selected)) = (qs.get(s.clause + 1), route.selected.as_ref()) {
        let mut bytes = selected.bytes.clone();
        if c == Control::PayloadReversed {
            bytes.reverse();
        }
        let updates = binding::updates(
            &a.parent,
            g,
            m,
            records,
            next,
            &bytes,
            if c == Control::ExactIdentity {
                binding::Control::ExactIdentity
            } else {
                binding::Control::Full
            },
        )?;
        let mut admitted = updates
            .into_iter()
            .filter(|u| c != Control::ScorerDisabled && a.parent.matches(u.features));
        let first = admitted.next();
        if admitted.next().is_none() {
            update = first;
        }
    }
    let next_query = if let Some(u) = &update {
        lexical::Query::new(g, &u.question, a.parent.parent.parent.parent.operators)?
    } else {
        s.core.query.clone()
    };
    let next_available = update.is_some()
        && reader::route(
            &a.parent.parent,
            g,
            m,
            records,
            &next_query.bytes,
            read_control(c),
        )?
        .selected
        .is_some();
    let row = usize::from(value.is_some())
        | (usize::from(s.core.cursor == 0) << 1)
        | (usize::from(next_available && c != Control::ContinuationDisabled) << 2);
    Ok(Observed {
        core: Observation {
            selected: route.selected.as_ref().map(|x| x.source),
            byte: value.unwrap_or(s.core.last),
            present: value.is_some(),
            next_query,
            next_available,
            row,
        },
        route,
        update,
    })
}
pub fn execute(
    a: &Artifact,
    g: &BoundGeometry,
    o: &Observed,
    s: &mut Frame,
    action: Action,
    c: Control,
) -> Result<Option<u16>> {
    // Operation legality is separate from selection. No continuation is inferred
    // from a rejected route; unsupported input fails instead of proving absence.
    if o.route.selected.is_none()
        || (action == Action::Read && (!o.core.next_available || o.update.is_none()))
    {
        s.core.exhausted = true;
        return Ok(None);
    }
    let mut observation = o.core.clone();
    if action == Action::Emit && s.core.pending.bytes.len() >= ordered::MAX_PREFIX {
        s.core.exhausted = true;
        return Ok(None);
    }
    if c == Control::UpdateDisabled && action == Action::Read {
        observation.next_query = s.core.query.clone();
    }
    let rc = match c {
        Control::CursorDisabled => recurrent::Control::CursorDisabled,
        Control::StopDisabled => recurrent::Control::StopDisabled,
        Control::FeedbackDisabled => recurrent::Control::FeedbackDisabled,
        _ => recurrent::Control::Full,
    };
    let token = recurrent::execute_with_update(
        &a.parent.recurrent(),
        &observation,
        &mut s.core,
        action,
        rc,
        |q, b| q.push(g, b, a.parent.parent.parent.parent.operators),
    )?;
    if action == Action::Read && !s.core.exhausted {
        s.clause = s.clause.checked_add(1).ok_or(Error::State)?;
    }
    Ok(token)
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
    let qs = clauses(prompt)?;
    let mut state = start(a, g, &qs)?;
    let mut out = Generated {
        tokens: Vec::new(),
        steps: Vec::new(),
        exhausted: false,
    };
    for _ in 0..MAX_STEPS {
        let o = observe(a, g, m, records, &qs, &state, c)?;
        let mut action = Action::from_byte(a.actions[o.core.row])?;
        match c {
            Control::PolicyDisabled => action = Action::Stop,
            Control::AlwaysRead => action = Action::Read,
            Control::EmitInsteadOfRead if action == Action::Read => action = Action::Emit,
            _ => {}
        }
        let before = state.clone();
        let token = execute(a, g, &o, &mut state, action, c)?;
        if let Some(t) = token {
            out.tokens.push(t);
        }
        out.steps.push(Step {
            before,
            observation: o,
            action,
            token,
            after: state.clone(),
        });
        if state.core.done || state.core.exhausted {
            out.exhausted = state.core.exhausted;
            return Ok(out);
        }
    }
    out.exhausted = true;
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_question_protocol_has_no_target_depth_argument() -> Result<()> {
        assert_eq!(clauses(b"who did ruby guide?")?.len(), 1);
        assert_eq!(
            clauses(b"who did ruby guide? who did they trust?")?.len(),
            2
        );
        for bad in [b"none".as_slice(), b"?", b"a?b?c?", b"a?tail"] {
            assert!(clauses(bad).is_err());
        }
        Ok(())
    }
}
