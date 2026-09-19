//! Measured **complete training-step cost** for the order-2 geometric addressed-memory core.
//!
//! # Why this exists
//!
//! The recorded plan carried a ~1,800 s projection for "2,000 steps at `V=4096`, `dv=128`, batch 32,
//! 64-token windows", and a later arithmetic estimate that it was 3–10× low. Neither was measured.
//! #973 requires the run length to be chosen from an actual timing, so this times complete steps at
//! the proposed configuration and reports the projection it implies.
//!
//! # What a "complete step" includes
//!
//! `GeometricAttentionTrainer::train_batch` is timed as the unit, which covers: both full-table
//! quantizations (`wv` and `wo`, each `vocab × dv`), the touched-bucket reset (not a dense state
//! clear), the forward writes and reads, the backward pass, and the Adam update. The first call is
//! reported as **cold** because it also allocates the `scratch`/`dscratch` buffers
//! (`n_addr × dv` f32 each). A `to_core()` quantization and one held-out `sequence_loss` pass are
//! timed separately as the checkpoint and evaluation overhead.
//!
//! # What it is not
//!
//! A timing on one named host, not a portable figure. Peak RSS is measured by the caller
//! (`/usr/bin/time -l`), not asserted here. Nothing is trained to convergence and no capability is
//! claimed.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use uor_r4_core::native_geometric::learner::geometric_attention::GeometricAttentionTrainer;
use uor_r4_core::transformerless::bpe_derive::derive_tokenizer;

struct Args {
    tokenizer: PathBuf,
    vocab: usize,
    corpus: PathBuf,
    window: usize,
    batch: usize,
    steps: usize,
    dv: usize,
    norm_bits: u32,
    max_bytes: usize,
    seed: u64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            tokenizer: PathBuf::from(
                "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json",
            ),
            vocab: 4096,
            corpus: PathBuf::from("/Users/casey.allard/uor-r4/.worktrees/realtext-qual/docs"),
            window: 64,
            batch: 8,
            steps: 6,
            dv: 128,
            norm_bits: 6,
            max_bytes: 2 * 1024 * 1024,
            seed: 2026_0919,
        }
    }
}

fn usage(program: &str) {
    eprintln!(
        "Usage: {program} [OPTIONS]\n\
         \x20 --tokenizer <path>  source tokenizer.json\n\
         \x20 --vocab <n>         derived vocabulary size (default 4096)\n\
         \x20 --corpus <path>     file or directory of real text\n\
         \x20 --window <n>        tokens per training window/sequence (default 64)\n\
         \x20 --batch <n>         sequences per step (default 8)\n\
         \x20 --steps <n>         steps to time (default 6)\n\
         \x20 --dv <n>            value width (default 128)\n\
         \x20 --max-bytes <n>     corpus byte budget (default 2 MiB)\n\
         \x20 --seed <n>          init seed\n\
         \x20 --help"
    );
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args::default();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let key = argv[i].as_str();
        let val = || -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("{key} needs a value"))
        };
        match key {
            "--tokenizer" => a.tokenizer = PathBuf::from(val()?),
            "--vocab" => a.vocab = val()?.parse().map_err(|e| format!("{e}"))?,
            "--corpus" => a.corpus = PathBuf::from(val()?),
            "--window" => a.window = val()?.parse().map_err(|e| format!("{e}"))?,
            "--batch" => a.batch = val()?.parse().map_err(|e| format!("{e}"))?,
            "--steps" => a.steps = val()?.parse().map_err(|e| format!("{e}"))?,
            "--dv" => a.dv = val()?.parse().map_err(|e| format!("{e}"))?,
            "--max-bytes" => a.max_bytes = val()?.parse().map_err(|e| format!("{e}"))?,
            "--seed" => a.seed = val()?.parse().map_err(|e| format!("{e}"))?,
            "--help" | "-h" => return Err("help".into()),
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    Ok(a)
}

fn collect(path: &Path, budget: &mut usize, out: &mut Vec<String>) {
    if *budget == 0 {
        return;
    }
    let Ok(meta) = std::fs::metadata(path) else {
        return;
    };
    if meta.is_file() {
        let keep = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| matches!(e, "rs" | "md" | "txt"));
        if !keep {
            return;
        }
        if let Ok(text) = std::fs::read_to_string(path) {
            if text.len() > *budget {
                *budget = 0;
            } else {
                *budget -= text.len();
                out.push(text);
            }
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    children.sort();
    for child in children {
        if *budget == 0 {
            return;
        }
        if child
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| matches!(n, "target" | ".git" | ".worktrees"))
        {
            continue;
        }
        collect(&child, budget, out);
    }
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            if e != "help" {
                eprintln!("error: {e}");
            }
            usage(&std::env::args().next().unwrap_or_default());
            return ExitCode::from(2);
        }
    };

    println!("=== complete training-step timing (order 2) ===");
    println!(
        "config: V={} dv={} order=2 norm_bits={} window={} batch={} steps={}",
        args.vocab, args.dv, args.norm_bits, args.window, args.batch, args.steps
    );

    let original = match std::fs::read(&args.tokenizer) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read tokenizer: {e}");
            return ExitCode::from(1);
        }
    };
    let tokenizer = match derive_tokenizer(&original, args.vocab) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: derive tokenizer: {e}");
            return ExitCode::from(1);
        }
    };
    let vocab = tokenizer.vocab_size();

    let mut budget = args.max_bytes;
    let mut docs: Vec<String> = Vec::new();
    collect(&args.corpus, &mut budget, &mut docs);
    let mut windows: Vec<Vec<u32>> = Vec::new();
    for d in &docs {
        let toks = tokenizer.encode(d);
        for chunk in toks.chunks(args.window) {
            if chunk.len() == args.window {
                windows.push(chunk.to_vec());
            }
        }
    }
    if windows.len() < args.batch + 1 {
        eprintln!(
            "error: only {} full windows of {} tokens; need at least {}",
            windows.len(),
            args.window,
            args.batch + 1
        );
        return ExitCode::from(1);
    }
    let held = windows.pop().expect("non-empty");
    println!(
        "corpus: documents={} full_windows={} (held-out 1)",
        docs.len(),
        windows.len() + 1
    );

    let mut trainer =
        match GeometricAttentionTrainer::new(vocab, args.dv, 2, args.norm_bits, args.seed) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: build trainer: {e}");
                return ExitCode::from(1);
            }
        };

    let mut samples: Vec<(f64, bool)> = Vec::new();
    for step in 0..args.steps {
        let batch: Vec<Vec<u32>> = (0..args.batch)
            .map(|k| windows[(step * args.batch + k) % windows.len()].clone())
            .collect();
        let t0 = Instant::now();
        let loss = trainer.train_batch(&batch);
        let secs = t0.elapsed().as_secs_f64();
        let cold = step == 0;
        println!(
            "  step {:>2} {:>8}  {:.4} s   loss {loss:.4}{}",
            step + 1,
            if cold { "(cold)" } else { "(warm)" },
            secs,
            if cold {
                "   [includes scratch allocation]"
            } else {
                ""
            }
        );
        samples.push((secs, cold));
    }

    // Evaluation + checkpoint overhead, measured separately.
    let t_eval = Instant::now();
    let quantised = match trainer.to_core() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: to_core: {e}");
            return ExitCode::from(1);
        }
    };
    let quantise_s = t_eval.elapsed().as_secs_f64();
    let t_loss = Instant::now();
    let held_loss = quantised.sequence_loss(&held);
    let eval_s = t_loss.elapsed().as_secs_f64();

    let warm: Vec<f64> = samples
        .iter()
        .filter(|(_, cold)| !*cold)
        .map(|(s, _)| *s)
        .collect();
    let warm_mean = if warm.is_empty() {
        samples[0].0
    } else {
        warm.iter().sum::<f64>() / warm.len() as f64
    };
    let tokens_per_step = args.batch * args.window;

    println!();
    println!(
        "cold step            : {:.4} s (includes one-time scratch allocation)",
        samples[0].0
    );
    println!("warm step mean       : {warm_mean:.4} s");
    println!(
        "warm step range      : {:.4} .. {:.4} s",
        warm.iter().cloned().fold(f64::INFINITY, f64::min),
        warm.iter().cloned().fold(0.0f64, f64::max)
    );
    println!(
        "tokens per step      : {tokens_per_step} (batch {} x window {})",
        args.batch, args.window
    );
    println!(
        "throughput (warm)    : {:.1} tokens/s",
        tokens_per_step as f64 / warm_mean
    );
    println!("to_core() quantise   : {quantise_s:.4} s   held-out loss {held_loss:.4}");
    println!(
        "one eval pass        : {eval_s:.4} s ({} tokens)",
        held.len()
    );
    println!();
    println!("projection from this timing (warm step mean, arithmetic only):");
    for steps in [1000usize, 2000, 4000] {
        let secs = warm_mean * steps as f64;
        println!(
            "  {steps:>5} steps x {tokens_per_step} tokens = {:>9} tokens : {:>7.1} s = {:.2} h",
            steps * tokens_per_step,
            secs,
            secs / 3600.0
        );
    }
    println!();
    println!(
        "NOTE: one named host, one configuration, no convergence. Peak RSS is reported by the caller."
    );
    ExitCode::SUCCESS
}
