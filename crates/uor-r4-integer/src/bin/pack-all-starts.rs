use std::{env, fs, io::Write, path::Path};
#[cfg(feature = "serving-wasm")]
use uor_r4_integer::discrete_d11::pack_assertion_agreement_artifact;
use uor_r4_integer::discrete_d11::{pack_all_starts_artifact, IntegerDiscreteModel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    const USAGE: &str = "usage: pack-all-starts FROZEN_PAIRED SPAN_STATE NATIVE_BANK PROFILE_JSON OUT [--requested-policy assertion_agreement TOKENIZER_JSON]";
    let [carrier, span, native, profile, output, options @ ..] = args.as_slice() else {
        return Err(USAGE.into());
    };
    let profile = serde_json::from_slice(&fs::read(profile)?)?;
    let carrier = fs::read(carrier)?;
    let span = fs::read(span)?;
    let native = fs::read(native)?;
    let bytes = match options {
        [] => pack_all_starts_artifact(&carrier, &span, &native, &profile)?,
        #[cfg(feature = "serving-wasm")]
        [flag, mode, tokenizer]
            if flag == "--requested-policy" && mode == "assertion_agreement" =>
        {
            pack_assertion_agreement_artifact(
                &carrier,
                &span,
                &native,
                &profile,
                &fs::read(tokenizer)?,
            )?
        }
        _ => return Err(USAGE.into()),
    };
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
