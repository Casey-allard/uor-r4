//! claude-387: does a saved vector read's payload reach the decoded answer?
//!
//! usage: recall-payload-probe --checkpoint DIR --run DIR --out FILE [--scales 1,4,16,64]
//!        [--first-only]
//!
//! `--checkpoint` is a two-class recall run's `models/step-*` directory (`model.safetensors`,
//! `vector_read.safetensors`); `--run` is that run's directory, whose `config.json` gives the
//! seed and trunk flags and whose `recall.json` gives the form's header and held-out count. The
//! probe rebuilds the held-out windows from the seed and writes one JSON file: at each held-out
//! query row `|x|`, `|p|` and their cosine where the read adds, the read's kept picks against the
//! asked fact, and the asked token's decoded margin, with the payload times each scale and with
//! the read cut. With `--first-only` (claude-388) every pass with the read copies Flock's first
//! kept pick alone (the hard copy) instead of its picks' mean. Eval only, on the GPU (the CPU
//! backend refuses model compute, D24/D29).

use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_float_train::in_space_long::{recall_payload_probe, PayloadProbeSettings};

struct Args {
    checkpoint: PathBuf,
    run: PathBuf,
    out: PathBuf,
    scales: Vec<f32>,
    first_only: bool,
}

fn parse(mut argv: impl Iterator<Item = String>) -> Result<Args, String> {
    let (mut checkpoint, mut run, mut out) = (None, None, None);
    let mut scales = vec![1.0, 4.0, 16.0, 64.0];
    let mut first_only = false;
    while let Some(flag) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--first-only" => first_only = true,
            "--checkpoint" => checkpoint = Some(PathBuf::from(value()?)),
            "--run" => run = Some(PathBuf::from(value()?)),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--scales" => {
                scales = value()?
                    .split(',')
                    .map(|s| s.parse::<f32>().map_err(|e| format!("--scales {s}: {e}")))
                    .collect::<Result<_, _>>()?;
            }
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok(Args {
        checkpoint: checkpoint.ok_or("--checkpoint is required")?,
        run: run.ok_or("--run is required")?,
        out: out.ok_or("--out is required")?,
        scales,
        first_only,
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// The probe's settings from the run's `config.json` and `recall.json`; a two-class run only.
fn settings(
    run: &Path,
    scales: Vec<f32>,
    first_only: bool,
) -> Result<PayloadProbeSettings, String> {
    let config = read_json(&run.join("config.json"))?;
    let recall = read_json(&run.join("recall.json"))?;
    let flag = |name: &str| -> Result<bool, String> {
        config[name]
            .as_bool()
            .ok_or_else(|| format!("config.json has no boolean {name}"))
    };
    let form = &recall["form"];
    if form["query"].as_bool() != Some(true) {
        return Err("recall.json is not a two-class (query) run".to_owned());
    }
    Ok(PayloadProbeSettings {
        seed: config["seed"].as_u64().ok_or("config.json has no seed")?,
        retain_rates: flag("retain_rates")?,
        fold_scores: flag("fold_scores")?,
        reflect_floor: flag("reflect_floor")?,
        fix_input_table: flag("fix_input_table")?,
        header: form["header"].as_u64().ok_or("recall.json has no header")? as usize,
        eval_windows: form["eval_windows"]
            .as_u64()
            .ok_or("recall.json has no eval_windows")?,
        scales,
        first_only,
    })
}

fn main() {
    if let Err(e) = run() {
        eprintln!("recall-payload-probe: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse(std::env::args().skip(1))?;
    if args.out.exists() {
        return Err(format!(
            "{} exists; the probe never writes over one",
            args.out.display()
        ));
    }
    let settings = settings(&args.run, args.scales, args.first_only)?;
    let device = candle_core::Device::new_cuda(0).map_err(|e| e.to_string())?;
    let probe =
        recall_payload_probe(&args.checkpoint, &settings, &device).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&probe).map_err(|e| e.to_string())?;
    fs::write(&args.out, bytes).map_err(|e| format!("{}: {e}", args.out.display()))?;
    println!("wrote {}", args.out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &[&str]) -> impl Iterator<Item = String> {
        s.iter()
            .map(|a| (*a).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn the_flags_parse_with_the_default_scales() {
        let a = parse(argv(&["--checkpoint", "c", "--run", "r", "--out", "o"])).unwrap();
        assert_eq!(a.checkpoint, PathBuf::from("c"));
        assert_eq!(a.run, PathBuf::from("r"));
        assert_eq!(a.out, PathBuf::from("o"));
        assert_eq!(a.scales, vec![1.0, 4.0, 16.0, 64.0]);
        assert!(!a.first_only);
        let c = parse(argv(&[
            "--checkpoint",
            "c",
            "--first-only",
            "--run",
            "r",
            "--out",
            "o",
        ]))
        .unwrap();
        assert!(c.first_only);
        assert_eq!(c.run, PathBuf::from("r"));
        let b = parse(argv(&[
            "--checkpoint",
            "c",
            "--run",
            "r",
            "--out",
            "o",
            "--scales",
            "1,8",
        ]))
        .unwrap();
        assert_eq!(b.scales, vec![1.0, 8.0]);
        assert!(parse(argv(&["--run", "r", "--out", "o"])).is_err());
        assert!(parse(argv(&[
            "--checkpoint",
            "c",
            "--run",
            "r",
            "--out",
            "o",
            "--x"
        ]))
        .is_err());
        assert!(parse(argv(&["--checkpoint"])).is_err());
    }
}
