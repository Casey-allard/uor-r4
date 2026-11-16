//! claude-387: does a saved vector read's payload reach the decoded answer?
//!
//! usage: recall-payload-probe --checkpoint DIR --run DIR --out FILE [--scales 1,4,16,64]
//!        [--first-only [--native-replay]]
//!
//! `--checkpoint` is a two-class recall run's `models/step-*` directory (`model.safetensors`,
//! `vector_read.safetensors`); `--run` is that run's directory, whose `config.json` gives the
//! seed and trunk flags and whose `recall.json` gives the form's header and held-out count. The
//! probe rebuilds the held-out windows from the seed and writes one JSON file: at each held-out
//! query row `|x|`, `|p|` and their cosine where the read adds, the read's kept picks against the
//! asked fact, and the asked token's decoded margin, with the payload times each scale and with
//! the read cut. With `--first-only` (claude-388) every pass with the read copies Flock's first
//! kept pick alone (the hard copy) instead of its picks' mean. With `--native-replay` (claude-389,
//! needs `--first-only`) one more hard-copy pass at payload scale 1 captures the read's inputs and
//! replays them through the native Q31 read; the result is the JSON's `native_replay`. Eval only, on
//! the GPU (the CPU backend refuses model compute, D24/D29).

use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_float_train::in_space_long::{recall_payload_probe, PayloadProbeSettings};

struct Args {
    checkpoint: PathBuf,
    run: PathBuf,
    out: PathBuf,
    scales: Vec<f32>,
    first_only: bool,
    native_replay: bool,
    native_dump: Option<PathBuf>,
    native_train: Option<u64>,
    native_picks: Option<PathBuf>,
}

fn parse(mut argv: impl Iterator<Item = String>) -> Result<Args, String> {
    let (mut checkpoint, mut run, mut out) = (None, None, None);
    let mut scales = vec![1.0, 4.0, 16.0, 64.0];
    let mut first_only = false;
    let mut native_replay = false;
    let mut native_dump = None;
    let mut native_train = None;
    let mut native_picks = None;
    while let Some(flag) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--first-only" => first_only = true,
            "--native-replay" => native_replay = true,
            "--native-dump" => native_dump = Some(PathBuf::from(value()?)),
            "--native-train" => {
                let v = value()?;
                let n = v
                    .parse::<u64>()
                    .map_err(|e| format!("--native-train {v}: {e}"))?;
                native_train = Some(n);
            }
            "--native-picks" => native_picks = Some(PathBuf::from(value()?)),
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
    if native_replay && !first_only {
        return Err(
            "--native-replay needs --first-only: the sheet replays the hard copy".to_owned(),
        );
    }
    if native_dump.is_some() && !native_replay {
        return Err("--native-dump needs --native-replay: it writes the replay's input".to_owned());
    }
    if native_train.is_some() && native_dump.is_none() {
        return Err("--native-train needs --native-dump: it only makes the dump".to_owned());
    }
    if native_train == Some(0) {
        return Err("--native-train reads at least one window".to_owned());
    }
    if native_picks.is_some() && !first_only {
        return Err("--native-picks needs --first-only: they are the hard copy's pick".to_owned());
    }
    let out: PathBuf = out.ok_or("--out is required")?;
    if native_dump.as_ref() == Some(&out) {
        return Err("--native-dump and --out name the same file".to_owned());
    }
    Ok(Args {
        checkpoint: checkpoint.ok_or("--checkpoint is required")?,
        run: run.ok_or("--run is required")?,
        out,
        scales,
        first_only,
        native_replay,
        native_dump,
        native_train,
        native_picks,
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
    native_replay: bool,
    native_dump: Option<PathBuf>,
    native_train: Option<u64>,
    native_picks: Option<PathBuf>,
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
        native_replay,
        native_dump,
        native_train_windows: native_train,
        native_picks,
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
    if let Some(dump) = args.native_dump.as_ref().filter(|d| d.exists()) {
        return Err(format!(
            "{} exists; the probe never writes over one",
            dump.display()
        ));
    }
    let settings = settings(
        &args.run,
        args.scales,
        args.first_only,
        args.native_replay,
        args.native_dump,
        args.native_train,
        args.native_picks,
    )?;
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

    #[test]
    fn the_native_replay_needs_the_hard_copy() {
        let base = ["--checkpoint", "c", "--run", "r", "--out", "o"];
        let alone: Vec<&str> = base.iter().copied().chain(["--native-replay"]).collect();
        let refused = parse(argv(&alone)).err().unwrap();
        assert!(refused.contains("--first-only"));
        let both: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--native-replay", "--first-only"])
            .collect();
        let a = parse(argv(&both)).unwrap();
        assert!(a.native_replay && a.first_only);
        assert!(!parse(argv(&base)).unwrap().native_replay);
    }

    #[test]
    fn the_native_dump_needs_the_replay() {
        let base = ["--checkpoint", "c", "--run", "r", "--out", "o", "--first-only"];
        let alone: Vec<&str> = base.iter().copied().chain(["--native-dump", "d"]).collect();
        assert!(parse(argv(&alone)).err().unwrap().contains("--native-replay"));
        let both: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--native-replay", "--native-dump", "d"])
            .collect();
        let a = parse(argv(&both)).unwrap();
        assert_eq!(a.native_dump, Some(PathBuf::from("d")));
        assert!(parse(argv(&base)).unwrap().native_dump.is_none());
        let same: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--native-replay", "--native-dump", "o"])
            .collect();
        assert!(parse(argv(&same)).err().unwrap().contains("same file"));
    }

    #[test]
    fn the_train_replay_needs_the_dump() {
        let base = [
            "--checkpoint",
            "c",
            "--run",
            "r",
            "--out",
            "o",
            "--first-only",
            "--native-replay",
        ];
        let alone: Vec<&str> = base.iter().copied().chain(["--native-train", "8"]).collect();
        assert!(parse(argv(&alone)).err().unwrap().contains("--native-dump"));
        let both: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--native-dump", "d", "--native-train", "2048"])
            .collect();
        assert_eq!(parse(argv(&both)).unwrap().native_train, Some(2048));
        let zero: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--native-dump", "d", "--native-train", "0"])
            .collect();
        assert!(parse(argv(&zero)).is_err());
        let bad: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--native-dump", "d", "--native-train", "x"])
            .collect();
        assert!(parse(argv(&bad)).is_err());
        assert!(parse(argv(&base)).unwrap().native_train.is_none());
    }

    #[test]
    fn the_native_picks_need_the_hard_copy() {
        let base = ["--checkpoint", "c", "--run", "r", "--out", "o"];
        let alone: Vec<&str> = base.iter().copied().chain(["--native-picks", "p"]).collect();
        assert!(parse(argv(&alone)).err().unwrap().contains("--first-only"));
        let both: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--first-only", "--native-picks", "p"])
            .collect();
        assert_eq!(parse(argv(&both)).unwrap().native_picks, Some(PathBuf::from("p")));
        assert!(parse(argv(&base)).unwrap().native_picks.is_none());
    }
}
