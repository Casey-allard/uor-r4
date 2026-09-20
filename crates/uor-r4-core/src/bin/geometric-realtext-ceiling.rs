//! A **static function-of-address backoff comparison** on real text.
//!
//! # What this is, after the takeover correction
//!
//! An earlier version of this tool called its output an "achievable ceiling" for the order-2 core and
//! concluded the mechanism's context was just the residue pair. **That framing was wrong** and is
//! withdrawn: the two residues select the *bucket*, but the bucket holds successor values accumulated
//! from the prefix, so `[0,1,2,0,1]` and `[0,1,3,0,1]` share a query address yet carry different
//! values. What this tool measures is narrower and still useful — the best predictor that sees **only
//! the address**, i.e. a static count model over residue pairs, compared against the same model over
//! real token pairs and against the unigram.
//!
//! Two further corrections are carried:
//!
//! * A fitted Jelinek–Mercer estimator's held-out cross-entropy is an *achieved* score, not the Bayes
//!   conditional entropy. A higher-order finite estimator can score worse than a lower-order one
//!   without contradicting conditional-entropy monotonicity, so these are development baselines.
//! * Cross-entropy and top-1 are now derived from the **same** interpolated distribution, and the
//!   argmax is the argmax of that distribution. A `lambda = 0` fixture must therefore reproduce the
//!   unigram distribution *and* the unigram argmax exactly.
//!
//! # Protocol (explicit, because an earlier version claimed more than it did)
//!
//! Documents are split: every `held_every`-th document is held out, and every fifth remaining document
//! is validation. Counts are fit on the **fit split only**; the interpolation weights are tuned on the
//! disjoint validation split by target-probability cross-entropy; the held-out figures use those fit
//! counts and those weights. **There is no refit on the full training split**, and no tuning on the
//! held-out split. Scored positions are the same range for every family (`i >= 2`), and the bits/byte
//! denominator covers exactly the held-out documents that produced the scored positions, with no
//! cross-document context.
//!
//! # What it is not
//!
//! Not a ceiling, not a model result, and not a verdict on the geometric architecture. The mechanism
//! as configured only (fixed `element(token) = token % 120`, `order <= 2`). Learned packaging is
//! untested here. No chat or language capability is claimed.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use sha2::{Digest, Sha256};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

/// The `2I` radix the mechanism addresses over (`element(token) = token % RADIX`).
const RADIX: u64 = 120;
/// Jelinek–Mercer grid for the interpolation weights. Capped below 1.0 so a held-out position
/// whose pair is unseen in a seen context still receives a non-zero unigram floor.
const LAMBDA_GRID: [f64; 10] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

struct Args {
    tokenizer: PathBuf,
    vocab: usize,
    corpus: PathBuf,
    held_every: usize,
    max_bytes: usize,
    shuffle_control: bool,
    seed: u64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            tokenizer: PathBuf::from(
                "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json",
            ),
            vocab: 4096,
            corpus: PathBuf::from("/Users/casey.allard/uor-r4/crates"),
            held_every: 10,
            max_bytes: 4 * 1024 * 1024,
            shuffle_control: true,
            seed: 2026_0919,
        }
    }
}

fn usage(program: &str) {
    eprintln!(
        "Usage: {program} [OPTIONS]\n\
         \x20 --tokenizer <path>   tokenizer.json to derive from (default: SmolLM2-135M-Instruct)\n\
         \x20 --vocab <n>          derived vocabulary size (default 4096; 0 = use as-is)\n\
         \x20 --corpus <path>      file or directory of real text (default: crates/)\n\
         \x20 --held-every <n>     every n-th document is held out (default 10)\n\
         \x20 --max-bytes <n>      byte budget for the corpus (default 4 MiB)\n\
         \x20 --no-shuffle        skip the shuffled-corpus control\n\
         \x20 --seed <n>          shuffle seed (default 20260919)\n\
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
            "--held-every" => a.held_every = val()?.parse().map_err(|e| format!("{e}"))?,
            "--max-bytes" => a.max_bytes = val()?.parse().map_err(|e| format!("{e}"))?,
            "--seed" => a.seed = val()?.parse().map_err(|e| format!("{e}"))?,
            "--no-shuffle" => a.shuffle_control = false,
            "--help" | "-h" => return Err("help".into()),
            other => return Err(format!("unknown argument {other}")),
        }
        i += if key == "--no-shuffle" { 1 } else { 2 };
    }
    Ok(a)
}

/// Derive a dense `vocab`-sized byte-level BPE `tokenizer.json` from a larger one.
///
/// The derived vocabulary is the dense id prefix `0..vocab`. Merges are kept in their original
/// order but only when their product token survives in that prefix, which is exactly the
/// truncation a fresh BPE at this size produces: base byte tokens plus the first `vocab - 256`
/// merges. The result is handed to the project's existing verified parser rather than to a
/// hand-written encoder.
fn derive_tokenizer_json(original: &[u8], vocab: usize) -> Result<Vec<u8>, String> {
    // Single-sourced with the coverage instrument: see `transformerless::bpe_derive`.
    uor_r4_core::transformerless::bpe_derive::derive_tokenizer_json(original, vocab)
}

/// Recursively collect files with the given extensions, bounded by a byte budget.
fn collect_documents(path: &Path, exts: &[&str], budget: &mut usize, out: &mut Vec<String>) {
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
            .is_some_and(|e| exts.contains(&e));
        if !keep {
            return;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        if text.len() > *budget {
            let mut cut = *budget;
            while cut > 0 && !text.is_char_boundary(cut) {
                cut -= 1;
            }
            out.push(text[..cut].to_string());
            *budget = 0;
            return;
        }
        *budget -= text.len();
        out.push(text);
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
            .is_some_and(|n| n == "target" || n == ".git")
        {
            continue;
        }
        collect_documents(&child, exts, budget, out);
    }
}

/// One conditional context model over sparse `(context, next) -> count` statistics.
#[derive(Default)]
struct Cond {
    counts: HashMap<(u64, u32), u32>,
    totals: HashMap<u64, (u64, u32, u32)>,
    /// Distinct successors seen per context, so the interpolated argmax can scan candidates only.
    by_ctx: HashMap<u64, Vec<u32>>,
}

impl Cond {
    fn observe(&mut self, ctx: u64, next: u32) {
        let e = self.counts.entry((ctx, next)).or_insert(0);
        *e += 1;
        let t = self.totals.entry(ctx).or_insert((0, 0, 0));
        t.0 += 1;
        if *e > t.2 {
            t.1 = next;
            t.2 = *e;
        }
        let cands = self.by_ctx.entry(ctx).or_default();
        if !cands.contains(&next) {
            cands.push(next);
        }
    }

    fn total(&self, ctx: u64) -> u64 {
        self.totals.get(&ctx).map(|t| t.0).unwrap_or(0)
    }

    fn count_of(&self, ctx: u64, next: u32) -> u32 {
        self.counts.get(&(ctx, next)).copied().unwrap_or(0)
    }

    /// Mean observations per observed context, a sparsity diagnostic.
    fn mean_obs(&self) -> f64 {
        if self.totals.is_empty() {
            0.0
        } else {
            self.totals.values().map(|t| t.0).sum::<u64>() as f64 / self.totals.len() as f64
        }
    }
}

/// Everything fitted on one token stream.
struct Fit {
    unigram: Vec<u64>,
    uni_total: u64,
    uni_argmax: u32,
    res1: Cond,
    res2: Cond,
    true1: Cond,
    true2: Cond,
}

fn fit_all(seqs: &[&Vec<u32>], vocab: usize) -> Fit {
    let mut f = Fit {
        unigram: vec![0u64; vocab],
        uni_total: 0,
        uni_argmax: 0,
        res1: Cond::default(),
        res2: Cond::default(),
        true1: Cond::default(),
        true2: Cond::default(),
    };
    for seq in seqs {
        for &t in seq.iter() {
            f.unigram[t as usize] += 1;
            f.uni_total += 1;
        }
        for i in 1..seq.len() {
            let p = seq[i - 1] as u64;
            f.res1.observe(p % RADIX, seq[i]);
            f.true1.observe(p, seq[i]);
        }
        for i in 2..seq.len() {
            let pp = seq[i - 2] as u64;
            let p = seq[i - 1] as u64;
            f.res2.observe((pp % RADIX) * RADIX + (p % RADIX), seq[i]);
            f.true2.observe(pp * vocab as u64 + p, seq[i]);
        }
    }
    f.uni_argmax = f
        .unigram
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| **c)
        .map(|(i, _)| i as u32)
        .unwrap_or(0);
    f
}

impl Fit {
    fn unigram_p(&self, next: u32, vocab: usize) -> f64 {
        (self.unigram[next as usize] as f64 + 1.0) / (self.uni_total as f64 + vocab as f64)
    }

    fn level_p(&self, cond: &Cond, ctx: u64, next: u32, vocab: usize) -> f64 {
        let total = cond.total(ctx);
        if total == 0 {
            return self.unigram_p(next, vocab);
        }
        // Maximum likelihood inside the level; Jelinek–Mercer interpolation supplies the floor.
        cond.count_of(ctx, next) as f64 / total as f64
    }
}

/// A named context family: one or two conditional levels over a shared context extractor pair.
struct Family<'a> {
    name: &'a str,
    contexts: Vec<&'a dyn Fn(&[u32], usize) -> u64>,
    /// Selects the fitted `Cond` for level `k` (0 = highest order).
    level: fn(&Fit, usize) -> &Cond,
    description: &'a str,
}

/// Blended probability of `next` under a family, given per-level interpolation weights.
fn family_p(f: &Fit, fam: &Family, ctxs: &[u64], next: u32, lambdas: &[f64], vocab: usize) -> f64 {
    let mut p = f.unigram_p(next, vocab);
    for k in (0..ctxs.len()).rev() {
        let cond = (fam.level)(f, k);
        let pk = f.level_p(cond, ctxs[k], next, vocab);
        p = lambdas[k] * pk + (1.0 - lambdas[k]) * p;
    }
    p
}

/// The argmax of the **same interpolated distribution** whose cross-entropy is reported.
///
/// Candidates are the successors observed in each level's context plus the unigram argmax, so with
/// `lambda = 0` the result is exactly the unigram argmax. This is computed once per scored position at
/// the final weights, never inside the tuning loop.
fn blended_argmax(f: &Fit, fam: &Family, ctxs: &[u64], lambdas: &[f64], vocab: usize) -> u32 {
    let mut best = f.uni_argmax;
    // The incumbent must be scored under the SAME interpolated distribution as the candidates. Seeding
    // it with the unmixed unigram probability compares an unmixed threshold against mixed values and
    // keeps the wrong class whenever the interpolation moves mass away from the unigram argmax
    // (unigram (0.6,0.3,0.1), conditional (0.1,0.5,0.4), lambda 0.5 gives (0.35,0.40,0.25)).
    let mut best_p = family_p(f, fam, ctxs, best, lambdas, vocab);
    for k in 0..ctxs.len() {
        let cond = (fam.level)(f, k);
        if cond.total(ctxs[k]) == 0 {
            continue;
        }
        if let Some(cands) = cond.by_ctx.get(&ctxs[k]) {
            for &next in cands {
                let p = family_p(f, fam, ctxs, next, lambdas, vocab);
                if p > best_p {
                    best_p = p;
                    best = next;
                }
            }
        }
    }
    best
}

/// Cross-entropy of a family on held-out sequences, and optionally top-1 accuracy.
///
/// Accuracy is computed **only** when `accuracy` is true: the argmax of the interpolated distribution
/// is far more expensive than its cross-entropy at the observed target, and the tuning sweep must not
/// pay it once per candidate weight setting. Tuning therefore sees cross-entropy alone and the final
/// reported row computes the argmax once per scored position at the selected weights.
fn evaluate(
    f: &Fit,
    fam: &Family,
    lambdas: &[f64],
    vocab: usize,
    held: &[&Vec<u32>],
    accuracy: bool,
) -> (f64, f64, usize) {
    let mut bits = 0.0f64;
    let mut n = 0usize;
    let mut correct = 0usize;
    for seq in held {
        // A common position range for every family, so the per-token figures are comparable.
        let start = 2usize;
        if seq.len() <= start {
            continue;
        }
        for i in start..seq.len() {
            let next = seq[i];
            let ctxs: Vec<u64> = fam.contexts.iter().map(|cf| cf(seq, i)).collect();
            let p = family_p(f, fam, &ctxs, next, lambdas, vocab);
            bits -= p.max(1e-12).log2();
            if accuracy && blended_argmax(f, fam, &ctxs, lambdas, vocab) == next {
                correct += 1;
            }
            n += 1;
        }
    }
    (bits / n.max(1) as f64, correct as f64 / n.max(1) as f64, n)
}

/// Tune the interpolation weights on a validation split by held-out perplexity.
fn tune(f: &Fit, fam: &Family, vocab: usize, val: &[&Vec<u32>]) -> Vec<f64> {
    let n = fam.contexts.len();
    if n == 0 {
        return Vec::new();
    }
    let mut best = vec![0.5f64; n];
    let mut best_bits = f64::INFINITY;
    // Exhaustive over one or two weights; the grid is small.
    let mut lambda = vec![0.0f64; n];
    let mut stack = vec![0usize; n];
    loop {
        // Emit the current candidate.
        for (k, slot) in lambda.iter_mut().enumerate() {
            *slot = LAMBDA_GRID[stack[k]];
        }
        let (bits, _, _) = evaluate(f, fam, &lambda, vocab, val, false);
        if bits < best_bits {
            best_bits = bits;
            best.clone_from(&lambda);
        }
        // Advance the odometer.
        let mut carry = 0;
        loop {
            if carry == n {
                return best;
            }
            stack[carry] += 1;
            if stack[carry] < LAMBDA_GRID.len() {
                break;
            }
            stack[carry] = 0;
            carry += 1;
        }
    }
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// One full experiment on a (train, validation, held) split. Returns the reported rows.
fn run_split(
    name: &str,
    fit_seqs: &[&Vec<u32>],
    val_seqs: &[&Vec<u32>],
    held: &[&Vec<u32>],
    vocab: usize,
    held_bytes: usize,
) -> Vec<(String, f64, f64, usize)> {
    let f = fit_all(fit_seqs, vocab);

    let ctx_res1 = |s: &[u32], i: usize| -> u64 { (s[i - 1] as u64) % RADIX };
    let ctx_res2 = |s: &[u32], i: usize| -> u64 {
        ((s[i - 2] as u64) % RADIX) * RADIX + ((s[i - 1] as u64) % RADIX)
    };
    let ctx_true1 = |s: &[u32], i: usize| -> u64 { s[i - 1] as u64 };
    let ctx_true2 =
        |s: &[u32], i: usize| -> u64 { (s[i - 2] as u64) * vocab as u64 + (s[i - 1] as u64) };

    let families = [
        Family {
            name: "unigram",
            contexts: vec![],
            level: |_: &Fit, _: usize| unreachable!(),
            description: "none",
        },
        Family {
            name: "res1",
            contexts: vec![&ctx_res1],
            level: |f: &Fit, _: usize| &f.res1,
            description: "t-1 mod 120",
        },
        Family {
            name: "res2",
            contexts: vec![&ctx_res2, &ctx_res1],
            level: |f: &Fit, k: usize| if k == 0 { &f.res2 } else { &f.res1 },
            description: "(t-2 mod 120, t-1 mod 120)  [MECHANISM]",
        },
        Family {
            name: "true1",
            contexts: vec![&ctx_true1],
            level: |f: &Fit, _: usize| &f.true1,
            description: "t-1",
        },
        Family {
            name: "true2",
            contexts: vec![&ctx_true2, &ctx_true1],
            level: |f: &Fit, k: usize| if k == 0 { &f.true2 } else { &f.true1 },
            description: "(t-2, t-1)",
        },
    ];

    println!();
    println!("--- {name} ---");
    println!(
        "contexts: res1={} (mean {:.1} obs) res2={} (mean {:.1}) true1={} (mean {:.1}) true2={} (mean {:.1})",
        f.res1.totals.len(),
        f.res1.mean_obs(),
        f.res2.totals.len(),
        f.res2.mean_obs(),
        f.true1.totals.len(),
        f.true1.mean_obs(),
        f.true2.totals.len(),
        f.true2.mean_obs(),
    );
    println!(
        "model   context                                     lambda          bits/token   top1"
    );
    let mut rows = Vec::new();
    for fam in &families {
        let lambdas = if fam.contexts.is_empty() {
            Vec::new()
        } else {
            tune(&f, fam, vocab, val_seqs)
        };
        let (bits, acc, n) = evaluate(&f, fam, &lambdas, vocab, held, true);
        let lam = if lambdas.is_empty() {
            "-".to_string()
        } else {
            lambdas
                .iter()
                .map(|l| format!("{l:.1}"))
                .collect::<Vec<_>>()
                .join(",")
        };
        println!(
            "{:<7} {:<43} {:<13} {bits:8.4}  {acc:.4}",
            fam.name, fam.description, lam
        );
        rows.push((fam.name.to_string(), bits, acc, n));
    }
    // The project's own instrument is bits/byte, so the ceiling is reported in it too.
    if held_bytes > 0 {
        println!("bits/byte (held bytes = {held_bytes}):");
        for (nm, bits, _, n) in &rows {
            println!(
                "  {nm:<7} {:.4} bits/byte",
                bits * *n as f64 / held_bytes as f64
            );
        }
    }
    rows
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

    println!(
        "=== STATIC BACKOFF COMPARISON on real text (NOT an information-theoretic ceiling) ==="
    );
    println!("tokenizer    : {}", args.tokenizer.display());
    println!("corpus       : {}", args.corpus.display());
    println!("vocab target : {}", args.vocab);
    println!("held_every   : {}", args.held_every);
    println!("max_bytes    : {}", args.max_bytes);

    let original = match std::fs::read(&args.tokenizer) {
        Ok(b) => b,
        Err(e) => {
            eprintln!(
                "error: cannot read tokenizer {}: {e}",
                args.tokenizer.display()
            );
            return ExitCode::from(1);
        }
    };
    let derived = if args.vocab == 0 {
        original
    } else {
        match derive_tokenizer_json(&original, args.vocab) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("error: derive tokenizer: {e}");
                return ExitCode::from(1);
            }
        }
    };
    let Some(tokenizer) = HfBpeTokenizer::from_tokenizer_json_bytes(&derived) else {
        eprintln!("error: derived tokenizer.json is not a valid byte-level BPE tokenizer");
        return ExitCode::from(1);
    };
    let vocab = tokenizer.vocab_size();
    println!("tokenizer    : derived vocab = {vocab} tokens");

    let mut budget = args.max_bytes;
    let mut docs: Vec<String> = Vec::new();
    collect_documents(&args.corpus, &["rs", "md", "txt"], &mut budget, &mut docs);
    if docs.is_empty() {
        eprintln!(
            "error: no corpus documents found under {}",
            args.corpus.display()
        );
        return ExitCode::from(1);
    }
    let started = Instant::now();
    let sequences: Vec<Vec<u32>> = docs.iter().map(|d| tokenizer.encode(d)).collect();
    let total_tokens: usize = sequences.iter().map(|s| s.len()).sum();

    // Pin the exact corpus: the byte digest of the collected documents, in collection order.
    let mut hasher = Sha256::new();
    for d in &docs {
        hasher.update((d.len() as u64).to_le_bytes());
        hasher.update(d.as_bytes());
    }
    println!(
        "corpus digest: sha256:{:x} ({} documents)",
        hasher.finalize(),
        docs.len()
    );

    // Split: every held_every-th document is held out; every fifth training document is validation.
    let (mut train, mut held): (Vec<&Vec<u32>>, Vec<&Vec<u32>>) = (Vec::new(), Vec::new());
    for (i, s) in sequences.iter().enumerate() {
        if i % args.held_every == 0 {
            held.push(s);
        } else {
            train.push(s);
        }
    }
    let (mut fit_set, mut val): (Vec<&Vec<u32>>, Vec<&Vec<u32>>) = (Vec::new(), Vec::new());
    for (i, s) in train.drain(..).enumerate() {
        if i % 5 == 0 {
            val.push(s);
        } else {
            fit_set.push(s);
        }
    }
    let count = |v: &[&Vec<u32>]| v.iter().map(|s| s.len()).sum::<usize>();
    println!(
        "corpus       : documents={} tokens={} (fit={} val={} held={})  tokenized in {:.1}s",
        sequences.len(),
        total_tokens,
        count(&fit_set),
        count(&val),
        count(&held),
        started.elapsed().as_secs_f32()
    );
    if count(&fit_set) < 10_000 || count(&held) < 1_000 || count(&val) < 1_000 {
        eprintln!("error: corpus too small for a meaningful measurement");
        return ExitCode::from(1);
    }

    let held_bytes: usize = held.iter().map(|s| tokenizer.decode_bytes(s).len()).sum();
    let rows = run_split("real text", &fit_set, &val, &held, vocab, held_bytes);
    let get = |n: &str| {
        rows.iter()
            .find(|r| r.0 == n)
            .map(|r| r.1)
            .unwrap_or(f64::NAN)
    };
    let (u, r1, r2, t1, t2) = (
        get("unigram"),
        get("res1"),
        get("res2"),
        get("true1"),
        get("true2"),
    );
    println!();
    println!(
        "mechanism (res2) gain over unigram   : {:.4} bits/token",
        u - r2
    );
    println!(
        "res1 gain over unigram               : {:.4} bits/token",
        u - r1
    );
    println!(
        "extra gain from the 2nd residue (res2-res1): {:.4} bits/token",
        r1 - r2
    );
    println!(
        "true1 gain over unigram              : {:.4} bits/token",
        u - t1
    );
    println!(
        "true2 gain over unigram              : {:.4} bits/token",
        u - t2
    );
    println!(
        "extra gain from true 2nd token (true1-true2): {:.4} bits/token",
        t1 - t2
    );
    println!(
        "residue context recovers             : {:.2}% of the true order-2 gain",
        100.0 * (u - r2) / (u - t2).max(1e-9)
    );

    // --- control: shuffle the whole corpus and refit end to end -------------
    if args.shuffle_control {
        let mut st = args.seed | 1;
        let mut flat: Vec<u32> = sequences.iter().flat_map(|s| s.iter().copied()).collect();
        for i in (1..flat.len()).rev() {
            let j = (xorshift(&mut st) as usize) % (i + 1);
            flat.swap(i, j);
        }
        // Re-chunk into equal pieces so the same document structure is preserved.
        let chunk = (total_tokens / sequences.len()).max(64);
        let shuffled: Vec<Vec<u32>> = flat.chunks(chunk).map(|c| c.to_vec()).collect();
        let (mut str_, mut sh): (Vec<&Vec<u32>>, Vec<&Vec<u32>>) = (Vec::new(), Vec::new());
        for (i, s) in shuffled.iter().enumerate() {
            if i % args.held_every == 0 {
                sh.push(s);
            } else {
                str_.push(s);
            }
        }
        let (mut sf, mut sv): (Vec<&Vec<u32>>, Vec<&Vec<u32>>) = (Vec::new(), Vec::new());
        for (i, s) in str_.drain(..).enumerate() {
            if i % 5 == 0 {
                sv.push(s);
            } else {
                sf.push(s);
            }
        }
        if count(&sf) >= 10_000 && count(&sh) >= 1_000 {
            let sh_bytes: usize = sh.iter().map(|s| tokenizer.decode_bytes(s).len()).sum();
            let srows = run_split(
                "control: shuffled corpus (refit)",
                &sf,
                &sv,
                &sh,
                vocab,
                sh_bytes,
            );
            let sget = |n: &str| {
                srows
                    .iter()
                    .find(|r| r.0 == n)
                    .map(|r| r.1)
                    .unwrap_or(f64::NAN)
            };
            println!();
            println!("control gains (expect ~0):");
            println!(
                "  res2 {:.4} | res1 {:.4} | true1 {:.4} | true2 {:.4} bits/token",
                sget("unigram") - sget("res2"),
                sget("unigram") - sget("res1"),
                sget("unigram") - sget("true1"),
                sget("unigram") - sget("true2"),
            );
        } else {
            println!("\ncontrol skipped: shuffled split too small");
        }
    }

    println!();
    println!("elapsed: {:.1}s", started.elapsed().as_secs_f32());
    println!(
        "scope: fixed element(token)=token%120, order<=2 — the mechanism AS CONFIGURED. \
         Not a verdict on the geometric architecture; learned packaging is untested here."
    );
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic 1000-token byte-level BPE `tokenizer.json` whose merges all produce tokens
    /// that are present in the vocabulary, so truncation is exactly observable.
    fn synthetic_tokenizer(total: usize) -> Vec<u8> {
        let base = 256usize;
        let mut vocab = serde_json::Map::new();
        for k in 0..base {
            vocab.insert(format!("a{k}"), serde_json::Value::from(k as u64));
        }
        for k in base..total {
            vocab.insert(
                format!("m{j}m{j}", j = k - base),
                serde_json::Value::from(k as u64),
            );
        }
        let merges: Vec<serde_json::Value> = (0..total - base)
            .map(|j| {
                serde_json::Value::Array(vec![
                    serde_json::Value::from(format!("m{j}")),
                    serde_json::Value::from(format!("m{j}")),
                ])
            })
            .collect();
        let root = serde_json::json!({
            "model": { "type": "BPE", "vocab": vocab, "merges": merges },
            "added_tokens": [],
        });
        serde_json::to_vec(&root).expect("serialize")
    }

    fn inspect(json: &[u8]) -> (usize, usize) {
        let v: serde_json::Value = serde_json::from_slice(json).expect("parse");
        let model = v.get("model").expect("model");
        (
            model
                .get("vocab")
                .and_then(serde_json::Value::as_object)
                .expect("vocab")
                .len(),
            model
                .get("merges")
                .and_then(serde_json::Value::as_array)
                .expect("merges")
                .len(),
        )
    }

    #[test]
    fn truncation_keeps_exactly_the_dense_prefix_and_its_merges() {
        let original = synthetic_tokenizer(1000);
        let derived = derive_tokenizer_json(&original, 500).expect("derive");
        let (vocab, merges) = inspect(&derived);
        assert_eq!(vocab, 500, "vocabulary must be the requested dense prefix");
        // Products surviving id < 500 are ids 256..=499, i.e. 244 of the 744 merges.
        assert_eq!(
            merges, 244,
            "only merges whose product survives may be kept"
        );
    }

    #[test]
    fn a_larger_request_than_the_source_is_rejected() {
        let original = synthetic_tokenizer(1000);
        assert!(derive_tokenizer_json(&original, 5000).is_err());
    }

    #[test]
    fn a_non_dense_prefix_is_rejected() {
        // Remove id 400 so the prefix 0..1000 has a hole.
        let original = synthetic_tokenizer(1000);
        let mut v: serde_json::Value = serde_json::from_slice(&original).expect("parse");
        let vocab = v["model"]["vocab"].as_object_mut().expect("vocab");
        vocab.retain(|_, id| id.as_u64() != Some(400));
        let holed = serde_json::to_vec(&v).expect("serialize");
        assert!(
            derive_tokenizer_json(&holed, 1000).is_err(),
            "a vocabulary with a hole must not be accepted as a dense prefix"
        );
    }

    #[test]
    fn lambda_zero_is_exactly_the_unigram_model() {
        // Cross-entropy and top-1 must come from the same interpolated distribution, and at
        // lambda = 0 that distribution is the unigram one, in probability *and* in argmax.
        let vocab = 8usize;
        let seq: Vec<u32> = vec![1, 2, 3, 1, 2, 3, 2, 2, 1];
        let f = fit_all(&[&seq], vocab);
        let ctx = |s: &[u32], i: usize| -> u64 { s[i - 1] as u64 };
        let fam = Family {
            name: "res1",
            contexts: vec![&ctx],
            level: |f: &Fit, _: usize| &f.res1,
            description: "test fixture",
        };
        let ctxs = vec![ctx(&seq, 4)];
        for next in 0..vocab as u32 {
            let p = family_p(&f, &fam, &ctxs, next, &[0.0], vocab);
            assert!(
                (p - f.unigram_p(next, vocab)).abs() < 1e-12,
                "lambda=0 must reproduce the unigram probability exactly"
            );
        }
        assert_eq!(
            blended_argmax(&f, &fam, &ctxs, &[0.0], vocab),
            f.uni_argmax,
            "lambda=0 must reproduce the unigram argmax"
        );
        // And the interpolated distribution normalises over the vocabulary at any weight.
        let sum: f64 = (0..vocab as u32)
            .map(|n| family_p(&f, &fam, &ctxs, n, &[0.7], vocab))
            .sum();
        assert!(
            (sum - 1.0).abs() < 1e-9,
            "the interpolated distribution must normalise, got {sum}"
        );
    }
}
