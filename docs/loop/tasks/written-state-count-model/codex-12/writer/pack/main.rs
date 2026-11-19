//! Package the device-trained bank with the original tokenizer-bound D11 writer.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io::Write};
use uor_r4_integer::{
    discrete_d11::{IntegerDiscreteModel, DISCRETE_PAIRED_SCHEMA, DISCRETE_ROLE_SCHEMA, MAGIC},
    paired_role::PairedRoleState,
};

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn main() -> Result<(), Box<dyn Error>> {
    let paths: Vec<_> = env::args_os().skip(1).collect();
    let [carrier, learned, support, train, grade, dest] = paths.as_slice() else {
        return Err("pack-paired-copy ROLE.d11 ROLE.bank LEARNED-DELIMITERS.json TRAIN-blind.json TRAIN-grade.json NEW.d11".into());
    };
    let previous = fs::read(carrier)?;
    let model = IntegerDiscreteModel::from_bytes(&previous)?;
    if model.artifact_schema() != DISCRETE_ROLE_SCHEMA {
        return Err("expected original whole-word role D11 carrier".into());
    }
    let n = usize::try_from(u64::from_le_bytes(previous[8..16].try_into()?))? + 16;
    let mut header: Value = serde_json::from_slice(&previous[16..n])?;
    let trunk = &previous[n.next_multiple_of(64)..];
    let raw = fs::read(learned)?;
    let bank = PairedRoleState::from_bytes(&raw)?;
    let delimiter_json = fs::read(support)?;
    let source: Value = serde_json::from_slice(&delimiter_json)?;
    let provenance = [
        digest(trunk),
        model
            .tokenizer_sha256()
            .ok_or("unbound tokenizer")?
            .to_owned(),
        digest(&fs::read(train)?),
        digest(&fs::read(grade)?),
    ];
    for (raw_hash, required) in bank.bindings().iter().zip(&provenance) {
        let observed: String = raw_hash.iter().map(|b| format!("{b:02x}")).collect();
        if &observed != required {
            return Err("mismatched role fit, TRAIN split or carrier".into());
        }
    }
    if source["model"].as_str() != Some(&provenance[0])
        || source["token"].as_str() != Some(&provenance[1])
    {
        return Err("wrong source for TRAIN fact delimiters".into());
    }
    header["schema"] = DISCRETE_PAIRED_SCHEMA.into();
    header["paired"] = json!({"bank_bytes":raw.len(),"bank_sha256":digest(&raw),"delimiters":source["delimiters"]});
    let encoded = serde_json::to_vec(&header)?;
    let mut file = MAGIC.to_vec();
    file.extend_from_slice(&(encoded.len() as u64).to_le_bytes());
    file.extend(encoded);
    file.resize(file.len().next_multiple_of(64), 0);
    file.extend_from_slice(&raw);
    file.extend_from_slice(trunk);
    let check = IntegerDiscreteModel::from_bytes(&file)?;
    if check.artifact_schema() != DISCRETE_PAIRED_SCHEMA
        || check.tokenizer_sha256() != model.tokenizer_sha256()
    {
        return Err("paired carrier reload mismatch".into());
    }
    let mut out = fs::File::options()
        .create_new(true)
        .write(true)
        .open(dest)?;
    out.write_all(&file)?;
    out.sync_all()?;
    println!(
        "{}",
        json!({"schema":check.artifact_schema(),"sha256":check.artifact_sha256(),"bytes":file.len(),
        "role_parent":model.artifact_sha256(),"bank":digest(&raw),"delimiters_input":digest(&delimiter_json),
        "base":provenance[0],"tokenizer":provenance[1],"train":provenance[2],"train_grade":provenance[3]})
    );
    Ok(())
}
