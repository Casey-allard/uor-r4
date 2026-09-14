//! Exported Boolean relation comparator and learned byte/EOS decoder.
//! No score contraction, floating point or external model executes in prediction.
use super::{
    circuit::{self, Circuit},
    data::Record,
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
};
use serde::{Deserialize, Serialize};
pub const FEATURES: usize = 288;
#[derive(Debug)]
pub enum Error {
    Geometry,
    Circuit(circuit::Error),
    Artifact,
    State,
    Shape,
    CorrespondenceLimit { limit: usize },
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<circuit::Error> for Error {
    fn from(e: circuit::Error) -> Self {
        Self::Circuit(e)
    }
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub schema: u16,
    pub geometry: [u8; 32],
    pub data_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub scorer: Circuit,
    pub score_tables: Vec<u8>,
    pub decoder: Circuit,
    pub decode_tables: Vec<u8>,
    pub training: String,
}
impl Artifact {
    pub fn validate(&self, g: &BoundGeometry) -> Result<()> {
        if self.schema != 1
            || self.geometry != g.id()
            || self.scorer.input_count != FEATURES
            || self.scorer.outputs.len() != 1
            || self.decoder.input_count != 9
            || self.decoder.outputs.len() != 9
            || self.scorer.nodes.len() > 4096
            || self.decoder.nodes.len() > 256
        {
            return Err(Error::Artifact);
        }
        circuit::hard(&self.scorer, &self.score_tables, &[false; FEATURES])?;
        circuit::hard(&self.decoder, &self.decode_tables, &[false; 9])?;
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|_| Error::Artifact)
    }
    pub fn decode(bytes: &[u8], g: &BoundGeometry) -> Result<Self> {
        if bytes.len() > 262144 {
            return Err(Error::Artifact);
        }
        let a: Self = serde_json::from_slice(bytes).map_err(|_| Error::Artifact)?;
        a.validate(g)?;
        Ok(a)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Relation {
    pub roots: [[u16; 4]; 2],
    pub directed: [[u16; 4]; 4],
    pub hamming: [[u16; 4]; 4],
    pub phase_difference: [u16; 4],
    pub ordered_prime_identity: [[u32; 2]; 2],
}
fn roots(g: &BoundGeometry, k: [u8; 2]) -> Result<[u16; 4]> {
    let a = g.byte_leaf(k[0]);
    let b = g.byte_leaf(k[1]);
    Ok([
        a,
        b,
        g.product(a, b).map_err(|_| Error::Geometry)?,
        g.product(b, a).map_err(|_| Error::Geometry)?,
    ])
}
pub fn relation(g: &BoundGeometry, m: &Metric, q: [u8; 2], k: [u8; 2]) -> Result<Relation> {
    if m.geometry_digest() != g.id() {
        return Err(Error::Geometry);
    }
    let a = roots(g, q)?;
    let b = roots(g, k)?;
    let mut directed = [[0; 4]; 4];
    let mut hamming = [[0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            directed[i][j] = g
                .product(g.inverse(a[i]).map_err(|_| Error::Geometry)?, b[j])
                .map_err(|_| Error::Geometry)?;
            hamming[i][j] = m.root_distance(a[i], b[j]).map_err(|_| Error::Geometry)?;
        }
    }
    // Ordered phase differences are separate; commutative sums cannot encode order.
    let qa = g.byte_phases(q[0]);
    let qb = g.byte_phases(q[1]);
    let ka = g.byte_phases(k[0]);
    let kb = g.byte_phases(k[1]);
    Ok(Relation {
        roots: [a, b],
        directed,
        hamming,
        phase_difference: [
            ka[0].wrapping_sub(qa[0]),
            kb[0].wrapping_sub(qb[0]),
            ka[1].wrapping_sub(qa[1]),
            kb[1].wrapping_sub(qb[1]),
        ],
        ordered_prime_identity: [
            [g.byte_prime(q[0]), g.byte_prime(q[1])],
            [g.byte_prime(k[0]), g.byte_prime(k[1])],
        ],
    })
}
pub fn features(r: &Relation) -> Vec<bool> {
    let mut x = Vec::with_capacity(FEATURES);
    for row in r.directed.iter().chain(&r.hamming) {
        for value in row {
            for bit in 0..7 {
                x.push(value & (1 << bit) != 0)
            }
        }
    }
    for value in r.phase_difference {
        for bit in 0..16 {
            x.push(value & (1 << bit) != 0)
        }
    }
    x
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    ReadDisabled,
    QueryReversed,
    PhasesDisabled,
    DiagonalOnly,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Generated {
    pub tokens: Vec<u16>,
    pub selected: Option<usize>,
    pub scores: [u8; 4],
    pub admitted: usize,
    pub relations: Vec<Relation>,
}
pub fn decoder_input(byte: u8, observed: bool) -> [bool; 9] {
    std::array::from_fn(|i| {
        if i == 8 {
            observed
        } else {
            byte & (1 << i) != 0
        }
    })
}
pub fn decode_symbol(a: &Artifact, byte: u8, observed: bool) -> Result<u16> {
    let t = circuit::hard(&a.decoder, &a.decode_tables, &decoder_input(byte, observed))?;
    if t.outputs[8] {
        return Ok(256);
    }
    let mut b = 0u16;
    for (i, &bit) in t.outputs[..8].iter().enumerate() {
        b |= u16::from(bit) << i;
    }
    Ok(b)
}
/// Exactly two generation steps; EOS on step one or absence on step two is a failure.
/// Observed-output state changes only after the actual first output. No target enters.
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    records: &[Record; 4],
    query: [u8; 2],
    control: Control,
) -> Result<Generated> {
    a.validate(g)?;
    let mut scores = [0; 4];
    let mut relations = Vec::new();
    let q = if control == Control::QueryReversed {
        [query[1], query[0]]
    } else {
        query
    };
    for (i, record) in records.iter().enumerate() {
        let mut r = relation(g, m, q, record.key)?;
        if control == Control::PhasesDisabled {
            r.phase_difference = [0; 4];
        }
        if control == Control::DiagonalOnly {
            for u in 0..4 {
                for v in 0..4 {
                    if u != v {
                        r.directed[u][v] = g.identity();
                        r.hamming[u][v] = 0;
                    }
                }
            }
        }
        scores[i] = u8::from(circuit::hard(&a.scorer, &a.score_tables, &features(&r))?.outputs[0]);
        relations.push(r);
    }
    let selected = if control == Control::ReadDisabled {
        None
    } else {
        let mut best = 0;
        for i in 1..4 {
            if scores[i] > scores[best] {
                best = i;
            }
        }
        Some(best)
    };
    let payload = selected.map_or(0, |i| records[i].value);
    let mut tokens = Vec::with_capacity(2);
    for step in 0..2 {
        let token = decode_symbol(a, payload, step != 0)?;
        tokens.push(token);
        if token == 256 {
            break;
        }
    }
    Ok(Generated {
        tokens,
        selected,
        scores,
        admitted: 4,
        relations,
    })
}
