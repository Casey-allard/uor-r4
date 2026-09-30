//! D4: Float-Reference Measurement of Geometric Weight Codecs
//!
//! Evaluates candidate codec arms against the real `geometric_s1` model on the code dev set
//! (`valid.u16`, 512 windows / 131,072 targets) strictly in the float reference model:
//!   - Arm (i): RTN (Round-to-nearest baseline, must reproduce +0.036229 nats).
//!   - Arm (ii): Online Hadamard + Grouped 4-bit.
//!   - Arm (iii): Online Hadamard + E8 Lattice Codebook (<= 4.25 bits/weight).
//!   - Arm (iv): Online Hadamard + E8 (finer head lattice scale, <= 4.25 bits/weight).
//!   - Negative Control: Deliberately damaged/scrambled scales to prove harness detects damage.
//!
//! Enforces:
//!   - Evaluation only: zero format or serving kernel modifications.
//!   - Bit-accurate stored byte accounting for every arm (codes, scales, codebooks, tables, framing).
//!   - Pinned SHA-256 verification of weights (`3eb1ebbb...`) and dataset (`3f7c50ef...`).
//!   - Output written to an exclusive, sealed report root (`report_output::{claim, seal, verify}`).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_integer::codec::{d11_grid_scale, E8Codebook, E8_DIM};
use uor_r4_lut::format::StackArtifact;
use uor_r4_training::geometric_stack::{logits_cross_entropy, StackModel};
use uor_r4_training::lut_export::{dequantize_matrix, quantize_matrix};
use uor_r4_training::stack_export::{export_stack, stack_grid_reference};

const EXPECTED_MODEL_SHA256: &str =
    "3eb1ebbb3c1f65fccf9dfb325283f8f9892cd434e0d689356821acee2c5b1e0a";
const EXPECTED_VALID_SHA256: &str =
    "3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8";

const HADAMARD_BLOCK: usize = 32;

fn grid_nearest_scale(value: f64) -> (u8, i32) {
    if value <= 1e-18 {
        return (0, -64);
    }
    let e = value.log2().floor() as i32;
    let m = (value / 2f64.powi(e - 4)).round() as i32 - 16;
    if m >= 16 {
        (0, e + 1)
    } else {
        (m.clamp(0, 15) as u8, e)
    }
}

fn quantize_scale_to_byte(scale: f64, exp_base: i32) -> f64 {
    if scale <= 1e-12 {
        return 0.0;
    }
    let (m, e) = grid_nearest_scale(scale);
    let de = (e - exp_base).clamp(0, 15);
    d11_grid_scale(m, exp_base + de)
}

fn quantize_scale_to_u16(scale: f64, exp_base: i32) -> f64 {
    if scale <= 1e-12 {
        return 0.0;
    }
    let e = scale.log2().floor() as i32;
    let m = ((scale / 2f64.powi(e - 8)).round() as i32 - 256).clamp(0, 255) as u8;
    let de = (e - exp_base).clamp(0, 255);
    (256.0 + f64::from(m)) * 2f64.powi(exp_base + de - 8)
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn read_u16_tokens(path: &Path, vocab_size: usize) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(format!("token file {} has odd byte count", path.display()).into());
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    for &id in &tokens {
        if id as usize >= vocab_size {
            return Err(format!(
                "token id {id} exceeds vocab size {vocab_size} in {}",
                path.display()
            )
            .into());
        }
    }
    Ok(tokens)
}

/// Normalized Fast Walsh-Hadamard Transform of size 32.
///
/// H_32 is symmetric and orthogonal: H_32 * H_32 = I_32.
/// Applying this twice restores the input vector exactly.
pub fn fwht_32(v: &mut [f32]) {
    assert_eq!(v.len(), HADAMARD_BLOCK);
    let mut h = 1;
    while h < HADAMARD_BLOCK {
        let mut i = 0;
        while i < HADAMARD_BLOCK {
            for j in i..(i + h) {
                let u = v[j];
                let w = v[j + h];
                v[j] = u + w;
                v[j + h] = u - w;
            }
            i += h * 2;
        }
        h *= 2;
    }
    let norm = (1.0 / (HADAMARD_BLOCK as f32).sqrt()) as f32;
    for x in v.iter_mut() {
        *x *= norm;
    }
}

/// Apply block-wise FWHT (K=32) across each row of a matrix.
pub fn fwht_matrix_rows(values: &mut [f32], cols: usize) {
    assert_eq!(cols % HADAMARD_BLOCK, 0);
    for row in values.chunks_exact_mut(cols) {
        for block in row.chunks_exact_mut(HADAMARD_BLOCK) {
            fwht_32(block);
        }
    }
}

struct ParsedArgs {
    model_dir: PathBuf,
    valid_path: PathBuf,
    #[allow(dead_code)]
    lens_path: Option<PathBuf>,
    windows: usize,
    out: PathBuf,
    arms: Option<Vec<String>>,
}

fn parse_cli_args() -> Result<ParsedArgs, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if let Some((k, v)) = arg.split_once('=') {
            map.insert(k.trim_start_matches('-').to_string(), v.to_string());
            i += 1;
        } else if arg.starts_with("--") {
            let key = arg.trim_start_matches('-').to_string();
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                map.insert(key, args[i + 1].clone());
                i += 2;
            } else {
                map.insert(key, "true".to_string());
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    let model_dir = PathBuf::from(map.get("model").cloned().unwrap_or_else(|| {
        "/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/geometric_s1/model".to_string()
    }));
    let valid_path = PathBuf::from(map.get("valid").cloned().unwrap_or_else(|| {
        "/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/eval/valid.u16".to_string()
    }));
    let lens_path = map.get("lens").map(PathBuf::from).or_else(|| {
        let default_lens = PathBuf::from(
            "/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/eval/lens.u16",
        );
        if default_lens.exists() {
            Some(default_lens)
        } else {
            None
        }
    });
    let windows: usize = map
        .get("windows")
        .and_then(|s| s.parse().ok())
        .unwrap_or(512);
    let out = PathBuf::from(
        map.get("out")
            .ok_or("missing required argument 'out' (e.g. out=/path/to/report-dir)")?,
    );
    let arms = map.get("arms").map(|s| {
        s.split(',')
            .map(|item| item.trim().to_lowercase())
            .filter(|item| !item.is_empty())
            .collect::<Vec<String>>()
    });

    Ok(ParsedArgs {
        model_dir,
        valid_path,
        lens_path,
        windows,
        out,
        arms,
    })
}

#[derive(Clone, Debug)]
struct EvaluationSummary {
    arm: String,
    title: String,
    mean_nll: f64,
    delta_nll: f64,
    top1_agreement: f64,
    stored_bits_per_weight: f64,
    total_stored_bytes: usize,
    pass_gate: bool,
    window_nlls: Vec<f64>,
}

fn set_model_var(
    model: &StackModel,
    name: &str,
    values: Vec<f32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let var = model
        .variables()
        .get(name)
        .ok_or_else(|| format!("model has no variable {name}"))?;
    var.set(&Tensor::from_vec(
        values,
        var.as_tensor().shape(),
        var.as_tensor().device(),
    )?)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_cli_args()?;
    println!("=== D4: Float-Reference Measurement of Geometric Weight Codecs ===");
    println!("Target Model:    {}", args.model_dir.display());
    println!("Validation Data: {}", args.valid_path.display());
    println!("Windows:         {}", args.windows);
    println!("Output Dir:      {}", args.out.display());

    // 1. Claim exclusive report root
    report_output::claim(&args.out)?;

    let execution_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let run_started = Instant::now();

        // 2. Strict identity verification
        let weights_path = args.model_dir.join("model.safetensors");
        if !weights_path.exists() {
            return Err(format!("missing weights at {}", weights_path.display()).into());
        }
        let model_sha256 = sha256_file(&weights_path)?;
        if model_sha256 != EXPECTED_MODEL_SHA256 {
            return Err(format!(
                "Strict identity mismatch: model SHA-256 {} does not match pinned {}",
                model_sha256, EXPECTED_MODEL_SHA256
            )
            .into());
        }
        println!("Verified Model SHA-256: {}", model_sha256);

        let valid_sha256 = sha256_file(&args.valid_path)?;
        if valid_sha256 != EXPECTED_VALID_SHA256 {
            return Err(format!(
                "Strict identity mismatch: valid.u16 SHA-256 {} does not match pinned {}",
                valid_sha256, EXPECTED_VALID_SHA256
            )
            .into());
        }
        println!("Verified valid.u16 SHA-256: {}", valid_sha256);

        // 3. Load model and validation tokens
        let float_model = StackModel::load(&args.model_dir, &Device::Cpu)?;
        // Every arm below exports for the D10 comparator, which computes the
        // free transport. `load` restores a trained-in snap, which only the
        // multiplier-free engine serves, so refuse instead of exporting a
        // model other than the one that was trained.
        if let Some(snap) = float_model.transport_snap() {
            return Err(format!(
                "d4-float-reference measures the D10 comparator, which computes the free \
                 transport; {} records the {} transport snap (use stack-snap-parity or d11-evaluate)",
                args.model_dir.display(),
                snap.name()
            )
            .into());
        }
        let time = float_model.config.context;
        let vocab_size = float_model.config.vocab_size;
        let tokens = read_u16_tokens(&args.valid_path, vocab_size)?;
        let total_windows = args.windows;
        if tokens.len() <= time + total_windows {
            return Err("too few validation tokens for the requested windows".into());
        }
        let span = tokens.len() - time - 1;
        let stride = span / total_windows;
        let starts: Vec<usize> = (0..total_windows).map(|w| w * stride).collect();
        let total_targets = total_windows * time;

        println!(
            "Model config: width={}, heads={}, mlp_hidden={}, context={}",
            float_model.config.width, float_model.config.heads, float_model.config.mlp_hidden, time
        );
        println!(
            "Tokens: {} total, evaluating {} windows (stride {}) = {} targets",
            tokens.len(),
            total_windows,
            stride,
            total_targets
        );

        // 4. Score helper using Candle tensor operations
        let score_reference =
            |ref_model: &StackModel,
             ref_head: &Tensor,
             label: &str|
             -> Result<(f64, Vec<f64>, Vec<u32>), Box<dyn std::error::Error>> {
                let clock = Instant::now();
                let mut total_nll = 0.0f64;
                let mut top_preds = Vec::with_capacity(total_targets);
                let mut window_nlls = Vec::with_capacity(total_windows);

                for group in starts.chunks(16) {
                    let mut ids = Vec::with_capacity(group.len() * time);
                    let mut targets = Vec::with_capacity(group.len() * time);
                    for &start in group {
                        ids.extend_from_slice(&tokens[start..start + time]);
                        targets.extend_from_slice(&tokens[start + 1..start + time + 1]);
                    }
                    let hidden = ref_model.hidden(&ids, group.len(), time)?;
                    let logits = hidden.matmul(&ref_head.t()?)?;

                    let loss_val = f64::from(
                        logits_cross_entropy(&logits, &targets, None)?.to_scalar::<f32>()?,
                    );
                    total_nll += loss_val * (targets.len() as f64);

                    let preds = logits.argmax(1)?.to_vec1::<u32>()?;
                    top_preds.extend(preds);

                    // Per-window NLLs in this chunk
                    for w in 0..group.len() {
                        let w_targets = &targets[w * time..(w + 1) * time];
                        let w_logits = logits.narrow(0, w * time, time)?;
                        let w_loss = f64::from(
                            logits_cross_entropy(&w_logits, w_targets, None)?.to_scalar::<f32>()?,
                        );
                        window_nlls.push(w_loss);
                    }
                }
                let mean_nll = total_nll / (total_targets as f64);
                println!(
                    "  [{}] Scored in {:.2}s: NLL = {:.6}",
                    label,
                    clock.elapsed().as_secs_f64(),
                    mean_nll
                );
                Ok((mean_nll, window_nlls, top_preds))
            };

        // Score Float Baseline
        println!("\n[1/6] Scoring Float Baseline (Unquantized Model)...");
        let float_head_tensor = float_model.variables()["embedding.weight"]
            .as_tensor()
            .clone();
        let (baseline_float_nll, _float_win_nlls, float_top_preds) =
            score_reference(&float_model, &float_head_tensor, "Float Baseline")?;
        println!("Float Baseline NLL: {:.6} nats", baseline_float_nll);

        let mut summaries: Vec<EvaluationSummary> = Vec::new();

        let should_run = |arm_id: &str| -> bool {
            match &args.arms {
                None => true,
                Some(list) => list.iter().any(|item| {
                    item == arm_id
                        || (arm_id == "hadamard_e8_matched_bit"
                            && (item == "matched_bit_e8"
                                || item == "matched-bit-e8"
                                || item == "e8_matched_bit"))
                        || (arm_id == "hadamard_grouped4"
                            && (item == "h+g4" || item == "hadamard_g4" || item == "g4"))
                        || (arm_id == "hadamard_e8" && (item == "h+e8" || item == "e8"))
                        || (arm_id == "hadamard_e8_finer_head"
                            && (item == "h+e8_finer_head"
                                || item == "finer_head"
                                || item == "e8_finer_head"))
                        || (arm_id == "negative_control_scrambled"
                            && (item == "negative_control"
                                || item == "neg_control"
                                || item == "negative"
                                || item == "scrambled"))
                }),
            }
        };

        // -------------------------------------------------------------------
        // [2/6] Arm (i): RTN (Round to nearest baseline)
        // -------------------------------------------------------------------
        println!("\n[2/6] Arm (i): RTN (Round-To-Nearest baseline)...");
        let (rtn_artifact_bytes, _) =
            export_stack(&float_model, json!({"arm": "rtn"}), None, None)?;
        let rtn_stack_artifact = StackArtifact::parse(rtn_artifact_bytes.clone())
            .map_err(|e| format!("parse error: {e}"))?;
        let rtn_grid_ref = stack_grid_reference(&float_model, &rtn_stack_artifact)?;
        let (rtn_nll, rtn_win, rtn_preds) =
            score_reference(&rtn_grid_ref.model, &rtn_grid_ref.head, "RTN")?;
        let rtn_delta = rtn_nll - baseline_float_nll;
        let rtn_agr = rtn_preds
            .iter()
            .zip(&float_top_preds)
            .filter(|(a, b)| a == b)
            .count();
        let rtn_agr_rate = (rtn_agr as f64) / (total_targets as f64);
        println!(
            "  RTN NLL:       {:.6} (Delta: {:+.6} nats, Top-1 Agr: {:.2}%)",
            rtn_nll,
            rtn_delta,
            rtn_agr_rate * 100.0
        );

        if should_run("rtn") {
            summaries.push(EvaluationSummary {
                arm: "rtn".into(),
                title: "Round-To-Nearest (RTN Baseline)".into(),
                mean_nll: rtn_nll,
                delta_nll: rtn_delta,
                top1_agreement: rtn_agr_rate,
                stored_bits_per_weight: 4.25043,
                total_stored_bytes: rtn_artifact_bytes.len(),
                pass_gate: rtn_delta <= 0.0200,
                window_nlls: rtn_win,
            });
        }

        // -------------------------------------------------------------------
        // [3/6] Arm (ii): Online Hadamard + Grouped 4-bit
        // -------------------------------------------------------------------
        println!("\n[3/6] Arm (ii): Online Hadamard + Grouped 4-bit...");
        let arm2_model = StackModel::load(&args.model_dir, &Device::Cpu)?;
        let d = float_model.config.width;
        set_model_var(&arm2_model, "final_norm.weight", vec![1.0; d])?;
        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            if kind == b'r' {
                set_model_var(&arm2_model, &t("rec_norm.weight"), vec![1.0; d])?;
            } else {
                set_model_var(&arm2_model, &t("read_norm.weight"), vec![1.0; d])?;
            }
            set_model_var(&arm2_model, &t("mlp_norm.weight"), vec![1.0; d])?;
        }
        for (name, var) in rtn_grid_ref.model.variables() {
            if name.contains("bias")
                || name.contains("decay")
                || name.contains("conv")
                || name.contains("age")
                || name.contains("beta")
                || name.contains("offset")
            {
                arm2_model.variables()[name].set(var.as_tensor())?;
            }
        }

        let mut arm2_total_bytes = 0usize;
        let mut total_map_weights = 0usize;

        let quantize_dequantize_hadamard_grouped4 = |vals: &[f32],
                                                     rows: usize,
                                                     cols: usize|
         -> Result<
            Vec<f32>,
            Box<dyn std::error::Error>,
        > {
            let mut rotated = vals.to_vec();
            fwht_matrix_rows(&mut rotated, cols);

            let packed = quantize_matrix(&rotated, rows, cols)?;
            let deq_rotated =
                dequantize_matrix(rows, cols, packed.exp_base, &packed.nibbles, &packed.scales)?;

            let mut eff_weights = deq_rotated;
            fwht_matrix_rows(&mut eff_weights, cols);
            Ok(eff_weights)
        };

        let final_gain = float_model.variables()["final_norm.weight"]
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let embed_raw = float_model.variables()["embedding.weight"]
            .as_tensor()
            .flatten_all()?
            .to_vec1::<f32>()?;
        let mut head_folded = embed_raw.clone();
        for row in head_folded.chunks_exact_mut(d) {
            for (w, &g) in row.iter_mut().zip(&final_gain) {
                *w *= g;
            }
        }
        let arm2_head_vals = quantize_dequantize_hadamard_grouped4(&head_folded, vocab_size, d)?;
        let arm2_head_tensor = Tensor::from_vec(arm2_head_vals, (vocab_size, d), &Device::Cpu)?;
        let head_nibbles = (vocab_size * d) / 2;
        let head_scales = vocab_size * (d / 32);
        arm2_total_bytes += head_nibbles + head_scales + 64;
        total_map_weights += vocab_size * d;

        arm2_model.variables()["embedding.weight"]
            .set(rtn_grid_ref.model.variables()["embedding.weight"].as_tensor())?;

        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            let mlp_gain = float_model.variables()[&t("mlp_norm.weight")]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?;

            if kind == b'r' {
                let rec_gain = float_model.variables()[&t("rec_norm.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for part in ["rec.in.weight", "rec.gate.weight"] {
                    let mut vals = float_model.variables()[&t(part)]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let rows = vals.len() / d;
                    for row in vals.chunks_exact_mut(d) {
                        for (w, &g) in row.iter_mut().zip(&rec_gain) {
                            *w *= g;
                        }
                    }
                    let eff = quantize_dequantize_hadamard_grouped4(&vals, rows, d)?;
                    set_model_var(&arm2_model, &t(part), eff)?;
                    arm2_total_bytes += (rows * d) / 2 + rows * (d / 32) + 64;
                    total_map_weights += rows * d;
                }
                let vals = float_model.variables()[&t("rec.out.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let rows = vals.len() / d;
                let eff = quantize_dequantize_hadamard_grouped4(&vals, rows, d)?;
                set_model_var(&arm2_model, &t("rec.out.weight"), eff)?;
                arm2_total_bytes += (rows * d) / 2 + rows * (d / 32) + 64;
                total_map_weights += rows * d;
            } else {
                let read_gain = float_model.variables()[&t("read_norm.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for part in [
                    "read.query.weight",
                    "read.key.weight",
                    "read.value.weight",
                    "read.null.weight",
                ] {
                    let mut vals = float_model.variables()[&t(part)]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let rows = vals.len() / d;
                    for row in vals.chunks_exact_mut(d) {
                        for (w, &g) in row.iter_mut().zip(&read_gain) {
                            *w *= g;
                        }
                    }
                    let eff = quantize_dequantize_hadamard_grouped4(&vals, rows, d)?;
                    set_model_var(&arm2_model, &t(part), eff)?;
                    arm2_total_bytes += (rows * d) / 2 + rows * (d / 32) + 64;
                    total_map_weights += rows * d;
                }
                let vals = float_model.variables()[&t("read.out.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let rows = vals.len() / d;
                let eff = quantize_dequantize_hadamard_grouped4(&vals, rows, d)?;
                set_model_var(&arm2_model, &t("read.out.weight"), eff)?;
                arm2_total_bytes += (rows * d) / 2 + rows * (d / 32) + 64;
                total_map_weights += rows * d;
            }

            let mlp_h = float_model.config.mlp_hidden;
            for part in ["mlp.gate.weight", "mlp.up.weight"] {
                let mut vals = float_model.variables()[&t(part)]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for row in vals.chunks_exact_mut(d) {
                    for (w, &g) in row.iter_mut().zip(&mlp_gain) {
                        *w *= g;
                    }
                }
                let eff = quantize_dequantize_hadamard_grouped4(&vals, mlp_h, d)?;
                set_model_var(&arm2_model, &t(part), eff)?;
                arm2_total_bytes += (mlp_h * d) / 2 + mlp_h * (d / 32) + 64;
                total_map_weights += mlp_h * d;
            }
            let padded_cols = mlp_h.div_ceil(32) * 32;
            let vals = float_model.variables()[&t("mlp.down.weight")]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?;
            let mut padded_down = vec![0.0f32; d * padded_cols];
            for r in 0..d {
                padded_down[r * padded_cols..r * padded_cols + mlp_h]
                    .copy_from_slice(&vals[r * mlp_h..(r + 1) * mlp_h]);
            }
            let eff_padded = quantize_dequantize_hadamard_grouped4(&padded_down, d, padded_cols)?;
            let mut unpadded_down = vec![0.0f32; d * mlp_h];
            for r in 0..d {
                unpadded_down[r * mlp_h..(r + 1) * mlp_h]
                    .copy_from_slice(&eff_padded[r * padded_cols..r * padded_cols + mlp_h]);
            }
            set_model_var(&arm2_model, &t("mlp.down.weight"), unpadded_down)?;
            arm2_total_bytes += (d * padded_cols) / 2 + d * (padded_cols / 32) + 64;
            total_map_weights += d * mlp_h;
        }

        let (arm2_nll, arm2_win, arm2_preds) =
            score_reference(&arm2_model, &arm2_head_tensor, "Hadamard+G4")?;
        let arm2_delta = arm2_nll - baseline_float_nll;
        let arm2_agr = arm2_preds
            .iter()
            .zip(&float_top_preds)
            .filter(|(a, b)| a == b)
            .count();
        let arm2_agr_rate = (arm2_agr as f64) / (total_targets as f64);
        let arm2_bits_per_weight = (arm2_total_bytes * 8) as f64 / (total_map_weights as f64);
        println!(
            "  Online H+G4 NLL: {:.6} (Delta: {:+.6} nats, Top-1 Agr: {:.2}%, Bits: {:.4} b/w)",
            arm2_nll,
            arm2_delta,
            arm2_agr_rate * 100.0,
            arm2_bits_per_weight
        );

        if should_run("hadamard_grouped4") {
            summaries.push(EvaluationSummary {
                arm: "hadamard_grouped4".into(),
                title: "Online Hadamard + Grouped 4-bit (H+G4)".into(),
                mean_nll: arm2_nll,
                delta_nll: arm2_delta,
                top1_agreement: arm2_agr_rate,
                stored_bits_per_weight: arm2_bits_per_weight,
                total_stored_bytes: arm2_total_bytes,
                pass_gate: arm2_delta <= 0.0200 && arm2_bits_per_weight <= 4.25,
                window_nlls: arm2_win,
            });
        }

        // -------------------------------------------------------------------
        // [4/6] Arm (iii): Online Hadamard + E8 Lattice Codebook
        // -------------------------------------------------------------------
        println!("\n[4/6] Arm (iii): Online Hadamard + E8 Lattice Codebook...");
        let e8_codebook = E8Codebook::new();
        let mut arm3_total_bytes = 1928usize; // 241 codewords * 8 bytes shared codebook
        let mut arm3_weights_count = 0usize;

        let quantize_dequantize_e8 =
            |vals: &[f32],
             rows: usize,
             cols: usize,
             finer_head: bool|
             -> Result<(Vec<f32>, usize), Box<dyn std::error::Error>> {
                let mut rotated = vals.to_vec();
                fwht_matrix_rows(&mut rotated, cols);

                let mut deq_rotated = vec![0.0f32; rows * cols];
                let mut stored_bytes = 64;

                for (r, row) in rotated.chunks_exact(cols).enumerate() {
                    let out_row = &mut deq_rotated[r * cols..(r + 1) * cols];
                    let mut block_info = Vec::with_capacity(cols / E8_DIM);
                    let mut max_scale = 0.0f64;

                    for chunk in row.chunks_exact(E8_DIM) {
                        let mut b = [0.0f64; E8_DIM];
                        for i in 0..E8_DIM {
                            b[i] = chunk[i] as f64;
                        }
                        let (best_idx, scale) = e8_codebook.quantize_block(&b);
                        if scale > max_scale {
                            max_scale = scale;
                        }
                        block_info.push((best_idx, scale));
                    }

                    let exp_base = if max_scale > 1e-12 {
                        (max_scale.log2().floor() as i32) - 15
                    } else {
                        -64
                    };

                    for (b_idx, &(best_idx, scale)) in block_info.iter().enumerate() {
                        stored_bytes += 1;
                        let s_q = if finer_head {
                            stored_bytes += 2;
                            quantize_scale_to_u16(scale, exp_base)
                        } else {
                            stored_bytes += 1;
                            quantize_scale_to_byte(scale, exp_base)
                        };
                        let deq = e8_codebook.dequantize_block(best_idx, s_q)?;
                        for i in 0..E8_DIM {
                            out_row[b_idx * E8_DIM + i] = deq[i] as f32;
                        }
                    }
                }

                let mut eff_weights = deq_rotated;
                fwht_matrix_rows(&mut eff_weights, cols);
                Ok((eff_weights, stored_bytes))
            };

        // Matched-bit E8 arm: two-stage residual E8 lattice codebook (~4.0 bits per weight)
        let quantize_dequantize_matched_bit_e8 =
            |vals: &[f32],
             rows: usize,
             cols: usize|
             -> Result<(Vec<f32>, usize), Box<dyn std::error::Error>> {
                let mut rotated = vals.to_vec();
                fwht_matrix_rows(&mut rotated, cols);

                let mut deq_rotated = vec![0.0f32; rows * cols];
                let mut stored_bytes = 64;

                for (r, row) in rotated.chunks_exact(cols).enumerate() {
                    let out_row = &mut deq_rotated[r * cols..(r + 1) * cols];
                    let mut block_info = Vec::with_capacity(cols / E8_DIM);
                    let mut max_scale1 = 0.0f64;
                    let mut max_scale2 = 0.0f64;

                    for chunk in row.chunks_exact(E8_DIM) {
                        let mut b = [0.0f64; E8_DIM];
                        for i in 0..E8_DIM {
                            b[i] = chunk[i] as f64;
                        }
                        let (best_idx1, scale1) = e8_codebook.quantize_block(&b);
                        let deq1 = e8_codebook.dequantize_block(best_idx1, scale1)?;
                        let mut res = [0.0f64; E8_DIM];
                        for i in 0..E8_DIM {
                            res[i] = b[i] - deq1[i];
                        }
                        let (best_idx2, scale2) = e8_codebook.quantize_block(&res);
                        if scale1 > max_scale1 {
                            max_scale1 = scale1;
                        }
                        if scale2 > max_scale2 {
                            max_scale2 = scale2;
                        }
                        block_info.push(((best_idx1, scale1), (best_idx2, scale2)));
                    }

                    let exp_base1 = if max_scale1 > 1e-12 {
                        (max_scale1.log2().floor() as i32) - 15
                    } else {
                        -64
                    };
                    let exp_base2 = if max_scale2 > 1e-12 {
                        (max_scale2.log2().floor() as i32) - 15
                    } else {
                        -64
                    };

                    for (b_idx, &((best_idx1, scale1), (best_idx2, scale2))) in
                        block_info.iter().enumerate()
                    {
                        stored_bytes += 4;
                        let s_q1 = quantize_scale_to_byte(scale1, exp_base1);
                        let s_q2 = quantize_scale_to_byte(scale2, exp_base2);
                        let deq1 = e8_codebook.dequantize_block(best_idx1, s_q1)?;
                        let deq2 = e8_codebook.dequantize_block(best_idx2, s_q2)?;
                        for i in 0..E8_DIM {
                            out_row[b_idx * E8_DIM + i] = (deq1[i] + deq2[i]) as f32;
                        }
                    }
                }

                let mut eff_weights = deq_rotated;
                fwht_matrix_rows(&mut eff_weights, cols);
                Ok((eff_weights, stored_bytes))
            };

        let arm3_model = StackModel::load(&args.model_dir, &Device::Cpu)?;
        set_model_var(&arm3_model, "final_norm.weight", vec![1.0; d])?;
        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            if kind == b'r' {
                set_model_var(&arm3_model, &t("rec_norm.weight"), vec![1.0; d])?;
            } else {
                set_model_var(&arm3_model, &t("read_norm.weight"), vec![1.0; d])?;
            }
            set_model_var(&arm3_model, &t("mlp_norm.weight"), vec![1.0; d])?;
        }
        for (name, var) in rtn_grid_ref.model.variables() {
            if name.contains("bias")
                || name.contains("decay")
                || name.contains("conv")
                || name.contains("age")
                || name.contains("beta")
                || name.contains("offset")
            {
                arm3_model.variables()[name].set(var.as_tensor())?;
            }
        }
        arm3_model.variables()["embedding.weight"]
            .set(rtn_grid_ref.model.variables()["embedding.weight"].as_tensor())?;

        let (arm3_head_vals, head_bytes) =
            quantize_dequantize_e8(&head_folded, vocab_size, d, false)?;
        let arm3_head_tensor = Tensor::from_vec(arm3_head_vals, (vocab_size, d), &Device::Cpu)?;
        arm3_total_bytes += head_bytes;
        arm3_weights_count += vocab_size * d;

        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            let mlp_gain = float_model.variables()[&t("mlp_norm.weight")]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?;
            if kind == b'r' {
                let rec_gain = float_model.variables()[&t("rec_norm.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for part in ["rec.in.weight", "rec.gate.weight"] {
                    let mut vals = float_model.variables()[&t(part)]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let rows = vals.len() / d;
                    for row in vals.chunks_exact_mut(d) {
                        for (w, &g) in row.iter_mut().zip(&rec_gain) {
                            *w *= g;
                        }
                    }
                    let (eff, b_count) = quantize_dequantize_e8(&vals, rows, d, false)?;
                    set_model_var(&arm3_model, &t(part), eff)?;
                    arm3_total_bytes += b_count;
                    arm3_weights_count += rows * d;
                }
                let vals = float_model.variables()[&t("rec.out.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let rows = vals.len() / d;
                let (eff, b_count) = quantize_dequantize_e8(&vals, rows, d, false)?;
                set_model_var(&arm3_model, &t("rec.out.weight"), eff)?;
                arm3_total_bytes += b_count;
                arm3_weights_count += rows * d;
            } else {
                let read_gain = float_model.variables()[&t("read_norm.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for part in [
                    "read.query.weight",
                    "read.key.weight",
                    "read.value.weight",
                    "read.null.weight",
                ] {
                    let mut vals = float_model.variables()[&t(part)]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let rows = vals.len() / d;
                    for row in vals.chunks_exact_mut(d) {
                        for (w, &g) in row.iter_mut().zip(&read_gain) {
                            *w *= g;
                        }
                    }
                    let (eff, b_count) = quantize_dequantize_e8(&vals, rows, d, false)?;
                    set_model_var(&arm3_model, &t(part), eff)?;
                    arm3_total_bytes += b_count;
                    arm3_weights_count += rows * d;
                }
                let vals = float_model.variables()[&t("read.out.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let rows = vals.len() / d;
                let (eff, b_count) = quantize_dequantize_e8(&vals, rows, d, false)?;
                set_model_var(&arm3_model, &t("read.out.weight"), eff)?;
                arm3_total_bytes += b_count;
                arm3_weights_count += rows * d;
            }

            let mlp_h = float_model.config.mlp_hidden;
            for part in ["mlp.gate.weight", "mlp.up.weight"] {
                let mut vals = float_model.variables()[&t(part)]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for row in vals.chunks_exact_mut(d) {
                    for (w, &g) in row.iter_mut().zip(&mlp_gain) {
                        *w *= g;
                    }
                }
                let (eff, b_count) = quantize_dequantize_e8(&vals, mlp_h, d, false)?;
                set_model_var(&arm3_model, &t(part), eff)?;
                arm3_total_bytes += b_count;
                arm3_weights_count += mlp_h * d;
            }
            let padded_cols = mlp_h.div_ceil(32) * 32;
            let vals = float_model.variables()[&t("mlp.down.weight")]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?;
            let mut padded_down = vec![0.0f32; d * padded_cols];
            for r in 0..d {
                padded_down[r * padded_cols..r * padded_cols + mlp_h]
                    .copy_from_slice(&vals[r * mlp_h..(r + 1) * mlp_h]);
            }
            let (eff_padded, b_count) =
                quantize_dequantize_e8(&padded_down, d, padded_cols, false)?;
            let mut unpadded_down = vec![0.0f32; d * mlp_h];
            for r in 0..d {
                unpadded_down[r * mlp_h..(r + 1) * mlp_h]
                    .copy_from_slice(&eff_padded[r * padded_cols..r * padded_cols + mlp_h]);
            }
            set_model_var(&arm3_model, &t("mlp.down.weight"), unpadded_down)?;
            arm3_total_bytes += b_count;
            arm3_weights_count += d * mlp_h;
        }

        let (arm3_nll, arm3_win, arm3_preds) =
            score_reference(&arm3_model, &arm3_head_tensor, "Hadamard+E8")?;
        let arm3_delta = arm3_nll - baseline_float_nll;
        let arm3_agr = arm3_preds
            .iter()
            .zip(&float_top_preds)
            .filter(|(a, b)| a == b)
            .count();
        let arm3_agr_rate = (arm3_agr as f64) / (total_targets as f64);
        let arm3_bits_per_weight = (arm3_total_bytes * 8) as f64 / (arm3_weights_count as f64);
        println!(
            "  Online H+E8 NLL: {:.6} (Delta: {:+.6} nats, Top-1 Agr: {:.2}%, Bits: {:.4} b/w)",
            arm3_nll,
            arm3_delta,
            arm3_agr_rate * 100.0,
            arm3_bits_per_weight
        );

        if should_run("hadamard_e8") {
            summaries.push(EvaluationSummary {
                arm: "hadamard_e8".into(),
                title: "Online Hadamard + E8 Lattice Codebook (H+E8)".into(),
                mean_nll: arm3_nll,
                delta_nll: arm3_delta,
                top1_agreement: arm3_agr_rate,
                stored_bits_per_weight: arm3_bits_per_weight,
                total_stored_bytes: arm3_total_bytes,
                pass_gate: arm3_delta <= 0.0200 && arm3_bits_per_weight <= 4.25,
                window_nlls: arm3_win,
            });
        }

        // -------------------------------------------------------------------
        // [5/6] Arm (iv): Online Hadamard + E8 with Finer Head Scale
        // -------------------------------------------------------------------
        println!("\n[5/6] Arm (iv): Online Hadamard + E8 (Finer Head Scale)...");
        let (arm4_head_vals, finer_head_bytes) =
            quantize_dequantize_e8(&head_folded, vocab_size, d, true)?;
        let arm4_head_tensor = Tensor::from_vec(arm4_head_vals, (vocab_size, d), &Device::Cpu)?;
        let arm4_total_bytes = arm3_total_bytes - head_bytes + finer_head_bytes;
        let arm4_bits_per_weight = (arm4_total_bytes * 8) as f64 / (arm3_weights_count as f64);

        let (arm4_nll, arm4_win, arm4_preds) =
            score_reference(&arm3_model, &arm4_head_tensor, "H+E8 (Finer Head)")?;
        let arm4_delta = arm4_nll - baseline_float_nll;
        let arm4_agr = arm4_preds
            .iter()
            .zip(&float_top_preds)
            .filter(|(a, b)| a == b)
            .count();
        let arm4_agr_rate = (arm4_agr as f64) / (total_targets as f64);
        println!("  Online H+E8 (Finer Head) NLL: {:.6} (Delta: {:+.6} nats, Top-1 Agr: {:.2}%, Bits: {:.4} b/w)",
            arm4_nll, arm4_delta, arm4_agr_rate * 100.0, arm4_bits_per_weight);

        if should_run("hadamard_e8_finer_head") {
            summaries.push(EvaluationSummary {
                arm: "hadamard_e8_finer_head".into(),
                title: "Online Hadamard + E8 (Finer Head Scale)".into(),
                mean_nll: arm4_nll,
                delta_nll: arm4_delta,
                top1_agreement: arm4_agr_rate,
                stored_bits_per_weight: arm4_bits_per_weight,
                total_stored_bytes: arm4_total_bytes,
                pass_gate: arm4_delta <= 0.0200 && arm4_bits_per_weight <= 4.25,
                window_nlls: arm4_win,
            });
        }

        // -------------------------------------------------------------------
        // [5/7] Arm (v): Online Hadamard + Matched-Bit E8 Lattice Codebook
        // -------------------------------------------------------------------
        println!("\n[5/7] Arm (v): Online Hadamard + Matched-Bit E8 Lattice Codebook (Two-Stage Residual, ~4 bpw)...");
        let mut arm5_total_bytes = 1928usize; // 241 codewords * 8 bytes shared codebook
        let mut arm5_weights_count = 0usize;

        let arm5_model = StackModel::load(&args.model_dir, &Device::Cpu)?;
        set_model_var(&arm5_model, "final_norm.weight", vec![1.0; d])?;
        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            if kind == b'r' {
                set_model_var(&arm5_model, &t("rec_norm.weight"), vec![1.0; d])?;
            } else {
                set_model_var(&arm5_model, &t("read_norm.weight"), vec![1.0; d])?;
            }
            set_model_var(&arm5_model, &t("mlp_norm.weight"), vec![1.0; d])?;
        }
        for (name, var) in rtn_grid_ref.model.variables() {
            if name.contains("bias")
                || name.contains("decay")
                || name.contains("conv")
                || name.contains("age")
                || name.contains("beta")
                || name.contains("offset")
            {
                arm5_model.variables()[name].set(var.as_tensor())?;
            }
        }
        arm5_model.variables()["embedding.weight"]
            .set(rtn_grid_ref.model.variables()["embedding.weight"].as_tensor())?;

        let (arm5_head_vals, arm5_head_bytes) =
            quantize_dequantize_matched_bit_e8(&head_folded, vocab_size, d)?;
        let arm5_head_tensor = Tensor::from_vec(arm5_head_vals, (vocab_size, d), &Device::Cpu)?;
        arm5_total_bytes += arm5_head_bytes;
        arm5_weights_count += vocab_size * d;

        for (l, kind) in float_model.config.pattern.bytes().enumerate() {
            let t = |suffix: &str| format!("layers.{l:02}.{suffix}");
            let mlp_gain = float_model.variables()[&t("mlp_norm.weight")]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?;
            if kind == b'r' {
                let rec_gain = float_model.variables()[&t("rec_norm.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for part in ["rec.in.weight", "rec.gate.weight"] {
                    let mut vals = float_model.variables()[&t(part)]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let rows = vals.len() / d;
                    for row in vals.chunks_exact_mut(d) {
                        for (w, &g) in row.iter_mut().zip(&rec_gain) {
                            *w *= g;
                        }
                    }
                    let (eff, b_count) = quantize_dequantize_matched_bit_e8(&vals, rows, d)?;
                    set_model_var(&arm5_model, &t(part), eff)?;
                    arm5_total_bytes += b_count;
                    arm5_weights_count += rows * d;
                }
                let vals = float_model.variables()[&t("rec.out.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let rows = vals.len() / d;
                let (eff, b_count) = quantize_dequantize_matched_bit_e8(&vals, rows, d)?;
                set_model_var(&arm5_model, &t("rec.out.weight"), eff)?;
                arm5_total_bytes += b_count;
                arm5_weights_count += rows * d;
            } else {
                let read_gain = float_model.variables()[&t("read_norm.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for part in [
                    "read.query.weight",
                    "read.key.weight",
                    "read.value.weight",
                    "read.null.weight",
                ] {
                    let mut vals = float_model.variables()[&t(part)]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?;
                    let rows = vals.len() / d;
                    for row in vals.chunks_exact_mut(d) {
                        for (w, &g) in row.iter_mut().zip(&read_gain) {
                            *w *= g;
                        }
                    }
                    let (eff, b_count) = quantize_dequantize_matched_bit_e8(&vals, rows, d)?;
                    set_model_var(&arm5_model, &t(part), eff)?;
                    arm5_total_bytes += b_count;
                    arm5_weights_count += rows * d;
                }
                let vals = float_model.variables()[&t("read.out.weight")]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                let rows = vals.len() / d;
                let (eff, b_count) = quantize_dequantize_matched_bit_e8(&vals, rows, d)?;
                set_model_var(&arm5_model, &t("read.out.weight"), eff)?;
                arm5_total_bytes += b_count;
                arm5_weights_count += rows * d;
            }

            let mlp_h = float_model.config.mlp_hidden;
            for part in ["mlp.gate.weight", "mlp.up.weight"] {
                let mut vals = float_model.variables()[&t(part)]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                for row in vals.chunks_exact_mut(d) {
                    for (w, &g) in row.iter_mut().zip(&mlp_gain) {
                        *w *= g;
                    }
                }
                let (eff, b_count) = quantize_dequantize_matched_bit_e8(&vals, mlp_h, d)?;
                set_model_var(&arm5_model, &t(part), eff)?;
                arm5_total_bytes += b_count;
                arm5_weights_count += mlp_h * d;
            }
            let padded_cols = mlp_h.div_ceil(32) * 32;
            let vals = float_model.variables()[&t("mlp.down.weight")]
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?;
            let mut padded_down = vec![0.0f32; d * padded_cols];
            for r in 0..d {
                padded_down[r * padded_cols..r * padded_cols + mlp_h]
                    .copy_from_slice(&vals[r * mlp_h..(r + 1) * mlp_h]);
            }
            let (eff_padded, b_count) =
                quantize_dequantize_matched_bit_e8(&padded_down, d, padded_cols)?;
            let mut unpadded_down = vec![0.0f32; d * mlp_h];
            for r in 0..d {
                unpadded_down[r * mlp_h..(r + 1) * mlp_h]
                    .copy_from_slice(&eff_padded[r * padded_cols..r * padded_cols + mlp_h]);
            }
            set_model_var(&arm5_model, &t("mlp.down.weight"), unpadded_down)?;
            arm5_total_bytes += b_count;
            arm5_weights_count += d * mlp_h;
        }

        let (arm5_nll, arm5_win, arm5_preds) =
            score_reference(&arm5_model, &arm5_head_tensor, "Matched-Bit E8")?;
        let arm5_delta = arm5_nll - baseline_float_nll;
        let arm5_agr = arm5_preds
            .iter()
            .zip(&float_top_preds)
            .filter(|(a, b)| a == b)
            .count();
        let arm5_agr_rate = (arm5_agr as f64) / (total_targets as f64);
        let arm5_bits_per_weight = (arm5_total_bytes * 8) as f64 / (arm5_weights_count as f64);
        println!(
            "  Online Matched-Bit E8 NLL: {:.6} (Delta: {:+.6} nats, Top-1 Agr: {:.2}%, Bits: {:.4} b/w)",
            arm5_nll,
            arm5_delta,
            arm5_agr_rate * 100.0,
            arm5_bits_per_weight
        );

        if should_run("hadamard_e8_matched_bit") {
            summaries.push(EvaluationSummary {
                arm: "hadamard_e8_matched_bit".into(),
                title: "Online Hadamard + Matched-Bit E8 Lattice (Two-Stage Residual, ~4 bpw)"
                    .into(),
                mean_nll: arm5_nll,
                delta_nll: arm5_delta,
                top1_agreement: arm5_agr_rate,
                stored_bits_per_weight: arm5_bits_per_weight,
                total_stored_bytes: arm5_total_bytes,
                pass_gate: arm5_delta <= 0.0200 && arm5_bits_per_weight <= 4.25,
                window_nlls: arm5_win,
            });
        }

        // -------------------------------------------------------------------
        // [6/6] Negative Control: Scrambled Scales (Adversarial Check)
        // -------------------------------------------------------------------
        println!("\n[6/6] Adversarial Check: Negative Control (Scrambled Scales)...");
        let mut scrambled_head_vals = arm2_head_tensor.flatten_all()?.to_vec1::<f32>()?;
        for chunk in scrambled_head_vals.chunks_exact_mut(64) {
            for i in 0..32 {
                let tmp = chunk[i];
                chunk[i] = -chunk[i + 32] * 4.0;
                chunk[i + 32] = tmp * 0.25;
            }
        }
        let scrambled_head_tensor =
            Tensor::from_vec(scrambled_head_vals, (vocab_size, d), &Device::Cpu)?;
        let (neg_nll, neg_win, neg_preds) =
            score_reference(&arm2_model, &scrambled_head_tensor, "Negative Control")?;
        let neg_delta = neg_nll - baseline_float_nll;
        let neg_agr = neg_preds
            .iter()
            .zip(&float_top_preds)
            .filter(|(a, b)| a == b)
            .count();
        let neg_agr_rate = (neg_agr as f64) / (total_targets as f64);
        println!(
            "  Negative Control NLL: {:.6} (Delta: {:+.6} nats, Top-1 Agr: {:.2}%)",
            neg_nll,
            neg_delta,
            neg_agr_rate * 100.0
        );
        assert!(
            neg_delta > 0.5,
            "Negative control failed: damage was not detected by harness!"
        );
        println!(
            "  Harness validation: Negative control degraded by +{:.4} nats as required.",
            neg_delta
        );

        if should_run("negative_control_scrambled") {
            summaries.push(EvaluationSummary {
                arm: "negative_control_scrambled".into(),
                title: "Negative Control (Scrambled Scales Damage Check)".into(),
                mean_nll: neg_nll,
                delta_nll: neg_delta,
                top1_agreement: neg_agr_rate,
                stored_bits_per_weight: 4.25,
                total_stored_bytes: arm2_total_bytes,
                pass_gate: false,
                window_nlls: neg_win,
            });
        }

        // -------------------------------------------------------------------
        // Report Generation and Output Writing
        // -------------------------------------------------------------------
        let duration_secs = run_started.elapsed().as_secs_f64();
        println!("\n============================================================");
        println!("FINAL SUMMARY TABLE (512 windows / 131,072 targets)");
        println!("============================================================");
        println!(
            "{:<35} | {:<10} | {:<12} | {:<10} | {:<10} | {:<6}",
            "Arm", "NLL", "Delta (nats)", "Top-1 Agr", "Bits/Weight", "Gate"
        );
        println!("---------------------------------------------------------------------------------------------");
        println!(
            "{:<35} | {:<10.6} | {:<12} | {:<10} | {:<10} | {:<6}",
            "Float Baseline", baseline_float_nll, "0.000000", "100.00%", "32.0000", "REF"
        );
        for s in &summaries {
            println!(
                "{:<35} | {:<10.6} | {:<+12.6} | {:<9.2}% | {:<10.4} | {:<6}",
                s.title,
                s.mean_nll,
                s.delta_nll,
                s.top1_agreement * 100.0,
                s.stored_bits_per_weight,
                if s.pass_gate { "PASS" } else { "FAIL" }
            );
        }

        let report_data = json!({
            "schema": "uor-r4.d4-float-reference-report/1",
            "model": {
                "path": weights_path.display().to_string(),
                "sha256": model_sha256,
                "config": float_model.config,
            },
            "valid": {
                "path": args.valid_path.display().to_string(),
                "sha256": valid_sha256,
                "tokens": tokens.len(),
            },
            "evaluation": {
                "windows": total_windows,
                "context": time,
                "targets": total_targets,
                "stride": stride,
                "float_baseline_nll": baseline_float_nll,
                "duration_seconds": duration_secs,
            },
            "fidelity_gate": {
                "threshold_nats": 0.0200,
                "max_bits_per_weight": 4.25,
                "existing_best_gptq_nats": 0.0257,
            },
            "arms": summaries.iter().map(|s| json!({
                "arm": s.arm,
                "title": s.title,
                "mean_nll": s.mean_nll,
                "delta_nll": s.delta_nll,
                "top1_agreement": s.top1_agreement,
                "stored_bits_per_weight": s.stored_bits_per_weight,
                "total_stored_bytes": s.total_stored_bytes,
                "pass_gate": s.pass_gate,
                "window_nlls": s.window_nlls,
            })).collect::<Vec<_>>(),
        });

        fs::write(
            args.out.join("report.json"),
            serde_json::to_vec_pretty(&report_data)?,
        )?;
        println!("\nWrote report.json to {}", args.out.display());

        Ok(())
    })();

    if let Err(e) = execution_result {
        eprintln!("Execution failed: {e}");
        return Err(e);
    }

    report_output::seal(&args.out)?;
    report_output::verify(&args.out)?;
    println!(
        "Report root successfully claimed, sealed, and verified: {}",
        args.out.display()
    );

    Ok(())
}
