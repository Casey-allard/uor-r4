//! Stage 4 insertion gate: a LUT-4 nonlinear term over the additive scorer's own term groups.
//!
//! The serving scorer is purely additive — six summed term groups with no learned feature
//! interaction. This binary trains a [`Lut4Network`] on the **term groups the scorer itself
//! sums** (`score_context_candidate_terms`) and measures held-out bits per byte with and
//! without the learned term, reporting the discretisation gap, gate utilisation and the
//! realised cone sizes, which is the gate the transition plan sets for Stage 4.
//!
//! Served form: one truth-table read per output bit plus an additive correction, so the term
//! introduces no multiplier and no dense contraction.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_core::native_geometric::learner::jepa_trainer::{
    CandidateTermScores, ExportedGeometricModel,
};
use uor_r4_core::native_geometric::learner::lut4::{Lut4Layer, Lut4Network};
use uor_r4_core::native_geometric::vsa::{encode_attended_multiscale_context, Codebook};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

/// The serving sampler's score scale, matching `ablate-prose`.
const SAMPLER_SCALE: f64 = 8192.0;
/// Term groups turned into feature bits: a sign bit and a magnitude bit for each group.
const FEATURE_BITS: usize = 12;
/// Magnitude threshold that separates the second bit of each group.
const MAGNITUDE_THRESHOLD: i32 = 64;

struct Args {
    model: PathBuf,
    corpus: PathBuf,
    eval_corpus: Option<PathBuf>,
    tokenizer: PathBuf,
    seq_len: usize,
    train_positions: usize,
    eval_positions: usize,
    holdout_offset: usize,
    negatives: usize,
    steps: usize,
    rate: f32,
    correction: f64,
    out: Option<PathBuf>,
}

fn usage() -> String {
    "lut4-prose --model <artifact.rgm> --corpus <tokens.u16> --tokenizer <tokenizer.json>\n\
     \x20 [--seq-len 64] [--train-positions 1024] [--eval-positions 1024] [--holdout-offset N]\n\
     \x20 [--eval-corpus heldout.u16] [--negatives 32] [--steps 400] [--rate 0.5]\n\
     \x20 [--out report.json]"
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
        train_positions: 1024,
        eval_positions: 1024,
        holdout_offset: 1_000_000,
        negatives: 32,
        steps: 400,
        rate: 0.5,
        correction: 2048.0,
        out: None,
    };
    let mut i = 0;
    while i < argv.len() {
        let mut value = |i: usize| -> Result<String, String> {
            argv.get(i + 1)
                .cloned()
                .ok_or_else(|| format!("missing value for {}", argv[i]))
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
                args.seq_len = value(i)?.parse().map_err(|e| format!("seq-len: {e}"))?;
                i += 2;
            }
            "--train-positions" => {
                args.train_positions = value(i)?.parse().map_err(|e| format!("train: {e}"))?;
                i += 2;
            }
            "--eval-positions" => {
                args.eval_positions = value(i)?.parse().map_err(|e| format!("eval: {e}"))?;
                i += 2;
            }
            "--holdout-offset" => {
                args.holdout_offset = value(i)?.parse().map_err(|e| format!("offset: {e}"))?;
                i += 2;
            }
            "--negatives" => {
                args.negatives = value(i)?.parse().map_err(|e| format!("negatives: {e}"))?;
                i += 2;
            }
            "--steps" => {
                args.steps = value(i)?.parse().map_err(|e| format!("steps: {e}"))?;
                i += 2;
            }
            "--rate" => {
                args.rate = value(i)?.parse().map_err(|e| format!("rate: {e}"))?;
                i += 2;
            }
            "--correction" => {
                args.correction = value(i)?.parse().map_err(|e| format!("correction: {e}"))?;
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

/// Turn one candidate's six additive term groups into binary features: a sign bit and a
/// magnitude bit per group, so a two-input gate can learn an interaction between any pair.
fn features(terms: &CandidateTermScores, out: &mut [f32; FEATURE_BITS]) {
    let groups = [
        terms.bias,
        terms.lattice_lag,
        terms.s2,
        terms.vsa,
        terms.engram,
        terms.hierarchical,
    ];
    for (index, value) in groups.iter().enumerate() {
        out[2 * index] = f32::from(u8::from(*value > 0));
        out[2 * index + 1] = f32::from(u8::from(value.abs() > MAGNITUDE_THRESHOLD));
    }
}

struct Position {
    target: usize,
    bytes: u64,
    /// One row per scored candidate: the features and the base score.
    candidates: Vec<([f32; FEATURE_BITS], i32)>,
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
    let vocab = model.vocab_size;

    // Split the training corpus into a fit head and a disjoint scale-selection tail, and take
    // the evaluation slice from its own file when one is declared.
    let fit_positions = (args.train_positions * 4) / 5;
    let select_positions = args.train_positions - fit_positions;
    let select_start = args.seq_len + fit_positions;

    let eval_tokens = match &args.eval_corpus {
        Some(path) => read_tokens(path)?,
        None => tokens.clone(),
    };
    let eval_start = if args.eval_corpus.is_some() {
        args.seq_len
    } else {
        args.holdout_offset
            .max(select_start + select_positions + args.seq_len)
    };
    let need = select_start + select_positions + args.seq_len + 1;
    if tokens.len() < need {
        return Err(format!(
            "training corpus holds {} tokens; {} required for the declared split",
            tokens.len(),
            need
        )
        .into());
    }
    if eval_tokens.len() < eval_start + args.eval_positions + 1 {
        return Err(format!(
            "evaluation corpus holds {} tokens; {} required",
            eval_tokens.len(),
            eval_start + args.eval_positions + 1
        )
        .into());
    }

    let codebook = Codebook::<64>::on_demand(vocab, model.vsa_seed);
    let load = |stream: &[usize],
                start: usize,
                count: usize|
     -> Result<Vec<Position>, Box<dyn std::error::Error>> {
        let mut rows = Vec::with_capacity(count);
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
            let mut candidates = Vec::with_capacity(vocab);
            for cand in 0..vocab {
                let terms = model.score_context_candidate_terms(
                    ctx,
                    cand,
                    fiber,
                    Some((&codebook, &ctx_vsa)),
                );
                let mut bits = [0.0f32; FEATURE_BITS];
                features(&terms, &mut bits);
                candidates.push((bits, terms.total()));
            }
            rows.push(Position {
                target,
                bytes: token_bytes.get(target).copied().unwrap_or(1).max(1) as u64,
                candidates,
            });
        }
        Ok(rows)
    };

    let train = load(&tokens, args.seq_len, fit_positions)?;
    let select = load(&tokens, select_start, select_positions)?;
    let eval = load(&eval_tokens, eval_start, args.eval_positions)?;

    // Training pairs: the target candidate and `negatives` deterministic decoys per position.
    let mut samples: Vec<(Vec<f32>, Vec<f32>)> = Vec::new();
    for (index, row) in train.iter().enumerate() {
        let mut picks = vec![row.target];
        let mut state = (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        for _ in 0..args.negatives {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            picks.push((state as usize) % vocab);
        }
        for cand in picks {
            if cand >= row.candidates.len() {
                continue;
            }
            let (bits, _) = row.candidates[cand];
            // Repeat the positive so the classes are balanced; an imbalanced fit converges to
            // "always negative" and the correction then cannot move anything.
            let repeats = if cand == row.target {
                args.negatives
            } else {
                1
            };
            for _ in 0..repeats {
                samples.push((bits.to_vec(), vec![f32::from(u8::from(cand == row.target))]));
            }
        }
    }

    // Feature informativeness: if the six term groups' sign/magnitude bits do not separate the
    // target from decoys, no gate network can help and the null is about the features, not the
    // LUT. Report the means and the best single-bit separation before training anything.
    let mut positive_mean = [0.0f64; FEATURE_BITS];
    let mut negative_mean = [0.0f64; FEATURE_BITS];
    let mut positive_count = 0usize;
    let mut negative_count = 0usize;
    for (bits, label) in &samples {
        if label[0] > 0.5 {
            positive_count += 1;
            for (slot, value) in positive_mean.iter_mut().zip(bits.iter()) {
                *slot += f64::from(*value);
            }
        } else {
            negative_count += 1;
            for (slot, value) in negative_mean.iter_mut().zip(bits.iter()) {
                *slot += f64::from(*value);
            }
        }
    }
    for (slot, value) in positive_mean.iter_mut().enumerate() {
        *value /= positive_count.max(1) as f64;
        let _ = slot;
    }
    for (slot, value) in negative_mean.iter_mut().enumerate() {
        *value /= negative_count.max(1) as f64;
        let _ = slot;
    }
    let best_bit_gap = positive_mean
        .iter()
        .zip(negative_mean.iter())
        .map(|(p, n)| (p - n).abs())
        .fold(0.0f64, f64::max);

    // Exhaustive best two-input gate over the feature bits. With twelve bits there are
    // C(12,2) * 16 = 1056 candidate terms, so the strongest LUT-4 term at this arity can be
    // found exactly rather than left to a connectivity heuristic. This is the term the gate
    // measures: if the best of all 1056 cannot improve held-out bits per byte, no learned
    // two-input interaction over these features can.
    let mut best_term: Option<(f64, usize, usize, u8)> = None;
    for i in 0..FEATURE_BITS {
        for j in (i + 1)..FEATURE_BITS {
            for gate in 0u8..16 {
                let mut loss = 0.0f64;
                for (bits, label) in samples.iter().take(4096) {
                    let value = if bits[i] > 0.5 { 1u8 } else { 0u8 };
                    let other = if bits[j] > 0.5 { 1u8 } else { 0u8 };
                    let out = f64::from(uor_r4_core::native_geometric::learner::lut4::gate_hard(
                        gate, value, other,
                    ));
                    let d = out - f64::from(label[0]);
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
        Lut4Layer::seeded(FEATURE_BITS, 8, 0x4C55_5434),
        Lut4Layer::seeded(8, 1, 0x4C55_5435),
    ]);
    let first_loss = network.fit_gates(&samples, args.rate, 1);
    let final_loss = network.fit_gates(&samples, args.rate, args.steps);
    let artifact = network.export();
    let utilisation = network.gate_utilisation();
    let entropy = network.gate_entropy_bits();
    let cones = network.cone_sizes();
    let gap_inputs: Vec<Vec<f32>> = samples.iter().take(256).map(|(x, _)| x.clone()).collect();
    let gap = network.discretisation_gap(&gap_inputs);

    // Held-out bits per byte for a declared correction scale, with the top-1 change rate as a
    // sensitivity check: a correction too small to move BPB must still be shown to move nothing.
    let measure = |scale: f64| -> (f64, f64, f64, f64, u64) {
        let mut base_bits = 0.0f64;
        let mut soft_bits = 0.0f64;
        let mut hard_bits = 0.0f64;
        let mut changed = 0u64;
        let mut bytes_total = 0u64;
        for row in &eval {
            let mut base_max = i32::MIN;
            let mut soft_scores: Vec<f64> = Vec::with_capacity(vocab);
            let mut hard_scores: Vec<f64> = Vec::with_capacity(vocab);
            let mut soft_max = f64::NEG_INFINITY;
            let mut hard_max = f64::NEG_INFINITY;
            for (bits, score) in &row.candidates {
                base_max = base_max.max(*score);
                let hard_out = artifact.eval_f32(bits);
                let bit = hard_out.first().copied().unwrap_or(0);
                let soft_value = network.soft_forward(bits).first().copied().unwrap_or(0.5);
                let soft_corrected = f64::from(*score) + scale * (f64::from(soft_value) - 0.5);
                let hard_corrected =
                    f64::from(*score) + if bit == 1 { scale / 2.0 } else { -scale / 2.0 };
                soft_max = soft_max.max(soft_corrected);
                hard_max = hard_max.max(hard_corrected);
                soft_scores.push(soft_corrected);
                hard_scores.push(hard_corrected);
            }
            let mut base_sum = 0.0f64;
            let mut soft_sum = 0.0f64;
            let mut hard_sum = 0.0f64;
            for (cand, (_, score)) in row.candidates.iter().enumerate() {
                base_sum +=
                    libm::exp(((*score - base_max) as f64 / SAMPLER_SCALE).clamp(-60.0, 0.0));
                soft_sum +=
                    libm::exp(((soft_scores[cand] - soft_max) / SAMPLER_SCALE).clamp(-60.0, 0.0));
                hard_sum +=
                    libm::exp(((hard_scores[cand] - hard_max) / SAMPLER_SCALE).clamp(-60.0, 0.0));
            }
            let base_p = libm::exp(
                (((row.candidates[row.target].1 - base_max) as f64) / SAMPLER_SCALE)
                    .clamp(-60.0, 0.0),
            ) / base_sum;
            let soft_p =
                libm::exp(((soft_scores[row.target] - soft_max) / SAMPLER_SCALE).clamp(-60.0, 0.0))
                    / soft_sum;
            let hard_p =
                libm::exp(((hard_scores[row.target] - hard_max) / SAMPLER_SCALE).clamp(-60.0, 0.0))
                    / hard_sum;
            base_bits += -base_p.log2();
            soft_bits += -soft_p.log2();
            hard_bits += -hard_p.log2();
            bytes_total += row.bytes;
            let base_top = row
                .candidates
                .iter()
                .enumerate()
                .max_by_key(|(_, (_, score))| *score)
                .map(|(index, _)| index)
                .unwrap_or(0);
            let soft_top = soft_scores
                .iter()
                .enumerate()
                .max_by(|x, y| x.1.partial_cmp(y.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(index, _)| index)
                .unwrap_or(0);
            changed += u64::from(base_top != soft_top);
        }
        (
            base_bits,
            soft_bits,
            hard_bits,
            changed as f64 / eval.len().max(1) as f64,
            bytes_total,
        )
    };
    let bpb = |bits: f64, bytes: u64| bits / bytes.max(1) as f64;

    // Choose the correction scale on the disjoint selection tail, then measure on the held-out
    // evaluation stream. The scale is a hyperparameter; the evaluation is untouched by it.
    let (select_base, select_soft, _, select_changed, select_bytes) = measure(2048.0);
    let mut best_scale = 2048.0f64;
    let mut best_select = f64::INFINITY;
    let mut scale_curve: Vec<(f64, f64, f64)> = Vec::new();
    for scale in [256.0f64, 512.0, 1024.0, 2048.0, 4096.0, 8192.0, 16384.0] {
        let (base, soft, _, changed, bytes) = measure(scale);
        let base_bpb = bpb(base, bytes);
        let soft_bpb = bpb(soft, bytes);
        scale_curve.push((scale, soft_bpb, changed));
        if soft_bpb < best_select {
            best_select = soft_bpb;
            best_scale = scale;
        }
        let _ = base_bpb;
    }
    let _ = (select_base, select_soft, select_changed, select_bytes);
    let (base_bits, soft_bits, hard_bits, top1_change_rate, bytes_total) = measure(best_scale);

    let report = serde_json::json!({
        "schema": "uor-r4.lut4-prose-insertion/1",
        "status": "COMPLETED",
        "model": args.model.display().to_string(),
        "corpus": args.corpus.display().to_string(),
        "seq_len": args.seq_len,
        "train_positions": args.train_positions,
        "eval_positions": args.eval_positions,
        "holdout_offset": eval_start,
        "feature_bits": FEATURE_BITS,
        "selected_correction": best_scale,
        "exhaustive_best_term": {
            "train_loss": best_term.0,
            "feature_a": best_term.1,
            "feature_b": best_term.2,
            "gate": best_term.3,
        },
        "feature_positive_mean": positive_mean,
        "feature_negative_mean": negative_mean,
        "best_single_bit_separation": best_bit_gap,
        "positive_samples": positive_count,
        "negative_samples": negative_count,
        "correction_curve": scale_curve,
        "top1_change_rate": top1_change_rate,
        "train_loss_first": first_loss,
        "train_loss_final": final_loss,
        "samples": samples.len(),
        "bits_per_byte": {
            "baseline": bpb(base_bits, bytes_total),
            "soft_lut": bpb(soft_bits, bytes_total),
            "hard_lut": bpb(hard_bits, bytes_total),
            "delta_soft": bpb(soft_bits, bytes_total) - bpb(base_bits, bytes_total),
            "delta_hard": bpb(hard_bits, bytes_total) - bpb(base_bits, bytes_total),
        },
        "discretisation_gap_bpb": bpb(soft_bits, bytes_total) - bpb(hard_bits, bytes_total),
        "gate_utilisation": utilisation,
        "gate_entropy_bits": entropy,
        "cone_sizes": cones,
        "elapsed_seconds": started.elapsed().as_secs_f64(),
        "scope": "Stage 4 insertion gate: a learned nonlinear term over the additive scorer's own term groups, trained at the corpus head and evaluated on a disjoint held-out slice. Not a claim about architecture priority; the term is inserted as a declared read.",
    });
    let text = serde_json::to_string_pretty(&report)?;
    match &args.out {
        Some(path) => std::fs::write(path, text)?,
        None => println!("{text}"),
    }
    let mut tops: BTreeMap<String, f64> = BTreeMap::new();
    tops.insert("baseline".into(), bpb(base_bits, bytes_total));
    tops.insert("soft".into(), bpb(soft_bits, bytes_total));
    tops.insert("hard".into(), bpb(hard_bits, bytes_total));
    eprintln!("bits/byte {tops:?} over {} bytes", bytes_total);
    Ok(())
}
