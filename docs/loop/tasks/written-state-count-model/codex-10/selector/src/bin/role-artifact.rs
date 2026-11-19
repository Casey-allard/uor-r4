//! Build one UORLUT01 client artifact from the device-trained role-only policy.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use uor_r4_integer::discrete_d11::{IntegerDiscreteModel, DISCRETE_ROLE_SCHEMA, MAGIC};

type Error = Box<dyn std::error::Error>;

fn state(path: &Path) -> Result<(Value, String), Error> {
    let bytes = fs::read(path)?;
    Ok((
        serde_json::from_slice(&bytes)?,
        format!("{:x}", Sha256::digest(bytes)),
    ))
}

fn main() -> Result<(), Error> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [base, role, content, span, exact, dest] = args.as_slice() else {
        return Err(
            "role-artifact BASE.d11 ROLE.json CONTENT.json SPAN.json EXACT.json NEW.d11".into(),
        );
    };
    let bytes = fs::read(base)?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)?;
    let base_hash = model.artifact_sha256();
    let tokenizer = model.tokenizer_sha256().ok_or("missing bound tokenizer")?;
    let (role, role_hash) = state(role.as_ref())?;
    let (content, content_hash) = state(content.as_ref())?;
    let (span, span_hash) = state(span.as_ref())?;
    let (exact, exact_hash) = state(exact.as_ref())?;
    if role["artifact"] != base_hash
        || content["artifact_sha256"] != base_hash
        || span["artifact_sha256"] != base_hash
        || exact["artifact_sha256"] != base_hash
        || span["tokenizer_sha256"] != tokenizer
        || exact["span_state_sha256"] != span_hash
    {
        return Err("training bank identity mismatch".into());
    }
    let groups = role["groups"]
        .as_array()
        .filter(|g| g.len() == 5)
        .ok_or("expected five occurrence/boundary groups")?;
    let relation = content["prototypes"]["relation"]
        .as_array()
        .filter(|r| r.len() == 1)
        .and_then(|r| r[0].as_array())
        .filter(|r| r.len() == 16 && r[0] == 1)
        .ok_or("expected single legacy edge")?;
    let header = serde_json::to_vec(&json!({
        "schema":DISCRETE_ROLE_SCHEMA,"base_sha256":base_hash,"tokenizer_sha256":tokenizer,
        "policy":{"occurrence":&groups[..3],"span":span["bank"],"exact":exact["bank"],"legacy_root":relation[1]}
    }))?;
    let mut artifact = Vec::with_capacity(80 + header.len() + bytes.len());
    artifact.extend_from_slice(MAGIC);
    artifact.extend_from_slice(&(header.len() as u64).to_le_bytes());
    artifact.extend_from_slice(&header);
    artifact.resize(artifact.len().next_multiple_of(64), 0);
    artifact.extend_from_slice(&bytes);
    let loaded = IntegerDiscreteModel::from_bytes(&artifact)?;
    if loaded.artifact_schema() != DISCRETE_ROLE_SCHEMA
        || loaded.tokenizer_sha256() != Some(tokenizer)
    {
        return Err("pack/load mismatch".into());
    }
    let mut f = File::options().create_new(true).write(true).open(dest)?;
    f.write_all(&artifact)?;
    f.sync_all()?;
    println!(
        "{}",
        json!({"base":base_hash,"role":role_hash,"content":content_hash,"span":span_hash,
        "exact":exact_hash,"tokenizer":tokenizer,"artifact":loaded.artifact_sha256(),"bytes":artifact.len(),
        "train":role["train"]})
    );
    Ok(())
}
