//! Train the low-bit recurrent core on chat/instruction data, then read what it produces.
//!
//! This is the first end-to-end use of [`LowBitCoreTrainer`]: a tokenizer, a corpus, gradient
//! descent, and raw generations printed from the **quantised** model that would serve.
//!
//! # What it prints, and why each number is here
//!
//! * **held-out loss and top-1 next-token accuracy** under the quantised weights. Perplexity is
//!   necessary and not sufficient, so accuracy is reported beside it.
//! * **raw generations**, not a summary of them, because a sample is the only honest evidence about
//!   what a language model says. They are expected to be bad; a small core on a small corpus is not
//!   a chat model, and the point of this run is to establish that the loop learns at all.
//!
//! # Reproducibility
//!
//! Prints the exact invocation, corpus counts and seed. Any result here is reproducible from those
//! plus the corpus file; nothing is sampled from an unseeded source.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use uor_r4_core::native_geometric::learner::chat::{self, CHAT_VOCAB, TOK_END};
use uor_r4_core::native_geometric::learner::{LowBitCoreTrainer, TrainConfig};

struct Args {
    corpus: Option<PathBuf>,
    procedural: usize,
    steps: usize,
    dim: usize,
    batch: usize,
    lr: f32,
    max_seq: usize,
    held_every: usize,
    samples: usize,
    out: Option<PathBuf>,
    seed: u64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            corpus: None,
            procedural: 3000,
            steps: 600,
            dim: 96,
            batch: 16,
            lr: 0.02,
            max_seq: 72,
            held_every: 10,
            samples: 6,
            out: None,
            seed: 2026_0919,
        }
    }
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
            "--corpus" => a.corpus = Some(PathBuf::from(val()?)),
            "--procedural" => a.procedural = val()?.parse().map_err(|e| format!("{e}"))?,
            "--steps" => a.steps = val()?.parse().map_err(|e| format!("{e}"))?,
            "--dim" => a.dim = val()?.parse().map_err(|e| format!("{e}"))?,
            "--batch" => a.batch = val()?.parse().map_err(|e| format!("{e}"))?,
            "--lr" => a.lr = val()?.parse().map_err(|e| format!("{e}"))?,
            "--max-seq" => a.max_seq = val()?.parse().map_err(|e| format!("{e}"))?,
            "--held-every" => a.held_every = val()?.parse().map_err(|e| format!("{e}"))?,
            "--samples" => a.samples = val()?.parse().map_err(|e| format!("{e}"))?,
            "--out" => a.out = Some(PathBuf::from(val()?)),
            "--seed" => a.seed = val()?.parse().map_err(|e| format!("{e}"))?,
            "--help" | "-h" => return Err("help".into()),
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    Ok(a)
}

fn usage() {
    eprintln!(
        "train-lowbit-chat [--corpus FILE] [--procedural N] [--steps N] [--dim N] [--batch N]\n\
         \x20                 [--lr F] [--max-seq N] [--held-every N] [--samples N] [--out FILE]\n\
         \x20                 [--seed N]\n\
         \n\
         --corpus is a tab-separated instruction<TAB>response file; --procedural appends that many\n\
         synthetic instruction pairs. Results are printed, and the weight masters are written to\n\
         --out if given."
    );
}

fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state >> 33
}

/// Greedy decode from the prompt, stopping at `<|end|>` or `max_new` tokens.
fn generate(
    core: &uor_r4_core::native_geometric::learner::LowBitCore,
    prompt: &str,
    max_new: usize,
) -> String {
    let mut tokens = chat::encode_prompt(prompt);
    for _ in 0..max_new {
        let logits = core.forward_reference_f32(&tokens);
        let mut best = 0usize;
        for (j, &v) in logits.iter().enumerate() {
            if v > logits[best] {
                best = j;
            }
        }
        if best as u32 == TOK_END {
            break;
        }
        tokens.push(best as u32);
    }
    // Strip the prompt back off, so the printed text is the model's own continuation.
    let text = chat::decode(&tokens);
    text.split_once("<|assistant|>")
        .map(|(_, r)| r.to_string())
        .unwrap_or(text)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            if e != "help" {
                eprintln!("error: {e}");
            }
            usage();
            return ExitCode::from(2);
        }
    };

    eprintln!(
        "train-lowbit-chat corpus={:?} procedural={} steps={} dim={} batch={} lr={} max_seq={} seed={}",
        args.corpus, args.procedural, args.steps, args.dim, args.batch, args.lr, args.max_seq, args.seed
    );

    // --- corpus -------------------------------------------------------------
    let mut pairs = chat::procedural_examples(args.procedural, args.seed);
    let procedural_count = pairs.len();
    if let Some(path) = &args.corpus {
        match fs::read_to_string(path) {
            Ok(text) => {
                let loaded = chat::parse_tsv(&text);
                eprintln!(
                    "loaded {} instruction pairs from {}",
                    loaded.len(),
                    path.display()
                );
                pairs.extend(loaded);
            }
            Err(e) => {
                eprintln!("error: cannot read corpus {}: {e}", path.display());
                return ExitCode::from(1);
            }
        }
    }

    let mut encoded: Vec<Vec<u32>> = Vec::new();
    let mut skipped = 0usize;
    for (instruction, response) in &pairs {
        let seq = chat::encode_turn(instruction, response);
        if seq.len() > args.max_seq {
            skipped += 1;
            continue;
        }
        encoded.push(seq);
    }
    if encoded.len() < 32 {
        eprintln!(
            "error: only {} usable examples; corpus too small",
            encoded.len()
        );
        return ExitCode::from(1);
    }
    let held: Vec<Vec<u32>> = encoded
        .iter()
        .enumerate()
        .filter(|(i, _)| i % args.held_every == 0)
        .map(|(_, s)| s.clone())
        .collect();
    let train: Vec<Vec<u32>> = encoded
        .iter()
        .enumerate()
        .filter(|(i, _)| i % args.held_every != 0)
        .map(|(_, s)| s.clone())
        .collect();
    let tokens_in: usize = encoded.iter().map(|s| s.len()).sum();
    let held_tokens: usize = held.iter().map(|s| s.len()).sum();
    eprintln!(
        "examples: procedural={} total={} train={} held={} skipped={}  tokens train={} held={}",
        procedural_count,
        encoded.len(),
        train.len(),
        held.len(),
        skipped,
        tokens_in - held_tokens,
        held_tokens
    );

    // --- train --------------------------------------------------------------
    let mut trainer = match LowBitCoreTrainer::new(CHAT_VOCAB, args.dim, args.seed) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    trainer.cfg = TrainConfig {
        lr: args.lr,
        ..TrainConfig::default()
    };

    let initial_held: f64 =
        held.iter().map(|s| trainer.loss(s) as f64).sum::<f64>() / held.len() as f64;
    let started = Instant::now();
    let mut order: Vec<usize> = (0..train.len()).collect();
    let mut rng = args.seed ^ 0xDEAD_BEEF;
    let report_every = (args.steps / 12).max(1);
    for step in 0..args.steps {
        // Fisher-Yates over a small batch window.
        for k in 0..order.len() {
            let j = (lcg(&mut rng) as usize) % order.len();
            order.swap(k, j);
        }
        let batch: Vec<Vec<u32>> = order
            .iter()
            .take(args.batch.min(order.len()))
            .map(|&i| train[i].clone())
            .collect();
        let last = trainer.train_batch(&batch);
        if step % report_every == 0 || step + 1 == args.steps {
            let elapsed = started.elapsed().as_secs_f32();
            eprintln!(
                "step {:>5}/{}  train_loss={:.4}  elapsed={:.1}s  ({:.2}s/step)",
                step + 1,
                args.steps,
                last,
                elapsed,
                elapsed / (step + 1) as f32
            );
        }
    }
    let train_secs = started.elapsed().as_secs_f32();

    // --- evaluate the quantised model ---------------------------------------
    let core = match trainer.to_core() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: quantise: {e}");
            return ExitCode::from(1);
        }
    };
    let quant_held: f64 = held
        .iter()
        .map(|s| core.sequence_loss(s) as f64)
        .sum::<f64>()
        / held.len() as f64;
    let (mut correct, mut total) = (0usize, 0usize);
    for s in &held {
        let (c, t) = trainer.next_token_accuracy(s);
        correct += c;
        total += t;
    }
    let acc = if total > 0 {
        correct as f64 / total as f64
    } else {
        0.0
    };

    // Response-only accuracy: teacher-forced predictions strictly inside the assistant turn.
    // This separates "learned to copy the instruction" from "learned to answer it".
    let mut resp_correct = 0usize;
    let mut resp_total = 0usize;
    let mut exact = 0usize;
    let mut scored = 0usize;
    for s in &held {
        let Some(assistant) = s.iter().position(|&t| t == chat::TOK_ASSISTANT) else {
            continue;
        };
        scored += 1;
        let mut all_ok = true;
        for i in assistant..s.len().saturating_sub(1) {
            let logits = core.forward_reference_f32(&s[..=i]);
            let mut best = 0usize;
            for (j, &v) in logits.iter().enumerate() {
                if v > logits[best] {
                    best = j;
                }
            }
            if best == s[i + 1] as usize {
                resp_correct += 1;
            } else {
                all_ok = false;
            }
            resp_total += 1;
        }
        if all_ok {
            exact += 1;
        }
    }
    let resp_acc = if resp_total > 0 {
        resp_correct as f64 / resp_total as f64
    } else {
        0.0
    };
    eprintln!(
        "response-only (teacher-forced): accuracy={:.3} ({}/{})  exact-response={}/{} ({:.1}%)",
        resp_acc,
        resp_correct,
        resp_total,
        exact,
        scored,
        100.0 * exact as f64 / scored.max(1) as f64
    );
    eprintln!(
        "\nheld-out: initial_loss={:.4}  final_loss(quantised)={:.4}  top1_accuracy={:.3} ({}/{})",
        initial_held, quant_held, acc, correct, total
    );
    eprintln!(
        "weights: {} bytes packed ({} bits/weight)  train_wall={:.1}s",
        core.weight_bytes(),
        (core.weight_bytes() * 8) as f64
            / ((core.vocab * core.dim * 3 + core.dim * core.dim) as f64),
        train_secs
    );

    // --- read the actual generations ----------------------------------------
    println!("\n=== raw generations (greedy, from the quantised model) ===");
    let prompts: Vec<String> = {
        let mut p: Vec<String> = held
            .iter()
            .take(args.samples)
            .map(|s| {
                let text = chat::decode(s);
                let after_user = text
                    .strip_prefix("<|user|>")
                    .and_then(|t| t.split_once("<|assistant|>"))
                    .map(|(i, _)| i.to_string())
                    .unwrap_or_default();
                after_user
            })
            .collect();
        p.push("Hello!".to_string());
        p.push("What is 12 + 7?".to_string());
        p
    };
    for p in prompts.iter().take(args.samples + 2) {
        let out = generate(&core, p, 64);
        println!("PROMPT: {p}");
        println!("OUTPUT: {out}");
        println!();
    }

    // --- persist ------------------------------------------------------------
    if let Some(path) = &args.out {
        if let Some(dir) = path.parent() {
            if let Err(e) = fs::create_dir_all(dir) {
                eprintln!("error: cannot create {}: {e}", dir.display());
                return ExitCode::from(1);
            }
        }
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"LBCH");
        bytes.extend_from_slice(&(core.vocab as u32).to_le_bytes());
        bytes.extend_from_slice(&(core.dim as u32).to_le_bytes());
        for v in trainer.wx.iter().chain(&trainer.wh).chain(&trainer.wo) {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        if let Err(e) = fs::write(path, &bytes) {
            eprintln!("error: cannot write {}: {e}", path.display());
            return ExitCode::from(1);
        }
        eprintln!("wrote {} ({} bytes)", path.display(), bytes.len());
    }

    ExitCode::SUCCESS
}
