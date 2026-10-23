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
#![forbid(unsafe_code)]
#![recursion_limit = "256"]

use std::path::Path;
use std::time::Instant;

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_integer::stack::{
    sample_mean_sd_t_ci95, stack_argmax, student_t_0975,
    validate_protocol_v3_energy_receipt_with_mode, verify_step_objdump_zero_mul_zero_float,
    IntegerStackModel, StackError,
};

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

const USAGE: &str = "usage: uor-r4-stack generate ARTIFACT PROMPT_IDS NEW_TOKENS | digest ARTIFACT TOKENS.u16 WINDOWS | cost ARTIFACT TOKENS.u16 WINDOWS PROMPT_TOKENS [OUT_DIR] [--energy-receipt PATH] [--allow-provisional]";

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

fn parse_vmmap_size_bytes(token: &str) -> Option<u64> {
    let token = token.trim().trim_end_matches('.');
    let (num_str, mult) = if let Some(s) = token
        .strip_suffix("KB")
        .or_else(|| token.strip_suffix("KiB"))
        .or_else(|| token.strip_suffix('K'))
    {
        (s, 1024.0)
    } else if let Some(s) = token
        .strip_suffix("MB")
        .or_else(|| token.strip_suffix("MiB"))
        .or_else(|| token.strip_suffix('M'))
    {
        (s, 1024.0 * 1024.0)
    } else if let Some(s) = token
        .strip_suffix("GB")
        .or_else(|| token.strip_suffix("GiB"))
        .or_else(|| token.strip_suffix('G'))
    {
        (s, 1024.0 * 1024.0 * 1024.0)
    } else if let Some(s) = token.strip_suffix('B') {
        (s, 1.0)
    } else {
        (token, 1.0)
    };
    let val: f64 = num_str.trim().parse().ok()?;
    if val <= 0.0 {
        return None;
    }
    Some((val * mult).round() as u64)
}

/// Kernel high-water-mark memory footprint in bytes.
///
/// Because this binary forbids unsafe code, it cannot invoke FFI
/// `libc::getrusage(RUSAGE_SELF)` (`ru_maxrss`). Instead, it queries the OS kernel's
/// high-water-mark physical footprint via `vmmap -summary <pid>` (`Physical footprint (peak)`)
/// on macOS and `/proc/self/status` (`VmHWM`) on Linux.
fn kernel_peak_memory_bytes() -> Option<u64> {
    if cfg!(target_os = "macos") {
        let output = std::process::Command::new("vmmap")
            .args(["-summary", &std::process::id().to_string()])
            .output()
            .ok()?;
        let text = std::str::from_utf8(&output.stdout).ok()?;
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix("Physical footprint (peak):") {
                let first_token = rest.split_whitespace().next()?;
                if let Some(bytes) = parse_vmmap_size_bytes(first_token) {
                    return Some(bytes);
                }
            }
        }
        None
    } else if cfg!(target_os = "linux") {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmHWM:") {
                let kib: u64 = rest.split_whitespace().next()?.parse().ok()?;
                return Some(kib.saturating_mul(1024));
            }
        }
        None
    } else {
        None
    }
}

fn detect_external_inference_daemons() -> Vec<String> {
    const FORBIDDEN_DAEMONS: &[&str] = &[
        "ollama",
        "llama-server",
        "llama-cli",
        "vllm",
        "lmstudio",
        "mlx_lm",
    ];
    let Ok(output) = std::process::Command::new("ps")
        .args(["-axo", "comm="])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let Ok(text) = std::str::from_utf8(&output.stdout) else {
        return Vec::new();
    };
    let mut detected = Vec::new();
    for line in text.lines() {
        let base = line.trim().rsplit('/').next().unwrap_or("");
        for &daemon in FORBIDDEN_DAEMONS {
            if base == daemon && !detected.iter().any(|d| d == daemon) {
                detected.push(daemon.to_string());
            }
        }
    }
    detected
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

    for kind in s.pattern.bytes() {
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
    let mut positional: Vec<&String> = Vec::new();
    let mut energy_receipt_path: Option<String> = None;
    let mut allow_provisional = false;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--allow-provisional" => {
                allow_provisional = true;
                i += 1;
            }
            "--energy-receipt" => {
                let Some(path) = args.get(i + 1) else {
                    return Err(usage("--energy-receipt requires a PATH argument"));
                };
                energy_receipt_path = Some(path.clone());
                i += 2;
            }
            arg if arg.starts_with("--") => {
                return Err(usage(&format!("unrecognized cost flag: {arg}")));
            }
            _ => {
                positional.push(&args[i]);
                i += 1;
            }
        }
    }

    let (artifact, tokens, windows, prompt_len, out_dir) = match positional.as_slice() {
        [a, t, w, p] => (
            *a,
            *t,
            number(w, "WINDOWS")?,
            number(p, "PROMPT_TOKENS")?,
            None,
        ),
        [a, t, w, p, o] => (
            *a,
            *t,
            number(w, "WINDOWS")?,
            number(p, "PROMPT_TOKENS")?,
            Some(Path::new(o.as_str())),
        ),
        _ => {
            return Err(usage(
                "cost takes ARTIFACT TOKENS.u16 WINDOWS PROMPT_TOKENS [OUT_DIR] [--energy-receipt PATH] [--allow-provisional]",
            ))
        }
    };
    if let Some(out) = out_dir {
        if out.exists() {
            return Err(CliError::Io(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "report directory {} already exists; choose a new attempt path instead of reusing or overwriting an existing report",
                    out.display()
                ),
            )));
        }
    }

    // 0. If an external energy receipt is supplied via `--energy-receipt`, validate it against
    // Protocol v3 (`work_rate_w == gross_j_per_tok * tokens_per_s`, Student-t CI95, required v3
    // fields, and in strict mode `work_rate_w > idle_rate_w` and empty `external_inference_daemons_detected`).
    // In strict qualification mode (`!allow_provisional`), `--energy-receipt` is required so the
    // physical work-rate identity `W = (J/tok) * (tok/s)` is always verified.
    let validated_energy_receipt = if let Some(ref receipt_path) = energy_receipt_path {
        let receipt_bytes = std::fs::read(receipt_path)?;
        let receipt_json: serde_json::Value = serde_json::from_slice(&receipt_bytes)?;
        let errors =
            validate_protocol_v3_energy_receipt_with_mode(&receipt_json, allow_provisional);
        if !errors.is_empty() {
            return Err(CliError::Usage(format!(
                "Protocol v3 energy receipt validation failed on {receipt_path}: {errors:?}"
            )));
        }
        Some(receipt_json)
    } else if !allow_provisional {
        return Err(CliError::Usage(
            "refusing strict M5 cost qualification without --energy-receipt <PATH> to verify W = (J/tok) * (tok/s) (pass --allow-provisional for latency/memory-only rehearsal)".to_owned(),
        ));
    } else {
        None
    };

    // 0b. Static disassembly gate: verify via `objdump --demangle -d` (or `otool -tV`) on the
    // running executable that `IntegerStackSession::step`, `stack_step`, and all emitted
    // `uor_r4_integer::stack` mixer and kernel symbols contain zero multiply, divide, or
    // floating-point instructions.
    let current_exe = std::env::current_exe()?;
    let (disasm_symbols_checked, disasm_instructions_checked) =
        verify_step_objdump_zero_mul_zero_float(&current_exe)
            .map_err(|e| CliError::Usage(format!("step disassembly audit failed: {e}")))?;

    // 0c. Pre-flight external inference daemon refusal (before model work or claiming out_dir).
    let external_daemons_before = detect_external_inference_daemons();
    if !allow_provisional && !external_daemons_before.is_empty() {
        return Err(CliError::Usage(format!(
            "refusing cost qualification while external inference daemons are running: {external_daemons_before:?} (pass --allow-provisional for non-qualification rehearsal)"
        )));
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
    let expected_steps = windows
        .checked_mul(context)
        .ok_or_else(|| usage("WINDOWS times the context overflows"))?;
    if expected_steps < 2 {
        return Err(usage(
            "cost requires at least 2 total steps (WINDOWS * context >= 2) for Student-t confidence intervals",
        ));
    }

    let binary_sha256 = std::fs::read(&current_exe).ok().map(|bytes| {
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
    let post_session_alloc_rss_mb = process_rss_mb();
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
    let mut unphased_hash = Sha256::new();
    let eval_clock = Instant::now();
    for window in 0..windows {
        let start = window * stride;
        session.reset();
        for &id in &ids[start..start + context] {
            let step_start = Instant::now();
            let logits = session.step(id)?;
            step_us.push(step_start.elapsed().as_nanos() as f64 / 1000.0);
            for value in logits {
                unphased_hash.update(value.to_le_bytes());
            }
            total_steps += 1;
        }
    }
    let total_eval_seconds = eval_clock.elapsed().as_secs_f64();
    let unphased_logits_sha256 = hex::encode(unphased_hash.finalize());
    let post_eval_rss_mb = process_rss_mb();

    // 2b. Four-phase latency breakdown over the identical WINDOWS * context steps
    // using `measure_step_phases` (outside the audited `IntegerStackSession::step` call graph),
    // verifying bit-identical logits against the unphased pass.
    let mut ingestion_us: Vec<f64> = Vec::with_capacity(windows * context);
    let mut routing_us: Vec<f64> = Vec::with_capacity(windows * context);
    let mut memory_lookup_us: Vec<f64> = Vec::with_capacity(windows * context);
    let mut emission_us: Vec<f64> = Vec::with_capacity(windows * context);
    let mut phased_total_us: Vec<f64> = Vec::with_capacity(windows * context);
    let mut phased_hash = Sha256::new();
    for window in 0..windows {
        let start = window * stride;
        session.reset();
        for &id in &ids[start..start + context] {
            let timing = session.measure_step_phases(id)?;
            ingestion_us.push(timing.ingestion_ns as f64 / 1000.0);
            routing_us.push(timing.routing_ns as f64 / 1000.0);
            memory_lookup_us.push(timing.memory_lookup_ns as f64 / 1000.0);
            emission_us.push(timing.emission_ns as f64 / 1000.0);
            phased_total_us.push(timing.total_ns as f64 / 1000.0);
            for value in session.logits() {
                phased_hash.update(value.to_le_bytes());
            }
        }
    }
    let phased_logits_sha256 = hex::encode(phased_hash.finalize());
    let phase_instrumentation_logits_match = unphased_logits_sha256 == phased_logits_sha256;
    if !phase_instrumentation_logits_match {
        return Err(CliError::Usage(format!(
            "phase instrumentation logits drift: unphased={unphased_logits_sha256} vs phased={phased_logits_sha256}"
        )));
    }

    let max_sampled_rss_mb = [
        initial_rss_mb,
        post_load_rss_mb,
        post_session_alloc_rss_mb,
        post_eval_rss_mb,
    ]
    .into_iter()
    .flatten()
    .reduce(f64::max);
    let kernel_peak_bytes = kernel_peak_memory_bytes();
    if !allow_provisional && kernel_peak_bytes.is_none() {
        return Err(CliError::Usage(
            "refusing cost qualification because kernel high-water-mark footprint (vmmap on macOS / VmHWM on Linux) is unavailable; pass --allow-provisional to permit ps RSS fallback".to_owned(),
        ));
    }
    let kernel_peak_mb = kernel_peak_bytes.map(|b| (b as f64) / (1024.0 * 1024.0));
    let peak_rss_mb = kernel_peak_mb.or(max_sampled_rss_mb);
    let mut external_daemons = external_daemons_before;
    for d in detect_external_inference_daemons() {
        if !external_daemons.contains(&d) {
            external_daemons.push(d);
        }
    }
    if !allow_provisional && !external_daemons.is_empty() {
        return Err(CliError::Usage(format!(
            "refusing cost qualification while external inference daemons are running: {external_daemons:?} (pass --allow-provisional for non-qualification rehearsal)"
        )));
    }

    step_us.sort_by(f64::total_cmp);
    ingestion_us.sort_by(f64::total_cmp);
    routing_us.sort_by(f64::total_cmp);
    memory_lookup_us.sort_by(f64::total_cmp);
    emission_us.sort_by(f64::total_cmp);
    phased_total_us.sort_by(f64::total_cmp);

    let df = step_us.len().saturating_sub(1);
    let t_crit = student_t_0975(df);
    let (mean_us, stddev_us, ci95_us) = sample_mean_sd_t_ci95(&step_us).ok_or_else(|| {
        CliError::Usage("insufficient step samples (< 2) for Student-t CI95".to_owned())
    })?;
    let p50_ms = percentile(&step_us, 50) / 1000.0;
    let p90_ms = percentile(&step_us, 90) / 1000.0;
    let p95_ms = percentile(&step_us, 95) / 1000.0;
    let p99_ms = percentile(&step_us, 99) / 1000.0;

    let (ing_mean_us, ing_sd_us, ing_ci95_us) =
        sample_mean_sd_t_ci95(&ingestion_us).ok_or_else(|| {
            CliError::Usage("insufficient ingestion samples (< 2) for Student-t CI95".to_owned())
        })?;
    let (rout_mean_us, rout_sd_us, rout_ci95_us) =
        sample_mean_sd_t_ci95(&routing_us).ok_or_else(|| {
            CliError::Usage("insufficient routing samples (< 2) for Student-t CI95".to_owned())
        })?;
    let (mem_mean_us, mem_sd_us, mem_ci95_us) = sample_mean_sd_t_ci95(&memory_lookup_us)
        .ok_or_else(|| {
            CliError::Usage(
                "insufficient memory_lookup samples (< 2) for Student-t CI95".to_owned(),
            )
        })?;
    let (em_mean_us, em_sd_us, em_ci95_us) =
        sample_mean_sd_t_ci95(&emission_us).ok_or_else(|| {
            CliError::Usage("insufficient emission samples (< 2) for Student-t CI95".to_owned())
        })?;
    let (phased_total_mean_us, phased_total_sd_us, phased_total_ci95_us) =
        sample_mean_sd_t_ci95(&phased_total_us).ok_or_else(|| {
            CliError::Usage("insufficient phased_total samples (< 2) for Student-t CI95".to_owned())
        })?;
    let phase_sum_mean_us = (ing_mean_us + rout_mean_us + mem_mean_us + em_mean_us).max(1e-9);
    let measured_total_denom_us = phased_total_mean_us.max(phase_sum_mean_us);
    let unaccounted_overhead_us = (phased_total_mean_us - phase_sum_mean_us).max(0.0);

    let mean_s = (mean_us / 1_000_000.0).max(1e-12);
    let step_implied_tokens_per_second = 1.0 / mean_s;
    let work_rate_identity_verified = validated_energy_receipt.is_some();

    let load_avg_after = sysctl_loadavg();

    // 3. Analytical bytes per token estimate & dynamic parameter breakdown
    let bytes_touched_per_token = estimate_analytical_bytes_per_token(&model);
    let param_breakdown = model.parameter_breakdown();
    let active_4bit_ratio = if param_breakdown.total_4bit_weights > 0 {
        (param_breakdown.weights_per_token as f64) / (param_breakdown.total_4bit_weights as f64)
    } else {
        0.0
    };
    let active_total_param_ratio = if param_breakdown.total_learned_parameters > 0 {
        (param_breakdown.active_learned_parameters_per_token as f64)
            / (param_breakdown.total_learned_parameters as f64)
    } else {
        0.0
    };
    let weights_notes = format!(
        "analytical shape-based count: {} 4-bit weights read per token out of {} total 4-bit weights ({:.4}% active; {} bytes active weights/scales out of {} total bytes; {:.2} effective bits per weight; {} active / {} total learned scalars, {} B)",
        param_breakdown.weights_per_token,
        param_breakdown.total_4bit_weights,
        active_4bit_ratio * 100.0,
        param_breakdown.active_4bit_weight_and_scale_bytes_per_token,
        param_breakdown.total_4bit_weight_and_scale_bytes,
        D11_EFFECTIVE_BITS_PER_WEIGHT,
        param_breakdown.active_learned_scalars_per_token,
        param_breakdown.total_learned_scalars,
        param_breakdown.total_learned_scalar_bytes,
    );
    let input_driven_forward_throughput = total_steps as f64 / total_eval_seconds;

    let mut record = json!({
        "schema": "uor-r4.stack-d11-cost/1",
        "started_unix_s": started_unix_s,
        "allow_provisional": allow_provisional,
        "artifact": {
            "path": artifact,
            "sha256": model.artifact_sha256(),
        },
        "binary": {
            "path": current_exe.display().to_string(),
            "sha256": binary_sha256,
        },
        "disassembly_audit": {
            "step_objdump_zero_mul_zero_float": true,
            "scope": "Transitive call graph from IntegerStackSession::step and stack::kernels::stack_argmax across all 43 mandatory uor_r4_integer::stack symbols, stack closure instantiations, and reachable non-formatting core/alloc/compiler_builtins callees; indirect calls (blr/call *) are rejected; 64-bit d0..d31 loads/stores are permitted solely for AArch64 NEON 2xi32 integer SIMD (sqadd.2s) while h0..h31/s0..s31 scalar float loads/stores and all float/multiply/divide opcodes are banned",
            "legacy_ratchet_note": "stack_exp_neg and stack_sigmoid_q31 are D11 sealed integer-table kernels tracked as legacy sites by the D23 ratchet; this disassembly check certifies zero hardware multiply/divide/float opcodes, not CHARTER §3.2 mechanism compliance",
            "symbols_checked": disasm_symbols_checked,
            "instructions_checked": disasm_instructions_checked,
        },
        "tokens": {
            "path": tokens,
            "sha256": data_sha256,
            "windows": windows,
            "context": context,
            "total_steps": total_steps,
            "logits_sha256": unphased_logits_sha256,
            "phase_instrumentation_logits_match": phase_instrumentation_logits_match,
        },
        "machine": {
            "threads": 1,
            "load_average_before": load_avg_before,
            "load_average_after": load_avg_after,
            "external_inference_daemons_detected": external_daemons,
        },
        "memory": {
            "initial_rss_mb": initial_rss_mb,
            "post_load_rss_mb": post_load_rss_mb,
            "post_session_alloc_rss_mb": post_session_alloc_rss_mb,
            "post_eval_rss_mb": post_eval_rss_mb,
            "max_sampled_rss_mb": max_sampled_rss_mb,
            "kernel_peak_footprint_bytes": kernel_peak_bytes,
            "kernel_peak_footprint_mb": kernel_peak_mb,
            "peak_rss_mb": peak_rss_mb,
            "weights_read_per_token": model.weights_per_token(),
            "analytical_weights_read_per_token": model.weights_per_token(),
            "total_4bit_weights": param_breakdown.total_4bit_weights,
            "weights_read_over_total_4bit_weights": active_4bit_ratio,
            "active_learned_scalars_per_token": param_breakdown.active_learned_scalars_per_token,
            "total_learned_scalars": param_breakdown.total_learned_scalars,
            "total_learned_parameters": param_breakdown.total_learned_parameters,
            "active_learned_parameters_per_token": param_breakdown.active_learned_parameters_per_token,
            "active_over_total_learned_parameters": active_total_param_ratio,
            "parameter_breakdown": param_breakdown,
            "nominal_bits_per_weight": D11_NOMINAL_BITS_PER_WEIGHT,
            "scale_bits_per_weight": D11_SCALE_BITS_PER_WEIGHT,
            "bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
            "effective_bits_per_weight": D11_EFFECTIVE_BITS_PER_WEIGHT,
            "weights_notes": weights_notes,
            "bytes_touched_per_token": bytes_touched_per_token,
            "analytical_bytes_touched_per_token_estimate": bytes_touched_per_token,
            "rss_notes": "kernel_peak_footprint_bytes / kernel_peak_footprint_mb is the kernel high-water-mark footprint (vmmap Physical footprint (peak) on macOS, /proc/self/status VmHWM on Linux; forbid(unsafe_code) forbids libc::getrusage FFI); strict mode refuses if kernel_peak_footprint_bytes is None; max_sampled_rss_mb is the maximum across initial, post-load, post-session-alloc, and post-eval ps -o rss= samples",
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
            "ci95_student_t_ms": ci95_us / 1000.0,
            "student_t_0975_df": df,
            "student_t_0975": t_crit,
            "min_ms": step_us[0] / 1000.0,
            "p50_ms": p50_ms,
            "p90_ms": p90_ms,
            "p95_ms": p95_ms,
            "p99_ms": p99_ms,
            "max_ms": step_us[step_us.len() - 1] / 1000.0,
            "tokens_per_second": input_driven_forward_throughput,
            "step_implied_tokens_per_second": step_implied_tokens_per_second,
            "work_rate_identity_verified": work_rate_identity_verified,
            "input_driven_forward_throughput_tokens_per_second": input_driven_forward_throughput,
            "throughput_mode": format!(
                "input-driven teacher-forced stepping over dataset tokens (1 thread, {} steps across {} windows of context {}), not free-running autoregressive generation",
                total_steps, windows, context
            ),
            "threads": 1,
            "phase_latency_us": {
                "measured_total_step_us": phased_total_mean_us,
                "measured_total_step_stddev_us": phased_total_sd_us,
                "measured_total_step_ci95_student_t_us": phased_total_ci95_us,
                "phase_sum_us": phase_sum_mean_us,
                "unaccounted_overhead_us": unaccounted_overhead_us,
                "ingestion": {
                    "mean_us": ing_mean_us,
                    "stddev_us": ing_sd_us,
                    "ci95_student_t_us": ing_ci95_us,
                    "p50_us": percentile(&ingestion_us, 50),
                    "p90_us": percentile(&ingestion_us, 90),
                    "p99_us": percentile(&ingestion_us, 99),
                    "share_of_step": ing_mean_us / phase_sum_mean_us,
                    "share_of_measured_step": ing_mean_us / measured_total_denom_us,
                },
                "routing": {
                    "mean_us": rout_mean_us,
                    "stddev_us": rout_sd_us,
                    "ci95_student_t_us": rout_ci95_us,
                    "p50_us": percentile(&routing_us, 50),
                    "p90_us": percentile(&routing_us, 90),
                    "p99_us": percentile(&routing_us, 99),
                    "share_of_step": rout_mean_us / phase_sum_mean_us,
                    "share_of_measured_step": rout_mean_us / measured_total_denom_us,
                },
                "memory_lookup": {
                    "mean_us": mem_mean_us,
                    "stddev_us": mem_sd_us,
                    "ci95_student_t_us": mem_ci95_us,
                    "p50_us": percentile(&memory_lookup_us, 50),
                    "p90_us": percentile(&memory_lookup_us, 90),
                    "p99_us": percentile(&memory_lookup_us, 99),
                    "share_of_step": mem_mean_us / phase_sum_mean_us,
                    "share_of_measured_step": mem_mean_us / measured_total_denom_us,
                },
                "emission": {
                    "mean_us": em_mean_us,
                    "stddev_us": em_sd_us,
                    "ci95_student_t_us": em_ci95_us,
                    "p50_us": percentile(&emission_us, 50),
                    "p90_us": percentile(&emission_us, 90),
                    "p99_us": percentile(&emission_us, 99),
                    "share_of_step": em_mean_us / phase_sum_mean_us,
                    "share_of_measured_step": em_mean_us / measured_total_denom_us,
                },
            },
        },
    });
    if let Some(energy_json) = validated_energy_receipt {
        record["validated_energy_receipt"] = energy_json;
    }

    println!("{}", serde_json::to_string_pretty(&record).unwrap());

    if let Some(out) = out_dir {
        uor_r4_integer::report_output::claim(out)?;
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
