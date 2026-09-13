use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    dependent_attention::runtime as dependent,
    hamming_refinement::metric::Metric,
    relational_attention::{
        data::Record,
        runtime::{self as read, Error, Result},
    },
};
use serde::{Deserialize, Serialize};
pub const MAX_READS: usize = 4;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: dependent::Artifact,
    pub parent_digest: [u8; 32],
    pub emit: Vec<u8>,
    pub data_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 1
            || self.emit.len() != 256
            || self.emit.iter().any(|&v| v > 1)
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
        if b.len() > 1048576 {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(b).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    AlwaysRead,
    AlwaysEmit,
    FixedTwo,
    UpdateDisabled,
    PayloadDisabled,
    ReadDisabled,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Step {
    pub query: [u8; 2],
    pub selected: Option<usize>,
    pub payload: u8,
    pub emit: bool,
    pub scores: [u8; 4],
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Generated {
    pub tokens: Vec<u16>,
    pub steps: Vec<Step>,
    pub exhausted: bool,
}
/// No target, depth, terminal marker, type annotation or expected stopping index.
/// MAX_READS is a resource bound: reaching it without learned emission is failure.
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Record; 4],
    initial_query: [u8; 2],
    control: Control,
) -> Result<Generated> {
    a.validate(g)?;
    let mut query = initial_query;
    let mut steps = Vec::with_capacity(MAX_READS);
    for hop in 0..MAX_READS {
        let selected = read::generate(
            &a.parent.reader,
            g,
            m,
            records,
            query,
            if control == Control::ReadDisabled {
                read::Control::ReadDisabled
            } else {
                read::Control::Full
            },
        )?;
        let payload = selected.selected.map_or(0, |i| records[i].value);
        let emit = match control {
            Control::AlwaysRead => false,
            Control::AlwaysEmit => true,
            Control::FixedTwo => hop == 1,
            _ => a.emit[usize::from(payload)] != 0,
        };
        steps.push(Step {
            query,
            selected: selected.selected,
            payload,
            emit,
            scores: selected.scores,
        });
        if emit {
            return Ok(Generated {
                tokens: selected.tokens,
                steps,
                exhausted: false,
            });
        }
        if control != Control::UpdateDisabled {
            query = dependent::update_query(
                &a.parent,
                query,
                if control == Control::PayloadDisabled {
                    0
                } else {
                    payload
                },
            )?;
        }
    }
    Ok(Generated {
        tokens: Vec::new(),
        steps,
        exhausted: true,
    })
}
