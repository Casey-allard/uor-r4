//! `d87-split --out <dir> [--key K] [--dev-tune N/D] [--dev-test N/D] [--near-floor B]
//!  [--group prompt|template] [--threads T] <shard>...`
//!
//! Reads D87 shards read-only and writes `<dir>/{train,dev-tune,dev-test}.ids` (sorted record
//! ids) and `<dir>/split-summary.json`. Never writes next to its inputs.

use std::path::PathBuf;
use std::process::ExitCode;

use d87_split_audit::{plan, Config, Group, Splitter};

fn ratio(s: Option<String>) -> Result<(u64, u64), String> {
    let s = s.ok_or("missing ratio")?;
    let (n, d) = s.split_once('/').ok_or(format!("ratio {s} is not N/D"))?;
    Ok((n.parse().map_err(|_| format!("ratio {s}"))?, d.parse().map_err(|_| format!("ratio {s}"))?))
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut cfg, mut out, mut threads, mut shards) = (Config::default(), None, 12usize, vec![]);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out = args.next().map(PathBuf::from),
            "--key" => cfg.key = args.next().ok_or("--key")?,
            "--dev-tune" => cfg.dev_tune = ratio(args.next())?,
            "--dev-test" => cfg.dev_test = ratio(args.next())?,
            "--group" => cfg.group = Group::parse(&args.next().ok_or("--group")?)?,
            "--near-floor" => cfg.near_floor = args.next().and_then(|s| s.parse().ok()).ok_or("--near-floor")?,
            "--threads" => threads = args.next().and_then(|s| s.parse().ok()).ok_or("--threads")?,
            s if s.starts_with("--") => return Err(format!("unknown flag {s}")),
            s => shards.push(PathBuf::from(s)),
        }
    }
    let out = out.ok_or("--out is required")?;
    if shards.is_empty() {
        return Err("no shards".into());
    }
    let sp = Splitter::new(cfg)?;
    let p = plan(&sp, &shards, threads)?;
    p.write(&out)?;
    let s = p.summary();
    eprintln!("d87-split: {} lines, {} records, {} prompts kept; dropped {}; written to {}",
        s["lines"], s["records"], s["prompts"], s["dropped"], out.display());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("d87-split: {e}");
            ExitCode::FAILURE
        }
    }
}
