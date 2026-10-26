//! CLI runner for `m1-trunk-assembly-and-training` (`#2844`):
//! - `step0`: runs `G-T1`..`G-T5` GPU and algebraic trunk instrument controls
//! - `arm`: runs full Stage 3 trunk assembly, 1,049,856-row TRAIN sufficient-statistic fitting,
//!   D11 integer artifact export, and five-level + ablation evaluation on `ft/dev` and `fresh_v2`
//! - `grade`: grades the two-seed `T16_fz` and `T16_rel` trunk reports against the Stage 3 bar

use std::env;
use std::path::PathBuf;
use std::process;

use uor_r4_discrete::trunk::{grade_trunk_arms, read_json, write_pinned_json, TrunkArmReport};
use uor_r4_discrete::{Error, Result};

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

fn require_arg(args: &[String], flag: &str) -> Result<String> {
    let mut iter = args.iter();
    while let Some(item) = iter.next() {
        if item == flag {
            return iter
                .next()
                .cloned()
                .ok_or_else(|| invalid(format!("missing value for {flag}")));
        }
    }
    Err(invalid(format!("missing required argument {flag}")))
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(subcmd) = args.first().map(String::as_str) else {
        return Err(invalid("usage: trunk-run <step0|arm|grade> [flags]"));
    };
    let rest = &args[1..];
    match subcmd {
        "step0" => {
            #[cfg(feature = "cuda")]
            {
                let fz_cfg = PathBuf::from(require_arg(rest, "--fz-config")?);
                let fz_ckpt = PathBuf::from(require_arg(rest, "--fz-checkpoint")?);
                let out_dir = PathBuf::from(require_arg(rest, "--out-dir")?);
                let receipt =
                    uor_r4_discrete::trunk::gpu::run_step0_controls(&fz_cfg, &fz_ckpt, &out_dir)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&receipt).map_err(|e| invalid(e.to_string()))?
                );
                Ok(())
            }
            #[cfg(not(feature = "cuda"))]
            {
                let _ = rest;
                Err(invalid("trunk-run step0 requires --features cuda"))
            }
        }
        "arm" => {
            #[cfg(feature = "cuda")]
            {
                let name = require_arg(rest, "--name")?;
                let cfg = PathBuf::from(require_arg(rest, "--config")?);
                let ckpt = PathBuf::from(require_arg(rest, "--checkpoint")?);
                let out_dir = PathBuf::from(require_arg(rest, "--out-dir")?);
                let report =
                    uor_r4_discrete::trunk::gpu::run_trunk_arm(&name, &cfg, &ckpt, &out_dir)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).map_err(|e| invalid(e.to_string()))?
                );
                Ok(())
            }
            #[cfg(not(feature = "cuda"))]
            {
                let _ = rest;
                Err(invalid("trunk-run arm requires --features cuda"))
            }
        }
        "grade" => {
            let fz_s1: TrunkArmReport =
                read_json(&PathBuf::from(require_arg(rest, "--fz-seed1")?))?;
            let fz_s2: TrunkArmReport =
                read_json(&PathBuf::from(require_arg(rest, "--fz-seed2")?))?;
            let rel_s1: TrunkArmReport =
                read_json(&PathBuf::from(require_arg(rest, "--rel-seed1")?))?;
            let rel_s2: TrunkArmReport =
                read_json(&PathBuf::from(require_arg(rest, "--rel-seed2")?))?;
            let out_path = PathBuf::from(require_arg(rest, "--out")?);
            let grade = grade_trunk_arms(&fz_s1, &fz_s2, &rel_s1, &rel_s2)?;
            write_pinned_json(&out_path, &grade)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&grade).map_err(|e| invalid(e.to_string()))?
            );
            Ok(())
        }
        other => Err(invalid(format!("unknown trunk-run subcommand: {other}"))),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("trunk-run error: {err}");
        process::exit(1);
    }
}
