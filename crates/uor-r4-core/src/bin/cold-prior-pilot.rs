//! Bounded paired pilot: learned cold-context prior (A) versus prior + causal memory (B).
//!
//! One pinned manifest, two arms with matched initialisation and optimizer settings, same-artifact
//! interventions, labelled count controls, predeclared gates, generations, export/reload parity and
//! timings. No chat capability is claimed; this is a small development experiment.

#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::cold_prior::{
    Channels, ColdPriorConfig, ColdPriorCore, ColdPriorTrainer,
};
use uor_r4_core::native_geometric::learner::TrainConfig;
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
    seed: u64,
    max_bytes: usize,
    out_dir: PathBuf,
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
            steps: 256,
            dv: 128,
            norm_bits: 6,
            seed: 2026_0919,
            max_bytes: 4 * 1024 * 1024,
            out_dir: PathBuf::from(".uor-models/cold-prior-2026-09-19"),
        }
    }
}

fn usage(p: &str) {
    eprintln!(
        "Usage: {p} [--tokenizer F] [--vocab N] [--corpus D] [--window N] [--batch N] [--steps N]\n\
         \x20          [--dv N] [--seed N] [--max-bytes N] [--out D]"
    );
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args::default();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let k = argv[i].as_str();
        let v = || -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("{k} needs a value"))
        };
        match k {
            "--tokenizer" => a.tokenizer = PathBuf::from(v()?),
            "--vocab" => a.vocab = v()?.parse().map_err(|e| format!("{e}"))?,
            "--corpus" => a.corpus = PathBuf::from(v()?),
            "--window" => a.window = v()?.parse().map_err(|e| format!("{e}"))?,
            "--batch" => a.batch = v()?.parse().map_err(|e| format!("{e}"))?,
            "--steps" => a.steps = v()?.parse().map_err(|e| format!("{e}"))?,
            "--dv" => a.dv = v()?.parse().map_err(|e| format!("{e}"))?,
            "--norm-bits" => a.norm_bits = v()?.parse().map_err(|e| format!("{e}"))?,
            "--seed" => a.seed = v()?.parse().map_err(|e| format!("{e}"))?,
            "--max-bytes" => a.max_bytes = v()?.parse().map_err(|e| format!("{e}"))?,
            "--out" => a.out_dir = PathBuf::from(v()?),
            "--help" | "-h" => return Err("help".into()),
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    Ok(a)
}

fn collect(path: &Path, rel: &Path, budget: &mut usize, out: &mut Vec<(String, String)>) {
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
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        if text.len() > *budget {
            return;
        }
        *budget -= text.len();
        out.push((rel.display().to_string(), text));
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    let mut kids: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    kids.sort();
    for kid in kids {
        if *budget == 0 {
            return;
        }
        if kid
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| matches!(n, "target" | ".git" | ".worktrees"))
        {
            continue;
        }
        let r = rel.join(kid.file_name().unwrap_or_default());
        collect(&kid, &r, budget, out);
    }
}

/// Per-target evaluation: aggregate, cold and warm losses, plus top-1.
struct Eval {
    loss: f64,
    cold_loss: f64,
    warm_loss: f64,
    top1: f64,
    cold: usize,
    warm: usize,
    n: usize,
    /// Mean absolute logit magnitude: the activation-scale diagnostic. A well-scaled model sits in the
    /// single digits; a value in the hundreds means the readout is saturated and the softmax is a
    /// near-argmax, which produces confident errors and a loss far above `log2(V)`.
    mean_abs_logit: f64,
    per_doc: HashMap<String, (f64, usize)>,
}

fn evaluate(core: &ColdPriorCore, dev: &[(String, Vec<u32>)], warm_only: bool) -> Eval {
    let v = core.cfg.vocab;
    let mut e = Eval {
        loss: 0.0,
        cold_loss: 0.0,
        warm_loss: 0.0,
        top1: 0.0,
        cold: 0,
        warm: 0,
        n: 0,
        mean_abs_logit: 0.0,
        per_doc: HashMap::new(),
    };
    let mut hits = 0usize;
    for (name, w) in dev {
        let n = w.len();
        if n < 3 {
            continue;
        }
        let mut s = core.initial_state();
        let mut doc_loss = 0f64;
        let mut doc_n = 0usize;
        for i in 0..(n - 2) {
            if i >= 2 {
                core.observe(&mut s, w[i - 2], w[i - 1], w[i]);
            }
            let prev = if i == 0 { None } else { Some(w[i - 1]) };
            let a = core.address(prev.unwrap_or(0), w[i]);
            let live = s[core.cfg.n_addr() * core.cfg.dv + a] > 0;
            if warm_only && !live {
                continue;
            }
            let logits = core.logits(&s, prev, w[i]);
            e.mean_abs_logit += logits.iter().map(|&x| x.abs() as f64).sum::<f64>();
            let lf: Vec<f32> = logits.iter().map(|&x| x as f32).collect();
            let p = softmax(&lf);
            let target = (w[i + 1] as usize) % v;
            let l = -(p[target].max(1e-9)).ln() as f64;
            doc_loss += l;
            doc_n += 1;
            e.n += 1;
            e.loss += l;
            if live {
                e.warm += 1;
                e.warm_loss += l;
            } else {
                e.cold += 1;
                e.cold_loss += l;
            }
            let mut best = 0usize;
            for (r, &x) in logits.iter().enumerate() {
                if x > logits[best] {
                    best = r;
                }
            }
            if best == target {
                hits += 1;
            }
        }
        let ent = e.per_doc.entry(name.clone()).or_insert((0.0, 0));
        ent.0 += doc_loss;
        ent.1 += doc_n;
    }
    if e.n > 0 {
        e.loss /= e.n as f64;
        e.top1 = hits as f64 / e.n as f64;
        e.mean_abs_logit /= (e.n * v) as f64;
    }
    if e.cold > 0 {
        e.cold_loss /= e.cold as f64;
    }
    if e.warm > 0 {
        e.warm_loss /= e.warm as f64;
    }
    e
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let m = logits.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let ex: Vec<f32> = logits.iter().map(|v| (v - m).exp()).collect();
    let s: f32 = ex.iter().sum();
    ex.iter().map(|v| v / s).collect()
}

/// Document-level paired bootstrap on the aggregate loss difference.
fn paired_docs(a: &Eval, b: &Eval) -> (f64, f64, f64) {
    let mut keys: Vec<&String> = a.per_doc.keys().collect();
    keys.sort();
    let mut diffs: Vec<f64> = Vec::new();
    for k in keys {
        let (la, na) = a.per_doc[k];
        let (lb, nb) = b.per_doc.get(k).copied().unwrap_or((0.0, 0));
        if na > 0 && nb > 0 {
            diffs.push(la / na as f64 - lb / nb as f64);
        }
    }
    if diffs.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
    let mut st = 0x9E37_79B9_u64;
    let mut boot = Vec::with_capacity(400);
    for _ in 0..400 {
        let mut s = 0f64;
        for _ in 0..diffs.len() {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            s += diffs[(st as usize) % diffs.len()];
        }
        boot.push(s / diffs.len() as f64);
    }
    boot.sort_by(|x, y| x.partial_cmp(y).unwrap());
    (
        mean,
        boot[boot.len() / 40],
        boot[boot.len() - 1 - boot.len() / 40],
    )
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
    let started = Instant::now();
    println!("=== cold-context prior + causal memory: bounded paired pilot ===");
    println!(
        "config: V={} dv={} window={} batch={} steps={} seed={}",
        args.vocab, args.dv, args.window, args.batch, args.steps, args.seed
    );

    // --- manifest -----------------------------------------------------------
    let original = match std::fs::read(&args.tokenizer) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: tokenizer: {e}");
            return ExitCode::from(1);
        }
    };
    let tok_digest: [u8; 32] = Sha256::digest(&original).into();
    println!("source tokenizer sha256:{}", hex::encode(tok_digest));
    let tokenizer = match derive_tokenizer(&original, args.vocab) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: derive: {e}");
            return ExitCode::from(1);
        }
    };
    let vocab = tokenizer.vocab_size();

    let mut budget = args.max_bytes;
    let mut raw: Vec<(String, String)> = Vec::new();
    collect(&args.corpus, Path::new(""), &mut budget, &mut raw);
    if raw.is_empty() {
        eprintln!("error: no corpus documents");
        return ExitCode::from(1);
    }
    // Group exact duplicates by content hash before splitting: a duplicate must not straddle the split.
    let mut seen: HashSet<[u8; 32]> = HashSet::new();
    let mut docs: Vec<(String, Vec<u32>)> = Vec::new();
    let mut dups = 0usize;
    let mut corpus_hasher = Sha256::new();
    for (rel, text) in &raw {
        let h: [u8; 32] = Sha256::digest(text.as_bytes()).into();
        corpus_hasher.update((text.len() as u64).to_le_bytes());
        corpus_hasher.update(text.as_bytes());
        if !seen.insert(h) {
            dups += 1;
            continue;
        }
        docs.push((rel.clone(), tokenizer.encode(text)));
    }
    let total_tokens: usize = docs.iter().map(|d| d.1.len()).sum();
    let corpus_digest: [u8; 32] = corpus_hasher.finalize().into();
    println!(
        "manifest: documents={} exact_duplicates_grouped={} tokens={} corpus sha256:{}",
        docs.len(),
        dups,
        total_tokens,
        hex::encode(corpus_digest)
    );

    // Deterministic document-separated split: every 10th document is development.
    //
    // The development set is capped for tractable evaluation (full-vocabulary scoring is O(V*dv) per
    // target) and the cap is spread *across* dev documents with a per-document quota, so the paired
    // document-level uncertainty is not computed over one or two documents.
    const DEV_CAP: usize = 128;
    let n_dev_docs = docs
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 10 == 0)
        .count()
        .max(1);
    let quota = (DEV_CAP / n_dev_docs).max(1);
    let mut fit_windows: Vec<Vec<u32>> = Vec::new();
    let mut dev: Vec<(String, Vec<u32>)> = Vec::new();
    for (i, (name, toks)) in docs.iter().enumerate() {
        let chunks: Vec<Vec<u32>> = toks
            .chunks(args.window)
            .filter(|c| c.len() == args.window)
            .map(|c| c.to_vec())
            .collect();
        if i % 10 == 0 {
            for c in chunks.iter().take(quota) {
                dev.push((name.clone(), c.clone()));
            }
        } else {
            for c in chunks {
                fit_windows.push(c);
            }
        }
    }
    let fit_targets: usize = fit_windows.len() * (args.window - 2);
    let dev_targets: usize = dev.iter().map(|(_, w)| w.len().saturating_sub(2)).sum();
    println!(
        "split: fit_windows={} fit_targets={} dev_windows={} dev_documents={} (per-doc quota {}) dev_targets={}",
        fit_windows.len(),
        fit_targets,
        dev.len(),
        n_dev_docs,
        quota,
        dev_targets
    );
    if fit_windows.len() < args.batch || dev.len() < 4 {
        eprintln!("error: manifest too small");
        return ExitCode::from(1);
    }

    // Fit-only unigram counts and targets for the matched references.
    let mut uni: HashMap<u32, u64> = HashMap::new();
    let mut uni_total = 0u64;
    for w in &fit_windows {
        for &t in w {
            *uni.entry(t).or_insert(0) += 1;
            uni_total += 1;
        }
    }
    let mut dev_target_list: Vec<u32> = Vec::new();
    for (_, w) in &dev {
        for i in 0..(w.len() - 2) {
            dev_target_list.push(w[i + 1]);
        }
    }
    // Fit-only conditional counts for the labelled references.
    let mut bi: HashMap<(u32, u32), u64> = HashMap::new();
    let mut bi_ctx: HashMap<u32, u64> = HashMap::new();
    let mut tri: HashMap<(u64, u32), u64> = HashMap::new();
    let mut tri_ctx: HashMap<u64, u64> = HashMap::new();
    for w in &fit_windows {
        for i in 1..w.len() {
            *bi.entry((w[i - 1], w[i])).or_insert(0) += 1;
            *bi_ctx.entry(w[i - 1]).or_insert(0) += 1;
            if i >= 2 {
                let c = ((w[i - 2] as u64) << 32) | w[i - 1] as u64;
                *tri.entry((c, w[i])).or_insert(0) += 1;
                *tri_ctx.entry(c).or_insert(0) += 1;
            }
        }
    }
    let uni_p = |t: u32| -> f64 {
        (uni.get(&t).copied().unwrap_or(0) as f64 + 1.0) / (uni_total as f64 + vocab as f64)
    };
    let mut uni_bits = 0f64;
    let mut t1_bits = 0f64;
    let mut t2_bits = 0f64;
    let mut rescored = 0usize;
    for (_, w) in &dev {
        for i in 0..(w.len() - 2) {
            let next = w[i + 1];
            uni_bits += -uni_p(next).ln();
            // The context available at position `i` is the same left-padded one the core sees, so a
            // missing predecessor falls back to the unigram rather than indexing out of bounds.
            let p1 = if i == 0 {
                uni_p(next)
            } else {
                match bi_ctx.get(&w[i]) {
                    Some(&tot) => {
                        let c = bi.get(&(w[i], next)).copied().unwrap_or(0) as f64;
                        (c + 1.0) / (tot as f64 + vocab as f64)
                    }
                    None => uni_p(next),
                }
            };
            t1_bits += -p1.ln();
            let p2 = if i < 2 {
                uni_p(next)
            } else {
                let key = ((w[i - 2] as u64) << 32) | w[i - 1] as u64;
                match tri_ctx.get(&key) {
                    Some(&tot) => {
                        let c = tri.get(&(key, next)).copied().unwrap_or(0) as f64;
                        (c + 1.0) / (tot as f64 + vocab as f64)
                    }
                    None => uni_p(next),
                }
            };
            t2_bits += -p2.ln();
            rescored += 1;
        }
    }
    let uni_ce = (uni_bits / rescored.max(1) as f64) as f32;
    let t1_ce = (t1_bits / rescored.max(1) as f64) as f32;
    let t2_ce = (t2_bits / rescored.max(1) as f64) as f32;
    let _ = &dev_target_list;
    println!();
    println!("references on the SAME dev targets (fit-only counts, add-1 smoothing, NOT tuned):");
    println!("  unigram (matched)     {uni_ce:.4} bits/target");
    println!("  token order-1         {t1_ce:.4} bits/target");
    println!("  token order-2         {t2_ce:.4} bits/target");
    println!(
        "  [labelled controls: a global count model's information regime differs from the core's]"
    );

    // --- two matched arms ---------------------------------------------------
    let mut base = ColdPriorConfig::new(vocab, args.dv);
    base.norm_bits = args.norm_bits;
    println!();
    println!("training two arms, matched init and optimizer, 256 steps each unless overridden:");
    let mut arms: Vec<(&str, ColdPriorTrainer, f64)> = Vec::new();
    for (label, channels) in [
        ("A prior-only", Channels::PRIOR_ONLY),
        ("B prior+memory", Channels::BOTH),
    ] {
        let mut c = base.clone();
        c.channels = channels;
        let mut t = match ColdPriorTrainer::new(c, args.seed) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: trainer: {e}");
                return ExitCode::from(1);
            }
        };
        t.cfg_train = TrainConfig {
            lr: 0.05,
            ..TrainConfig::default()
        };
        let t0 = Instant::now();
        let mut first = f32::NAN;
        for step in 0..args.steps {
            let batch: Vec<Vec<u32>> = (0..args.batch)
                .map(|k| fit_windows[(step * args.batch + k) % fit_windows.len()].clone())
                .collect();
            let l = t.train_batch(&batch);
            if step == 0 {
                first = l;
            }
        }
        let secs = t0.elapsed().as_secs_f64();
        let grad = t.prior_grad_norm();
        let core = match t.to_core() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: to_core: {e}");
                return ExitCode::from(1);
            }
        };
        let ev = evaluate(&core, &dev, false);
        println!(
            "  {label:<16} {:.1}s ({:.3} s/step)  first_batch_loss {first:.4}  final_prior_grad_norm {grad:.4}",
            secs,
            secs / args.steps as f64,
        );
        println!(
            "      aggregate {:.4} bits/target  top1 {:.4} | cold {:.4} (n={}) | warm {:.4} (n={}) | mean|logit| {:.1}",
            ev.loss, ev.top1, ev.cold_loss, ev.cold, ev.warm_loss, ev.warm, ev.mean_abs_logit
        );
        println!(
            "      weights {} bytes | quantization occupancy: e_old {:.3} e_new {:.3} w_v {:.3} w_o {:.3}",
            core.weight_bytes(),
            occupancy(&core.e_old),
            occupancy(&core.e_new),
            occupancy(&core.w_v),
            occupancy(&core.w_o),
        );
        arms.push((label, t, ev.loss));
    }

    // --- interventions on the joint artifact --------------------------------
    let joint_core = match arms[1].1.to_core() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    let mut prior_only_core = joint_core.clone();
    prior_only_core.cfg.channels = Channels::PRIOR_ONLY;
    let mut memory_only_core = joint_core.clone();
    memory_only_core.cfg.channels = Channels::MEMORY_ONLY;
    let mut neither_core = joint_core.clone();
    neither_core.cfg.channels = Channels::NEITHER;

    let full = evaluate(&joint_core, &dev, false);
    let mem_off = evaluate(&prior_only_core, &dev, false);
    let prior_off = evaluate(&memory_only_core, &dev, false);
    let bias_only = evaluate(&neither_core, &dev, false);
    let warm_mem_off = evaluate(&prior_only_core, &dev, true);

    println!();
    println!("same-artifact interventions (identical targets and scales):");
    println!(
        "  full            {:.4} bits/target  top1 {:.4}  mean|logit| {:.1}",
        full.loss, full.top1, full.mean_abs_logit
    );
    println!(
        "  memory disabled {:.4} bits/target  top1 {:.4}  mean|logit| {:.1}",
        mem_off.loss, mem_off.top1, mem_off.mean_abs_logit
    );
    println!(
        "  prior disabled  {:.4} bits/target  top1 {:.4}",
        prior_off.loss, prior_off.top1
    );
    println!(
        "  both disabled (bias only) {:.4} bits/target  top1 {:.4}",
        bias_only.loss, bias_only.top1
    );
    println!(
        "  memory gain on WARM targets: full {:.4} vs memory-disabled {:.4} = {:.4} bits/target (n warm={})",
        full.warm_loss,
        warm_mem_off.warm_loss,
        warm_mem_off.warm_loss - full.warm_loss,
        full.warm
    );

    // Quantised constant control, fitted offline from the fit-only distribution. Logits are the log of
    // the smoothed fit-unigram probability, so the softmax of the control *is* that distribution; a
    // large arbitrary scale would make the control a confidently-wrong argmax predictor and would not
    // be a constant-probability baseline at all.
    let mut const_core = joint_core.clone();
    const_core.cfg.channels = Channels::NEITHER;
    for r in 0..vocab {
        let p = (uni.get(&(r as u32)).copied().unwrap_or(0) as f64 + 1.0)
            / (uni_total as f64 + vocab as f64);
        const_core.bias[r] = (p.ln()).round() as i32;
    }
    let constant = evaluate(&const_core, &dev, false);
    println!(
        "  quantized constant control (fit-only log-unigram, unit logit scale) {:.4} bits/target  top1 {:.4}",
        constant.loss, constant.top1
    );

    // --- gates --------------------------------------------------------------
    let (prior_only_loss, joint_loss) = (arms[0].2 as f64, arms[1].2 as f64);
    let gain_vs_unigram = uni_ce as f64 - prior_only_loss;
    let gain_vs_constant = constant.loss - prior_only_loss;
    let mem_gain = mem_off.warm_loss - full.warm_loss;
    let regress = prior_only_loss - joint_loss;
    let (d_prior, lo_prior, hi_prior) =
        paired_docs(&eval_of(&arms[0].1, &dev), &const_eval(&const_core, &dev));
    println!();
    println!("PREDECLARED GATES (development continuation thresholds, not significance):");
    println!(
        "  1 prior contextual gain >= 0.10 bits/target over matched unigram: {gain_vs_unigram:.4} -> {}",
        pass(gain_vs_unigram >= 0.10)
    );
    println!(
        "    prior contextual gain over quantized constant control: {gain_vs_constant:.4} -> {}",
        pass(gain_vs_constant >= 0.10)
    );
    println!(
        "  2 joint memory gain >= 0.02 bits/target on warm targets vs memory-disabled: {mem_gain:.4} -> {}",
        pass(mem_gain >= 0.02)
    );
    println!(
        "    ...and vs the separately trained prior-only model (aggregate): {:.4} -> {}",
        prior_only_loss - joint_loss,
        pass(prior_only_loss - joint_loss >= 0.0)
    );
    println!(
        "  3 joint aggregate not worse than prior-only by > 0.01: {regress:.4} -> {}",
        pass(regress >= -0.01)
    );
    println!(
        "  paired document-level difference (prior-only minus constant): {d_prior:.4} bits/target, 90% CI [{lo_prior:.4}, {hi_prior:.4}] over {} documents",
        dev_doc_count(&dev)
    );
    println!(
        "  warm coverage {:.4} of dev targets -> {}",
        full.warm as f64 / full.n.max(1) as f64,
        if (full.warm as f64) < 0.10 * full.n as f64 {
            "TOO SMALL to resolve the memory gate; report as unresolved"
        } else {
            "sufficient to attempt resolution"
        }
    );

    // --- export, reload parity, generation ----------------------------------
    let out_dir = &args.out_dir;
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        eprintln!("error: create {}: {e}", out_dir.display());
        return ExitCode::from(1);
    }
    let bytes = joint_core.to_bytes(&tok_digest, arms[1].1.seed());
    let path = out_dir.join("cold_prior_joint.cpr");
    let t_save = Instant::now();
    if let Err(e) = std::fs::write(&path, &bytes) {
        eprintln!("error: write artifact: {e}");
        return ExitCode::from(1);
    }
    let save_s = t_save.elapsed().as_secs_f64();
    let t_load = Instant::now();
    let reloaded = match std::fs::read(&path).and_then(|b| {
        ColdPriorCore::from_bytes(&b)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: reload: {e}");
            return ExitCode::from(1);
        }
    };
    let load_s = t_load.elapsed().as_secs_f64();
    let checksum = Sha256::digest(&bytes);
    println!();
    println!(
        "artifact: {} ({} bytes, sha256:{}) save {save_s:.4}s load {load_s:.4}s",
        path.display(),
        bytes.len(),
        hex::encode(checksum)
    );
    let same = reloaded == joint_core;
    println!("export/reload byte-exact core equality: {}", pass(same));
    let parity = dev.iter().all(|(_, w)| {
        let a = joint_core.forward(&w[..w.len().min(32)]);
        let b = reloaded.forward(&w[..w.len().min(32)]);
        a == b
    });
    println!(
        "export/reload integer-logit parity on dev windows: {}",
        pass(parity)
    );

    println!();
    println!("=== complete deterministic continuations (integer greedy, 24 tokens) ===");
    let bias_only_v = bias_only_core(&joint_core);
    let prompts: Vec<(String, Vec<u32>)> = dev
        .iter()
        .take(3)
        .map(|(n, w)| (n.clone(), w[..16].to_vec()))
        .collect();
    for (name, prompt) in &prompts {
        println!("--- prompt from {name} ---");
        for (label, core) in [
            ("full        ", &joint_core),
            ("memory off  ", &prior_only_core),
            ("prior off   ", &memory_only_core),
            ("bias only   ", &bias_only_v),
        ] {
            let text = greedy(core, prompt, 24);
            println!("  {label}: {text}");
        }
    }

    println!();
    println!(
        "elapsed {:.1}s | no chat/reasoning/coding/energy claim follows from this pilot",
        started.elapsed().as_secs_f32()
    );
    ExitCode::SUCCESS
}

fn pass(b: bool) -> &'static str {
    if b {
        "PASS"
    } else {
        "FAIL"
    }
}

fn occupancy(t: &uor_r4_core::native_geometric::learner::TernaryLinear) -> f32 {
    let mut nz = 0usize;
    for r in 0..t.rows {
        for c in 0..t.cols {
            if t.weight(r, c) != 0 {
                nz += 1;
            }
        }
    }
    nz as f32 / (t.rows * t.cols).max(1) as f32
}

fn bias_only_core(core: &ColdPriorCore) -> ColdPriorCore {
    let mut c = core.clone();
    c.cfg.channels = Channels::NEITHER;
    c
}

fn greedy(core: &ColdPriorCore, prompt: &[u32], n_new: usize) -> String {
    let mut toks: Vec<u32> = prompt.to_vec();
    for _ in 0..n_new {
        let mut s = core.initial_state();
        for i in 0..toks.len() {
            if i >= 2 {
                core.observe(&mut s, toks[i - 2], toks[i - 1], toks[i]);
            }
        }
        let last = toks.len() - 1;
        let prev = if last == 0 {
            None
        } else {
            Some(toks[last - 1])
        };
        let logits = core.logits(&s, prev, toks[last]);
        let mut best = 0usize;
        for (r, &x) in logits.iter().enumerate() {
            if x > logits[best] {
                best = r;
            }
        }
        toks.push(best as u32);
    }
    format!("ids[{}..]={:?}", prompt.len(), &toks[prompt.len()..])
}

fn eval_of(t: &ColdPriorTrainer, dev: &[(String, Vec<u32>)]) -> Eval {
    match t.to_core() {
        Ok(c) => evaluate(&c, dev, false),
        Err(_) => Eval {
            loss: 0.0,
            cold_loss: 0.0,
            warm_loss: 0.0,
            top1: 0.0,
            cold: 0,
            warm: 0,
            n: 0,
            mean_abs_logit: 0.0,
            per_doc: HashMap::new(),
        },
    }
}

fn const_eval(c: &ColdPriorCore, dev: &[(String, Vec<u32>)]) -> Eval {
    evaluate(c, dev, false)
}

fn dev_doc_count(dev: &[(String, Vec<u32>)]) -> usize {
    let mut s: HashSet<&String> = HashSet::new();
    for (n, _) in dev {
        s.insert(n);
    }
    s.len()
}
