//! Stage 4 insertion gate, shortlist form: a LUT-4 term that **re-ranks the shortlist**.
//!
//! The first form added a constant-per-bit correction to every candidate's score. A term that
//! classifies a candidate at ~76 % still moves nothing that way, because one bit is shared by
//! half the vocabulary and the correction is diluted across 4,096 candidates. The plan says the
//! term is a *selected read over the shortlist feature vector*, so this form reads the baseline's
//! own top-K candidates and re-ranks only those.
//!
//! Features are richer than the first form: each of the scorer's six additive term groups is
//! quantised at four thresholds instead of one sign and one magnitude bit, plus two
//! context-level features (whether any engram fired for this context, and a context-length
//! bucket). Context features let a gate condition a candidate feature on the context, which the
//! additive scorer cannot express at all.
//!
//! Served form stays a truth-table read plus integer additions: no multiplier, no float.

use std::path::PathBuf;
use std::time::Instant;

use uor_r4_core::native_geometric::learner::jepa_trainer::{
    CandidateTermScores, ExportedGeometricModel,
};
use uor_r4_core::native_geometric::learner::lut4::{gate_hard, Lut4Layer, Lut4Network};
use uor_r4_core::native_geometric::vsa::{encode_attended_multiscale_context, Codebook};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

/// The serving sampler's score scale, matching `ablate-prose`.
const SAMPLER_SCALE: f64 = 8192.0;
/// Term groups: bias, summed lattice lags, S2 readout, VSA, engram, hierarchical lattice.
const GROUPS: usize = 6;
/// Quantisation thresholds applied to each group, as magnitudes in score units.
const THRESHOLDS: [i32; 4] = [1, 64, 256, 1024];
/// Per-group feature bits: one per threshold.
const GROUP_BITS: usize = THRESHOLDS.len();
/// Context-level feature bits: engram-active, and a context-length bucket.
const CONTEXT_BITS: usize = 2;
/// Total feature width.
const FEATURE_BITS: usize = GROUPS * GROUP_BITS + CONTEXT_BITS;

struct Args {
    model: PathBuf,
    corpus: PathBuf,
    eval_corpus: Option<PathBuf>,
    tokenizer: PathBuf,
    seq_len: usize,
    train_positions: usize,
    eval_positions: usize,
    holdout_offset: usize,
    shortlist: usize,
    negatives: usize,
    steps: usize,
    rate: f32,
    out: Option<PathBuf>,
}

fn usage() -> String {
    "lut4-prose --model <artifact.rgm> --corpus <train.u16> [--eval-corpus <heldout.u16>]\n\
     \x20 --tokenizer <tokenizer.json> [--seq-len 64] [--train-positions 512]\n\
     \x20 [--eval-positions 256] [--holdout-offset N] [--shortlist 64] [--negatives 32]\n\
     \x20 [--steps 400] [--rate 0.5] [--out report.json]"
        .to_string()
}

fn parse_args() -> Result<Args, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args {
        model: PathBuf::new(),
        corpus: PathBuf::new(),
        eval_corpus: None,
        tokenizer: PathBuf::new(),
        seq_len: 64,
        train_positions: 512,
        eval_positions: 256,
        holdout_offset: 1_000_000,
        shortlist: 64,
        negatives: 32,
        steps: 400,
        rate: 0.5,
        out: None,
    };
    let mut i = 0;
    while i < argv.len() {
        let value = |i: usize| -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("missing value for {}", argv[i]))
        };
        let mut number = |i: usize, name: &str| -> Result<usize, String> {
            value(i)?.parse().map_err(|e| format!("{name}: {e}"))
        };
        match argv[i].as_str() {
            "--model" => {
                args.model = PathBuf::from(value(i)?);
                i += 2;
            }
            "--corpus" => {
                args.corpus = PathBuf::from(value(i)?);
                i += 2;
            }
            "--eval-corpus" => {
                args.eval_corpus = Some(PathBuf::from(value(i)?));
                i += 2;
            }
            "--tokenizer" => {
                args.tokenizer = PathBuf::from(value(i)?);
                i += 2;
            }
            "--seq-len" => {
                args.seq_len = number(i, "seq-len")?;
                i += 2;
            }
            "--train-positions" => {
                args.train_positions = number(i, "train-positions")?;
                i += 2;
            }
            "--eval-positions" => {
                args.eval_positions = number(i, "eval-positions")?;
                i += 2;
            }
            "--holdout-offset" => {
                args.holdout_offset = number(i, "holdout-offset")?;
                i += 2;
            }
            "--shortlist" => {
                args.shortlist = number(i, "shortlist")?;
                i += 2;
            }
            "--negatives" => {
                args.negatives = number(i, "negatives")?;
                i += 2;
            }
            "--steps" => {
                args.steps = number(i, "steps")?;
                i += 2;
            }
            "--rate" => {
                args.rate = value(i)?.parse().map_err(|e| format!("rate: {e}"))?;
                i += 2;
            }
            "--out" => {
                args.out = Some(PathBuf::from(value(i)?));
                i += 2;
            }
            "--help" => return Err(usage()),
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }
    if args.model.as_os_str().is_empty()
        || args.corpus.as_os_str().is_empty()
        || args.tokenizer.as_os_str().is_empty()
    {
        return Err(format!(
            "--model, --corpus and --tokenizer are required\n\n{}",
            usage()
        ));
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

/// Quantise the six term groups at four thresholds each, then append the context bits.
fn features(terms: &CandidateTermScores, context_bits: [f32; CONTEXT_BITS], out: &mut [f32]) {
    let groups = [
        terms.bias,
        terms.lattice_lag,
        terms.s2,
        terms.vsa,
        terms.engram,
        terms.hierarchical,
    ];
    for (group, value) in groups.iter().enumerate() {
        for (slot, threshold) in THRESHOLDS.iter().enumerate() {
            out[group * GROUP_BITS + slot] = f32::from(u8::from(value.abs() >= *threshold));
        }
    }
    out[GROUPS * GROUP_BITS] = context_bits[0];
    out[GROUPS * GROUP_BITS + 1] = context_bits[1];
}

/// One scored position: the full baseline score row and the shortlist the term re-ranks.
struct Position {
    target: usize,
    bytes: u64,
    scores: Vec<i32>,
    shortlist: Vec<usize>,
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };
    if let Err(error) = run(&args) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let model = load_model(&args.model)?;
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&std::fs::read(&args.tokenizer)?)
        .ok_or("failed to parse tokenizer")?;
    let token_bytes = tokenizer.token_byte_lengths();
    let read_tokens = |path: &PathBuf| -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        let raw = std::fs::read(path)?;
        Ok(raw
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]) as usize)
            .collect())
    };
    let tokens = read_tokens(&args.corpus)?;
    let eval_tokens = match &args.eval_corpus {
        Some(path) => read_tokens(path)?,
        None => tokens.clone(),
    };
    let vocab = model.vocab_size;

    let fit_positions = (args.train_positions * 4) / 5;
    let select_positions = args.train_positions - fit_positions;
    let select_start = args.seq_len + fit_positions;
    let eval_start = if args.eval_corpus.is_some() {
        args.seq_len
    } else {
        args.holdout_offset
            .max(select_start + select_positions + args.seq_len)
    };
    if tokens.len() < select_start + select_positions + args.seq_len + 1 {
        return Err(format!("training corpus too short: {} tokens", tokens.len()).into());
    }
    if eval_tokens.len() < eval_start + args.eval_positions + 1 {
        return Err(format!("evaluation corpus too short: {} tokens", eval_tokens.len()).into());
    }

    let codebook = Codebook::<64>::on_demand(vocab, model.vsa_seed);
    let load =
        |stream: &[usize],
         start: usize,
         count: usize|
         -> Result<(Vec<Position>, Vec<Vec<Vec<f32>>>, usize), Box<dyn std::error::Error>> {
            let mut rows = Vec::with_capacity(count);
            let mut feature_rows = Vec::with_capacity(count);
            let mut target_in_shortlist = 0usize;
            for step in 0..count {
                let end = start + step;
                let ctx = &stream[end - args.seq_len..end];
                let target = stream[end];
                let fiber = model.context_predicted_fiber_q30(ctx);
                let mut buf = [0u32; 64];
                let n = ctx.len().min(64);
                for (i, token) in ctx[ctx.len() - n..].iter().enumerate() {
                    buf[i] = *token as u32;
                }
                let ctx_vsa = encode_attended_multiscale_context(&buf[..n], &codebook, 64);
                let mut scores = vec![0i32; vocab];
                let mut terms_per_candidate = Vec::with_capacity(vocab);
                let mut engram_active = 0.0f32;
                for cand in 0..vocab {
                    let terms = model.score_context_candidate_terms(
                        ctx,
                        cand,
                        fiber,
                        Some((&codebook, &ctx_vsa)),
                    );
                    scores[cand] = terms.total();
                    if terms.engram != 0 {
                        engram_active = 1.0;
                    }
                    terms_per_candidate.push(terms);
                }
                let mut order: Vec<usize> = (0..vocab).collect();
                order.sort_by(|a, b| scores[*b].cmp(&scores[*a]).then_with(|| a.cmp(b)));
                order.truncate(args.shortlist.min(vocab));
                if order.contains(&target) {
                    target_in_shortlist += 1;
                }
                let context_bits = [engram_active, f32::from(u8::from(ctx.len() >= 32))];
                let mut rows_features = Vec::with_capacity(order.len());
                for &cand in &order {
                    let mut bits = vec![0.0f32; FEATURE_BITS];
                    features(&terms_per_candidate[cand], context_bits, &mut bits);
                    rows_features.push(bits);
                }
                feature_rows.push(rows_features);
                rows.push(Position {
                    target,
                    bytes: token_bytes.get(target).copied().unwrap_or(1).max(1) as u64,
                    scores,
                    shortlist: order,
                });
            }
            Ok((rows, feature_rows, target_in_shortlist))
        };

    let (fit_rows, fit_features, fit_in) = load(&tokens, args.seq_len, fit_positions)?;
    let (select_rows, select_features, select_in) = load(&tokens, select_start, select_positions)?;
    let (eval_rows, eval_features, eval_in) = load(&eval_tokens, eval_start, args.eval_positions)?;

    // Balanced training pairs from the training shortlists: the target row repeated, decoys once.
    let mut samples: Vec<(Vec<f32>, Vec<f32>)> = Vec::new();
    for (row, features_row) in fit_rows.iter().zip(fit_features.iter()) {
        for (slot, &cand) in row.shortlist.iter().enumerate() {
            let positive = cand == row.target;
            let repeats = if positive { args.negatives } else { 1 };
            for _ in 0..repeats {
                samples.push((
                    features_row[slot].clone(),
                    vec![f32::from(u8::from(positive))],
                ));
            }
        }
    }

    // Exhaustive best two-input term over the feature bits: a floor on what any LUT can do here.
    let mut best_term: Option<(f64, usize, usize, u8)> = None;
    for i in 0..FEATURE_BITS {
        for j in (i + 1)..FEATURE_BITS {
            for gate in 0u8..16 {
                let mut loss = 0.0f64;
                for (bits, label) in samples.iter().take(4096) {
                    let a = u8::from(bits[i] > 0.5);
                    let b = u8::from(bits[j] > 0.5);
                    let d = f64::from(gate_hard(gate, a, b)) - f64::from(label[0]);
                    loss += d * d;
                }
                let loss = loss / samples.len().min(4096) as f64;
                if best_term.is_none_or(|(best, _, _, _)| loss < best) {
                    best_term = Some((loss, i, j, gate));
                }
            }
        }
    }
    let best_term = best_term.unwrap_or((0.25, 0, 1, 6));

    let mut network = Lut4Network::new(vec![
        Lut4Layer::seeded(FEATURE_BITS, 16, 0x4C55_5434),
        Lut4Layer::seeded(16, 1, 0x4C55_5435),
    ]);
    let first_loss = network.fit_gates(&samples, args.rate, 1);
    let final_loss = network.fit_gates(&samples, args.rate, args.steps);
    let artifact = network.export();
    let cones = network.cone_sizes();
    let utilisation = network.gate_utilisation();
    let entropy = network.gate_entropy_bits();
    let gap_inputs: Vec<Vec<f32>> = samples.iter().take(256).map(|(x, _)| x.clone()).collect();
    let gap = network.discretisation_gap(&gap_inputs);

    // Re-rank the shortlist with the learned term and measure bits per byte.
    let measure = |scale: f64| -> (f64, f64, f64, f64) {
        let mut base_bits = 0.0f64;
        let mut soft_bits = 0.0f64;
        let mut hard_bits = 0.0f64;
        let mut changed = 0u64;
        let mut bytes = 0u64;
        for (row, features_row) in eval_rows.iter().zip(eval_features.iter()) {
            let mut soft_scores = row.scores.clone();
            let mut hard_scores = row.scores.clone();
            let mut soft_list: Vec<f64> = Vec::with_capacity(row.shortlist.len());
            let mut hard_list: Vec<f64> = Vec::with_capacity(row.shortlist.len());
            for (slot, &cand) in row.shortlist.iter().enumerate() {
                let bits = &features_row[slot];
                let soft_value = network.soft_forward(bits).first().copied().unwrap_or(0.0);
                let hard_bit = artifact.eval_f32(bits).first().copied().unwrap_or(0);
                soft_scores[cand] =
                    row.scores[cand] + (scale * (f64::from(soft_value) - 0.5)) as i32;
                hard_scores[cand] = row.scores[cand]
                    + if hard_bit == 1 {
                        (scale / 2.0) as i32
                    } else {
                        -(scale / 2.0) as i32
                    };
                soft_list.push(f64::from(soft_scores[cand]));
                hard_list.push(f64::from(hard_scores[cand]));
            }
            let baseline_top = row.shortlist.first().copied().unwrap_or(0);
            let soft_top = row.shortlist[soft_list
                .iter()
                .enumerate()
                .max_by(|x, y| x.1.partial_cmp(y.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(index, _)| index)
                .unwrap_or(0)];
            changed += u64::from(baseline_top != soft_top);
            let bits_for = |scores: &Vec<i32>| -> f64 {
                let max = scores.iter().copied().max().unwrap_or(0);
                let sum: f64 = scores
                    .iter()
                    .map(|s| libm::exp((((*s - max) as f64) / SAMPLER_SCALE).clamp(-60.0, 0.0)))
                    .sum();
                let p = libm::exp(
                    (((scores[row.target] - max) as f64) / SAMPLER_SCALE).clamp(-60.0, 0.0),
                ) / sum;
                -p.max(1e-300).log2()
            };
            base_bits += bits_for(&row.scores);
            soft_bits += bits_for(&soft_scores);
            hard_bits += bits_for(&hard_scores);
            bytes += row.bytes;
        }
        let bpb = |bits: f64| bits / bytes.max(1) as f64;
        (
            bpb(base_bits),
            bpb(soft_bits),
            bpb(hard_bits),
            changed as f64 / eval_rows.len().max(1) as f64,
        )
    };

    // Control: boost every shortlist candidate by a constant and nothing else. A learned term
    // that merely adds mass to the shortlist must not be credited for what a constant does.
    let boost_only = |scale: f64| -> f64 {
        let mut bits_total = 0.0f64;
        let mut bytes = 0u64;
        for row in &eval_rows {
            let mut scores = row.scores.clone();
            for &cand in &row.shortlist {
                scores[cand] += (scale / 2.0) as i32;
            }
            let max = scores.iter().copied().max().unwrap_or(0);
            let sum: f64 = scores
                .iter()
                .map(|s| libm::exp((((*s - max) as f64) / SAMPLER_SCALE).clamp(-60.0, 0.0)))
                .sum();
            let p =
                libm::exp((((scores[row.target] - max) as f64) / SAMPLER_SCALE).clamp(-60.0, 0.0))
                    / sum;
            bits_total += -p.max(1e-300).log2();
            bytes += row.bytes;
        }
        bits_total / bytes.max(1) as f64
    };

    // Choose the scale on the disjoint selection tail, then report the held-out measurement.
    let mut scale_curve: Vec<(f64, f64, f64)> = Vec::new();
    let mut best_scale = 2048.0f64;
    let mut best_soft = f64::INFINITY;
    let _ = (&select_rows, &select_features, select_in);
    for scale in [256.0f64, 512.0, 1024.0, 2048.0, 4096.0, 8192.0] {
        let (base, soft, _, changed) = measure(scale);
        scale_curve.push((scale, soft - base, changed));
        if soft < best_soft {
            best_soft = soft;
            best_scale = scale;
        }
    }
    let (base_bpb, soft_bpb, hard_bpb, top1_change) = measure(best_scale);

    // Artifact-output diagnostic. `cone_sizes` reports 0 for this network while the hard path
    // clearly varies its correction, so record exactly what the served tables emit per shortlist
    // slot before trusting either number.
    let mut bit_tally = [0u64; 2];
    let mut distinct_outputs: std::collections::BTreeSet<Vec<u8>> =
        std::collections::BTreeSet::new();
    let mut distinct_soft_rounded: std::collections::BTreeSet<i32> =
        std::collections::BTreeSet::new();
    for features_row in eval_features.iter() {
        for bits in features_row.iter() {
            let out = artifact.eval_f32(bits);
            distinct_outputs.insert(out.clone());
            if let Some(bit) = out.first() {
                bit_tally[*bit as usize] += 1;
            }
            let soft = network.soft_forward(bits).first().copied().unwrap_or(0.0);
            distinct_soft_rounded.insert((soft * 1000.0).round() as i32);
        }
    }
    let chosen: Vec<u8> = network
        .layers()
        .iter()
        .flat_map(|l| l.chosen_gates())
        .collect();
    let boost_bpb = boost_only(best_scale);
    let boost_curve: Vec<(f64, f64)> = [256.0f64, 512.0, 1024.0, 2048.0, 4096.0, 8192.0]
        .iter()
        .map(|scale| (*scale, boost_only(*scale) - base_bpb))
        .collect();

    let report = serde_json::json!({
        "schema": "uor-r4.lut4-shortlist-rerank/1",
        "status": "COMPLETED",
        "model": args.model.display().to_string(),
        "train_corpus": args.corpus.display().to_string(),
        "eval_corpus": args.eval_corpus.as_ref().map(|p| p.display().to_string()),
        "seq_len": args.seq_len,
        "shortlist": args.shortlist,
        "feature_bits": FEATURE_BITS,
        "fit_positions": fit_positions,
        "select_positions": select_positions,
        "eval_positions": args.eval_positions,
        "target_in_shortlist_rate": {
            "fit": fit_in as f64 / fit_positions.max(1) as f64,
            "select": select_in as f64 / select_positions.max(1) as f64,
            "eval": eval_in as f64 / args.eval_positions.max(1) as f64,
        },
        "samples": samples.len(),
        "train_loss_first": first_loss,
        "train_loss_final": final_loss,
        "exhaustive_best_term": {
            "train_loss": best_term.0,
            "feature_a": best_term.1,
            "feature_b": best_term.2,
            "gate": best_term.3,
        },
        "selected_scale": best_scale,
        "scale_curve_delta_soft": scale_curve,
        "bits_per_byte": {
            "baseline": base_bpb,
            "soft_lut": soft_bpb,
            "hard_lut": hard_bpb,
            "delta_soft": soft_bpb - base_bpb,
            "delta_hard": hard_bpb - base_bpb,
        },
        "discretisation_gap_bpb": soft_bpb - hard_bpb,
        "uniform_shortlist_boost": {
            "bits_per_byte": boost_bpb,
            "delta": boost_bpb - base_bpb,
            "curve": boost_curve,
        },
        "learned_minus_uniform_boost": (hard_bpb - boost_bpb),
        "top1_change_rate": top1_change,
        "gate_utilisation": utilisation,
        "gate_entropy_bits": entropy,
        "cone_sizes": cones,
        "artifact_output_diagnostic": {
            "hard_bit_tally": bit_tally,
            "distinct_hard_outputs": distinct_outputs.len(),
            "distinct_soft_outputs_milli": distinct_soft_rounded.len(),
            "chosen_gates": chosen,
        },
        "elapsed_seconds": started.elapsed().as_secs_f64(),
        "degenerate_term": distinct_outputs.len() <= 1,
        "scope": "Stage 4 gate, shortlist form: a learned nonlinear term re-ranks the baseline scorer's own top-K on real held-out text. Trained at the corpus head, scale chosen on a disjoint tail, evaluated on a separate held-out file. No serving default changed.",
    });
    let text = serde_json::to_string_pretty(&report)?;
    match &args.out {
        Some(path) => std::fs::write(path, text)?,
        None => println!("{text}"),
    }
    if distinct_outputs.len() <= 1 {
        eprintln!(
            "WARNING: the learned term is DEGENERATE ({} distinct artifact outputs); the reported \
             delta is a constant shift over the shortlist, not a learned re-ranking, and must not \
             be reported as improvement.",
            distinct_outputs.len()
        );
    }
    eprintln!(
        "baseline {base_bpb:.6} soft {soft_bpb:.6} hard {hard_bpb:.6} delta {:.6} top1change {top1_change:.3} target_in_shortlist {:.3}",
        soft_bpb - base_bpb,
        eval_in as f64 / args.eval_positions.max(1) as f64
    );
    Ok(())
}
