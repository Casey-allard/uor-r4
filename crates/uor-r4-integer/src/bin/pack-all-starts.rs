use std::{env, fs, io::Write, path::Path};
use uor_r4_integer::discrete_d11::{pack_all_starts_artifact, IntegerDiscreteModel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [carrier, span, native, profile, output] = args.as_slice() else {
        return Err(
            "usage: pack-all-starts FROZEN_PAIRED SPAN_STATE NATIVE_BANK PROFILE_JSON OUT".into(),
        );
    };
    let profile = serde_json::from_slice(&fs::read(profile)?)?;
    let bytes = pack_all_starts_artifact(
        &fs::read(carrier)?,
        &fs::read(span)?,
        &fs::read(native)?,
        &profile,
    )?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(Path::new(output))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    println!("{} {}", model.artifact_schema(), model.artifact_sha256());
    Ok(())
}
