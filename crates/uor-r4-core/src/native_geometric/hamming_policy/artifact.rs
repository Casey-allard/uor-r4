//! Host-side, source-bound parameter codec for the deterministic Hamming policy.
//! This experimental artifact carries no training or language qualification claim.
use super::policy::Params;
use crate::native_geometric::addressed_attention::artifact::BoundGeometry;
use serde::{Deserialize, Serialize};

pub const MAX_ARTIFACT_BYTES: usize = 64 * 1024;
pub const SCHEMA: &str = "uor-r4.hamming-policy/1";
pub const VERSION: u16 = 1;
pub const REFINEMENTS: u8 = 2;
const STATUS: &str = "INITIALIZED_OR_CONFORMANCE_INTERVENTION_NO_OPTIMIZER";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactError {
    Limit,
    Wire,
    Version,
    Configuration,
    Source,
    Geometry,
    Parameters,
    Checksum,
}
impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hamming policy artifact: {self:?}")
    }
}
impl std::error::Error for ArtifactError {}
pub type Result<T> = std::result::Result<T, ArtifactError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    topology: [u16; 4],
    truth_table_bits: u8,
    geometric_output: [u16; 2],
    extent_output: [u16; 2],
    action_output: [u16; 2],
    emission_output: [u16; 2],
    geometric_decoder: String,
    score_decoder: String,
    input_encoding: String,
    vocabulary: u16,
    eos: u16,
    refinements: u8,
    max_span: u8,
    occurrences: u16,
    results: u8,
    operator_version: u16,
}
impl Configuration {
    fn current() -> Self {
        Self {
            topology: [4096, 1024, 256, 1796],
            truth_table_bits: 16,
            geometric_output: [0, 480],
            extent_output: [480, 736],
            action_output: [736, 764],
            emission_output: [768, 1796],
            geometric_decoder: "four-120-bit-signatures-nearest-Hamming-canonical-root-tie/1"
                .into(),
            score_decoder: "unsigned-4-bit-legal-argmax-lowest-index-tie/1".into(),
            input_encoding: "typed-four-root-two-full-span-4096-bit-frame/1".into(),
            vocabulary: 257,
            eos: 256,
            refinements: REFINEMENTS,
            max_span: 64,
            occurrences: 256,
            results: 8,
            operator_version: 1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireParams {
    seed: u64,
    tables: Vec<u16>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    schema: String,
    version: u16,
    configuration: Configuration,
    source_digest: [u8; 32],
    geometry_digest: [u8; 32],
    parameter_digest: [u8; 32],
    training_status: String,
    optimizer_updates: u64,
    fit_calls: u64,
    parameters: WireParams,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload: Payload,
    checksum: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct Artifact {
    payload: Payload,
    params: Params,
    id: [u8; 32],
}
impl Artifact {
    /// Initialized parameters and declared finite interventions only. This codec
    /// does not confer optimizer provenance or mark an artifact as learned.
    pub fn new(params: Params, geometry: &BoundGeometry) -> Result<Self> {
        params.validate().map_err(|_| ArtifactError::Parameters)?;
        let payload = Payload {
            schema: SCHEMA.into(),
            version: VERSION,
            configuration: Configuration::current(),
            source_digest: implementation_digest(),
            geometry_digest: geometry.identity_digest(),
            parameter_digest: params.digest(),
            training_status: STATUS.into(),
            optimizer_updates: 0,
            fit_calls: 0,
            parameters: WireParams {
                seed: params.seed,
                tables: params.tables.clone(),
            },
        };
        let id = checksum(&payload)?;
        Ok(Self {
            payload,
            params,
            id,
        })
    }
    pub fn params(&self) -> &Params {
        &self.params
    }
    pub fn id(&self) -> [u8; 32] {
        self.id
    }
    pub fn geometry_digest(&self) -> [u8; 32] {
        self.payload.geometry_digest
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        let bytes = serde_json::to_vec(&Envelope {
            payload: self.payload.clone(),
            checksum: self.id,
        })
        .map_err(|_| ArtifactError::Wire)?;
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ArtifactError::Limit);
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], geometry: &BoundGeometry) -> Result<Self> {
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ArtifactError::Limit);
        }
        let envelope: Envelope = serde_json::from_slice(bytes).map_err(|_| ArtifactError::Wire)?;
        let p = &envelope.payload;
        if p.schema != SCHEMA || p.version != VERSION {
            return Err(ArtifactError::Version);
        }
        if p.configuration != Configuration::current()
            || p.training_status != STATUS
            || p.optimizer_updates != 0
            || p.fit_calls != 0
        {
            return Err(ArtifactError::Configuration);
        }
        if p.source_digest != implementation_digest() {
            return Err(ArtifactError::Source);
        }
        if p.geometry_digest != geometry.identity_digest() {
            return Err(ArtifactError::Geometry);
        }
        let params = Params {
            seed: p.parameters.seed,
            tables: p.parameters.tables.clone(),
        };
        params.validate().map_err(|_| ArtifactError::Parameters)?;
        if params.digest() != p.parameter_digest {
            return Err(ArtifactError::Parameters);
        }
        if checksum(p)? != envelope.checksum {
            return Err(ArtifactError::Checksum);
        }
        Ok(Self {
            payload: envelope.payload,
            params,
            id: envelope.checksum,
        })
    }
}

fn checksum(payload: &Payload) -> Result<[u8; 32]> {
    let encoded = serde_json::to_vec(payload).map_err(|_| ArtifactError::Wire)?;
    let mut h = blake3::Hasher::new();
    h.update(b"uor-r4.hamming-policy-artifact/1");
    h.update(&(encoded.len() as u64).to_le_bytes());
    h.update(&encoded);
    Ok(*h.finalize().as_bytes())
}

/// Name/length framed implementation bytes, plus the separate actual geometry
/// table identity in each artifact. Test/report drivers are evidence, not serving
/// dependencies; their exact identities belong in the external source receipt.
pub fn implementation_digest() -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"uor-r4.hamming-policy-implementation/1");
    for (name, bytes) in [
        ("policy/mod", include_bytes!("mod.rs").as_slice()),
        ("policy/policy", include_bytes!("policy.rs").as_slice()),
        ("policy/session", include_bytes!("session.rs").as_slice()),
        ("policy/artifact", include_bytes!("artifact.rs").as_slice()),
        ("policy/credit", include_bytes!("credit.rs").as_slice()),
        (
            "refinement/mod",
            include_bytes!("../hamming_refinement/mod.rs").as_slice(),
        ),
        (
            "refinement/metric",
            include_bytes!("../hamming_refinement/metric.rs").as_slice(),
        ),
        (
            "refinement/runtime",
            include_bytes!("../hamming_refinement/runtime.rs").as_slice(),
        ),
        (
            "addressed/artifact",
            include_bytes!("../addressed_attention/artifact.rs").as_slice(),
        ),
        (
            "addressed/objects",
            include_bytes!("../addressed_attention/objects.rs").as_slice(),
        ),
        (
            "addressed/circuit",
            include_bytes!("../addressed_attention/circuit.rs").as_slice(),
        ),
        (
            "native/training",
            include_bytes!("../training.rs").as_slice(),
        ),
        ("native/anchors", include_bytes!("../anchors.rs").as_slice()),
        (
            "canonical_lexical_ingestion",
            include_bytes!("../../canonical_lexical_ingestion.rs").as_slice(),
        ),
        (
            "prime_route_attention",
            include_bytes!("../../prime_route_attention.rs").as_slice(),
        ),
        (
            "bounded_global_exact_spin_attention",
            include_bytes!("../../bounded_global_exact_spin_attention.rs").as_slice(),
        ),
    ] {
        h.update(&(name.len() as u64).to_le_bytes());
        h.update(name.as_bytes());
        h.update(&(bytes.len() as u64).to_le_bytes());
        h.update(bytes);
    }
    *h.finalize().as_bytes()
}

#[cfg(test)]
#[path = "artifact_tests.rs"]
mod tests;
