//! B3: E8 Lattice Weight Codec Evaluation on SmolLM2-360M MLP Layers.
//!
//! Evaluates E8 lattice vector quantization at 2, 3, and 4 bits per weight against
//! 4-bit round-to-nearest (RTN) and transform controls on SmolLM2-360M's MLP weights:
//! `gate_proj`, `up_proj`, `down_proj` (235,929,600 total weights across 32 layers).
//!
//! Pre-registered Arms:
//! - Arm 0: Float reference (SmolLM2-360M-Instruct unquantized baseline)
//! - Arm 1: Plain RTN 4-bit (Group size 32: 16 code bytes + 1 scale byte per 32 weights = 4.2500 bpw)
//! - Arm 2: RHT + RTN 4-bit (Randomized Hadamard Transform + RTN 4-bit)
//! - Arm 3: RHT + RTN 3-bit (Randomized Hadamard Transform + RTN 3-bit scalar control)
//! - Arm 4: RHT + E8P 2-bit (Conway–Sloane E8P 2-bit + amortized row scale)
//! - Arm 5: RHT + E8P 3-bit (Conway–Sloane E8P 2-bit + 1-bpw residual stage + amortized row scale)
//! - Arm 6: RHT + E8P 4-bit (Conway–Sloane E8P 2-bit + E8P 2-bit residual RVQ + amortized row scale)
//!
//! Invariants:
//! - Bits per weight derived strictly from serialized bytes: `8 * serialized_bytes / num_weights`.
//! - Codec round trip gate: verified bit-for-bit exact deserialization on every quantized matrix.
//! - Kill criterion: RHT + E8P 3-bit delta NLL > RTN 4-bit delta NLL + 0.05 nats.
//! - Sealed report root via `report_output::{claim, seal, verify}`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use serde_json::json;

use uor_r4_core::report_output;
use uor_r4_training::b3_e8_codecs::{
    RhtE8P2BitMatrix, RhtE8P3BitMatrix, RhtE8P4BitMatrix, RhtRtn3BitMatrix, RhtRtn4BitMatrix,
    Rtn4BitMatrix,
};
use uor_r4_training::kappa_llama::{
    distillation_kl, load_checkpoint, next_token_nll, KappaLlama, ScoreKind, Trainable,
};

type GenericResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn err(msg: impl Into<String>) -> Box<dyn std::error::Error> {
    msg.into().into()
}

#[derive(Clone, Debug)]
struct CliArgs {
    model_dir: PathBuf,
    tokens_path: PathBuf,
    windows: usize,
    time: usize,
    batch: usize,
    out: PathBuf,
    layers: usize,
}

fn parse_cli_args() -> GenericResult<CliArgs> {
    let mut model_dir =
        PathBuf::from("/Volumes/UOR-Workspace/uor-r4-models/sources/smollm2-360m-instruct");
    let mut tokens_path = PathBuf::from(
        "/Volumes/UOR-Workspace/uor-r4-models/corpora/simple-wiki-20231101/simplewiki_32x1024.u16",
    );
    let mut windows = 32;
    let mut time = 1024;
    let mut batch = 1;
    let mut out = PathBuf::from("/Volumes/UOR-Workspace/uor-r4-lab/b3-e8-smollm2-mlp/attempt-1");
    let mut layers = 32;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if let Some(val) = arg.strip_prefix("--model=") {
            model_dir = PathBuf::from(val);
        } else if let Some(val) = arg.strip_prefix("--tokens=") {
            tokens_path = PathBuf::from(val);
        } else if let Some(val) = arg.strip_prefix("--windows=") {
            windows = val.parse()?;
        } else if let Some(val) = arg.strip_prefix("--time=") {
            time = val.parse()?;
        } else if let Some(val) = arg.strip_prefix("--batch=") {
            batch = val.parse()?;
        } else if let Some(val) = arg.strip_prefix("--out=") {
            out = PathBuf::from(val);
        } else if let Some(val) = arg.strip_prefix("--layers=") {
            layers = val.parse()?;
        } else if arg == "--model" && i + 1 < args.len() {
            model_dir = PathBuf::from(&args[i + 1]);
            i += 1;
        } else if arg == "--tokens" && i + 1 < args.len() {
            tokens_path = PathBuf::from(&args[i + 1]);
            i += 1;
        } else if arg == "--windows" && i + 1 < args.len() {
            windows = args[i + 1].parse()?;
            i += 1;
        } else if arg == "--time" && i + 1 < args.len() {
            time = args[i + 1].parse()?;
            i += 1;
        } else if arg == "--batch" && i + 1 < args.len() {
            batch = args[i + 1].parse()?;
            i += 1;
        } else if arg == "--out" && i + 1 < args.len() {
            out = PathBuf::from(&args[i + 1]);
            i += 1;
        } else if arg == "--layers" && i + 1 < args.len() {
            layers = args[i + 1].parse()?;
            i += 1;
        } else if arg == "--help" || arg == "-h" {
            println!("Usage: b3-e8-smollm2 [options]");
            println!("  --model=PATH    SmolLM2 checkpoint directory");
            println!("  --tokens=PATH   Prepared .u16 token file");
            println!("  --windows=N     Evaluation windows (default: 32)");
            println!("  --time=N        Window length (default: 1024)");
            println!("  --batch=N       Batch size (default: 1)");
            println!("  --layers=N      Number of layers to quantize (default: 32)");
            println!("  --out=PATH      Report output directory");
            std::process::exit(0);
        } else {
            return Err(err(format!("unrecognized argument: {arg}")));
        }
        i += 1;
    }

    Ok(CliArgs {
        model_dir,
        tokens_path,
        windows,
        time,
        batch,
        out,
        layers,
    })
}

fn read_u16_tokens(path: &Path, vocab: usize) -> GenericResult<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.len() % 2 != 0 {
        return Err(err(format!("odd token file length {}", bytes.len())));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|chunk| u32::from(u16::from_le_bytes([chunk[0], chunk[1]])))
        .collect();
    if tokens.iter().any(|&t| t as usize >= vocab) {
        return Err(err(format!(
            "{} has a token outside the vocabulary ({vocab})",
            path.display()
        )));
    }
    Ok(tokens)
}

fn evenly_spaced(len: usize, time: usize, count: usize) -> GenericResult<Vec<usize>> {
    if len < time + 2 || count == 0 {
        return Err(err("token file shorter than one window"));
    }
    let span = len - time - 1;
    Ok((0..count).map(|i| i * span / count).collect())
}

type Chunk = (Vec<u32>, Vec<u32>, usize);

fn build_chunks(tokens: &[u32], starts: &[usize], time: usize, batch: usize) -> Vec<Chunk> {
    starts
        .chunks(batch)
        .map(|chunk| {
            let mut inputs = Vec::with_capacity(chunk.len() * time);
            let mut targets = Vec::with_capacity(chunk.len() * time);
            for &start in chunk {
                inputs.extend_from_slice(&tokens[start..start + time]);
                targets.extend_from_slice(&tokens[start + 1..start + time + 1]);
            }
            (inputs, targets, chunk.len())
        })
        .collect()
}

fn compute_top1_agreement(logits: &Tensor, ref_logits: &Tensor) -> GenericResult<f64> {
    let (b, t, v) = logits.dims3()?;
    let pred = logits.reshape((b * t, v))?.argmax(1)?;
    let ref_pred = ref_logits.reshape((b * t, v))?.argmax(1)?;
    let pred_v: Vec<u32> = pred.to_vec1()?;
    let ref_v: Vec<u32> = ref_pred.to_vec1()?;
    let matches = pred_v
        .iter()
        .zip(ref_v.iter())
        .filter(|(&a, &b)| a == b)
        .count();
    Ok((matches as f64) / (pred_v.len() as f64))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ArmMetrics {
    arm_id: usize,
    name: String,
    title: String,
    target_bpw: f64,
    payload_bpw: f64,
    total_serialized_bytes: usize,
    quantized_weights_count: usize,
    mean_nll: f64,
    delta_nll: f64,
    mean_kl: f64,
    top1_agreement_pct: f64,
    window_nlls: Vec<f64>,
    codec_roundtrip_gate_passed: Option<bool>,
}

fn main() -> GenericResult<()> {
    let args = parse_cli_args()?;
    println!("=== B3: E8 Lattice Weight Coding & Incoherence Transform on SmolLM2-360M ===");
    println!("Model:      {}", args.model_dir.display());
    println!("Tokens:     {}", args.tokens_path.display());
    println!(
        "Evaluation: {} windows x {} tokens (batch {})",
        args.windows, args.time, args.batch
    );
    println!("Layers:     {} / 32", args.layers);
    println!("Output:     {}", args.out.display());

    // 1. Claim output directory
    report_output::claim(&args.out)?;
    println!("Claimed report root: {}", args.out.display());

    // 2. Setup Device
    let device = Device::Cpu;

    // 3. Load SmolLM2-360M Checkpoint
    println!("\nLoading SmolLM2-360M checkpoint...");
    let checkpoint = load_checkpoint(&args.model_dir, &device)?;
    println!(
        "Loaded checkpoint (layers: {}, width: {}, ffn: {})",
        checkpoint.shape.layers, checkpoint.shape.width, checkpoint.shape.ffn
    );

    // 4. Load Evaluation Tokens
    let tokens = read_u16_tokens(&args.tokens_path, checkpoint.shape.vocab)?;
    println!(
        "Loaded {} tokens from {}",
        tokens.len(),
        args.tokens_path.display()
    );
    let starts = evenly_spaced(tokens.len(), args.time, args.windows)?;
    let chunks = build_chunks(&tokens, &starts, args.time, args.batch);

    // 5. Build Initial Float Model
    let mut model = KappaLlama::new(
        checkpoint.clone(),
        ScoreKind::Dot,
        -10.0,
        Trainable::Scalars,
        &device,
    )?;

    // -----------------------------------------------------------------------
    // Arm 0: Float Reference Baseline
    // -----------------------------------------------------------------------
    println!("\n[Arm 0] Evaluating Float Reference Baseline...");
    let float_start = Instant::now();
    let ref_temp_dir = std::env::temp_dir().join(format!("b3_ref_logits_{}", std::process::id()));
    fs::create_dir_all(&ref_temp_dir)?;

    let mut ref_nlls = Vec::with_capacity(chunks.len());

    for (i, (inputs, targets, n)) in chunks.iter().enumerate() {
        let logits = model.forward(inputs, *n, args.time, true)?;
        let loss = next_token_nll(&logits, targets)?;
        ref_nlls.push(f64::from(loss.to_scalar::<f32>()?));
        let ref_path = ref_temp_dir.join(format!("window_{i}.safetensors"));
        logits.save_safetensors("logits", &ref_path)?;
    }

    let float_mean_nll = ref_nlls.iter().sum::<f64>() / (ref_nlls.len() as f64);
    println!(
        "  Float Reference NLL: {:.6} (computed in {:.2}s)",
        float_mean_nll,
        float_start.elapsed().as_secs_f64()
    );

    let mlp_projections = ["gate_proj", "up_proj", "down_proj"];
    let num_mlp_weights_per_layer = 3 * (checkpoint.shape.width * checkpoint.shape.ffn);
    let total_mlp_weights = args.layers * num_mlp_weights_per_layer;

    let arm0 = ArmMetrics {
        arm_id: 0,
        name: "float".to_string(),
        title: "Float Reference Baseline (SmolLM2-360M unquantized)".to_string(),
        target_bpw: 32.0,
        payload_bpw: 32.0,
        total_serialized_bytes: total_mlp_weights * 4,
        quantized_weights_count: total_mlp_weights,
        mean_nll: float_mean_nll,
        delta_nll: 0.0,
        mean_kl: 0.0,
        top1_agreement_pct: 100.0,
        window_nlls: ref_nlls.clone(),
        codec_roundtrip_gate_passed: None,
    };

    // Helper closure to evaluate an arm
    let evaluate_current_model = |model: &KappaLlama| -> GenericResult<(f64, f64, f64, Vec<f64>)> {
        let mut nlls = Vec::with_capacity(chunks.len());
        let mut kls = Vec::with_capacity(chunks.len());
        let mut agreements = Vec::with_capacity(chunks.len());

        for (i, (inputs, targets, n)) in chunks.iter().enumerate() {
            let logits = model.forward(inputs, *n, args.time, true)?;
            let loss = next_token_nll(&logits, targets)?;
            nlls.push(f64::from(loss.to_scalar::<f32>()?));

            let ref_path = ref_temp_dir.join(format!("window_{i}.safetensors"));
            let ref_map = candle_core::safetensors::load(&ref_path, &device)?;
            let ref_logits = ref_map
                .get("logits")
                .ok_or_else(|| err("missing ref logits"))?;

            let kl_tensor = distillation_kl(&logits, ref_logits)?;
            kls.push(f64::from(kl_tensor.to_scalar::<f32>()?));

            let agr = compute_top1_agreement(&logits, ref_logits)?;
            agreements.push(agr);
        }

        let mean_nll = nlls.iter().sum::<f64>() / (nlls.len() as f64);
        let mean_kl = kls.iter().sum::<f64>() / (kls.len() as f64);
        let mean_agr = agreements.iter().sum::<f64>() / (agreements.len() as f64) * 100.0;
        Ok((mean_nll, mean_kl, mean_agr, nlls))
    };

    let seeds = (12345u64, 67890u64);

    // -----------------------------------------------------------------------
    // Arm 1: Plain RTN 4-bit (Group size 32, ~4.25 bpw)
    // -----------------------------------------------------------------------
    println!("\n[Arm 1] Quantizing MLP layers with Plain RTN 4-bit (group size 32)...");
    let rtn_start = Instant::now();
    let mut rtn_bytes = 0usize;
    let mut rtn_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint
                .tensors
                .get(&name)
                .ok_or_else(|| err(format!("tensor {name} not found")))?;
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = Rtn4BitMatrix::encode(&weights, rows, cols)?;
            let bytes = matrix.to_bytes();
            rtn_bytes += bytes.len();

            let deserialized = Rtn4BitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                rtn_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let rtn_bpw = (rtn_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  RTN 4-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        rtn_start.elapsed().as_secs_f64(),
        rtn_bytes,
        rtn_bpw,
        if rtn_roundtrip_ok { "PASSED" } else { "FAILED" }
    );
    let (rtn_nll, rtn_kl, rtn_agr, rtn_nlls) = evaluate_current_model(&model)?;
    let rtn_delta = rtn_nll - float_mean_nll;
    println!(
        "  RTN 4-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        rtn_nll, rtn_delta, rtn_kl, rtn_agr
    );

    let arm1 = ArmMetrics {
        arm_id: 1,
        name: "rtn_4bit".to_string(),
        title: "Plain Round-to-Nearest 4-bit (Group size 32)".to_string(),
        target_bpw: 4.25,
        payload_bpw: rtn_bpw,
        total_serialized_bytes: rtn_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: rtn_nll,
        delta_nll: rtn_delta,
        mean_kl: rtn_kl,
        top1_agreement_pct: rtn_agr,
        window_nlls: rtn_nlls,
        codec_roundtrip_gate_passed: Some(rtn_roundtrip_ok),
    };

    // -----------------------------------------------------------------------
    // Arm 2: RHT + RTN 4-bit
    // -----------------------------------------------------------------------
    println!("\n[Arm 2] Quantizing MLP layers with RHT + RTN 4-bit...");
    let rht_rtn4_start = Instant::now();
    let mut rht_rtn4_bytes = 0usize;
    let mut rht_rtn4_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint.tensors.get(&name).unwrap();
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = RhtRtn4BitMatrix::encode(&weights, rows, cols, seeds.0, seeds.1)?;
            let bytes = matrix.to_bytes();
            rht_rtn4_bytes += bytes.len();

            let deserialized = RhtRtn4BitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                rht_rtn4_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let rht_rtn4_bpw = (rht_rtn4_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  RHT + RTN 4-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        rht_rtn4_start.elapsed().as_secs_f64(),
        rht_rtn4_bytes,
        rht_rtn4_bpw,
        if rht_rtn4_roundtrip_ok {
            "PASSED"
        } else {
            "FAILED"
        }
    );
    let (rht_rtn4_nll, rht_rtn4_kl, rht_rtn4_agr, rht_rtn4_nlls) = evaluate_current_model(&model)?;
    let rht_rtn4_delta = rht_rtn4_nll - float_mean_nll;
    println!(
        "  RHT + RTN 4-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        rht_rtn4_nll, rht_rtn4_delta, rht_rtn4_kl, rht_rtn4_agr
    );

    let arm2 = ArmMetrics {
        arm_id: 2,
        name: "rht_rtn_4bit".to_string(),
        title: "RHT + RTN 4-bit (Incoherence-transformed scalar)".to_string(),
        target_bpw: 4.25,
        payload_bpw: rht_rtn4_bpw,
        total_serialized_bytes: rht_rtn4_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: rht_rtn4_nll,
        delta_nll: rht_rtn4_delta,
        mean_kl: rht_rtn4_kl,
        top1_agreement_pct: rht_rtn4_agr,
        window_nlls: rht_rtn4_nlls,
        codec_roundtrip_gate_passed: Some(rht_rtn4_roundtrip_ok),
    };

    // -----------------------------------------------------------------------
    // Arm 3: RHT + RTN 3-bit (Scalar Control at 3 bpw)
    // -----------------------------------------------------------------------
    println!("\n[Arm 3] Quantizing MLP layers with RHT + RTN 3-bit (scalar control)...");
    let rht_rtn3_start = Instant::now();
    let mut rht_rtn3_bytes = 0usize;
    let mut rht_rtn3_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint.tensors.get(&name).unwrap();
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = RhtRtn3BitMatrix::encode(&weights, rows, cols, seeds.0, seeds.1)?;
            let bytes = matrix.to_bytes();
            rht_rtn3_bytes += bytes.len();

            let deserialized = RhtRtn3BitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                rht_rtn3_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let rht_rtn3_bpw = (rht_rtn3_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  RHT + RTN 3-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        rht_rtn3_start.elapsed().as_secs_f64(),
        rht_rtn3_bytes,
        rht_rtn3_bpw,
        if rht_rtn3_roundtrip_ok {
            "PASSED"
        } else {
            "FAILED"
        }
    );
    let (rht_rtn3_nll, rht_rtn3_kl, rht_rtn3_agr, rht_rtn3_nlls) = evaluate_current_model(&model)?;
    let rht_rtn3_delta = rht_rtn3_nll - float_mean_nll;
    println!(
        "  RHT + RTN 3-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        rht_rtn3_nll, rht_rtn3_delta, rht_rtn3_kl, rht_rtn3_agr
    );

    let arm3 = ArmMetrics {
        arm_id: 3,
        name: "rht_rtn_3bit".to_string(),
        title: "RHT + RTN 3-bit (Incoherence-transformed 3-bit scalar control)".to_string(),
        target_bpw: 3.0,
        payload_bpw: rht_rtn3_bpw,
        total_serialized_bytes: rht_rtn3_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: rht_rtn3_nll,
        delta_nll: rht_rtn3_delta,
        mean_kl: rht_rtn3_kl,
        top1_agreement_pct: rht_rtn3_agr,
        window_nlls: rht_rtn3_nlls,
        codec_roundtrip_gate_passed: Some(rht_rtn3_roundtrip_ok),
    };

    // -----------------------------------------------------------------------
    // Arm 4: RHT + E8P 2-bit
    // -----------------------------------------------------------------------
    println!("\n[Arm 4] Quantizing MLP layers with RHT + E8P 2-bit (Conway–Sloane lattice)...");
    let e8p_2bit_start = Instant::now();
    let mut e8p_2bit_bytes = 0usize;
    let mut e8p_2bit_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint.tensors.get(&name).unwrap();
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = RhtE8P2BitMatrix::encode(&weights, rows, cols, seeds.0, seeds.1)?;
            let bytes = matrix.to_bytes();
            e8p_2bit_bytes += bytes.len();

            let deserialized = RhtE8P2BitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                e8p_2bit_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let e8p_2bit_bpw = (e8p_2bit_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  RHT + E8P 2-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        e8p_2bit_start.elapsed().as_secs_f64(),
        e8p_2bit_bytes,
        e8p_2bit_bpw,
        if e8p_2bit_roundtrip_ok {
            "PASSED"
        } else {
            "FAILED"
        }
    );
    let (e8p_2bit_nll, e8p_2bit_kl, e8p_2bit_agr, e8p_2bit_nlls) = evaluate_current_model(&model)?;
    let e8p_2bit_delta = e8p_2bit_nll - float_mean_nll;
    println!(
        "  RHT + E8P 2-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        e8p_2bit_nll, e8p_2bit_delta, e8p_2bit_kl, e8p_2bit_agr
    );

    let arm4 = ArmMetrics {
        arm_id: 4,
        name: "rht_e8p_2bit".to_string(),
        title: "RHT + E8P 2-bit (Conway–Sloane lattice, 2 bpw)".to_string(),
        target_bpw: 2.0,
        payload_bpw: e8p_2bit_bpw,
        total_serialized_bytes: e8p_2bit_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: e8p_2bit_nll,
        delta_nll: e8p_2bit_delta,
        mean_kl: e8p_2bit_kl,
        top1_agreement_pct: e8p_2bit_agr,
        window_nlls: e8p_2bit_nlls,
        codec_roundtrip_gate_passed: Some(e8p_2bit_roundtrip_ok),
    };

    // -----------------------------------------------------------------------
    // Arm 5: RHT + E8P 3-bit (E8P + 1 bpw residual)
    // -----------------------------------------------------------------------
    println!("\n[Arm 5] Quantizing MLP layers with RHT + E8P 3-bit (E8P + 1-bpw residual)...");
    let e8p_3bit_start = Instant::now();
    let mut e8p_3bit_bytes = 0usize;
    let mut e8p_3bit_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint.tensors.get(&name).unwrap();
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = RhtE8P3BitMatrix::encode(&weights, rows, cols, seeds.0, seeds.1)?;
            let bytes = matrix.to_bytes();
            e8p_3bit_bytes += bytes.len();

            let deserialized = RhtE8P3BitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                e8p_3bit_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let e8p_3bit_bpw = (e8p_3bit_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  RHT + E8P 3-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        e8p_3bit_start.elapsed().as_secs_f64(),
        e8p_3bit_bytes,
        e8p_3bit_bpw,
        if e8p_3bit_roundtrip_ok {
            "PASSED"
        } else {
            "FAILED"
        }
    );
    let (e8p_3bit_nll, e8p_3bit_kl, e8p_3bit_agr, e8p_3bit_nlls) = evaluate_current_model(&model)?;
    let e8p_3bit_delta = e8p_3bit_nll - float_mean_nll;
    println!(
        "  RHT + E8P 3-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        e8p_3bit_nll, e8p_3bit_delta, e8p_3bit_kl, e8p_3bit_agr
    );

    let arm5 = ArmMetrics {
        arm_id: 5,
        name: "rht_e8p_3bit".to_string(),
        title: "RHT + E8P 3-bit (E8P 2-bit + 1-bpw residual stage)".to_string(),
        target_bpw: 3.0,
        payload_bpw: e8p_3bit_bpw,
        total_serialized_bytes: e8p_3bit_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: e8p_3bit_nll,
        delta_nll: e8p_3bit_delta,
        mean_kl: e8p_3bit_kl,
        top1_agreement_pct: e8p_3bit_agr,
        window_nlls: e8p_3bit_nlls,
        codec_roundtrip_gate_passed: Some(e8p_3bit_roundtrip_ok),
    };

    // -----------------------------------------------------------------------
    // Arm 6: RHT + E8P 4-bit (E8P + E8P residual RVQ)
    // -----------------------------------------------------------------------
    println!("\n[Arm 6] Quantizing MLP layers with RHT + E8P 4-bit (E8P + E8P RVQ)...");
    let e8p_4bit_start = Instant::now();
    let mut e8p_4bit_bytes = 0usize;
    let mut e8p_4bit_roundtrip_ok = true;

    for l in 0..args.layers {
        for proj in &mlp_projections {
            let name = format!("model.layers.{l}.mlp.{proj}.weight");
            let orig_tensor = checkpoint.tensors.get(&name).unwrap();
            let dims = orig_tensor.dims();
            let (rows, cols) = (dims[0], dims[1]);
            let weights: Vec<f32> = orig_tensor.flatten_all()?.to_vec1()?;

            let matrix = RhtE8P4BitMatrix::encode(&weights, rows, cols, seeds.0, seeds.1)?;
            let bytes = matrix.to_bytes();
            e8p_4bit_bytes += bytes.len();

            let deserialized = RhtE8P4BitMatrix::from_bytes(&bytes)?;
            if matrix != deserialized {
                e8p_4bit_roundtrip_ok = false;
            }

            let deq = deserialized.dequantize()?;
            let new_tensor = Tensor::from_vec(deq, (rows, cols), &device)?;
            model.set_tensor(&name, new_tensor)?;
        }
    }

    let e8p_4bit_bpw = (e8p_4bit_bytes as f64 * 8.0) / (total_mlp_weights as f64);
    println!(
        "  RHT + E8P 4-bit encoded in {:.2}s (bytes: {}, bpw: {:.4}, roundtrip gate: {})",
        e8p_4bit_start.elapsed().as_secs_f64(),
        e8p_4bit_bytes,
        e8p_4bit_bpw,
        if e8p_4bit_roundtrip_ok {
            "PASSED"
        } else {
            "FAILED"
        }
    );
    let (e8p_4bit_nll, e8p_4bit_kl, e8p_4bit_agr, e8p_4bit_nlls) = evaluate_current_model(&model)?;
    let e8p_4bit_delta = e8p_4bit_nll - float_mean_nll;
    println!(
        "  RHT + E8P 4-bit NLL: {:.6} (Delta: {:+.6} nats, KL: {:.6}, Agr: {:.2}%)",
        e8p_4bit_nll, e8p_4bit_delta, e8p_4bit_kl, e8p_4bit_agr
    );

    let arm6 = ArmMetrics {
        arm_id: 6,
        name: "rht_e8p_4bit".to_string(),
        title: "RHT + E8P 4-bit (E8P 2-bit + E8P 2-bit RVQ)".to_string(),
        target_bpw: 4.0,
        payload_bpw: e8p_4bit_bpw,
        total_serialized_bytes: e8p_4bit_bytes,
        quantized_weights_count: total_mlp_weights,
        mean_nll: e8p_4bit_nll,
        delta_nll: e8p_4bit_delta,
        mean_kl: e8p_4bit_kl,
        top1_agreement_pct: e8p_4bit_agr,
        window_nlls: e8p_4bit_nlls,
        codec_roundtrip_gate_passed: Some(e8p_4bit_roundtrip_ok),
    };

    // -----------------------------------------------------------------------
    // Verdicts & Diagnostics
    // -----------------------------------------------------------------------
    let arms = vec![
        arm0.clone(),
        arm1.clone(),
        arm2.clone(),
        arm3.clone(),
        arm4.clone(),
        arm5.clone(),
        arm6.clone(),
    ];

    let excess_over_rtn4 = arm5.delta_nll - arm1.delta_nll;
    let kill_triggered = excess_over_rtn4 > 0.05;
    let lattice_gain_over_scalar3 = arm3.delta_nll - arm5.delta_nll;
    let transform_gain_at_4bit = arm1.delta_nll - arm2.delta_nll;

    println!("\n=======================================================");
    println!("B3 SUMMARY & VERDICT");
    println!("=======================================================");
    println!("Arm 0 (Float Ref):       NLL = {:.6}", arm0.mean_nll);
    println!(
        "Arm 1 (Plain RTN 4-bit): NLL = {:.6} (Delta = {:+.6} nats, bpw = {:.4})",
        arm1.mean_nll, arm1.delta_nll, arm1.payload_bpw
    );
    println!(
        "Arm 2 (RHT + RTN 4-bit): NLL = {:.6} (Delta = {:+.6} nats, bpw = {:.4})",
        arm2.mean_nll, arm2.delta_nll, arm2.payload_bpw
    );
    println!(
        "Arm 3 (RHT + RTN 3-bit): NLL = {:.6} (Delta = {:+.6} nats, bpw = {:.4})",
        arm3.mean_nll, arm3.delta_nll, arm3.payload_bpw
    );
    println!(
        "Arm 4 (RHT + E8P 2-bit): NLL = {:.6} (Delta = {:+.6} nats, bpw = {:.4})",
        arm4.mean_nll, arm4.delta_nll, arm4.payload_bpw
    );
    println!(
        "Arm 5 (RHT + E8P 3-bit): NLL = {:.6} (Delta = {:+.6} nats, bpw = {:.4})",
        arm5.mean_nll, arm5.delta_nll, arm5.payload_bpw
    );
    println!(
        "Arm 6 (RHT + E8P 4-bit): NLL = {:.6} (Delta = {:+.6} nats, bpw = {:.4})",
        arm6.mean_nll, arm6.delta_nll, arm6.payload_bpw
    );
    println!("-------------------------------------------------------");
    println!(
        "Transform effect at 4-bit (RTN4 - RHT_RTN4):       {:+.6} nats",
        transform_gain_at_4bit
    );
    println!(
        "Lattice gain at 3-bit (RHT_RTN3 - RHT_E8P3):       {:+.6} nats",
        lattice_gain_over_scalar3
    );
    println!(
        "Excess degradation vs RTN 4-bit (E8P3 - RTN4):    {:+.6} nats",
        excess_over_rtn4
    );
    println!("Kill threshold:                                    +0.050000 nats");
    println!(
        "Verdict: {}",
        if kill_triggered {
            format!(
                "KILL CRITERION TRIGGERED (+{:.6} > 0.05 nats)",
                excess_over_rtn4
            )
        } else {
            format!(
                "KILL CRITERION CLEARED (+{:.6} <= 0.05 nats)",
                excess_over_rtn4
            )
        }
    );
    println!("=======================================================");

    // -----------------------------------------------------------------------
    // Write Sealed Report Root
    // -----------------------------------------------------------------------
    let report_json = json!({
        "directive": "B3",
        "description": "E8P Lattice Weight Coding with Incoherence Transform on SmolLM2-360M MLP Layers",
        "corpus": {
            "path": "/Volumes/UOR-Workspace/uor-r4-models/corpora/simple-wiki-20231101/articles.jsonl",
            "cid": "blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf",
            "sha256": "19e39820b4b367aca30ad8d1b2314727c858f6a0d4595fd1f0a68e505f654b01",
            "binding_status": "declared-not-verified"
        },
        "tokenizer": {
            "path": "/Volumes/UOR-Workspace/uor-r4-models/sources/smollm2-360m-instruct/tokenizer.json",
            "sha256": "9ca9acddb6525a194ec8ac7a87f24fbba7232a9a15ffa1af0c1224fcd888e47c",
            "binding_status": "declared-not-verified"
        },
        "evaluation_tokens": {
            "path": args.tokens_path.to_string_lossy(),
            "sha256": "9bf6a8334cfe86d0e07ceceb8436002c6c0671a12f79f2e356b233c7a63f2bbd",
            "binding_status": "declared-not-verified",
            "windows": args.windows,
            "window_len": args.time,
            "total_tokens": args.windows * args.time
        },
        "execution_provenance_note": "source/executable binding unavailable (historical run)",
        "model_architecture": {
            "layers": checkpoint.shape.layers,
            "quantized_layers": args.layers,
            "hidden_size": checkpoint.shape.width,
            "intermediate_size": checkpoint.shape.ffn,
            "vocab_size": checkpoint.shape.vocab,
            "total_mlp_weights": total_mlp_weights
        },
        "kill_criterion": {
            "rule": "RHT + E8P 3-bit delta NLL > RTN 4-bit delta NLL + 0.05 nats",
            "excess_nats": excess_over_rtn4,
            "threshold_nats": 0.05,
            "triggered": kill_triggered
        },
        "diagnostics": {
            "transform_gain_at_4bit": transform_gain_at_4bit,
            "lattice_gain_at_3bit": lattice_gain_over_scalar3
        },
        "arms": arms
    });

    fs::write(
        args.out.join("report.json"),
        serde_json::to_string_pretty(&report_json)?,
    )?;
    fs::write(args.out.join("summary.txt"), format!(
        "B3 Evaluation Summary\nExcess vs RTN 4-bit: {:+.6} nats\nLattice gain vs Scalar 3-bit: {:+.6} nats\nKill triggered: {}\n",
        excess_over_rtn4, lattice_gain_over_scalar3, kill_triggered
    ))?;

    let _ = fs::remove_dir_all(&ref_temp_dir);

    report_output::seal(&args.out)?;
    report_output::verify(&args.out)?;
    println!("\nSealed and verified report root: {}", args.out.display());
    println!(
        "Report JSON written to: {}",
        args.out.join("report.json").display()
    );

    Ok(())
}
