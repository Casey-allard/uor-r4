use std::{env, fs, io::Write};
use uor_r4_integer::discrete_d11::{
    contextual_source::{pack_contextual_source_artifact, SourceArm},
    IntegerDiscreteModel,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    const USAGE: &str = "usage: pack-contextual-source D21_PARENT D25_BANK TRAIN_SHA256 ARM OUT";
    let [parent, bank, train, arm, output] = args.as_slice() else {
        return Err(USAGE.into());
    };
    let arm = match arm.to_str() {
        Some("narrow_local") => SourceArm::NarrowLocal,
        Some("wide_local") => SourceArm::WideLocal,
        Some("narrow_contextual") => SourceArm::NarrowContextual,
        Some("wide_contextual") => SourceArm::WideContextual,
        _ => return Err(USAGE.into()),
    };
    let train = hex::decode(train.to_str().ok_or(USAGE)?)?
        .try_into()
        .map_err(|_| "TRAIN_SHA256 must decode into 32 bytes")?;
    let bytes = pack_contextual_source_artifact(&fs::read(parent)?, &fs::read(bank)?, train, arm)?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    println!("{} {}", model.artifact_schema(), model.artifact_sha256());
    Ok(())
}
