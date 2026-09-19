//! Shortlist routing recall under different VSA codebook modes.
//!
//! # Why this instrument exists
//!
//! `ablate-prose` scores the **full vocabulary**, so it cannot see the routing effect of a
//! codebook change. But the VSA context vector is also a routing input: it feeds
//! `HierarchicalCodebook::route_shortlist_q30_with_memory` (runtime.rs:1200), where the coarse
//! H4 sector ranking uses the predicted geometric state `s3` while the **fine leaf-bucket**
//! selection compares the context hypervector against bucket centroids. A codebook change
//! therefore acts on routing through the fine level, and that path was unmeasured.
//!
//! # The coherence requirement this tool makes explicit
//!
//! The artifact stores a `HierarchicalCodebook` whose centroids are bundles of the token
//! vectors used **at export time** — the fixed `splitmix64(vsa_seed, token_id)` codes. A
//! root-derived query vector compared against those centroids would be meaningless, so the
//! `root` mode must **rebuild** the codebook in the matching space via
//! `HierarchicalCodebook::new`. This is not optional: a codebook change without a matching
//! rebuild would measure an incoherent system.
//!
//! # What is measured
//!
//! Per scored position, for each mode:
//!   * `recall` — fraction of positions whose teacher-forced target appears in the 64-slot
//!     routed shortlist. A random 64-of-4096 shortlist gives 1.56 %, which is the floor.
//!   * `mean routed size` — how many slots the router actually fills.
//!   * `mean routing rank` of the target when present.
//! Plus, between the two modes: mean Jaccard overlap of the routed sets, i.e. **how much the
//! codebook changes routing at all**.
//!
//! The Voronoi leg is isolated by passing an empty memory-seed list; the served path also
//! injects engram/induction candidates first, which this tool deliberately excludes.
//!
//! Offline measurement harness. Integer-only routing; no training.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use uor_r4_core::native_geometric::hopf_metric::UnitS3Q30;
use uor_r4_core::native_geometric::learner::{build_root_codebook, ExportedGeometricModel};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::native_geometric::vsa::{
    encode_attended_multiscale_context, Codebook, HierarchicalCodebook, Hypervector, Shortlist,
};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_MODEL: &str = "native_geometric_prose_model.rgm";
const DEFAULT_CORPUS: &str = "tinystories_train.u16";
const DEFAULT_SEQ_LEN: usize = 64;
const DEFAULT_POSITIONS: usize = 2_048;
const SHORTLIST_SLOTS: usize = 64;
/// The trainer's held-out token count; byte lengths come from the tokenizer.
const DEFAULT_TOKENIZER: &str = ".uor-models/research/issue-1014/export/tokenizer.json";
/// Serving sampler divisor: `exp((s - max) / (temperature * 8192))` at temperature 1.0.
const SAMPLER_SCALE: f64 = 8192.0;

struct Args {
    model: PathBuf,
    corpus: PathBuf,
    tokenizer: PathBuf,
    seq_len: usize,
    positions: usize,
    holdout_offset: usize,
}

fn usage() -> String {
    format!(
        "shortlist-recall -- routed-shortlist recall under VSA codebook modes\n\
         \n\
         USAGE:\n  shortlist-recall [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \x20 --model <path>          artifact (.rgm or .json)  [default: {DEFAULT_MODEL}]\n\
         \x20 --corpus <path>         .u16 token stream         [default: {DEFAULT_CORPUS}]\n\
         \x20 --tokenizer <path>      tokenizer.json (byte lengths for BPB) [default: default]\n\
         \x20 --seq-len <n>           teacher-forced chunk size [default: {DEFAULT_SEQ_LEN}]\n\
         \x20 --positions <n>         positions to score        [default: {DEFAULT_POSITIONS}]\n\
         \x20 --holdout-offset <n>    token offset of the slice [default: 0]\n\
         \x20 --help\n\
         \n\
         MODES measured: `fixed` (artifact's stored codebook) and `root` (codes derived from\n\
         the learned 120-root assignment, with the hierarchical codebook REBUILT to match).\n"
    )
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        model: PathBuf::from(DEFAULT_MODEL),
        corpus: PathBuf::from(DEFAULT_CORPUS),
        tokenizer: PathBuf::from(DEFAULT_TOKENIZER),
        seq_len: DEFAULT_SEQ_LEN,
        positions: DEFAULT_POSITIONS,
        holdout_offset: 0,
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let val = |i: usize| -> Result<&String, String> {
            argv.get(i + 1)
                .ok_or_else(|| format!("missing value for {}", argv[i]))
        };
        match argv[i].as_str() {
            "--help" | "-h" => return Err(usage()),
            "--model" => {
                args.model = PathBuf::from(val(i)?);
                i += 2;
            }
            "--corpus" => {
                args.corpus = PathBuf::from(val(i)?);
                i += 2;
            }
            "--tokenizer" => {
                args.tokenizer = PathBuf::from(val(i)?);
                i += 2;
            }
            "--seq-len" => {
                args.seq_len = val(i)?.parse().map_err(|_| "bad --seq-len".to_string())?;
                i += 2;
            }
            "--positions" => {
                args.positions = val(i)?.parse().map_err(|_| "bad --positions".to_string())?;
                i += 2;
            }
            "--holdout-offset" => {
                args.holdout_offset = val(i)?
                    .parse()
                    .map_err(|_| "bad --holdout-offset".to_string())?;
                i += 2;
            }
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }
    if args.seq_len < 2 {
        return Err("--seq-len must be >= 2".into());
    }
    if args.positions == 0 {
        return Err("--positions must be >= 1".into());
    }
    Ok(args)
}

/// One routing configuration: a codebook plus the hierarchical codebook built in its space.
struct Router {
    label: &'static str,
    codebook: Codebook<64>,
    hierarchical: HierarchicalCodebook<64>,
}

impl Router {
    /// The context hypervector in this mode's codebook space.
    fn context_vsa(&self, ctx: &[usize]) -> Hypervector<64> {
        let mut buf = [0u32; 64];
        let n = ctx.len().min(64);
        for i in 0..n {
            buf[i] = ctx[ctx.len() - n + i] as u32;
        }
        encode_attended_multiscale_context(&buf[..n], &self.codebook, 64)
    }

    /// Route one position. `s3` is the predicted geometric state, which is codebook-independent
    /// (it accumulates canonical root quaternions, not VSA codes) and is computed once per
    /// position and shared across modes.
    fn route(&self, s3: Option<UnitS3Q30>, vsa: &Hypervector<64>) -> Shortlist<SHORTLIST_SLOTS> {
        // The coarse sector ranking uses `s3`; the fine leaf-bucket selection compares `vsa`
        // against the bucket centroids, which is where the codebook acts. An empty seed list
        // isolates the Voronoi leg from the engram/induction seeds.
        self.hierarchical
            .route_shortlist_q30_with_memory::<SHORTLIST_SLOTS>(s3, Some(vsa), &[])
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return Ok(());
        }
    };
    let started = Instant::now();

    let bytes = std::fs::read(&args.model)?;
    let model = if bytes.len() >= 4 && &bytes[..4] == b"RGM1" {
        ExportedGeometricModel::from_binary(&bytes)?
    } else {
        serde_json::from_slice::<ExportedGeometricModel>(&bytes)?
    };
    let vocab = model.vocab_size;

    let tok_bytes = std::fs::read(&args.tokenizer)?;
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tok_bytes)
        .ok_or_else(|| format!("failed to parse tokenizer {}", args.tokenizer.display()))?;
    let token_lens = tokenizer.token_byte_lengths();

    let reader = MmapCorpusReader::open(&args.corpus)?;
    let corpus = reader.as_slice();
    let needed = args
        .positions
        .next_multiple_of(args.seq_len)
        .saturating_add(args.seq_len);
    let slice_start = args.holdout_offset.min(corpus.len());
    let slice_end = slice_start.saturating_add(needed).min(corpus.len());
    if slice_end <= slice_start {
        return Err("empty held-out slice".into());
    }
    let held_out = &corpus[slice_start..slice_end];

    let stored = model
        .hierarchical_codebook
        .clone()
        .ok_or("artifact has no hierarchical codebook; routing recall is undefined")?;

    let fixed_codebook = Codebook::<64>::on_demand(vocab, model.vsa_seed);
    let root_codebook = build_root_codebook(vocab, &model.token_to_root, model.vsa_seed);
    let root_hierarchical =
        HierarchicalCodebook::<64>::new(vocab, &model.token_to_root, &root_codebook);

    let routers = [
        Router {
            label: "fixed",
            codebook: fixed_codebook,
            hierarchical: stored,
        },
        Router {
            label: "root",
            codebook: root_codebook,
            hierarchical: root_hierarchical,
        },
    ];

    println!("shortlist-recall -- routed-shortlist recall under VSA codebook modes");
    println!("  model     : {} (vocab {vocab})", args.model.display());
    println!(
        "  corpus    : {} (slice [{}..{}) = {} tokens)",
        args.corpus.display(),
        slice_start,
        slice_end,
        held_out.len()
    );
    println!(
        "  shortlist : {SHORTLIST_SLOTS} slots; random-shortlist recall floor = {:.2} %",
        100.0 * SHORTLIST_SLOTS as f64 / vocab as f64
    );
    println!("  seeds     : none (Voronoi leg isolated from engram/induction seeds)");
    println!();

    let mut hits = [0usize; 2];
    let mut totals = 0usize;
    let mut size_sum = [0usize; 2];
    let mut rank_sum = [0usize; 2];
    let mut rank_n = [0usize; 2];
    let mut inter_sum = 0usize;
    let mut union_sum = 0usize;
    let mut identical = 0usize;
    // Served-path bits: routing plus scoring, with an unrouted target charged as unreachable.
    let mut bits_sum = [0.0f64; 2];
    let mut bytes_sum = [0u64; 2];
    let mut unrouted = [0usize; 2];
    let mut ctx: Vec<usize> = Vec::with_capacity(args.seq_len);
    let mut scores = [i32::MIN; SHORTLIST_SLOTS];

    'outer: for chunk in held_out.chunks_exact(args.seq_len) {
        ctx.clear();
        for j in 1..args.seq_len {
            if totals >= args.positions {
                break 'outer;
            }
            ctx.push(chunk[j - 1] as usize);
            let target = chunk[j] as u32;
            if target as usize >= vocab {
                continue;
            }

            let fiber = model.context_predicted_fiber_q30(&ctx);
            let s3 = fiber.to_unit_s3_q30();
            let target_bytes = token_lens.get(target as usize).copied().unwrap_or(1).max(1) as u64;
            let mut sets: [BTreeSet<u32>; 2] = [BTreeSet::new(), BTreeSet::new()];

            for (m, router) in routers.iter().enumerate() {
                let vsa = router.context_vsa(&ctx);
                let sl = router.route(Some(s3), &vsa);
                let slice = sl.as_slice();
                size_sum[m] += slice.len();
                if let Some(pos) = slice.iter().position(|&t| t == target) {
                    hits[m] += 1;
                    rank_sum[m] += pos;
                    rank_n[m] += 1;
                }

                // Score exactly the routed candidates, as serving does.
                let mut max_score = i32::MIN;
                for (k, &cand) in slice.iter().enumerate() {
                    let s = model.score_context_candidate_with_vsa(
                        &ctx,
                        cand as usize,
                        fiber,
                        Some((&router.codebook, &vsa)),
                    );
                    scores[k] = s;
                    if s > max_score {
                        max_score = s;
                    }
                }
                let mut sum = 0.0f64;
                let mut target_w = 0.0f64;
                for (k, &cand) in slice.iter().enumerate() {
                    let w = libm::exp(
                        (((scores[k] - max_score) as f64) / SAMPLER_SCALE).clamp(-60.0, 0.0),
                    );
                    sum += w;
                    if cand == target {
                        target_w = w;
                    }
                }
                let bits = if target_w > 0.0 && sum > 0.0 {
                    -(target_w / sum).log2()
                } else {
                    // Unreachable: not routed, so no score can select it. Charged at the cost of
                    // guessing among the whole vocabulary, a conservative floor.
                    unrouted[m] += 1;
                    (vocab as f64).log2()
                };
                bits_sum[m] += bits;
                bytes_sum[m] += target_bytes;
                sets[m] = slice.iter().copied().collect();
            }
            totals += 1;
            inter_sum += sets[0].intersection(&sets[1]).count();
            union_sum += sets[0].union(&sets[1]).count();
            if sets[0] == sets[1] {
                identical += 1;
            }
        }
    }

    println!("positions scored: {totals}");
    println!();
    println!(
        "{:<8} {:>10} {:>12} {:>12} {:>12} {:>12}",
        "mode", "recall", "mean_size", "mean_rank", "shortlistBPB", "unrouted"
    );
    for (m, router) in routers.iter().enumerate() {
        let recall = 100.0 * hits[m] as f64 / totals as f64;
        let mean_size = size_sum[m] as f64 / totals as f64;
        let mean_rank = if rank_n[m] > 0 {
            rank_sum[m] as f64 / rank_n[m] as f64
        } else {
            f64::NAN
        };
        let bpb = if bytes_sum[m] > 0 {
            bits_sum[m] / bytes_sum[m] as f64
        } else {
            f64::NAN
        };
        println!(
            "{:<8} {:>9.2}% {:>12.1} {:>12.2} {:>12.4} {:>11.2}%",
            router.label,
            recall,
            mean_size,
            mean_rank,
            bpb,
            100.0 * unrouted[m] as f64 / totals as f64
        );
    }
    println!();
    if union_sum > 0 {
        println!(
            "routing agreement between modes: mean Jaccard {:.4}; identical shortlist on {:.1} % of positions",
            inter_sum as f64 / union_sum as f64,
            100.0 * identical as f64 / totals as f64
        );
    }
    println!("elapsed_total_s: {:.1}", started.elapsed().as_secs_f64());
    println!();
    println!("Interpretation:");
    println!("  `recall` is the routed shortlist's ceiling on served quality; the floor is 1.56 %");
    println!("  for random 64-of-4096. `shortlistBPB` is the served path: route, then softmax");
    println!(
        "  over exactly the routed candidates, with an unrouted target charged at log2(vocab)"
    );
    println!("  because no score can select a candidate that was never routed.");
    println!("  The engram/induction seeds that the served path also injects are excluded here.");
    Ok(())
}
