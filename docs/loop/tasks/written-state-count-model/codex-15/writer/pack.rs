use super::{hash, save, Error};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};
use uor_r4_integer::{
    discrete_d11::{pack_ordered_role_artifact, IntegerDiscreteModel},
    ordered_role::RoleBank,
};
pub fn pack(args: &[String]) -> Result<(), Error> {
    let [paired, tokenizer, weights, receipt, out] = args else {
        return Err("pack PAIRED TOKENIZER GPU_BANK TRAIN_RECEIPT NEW_MODEL".into());
    };
    let record = fs::read(weights)?;
    let bank = RoleBank::from_bytes(&record)?;
    let digest = Sha256::digest(fs::read(receipt)?).into();
    let bytes =
        pack_ordered_role_artifact(&fs::read(paired)?, &fs::read(tokenizer)?, &bank, digest)?;
    let mut file = fs::File::create_new(out)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    let restored = IntegerDiscreteModel::load(Path::new(out))?;
    if restored.ordered_role_bank().map(RoleBank::to_bytes) != Some(bank.to_bytes()) {
        return Err("packed model changed trained bank".into());
    }
    save(
        &Path::new(out).with_extension("receipt.json"),
        &json!({"base":hash(Path::new(paired))?,"tokenizer":hash(Path::new(tokenizer))?,"bank":hash(Path::new(weights))?,"train_receipt":hash(Path::new(receipt))?,"artifact":hash(Path::new(out))?,"size":bytes.len(),"bank_bytes":record.len()}),
    )
}
