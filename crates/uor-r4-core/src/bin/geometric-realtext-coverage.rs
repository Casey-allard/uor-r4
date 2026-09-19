//! Real-text **causal route coverage** for the order-2 geometric addressed-memory core.
//!
//! # The question this answers, before any fit
//!
//! The core resets state between training sequences. At `order = 2` it writes the value of token `i`
//! at the address of the pair `(t_{i-2}, t_{i-1})` and reads the address of the pair `(t_{i-1}, t_i)`.
//! A read therefore only finds a write when the same ordered pair occurred earlier in the *same*
//! window; on a first occurrence the bucket is empty, the read is the zero vector, the default
//! readout has no bias, the logits are zero and the prediction is uniform. Its loss is exactly
//! `ln V` nats = `log2 V` bits, which is **12 bits/token at V = 4096**.
//!
//! So the default path's mean loss is at least `p_empty · log2 V`, and `p_empty` must be measured
//! under the actual window/reset policy before any target or long run is justified. That is what this
//! tool measures, together with the route statistics that decide whether a learner could use these
//! routes at all.
//!
//! # What it reports
//!
//! For every scored position (the positions the trainer would score) inside each window:
//!
//! * **empty vs previously-written** query routes, in total and split by early/late position, since
//!   cold routes concentrate at the start of a window;
//! * the implied floor `p_empty · log2 V` in bits/token;
//! * **exact-context collisions**: how many *distinct* exact pairs `(t_{i-1}, t_i)` share one
//!   address. A distinct-context count above one means the address is a many-to-one function of the
//!   context — the modulo structure that a static count model cannot separate and a learned value
//!   table might;
//! * **distinct successors** written per address, which bounds how much a bucket can carry;
//! * per-document `p_empty` spread, so a corpus mean is not hiding a bimodal split.
//!
//! # What it does not claim
//!
//! Token-only counting establishes **unwritten** buckets. It cannot see vector **cancellation**, where
//! written values sum to zero. That needs pinned value weights, so a sampled second pass measures it
//! with a **named, seeded initialisation** and reports it as initialization-specific, never as learned
//! behaviour. No model is trained here, and no language capability is claimed.

#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::geometric_attention::{
    element_table, word_ending_at, GeometricAttentionTrainer,
};
use uor_r4_core::transformerless::bpe_derive::derive_tokenizer;

const RADIX: usize = 120;

struct Args {
    tokenizer: PathBuf,
    vocab: usize,
    corpus: PathBuf,
    windows: Vec<usize>,
    max_bytes: usize,
    cancel_sample: usize,
    dv: usize,
    seed: u64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            tokenizer: PathBuf::from(
                "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json",
            ),
            vocab: 4096,
            corpus: PathBuf::from("/Users/casey.allard/uor-r4/.worktrees/realtext-qual/crates"),
            windows: vec![64, 128, 256],
            max_bytes: 4 * 1024 * 1024,
            cancel_sample: 128,
            dv: 128,
            seed: 2026_0919,
        }
    }
}

fn usage(program: &str) {
    eprintln!(
        "Usage: {program} [OPTIONS]\n\
         \x20 --tokenizer <path>   source tokenizer.json\n\
         \x20 --vocab <n>          derived vocabulary size (default 4096)\n\
         \x20 --corpus <path>      file or directory of real text\n\
         \x20 --windows <a,b,..>   window sizes for the coverage comparison (default 64,128,256)\n\
         \x20 --max-bytes <n>      corpus byte budget (default 4 MiB)\n\
         \x20 --cancel-sample <n>  windows sampled for the cancellation check (0 disables)\n\
         \x20 --dv <n>             value width for the seeded cancellation pass (default 128)\n\
         \x20 --seed <n>           seed for the named cancellation initialisation\n\
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
            "--windows" => {
                a.windows = val()?
                    .split(',')
                    .map(|s| s.trim().parse::<usize>().map_err(|e| format!("{e}")))
                    .collect::<Result<Vec<_>, _>>()?;
                a.windows.retain(|w| *w >= 4);
                if a.windows.is_empty() {
                    return Err("--windows needs at least one size >= 4".into());
                }
            }
            "--max-bytes" => a.max_bytes = val()?.parse().map_err(|e| format!("{e}"))?,
            "--cancel-sample" => a.cancel_sample = val()?.parse().map_err(|e| format!("{e}"))?,
            "--dv" => a.dv = val()?.parse().map_err(|e| format!("{e}"))?,
            "--seed" => a.seed = val()?.parse().map_err(|e| format!("{e}"))?,
            "--help" | "-h" => return Err("help".into()),
            other => return Err(format!("unknown argument {other}")),
        }
        i += 2;
    }
    Ok(a)
}

/// Recursively collect text documents, bounded by a byte budget.
///
/// Skips build output, version control and the isolated worktree directory so the snapshot cannot
/// recurse into a copy of itself.
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
            .is_some_and(|n| matches!(n, "target" | ".git" | ".worktrees"))
        {
            continue;
        }
        collect(&child, budget, out);
    }
}

#[inline]
fn addr(elements: &[u16], vocab: usize, a: u32, b: u32) -> usize {
    let ea = elements[(a as usize).min(vocab - 1)] as usize;
    let eb = elements[(b as usize).min(vocab - 1)] as usize;
    ea * RADIX + eb
}

/// Coverage over one window size. Returns the aggregate counters.
struct Coverage {
    scored: usize,
    empty: usize,
    early_scored: usize,
    early_empty: usize,
    /// Sum over addresses of (distinct exact contexts - 1); divided by addresses this is the mean
    /// over-count caused by the many-to-one address map.
    collision_extra: usize,
    addresses: usize,
    successors_total: usize,
    successors_max: usize,
    /// Per-document empty fractions, for spread.
    doc_fracs: Vec<f64>,
}

fn measure_window(
    windows: &[Vec<u32>],
    elements: &[u16],
    vocab: usize,
    doc_of: &[usize],
    docs: usize,
) -> Coverage {
    let mut c = Coverage {
        scored: 0,
        empty: 0,
        early_scored: 0,
        early_empty: 0,
        collision_extra: 0,
        addresses: 0,
        successors_total: 0,
        successors_max: 0,
        doc_fracs: Vec::new(),
    };
    let mut per_doc: Vec<(usize, usize)> = vec![(0, 0); docs];
    let mut written: HashMap<usize, u32> = HashMap::new();
    let mut distinct_ctx: HashMap<usize, HashSet<u64>> = HashMap::new();
    let mut successors: HashMap<usize, HashSet<u32>> = HashMap::new();

    for (wi, w) in windows.iter().enumerate() {
        written.clear();
        distinct_ctx.clear();
        successors.clear();
        let n = w.len();
        if n < 3 {
            continue;
        }
        for i in 0..n - 1 {
            if i >= 2 {
                // The write at step i stores value(t_i) at the pair (t_{i-2}, t_{i-1}).
                let wa = addr(elements, vocab, w[i - 2], w[i - 1]);
                *written.entry(wa).or_insert(0) += 1;
                successors.entry(wa).or_default().insert(w[i]);
            }
            // The read at step i addresses the pair (t_{i-1}, t_i), left-padded on a short prefix.
            let ctx = word_ending_at(w, i, 2);
            let ra = addr(elements, vocab, ctx[0], ctx[1]);
            let key = ((ctx[0] as u64) << 32) | ctx[1] as u64;
            distinct_ctx.entry(ra).or_default().insert(key);

            let (d, s) = per_doc[doc_of[wi]];
            per_doc[doc_of[wi]] = (d + 1, s);
            c.scored += 1;
            if i < 8 {
                c.early_scored += 1;
            }
            if written.contains_key(&ra) {
                // previously written
            } else {
                c.empty += 1;
                if i < 8 {
                    c.early_empty += 1;
                }
                per_doc[doc_of[wi]].1 += 1;
            }
        }
        for ctxs in distinct_ctx.values() {
            c.collision_extra += ctxs.len().saturating_sub(1);
        }
        c.addresses += distinct_ctx.len();
        for succs in successors.values() {
            c.successors_total += succs.len();
            c.successors_max = c.successors_max.max(succs.len());
        }
    }
    c.doc_fracs = per_doc
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, e)| *e as f64 / *n as f64)
        .collect();
    c
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
    println!(
        "=== real-text causal route coverage (order 2, fixed element(token) = token % 120) ==="
    );
    println!("tokenizer : {}", args.tokenizer.display());
    println!("corpus    : {}", args.corpus.display());
    println!("vocab     : {}", args.vocab);
    println!("windows   : {:?}", args.windows);

    let original = match std::fs::read(&args.tokenizer) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read tokenizer: {e}");
            return ExitCode::from(1);
        }
    };
    println!("tokenizer digest: sha256:{:x}", Sha256::digest(&original));
    let tokenizer = match derive_tokenizer(&original, args.vocab) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: derive tokenizer: {e}");
            return ExitCode::from(1);
        }
    };
    let vocab = tokenizer.vocab_size();
    if vocab != args.vocab {
        eprintln!("error: derived vocab {vocab} != requested {}", args.vocab);
        return ExitCode::from(1);
    }

    let mut budget = args.max_bytes;
    let mut docs_text: Vec<String> = Vec::new();
    collect(&args.corpus, &mut budget, &mut docs_text);
    if docs_text.is_empty() {
        eprintln!("error: no documents found under {}", args.corpus.display());
        return ExitCode::from(1);
    }
    let mut hasher = Sha256::new();
    for d in &docs_text {
        hasher.update((d.len() as u64).to_le_bytes());
        hasher.update(d.as_bytes());
    }
    println!(
        "corpus digest: sha256:{:x} ({} documents)",
        hasher.finalize(),
        docs_text.len()
    );

    let sequences: Vec<Vec<u32>> = docs_text.iter().map(|d| tokenizer.encode(d)).collect();
    let total_tokens: usize = sequences.iter().map(|s| s.len()).sum();
    let elements = element_table(vocab);
    println!(
        "corpus    : documents={} tokens={}",
        sequences.len(),
        total_tokens
    );

    let bits_per_empty = (vocab as f64).log2();
    println!("an empty route costs log2(V) = {bits_per_empty:.4} bits/token at V={vocab}\n");

    // --- coverage for each window size --------------------------------------
    for &wlen in &args.windows {
        let mut windows: Vec<Vec<u32>> = Vec::new();
        let mut doc_of: Vec<usize> = Vec::new();
        for (di, s) in sequences.iter().enumerate() {
            for chunk in s.chunks(wlen) {
                if chunk.len() >= 3 {
                    windows.push(chunk.to_vec());
                    doc_of.push(di);
                }
            }
        }
        let c = measure_window(&windows, &elements, vocab, &doc_of, sequences.len());
        if c.scored == 0 {
            println!("window {wlen}: no scorable positions\n");
            continue;
        }
        let p_empty = c.empty as f64 / c.scored as f64;
        let early = if c.early_scored > 0 {
            c.early_empty as f64 / c.early_scored as f64
        } else {
            0.0
        };
        let mean_ctx = if c.addresses > 0 {
            c.collision_extra as f64 / c.addresses as f64 + 1.0
        } else {
            0.0
        };
        let doc_min = c.doc_fracs.iter().cloned().fold(f64::INFINITY, f64::min);
        let doc_max = c.doc_fracs.iter().cloned().fold(0.0f64, f64::max);
        let doc_mean = if c.doc_fracs.is_empty() {
            0.0
        } else {
            c.doc_fracs.iter().sum::<f64>() / c.doc_fracs.len() as f64
        };
        println!("--- window {wlen} tokens ---");
        println!("  windows={} scored_positions={}", windows.len(), c.scored);
        println!(
            "  empty routes {}/{} = p_empty {:.4}   (first 8 positions of a window: {:.4})",
            c.empty, c.scored, p_empty, early
        );
        println!("  previously written routes: {:.4}", 1.0 - p_empty);
        println!(
            "  FLOOR from cold routes: p_empty * log2(V) = {:.4} bits/token",
            p_empty * bits_per_empty
        );
        println!(
            "  distinct exact contexts per address: mean {mean_ctx:.3} over {} addresses (1.000 = injective on this window)",
            c.addresses
        );
        println!(
            "  distinct successors per address: mean {:.3}, max {}",
            if c.addresses > 0 {
                c.successors_total as f64 / c.addresses as f64
            } else {
                0.0
            },
            c.successors_max
        );
        println!("  per-document p_empty: min {doc_min:.4} mean {doc_mean:.4} max {doc_max:.4}");
        println!();
    }

    // --- cancellation with a named, seeded initialisation --------------------
    if args.cancel_sample > 0 {
        let wlen = args.windows[0];
        let mut windows: Vec<Vec<u32>> = Vec::new();
        for s in sequences.iter() {
            for chunk in s.chunks(wlen) {
                if chunk.len() >= 3 && windows.len() < args.cancel_sample {
                    windows.push(chunk.to_vec());
                }
            }
            if windows.len() >= args.cancel_sample {
                break;
            }
        }
        let trainer = match GeometricAttentionTrainer::new(vocab, args.dv, 2, 6, args.seed) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: build trainer: {e}");
                return ExitCode::from(1);
            }
        };
        // Quantised value table of the named initialisation, so cancellation is measurable.
        let core = match trainer.to_core() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: to_core: {e}");
                return ExitCode::from(1);
            }
        };
        let mut written = 0usize;
        let mut cancelled = 0usize;
        let mut state = core.initial_state();
        for w in &windows {
            for slot in state.iter_mut() {
                *slot = 0;
            }
            let n = w.len();
            for i in 0..n - 1 {
                if i >= 2 {
                    core.observe(&mut state, &w[i - 2..i], w[i]);
                }
                let ctx = word_ending_at(w, i, 2);
                if core.route_count(&state, &ctx) > 0 {
                    let a = addr(&elements, vocab, ctx[0], ctx[1]);
                    let base = a * core.dv;
                    written += 1;
                    if state[base..base + core.dv].iter().all(|&v| v == 0) {
                        cancelled += 1;
                    }
                }
            }
        }
        println!("--- vector cancellation (SEEDED INITIALISATION, not learned) ---");
        println!(
            "  dv={} seed={} windows={} written_routes={}",
            args.dv,
            args.seed,
            windows.len(),
            written
        );
        println!(
            "  written routes whose value vector is exactly zero: {}/{} = {:.4}",
            cancelled,
            written,
            if written > 0 {
                cancelled as f64 / written as f64
            } else {
                0.0
            }
        );
        println!(
            "  scope: this is one seeded initialisation of the value table. It bounds cancellation\n\
             \x20 attributable to that table, not a learned or trained behaviour."
        );
        println!();
    }

    println!("elapsed: {:.1}s", started.elapsed().as_secs_f32());
    println!(
        "scope: token-only coverage establishes UNWRITTEN buckets. No model was trained, no language\n\
         capability is claimed, and this says nothing about the group-composition path."
    );
    ExitCode::SUCCESS
}
