//! Versioned experimental addressed-attention artifact and bound integer geometry.
//! Canonical construction/validation is host work; serving reads finite tables.
use super::circuit::{CircuitError, PrimitiveExport, Topology, COMPILED_BYTES};

pub const MAX_ARTIFACT_BYTES: usize = 256 * 1024;
pub const SCHEMA_VERSION: u16 = 1;
pub const LAYOUT_VERSION: u16 = 1;
pub const OPERATOR_VERSION: u16 = 1;
pub const CODEC_VERSION: u16 = 1;
const MAGIC: &[u8; 8] = b"UORAAF01";
// Bit 0: exact paired-H4 companion integration unfinished.
// Bit 1: learned durable prime/n-let page writer unfinished.
const UNFINISHED: u16 = 3;
const INITIALIZED_NO_TRAINING: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactError {
    Wire,
    Version,
    Limit,
    Geometry,
    Domain,
    Provenance,
    Circuit(CircuitError),
}
impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ArtifactError {}
impl From<CircuitError> for ArtifactError {
    fn from(value: CircuitError) -> Self {
        Self::Circuit(value)
    }
}
pub type Result<T> = std::result::Result<T, ArtifactError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundGeometry {
    construction: [u8; 32],
    identity: u16,
    row_bases: [usize; 120],
    products: Vec<u16>,
    inverses: [u16; 120],
    ranks: [i16; 120],
    signatures: [[u64; 2]; 120],
    primes: [u32; 258],
    leaves: [u16; 258],
    phases: [[u16; 8]; 258],
    digest: [u8; 32],
}
impl BoundGeometry {
    /// Host-only construction, retaining exact canonical prime, fixed-zeta,
    /// orientation and anchor identities through the construction digest.
    pub fn canonical() -> Result<Self> {
        let native = crate::native_geometric::training::geometry(258, 256)
            .map_err(|_| ArtifactError::Geometry)?;
        let construction_bytes =
            serde_json::to_vec(&native).map_err(|_| ArtifactError::Geometry)?;
        let construction = digest(b"uor-addressed-canonical-geometry-v1", &construction_bytes);
        let mut values: Vec<_> = native
            .anchors
            .rows
            .iter()
            .map(|r| r.root_scaled_zphi[0])
            .collect();
        values.sort_by(|a, b| {
            crate::native_geometric::training::exact_sign([a[0] - b[0], a[1] - b[1]]).cmp(&0)
        });
        values.dedup();
        let zero = values
            .iter()
            .position(|v| *v == [0, 0])
            .ok_or(ArtifactError::Geometry)? as i16;
        let mut ranks = [0i16; 120];
        for (i, row) in native.anchors.rows.iter().enumerate() {
            let rank = values
                .iter()
                .position(|v| *v == row.root_scaled_zphi[0])
                .ok_or(ArtifactError::Geometry)?;
            *ranks.get_mut(i).ok_or(ArtifactError::Geometry)? = rank as i16 - zero;
        }
        let row_bases: [usize; 120] = native
            .row_bases
            .try_into()
            .map_err(|_| ArtifactError::Geometry)?;
        let inverses: [u16; 120] = native
            .inverses
            .try_into()
            .map_err(|_| ArtifactError::Geometry)?;
        if native.identity >= 120
            || native.products.len() != 14_400
            || native.products.iter().any(|&r| r >= 120)
            || inverses.iter().any(|&r| r >= 120)
            || row_bases.iter().enumerate().any(|(i, &r)| r != i * 120)
            || native.tokens.len() != 258
        {
            return Err(ArtifactError::Geometry);
        }
        let mut signatures = [[0u64; 2]; 120];
        for root in 0..120 {
            for landmark in 0..120 {
                let relative = native.products[row_bases[root] + usize::from(inverses[landmark])];
                if ranks[usize::from(relative)] > 0 {
                    signatures[root][landmark >> 6] |= 1u64 << (landmark & 63);
                }
            }
        }
        let mut result = Self {
            construction,
            identity: native.identity,
            row_bases,
            products: native.products,
            inverses,
            ranks,
            signatures,
            primes: [0; 258],
            leaves: [0; 258],
            phases: [[0; 8]; 258],
            digest: [0; 32],
        };
        for (i, token) in native.tokens.iter().enumerate() {
            result.primes[i] = token.prime;
            result.leaves[i] = token.leaf;
            result.phases[i] = token.phases;
        }
        result.digest = digest(b"uor-addressed-bound-geometry-v1", &result.encode_tables());
        Ok(result)
    }
    pub fn identity(&self) -> u16 {
        self.identity
    }
    pub fn identity_digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn id(&self) -> [u8; 32] {
        self.identity_digest()
    }
    pub fn product(&self, a: u16, b: u16) -> Result<u16> {
        if a >= 120 || b >= 120 {
            return Err(ArtifactError::Domain);
        }
        Ok(self.products[self.row_bases[usize::from(a)] + usize::from(b)])
    }
    pub fn score(&self, query: [u16; 2], keys: [u16; 2]) -> Result<i16> {
        if query.iter().chain(keys.iter()).any(|&r| r >= 120) {
            return Err(ArtifactError::Domain);
        }
        let mut result = 0;
        for lane in 0..2 {
            result += self.ranks
                [usize::from(self.product(query[lane], self.inverses[usize::from(keys[lane])])?)];
        }
        Ok(result)
    }
    pub fn signature(&self, root: u16) -> Result<[u64; 2]> {
        self.signatures
            .get(usize::from(root))
            .copied()
            .ok_or(ArtifactError::Domain)
    }
    /// Byte b occupies token b+2; BOS/EOS keep their canonical reserved rows.
    pub fn byte_phases(&self, byte: u8) -> [u16; 4] {
        let phases = self.phases[usize::from(byte) + 2];
        [phases[0], phases[1], phases[2], phases[3]]
    }
    fn encode_tables(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(38_000);
        out.extend_from_slice(b"UORABG01");
        out.extend_from_slice(&self.construction);
        put16(&mut out, self.identity);
        for &base in &self.row_bases {
            put16(&mut out, base as u16);
        }
        for &root in &self.products {
            put16(&mut out, root);
        }
        for &root in &self.inverses {
            put16(&mut out, root);
        }
        for &rank in &self.ranks {
            out.extend_from_slice(&rank.to_le_bytes());
        }
        for signature in &self.signatures {
            for word in signature {
                out.extend_from_slice(&word.to_le_bytes());
            }
        }
        for i in 0..258 {
            out.extend_from_slice(&self.primes[i].to_le_bytes());
            put16(&mut out, self.leaves[i]);
            for &phase in &self.phases[i] {
                put16(&mut out, phase);
            }
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provenance {
    pub seed: u64,
    pub training_data_digest: [u8; 32],
    pub training_config_digest: [u8; 32],
    pub parameter_digest: [u8; 32],
    pub source_digest: [u8; 32],
    pub parent: Option<[u8; 32]>,
}
impl Provenance {
    fn validate(&self) -> Result<()> {
        if [
            self.training_data_digest,
            self.training_config_digest,
            self.parameter_digest,
            self.source_digest,
        ]
        .iter()
        .any(|d| *d == [0; 32])
            || self.parent == Some([0; 32])
            || self.source_digest != super::policy::implementation_digest()
        {
            return Err(ArtifactError::Provenance);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    compiled: PrimitiveExport,
    geometry: BoundGeometry,
    provenance: Provenance,
    id: [u8; 32],
}
impl Model {
    pub fn new(compiled: PrimitiveExport, provenance: Provenance) -> Result<Self> {
        provenance.validate()?;
        let compiled = PrimitiveExport::decode(&compiled.encode())?;
        validate_wiring(&compiled)?;
        let geometry = BoundGeometry::canonical()?;
        let mut model = Self {
            compiled,
            geometry,
            provenance,
            id: [0; 32],
        };
        let body = model.body();
        if body.len() + 32 > MAX_ARTIFACT_BYTES {
            return Err(ArtifactError::Limit);
        }
        model.id = digest(b"uor-addressed-model-v1", &body);
        Ok(model)
    }
    pub fn id(&self) -> [u8; 32] {
        self.id
    }
    pub fn geometry(&self) -> &BoundGeometry {
        &self.geometry
    }
    pub fn compiled(&self) -> &PrimitiveExport {
        &self.compiled
    }
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
    pub fn paired_h4_implemented(&self) -> bool {
        false
    }
    pub fn durable_writer_implemented(&self) -> bool {
        false
    }
    pub fn initialized_parameters_no_training(&self) -> bool {
        true
    }
    fn body(&self) -> Vec<u8> {
        let geometry = self.geometry.encode_tables();
        let compiled = self.compiled.encode();
        let mut out = Vec::with_capacity(geometry.len() + compiled.len() + 256);
        out.extend_from_slice(MAGIC);
        for version in [
            SCHEMA_VERSION,
            LAYOUT_VERSION,
            OPERATOR_VERSION,
            CODEC_VERSION,
            UNFINISHED,
            INITIALIZED_NO_TRAINING,
        ] {
            put16(&mut out, version);
        }
        out.extend_from_slice(&self.provenance.seed.to_le_bytes());
        for value in [
            self.provenance.training_data_digest,
            self.provenance.training_config_digest,
            self.provenance.parameter_digest,
            self.provenance.source_digest,
        ] {
            out.extend_from_slice(&value);
        }
        out.push(u8::from(self.provenance.parent.is_some()));
        out.extend_from_slice(&self.provenance.parent.unwrap_or([0; 32]));
        out.extend_from_slice(&(geometry.len() as u32).to_le_bytes());
        out.extend_from_slice(&geometry);
        out.extend_from_slice(&(compiled.len() as u32).to_le_bytes());
        out.extend_from_slice(&compiled);
        out
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut out = self.body();
        out.extend_from_slice(&self.id);
        out
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ArtifactError::Limit);
        }
        let body_len = bytes.len().checked_sub(32).ok_or(ArtifactError::Wire)?;
        let (body, checksum) = bytes.split_at(body_len);
        let id = digest(b"uor-addressed-model-v1", body);
        if checksum != id {
            return Err(ArtifactError::Wire);
        }
        let mut reader = Reader(body);
        if reader.take(8)? != MAGIC {
            return Err(ArtifactError::Wire);
        }
        for version in [
            SCHEMA_VERSION,
            LAYOUT_VERSION,
            OPERATOR_VERSION,
            CODEC_VERSION,
            UNFINISHED,
            INITIALIZED_NO_TRAINING,
        ] {
            if reader.u16()? != version {
                return Err(ArtifactError::Version);
            }
        }
        let seed = u64::from_le_bytes(reader.array()?);
        let training_data_digest = reader.array()?;
        let training_config_digest = reader.array()?;
        let parameter_digest = reader.array()?;
        let source_digest = reader.array()?;
        let parent_tag = reader.take(1)?[0];
        let parent_bytes = reader.array()?;
        let parent = match parent_tag {
            0 if parent_bytes == [0; 32] => None,
            1 => Some(parent_bytes),
            _ => return Err(ArtifactError::Provenance),
        };
        let provenance = Provenance {
            seed,
            training_data_digest,
            training_config_digest,
            parameter_digest,
            source_digest,
            parent,
        };
        provenance.validate()?;
        // Lengths are checked before canonical construction or copying input.
        let geometry_len = reader.u32()? as usize;
        let geometry_bytes = reader.take(geometry_len)?;
        let compiled_len = reader.u32()? as usize;
        if compiled_len != COMPILED_BYTES + 8 {
            return Err(ArtifactError::Wire);
        }
        let compiled_bytes = reader.take(compiled_len)?;
        if !reader.0.is_empty() {
            return Err(ArtifactError::Wire);
        }
        let compiled = PrimitiveExport::decode(compiled_bytes)?;
        validate_wiring(&compiled)?;
        let geometry = BoundGeometry::canonical()?;
        if geometry.encode_tables() != geometry_bytes {
            return Err(ArtifactError::Geometry);
        }
        Ok(Self {
            compiled,
            geometry,
            provenance,
            id,
        })
    }
}
fn validate_wiring(compiled: &PrimitiveExport) -> Result<()> {
    if compiled.circuit.topology() != &Topology::seeded(super::policy::WIRING_SEED) {
        return Err(ArtifactError::Circuit(CircuitError::Topology));
    }
    Ok(())
}
fn put16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn digest(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(domain);
    h.update(bytes);
    *h.finalize().as_bytes()
}
struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if n > self.0.len() {
            return Err(ArtifactError::Wire);
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.take(N)?.try_into().map_err(|_| ArtifactError::Wire)
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }
}

#[cfg(test)]
#[path = "artifact_tests.rs"]
mod tests;
