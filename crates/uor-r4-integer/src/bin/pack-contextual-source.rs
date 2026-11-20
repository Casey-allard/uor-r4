use std::{env, fs, io::Write};
use uor_r4_integer::discrete_d11::{
    contextual_source::{pack_contextual_source_artifact, SourceArm},
    pair_rank, IntegerDiscreteModel,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    const USAGE: &str = "usage: pack-contextual-source PARENT SOURCE_BANK TRAIN_SHA256 ARM OUT\nARM: narrow_local, wide_local, narrow_contextual, wide_contextual, wide_recurrence, pair_rank, or pair_rank_candidate";
    let [parent, bank, train, arm, output] = args.as_slice() else {
        return Err(USAGE.into());
    };
    let train = hex::decode(train.to_str().ok_or(USAGE)?)?
        .try_into()
        .map_err(|_| "TRAIN_SHA256 must decode into 32 bytes")?;
    let parent = fs::read(parent)?;
    let bank = fs::read(bank)?;
    let bytes = match arm.to_str() {
        Some("pair_rank") => {
            pair_rank::pack_artifact(&parent, &bank, train, pair_rank::Geometry::FactPrefix)?
        }
        Some("pair_rank_candidate") => {
            pair_rank::pack_artifact(&parent, &bank, train, pair_rank::Geometry::CandidatePrefix)?
        }
        arm => {
            let arm = match arm {
                Some("narrow_local") => SourceArm::NarrowLocal,
                Some("wide_local") => SourceArm::WideLocal,
                Some("narrow_contextual") => SourceArm::NarrowContextual,
                Some("wide_contextual") => SourceArm::WideContextual,
                Some("wide_recurrence") => SourceArm::WideRecurrence,
                _ => return Err(USAGE.into()),
            };
            pack_contextual_source_artifact(&parent, &bank, train, arm)?
        }
    };
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
