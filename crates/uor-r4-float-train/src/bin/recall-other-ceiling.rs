//! claude-397: can a two-class query's other fact be found from the stream at all?
//!
//! usage: recall-other-ceiling --seed S --header H --windows N (--held | --train) --out FILE
//!        [--targets DUMP] [--picks NAME=FILE ...]
//!
//! Rebuilds the windows `recall_query_window` draws (`--held`: the probe's held-out windows;
//! `--train`: the first N TRAIN windows), and at every query counts the facts consistent with
//! being its other fact (`recall_other_candidates`). With `--targets`, the run's `.ask.u32`,
//! `.other.u32` and `.bucket.u8` beside that dump must equal the rebuilt windows. Each `--picks`
//! file (one u32 per row) is classed at every scored row. Writes one JSON file of integer counts.
//! A host count: no model and no device.

use std::fs;
use std::path::PathBuf;

use uor_r4_float_train::in_space_long::{recall_other_ceiling, OtherCeilingSettings};

struct Args {
    settings: OtherCeilingSettings,
    out: PathBuf,
}

fn parse(mut argv: impl Iterator<Item = String>) -> Result<Args, String> {
    let (mut seed, mut header, mut windows, mut out) = (None, None, None, None);
    let (mut held, mut train, mut targets, mut picks) = (false, false, None, Vec::new());
    while let Some(flag) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{flag} needs a value"));
        let number = |v: String| v.parse::<u64>().map_err(|e| format!("{flag} {v}: {e}"));
        match flag.as_str() {
            "--seed" => seed = Some(number(value()?)?),
            "--header" => header = Some(number(value()?)? as usize),
            "--windows" => windows = Some(number(value()?)?),
            "--held" => held = true,
            "--train" => train = true,
            "--targets" => targets = Some(PathBuf::from(value()?)),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--picks" => {
                let v = value()?;
                let (name, path) = v
                    .split_once('=')
                    .ok_or_else(|| format!("--picks {v}: expected NAME=FILE"))?;
                picks.push((name.to_owned(), PathBuf::from(path)));
            }
            other => return Err(format!("unknown flag {other}")),
        }
    }
    if held == train {
        return Err("give exactly one of --held and --train".to_owned());
    }
    let windows = windows.ok_or("--windows is required")?;
    if windows == 0 {
        return Err("--windows reads at least one window".to_owned());
    }
    Ok(Args {
        settings: OtherCeilingSettings {
            seed: seed.ok_or("--seed is required")?,
            header: header.ok_or("--header is required")?,
            windows,
            held,
            targets,
            picks,
        },
        out: out.ok_or("--out is required")?,
    })
}

fn main() {
    if let Err(e) = run() {
        eprintln!("recall-other-ceiling: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse(std::env::args().skip(1))?;
    if args.out.exists() {
        return Err(format!("{} exists; never written over", args.out.display()));
    }
    let report = recall_other_ceiling(&args.settings).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    fs::File::create_new(&args.out)
        .and_then(|mut f| std::io::Write::write_all(&mut f, &bytes))
        .map_err(|e| format!("{}: {e}", args.out.display()))?;
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
    fn the_flags_parse_and_refuse() {
        let base = [
            "--seed",
            "1",
            "--header",
            "4",
            "--windows",
            "8",
            "--out",
            "o",
        ];
        let held: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--held", "--picks", "a=p"])
            .collect();
        let a = parse(argv(&held)).unwrap();
        assert!(a.settings.held && a.settings.windows == 8 && a.settings.header == 4);
        assert_eq!(a.settings.picks, vec![("a".to_owned(), PathBuf::from("p"))]);
        assert!(parse(argv(&base)).is_err(), "neither split");
        let both: Vec<&str> = base.iter().copied().chain(["--held", "--train"]).collect();
        assert!(parse(argv(&both)).is_err(), "both splits");
        let bad: Vec<&str> = base
            .iter()
            .copied()
            .chain(["--held", "--picks", "p"])
            .collect();
        assert!(parse(argv(&bad)).is_err(), "a pick file with no name");
        let zero = [
            "--seed",
            "1",
            "--header",
            "4",
            "--windows",
            "0",
            "--out",
            "o",
            "--held",
        ];
        assert!(parse(argv(&zero)).is_err());
    }
}
