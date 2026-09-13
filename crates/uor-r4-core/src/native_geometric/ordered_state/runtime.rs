use super::data::Record;
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    recurrent_text::runtime::{self as recurrent, Action, Observation, State},
    relational_attention::runtime::{Error, Result},
};
use serde::{Deserialize, Serialize};
pub const MAX_PREFIX: usize = 128;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operator {
    Hold,
    Right,
    Left,
    Replace,
}
pub const OPERATORS: [Operator; 4] = [
    Operator::Hold,
    Operator::Right,
    Operator::Left,
    Operator::Replace,
];
/// Signed prefix products at each exact occurrence position. A final product
/// alone is many-to-one; prefix history is retained rather than inverted into
/// invented byte coordinates. These channels are not a paired-H4/icosian lift.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Query {
    pub prefixes: Vec<[u16; 2]>,
    /// Exact canonical byte-token identity at each occurrence, kept separate
    /// from geometric distance and never treated as a semantic metric.
    pub occurrences: Vec<u32>,
}
impl Query {
    pub fn empty() -> Self {
        Self {
            prefixes: Vec::new(),
            occurrences: Vec::new(),
        }
    }
    pub fn push(&self, g: &BoundGeometry, byte: u8, ops: [Operator; 2]) -> Result<Self> {
        if self.prefixes.len() >= MAX_PREFIX || self.prefixes.len() != self.occurrences.len() {
            return Err(Error::Shape);
        }
        let old = self.prefixes.last().copied().unwrap_or([g.identity(); 2]);
        let leaf = g.byte_leaf(byte);
        let mut next = old;
        for i in 0..2 {
            next[i] = match ops[i] {
                Operator::Hold => old[i],
                Operator::Right => g.product(old[i], leaf).map_err(|_| Error::Geometry)?,
                Operator::Left => g.product(leaf, old[i]).map_err(|_| Error::Geometry)?,
                Operator::Replace => leaf,
            };
        }
        let mut q = self.clone();
        q.prefixes.push(next);
        q.occurrences.push(g.byte_prime(byte));
        Ok(q)
    }
    pub fn encode(g: &BoundGeometry, bytes: &[u8], ops: [Operator; 2]) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > MAX_PREFIX {
            return Err(Error::Shape);
        }
        let mut q = Self::empty();
        for &b in bytes {
            q = q.push(g, b, ops)?;
        }
        Ok(q)
    }
}
pub const CANONICAL: [Operator; 2] = [Operator::Right, Operator::Left];
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: recurrent::Artifact,
    pub parent_digest: [u8; 32],
    pub geometry_digest: [u8; 32],
    pub operators: [Operator; 2],
    pub source_digest: [u8; 32],
    pub data_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 1
            || self.geometry_digest != g.id()
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
        if bytes.len() > 3 * 1024 * 1024 {
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
    ReadDisabled,
    FeedbackDisabled,
    UpdateDisabled,
    CursorDisabled,
    BoundaryReadDisabled,
    QueryReversed,
    StopDisabled,
    StateDisabled,
    FinalRootOnly,
}
impl Control {
    fn recurrent(self) -> recurrent::Control {
        match self {
            Self::ReadDisabled => recurrent::Control::ReadDisabled,
            Self::FeedbackDisabled => recurrent::Control::FeedbackDisabled,
            Self::UpdateDisabled => recurrent::Control::UpdateDisabled,
            Self::CursorDisabled => recurrent::Control::CursorDisabled,
            Self::BoundaryReadDisabled => recurrent::Control::BoundaryReadDisabled,
            Self::QueryReversed => recurrent::Control::QueryReversed,
            Self::StopDisabled => recurrent::Control::StopDisabled,
            _ => recurrent::Control::Full,
        }
    }
}
/// All supplied candidates are considered. Zero distance offers a compatible
/// record; ties offer none. This is exact structural compatibility, not semantic
/// absence or a learned relevance kernel. Hamming is over bound hemisphere bits.
pub fn distance(m: &Metric, q: &Query, k: &Query, final_only: bool) -> Result<u16> {
    if q.prefixes.len() > MAX_PREFIX
        || k.prefixes.len() > MAX_PREFIX
        || q.prefixes.len() != q.occurrences.len()
        || k.prefixes.len() != k.occurrences.len()
        || q.prefixes
            .iter()
            .chain(&k.prefixes)
            .flatten()
            .any(|&r| r >= 120)
    {
        return Err(Error::Shape);
    }
    if final_only {
        return match (q.prefixes.last(), k.prefixes.last()) {
            (Some(&q), Some(&k)) => m.distance(q, k).map_err(|_| Error::State),
            _ => Err(Error::State),
        };
    }
    let mut d = 0u16;
    for (q, k) in q.prefixes.iter().zip(&k.prefixes) {
        d += m.distance(*q, *k).map_err(|_| Error::State)?;
    }
    // Maximum 128 * 240, representable in u16; integer addition only.
    for _ in 0..q.prefixes.len().abs_diff(k.prefixes.len()) {
        d += 240;
    }
    Ok(d)
}
fn route(m: &Metric, keys: &[Query; 4], q: &Query, c: Control) -> Result<Option<usize>> {
    if c == Control::ReadDisabled {
        return Ok(None);
    }
    let mut selected = None;
    for (i, k) in keys.iter().enumerate() {
        if distance(m, q, k, c == Control::FinalRootOnly)? == 0 {
            if selected.is_some() {
                return Ok(None);
            }
            selected = Some(i);
        }
    }
    Ok(selected)
}
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Record; 4],
    query: &[u8],
    c: Control,
) -> Result<recurrent::Generated<Query>> {
    a.validate(g)?;
    if m.geometry_digest() != g.id() || records.iter().any(|r| r.text.len() > 32) {
        return Err(Error::Shape);
    }
    let keys: Vec<_> = records
        .iter()
        .map(|r| Query::encode(g, &r.key, CANONICAL))
        .collect::<Result<_>>()?;
    let keys: [Query; 4] = keys.try_into().map_err(|_| Error::Shape)?;
    let ops = if c == Control::StateDisabled {
        [Operator::Hold; 2]
    } else {
        a.operators
    };
    let mut input = query.to_vec();
    if c == Control::QueryReversed {
        input.reverse();
    }
    let mut s = State::new(Query::encode(g, &input, ops)?);
    let mut tokens = Vec::new();
    let mut steps = Vec::new();
    for _ in 0..recurrent::MAX_STEPS {
        let selected = route(m, &keys, &s.query, c)?;
        let value = selected
            .and_then(|i| records[i].text.get(s.cursor))
            .copied();
        let present = value.is_some();
        if present && s.pending.prefixes.len() >= MAX_PREFIX {
            return Ok(recurrent::Generated {
                tokens,
                steps,
                exhausted: true,
            });
        }
        let byte = value.unwrap_or(s.last);
        let next_query = if c == Control::UpdateDisabled {
            s.query.clone()
        } else if present {
            s.pending.push(g, byte, ops)?
        } else {
            s.pending.clone()
        };
        let next_available = route(m, &keys, &next_query, c)?.is_some();
        let row = usize::from(byte)
            + 256 * usize::from(present)
            + 512 * usize::from(records.iter().any(|r| r.text.len() > 1))
            + 1024 * usize::from(next_available);
        let o = Observation {
            selected,
            byte,
            present,
            next_query,
            next_available,
            row,
        };
        let action = Action::from_byte(a.parent.actions[row])?;
        let before = s.clone();
        let token = recurrent::execute_with_update(
            &a.parent,
            &o,
            &mut s,
            action,
            c.recurrent(),
            |q, b| q.push(g, b, ops),
        )?;
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
        if s.done || s.exhausted {
            return Ok(recurrent::Generated {
                tokens,
                steps,
                exhausted: s.exhausted,
            });
        }
    }
    Ok(recurrent::Generated {
        tokens,
        steps,
        exhausted: true,
    })
}
