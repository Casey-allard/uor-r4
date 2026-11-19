//! Reconstruct the legacy assertion-fence input from a separately replayed role decision.
use serde_json::{from_reader, json, to_value, to_writer, Value};
use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    fs::{self, File},
    path::Path,
};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    paired_role::{PairedRoleDecision, RoleSpan},
};

fn rows(value: &Value) -> Result<&Vec<Value>, Box<dyn Error>> {
    value["rows"]
        .as_array()
        .ok_or_else(|| "missing rows".into())
}

fn span(words: &[u32], len: usize) -> Result<Option<RoleSpan>, Box<dyn Error>> {
    if words == [u32::MAX, u32::MAX] {
        return Ok(None);
    }
    let start = words[0] as usize;
    let end = words[1] as usize;
    if start >= end || end > len {
        return Err("role span outside original tape".into());
    }
    Ok(Some(RoleSpan { start, end }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [artifact, treatment_sha, blind, public, roles, out] = args.as_slice() else {
        return Err(
            "project-role LEGACY_PAIRED TREATMENT_SHA BLIND PUBLIC_SELECTION DEVICE_ROLES NEW_SELECTION".into(),
        );
    };
    let model = IntegerDiscreteModel::load(Path::new(artifact))?;
    let blind_bytes = fs::read(blind)?;
    let blind_sha = format!("{:x}", Sha256::digest(&blind_bytes));
    let prompts: Vec<Value> = serde_json::from_slice(&blind_bytes)?;
    let mut public: Value = from_reader(File::open(public)?)?;
    let device: Value = from_reader(File::open(roles)?)?;
    let selections = rows(&public)?;
    let decisions = rows(&device)?;
    if device["schema"] != "uor-r4.requested-role-device/1"
        || device["policy"] != "assertion_agreement"
        || device["artifact"] != treatment_sha.as_str()
        || device["blind"] != blind_sha
        || public["schema"] != "uor-r4.ordered-role.selections/1"
        || public["arm"] != "public"
        || public["blind"] != blind_sha
        || public["artifact"] != model.artifact_sha256()
        || prompts.len() != selections.len()
        || prompts.len() != decisions.len()
    {
        return Err("inconsistent role projection inputs".into());
    }
    let mut projected = Vec::with_capacity(prompts.len());
    for (index, ((prompt, prior), decision)) in
        prompts.iter().zip(selections).zip(decisions).enumerate()
    {
        let ids: Vec<u32> = serde_json::from_value(prompt["prefix"].clone())?;
        let words: [u32; 8] = serde_json::from_value(decision["role_words"].clone())?;
        let mut prefix = Sha256::new();
        for id in &ids {
            prefix.update(id.to_le_bytes());
        }
        let prefix_sha = format!("{:x}", prefix.finalize());
        if prior["index"] != index
            || prior["region"] != json!([0, ids.len()])
            || prior["prefix_sha256"] != prefix_sha
            || decision["index"] != index
            || decision["prefix_sha256"] != prefix_sha
        {
            return Err(format!("misordered projection at row {index}").into());
        }
        let roles = PairedRoleDecision {
            requested: span(&words[..2], ids.len())?,
            excluded: span(&words[2..4], ids.len())?,
            requested_score: words[4] as i32,
            excluded_score: words[5] as i32,
            requested_runner_up: words[6] as i32,
            excluded_runner_up: words[7] as i32,
        };
        let (picked, assertions) = model.paired_copy_witness(&ids, roles, None)?;
        if picked.role_words != Some(words) {
            return Err(format!("legacy role projection changed words at row {index}").into());
        }
        let mut row = prior.clone();
        row["original_role_words"] = json!(words);
        row["paired"] = to_value(picked)?;
        row["assertions"] = to_value(assertions)?;
        projected.push(row);
    }
    public["rows"] = json!(projected);
    to_writer(File::create_new(out)?, &public)?;
    Ok(())
}
