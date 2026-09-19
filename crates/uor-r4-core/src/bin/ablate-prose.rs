//! Per-mechanism ablation BPB sweep for the discrete geometric prose artifact (Card P7).
//!
//! Measures the held-out contribution of each mechanism that contributes to the
//! discrete scorer by disabling exactly one of them at a time in an in-memory copy
//! of the artifact and re-evaluating. No training, no fitting, no new draw: this is
//! a diagnostic, and the held-out split it uses is already-open development evidence.
//!
//! # What is measured
//!
//! Held-out **bits-per-byte under the discrete model's own additive scorer**
//! (`ExportedGeometricModel::score_context_candidate_with_vsa`), normalised over the
//! **full vocabulary** with the serving sampler's scaling
//! `exp((s - max) / (temperature * 8192))` at `temperature = 1.0`.
//!
//! This is deliberately *not* `JepaTrainer::evaluate_bpb_with_engram`, which:
//!   * runs on the continuous `f64` trainer, not on the quantised artifact that serves;
//!   * interpolates a second, different probability model (`root_prob * leaf_prob`)
//!     with engram probabilities at lambda 0.85..0.30.
//! The two are different scorers. Every number this tool prints is reproducible from
//! the artifact bytes alone; the training-time number is not.
//!
//! # Exactness
//!
//! Full-vocabulary normalisation (vocab is 4096) rather than the 64-candidate serving
//! shortlist, so the value is an upper bound on what the served path can achieve and is
//! free of shortlist-recall confounds. Ablation deltas are **paired** on identical
//! positions, so their bootstrap CI is far tighter than the absolute BPB uncertainty.
//!
//! # What an ablation delta does and does not tell you
//!
//! A paired ablation has high statistical power, so a *practically meaningless* effect can
//! still show a CI that excludes zero. Passing the CI test is therefore **not** a verdict
//! on a mechanism. Two axes are reported separately and must be judged separately:
//!
//! 1. **Magnitude** — `|delta|` against an equivalence margin `epsilon` (default 0.01 BPB,
//!    about one third of the 0.0316 BPB gap to the matched Kneser-Ney 5-gram, so anything
//!    below it cannot be decisive for the competitive question). Bands:
//!    MAJOR+ / MINOR+ / NEGLIGIBLE / MINOR- / MAJOR-.
//! 2. **Mechanism class** — declared per mechanism, never inferred from the number. A small
//!    delta is consistent with at least four different situations, and they demand opposite
//!    actions:
//!    * `PrimaryCarrier` — expected to carry signal; a NEGLIGIBLE delta is a defect.
//!    * `Modulator` — expected small, consistent effect; NEGLIGIBLE is acceptable.
//!    * `Selector` — routing/candidate selection. **BPB ablation is the wrong instrument**:
//!      an alternative path substitutes for the ablated selector, so a near-zero delta says
//!      nothing about whether the selector works. Use decision-flip rate and forced-choice
//!      tests.
//!    * `Enabler` — an observation channel whose value appears only once composed with a
//!      component that does not exist yet. A near-zero delta is *expected and uninformative*.
//!    * `CountTable` — a count statistic, not a learned geometric mechanism.
//!
//! A mechanism is therefore **never retired** on a near-zero delta if its class is
//! `Enabler` or `Selector`; the required action is to repair the wiring or change the
//! instrument and re-measure. Only a `CountTable` or `PrimaryCarrier` that is correctly
//! wired and measurably net-negative, or carrying a large resource cost for no measured
//! return, is a removal candidate.
//!
//! Decision-flip rate and top-1 accuracy change are reported alongside BPB because a
//! mechanism can be BPB-neutral while changing many decisions — which matters for the
//! *kind* of errors, not the average.
//!
//! # Contract note
//!
//! This is an offline evaluation harness and is deliberately outside the frozen
//! integer-kernel marker region. It uses floating point for the softmax and the
//! bootstrap; the served path it evaluates does not.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use uor_r4_core::native_geometric::learner::ExportedGeometricModel;
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;
use uor_r4_core::native_geometric::vsa::{encode_attended_multiscale_context, Codebook};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const DEFAULT_MODEL: &str = "native_geometric_prose_model.rgm";
const DEFAULT_CORPUS: &str = "tinystories_train.u16";
const DEFAULT_TOKENIZER: &str = ".uor-models/research/issue-1014/export/tokenizer.json";
const DEFAULT_SEQ_LEN: usize = 64;
const DEFAULT_POSITIONS: usize = 8_192;
/// Equivalence margin in BPB. |delta| below this is reported NEGLIGIBLE regardless of the
/// bootstrap CI, because a paired test can exclude zero on a meaningless effect. 0.01 BPB
/// is about one third of the 0.0316 BPB gap to the matched Kneser-Ney 5-gram.
const DEFAULT_EPSILON: f64 = 0.01;
/// Serving sampler divisor: `(s - max) / (temperature * 8192)`.
const SAMPLER_SCALE: f64 = 8192.0;
const BOOTSTRAP_RESAMPLES: usize = 1_000;

/// Every ablation this tool understands, in report order.
const ABLATIONS: [&str; 10] = [
    "none",
    "vsa",
    "engram",
    "lattice",
    "lattice_coarse",
    "lattice_fine",
    "jepa",
    "lanes",
    "s2_readout",
    "bias",
];

struct Args {
    model: PathBuf,
    corpus: PathBuf,
    tokenizer: PathBuf,
    seq_len: usize,
    positions: usize,
    holdout_offset: usize,
    epsilon: f64,
    vsa_code_mode: String,
    ablations: Vec<String>,
}

fn usage() -> String {
    format!(
        "ablate-prose -- per-mechanism ablation BPB sweep (Card P7)\n\
         \n\
         USAGE:\n  ablate-prose [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \x20 --model <path>      artifact (.rgm or .json)   [default: {DEFAULT_MODEL}]\n\
         \x20 --corpus <path>     .u16 token stream          [default: {DEFAULT_CORPUS}]\n\
         \x20 --tokenizer <path>  tokenizer.json             [default: {DEFAULT_TOKENIZER}]\n\
         \x20 --seq-len <n>       teacher-forced chunk size  [default: {DEFAULT_SEQ_LEN}]\n\
         \x20 --positions <n>     held-out positions to score [default: {DEFAULT_POSITIONS}]\n\
         \x20 --holdout-offset <n>  token offset of the held-out slice [default: 0]; use a\n\
         \x20                     disjoint offset to confirm a verdict on a second slice\n\
         \x20 --epsilon <f>      equivalence margin in BPB [default: {DEFAULT_EPSILON}]\n\
         \x20 --vsa-code-mode <m>  VSA token codes: `fixed` (token-id hash, current) or\n\
         \x20                     `root` (derived from the learned 120-root assignment)\n\
         \x20 --ablations <list>  comma-separated subset; default all\n\
         \x20 --help\n\
         \n\
         ABLATIONS: {}\n",
        ABLATIONS.join(", ")
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
        epsilon: DEFAULT_EPSILON,
        vsa_code_mode: "fixed".to_string(),
        ablations: ABLATIONS.iter().map(|s| s.to_string()).collect(),
    };
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let next = |i: usize| -> Result<&String, String> {
            argv.get(i + 1)
                .ok_or_else(|| format!("missing value for {}", argv[i]))
        };
        match argv[i].as_str() {
            "--help" | "-h" => return Err(usage()),
            "--model" => {
                args.model = PathBuf::from(next(i)?);
                i += 2;
            }
            "--corpus" => {
                args.corpus = PathBuf::from(next(i)?);
                i += 2;
            }
            "--tokenizer" => {
                args.tokenizer = PathBuf::from(next(i)?);
                i += 2;
            }
            "--seq-len" => {
                args.seq_len = next(i)?.parse().map_err(|_| "bad --seq-len".to_string())?;
                i += 2;
            }
            "--positions" => {
                args.positions = next(i)?
                    .parse()
                    .map_err(|_| "bad --positions".to_string())?;
                i += 2;
            }
            "--holdout-offset" => {
                args.holdout_offset = next(i)?
                    .parse()
                    .map_err(|_| "bad --holdout-offset".to_string())?;
                i += 2;
            }
            "--epsilon" => {
                args.epsilon = next(i)?.parse().map_err(|_| "bad --epsilon".to_string())?;
                i += 2;
            }
            "--vsa-code-mode" => {
                let mode = next(i)?.to_ascii_lowercase();
                if mode != "fixed" && mode != "root" {
                    return Err(format!(
                        "unknown --vsa-code-mode '{mode}'; known: fixed, root"
                    ));
                }
                args.vsa_code_mode = mode;
                i += 2;
            }
            "--ablations" => {
                args.ablations = next(i)?
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                i += 2;
            }
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }
    if args.seq_len < 2 {
        return Err("--seq-len must be >= 2 (teacher forcing needs a predecessor)".into());
    }
    if args.positions == 0 {
        return Err("--positions must be >= 1".into());
    }
    for name in &args.ablations {
        if !ABLATIONS.contains(&name.as_str()) {
            return Err(format!(
                "unknown ablation '{name}'; known: {}",
                ABLATIONS.join(", ")
            ));
        }
    }
    Ok(args)
}

fn load_model(path: &PathBuf) -> Result<ExportedGeometricModel, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if bytes.len() >= 4 && &bytes[..4] == b"RGM1" {
        ExportedGeometricModel::from_binary(&bytes)
            .map_err(|e| format!("binary artifact {}: {e:?}", path.display()))
    } else {
        serde_json::from_slice::<ExportedGeometricModel>(&bytes)
            .map_err(|e| format!("json artifact {}: {e}", path.display()))
    }
}

/// Disable exactly one mechanism in place. Unknown names are rejected by `parse_args`.
fn apply_ablation(model: &mut ExportedGeometricModel, name: &str) -> Result<(), String> {
    match name {
        "none" => {}
        "vsa" => model.vsa_scale_q15 = 0,
        "engram" => model.engram_table = None,
        "lattice" => model.hierarchical_lattice = None,
        "lattice_coarse" => {
            let lattice = model
                .hierarchical_lattice
                .as_mut()
                .ok_or("artifact has no hierarchical lattice to ablate")?;
            lattice.coarse_trigram.fill(0);
        }
        "lattice_fine" => {
            let lattice = model
                .hierarchical_lattice
                .as_mut()
                .ok_or("artifact has no hierarchical lattice to ablate")?;
            lattice.fine_residual.fill(0);
        }
        "jepa" => {
            model.discrete_jepa_w_state = [0; 9];
            model.discrete_jepa_w_token = [0; 9];
            model.discrete_jepa_bias = [0; 3];
            model.discrete_jepa_fiber_w_state = [0; 4];
            model.discrete_jepa_fiber_w_token = [0; 4];
            model.discrete_jepa_fiber_bias = [0; 2];
        }
        "lanes" => {
            for table in &mut model.discrete_tables {
                table.scores.fill(0);
            }
        }
        "s2_readout" => model.discrete_s2_readout.fill([0; 5]),
        "bias" => model.discrete_bias.fill(0),
        other => return Err(format!("unhandled ablation '{other}'")),
    }
    Ok(())
}

/// `(bits, bytes)` contributed by each scored position, plus the decision it produced.
struct Losses {
    bits: Vec<f64>,
    bytes: Vec<u64>,
    /// Argmax token chosen at each scored position under the serving scorer.
    argmax: Vec<u32>,
    /// Positions where the argmax equals the teacher-forced target.
    top1_hits: usize,
    scored: usize,
}

impl Losses {
    fn bpb(&self) -> f64 {
        let total_bytes: u64 = self.bytes.iter().sum();
        if total_bytes == 0 {
            return f64::NAN;
        }
        self.bits.iter().sum::<f64>() / total_bytes as f64
    }

    /// Top-1 accuracy in percentage points.
    fn top1_pct(&self) -> f64 {
        if self.scored == 0 {
            return f64::NAN;
        }
        100.0 * self.top1_hits as f64 / self.scored as f64
    }
}

/// Fraction of positions where `other` chooses a different argmax than `base`.
/// This is the instrument that can see a `Selector` or `Enabler` that BPB cannot.
fn decision_flip_rate(base: &Losses, other: &Losses) -> f64 {
    let n = base.argmax.len().min(other.argmax.len());
    if n == 0 {
        return f64::NAN;
    }
    let flips = (0..n)
        .filter(|&i| base.argmax[i] != other.argmax[i])
        .count();
    100.0 * flips as f64 / n as f64
}

/// Magnitude band against the equivalence margin. `.0` is true for a positive delta.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Band {
    MajorPositive,
    MinorPositive,
    Negligible,
    MinorNegative,
    MajorNegative,
}

impl Band {
    fn of(delta: f64, eps: f64) -> Band {
        if delta >= 0.10 {
            Band::MajorPositive
        } else if delta >= eps {
            Band::MinorPositive
        } else if delta > -eps {
            Band::Negligible
        } else if delta > -0.10 {
            Band::MinorNegative
        } else {
            Band::MajorNegative
        }
    }

    fn label(self) -> &'static str {
        match self {
            Band::MajorPositive => "MAJOR+",
            Band::MinorPositive => "MINOR+",
            Band::Negligible => "NEGLIGIBLE",
            Band::MinorNegative => "MINOR-",
            Band::MajorNegative => "MAJOR-",
        }
    }
}

/// Declared mechanism class. Never inferred from the measured delta.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MechClass {
    PrimaryCarrier,
    Modulator,
    Selector,
    Enabler,
    CountTable,
}

impl MechClass {
    fn label(self) -> &'static str {
        match self {
            MechClass::PrimaryCarrier => "primary-carrier",
            MechClass::Modulator => "modulator",
            MechClass::Selector => "selector",
            MechClass::Enabler => "enabler",
            MechClass::CountTable => "count-table",
        }
    }
}

fn mechanism_class(name: &str) -> MechClass {
    match name {
        // Learned geometric state prediction and its readout: expected to carry signal.
        "jepa" | "s2_readout" => MechClass::PrimaryCarrier,
        // Learned 120x120 compatibility tables: expected to carry signal.
        "lanes" => MechClass::PrimaryCarrier,
        // Emission bias: small, broad shaping.
        "bias" => MechClass::Modulator,
        // Observation channel over a codebook that is not yet consistent with the learned
        // representation. Its value can only appear once the codebook is learned.
        "vsa" => MechClass::Enabler,
        // Hop-free: the hand-coded induction term is candidate selection.
        "induction" => MechClass::Selector,
        // Count statistics, not learned geometric mechanisms.
        "engram" | "lattice" | "lattice_coarse" | "lattice_fine" => MechClass::CountTable,
        _ => MechClass::PrimaryCarrier,
    }
}

/// Known wiring status, stated so a small delta is not mistaken for a mechanism verdict.
fn wiring_status(name: &str) -> &'static str {
    match name {
        "vsa" => {
            "MIS-WIRED: heads bind a fixed random codebook (vsa/codebook.rs) disconnected \
                   from the learned 120-root assignment (jepa_trainer.rs:1119), so only token \
                   identity is recoverable. This delta measures THAT WIRING, not the mechanism."
        }
        "lanes" => "presumed wired; net-negative here, so diagnose rather than assume",
        "jepa" => "wired; note it also changes the fiber the S2 readout consumes",
        "lattice_coarse" => "wired; count table",
        _ => "presumed wired",
    }
}

/// Action implied by (band, class). Deliberately refuses to retire an `Enabler` or
/// `Selector` on a near-zero or negative delta.
fn action(band: Band, class: MechClass) -> &'static str {
    match (band, class) {
        (Band::MajorPositive | Band::MinorPositive, _) => "RETAIN",
        (Band::Negligible, MechClass::Enabler | MechClass::Selector) => {
            "DO NOT CONCLUDE: instrument cannot see this role; fix wiring / use a decision test"
        }
        (Band::Negligible, _) => {
            "no measurable contribution; removal candidate on resource cost only"
        }
        (Band::MajorNegative | Band::MinorNegative, MechClass::Enabler | MechClass::Selector) => {
            "DO NOT RETIRE ON THIS EVIDENCE: verify wiring first"
        }
        (Band::MajorNegative | Band::MinorNegative, _) => "net-negative; removal candidate",
    }
}

/// Teacher-forced evaluation of the discrete scorer over full-vocabulary
/// normalisation. Context never crosses a chunk boundary, matching the trainer's
/// `chunks_exact(seq_len)` evaluation.
fn evaluate(
    model: &ExportedGeometricModel,
    codebook: &Codebook<64>,
    tokens: &[u16],
    token_lens: &[u32],
    seq_len: usize,
    max_positions: usize,
    vocab: usize,
) -> Result<Losses, String> {
    let mut bits = Vec::with_capacity(max_positions);
    let mut bytes = Vec::with_capacity(max_positions);
    let mut argmax: Vec<u32> = Vec::with_capacity(max_positions);
    let mut top1_hits = 0usize;
    let mut ctx: Vec<usize> = Vec::with_capacity(seq_len);
    let mut scores: Vec<i32> = vec![0; vocab];

    for chunk in tokens.chunks_exact(seq_len) {
        if bits.len() >= max_positions {
            break;
        }
        ctx.clear();
        for j in 1..seq_len {
            if bits.len() >= max_positions {
                break;
            }
            ctx.push(chunk[j - 1] as usize);
            let target = chunk[j] as usize;
            if target >= vocab {
                continue;
            }

            let fiber = model.context_predicted_fiber_q30(&ctx);
            let mut buf = [0u32; 64];
            let n = ctx.len().min(64);
            for i in 0..n {
                buf[i] = ctx[ctx.len() - n + i] as u32;
            }
            let ctx_vsa = encode_attended_multiscale_context(&buf[..n], codebook, 64);

            // Full-vocabulary scores under the serving scorer, one pass.
            let mut max_score = i32::MIN;
            let mut best_cand = 0usize;
            for cand in 0..vocab {
                let s = model.score_context_candidate_with_vsa(
                    &ctx,
                    cand,
                    fiber,
                    Some((codebook, &ctx_vsa)),
                );
                scores[cand] = s;
                if s > max_score {
                    max_score = s;
                    best_cand = cand;
                }
            }

            // Serving sampler form: exp((s - max) / (temperature * 8192)), clamped.
            let mut sum = 0.0f64;
            let mut target_p = 0.0f64;
            for (cand, &s) in scores.iter().enumerate() {
                let w = libm::exp((((s - max_score) as f64) / SAMPLER_SCALE).clamp(-60.0, 0.0));
                sum += w;
                if cand == target {
                    target_p = w;
                }
            }
            if !(sum > 0.0) || !(target_p > 0.0) {
                return Err(format!(
                    "degenerate distribution at position {} (sum={sum}, target_p={target_p})",
                    bits.len()
                ));
            }
            bits.push(-(target_p / sum).log2());
            bytes.push(token_lens.get(target).copied().unwrap_or(1).max(1) as u64);
            if best_cand == target {
                top1_hits += 1;
            }
            argmax.push(best_cand as u32);
        }
    }
    let scored = bits.len();
    Ok(Losses {
        bits,
        bytes,
        argmax,
        top1_hits,
        scored,
    })
}

/// Paired bootstrap 95% CI for `bpb(ablated) - bpb(baseline)` over positions.
fn bootstrap_delta_ci(base: &Losses, abl: &Losses) -> (f64, f64, f64) {
    let n = base.bits.len().min(abl.bits.len());
    let delta = abl.bpb() - base.bpb();
    if n == 0 {
        return (delta, f64::NAN, f64::NAN);
    }
    // Deterministic xorshift bootstrap: reproducible without an RNG dependency.
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut samples = Vec::with_capacity(BOOTSTRAP_RESAMPLES);
    for _ in 0..BOOTSTRAP_RESAMPLES {
        let mut b_bits = 0.0f64;
        let mut b_bytes = 0u64;
        let mut a_bits = 0.0f64;
        let mut a_bytes = 0u64;
        for _ in 0..n {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let idx = (state as usize) % n;
            b_bits += base.bits[idx];
            b_bytes += base.bytes[idx];
            a_bits += abl.bits[idx];
            a_bytes += abl.bytes[idx];
        }
        if b_bytes == 0 || a_bytes == 0 {
            continue;
        }
        samples.push(a_bits / a_bytes as f64 - b_bits / b_bytes as f64);
    }
    if samples.is_empty() {
        return (delta, f64::NAN, f64::NAN);
    }
    samples.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    let lo = samples[samples.len() / 40];
    let hi = samples[samples.len() - 1 - samples.len() / 40];
    (delta, lo, hi)
}

fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let model = load_model(&args.model)?;

    let tokenizer_bytes = std::fs::read(&args.tokenizer)?;
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| format!("failed to parse tokenizer {}", args.tokenizer.display()))?;
    let token_lens = tokenizer.token_byte_lengths();

    let reader = MmapCorpusReader::open(&args.corpus)?;
    let corpus = reader.as_slice();
    let vocab = model.vocab_size;
    let codebook = match args.vsa_code_mode.as_str() {
        "root" => uor_r4_core::native_geometric::learner::build_root_codebook(
            vocab,
            &model.token_to_root,
            model.vsa_seed,
        ),
        _ => Codebook::<64>::on_demand(vocab, model.vsa_seed),
    };

    // Held-out convention matches `train-native-prose`: the first tokens of the
    // mapped stream are evaluation, the remainder is training. Positions are drawn
    // from the head of that region so the split is identical for every ablation.
    let needed = args
        .positions
        .next_multiple_of(args.seq_len)
        .saturating_add(args.seq_len);
    let slice_start = args.holdout_offset.min(corpus.len());
    let slice_end = slice_start.saturating_add(needed).min(corpus.len());
    if slice_end <= slice_start {
        return Err(format!(
            "empty held-out slice: offset {} with {} corpus tokens",
            slice_start,
            corpus.len()
        )
        .into());
    }
    let held_out = &corpus[slice_start..slice_end];

    println!("ablate-prose (Card P7) -- per-mechanism ablation BPB sweep");
    println!("  model      : {} (vocab {})", args.model.display(), vocab);
    println!(
        "  corpus     : {} ({} tokens; held-out slice [{}..{}) = {} tokens)",
        args.corpus.display(),
        corpus.len(),
        slice_start,
        slice_end,
        held_out.len()
    );
    println!("  scorer     : discrete additive scorer, full-vocabulary softmax at temperature 1.0");
    println!(
        "  seq_len    : {}   positions requested: {}",
        args.seq_len, args.positions
    );
    println!(
        "  epsilon    : {:.4} BPB (equivalence margin; |delta| below this is NEGLIGIBLE)",
        args.epsilon
    );
    println!(
        "  vsa codes  : {} ({} distinct code vectors)",
        args.vsa_code_mode,
        {
            let mut seen = std::collections::BTreeSet::new();
            for v in codebook.table.iter().take(vocab) {
                seen.insert(v.data.to_vec());
            }
            seen.len()
        }
    );
    println!();

    let baseline = evaluate(
        &model,
        &codebook,
        held_out,
        &token_lens,
        args.seq_len,
        args.positions,
        vocab,
    )?;
    println!(
        "baseline (no ablation): BPB = {:.4} over {} positions in {:.1}s",
        baseline.bpb(),
        baseline.scored,
        started.elapsed().as_secs_f64()
    );
    println!();
    println!(
        "{:<16} {:>9} {:>+9} {:>19} {:>11} {:>8} {:>7}",
        "ablation", "BPB", "delta", "paired 95% CI", "band", "dTop1pp", "flip%"
    );

    for name in &args.ablations {
        if name == "none" {
            continue;
        }
        let t = Instant::now();
        let mut ablated = model.clone();
        apply_ablation(&mut ablated, name)?;
        let losses = evaluate(
            &ablated,
            &codebook,
            held_out,
            &token_lens,
            args.seq_len,
            args.positions,
            vocab,
        )?;
        let (delta, lo, hi) = bootstrap_delta_ci(&baseline, &losses);
        let band = Band::of(delta, args.epsilon);
        let class = mechanism_class(name);
        let d_top1 = losses.top1_pct() - baseline.top1_pct();
        let flip = decision_flip_rate(&baseline, &losses);
        println!(
            "{:<16} {:>9.4} {:>+9.4} {:>19} {:>11} {:>+8.2} {:>6.1}%",
            name,
            losses.bpb(),
            delta,
            format!("[{lo:+.4}, {hi:+.4}]"),
            band.label(),
            d_top1,
            flip
        );
        // The CI is reported for completeness; the BAND and the declared class drive the
        // action, because a paired CI can exclude zero on a practically meaningless effect.
        let ci_excludes_zero = lo > 0.0 || hi < 0.0;
        println!(
            "{:<16}   class={}  ci_excludes_0={}  wiring: {}",
            "",
            class.label(),
            ci_excludes_zero,
            wiring_status(name)
        );
        println!("{:<16}   action: {}", "", action(band, class));
        eprintln!("  ({name}: {:.1}s)", t.elapsed().as_secs_f64());
    }

    println!();
    println!("baseline top-1 accuracy: {:.2}%", baseline.top1_pct());
    println!("elapsed_total_s: {:.1}", started.elapsed().as_secs_f64());
    println!();
    println!("Decision rule (an ablation is not a verdict):");
    println!(
        "  * A NEGLIGIBLE delta on an `enabler` or `selector` means the instrument cannot \
         see that role -- repair the wiring or use a decision test, do not retire."
    );
    println!(
        "  * Only a `count-table` or `primary-carrier` that is correctly wired and measurably \
         net-negative is a removal candidate, and then on resource cost as much as accuracy."
    );
    println!(
        "  * Statistical significance (CI excluding 0) is not practical significance; read the band."
    );
    Ok(())
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return if msg.starts_with("ablate-prose --") {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
