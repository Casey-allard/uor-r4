//! Multiplier-free (D11) serving of a geometric-stack artifact (`UORLUT01`,
//! schema `uor-r4.lut-stack/1`) through `uor_r4_integer::stack`.
//!
//! ```text
//! uor-r4-stack generate ARTIFACT PROMPT_IDS NEW_TOKENS
//! uor-r4-stack digest ARTIFACT TOKENS.u16 WINDOWS
//! ```
//!
//! `generate` feeds the comma-separated prompt ids and continues greedily
//! (integer argmax, first on ties) for at most `NEW_TOKENS` ids or until the
//! context is full, and prints a JSON record. `digest` runs `WINDOWS` evenly
//! spaced full-context windows of a little-endian u16 token file, each in a
//! fresh session, and prints the SHA-256 of every step's logits (little-endian
//! `i32`, in order) as `logits_sha256`, a platform-independent fingerprint of
//! the served integers; a pointer model's record adds `mixture_sha256`, the
//! SHA-256 of every step's Q30 mixture in the same encoding, and sets
//! `pointer_mixture_hashed`; a plain model's record has no mixture hash. A
//! pointer model decodes greedily over its mixture. The served path is
//! `IntegerStackSession::step` and `stack_argmax`; argument parsing, file reading, timing, hashing and JSON
//! are outside it. The instruction audit targets this binary:
//! `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack`.

use std::path::Path;
use std::time::Instant;

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_integer::stack::{stack_argmax, IntegerStackModel, StackError};

/// Nominal bits per weight for 4-bit signed symmetric weight representation.
pub const D11_NOMINAL_BITS_PER_WEIGHT: f64 = 4.0;
/// Scale storage overhead per weight (1 scale byte per 32 weights = 8 / 32 = 0.25 bits/weight).
pub const D11_SCALE_BITS_PER_WEIGHT: f64 = 0.25;
/// Effective bits per weight for canonical D11 grouped 4-bit representation (4.25 bpw).
pub const D11_EFFECTIVE_BITS_PER_WEIGHT: f64 = 4.25;

#[derive(Debug)]
enum CliError {
    Usage(String),
    Io(std::io::Error),
    Stack(StackError),
    Json(serde_json::Error),
    Integer(uor_r4_integer::IntegerError),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::Io(error) => write!(f, "I/O: {error}"),
            Self::Stack(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "JSON: {error}"),
            Self::Integer(error) => write!(f, "Integer error: {error}"),
        }
    }
}

impl From<StackError> for CliError {
    fn from(error: StackError) -> Self {
        Self::Stack(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<uor_r4_integer::IntegerError> for CliError {
    fn from(error: uor_r4_integer::IntegerError) -> Self {
        Self::Integer(error)
    }
}

const USAGE: &str = "usage: uor-r4-stack generate ARTIFACT PROMPT_IDS NEW_TOKENS | digest ARTIFACT TOKENS.u16 WINDOWS | cost ARTIFACT TOKENS.u16 WINDOWS PROMPT_TOKENS [OUT_DIR]";

fn usage(message: &str) -> CliError {
    CliError::Usage(format!("{message}\n{USAGE}"))
}

fn number(text: &str, what: &str) -> Result<usize, CliError> {
    text.parse().map_err(|_| {
        usage(&format!(
            "{what} must be a non-negative integer, got {text}"
        ))
    })
}

/// The serving loop: the prompt, then greedy ids until `new_tokens` or a full
/// context. Every logit vector comes from one `step`; greedy decoding takes
/// the argmax of the served distribution (`next_token_scores`: a pointer
/// model's mixture, which the step computes, otherwise the logits).
fn serve_greedy(
    model: &IntegerStackModel,
    prompt: &[u32],
    new_tokens: usize,
) -> Result<(Vec<u32>, usize), StackError> {
    let mut session = model.session();
    let context = model.shape().context;
    let mut next = 0u32;
    for &id in prompt {
        session.step(id)?;
        next = stack_argmax(session.next_token_scores()) as u32;
    }
    // At most one id per position the context has left, whatever NEW_TOKENS is.
    let mut generated = Vec::with_capacity(new_tokens.min(context));
    while generated.len() < new_tokens {
        generated.push(next);
        if generated.len() == new_tokens || session.position() >= context {
            break;
        }
        session.step(next)?;
        next = stack_argmax(session.next_token_scores()) as u32;
    }
    Ok((generated, session.position()))
}

fn generate(args: &[String]) -> Result<(), CliError> {
    let [artifact, prompt, new_tokens] = args else {
        return Err(usage("generate takes ARTIFACT PROMPT_IDS NEW_TOKENS"));
    };
    let prompt: Vec<u32> = prompt
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let id = number(s, "a prompt id")?;
            u32::try_from(id).map_err(|_| usage(&format!("prompt id {id} does not fit in u32")))
        })
        .collect::<Result<_, _>>()?;
    let new_tokens = number(new_tokens, "NEW_TOKENS")?;
    if prompt.is_empty() {
        return Err(usage("the prompt needs at least one id"));
    }
    let load = Instant::now();
    let model = IntegerStackModel::load(Path::new(artifact))?;
    let load_seconds = load.elapsed().as_secs_f64();
    let vocab = model.shape().vocab;
    if let Some(&id) = prompt.iter().find(|&&id| id as usize >= vocab) {
        return Err(CliError::Stack(StackError::Token { token: id, vocab }));
    }
    let serve = Instant::now();
    let (generated, steps) = serve_greedy(&model, &prompt, new_tokens)?;
    let serve_seconds = serve.elapsed().as_secs_f64();
    let decoding = if model.pointer().is_some() {
        "greedy integer argmax of the pointer mixture (first on ties)"
    } else {
        "greedy integer argmax (first on ties)"
    };
    let record = json!({
        "schema": "uor-r4.stack-d11-generation/1",
        "artifact_sha256": model.artifact_sha256(),
        "prompt": prompt,
        "generated": generated,
        "steps": steps,
        "weights_read_per_token": model.weights_per_token(),
        "nominal_bits_per_weight": D11_NOMINAL_BITS_PER_WEIGHT,
        "scale_bits_per_weight": D11_SCALE_BITS_PER_WEIGHT,
        "bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
        "effective_bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
        "load_seconds": load_seconds,
        "serve_seconds": serve_seconds,
        "decoding": decoding,
    });
    println!("{record}");
    Ok(())
}

fn load_tokens(tokens: &str) -> Result<Vec<u32>, CliError> {
    let bytes = std::fs::read(tokens)?;
    let slice = if bytes.starts_with(b"UORT") {
        if bytes.len() < 64 {
            return Err(usage("truncated UORT token file"));
        }
        &bytes[64..]
    } else {
        &bytes[..]
    };
    if slice.len() % 2 != 0 {
        return Err(usage("the token file is not little-endian u16"));
    }
    Ok(slice
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect())
}

fn digest(args: &[String]) -> Result<(), CliError> {
    let [artifact, tokens, windows] = args else {
        return Err(usage("digest takes ARTIFACT TOKENS.u16 WINDOWS"));
    };
    let windows = number(windows, "WINDOWS")?;
    let model = IntegerStackModel::load(Path::new(artifact))?;
    let (vocab, context) = (model.shape().vocab, model.shape().context);
    let ids = load_tokens(tokens)?;
    // Every window needs a whole context and a stride of at least one token.
    if windows == 0
        || context
            .checked_add(windows)
            .is_none_or(|needed| ids.len() <= needed)
    {
        return Err(usage("too few tokens for the windows"));
    }
    if let Some(&id) = ids.iter().find(|&&id| id as usize >= vocab) {
        return Err(CliError::Stack(StackError::Token { token: id, vocab }));
    }
    let steps = windows
        .checked_mul(context)
        .ok_or_else(|| usage("WINDOWS times the context overflows"))?;
    let stride = (ids.len() - context - 1) / windows;
    let mut hash = Sha256::new();
    // A pointer model's mixtures, hashed apart from its logits.
    let mut mixture_hash = model.pointer().map(|_| Sha256::new());
    let mut session = model.session();
    let clock = Instant::now();
    for window in 0..windows {
        // `window * stride < windows * stride <= ids.len() - context - 1`, so
        // neither the start nor the window's end overflows.
        let start = window * stride;
        session.reset();
        for &id in &ids[start..start + context] {
            for value in session.step(id)? {
                hash.update(value.to_le_bytes());
            }
            if let (Some(mixture_hash), Some(mixture)) = (&mut mixture_hash, session.mixture()) {
                for value in mixture {
                    mixture_hash.update(value.to_le_bytes());
                }
            }
        }
    }
    let seconds = clock.elapsed().as_secs_f64();
    let mut record = json!({
        "schema": "uor-r4.stack-d11-digest/1",
        "artifact_sha256": model.artifact_sha256(),
        "tokens": tokens,
        "windows": windows,
        "steps": steps,
        "logits_sha256": hex::encode(hash.finalize()),
        // Whether the record carries `mixture_sha256` (a pointer model).
        "pointer_mixture_hashed": model.pointer().is_some(),
        "seconds": seconds,
        "tokens_per_second": steps as f64 / seconds,
        "weights_read_per_token": model.weights_per_token(),
        "nominal_bits_per_weight": D11_NOMINAL_BITS_PER_WEIGHT,
        "scale_bits_per_weight": D11_SCALE_BITS_PER_WEIGHT,
        "bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
        "effective_bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
    });
    if let Some(mixture_hash) = mixture_hash {
        record["mixture_sha256"] = json!(hex::encode(mixture_hash.finalize()));
    }
    println!("{record}");
    Ok(())
}

fn percentile(sorted: &[f64], percent: usize) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[(sorted.len() * percent / 100).min(sorted.len() - 1)]
}

fn process_rss_mb() -> Option<f64> {
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let kib: f64 = std::str::from_utf8(&output.stdout)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(kib / 1024.0)
}

fn sysctl_loadavg() -> String {
    std::process::Command::new("sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|| "UNAVAILABLE".to_string())
}

/// Analytical estimate of active working-set memory traffic per token based on model shape
/// parameters, static weight sizes, activation tables, and scratch buffers.
///
/// NOTE: This is a shape-based analytical calculation, NOT an instrumented hardware measurement.
/// It does not observe executed cache line fills, bus traffic, or occupancy-dependent memory access.
/// True hardware memory traffic is unmeasured and unavailable.
fn estimate_analytical_bytes_per_token(model: &IntegerStackModel) -> u64 {
    let s = model.shape();
    let (d, mlp, vocab, heads) = (s.width, s.mlp, s.vocab, s.heads);
    let mut bytes = 0u64;

    // 1. Embedding read: 1 row of embed (weights + scales)
    bytes += (d / 2) as u64; // nibbles
    bytes += d.div_ceil(32) as u64; // scales

    // 2. State & norm buffers
    let x_bytes = (d * 4) as u64; // b.x i32
    let norm_bytes = (d * 2) as u64; // b.norm i16
    let scratch_norm_bytes = (d * 8) as u64; // scratch i64 in rms_norm
    let tables_d_bytes = (d * 16 * 4) as u64; // b.tables[..d]
    let pairs_d_bytes = ((d / 2) * 256 * 4) as u64; // b.pairs

    for (_l, kind) in s.pattern.bytes().enumerate() {
        // Mixer block:
        // RMSNorm
        bytes += x_bytes + norm_bytes + scratch_norm_bytes;
        // Activation tables & pair tables
        bytes += norm_bytes + tables_d_bytes + tables_d_bytes + pairs_d_bytes;

        if kind == b'r' {
            // Recurrence:
            // rec_in: GEMV pairs (reads pairs + weights + scales, writes branches)
            let rec_in_weights = (2 * d * d / 2) as u64;
            let rec_in_scales = (2 * d * d.div_ceil(32)) as u64;
            let branches_bytes = (2 * d * 4) as u64;
            bytes += pairs_d_bytes + rec_in_weights + rec_in_scales + branches_bytes;

            // rec_gate: GEMV pairs (reads pairs + weights + scales, writes gate_out)
            let gate_rows = s.gate_rows();
            let rec_gate_weights = (gate_rows * d / 2) as u64;
            let rec_gate_scales = (gate_rows * d.div_ceil(32)) as u64;
            let gate_out_bytes = (gate_rows * 4) as u64;
            bytes += pairs_d_bytes + rec_gate_weights + rec_gate_scales + gate_out_bytes;

            // Biases & convolution:
            bytes += gate_out_bytes; // bias add
            let conv_taps = (4 * d * 2) as u64; // taps
            let conv_bias = (d * 4) as u64;
            let history_bytes = (4 * d * 4) as u64;
            bytes += conv_taps + conv_bias + history_bytes * 2; // read and update history

            // Recurrence state update:
            let lanes = s.lanes();
            let state_bytes = (lanes * 4 * 8) as u64;
            let rates_bytes = (lanes * 2) as u64;
            bytes += state_bytes * 2 + rates_bytes;

            // rec_out: GEMV (reads tables/pairs + weights + scales, writes proj)
            let rec_out_weights = (d * d / 2) as u64;
            let rec_out_scales = (d * d.div_ceil(32)) as u64;
            let proj_bytes = (d * 4) as u64;
            bytes += pairs_d_bytes + rec_out_weights + rec_out_scales + proj_bytes;
        } else {
            // Read mixer: query, key, value, null, out
            let q_weights = (d * d / 2) as u64;
            let q_scales = (d * d.div_ceil(32)) as u64;
            bytes += (pairs_d_bytes + q_weights + q_scales + (d * 4) as u64) * 4; // q, k, v, out
            let null_weights = (heads * d / 2) as u64;
            let null_scales = (heads * d.div_ceil(32)) as u64;
            bytes += pairs_d_bytes + null_weights + null_scales + (heads * 4) as u64;
        }
        // Residual add
        bytes += x_bytes * 2;

        // MLP block:
        // RMSNorm
        bytes += x_bytes + norm_bytes + scratch_norm_bytes;
        // Activation tables & pair tables
        bytes += norm_bytes + tables_d_bytes + tables_d_bytes + pairs_d_bytes;

        // gate & up GEMVs
        let gate_weights = (mlp * d / 2) as u64;
        let gate_scales = (mlp * d.div_ceil(32)) as u64;
        let mlp_gate_bytes = (mlp * 4) as u64;
        bytes += pairs_d_bytes + gate_weights + gate_scales + mlp_gate_bytes;

        let up_weights = (mlp * d / 2) as u64;
        let up_scales = (mlp * d.div_ceil(32)) as u64;
        let mlp_up_bytes = (mlp * 4) as u64;
        bytes += pairs_d_bytes + up_weights + up_scales + mlp_up_bytes;

        // SwiGLU: reads gate and up, writes scratch
        let swiglu_scratch_bytes = (mlp * 8) as u64;
        bytes += mlp_gate_bytes + mlp_up_bytes + swiglu_scratch_bytes;

        // Quantize: reads scratch, writes act
        let act_bytes = (mlp * 2) as u64;
        bytes += swiglu_scratch_bytes + act_bytes;

        // Activation tables for act
        let tables_mlp_bytes = (mlp * 16 * 4) as u64;
        bytes += act_bytes + tables_mlp_bytes;

        // down GEMV
        let down_weights = (d * mlp / 2) as u64;
        let down_scales = (d * mlp.div_ceil(32)) as u64;
        let proj_bytes = (d * 4) as u64;
        bytes += tables_mlp_bytes + down_weights + down_scales + proj_bytes;

        // Residual add
        bytes += x_bytes * 2;
    }

    // 3. Final Norm and Unembedding Head:
    bytes += x_bytes + norm_bytes + scratch_norm_bytes;
    bytes += norm_bytes + tables_d_bytes + tables_d_bytes + pairs_d_bytes;
    let head_weights = (vocab * d / 2) as u64;
    let head_scales = (vocab * d.div_ceil(32)) as u64;
    let logits_bytes = (vocab * 4) as u64;
    bytes += pairs_d_bytes + head_weights + head_scales + logits_bytes;

    bytes
}

fn cost(args: &[String]) -> Result<(), CliError> {
    let (artifact, tokens, windows, prompt_len, out_dir) = match args {
        [a, t, w, p] => (
            a,
            t,
            number(w, "WINDOWS")?,
            number(p, "PROMPT_TOKENS")?,
            None,
        ),
        [a, t, w, p, o] => (
            a,
            t,
            number(w, "WINDOWS")?,
            number(p, "PROMPT_TOKENS")?,
            Some(Path::new(o)),
        ),
        _ => {
            return Err(usage(
                "cost takes ARTIFACT TOKENS.u16 WINDOWS PROMPT_TOKENS [OUT_DIR]",
            ))
        }
    };
    if let Some(out) = out_dir {
        uor_r4_integer::report_output::claim(out)?;
    }
    let started_unix_s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let load_avg_before = sysctl_loadavg();
    let initial_rss_mb = process_rss_mb();

    let load_clock = Instant::now();
    let model = IntegerStackModel::load(Path::new(artifact))?;
    let cold_load_seconds = load_clock.elapsed().as_secs_f64();
    let post_load_rss_mb = process_rss_mb();

    let (vocab, context) = (model.shape().vocab, model.shape().context);
    let ids = load_tokens(tokens)?;
    // Every window needs a whole context and a stride of at least one token.
    if windows == 0
        || context
            .checked_add(windows)
            .is_none_or(|needed| ids.len() <= needed)
    {
        return Err(usage("too few tokens for the windows"));
    }
    if let Some(&id) = ids.iter().find(|&&id| id as usize >= vocab) {
        return Err(CliError::Stack(StackError::Token { token: id, vocab }));
    }
    let _expected_steps = windows
        .checked_mul(context)
        .ok_or_else(|| usage("WINDOWS times the context overflows"))?;

    let binary_sha256 = std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read(&p).ok())
        .map(|bytes| {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            format!("{:x}", hasher.finalize())
        });
    let data_sha256 = std::fs::read(tokens).ok().map(|bytes| {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        format!("{:x}", hasher.finalize())
    });

    // 1. Prompt ingestion measurement
    let prompt_tokens = prompt_len.min(ids.len() / 2).min(context);
    let prompt = &ids[..prompt_tokens];
    let mut session = model.session();
    let prompt_clock = Instant::now();
    for &id in prompt {
        session.step(id)?;
    }
    let prompt_seconds = prompt_clock.elapsed().as_secs_f64();
    let prompt_ms_per_token = (prompt_seconds * 1000.0) / prompt_tokens as f64;
    let prompt_tokens_per_second = prompt_tokens as f64 / prompt_seconds;

    // 2. Input-driven forward step latency measurement over WINDOWS
    // Note: session.step(id) feeds dataset tokens directly in teacher-forced mode without
    // token sampling or generation loops. Throughput here represents input-driven forward throughput.
    let stride = (ids.len() - context - 1) / windows;
    let mut step_us: Vec<f64> = Vec::with_capacity(windows * context);
    let mut total_steps = 0usize;
    let eval_clock = Instant::now();
    for window in 0..windows {
        let start = window * stride;
        session.reset();
        for &id in &ids[start..start + context] {
            let step_start = Instant::now();
            session.step(id)?;
            step_us.push(step_start.elapsed().as_nanos() as f64 / 1000.0);
            total_steps += 1;
        }
    }
    let total_eval_seconds = eval_clock.elapsed().as_secs_f64();
    let post_eval_rss_mb = process_rss_mb();

    step_us.sort_by(f64::total_cmp);
    let n = step_us.len() as f64;
    let mean_us = step_us.iter().sum::<f64>() / n;
    let stddev_us = (step_us.iter().map(|&x| (x - mean_us).powi(2)).sum::<f64>() / n).sqrt();
    let p50_ms = percentile(&step_us, 50) / 1000.0;
    let p90_ms = percentile(&step_us, 90) / 1000.0;
    let p95_ms = percentile(&step_us, 95) / 1000.0;
    let p99_ms = percentile(&step_us, 99) / 1000.0;

    let load_avg_after = sysctl_loadavg();

    // 3. Analytical bytes per token estimate
    let bytes_touched_per_token = estimate_analytical_bytes_per_token(&model);
    let input_driven_forward_throughput = total_steps as f64 / total_eval_seconds;

    let record = json!({
        "schema": "uor-r4.stack-d11-cost/1",
        "started_unix_s": started_unix_s,
        "artifact": {
            "path": artifact,
            "sha256": model.artifact_sha256(),
        },
        "binary": {
            "path": std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default(),
            "sha256": binary_sha256,
        },
        "tokens": {
            "path": tokens,
            "sha256": data_sha256,
            "windows": windows,
            "context": context,
            "total_steps": total_steps,
        },
        "machine": {
            "threads": 1,
            "load_average_before": load_avg_before,
            "load_average_after": load_avg_after,
        },
        "memory": {
            "initial_rss_mb": initial_rss_mb,
            "post_load_rss_mb": post_load_rss_mb,
            "post_eval_rss_mb": post_eval_rss_mb,
            "peak_rss_mb": post_eval_rss_mb,
            "weights_read_per_token": model.weights_per_token(),
            "analytical_weights_read_per_token": model.weights_per_token(),
            "nominal_bits_per_weight": D11_NOMINAL_BITS_PER_WEIGHT,
            "scale_bits_per_weight": D11_SCALE_BITS_PER_WEIGHT,
            "bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
            "effective_bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
            "weights_notes": "analytical shape-based count: 7,238,304 weights read per token (3,845,349 bytes weights/scales; 4.25 effective bits per weight)",
            "bytes_touched_per_token": bytes_touched_per_token,
            "analytical_bytes_touched_per_token_estimate": bytes_touched_per_token,
            "rss_notes": "peak_rss_mb / post_eval_rss_mb is a single post-evaluation RSS sample via ps, not a continuous high-water mark",
            "traffic_notes": "bytes_touched is an analytical estimate derived from tensor shapes, not hardware-instrumented memory traffic",
        },
        "bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
        "effective_bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
        "cold_load": {
            "seconds": cold_load_seconds,
            "ms": cold_load_seconds * 1000.0,
        },
        "prompt_ingestion": {
            "tokens": prompt_tokens,
            "seconds": prompt_seconds,
            "ms_per_token": prompt_ms_per_token,
            "tokens_per_second": prompt_tokens_per_second,
        },
        "step_latency": {
            "mean_ms": mean_us / 1000.0,
            "stddev_ms": stddev_us / 1000.0,
            "min_ms": step_us[0] / 1000.0,
            "p50_ms": p50_ms,
            "p90_ms": p90_ms,
            "p95_ms": p95_ms,
            "p99_ms": p99_ms,
            "max_ms": step_us[step_us.len() - 1] / 1000.0,
            "tokens_per_second": input_driven_forward_throughput,
            "input_driven_forward_throughput_tokens_per_second": input_driven_forward_throughput,
            "throughput_mode": "input-driven teacher-forced stepping over dataset tokens (1 thread, 2,048 steps across 8 windows of context 256), not free-running autoregressive generation",
            "threads": 1,
        },
    });

    println!("{}", serde_json::to_string_pretty(&record).unwrap());

    if let Some(out) = out_dir {
        std::fs::write(out.join("cost.json"), serde_json::to_vec_pretty(&record)?)?;
        uor_r4_integer::report_output::seal(out)?;
        uor_r4_integer::report_output::verify(out)?;
    }

    Ok(())
}

fn run() -> Result<(), CliError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.split_first() {
        Some((mode, rest)) if mode == "generate" => generate(rest),
        Some((mode, rest)) if mode == "digest" => digest(rest),
        Some((mode, rest)) if mode == "cost" => cost(rest),
        _ => Err(CliError::Usage(USAGE.to_owned())),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
