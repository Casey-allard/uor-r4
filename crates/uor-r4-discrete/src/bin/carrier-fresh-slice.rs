//! Cut the sealed fresh slice for `carrier-credit-hard-selection` (#2819; prep
//! sheet §4). Host data preparation: it reads the pinned TRAIN corpus and no model.
//!
//! Usage: carrier-fresh-slice CONFIG VISITED_TARGETS FRESH_TARGETS OUT_DIR
//! OUT_DIR must not exist; it receives tokens.uort, mask.u8 and manifest.json.

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use uor_r4_discrete::carrier_fresh::cut_fresh_slice;
use uor_r4_discrete::corpus::PreparedCorpus;
use uor_r4_discrete::training::DriverConfig;

fn run(args: &[String]) -> Result<(), String> {
    let [config, visited, fresh, out] = args else {
        return Err(
            "usage: carrier-fresh-slice CONFIG VISITED_TARGETS FRESH_TARGETS OUT_DIR".into(),
        );
    };
    let config: DriverConfig =
        serde_json::from_slice(&fs::read(config).map_err(|e| format!("{config}: {e}"))?)
            .map_err(|e| format!("config: {e}"))?;
    let visited: usize = visited
        .parse()
        .map_err(|e| format!("VISITED_TARGETS: {e}"))?;
    let fresh: usize = fresh.parse().map_err(|e| format!("FRESH_TARGETS: {e}"))?;
    let out = Path::new(out);
    if out.exists() {
        return Err(format!(
            "{} exists; the slice is written once",
            out.display()
        ));
    }
    let train = PreparedCorpus::load(&config.train, config.spec.vocabulary, &config.bos)
        .map_err(|e| format!("TRAIN: {e:?}"))?;
    let slice = cut_fresh_slice(&train, config.spec.context, visited, fresh)
        .map_err(|e| format!("slice: {e:?}"))?;
    fs::create_dir(out).map_err(|e| format!("{}: {e}", out.display()))?;
    fs::write(out.join("tokens.uort"), &slice.tokens).map_err(|e| e.to_string())?;
    fs::write(out.join("mask.u8"), &slice.mask).map_err(|e| e.to_string())?;
    let manifest = serde_json::to_vec_pretty(&slice.manifest).map_err(|e| e.to_string())?;
    fs::write(out.join("manifest.json"), manifest).map_err(|e| e.to_string())?;
    println!("{}", slice.manifest);
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("carrier-fresh-slice: {message}");
            ExitCode::FAILURE
        }
    }
}
