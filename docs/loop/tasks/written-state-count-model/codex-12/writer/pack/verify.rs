//! Verify every held-out source decision after reopening the single client artifact.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

fn main() -> Result<(), Box<dyn Error>> {
    let paths: Vec<_> = env::args_os().skip(1).collect();
    let [model_path, pack_path, bare_path, roles_path, trace_path] = paths.as_slice() else {
        return Err("verify-paired-client D11 PACK BARE ROLES FENCED".into());
    };
    let model = IntegerDiscreteModel::from_bytes(&fs::read(model_path)?)?;
    let load = |p| -> Result<Value, Box<dyn Error>> { Ok(serde_json::from_slice(&fs::read(p)?)?) };
    let pack = load(pack_path)?;
    let raw = fs::read(bare_path)?;
    let bare: Vec<Value> = serde_json::from_slice(&raw)?;
    let roles = load(roles_path)?;
    let trace = load(trace_path)?;
    let role_rows = roles["rows"].as_array().ok_or("missing roles")?;
    let trace_rows = trace["rows"].as_array().ok_or("missing source handoffs")?;
    let bare_hash = format!("{:x}", Sha256::digest(&raw));
    if bare.is_empty()
        || bare.len() != role_rows.len()
        || bare.len() != trace_rows.len()
        || pack["sha256"] != model.artifact_sha256()
        || roles["state_sha256"] != pack["bank"]
        || roles["tokenizer_sha256"] != pack["tokenizer"]
        || [&roles, &trace]
            .iter()
            .any(|r| r["bare_sha256"] != bare_hash || r["artifact_sha256"] != pack["base"])
    {
        return Err("artifact/tape/row identity mismatch".into());
    }
    let mut copies = 0usize;
    for (i, ((row, role), address)) in bare.iter().zip(role_rows).zip(trace_rows).enumerate() {
        let ids: Vec<u32> = serde_json::from_value(row["prefix"].clone())?;
        let bytes: Vec<u8> = ids.iter().flat_map(|id| id.to_le_bytes()).collect();
        let hash = format!("{:x}", Sha256::digest(&bytes));
        let decision = model
            .paired_copy_decision(&ids, true)?
            .ok_or("missing policy")?;
        let source = address["continuation"]
            .get("source")
            .ok_or("missing source")?;
        if role["prefix_sha256"] != hash
            || address["prefix_sha256"] != hash
            || role["words"] != json!(decision.role_words)
            || source != &json!(decision.choice.source)
        {
            return Err(format!(
                "row {i}: hash {hash}, expected role {:?} source {source}, observed {:?}",
                role["words"], decision
            )
            .into());
        }
        copies += usize::from(decision.choice.source.is_some());
    }
    println!(
        "{}",
        json!({"rows":bare.len(),"copies":copies,"matched_role_words":8*bare.len(),
                          "matched_source_decisions":bare.len(),"bare_sha256":bare_hash,"artifact_sha256":model.artifact_sha256()})
    );
    Ok(())
}
