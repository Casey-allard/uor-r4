use crate::{discrete_d11::IntegerDiscreteModel, IntegerError, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_count_runtime::{CompactCountLookup, FrozenPickMixture};

const A1_TRUNK: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";

fn invalid(s: impl ToString) -> IntegerError {
    IntegerError::Invalid(s.to_string())
}

#[derive(Deserialize)]
struct Fit {
    schema: String,
    prepared: Preparation,
}

#[derive(Deserialize)]
struct Preparation {
    format: String,
    c_sha256: String,
    trunk_sha256: String,
    compact_sha256: String,
}

fn digest(hex_string: &str) -> Result<[u8; 32]> {
    let mut bytes = [0; 32];
    hex::decode_to_slice(hex_string, &mut bytes).map_err(invalid)?;
    Ok(bytes)
}

pub struct P1<'a> {
    counts: CompactCountLookup<'a>,
    weights: FrozenPickMixture,
    codes: Vec<[u16; 4]>,
    pub artifact_sha256: String,
    count_sha256: String,
    weights_sha256: String,
    fit_manifest_sha256: String,
}

impl<'a> P1<'a> {
    pub fn load(bytes: &'a [u8], model: &IntegerDiscreteModel) -> Result<Self> {
        let (header, payload) = bytes
            .split_at_checked(48)
            .ok_or_else(|| invalid("truncated P1 package"))?;
        if &header[..8] != b"UORP1A01" {
            return Err(invalid("unknown P1 package format"));
        }
        let size = |word: &[u8]| -> Result<usize> {
            Ok(u32::from_le_bytes(word.try_into().map_err(invalid)?) as usize)
        };
        let (fit, rest) = payload
            .split_at_checked(size(&header[8..12])?)
            .ok_or_else(|| invalid("truncated P1 fit manifest"))?;
        let (weights, counts) = rest
            .split_at_checked(size(&header[12..16])?)
            .ok_or_else(|| invalid("truncated P1 weights"))?;
        if [counts, weights]
            .iter()
            .any(|part| part.get(8..12) != Some(10u32.to_le_bytes().as_slice()))
        {
            return Err(invalid(
                "P1 requires order ten for both count and fit components",
            ));
        }
        let weight_digest = Sha256::digest(weights);
        if weight_digest[..] != header[16..] {
            return Err(invalid("P1 weight digest mismatch"));
        }
        let manifest: Fit = serde_json::from_slice(fit)?;
        let prepared = manifest.prepared;
        if manifest.schema != "uor-r4.d31.a1-fit/1"
            || prepared.format != "uor-r4.d31.a1-preparation/1"
            || prepared.trunk_sha256 != A1_TRUNK
            || prepared.trunk_sha256 != model.artifact_sha256()
            || model.shape().context != 192
            || model.shape().vocabulary != 4096
            || model.shape().blocks != 4
        {
            return Err(invalid("P1 fit or trunk identity mismatch"));
        }
        let fit_digest = Sha256::digest(fit);
        let weights = FrozenPickMixture::from_bytes(weights, fit_digest.into()).map_err(invalid)?;
        let counts = CompactCountLookup::from_bytes(
            counts,
            digest(&prepared.c_sha256)?,
            digest(&prepared.compact_sha256)?,
        )
        .map_err(invalid)?;
        Ok(Self {
            counts,
            weights,
            codes: model.identity_pick_codes()?,
            artifact_sha256: hex::encode(Sha256::digest(bytes)),
            count_sha256: prepared.compact_sha256,
            weights_sha256: hex::encode(weight_digest),
            fit_manifest_sha256: hex::encode(fit_digest),
        })
    }

    pub fn component_digests(&self) -> Value {
        json!({
            "count_sha256": self.count_sha256,
            "weights_sha256": self.weights_sha256,
            "fit_manifest_sha256": self.fit_manifest_sha256
        })
    }

    pub fn counts(&self) -> &CompactCountLookup<'a> {
        &self.counts
    }

    pub fn next(
        &self,
        context: &[u32],
        read_enabled: bool,
        trace: bool,
    ) -> Result<(u32, Option<Value>)> {
        // The client can slide a full window. Raw-id prompts may also include
        // completed records, so the native read begins at the last BOS.
        let start = context.iter().rposition(|&id| id == 0).unwrap_or(0);
        let history = &context[start..];
        if history.is_empty() || history.len() > 192 || history.contains(&1) {
            return Err(invalid("P1 decoding needs an open causal record"));
        }
        let mut ids = [0u16; 192];
        for (out, &id) in ids.iter_mut().zip(history) {
            *out = u16::try_from(id).map_err(invalid)?;
        }
        let ids = &ids[..history.len()];
        if ids.iter().any(|&id| usize::from(id) >= self.codes.len()) {
            return Err(invalid("P1 prefix contains an invalid original token ID"));
        }
        let pick = read_enabled
            .then(|| uor_r4_count_runtime::code_pick(&self.codes, ids))
            .flatten();
        let choice = pick.map(|(position, distance, _)| (ids[position + 1], distance));
        let mut best = (0u16, 0u16, 0u64);
        let mut ties = 0u32;
        let mut masses = trace.then(Sha256::new);
        self.counts
            .visit_next_masses(ids, &self.weights, choice, |id, bucket, mass| {
                if let Some(masses) = &mut masses {
                    masses.update(mass.to_le_bytes());
                }
                if mass > best.2 {
                    best = (id, bucket, mass);
                    ties = 1;
                } else if mass == best.2 {
                    ties += 1;
                }
            })
            .map_err(invalid)?;
        let (id, bucket, mass) = best;
        if id == 0 {
            return Err(invalid("P1 read found no positive legal mass"));
        }
        Ok((u32::from(id), masses.map(|masses| json!({"token":id,"bucket":bucket,"mass_q48":mass,"top_ties":ties,"mass_sha256":hex::encode(masses.finalize()),
            "prefix_sha256":({
                let mut hash = Sha256::new();
                for word in ids { hash.update(word.to_le_bytes()); }
                hex::encode(hash.finalize())
            }),"context_tokens":ids.len(),"pick_successor":choice.map(|(s,_)|s),"pick_words":pick.map(|(position,distance,ties)| [position as u32,distance,ties])}))))
    }
}

pub const SCHEMA: &str = "uor-r4.lut-discrete-count/1";
const MAX_DEPLOYMENT_HEADER: usize = 4096;

#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct DeploymentHeader {
    schema: String,
    trunk_bytes: usize,
    state_bytes: usize,
    trunk_sha256: String,
    state_sha256: String,
}

pub struct DeploymentIdentity {
    pub artifact_sha256: String,
    pub header_bytes: usize,
    pub trunk_bytes: usize,
    pub state_bytes: usize,
}

/// Ordinary discrete client state. Embedded compact tables borrow the artifact
/// buffer directly. For retained D31 callers they instead borrow the sidecar.
pub struct DiscreteCountModel<'a> {
    pub model: IntegerDiscreteModel,
    pub state: Option<P1<'a>>,
    pub identity: Option<DeploymentIdentity>,
}

type DeploymentParts<'a> = (DeploymentHeader, usize, &'a [u8], &'a [u8]);

fn deployment_header(bytes: &[u8]) -> Result<Option<(DeploymentHeader, usize)>> {
    if bytes.get(..8) != Some(crate::discrete_d11::MAGIC.as_slice()) {
        return Ok(None);
    }
    let Some(length) = bytes.get(8..16) else {
        return Ok(None);
    };
    let length = u64::from_le_bytes(length.try_into().map_err(invalid)?);
    if length > MAX_DEPLOYMENT_HEADER as u64 {
        return Ok(None);
    }
    let header_end = 16 + length as usize;
    let Some(header_bytes) = bytes.get(16..header_end) else {
        return Ok(None);
    };
    #[derive(Deserialize)]
    struct Schema<'a> {
        schema: &'a str,
    }
    if serde_json::from_slice::<Schema<'_>>(header_bytes)
        .map(|header| header.schema)
        .ok()
        != Some(SCHEMA)
    {
        return Ok(None);
    }
    let header: DeploymentHeader = serde_json::from_slice(header_bytes)?;
    Ok(Some((header, header_end)))
}

fn deployment_parts(bytes: &[u8]) -> Result<Option<DeploymentParts<'_>>> {
    let Some((header, header_end)) = deployment_header(bytes)? else {
        return Ok(None);
    };
    let (trunk, state) = bytes[header_end..]
        .split_at_checked(header.trunk_bytes)
        .ok_or_else(|| invalid("truncated embedded D11 trunk"))?;
    if state.len() != header.state_bytes || trunk.is_empty() || state.is_empty() {
        return Err(invalid("embedded count state length mismatch"));
    }
    Ok(Some((header, header_end, trunk, state)))
}

fn reject_sidecar() -> IntegerError {
    invalid("embedded count state conflicts with --p1-state")
}

/// Probe an open artifact before loading the body if a sidecar was supplied.
/// Preserve the file descriptor and recheck the complete bytes in the caller.
pub fn read_client_artifact(path: &std::path::Path, has_sidecar: bool) -> Result<Vec<u8>> {
    use std::io::Read;

    if !has_sidecar {
        return Ok(std::fs::read(path)?);
    }
    let mut file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    (&mut file)
        .take((16 + MAX_DEPLOYMENT_HEADER) as u64)
        .read_to_end(&mut bytes)?;
    if deployment_header(&bytes)?.is_some() {
        return Err(reject_sidecar());
    }
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn is_embedded(bytes: &[u8], has_sidecar: bool) -> Result<bool> {
    let embedded = deployment_header(bytes)?.is_some();
    if embedded && has_sidecar {
        return Err(reject_sidecar());
    }
    Ok(embedded)
}

impl<'a> DiscreteCountModel<'a> {
    pub fn bind_legacy(model: IntegerDiscreteModel, sidecar: Option<&'a [u8]>) -> Result<Self> {
        let state = sidecar.map(|sidecar| P1::load(sidecar, &model)).transpose()?;
        Ok(Self {
            model,
            state,
            identity: None,
        })
    }

    pub fn load(bytes: &'a [u8], sidecar: Option<&'a [u8]>) -> Result<Self> {
        let Some((header, header_end, trunk, state)) = deployment_parts(bytes)? else {
            return Self::bind_legacy(IntegerDiscreteModel::from_bytes(bytes)?, sidecar);
        };
        if sidecar.is_some() {
            return Err(reject_sidecar());
        }
        let model = IntegerDiscreteModel::from_bytes(trunk)?;
        if model.artifact_sha256() != header.trunk_sha256 {
            return Err(invalid("embedded D11 trunk digest mismatch"));
        }
        let state = P1::load(state, &model)?;
        if state.artifact_sha256 != header.state_sha256 {
            return Err(invalid("embedded P1 state digest mismatch"));
        }
        Ok(Self {
            model,
            state: Some(state),
            identity: Some(DeploymentIdentity {
                artifact_sha256: hex::encode(Sha256::digest(bytes)),
                header_bytes: header_end,
                trunk_bytes: header.trunk_bytes,
                state_bytes: header.state_bytes,
            }),
        })
    }
}

/// Write a lossless deployment using the same component validation as serving.
/// The writer receives the header and two original byte slices separately.
pub fn write_deployment(trunk: &[u8], state: &[u8], path: &std::path::Path) -> Result<()> {
    use std::io::Write;

    let model = IntegerDiscreteModel::from_bytes(trunk)?;
    let state_digest = P1::load(state, &model)?.artifact_sha256;
    let header = serde_json::to_vec(&DeploymentHeader {
        schema: SCHEMA.into(),
        trunk_bytes: trunk.len(),
        state_bytes: state.len(),
        trunk_sha256: model.artifact_sha256().into(),
        state_sha256: state_digest,
    })?;
    let mut output = std::fs::File::create_new(path)?;
    output.write_all(crate::discrete_d11::MAGIC)?;
    output.write_all(&(header.len() as u64).to_le_bytes())?;
    output.write_all(&header)?;
    output.write_all(trunk)?;
    output.write_all(state)?;
    output.sync_all()?;
    Ok(())
}
