//! `toks`: the step-6 serving-rate protocol (`docs/harness/prep/
//! m1-export-gap-and-toks-DRAFT-2026-10-23.md`, protocol step 3), timed
//! through `IntegerDiscreteModel`, the product serving path, on the machine it
//! runs on (the laptop, CHARTER §3.6).
//!
//! - Prompt reading (prefill): `evaluate_prefix` over the whole prompt.
//! - Reply writing (decode): from the prompt, append each predicted token and
//!   slide the window back to the prompt length, for `decode_tokens` steps.
//!
//! Each is one warmup and `runs` timed runs. Durations are integer
//! nanoseconds (and microseconds); the median of an even count is the integer
//! mean of the middle two; p95 is the nearest rank, `ceil(0.95 n)`; rates are
//! integer tokens per second.
use serde_json::{json, Value};
use std::process::Command;
use std::time::Instant;
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

/// One timing protocol.
#[derive(Debug, Clone, Copy)]
pub struct Protocol {
    pub warmup: usize,
    pub runs: usize,
    pub prompt_tokens: usize,
    pub decode_tokens: usize,
}

/// The registered step-6 protocol: 1 warmup, 20 runs, 192 prompt tokens, 64
/// reply tokens.
pub const STEP6: Protocol = Protocol {
    warmup: 1,
    runs: 20,
    prompt_tokens: 192,
    decode_tokens: 64,
};

/// The deterministic default prompt: token `i` is `(7919 i + 13) mod
/// vocabulary`. Recorded in the report; `--prompt` replaces it.
pub fn default_prompt(vocabulary: u32, length: usize) -> Vec<u32> {
    (0..length as u64)
        .map(|i| ((7919 * i + 13) % u64::from(vocabulary)) as u32)
        .collect()
}

/// Median, nearest-rank p95 and the rates they give, over `samples`.
pub fn duration_stats(tokens: usize, samples_ns: &[u64]) -> Result<Value, String> {
    if samples_ns.is_empty() {
        return Err("no timed samples".into());
    }
    if samples_ns.contains(&0) {
        return Err("a timed sample of 0 ns: the clock did not advance".into());
    }
    let mut sorted = samples_ns.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let median_ns = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2
    };
    let rank = (95 * n).div_ceil(100);
    let p95_ns = sorted[rank - 1];
    let rate = |ns: u64| -> u128 { tokens as u128 * 1_000_000_000 / u128::from(ns) };
    Ok(json!({
        "tokens": tokens,
        "runs": n,
        "raw_ns": samples_ns,
        "raw_us": samples_ns.iter().map(|ns| ns / 1000).collect::<Vec<_>>(),
        "median_ns": median_ns,
        "median_us": median_ns / 1000,
        "p95_rank": rank,
        "p95_ns": p95_ns,
        "p95_us": p95_ns / 1000,
        "median_tokens_per_s": rate(median_ns),
        "p95_tokens_per_s": rate(p95_ns),
    }))
}

fn elapsed_ns(started: Instant) -> Result<u64, String> {
    u64::try_from(started.elapsed().as_nanos()).map_err(|e| e.to_string())
}

/// The reply tokens one decode run writes.
fn decode(model: &IntegerDiscreteModel, prompt: &[u32], steps: usize) -> Result<Vec<u32>, String> {
    let mut window = prompt.to_vec();
    let mut reply = Vec::with_capacity(steps);
    for _ in 0..steps {
        let next = model
            .evaluate_prefix(&window)
            .map_err(|e| e.to_string())?
            .prediction;
        reply.push(next);
        window.push(next);
        if window.len() > prompt.len() {
            window.remove(0);
        }
    }
    Ok(reply)
}

/// Runs `protocol` on `model` with `prompt`. Every decode run must write the
/// same reply, or the run is refused.
pub fn time_serving(
    model: &IntegerDiscreteModel,
    prompt: &[u32],
    protocol: Protocol,
) -> Result<Value, String> {
    let context = model.shape().context as usize;
    if prompt.len() != protocol.prompt_tokens {
        return Err(format!(
            "the prompt has {} tokens; the protocol reads {}",
            prompt.len(),
            protocol.prompt_tokens
        ));
    }
    if context < protocol.prompt_tokens {
        return Err(format!(
            "the artifact's context {context} is shorter than the {}-token prompt",
            protocol.prompt_tokens
        ));
    }
    let mut prefill = Vec::with_capacity(protocol.runs);
    for run in 0..protocol.warmup + protocol.runs {
        let started = Instant::now();
        model.evaluate_prefix(prompt).map_err(|e| e.to_string())?;
        let ns = elapsed_ns(started)?;
        if run >= protocol.warmup {
            prefill.push(ns);
        }
    }
    let mut decode_ns = Vec::with_capacity(protocol.runs);
    let mut reply: Option<Vec<u32>> = None;
    for run in 0..protocol.warmup + protocol.runs {
        let started = Instant::now();
        let written = decode(model, prompt, protocol.decode_tokens)?;
        let ns = elapsed_ns(started)?;
        match &reply {
            Some(first) if *first != written => {
                return Err("decode runs wrote different replies".into())
            }
            Some(_) => {}
            None => reply = Some(written),
        }
        if run >= protocol.warmup {
            decode_ns.push(ns);
        }
    }
    let reply = reply.unwrap_or_default();
    let reply_bytes: Vec<u8> = reply.iter().flat_map(|t| t.to_le_bytes()).collect();
    let prompt_bytes: Vec<u8> = prompt.iter().flat_map(|t| t.to_le_bytes()).collect();
    Ok(json!({
        "schema": "uor-r4.d11-scorer-toks/1",
        "engine": "uor_r4_integer::discrete_d11::IntegerDiscreteModel::evaluate_prefix",
        "artifact_sha256": model.artifact_sha256(),
        "protocol": {"warmup": protocol.warmup, "runs": protocol.runs,
                     "prompt_tokens": protocol.prompt_tokens,
                     "decode_tokens": protocol.decode_tokens,
                     "median": "middle value; mean of the middle two for an even count",
                     "p95": "nearest rank ceil(0.95 n) of the sorted durations"},
        "prompt_sha256": sha256(&prompt_bytes),
        "reply_sha256": sha256(&reply_bytes),
        "reply": reply,
        "prefill": duration_stats(protocol.prompt_tokens, &prefill)?,
        "decode": duration_stats(protocol.decode_tokens, &decode_ns)?,
    }))
}

fn command_text(program: &str, args: &[&str]) -> Value {
    match Command::new(program).args(args).output() {
        Ok(out) if out.status.success() => {
            json!(String::from_utf8_lossy(&out.stdout).trim().to_owned())
        }
        _ => json!("unavailable"),
    }
}

/// The machine's profile, as the protocol lists it: OS version, CPU, cores,
/// memory and power state (macOS tools; "unavailable" elsewhere).
pub fn hardware_profile() -> Value {
    let sysctl = |key: &str| command_text("sysctl", &["-n", key]);
    json!({
        "sw_vers": command_text("sw_vers", &[]),
        "cpu": sysctl("machdep.cpu.brand_string"),
        "physical_cores": sysctl("hw.physicalcpu"),
        "performance_cores": sysctl("hw.perflevel0.physicalcpu"),
        "efficiency_cores": sysctl("hw.perflevel1.physicalcpu"),
        "memory_bytes": sysctl("hw.memsize"),
        "power": command_text("pmset", &["-g", "batt"]),
    })
}
