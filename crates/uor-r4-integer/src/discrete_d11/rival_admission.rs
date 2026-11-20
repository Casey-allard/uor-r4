//! Versioned opt-in calibration over the exact D28 fitted whole-word parent.
use super::*;
use crate::all_starts::{OnsetMode, NEGATIVE_CALIBRATION_BYTES};

pub(super) const SCHEMA: &str = "uor-r4.lut-discrete-rival-admission/1";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    parent_sha256: String,
    train_sha256: String,
    calibration_sha256: String,
}

pub fn pack_rival_admission(
    parent: &[u8],
    train_sha256: [u8; 32],
    calibration: &[u8],
) -> Result<Vec<u8>> {
    let header = serde_json::to_vec(&Envelope {
        schema: SCHEMA.into(),
        parent_sha256: hex::encode(Sha256::digest(parent)),
        train_sha256: hex::encode(train_sha256),
        calibration_sha256: hex::encode(Sha256::digest(calibration)),
    })?;
    let mut bytes = Vec::with_capacity(16 + header.len() + calibration.len() + parent.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(header.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(calibration);
    bytes.extend_from_slice(parent);
    IntegerDiscreteModel::from_bytes(&bytes)?;
    Ok(bytes)
}

pub(super) fn read(
    bytes: &[u8],
    header: &[u8],
    header_end: usize,
    allow_role: bool,
) -> Result<IntegerDiscreteModel> {
    if !allow_role {
        return Err(invalid("nested rival admission carrier"));
    }
    let envelope: Envelope = serde_json::from_slice(header)?;
    let (calibration, parent) = bytes[header_end..]
        .split_at_checked(NEGATIVE_CALIBRATION_BYTES)
        .ok_or_else(|| invalid("truncated rival admission calibration"))?;
    if calibration.len() != NEGATIVE_CALIBRATION_BYTES
        || Sha256::digest(calibration).as_slice() != read_role_digest(&envelope.calibration_sha256)?
        || Sha256::digest(parent).as_slice() != read_role_digest(&envelope.parent_sha256)?
        || parent.len() < 16
        || &parent[..8] != MAGIC
    {
        return Err(invalid(
            "rival admission parent or calibration binding mismatch",
        ));
    }
    let length = u64::from_le_bytes(parent[8..16].try_into().unwrap());
    if length > MAX_HEADER {
        return Err(invalid("rival admission parent header too long"));
    }
    let parent_header: OnsetEnvelope = serde_json::from_slice(
        parent
            .get(16..16 + length as usize)
            .ok_or_else(|| invalid("truncated rival admission parent"))?,
    )?;
    if parent_header.schema != DISCRETE_REQUESTED_OCCURRENCE_SCHEMA
        || parent_header.profile.mode != OnsetMode::OrderedWholeWord
        || read_role_digest(&parent_header.profile.train_sha256)?
            != read_role_digest(&envelope.train_sha256)?
    {
        return Err(invalid(
            "rival admission requires the same fitted whole-word TRAIN",
        ));
    }
    let mut model = IntegerDiscreteModel::from_bytes(parent)?;
    let onset = model
        .role_policy
        .as_mut()
        .and_then(|policy| policy.onset.as_mut())
        .ok_or_else(|| invalid("missing fitted whole-word onset state"))?;
    onset.bank = onset.bank.clone().with_negative_calibration(calibration)?;
    model.artifact_sha256 = hex::encode(Sha256::digest(bytes));
    Ok(model)
}
