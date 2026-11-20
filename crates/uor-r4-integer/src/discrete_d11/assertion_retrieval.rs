//! Opt-in query-conditioned assertion filtering on an unchanged source carrier.
use super::*;
use crate::relation_retrieval::{Arm, Plan};

pub(super) const SCHEMA: &str = "uor-r4.lut-discrete-assertion-retrieval/1";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    lane: String,
    parent_sha256: String,
}

pub fn pack_assertion_retrieval(parent: &[u8], arm: Arm) -> Result<Vec<u8>> {
    let envelope = Envelope {
        schema: SCHEMA.into(),
        lane: match arm {
            Arm::Exact => "exact",
            Arm::Geometry => "geometry",
        }
        .into(),
        parent_sha256: hex::encode(Sha256::digest(parent)),
    };
    let header = serde_json::to_vec(&envelope)?;
    let mut bytes = Vec::with_capacity(16 + header.len() + parent.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(header.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(parent);
    IntegerDiscreteModel::from_bytes(&bytes)?;
    Ok(bytes)
}

pub(super) fn read(
    bytes: &[u8],
    header: &[u8],
    end: usize,
    allow_role: bool,
) -> Result<IntegerDiscreteModel> {
    if !allow_role {
        return Err(invalid("nested assertion retrieval carrier"));
    }
    let envelope: Envelope = serde_json::from_slice(header)?;
    let arm = match envelope.lane.as_str() {
        "exact" => Arm::Exact,
        "geometry" => Arm::Geometry,
        _ => return Err(invalid("unknown assertion retrieval lane")),
    };
    let parent = &bytes[end..];
    if parent.len() < 16
        || &parent[..8] != MAGIC
        || hex::encode(Sha256::digest(parent)) != envelope.parent_sha256
    {
        return Err(invalid("assertion retrieval parent binding mismatch"));
    }
    let length = u64::from_le_bytes(parent[8..16].try_into().unwrap());
    if length > MAX_HEADER {
        return Err(invalid("assertion retrieval parent header too long"));
    }
    let h = parent
        .get(16..16 + length as usize)
        .ok_or_else(|| invalid("truncated assertion retrieval parent"))?;
    #[derive(Deserialize)]
    struct Schema {
        schema: String,
    }
    if !matches!(
        serde_json::from_slice::<Schema>(h)?.schema.as_str(),
        DISCRETE_REQUESTED_OCCURRENCE_SCHEMA | contextual_source::RECURRENCE_SCHEMA
    ) {
        return Err(invalid(
            "assertion retrieval requires an ordered-onset or recurrence parent",
        ));
    }
    let mut model = IntegerDiscreteModel::from_bytes(parent)?;
    model.assertion_arm = Some(arm);
    model.artifact_sha256 = hex::encode(Sha256::digest(bytes));
    Ok(model)
}

impl IntegerDiscreteModel {
    /// Frame a whole query directly from the carrier and one original person.
    /// Both supplied-person diagnostics and ordinary prediction call this path.
    pub fn assertion_retrieval_plan<'a>(
        &'a self,
        ids: &'a [u32],
        person: RoleSpan,
        excluded: Option<RoleSpan>,
    ) -> Result<Plan<'a>> {
        let (policy, paired) = self
            .paired_policy()
            .ok_or_else(|| invalid("missing paired reader"))?;
        let content = self
            .requested_role_framing()
            .ok_or_else(|| invalid("missing requested-role framing"))?
            .content(ids)?;
        if person.start < content.start || person.start >= person.end || person.end > content.end {
            return Err(invalid("assertion query person outside user content"));
        }
        let left = (content.start..person.start)
            .rev()
            .find(|&i| paired.delimiters.contains(&ids[i]))
            .map_or(content.start, |i| i + 1);
        let right = (person.end..content.end)
            .find(|&i| paired.delimiters.contains(&ids[i]))
            .unwrap_or(content.end);
        let onset = policy
            .onset
            .as_ref()
            .ok_or_else(|| invalid("missing frozen value onset"))?;
        Plan::new(
            ids,
            &onset.packed_roots,
            &policy.span.classes,
            content,
            left..right,
            person,
            excluded,
            &paired.delimiters,
            &ROLE_COPY_STOPS,
        )
    }
}
