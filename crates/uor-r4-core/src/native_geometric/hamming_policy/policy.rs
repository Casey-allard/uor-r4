//! Shared hard Boolean policy. Integer score bits are assembled, never contracted.
use crate::native_geometric::{
    addressed_attention::{
        artifact::BoundGeometry,
        objects::{ActiveLease, Lease},
    },
    hamming_refinement::runtime,
};
use serde::{Deserialize, Serialize};

pub const INPUT_BITS: usize = 4096;
pub const WIDTHS: [usize; 3] = [1024, 256, 1796];
pub const OUTPUT_BITS: usize = 1796;
pub const GATES: usize = 3076;
pub const CELLS: usize = GATES << 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Shape,
    Domain,
    Geometry,
    EmptyLegalSet,
    AuditOverflow,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

fn random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Params {
    pub seed: u64,
    pub tables: Vec<u16>,
}
impl Params {
    pub fn seeded(seed: u64) -> Self {
        let mut state = seed ^ 0x48414d4d494e4731;
        Self {
            seed,
            tables: (0..GATES).map(|_| random(&mut state) as u16).collect(),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.tables.len() != GATES {
            return Err(Error::Shape);
        }
        Ok(())
    }
    pub fn digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"uor-hamming-shared-policy-parameters/1");
        h.update(&self.seed.to_le_bytes());
        h.update(&(self.tables.len() as u64).to_le_bytes());
        for table in &self.tables {
            h.update(&table.to_le_bytes());
        }
        *h.finalize().as_bytes()
    }
    pub fn flip(&mut self, gate: usize, row: u8) -> Result<()> {
        self.validate()?;
        if row >= 16 {
            return Err(Error::Domain);
        }
        *self.tables.get_mut(gate).ok_or(Error::Domain)? ^= 1u16 << row;
        Ok(())
    }
}

/// Seeded partition wiring covers every input through the first two layers.
/// Each 64 consecutive output gates covers the entire second layer, including
/// within the 480 geometric output bits. Coverage is structural, not learned use.
#[derive(Clone, Debug)]
pub struct Topology {
    pub wires: Vec<[u16; 4]>,
}
impl Topology {
    pub fn seeded(seed: u64) -> Self {
        let mut state = seed ^ 0x746f706f6c6f6779;
        let mut wires = Vec::with_capacity(GATES);
        let mut previous = INPUT_BITS;
        for width in WIDTHS {
            let mut permutation: Vec<_> = (0..previous as u16).collect();
            for i in (1..previous).rev() {
                let j = (random(&mut state) % (i as u64 + 1)) as usize;
                permutation.swap(i, j);
            }
            for gate in 0..width {
                wires.push(std::array::from_fn(|pin| {
                    permutation[((gate << 2) + pin) % previous]
                }));
            }
            previous = width;
        }
        Self { wires }
    }
    pub fn validate(&self) -> Result<()> {
        if self.wires.len() != GATES {
            return Err(Error::Shape);
        }
        let mut start = 0;
        let mut previous = INPUT_BITS;
        for width in WIDTHS {
            let mut seen = [false; INPUT_BITS];
            for pins in &self.wires[start..start + width] {
                for (n, &p) in pins.iter().enumerate() {
                    if usize::from(p) >= previous || pins[..n].contains(&p) {
                        return Err(Error::Domain);
                    }
                    seen[usize::from(p)] = true;
                }
            }
            if !seen[..previous].iter().all(|x| *x) {
                return Err(Error::Domain);
            }
            start += width;
            previous = width;
        }
        let mut geometric_seen = [false; 256];
        for pins in &self.wires[1280..1760] {
            for &pin in pins {
                geometric_seen[usize::from(pin)] = true;
            }
        }
        if !geometric_seen.iter().all(|x| *x) {
            return Err(Error::Domain);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Query,
    Extent,
    Delta,
    Control,
    Emit,
    Observe,
    Key,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Intervention {
    Full,
    ReadDisabled,
    UpdateDisabled,
    FirstRootOnly,
    PayloadMasked,
}
#[derive(Clone, Copy, Debug)]
pub struct Fields {
    pub initial_roots: [u16; 4],
    pub roots: [u16; 4],
    pub selected: Option<Lease>,
    pub previous: Option<Lease>,
    pub active: Option<ActiveLease>,
    pub last_observed: Option<u8>,
    pub zeta_bins: [u8; 4],
    pub position: u8,
    pub round: u8,
    pub numeric_valid: [bool; 2],
}
#[derive(Clone, Copy, Debug)]
pub struct Environment {
    pub last_observed: Option<u8>,
    pub zeta_bins: [u8; 4],
    pub position: u8,
    pub initial_roots: [u16; 4],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub roots: [u16; 4],
    pub extent_scores: [u8; 64],
    pub control_scores: [u8; 7],
    pub emission_scores: [u8; 257],
}
impl Decision {
    pub fn extent(&self, legal: &[bool; 64]) -> Result<u8> {
        Ok(argmax(&self.extent_scores, legal)? as u8 + 1)
    }
    pub fn control(&self, legal: &[bool; 7]) -> Result<u8> {
        Ok(argmax(&self.control_scores, legal)? as u8)
    }
    pub fn emission(&self) -> u16 {
        let mut winner = 0;
        for n in 1..257 {
            if self.emission_scores[n] > self.emission_scores[winner] {
                winner = n;
            }
        }
        winner as u16
    }
}
fn argmax<const N: usize>(scores: &[u8; N], legal: &[bool; N]) -> Result<usize> {
    let mut winner = None;
    for n in 0..N {
        if legal[n] && winner.is_none_or(|old| scores[n] > scores[old]) {
            winner = Some(n);
        }
    }
    winner.ok_or(Error::EmptyLegalSet)
}

struct Packer {
    bits: [bool; INPUT_BITS],
    pos: usize,
}
impl Packer {
    fn bit(&mut self, bit: bool) -> Result<()> {
        *self.bits.get_mut(self.pos).ok_or(Error::Shape)? = bit;
        self.pos += 1;
        Ok(())
    }
    fn byte(&mut self, value: u8) -> Result<()> {
        for bit in 0..8 {
            self.bit((value >> bit) & 1 != 0)?;
        }
        Ok(())
    }
    fn root(&mut self, root: Option<u16>, geometry: &BoundGeometry) -> Result<()> {
        let signature = match root {
            Some(r) => geometry.signature(r).map_err(|_| Error::Geometry)?,
            None => [0; 2],
        };
        for bit in 0..120 {
            self.bit((signature[bit >> 6] >> (bit & 63)) & 1 != 0)?;
        }
        Ok(())
    }
}

/// No target bytes, expected actions or identity/hash bits are accepted.
/// Padding is zero; all present payload bytes have separate presence masks.
pub fn pack_fields(
    fields: &Fields,
    phase: Phase,
    geometry: &BoundGeometry,
) -> Result<[bool; INPUT_BITS]> {
    pack(fields, phase, geometry, Intervention::Full)
}
fn pack(
    fields: &Fields,
    phase: Phase,
    geometry: &BoundGeometry,
    intervention: Intervention,
) -> Result<[bool; INPUT_BITS]> {
    let mut out = Packer {
        bits: [false; INPUT_BITS],
        pos: 0,
    };
    for roots in [fields.initial_roots, fields.roots] {
        for root in roots {
            out.root(Some(root), geometry)?;
        }
    }
    let leases = if intervention == Intervention::ReadDisabled {
        [None, None]
    } else {
        [fields.selected, fields.previous]
    };
    for lease in leases {
        for lane in 0..4 {
            let root = lease.map(|l| {
                if intervention == Intervention::FirstRootOnly && lane != 0 {
                    geometry.identity()
                } else {
                    l.roots()[lane]
                }
            });
            out.root(root, geometry)?;
        }
    }
    for lease in leases {
        for index in 0..64 {
            let byte = if intervention == Intervention::PayloadMasked {
                0
            } else {
                lease
                    .and_then(|l| l.payload().get(index).copied())
                    .unwrap_or(0)
            };
            out.byte(byte)?;
        }
    }
    for lease in leases {
        for index in 0..64 {
            out.bit(lease.is_some_and(|l| index < l.payload().len()))?;
        }
    }
    for lease in leases {
        for lane in 0..2 {
            out.root(lease.map(|l| l.keys()[lane]), geometry)?;
        }
    }
    // The preceding fixed groups occupy precisely 3552 bits.
    if out.pos != 3552 {
        return Err(Error::Shape);
    }
    for n in 0..7 {
        out.bit(n == phase as u8)?;
    }
    out.byte(fields.round)?;
    out.byte(fields.position)?;
    out.bit(fields.last_observed.is_some())?;
    out.byte(fields.last_observed.unwrap_or(0))?;
    for z in fields.zeta_bins {
        out.byte(z)?;
    }
    for lease in leases {
        out.bit(lease.is_some())?;
        out.byte(lease.map_or(0, |l| l.payload().len() as u8))?;
    }
    out.bit(fields.active.is_some())?;
    out.byte(if intervention == Intervention::PayloadMasked {
        0
    } else {
        fields.active.map_or(0, |a| a.byte())
    })?;
    out.byte(fields.active.map_or(0, |a| a.cursor()))?;
    out.byte(fields.active.map_or(0, |a| a.lease().payload().len() as u8))?;
    out.bit(fields.active.is_some_and(|a| a.acknowledged()))?;
    out.bit(fields.active.is_some_and(|a| a.provisional().is_some()))?;
    for valid in fields.numeric_valid {
        out.bit(valid)?;
    }
    Ok(out.bits)
}

#[derive(Clone, Debug)]
pub struct Audit {
    pub cells: Vec<u32>,
    pub calls: u64,
    pub gates: u64,
}
impl Default for Audit {
    fn default() -> Self {
        Self {
            cells: vec![0; CELLS],
            calls: 0,
            gates: 0,
        }
    }
}

pub struct Policy<'g> {
    params: Params,
    topology: Topology,
    geometry: &'g BoundGeometry,
    signatures: [[u64; 2]; 120],
    digest: [u8; 32],
    audit: Audit,
    pub environment: Environment,
    pub intervention: Intervention,
}
impl<'g> Policy<'g> {
    pub fn new(
        params: Params,
        geometry: &'g BoundGeometry,
        environment: Environment,
        intervention: Intervention,
    ) -> Result<Self> {
        params.validate()?;
        let topology = Topology::seeded(params.seed);
        topology.validate()?;
        let mut signatures = [[0; 2]; 120];
        for (root, signature) in signatures.iter_mut().enumerate() {
            *signature = geometry
                .signature(root as u16)
                .map_err(|_| Error::Geometry)?;
        }
        let mut h = blake3::Hasher::new();
        h.update(b"uor-hamming-shared-policy/1");
        h.update(&params.digest());
        h.update(&geometry.identity_digest());
        let digest = *h.finalize().as_bytes();
        Ok(Self {
            params,
            topology,
            geometry,
            signatures,
            digest,
            audit: Audit::default(),
            environment,
            intervention,
        })
    }
    pub fn params(&self) -> &Params {
        &self.params
    }
    pub fn geometry_digest(&self) -> [u8; 32] {
        self.geometry.identity_digest()
    }
    pub fn audit(&self) -> &Audit {
        &self.audit
    }
    pub fn topology(&self) -> &Topology {
        &self.topology
    }
    pub fn identity(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(&self.digest);
        h.update(&[self.intervention as u8]);
        *h.finalize().as_bytes()
    }
    pub fn evaluate(&mut self, fields: &Fields, phase: Phase) -> Result<Decision> {
        let mut previous = pack(fields, phase, self.geometry, self.intervention)?;
        let mut next = [false; INPUT_BITS];
        let mut start = 0;
        for width in WIDTHS {
            for local in 0..width {
                let gate = start + local;
                let mut address = 0usize;
                for (pin, &source) in self.topology.wires[gate].iter().enumerate() {
                    address |= usize::from(previous[usize::from(source)]) << pin;
                }
                let cell = &mut self.audit.cells[(gate << 4) | address];
                *cell = cell.checked_add(1).ok_or(Error::AuditOverflow)?;
                next[local] = (self.params.tables[gate] >> address) & 1 != 0;
            }
            std::mem::swap(&mut previous, &mut next);
            start += width;
        }
        self.audit.calls = self
            .audit
            .calls
            .checked_add(1)
            .ok_or(Error::AuditOverflow)?;
        self.audit.gates = self
            .audit
            .gates
            .checked_add(GATES as u64)
            .ok_or(Error::AuditOverflow)?;
        let mut roots = [0; 4];
        for (lane, root) in roots.iter_mut().enumerate() {
            let mut signature = [0u64; 2];
            let base = [0, 120, 240, 360][lane];
            for bit in 0..120 {
                signature[bit >> 6] |= u64::from(previous[base + bit]) << (bit & 63);
            }
            let mut distance = u32::MAX;
            for (candidate, code) in self.signatures.iter().enumerate() {
                let d =
                    (signature[0] ^ code[0]).count_ones() + (signature[1] ^ code[1]).count_ones();
                if d < distance {
                    distance = d;
                    *root = candidate as u16;
                }
            }
        }
        fn scores<const N: usize>(bits: &[bool; INPUT_BITS], start: usize) -> [u8; N] {
            std::array::from_fn(|n| {
                (0..4).fold(0, |score, bit| {
                    score | (u8::from(bits[start + (n << 2) + bit]) << bit)
                })
            })
        }
        Ok(Decision {
            roots,
            extent_scores: scores(&previous, 480),
            control_scores: scores(&previous, 736),
            emission_scores: scores(&previous, 768),
        })
    }
    fn fields(&self, c: &runtime::Context<'_>, selected: Option<Lease>) -> Fields {
        Fields {
            initial_roots: c.initial_roots,
            roots: c.roots,
            selected,
            previous: c
                .history
                .iter()
                .rev()
                .filter_map(|h| h.and_then(|h| h.selected))
                .next(),
            active: None,
            last_observed: self.environment.last_observed,
            zeta_bins: self.environment.zeta_bins,
            position: self.environment.position,
            round: c.round,
            numeric_valid: [false; 2],
        }
    }
}
impl runtime::Policy for Policy<'_> {
    fn identity(&self) -> [u8; 32] {
        self.identity()
    }
    fn query(&mut self, c: &runtime::Context<'_>) -> runtime::Result<runtime::Query> {
        let d = self
            .evaluate(&self.fields(c, None), Phase::Query)
            .map_err(|_| runtime::Error::Policy("shared query"))?;
        let roots = [d.roots[0], d.roots[1]];
        Ok(runtime::Query {
            roots,
            null: if self.intervention == Intervention::ReadDisabled {
                roots
            } else {
                [d.roots[2], d.roots[3]]
            },
        })
    }
    fn extent(
        &mut self,
        c: &runtime::SelectedContext<'_>,
        legal: &[bool; 64],
    ) -> runtime::Result<u8> {
        self.evaluate(&self.fields(&c.context, Some(c.selected)), Phase::Extent)
            .and_then(|d| d.extent(legal))
            .map_err(|_| runtime::Error::Policy("shared extent"))
    }
    fn delta(&mut self, c: &runtime::SelectedContext<'_>) -> runtime::Result<[u16; 4]> {
        let d = self
            .evaluate(&self.fields(&c.context, Some(c.selected)), Phase::Delta)
            .map_err(|_| runtime::Error::Policy("shared delta"))?;
        Ok(if self.intervention == Intervention::UpdateDisabled {
            [self.geometry.identity(); 4]
        } else {
            d.roots
        })
    }
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
