use super::data::Record;
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    dependent_attention::runtime as dependent,
    hamming_refinement::metric::Metric,
    relational_attention::{
        data::Record as ByteRecord,
        runtime::{self as read, Error, Result},
    },
    text_attention::runtime as text,
};
use serde::{Deserialize, Serialize};
pub const ROWS: usize = 2048;
pub const MAX_READS: usize = 4;
pub const MAX_BYTES: usize = 128;
pub const MAX_STEPS: usize = 160;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: text::Artifact,
    pub parent_digest: [u8; 32],
    pub actions: Vec<u8>,
    pub source_digest: [u8; 32],
    pub data_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 2
            || self.actions.len() != ROWS
            || self.actions.iter().any(|&a| a > 2)
            || *blake3::hash(&self.parent.encode()?).as_bytes() != self.parent_digest
        {
            return Err(Error::Artifact);
        }
        self.parent.validate(g)
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| Error::Artifact)
    }
    pub fn decode(bytes: &[u8], g: &BoundGeometry) -> Result<Self> {
        if bytes.len() > 2 * 1024 * 1024 {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(bytes).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    Emit,
    Read,
    Stop,
}
impl Action {
    pub fn from_byte(a: u8) -> Result<Self> {
        match a {
            0 => Ok(Self::Emit),
            1 => Ok(Self::Read),
            2 => Ok(Self::Stop),
            _ => Err(Error::Artifact),
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::Emit => 0,
            Self::Read => 1,
            Self::Stop => 2,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    ReadDisabled,
    FeedbackDisabled,
    UpdateDisabled,
    CursorDisabled,
    BoundaryReadDisabled,
    QueryReversed,
    StopDisabled,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct State<Q = [u8; 2]> {
    pub query: Q,
    pub pending: Q,
    pub cursor: usize,
    pub last: u8,
    pub reads: usize,
    pub emitted: usize,
    pub done: bool,
    pub exhausted: bool,
}
impl<Q: Clone> State<Q> {
    pub fn new(query: Q) -> Self {
        Self {
            query: query.clone(),
            pending: query,
            cursor: 0,
            last: 0,
            reads: 1,
            emitted: 0,
            done: false,
            exhausted: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Observation<Q = [u8; 2]> {
    pub selected: Option<usize>,
    pub byte: u8,
    pub present: bool,
    pub next_query: Q,
    pub next_available: bool,
    pub row: usize,
}
fn route(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Record; 4],
    q: [u8; 2],
    disabled: bool,
) -> Result<read::Generated> {
    let keys = std::array::from_fn(|i| ByteRecord {
        key: records[i].key,
        value: 0,
    });
    read::generate(
        &a.parent.reader,
        g,
        m,
        &keys,
        q,
        if disabled {
            read::Control::ReadDisabled
        } else {
            read::Control::Full
        },
    )
}
pub fn observe(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Record; 4],
    s: &State,
    control: Control,
) -> Result<Observation> {
    let r = route(a, g, m, records, s.query, control == Control::ReadDisabled)?;
    // All four supplied records are scored. A zero-compatibility winner offers
    // no payload; this is a learned compatibility decision, not an absence proof.
    let selected = r.selected.filter(|&i| r.scores[i] != 0);
    let value = selected
        .and_then(|i| records[i].text.get(s.cursor))
        .copied();
    let present = value.is_some();
    let byte = value.unwrap_or(s.last);
    // Emitted symbols have already accumulated in pending. A non-emitting read
    // incorporates the selected current byte through the same learned update.
    let proposed = if present {
        dependent::update_query(&a.parent.parent.parent, s.pending, byte)?
    } else {
        s.pending
    };
    let next_query = if control == Control::UpdateDisabled {
        s.query
    } else {
        proposed
    };
    let next = route(
        a,
        g,
        m,
        records,
        next_query,
        control == Control::ReadDisabled,
    )?;
    let next_available = next.selected.is_some_and(|i| next.scores[i] != 0);
    let context_has_spans = records.iter().any(|r| r.text.len() > 1);
    let row = usize::from(byte)
        + 256 * usize::from(present)
        + 512 * usize::from(context_has_spans)
        + 1024 * usize::from(next_available);
    Ok(Observation {
        selected,
        byte,
        present,
        next_query,
        next_available,
        row,
    })
}
/// Shared transition implementation for training search and actual inference.
/// Legal action masks constrain operation types, not the correct action/answer.
pub fn execute(
    a: &Artifact,
    o: &Observation,
    s: &mut State,
    action: Action,
    control: Control,
) -> Result<Option<u16>> {
    execute_with_update(a, o, s, action, control, |q, byte| {
        dependent::update_query(&a.parent.parent.parent, *q, byte)
    })
}
/// Shared recurrent transition, with a typed learned state update supplied by
/// the caller. Both the retained byte state and ordered geometric state use it.
pub fn execute_with_update<Q: Clone>(
    a: &Artifact,
    o: &Observation<Q>,
    s: &mut State<Q>,
    mut action: Action,
    control: Control,
    update: impl Fn(&Q, u8) -> Result<Q>,
) -> Result<Option<u16>> {
    if control == Control::StopDisabled && action == Action::Stop {
        action = Action::Read;
    }
    match action {
        Action::Emit => {
            if !o.present || s.emitted >= MAX_BYTES {
                s.exhausted = true;
                return Ok(None);
            }
            let token = text::symbol(&a.parent, o.byte, true)?;
            if token == 256 {
                s.done = true;
                return Ok(Some(token));
            }
            let byte = u8::try_from(token).map_err(|_| Error::State)?;
            s.last = byte;
            if control != Control::FeedbackDisabled {
                s.pending = update(&s.pending, byte)?;
            }
            if control != Control::CursorDisabled && a.parent.advance[3] != 0 {
                s.cursor = s.cursor.checked_add(1).ok_or(Error::State)?;
            }
            s.emitted += 1;
            Ok(Some(token))
        }
        Action::Read => {
            if s.reads >= MAX_READS || (control == Control::BoundaryReadDisabled && !o.present) {
                s.exhausted = true;
                return Ok(None);
            }
            s.query = o.next_query.clone();
            s.pending = s.query.clone();
            s.cursor = 0;
            s.last = 0;
            s.reads += 1;
            Ok(None)
        }
        Action::Stop => {
            if o.present {
                s.exhausted = true;
                return Ok(None);
            }
            let token = text::symbol(&a.parent, 0, false)?;
            s.done = token == 256;
            s.exhausted = !s.done;
            Ok(Some(token))
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Step<Q = [u8; 2]> {
    pub before: State<Q>,
    pub observation: Observation<Q>,
    pub action: Action,
    pub token: Option<u16>,
    pub after: State<Q>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Generated<Q = [u8; 2]> {
    pub tokens: Vec<u16>,
    pub steps: Vec<Step<Q>>,
    pub exhausted: bool,
}
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Record; 4],
    query: [u8; 2],
    control: Control,
) -> Result<Generated> {
    a.validate(g)?;
    if records.iter().any(|r| r.text.len() > 32) {
        return Err(Error::Shape);
    }
    let mut s = State::new(if control == Control::QueryReversed {
        [query[1], query[0]]
    } else {
        query
    });
    let mut tokens = Vec::new();
    let mut steps = Vec::new();
    for _ in 0..MAX_STEPS {
        let o = observe(a, g, m, records, &s, control)?;
        let action = Action::from_byte(a.actions[o.row])?;
        let before = s.clone();
        let token = execute(a, &o, &mut s, action, control)?;
        if let Some(t) = token {
            tokens.push(t);
        }
        steps.push(Step {
            before,
            observation: o,
            action,
            token,
            after: s.clone(),
        });
        if s.done || s.exhausted {
            return Ok(Generated {
                tokens,
                steps,
                exhausted: s.exhausted,
            });
        }
    }
    Ok(Generated {
        tokens,
        steps,
        exhausted: true,
    })
}
