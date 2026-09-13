use super::data::Record;
use crate::native_geometric::{
    adaptive_attention::runtime as prior,
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    relational_attention::{
        circuit::{self, Circuit},
        data::Record as KeyRecord,
        runtime::{self as read, Error, Result},
    },
};
use serde::{Deserialize, Serialize};
pub const MAX_TEXT: usize = 32;
pub const MAX_SYMBOLS: usize = 33;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub parent: prior::Artifact,
    pub parent_digest: [u8; 32],
    pub reader: read::Artifact,
    pub writer: Circuit,
    pub tables: Vec<u8>,
    pub advance: [u8; 4],
    pub data_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 1
            || self.writer.input_count != 9
            || self.writer.nodes.len() != 9
            || self.writer.outputs != (9..18).collect::<Vec<_>>()
            || self
                .writer
                .nodes
                .iter()
                .enumerate()
                .any(|(i, n)| n.inputs != [i, 8])
            || self.advance.iter().any(|&v| v > 1)
            || *blake3::hash(&self.parent.encode()?).as_bytes() != self.parent_digest
        {
            return Err(Error::Artifact);
        }
        self.parent.validate(g)?;
        self.reader.validate(g)?;
        circuit::hard(&self.writer, &self.tables, &[false; 9])?;
        Ok(())
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
    ReadDisabled,
    FeedbackDisabled,
    PayloadDisabled,
    QueryReversed,
    EosDisabled,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Step {
    pub cursor: usize,
    pub selected: Option<usize>,
    pub present: bool,
    pub token: u16,
    pub advanced: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Generated {
    pub tokens: Vec<u16>,
    pub steps: Vec<Step>,
    pub exhausted: bool,
}
pub fn input(byte: u8, present: bool) -> [bool; 9] {
    std::array::from_fn(|i| {
        if i == 8 {
            present
        } else {
            byte & (1 << i) != 0
        }
    })
}
pub fn symbol(a: &Artifact, byte: u8, present: bool) -> Result<u16> {
    let t = circuit::hard(&a.writer, &a.tables, &input(byte, present))?;
    if t.outputs.len() != 9 {
        return Err(Error::Shape);
    }
    if t.outputs[8] {
        return Ok(256);
    }
    let mut value = 0;
    for (i, &v) in t.outputs[..8].iter().enumerate() {
        value |= u16::from(v) << i;
    }
    Ok(value)
}
pub fn select(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Record; 4],
    query: [u8; 2],
    disabled: bool,
) -> Result<Option<usize>> {
    let keys = std::array::from_fn(|i| KeyRecord {
        key: records[i].key,
        value: 0,
    });
    Ok(read::generate(
        &a.reader,
        g,
        m,
        &keys,
        query,
        if disabled {
            read::Control::ReadDisabled
        } else {
            read::Control::Full
        },
    )?
    .selected)
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
    if records.iter().any(|r| r.text.len() > MAX_TEXT) {
        return Err(Error::Shape);
    }
    let q = if control == Control::QueryReversed {
        [query[1], query[0]]
    } else {
        query
    };
    let mut cursor = 0;
    let mut tokens = Vec::new();
    let mut steps = Vec::new();
    for _ in 0..MAX_SYMBOLS {
        let selected = select(a, g, m, records, q, control == Control::ReadDisabled)?;
        let value = selected.and_then(|i| records[i].text.get(cursor)).copied();
        let present = value.is_some();
        let mut token = symbol(
            a,
            if control == Control::PayloadDisabled {
                0
            } else {
                value.unwrap_or(0)
            },
            present,
        )?;
        if control == Control::EosDisabled && token == 256 {
            token = 0;
        }
        let row = usize::from(present) + 2 * usize::from(token != 256);
        let advanced = token != 256 && control != Control::FeedbackDisabled && a.advance[row] != 0;
        steps.push(Step {
            cursor,
            selected,
            present,
            token,
            advanced,
        });
        tokens.push(token);
        if token == 256 {
            return Ok(Generated {
                tokens,
                steps,
                exhausted: false,
            });
        }
        if advanced {
            cursor += 1;
        }
    }
    Ok(Generated {
        tokens,
        steps,
        exhausted: true,
    })
}
