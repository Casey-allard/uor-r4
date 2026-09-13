use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    relational_attention::{
        circuit::{self, Circuit},
        data::Record,
        runtime::{self as read, Artifact as Reader, Error, Result},
    },
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub reader: Reader,
    pub reader_digest: [u8; 32],
    pub update: Circuit,
    pub tables: Vec<u8>,
    pub source_digest: [u8; 32],
    pub data_digest: [u8; 32],
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 1
            || self.update.input_count != 24
            || self.update.outputs.len() != 16
            || self.update.nodes.len() != 16
            || self
                .update
                .nodes
                .iter()
                .enumerate()
                .any(|(i, n)| n.inputs != [i, 16 + i % 8])
            || self
                .update
                .outputs
                .iter()
                .enumerate()
                .any(|(i, &wire)| wire != 24 + i)
            || *blake3::hash(&self.reader.encode()?).as_bytes() != self.reader_digest
        {
            return Err(Error::Artifact);
        }
        self.reader.validate(g)?;
        circuit::hard(&self.update, &self.tables, &[false; 24])?;
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| Error::Artifact)
    }
    pub fn decode(bytes: &[u8], g: &BoundGeometry) -> Result<Self> {
        if bytes.len() > 524288 {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(bytes).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
}
pub fn update_input(query: [u8; 2], payload: u8) -> [bool; 24] {
    std::array::from_fn(|i| {
        if i < 16 {
            query[i / 8] & (1 << (i % 8)) != 0
        } else {
            payload & (1 << (i - 16)) != 0
        }
    })
}
pub fn update_query(a: &Artifact, query: [u8; 2], payload: u8) -> Result<[u8; 2]> {
    let t = circuit::hard(&a.update, &a.tables, &update_input(query, payload))?;
    if t.outputs.len() != 16 {
        return Err(Error::Shape);
    }
    let mut q = [0u8; 2];
    for (i, &v) in t.outputs.iter().enumerate() {
        q[i / 8] |= u8::from(v) << (i % 8);
    }
    Ok(q)
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    UpdateDisabled,
    PayloadDisabled,
    PriorQueryDisabled,
    ReadDisabled,
    SecondReadDisabled,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Generated {
    pub first: read::Generated,
    pub next_query: [u8; 2],
    pub second: read::Generated,
}
/// No answer, expected next key, example id, or family enters prediction.
/// The same learned reader is used twice; every downstream read is recomputed.
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Record; 4],
    query: [u8; 2],
    control: Control,
) -> Result<Generated> {
    a.validate(g)?;
    let first = read::generate(
        &a.reader,
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
    let payload = if control == Control::PayloadDisabled {
        0
    } else {
        first.selected.map_or(0, |i| records[i].value)
    };
    let next_query = if control == Control::UpdateDisabled {
        query
    } else {
        update_query(
            a,
            if control == Control::PriorQueryDisabled {
                [0; 2]
            } else {
                query
            },
            payload,
        )?
    };
    let second = read::generate(
        &a.reader,
        g,
        m,
        records,
        next_query,
        if matches!(control, Control::ReadDisabled | Control::SecondReadDisabled) {
            read::Control::ReadDisabled
        } else {
            read::Control::Full
        },
    )?;
    Ok(Generated {
        first,
        next_query,
        second,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_geometric::relational_attention::circuit::Node;
    #[test]
    fn learned_update_has_both_dependencies_and_order() -> Result<()> {
        let c = Circuit {
            input_count: 24,
            nodes: (0..16)
                .map(|i| Node {
                    inputs: [i, 16 + i % 8],
                })
                .collect(),
            outputs: (24..40).collect(),
        };
        let tables = vec![circuit::XOR; 16];
        let input = update_input([0x13, 0xa9], 0x65);
        let t = circuit::hard(&c, &tables, &input)?;
        let out: Vec<bool> = update_input([0x76, 0xcc], 0)[..16].to_vec();
        assert_eq!(t.outputs, out);
        for i in 0..24 {
            let mut changed = input;
            changed[i] = !changed[i];
            assert_ne!(circuit::hard(&c, &tables, &changed)?.outputs, t.outputs);
        }
        Ok(())
    }
}
