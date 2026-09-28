//! Same-Input Stepping Runner for M1 Energy and Efficiency Benchmarks
//!
//! Steps through a fixed token file with sliding full256 context.
//! Executes zero stop-token logic and zero sampling so both Integer and
//! continuous floating-point (FF) models do identical work.
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use sha2::{Digest, Sha256};
use uor_r4_integer::bundle::Bundle;
use uor_r4_integer::ReadMode as IntReadMode;
use uor_r4_training::joint_model::{JointModel, ReadMode as JointReadMode};

#[derive(Debug, PartialEq, Eq)]
enum ModelKind {
    Integer,
    ContinuousFf,
}

fn read_u16_tokens(path: &Path) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(format!("{} is not a u16 token file (odd length)", path.display()).into());
    }
    let tokens: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    if tokens.is_empty() {
        return Err(format!("{} contains zero tokens", path.display()).into());
    }
    Ok(tokens)
}

fn resolve_tokens_file(custom: Option<PathBuf>) -> PathBuf {
    if let Some(p) = custom {
        if p.exists() {
            return p;
        }
    }
    let default_1 =
        PathBuf::from("/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/tokens/dev.u16");
    if default_1.exists() {
        return default_1;
    }
    let default_2 = PathBuf::from("/Volumes/UOR-Workspace/Backups/language-continuation-20260926-1/canonical/.uor-models/research/issue-1017/tokens/dev.u16");
    if default_2.exists() {
        return default_2;
    }
    PathBuf::from("dev.u16")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model_kind = None;
    let mut bundle_dir = PathBuf::from(
        "/Users/casey.allard/uor-r4/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1",
    );
    if !bundle_dir.exists() {
        bundle_dir = PathBuf::from(
            "/Volumes/UOR-Workspace/Backups/language-continuation-20260926-1/canonical/.uor-models/investigations/integer-serving-20260925/bundle-quaternion-1",
        );
    }
    let mut checkpoint_dir = PathBuf::from(
        "/Users/casey.allard/uor-r4-investigations/joint-recurrent-20260924/fit256-quaternion-3/checkpoint-final",
    );
    let mut tokens_file = None;
    let mut total_steps = 4096usize;
    let mut read_enabled = true;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--model-type" => {
                idx += 1;
                match args[idx].as_str() {
                    "integer" => model_kind = Some(ModelKind::Integer),
                    "continuous-ff" | "ff" => model_kind = Some(ModelKind::ContinuousFf),
                    other => {
                        eprintln!(
                            "Unknown model-type: {other} (expected 'integer' or 'continuous-ff')"
                        );
                        std::process::exit(1);
                    }
                }
            }
            "--bundle" => {
                idx += 1;
                bundle_dir = PathBuf::from(&args[idx]);
                if model_kind.is_none() {
                    model_kind = Some(ModelKind::Integer);
                }
            }
            "--checkpoint" => {
                idx += 1;
                checkpoint_dir = PathBuf::from(&args[idx]);
                if model_kind.is_none() {
                    model_kind = Some(ModelKind::ContinuousFf);
                }
            }
            "--tokens-file" => {
                idx += 1;
                tokens_file = Some(PathBuf::from(&args[idx]));
            }
            "-n" | "--tokens" | "--steps" => {
                idx += 1;
                total_steps = args[idx].parse()?;
            }
            "--read-mode" => {
                idx += 1;
                match args[idx].to_lowercase().as_str() {
                    "enabled" | "on" => read_enabled = true,
                    "no_read" | "noread" | "off" => read_enabled = false,
                    other => {
                        eprintln!("Unknown read mode: {other}");
                        std::process::exit(1);
                    }
                }
            }
            "-h" | "--help" => {
                println!("Usage: same-input-step [OPTIONS]");
                println!("Options:");
                println!("  --model-type <integer|continuous-ff>");
                println!("  --bundle <PATH>               Integer model bundle directory");
                println!("  --checkpoint <PATH>           Continuous FF checkpoint directory");
                println!("  --tokens-file <PATH>          Path to .u16 token file");
                println!("  -n, --tokens, --steps <INT>   Total steps to execute (default: 4096)");
                println!("  --read-mode <enabled|no_read> Memory read mode (default: enabled)");
                return Ok(());
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                std::process::exit(1);
            }
        }
        idx += 1;
    }

    let resolved_tokens_file = resolve_tokens_file(tokens_file);
    let tokens = read_u16_tokens(&resolved_tokens_file)?;
    let kind = model_kind.unwrap_or(ModelKind::Integer);

    eprintln!(
        "[same-input-step] Model: {:?}, Steps: {}, Token File: {} ({} tokens available)",
        kind,
        total_steps,
        resolved_tokens_file.display(),
        tokens.len()
    );

    let start_time = Instant::now();
    let mut stepped = 0usize;
    let mut trace_hasher = Sha256::new();

    match kind {
        ModelKind::Integer => {
            let bundle = Bundle::load(&bundle_dir)?;
            let model = bundle.model();
            let context = model.config().context; // 256
            let int_mode = if read_enabled {
                IntReadMode::Enabled
            } else {
                IntReadMode::NoRead
            };

            let num_blocks = (total_steps + context - 1) / context;
            for _ in 0..num_blocks {
                let mut session = model.new_session();
                let block_tokens = (total_steps - stepped).min(context);
                for _ in 0..block_tokens {
                    let token = tokens[stepped % tokens.len()] as u32;
                    let step = model.step(&mut session, token, int_mode)?;
                    trace_hasher.update(&token.to_le_bytes());
                    for p in &step.probabilities {
                        trace_hasher.update(&p.to_le_bytes());
                    }
                    for s in &step.state {
                        trace_hasher.update(&s.to_le_bytes());
                    }
                    trace_hasher.update(&step.no_read_mass.to_le_bytes());
                    for r in &step.read_masses {
                        trace_hasher.update(&r.to_le_bytes());
                    }
                    std::hint::black_box(&step);
                    stepped += 1;
                }
            }
        }
        ModelKind::ContinuousFf => {
            let model = JointModel::load(&checkpoint_dir, &Device::Cpu)?;
            let context = model.config.context; // 256
            let joint_mode = if read_enabled {
                JointReadMode::Enabled
            } else {
                JointReadMode::NoRead
            };

            let num_blocks = (total_steps + context - 1) / context;
            for _ in 0..num_blocks {
                let mut session = model.new_session(1)?;
                let block_tokens = (total_steps - stepped).min(context);
                for _ in 0..block_tokens {
                    let token = tokens[stepped % tokens.len()] as u32;
                    let step = model.step(&mut session, &[token], joint_mode)?;
                    std::hint::black_box(&step);
                    stepped += 1;
                }
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let tok_per_sec = if elapsed > 0.0 {
        stepped as f64 / elapsed
    } else {
        0.0
    };
    let ms_per_tok = if stepped > 0 {
        (elapsed * 1000.0) / stepped as f64
    } else {
        0.0
    };

    if kind == ModelKind::Integer {
        let trace_digest = hex::encode(trace_hasher.finalize());
        println!("[trace-sha256] {trace_digest}");
    }

    println!(
        "[telemetry] Stepped {} tokens in {:.2}s ({:.1} tok/s, {:.3} ms/tok)",
        stepped, elapsed, tok_per_sec, ms_per_tok
    );

    Ok(())
}
